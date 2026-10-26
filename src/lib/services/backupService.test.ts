import { afterEach, beforeEach, describe, expect, mock, test } from 'bun:test';

import { invokeMock, openDialogMock, saveDialogMock } from '#lib/testing/tauriMocks.js';
import type { BackupManifest } from '#lib/api/types.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));
mock.module('@tauri-apps/plugin-dialog', () => ({
  open: openDialogMock,
  save: saveDialogMock
}));

const { BackupService } = await import('./backupService.svelte.js');

const globalScope = globalThis as unknown as { window?: unknown };
const originalWindow = globalScope.window;

const manifest: BackupManifest = {
  app_version: '0.1.0',
  schema_version: 2,
  format_version: 1,
  created_at: '2026-10-22T00:00:00Z',
  counts: { staff: 1, places: 1, cleaning_services: 0, incidents: 0 }
};

function enableTauri(): void {
  globalScope.window = { __TAURI_INTERNALS__: {} };
}

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
  saveDialogMock.mockReset();
  saveDialogMock.mockResolvedValue(null);
  openDialogMock.mockReset();
  openDialogMock.mockResolvedValue(null);
});

afterEach(() => {
  if (originalWindow === undefined) {
    delete globalScope.window;
  } else {
    globalScope.window = originalWindow;
  }
});

describe('BackupService.createBackup', () => {
  test('rejects backup creation outside the Tauri runtime', async () => {
    delete globalScope.window;
    const service = new BackupService();

    const result = await service.createBackup();

    expect(result).toBeNull();
    expect(service.error).toBe(
      'La creación de respaldos solo está disponible en la aplicación de escritorio.'
    );
    expect(saveDialogMock).not.toHaveBeenCalled();
  });

  test('does nothing when the save dialog is cancelled', async () => {
    enableTauri();
    const service = new BackupService();

    const result = await service.createBackup();

    expect(result).toBeNull();
    expect(invokeMock).not.toHaveBeenCalled();
    expect(service.error).toBeNull();
  });

  test('creates the backup and stores the saved path', async () => {
    enableTauri();
    saveDialogMock.mockResolvedValueOnce('backup.zip');
    invokeMock.mockResolvedValueOnce('saved.zip');
    const service = new BackupService();

    const result = await service.createBackup();

    expect(result).toBe('saved.zip');
    expect(service.lastBackupPath).toBe('saved.zip');
    expect(invokeMock).toHaveBeenCalledWith('backup_create', { destPath: 'backup.zip' });
    expect(service.creating).toBe(false);
  });

  test('stores dialog and backend errors', async () => {
    enableTauri();
    saveDialogMock.mockRejectedValueOnce({ code: 'dialog', message: 'No se pudo guardar.' });
    const service = new BackupService();

    expect(await service.createBackup()).toBeNull();
    expect(service.error).toBe('No se pudo guardar.');

    saveDialogMock.mockResolvedValueOnce('backup.zip');
    invokeMock.mockRejectedValueOnce({ code: 'backup_failed', message: 'Falló el respaldo.' });
    expect(await service.createBackup()).toBeNull();
    expect(service.error).toBe('Falló el respaldo.');
  });
});

describe('BackupService.pickRestoreFile', () => {
  test('rejects picking a file outside the Tauri runtime', async () => {
    delete globalScope.window;
    const service = new BackupService();

    expect(await service.pickRestoreFile()).toBeNull();
    expect(service.error).toBe(
      'La restauración de respaldos solo está disponible en la aplicación de escritorio.'
    );
  });

  test('returns the selected file', async () => {
    enableTauri();
    openDialogMock.mockResolvedValueOnce('backup.zip');
    const service = new BackupService();

    expect(await service.pickRestoreFile()).toBe('backup.zip');
    expect(openDialogMock).toHaveBeenCalled();
  });

  test('returns null for cancelled or empty selections', async () => {
    enableTauri();
    const service = new BackupService();

    openDialogMock.mockResolvedValueOnce(null);
    expect(await service.pickRestoreFile()).toBeNull();

    openDialogMock.mockResolvedValueOnce('');
    expect(await service.pickRestoreFile()).toBeNull();

    openDialogMock.mockResolvedValueOnce([]);
    expect(await service.pickRestoreFile()).toBeNull();
  });

  test('stores dialog errors', async () => {
    enableTauri();
    openDialogMock.mockRejectedValueOnce({ code: 'dialog', message: 'No se pudo abrir.' });
    const service = new BackupService();

    expect(await service.pickRestoreFile()).toBeNull();
    expect(service.error).toBe('No se pudo abrir.');
  });
});

describe('BackupService.restoreBackup', () => {
  test('restores the backup and refreshes the cached services', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      switch (command) {
        case 'backup_restore':
          return manifest;
        case 'settings_get':
          return { recent_vacated_days: 2, upcoming_occupancy_days: 2 };
        case 'staff_list':
        case 'staff_absences_list':
        case 'staff_available_apartments':
        case 'places_list':
        case 'cleaning_list_by_date':
        case 'incidents_list':
          return [];
        default:
          return undefined;
      }
    });
    const service = new BackupService();

    const result = await service.restoreBackup('backup.zip');

    expect(result).toEqual(manifest);
    expect(service.lastRestoreManifest).toEqual(manifest);
    expect(invokeMock).toHaveBeenCalledWith('backup_restore', { srcPath: 'backup.zip' });
    expect(invokeMock).toHaveBeenCalledWith('settings_get', undefined);
    expect(invokeMock).toHaveBeenCalledWith('staff_list', undefined);
    expect(invokeMock).toHaveBeenCalledWith('incidents_list', { filters: {} });
    expect(service.restoring).toBe(false);
  });

  test('keeps the restore successful but reports reload failures', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      switch (command) {
        case 'backup_restore':
          return manifest;
        case 'settings_get':
          throw { code: 'database', message: 'No se pudo leer la configuración.' };
        default:
          return [];
      }
    });
    const service = new BackupService();

    const result = await service.restoreBackup('backup.zip');

    expect(result).toEqual(manifest);
    expect(service.lastRestoreManifest).toEqual(manifest);
    expect(service.error).toBe(
      'El respaldo se restauró, pero no se pudieron recargar todos los datos: No se pudo leer la configuración.'
    );
    expect(service.restoring).toBe(false);
  });

  test('stores the translated error when restoring fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'invalid_backup', message: 'Respaldo inválido.' });
    const service = new BackupService();

    const result = await service.restoreBackup('backup.zip');

    expect(result).toBeNull();
    expect(service.error).toBe('Respaldo inválido.');
    expect(service.restoring).toBe(false);
  });
});

describe('BackupService state helpers', () => {
  test('busy is true while creating or restoring', () => {
    const service = new BackupService();

    expect(service.busy).toBe(false);
    service.creating = true;
    expect(service.busy).toBe(true);
    service.creating = false;
    service.restoring = true;
    expect(service.busy).toBe(true);
  });

  test('clearError resets the message', () => {
    const service = new BackupService();
    service.error = 'Error';

    service.clearError();

    expect(service.error).toBeNull();
  });
});
