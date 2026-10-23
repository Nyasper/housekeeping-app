import { open as openFileDialog, save as saveFileDialog } from '@tauri-apps/plugin-dialog';

import { errorMessage } from '#lib/api/errors.js';
import { call, isTauriRuntime } from '#lib/api/invoke.js';
import type { BackupManifest } from '#lib/api/types.js';
import { cleaningService } from '#lib/services/cleaningService.svelte.js';
import { dateService } from '#lib/services/dateService.svelte.js';
import { incidentsService } from '#lib/services/incidentsService.svelte.js';
import { placesService } from '#lib/services/placesService.svelte.js';
import { settingsService } from '#lib/services/settingsService.svelte.js';
import { staffService } from '#lib/services/staffService.svelte.js';
import { todayIso } from '#lib/utils/format.js';

export class BackupService {
  creating = $state(false);
  restoring = $state(false);
  error = $state<string | null>(null);
  lastBackupPath = $state<string | null>(null);
  lastRestoreManifest = $state<BackupManifest | null>(null);

  busy = $derived(this.creating || this.restoring);

  clearError(): void {
    this.error = null;
  }

  async createBackup(): Promise<string | null> {
    if (!isTauriRuntime()) {
      this.error = 'La creación de respaldos solo está disponible en la aplicación de escritorio.';
      return null;
    }

    this.error = null;
    const defaultPath = `respaldo-limpieza-${todayIso()}.zip`;

    let destPath: string | null = null;
    try {
      destPath = await saveFileDialog({
        title: 'Guardar respaldo completo (ZIP)',
        defaultPath,
        filters: [
          {
            name: 'Respaldo ZIP (*.zip)',
            extensions: ['zip']
          }
        ]
      });
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    }

    if (!destPath) {
      return null;
    }

    this.creating = true;
    try {
      const saved = await call<string>('backup_create', {
        destPath
      });
      this.lastBackupPath = saved;
      return saved;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.creating = false;
    }
  }

  async pickRestoreFile(): Promise<string | null> {
    if (!isTauriRuntime()) {
      this.error =
        'La restauración de respaldos solo está disponible en la aplicación de escritorio.';
      return null;
    }

    this.error = null;
    try {
      const selected = await openFileDialog({
        title: 'Seleccionar respaldo ZIP para restaurar',
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Respaldo ZIP (*.zip)',
            extensions: ['zip']
          }
        ]
      });

      if (typeof selected === 'string' && selected.trim()) {
        return selected;
      }
      return null;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    }
  }

  async restoreBackup(srcPath: string): Promise<BackupManifest | null> {
    this.restoring = true;
    this.error = null;
    try {
      const manifest = await call<BackupManifest>('backup_restore', {
        srcPath
      });
      this.lastRestoreManifest = manifest;

      // Refresh cached frontend services after pool swap
      await Promise.all([
        settingsService.load(),
        staffService.load(),
        placesService.load(dateService.selectedDate),
        cleaningService.loadByDate(dateService.selectedDate),
        incidentsService.load()
      ]);

      // The reload helpers swallow their own errors; surface them as a warning
      // without turning a successful restore into a failure.
      const reloadError =
        settingsService.error ??
        staffService.error ??
        placesService.error ??
        cleaningService.error ??
        incidentsService.error;
      if (reloadError) {
        this.error = `El respaldo se restauró, pero no se pudieron recargar todos los datos: ${reloadError}`;
      }

      return manifest;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.restoring = false;
    }
  }
}

export const backupService = new BackupService();
