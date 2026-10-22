import { afterEach, beforeEach, describe, expect, mock, setSystemTime, test } from 'bun:test';

import {
  invokeMock,
  openDialogMock,
  saveDialogMock
} from '#lib/testing/tauriMocks.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));
mock.module('@tauri-apps/plugin-dialog', () => ({
  open: openDialogMock,
  save: saveDialogMock
}));

const { ExportService } = await import('./exportService.svelte.js');

const globalScope = globalThis as unknown as { window?: unknown };
const originalWindow = globalScope.window;

function enableTauri(): void {
  globalScope.window = { __TAURI_INTERNALS__: {} };
}

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
  saveDialogMock.mockReset();
  saveDialogMock.mockResolvedValue(null);
});

afterEach(() => {
  setSystemTime();
  if (originalWindow === undefined) {
    delete globalScope.window;
  } else {
    globalScope.window = originalWindow;
  }
});

describe('suggestedFileName', () => {
  const service = new ExportService();

  test('uses the filter date for the daily sheet', () => {
    expect(service.suggestedFileName('daily_sheet', { date: '2026-10-21' }, 'csv')).toBe(
      'planilla-diaria-2026-10-21.csv'
    );
  });

  test('falls back to today when the daily sheet has no date', () => {
    setSystemTime(new Date(2026, 9, 22, 12));

    expect(service.suggestedFileName('daily_sheet', {}, 'pdf')).toBe(
      'planilla-diaria-2026-10-22.pdf'
    );
  });

  test('uses the full range for range reports', () => {
    const filters = { from: '2026-10-01', to: '2026-10-31' };

    expect(service.suggestedFileName('services_by_staff', filters, 'xlsx')).toBe(
      'servicios-por-persona-2026-10-01-a-2026-10-31.xlsx'
    );
    expect(service.suggestedFileName('average_times', filters, 'csv')).toBe(
      'tiempos-promedio-2026-10-01-a-2026-10-31.csv'
    );
  });

  test('fills missing range endpoints with today', () => {
    setSystemTime(new Date(2026, 9, 22, 12));

    expect(service.suggestedFileName('services_by_staff', { from: '2026-10-01' }, 'csv')).toBe(
      'servicios-por-persona-2026-10-01-a-2026-10-22.csv'
    );
  });

  test('uses the month for absences and falls back to the current month', () => {
    expect(service.suggestedFileName('absences_by_month', { month: '2026-09' }, 'pdf')).toBe(
      'ausencias-2026-09.pdf'
    );

    setSystemTime(new Date(2026, 9, 22, 12));
    expect(service.suggestedFileName('absences_by_month', {}, 'pdf')).toBe(
      'ausencias-2026-10.pdf'
    );
  });
});

describe('exportReport', () => {
  test('rejects exports outside the Tauri runtime', async () => {
    delete globalScope.window;
    const service = new ExportService();

    const result = await service.exportReport('daily_sheet', 'csv', { date: '2026-10-21' });

    expect(result).toBeNull();
    expect(service.error).toBe(
      'La exportación de archivos solo está disponible en la aplicación de escritorio.'
    );
    expect(saveDialogMock).not.toHaveBeenCalled();
  });

  test('does nothing when the save dialog is cancelled', async () => {
    enableTauri();
    const service = new ExportService();

    const result = await service.exportReport('daily_sheet', 'csv', { date: '2026-10-21' });

    expect(result).toBeNull();
    expect(saveDialogMock).toHaveBeenCalled();
    expect(invokeMock).not.toHaveBeenCalled();
    expect(service.error).toBeNull();
  });

  test('exports CSV with the expected command and arguments', async () => {
    enableTauri();
    saveDialogMock.mockResolvedValueOnce('out.csv');
    invokeMock.mockResolvedValueOnce('saved.csv');
    const service = new ExportService();

    const result = await service.exportReport('daily_sheet', 'csv', { date: '2026-10-21' });

    expect(result).toBe('saved.csv');
    expect(invokeMock).toHaveBeenCalledWith('export_csv', {
      kind: 'daily_sheet',
      filters: { date: '2026-10-21' },
      destPath: 'out.csv'
    });
    expect(service.lastExportedPath).toBe('saved.csv');
    expect(service.exporting).toBe(false);
    expect(service.activeFormat).toBeNull();
  });

  test('maps xlsx and pdf formats to their commands', async () => {
    enableTauri();
    saveDialogMock.mockResolvedValueOnce('out.xlsx');
    invokeMock.mockResolvedValueOnce('saved.xlsx');
    const service = new ExportService();

    await service.exportReport('average_times', 'xlsx', { from: '2026-10-01', to: '2026-10-31' });
    expect(invokeMock).toHaveBeenLastCalledWith(
      'export_xlsx',
      expect.objectContaining({ destPath: 'out.xlsx' })
    );

    saveDialogMock.mockResolvedValueOnce('out.pdf');
    invokeMock.mockResolvedValueOnce('saved.pdf');
    await service.exportReport('absences_by_month', 'pdf', { month: '2026-10' });
    expect(invokeMock).toHaveBeenLastCalledWith(
      'export_pdf',
      expect.objectContaining({ destPath: 'out.pdf' })
    );
  });

  test('reports dialog errors', async () => {
    enableTauri();
    saveDialogMock.mockRejectedValueOnce({ code: 'dialog', message: 'No se pudo abrir.' });
    const service = new ExportService();

    const result = await service.exportReport('daily_sheet', 'csv', { date: '2026-10-21' });

    expect(result).toBeNull();
    expect(service.error).toBe('No se pudo abrir.');
  });

  test('reports export errors and clears the active format', async () => {
    enableTauri();
    saveDialogMock.mockResolvedValueOnce('out.csv');
    invokeMock.mockRejectedValueOnce({ code: 'export_failed', message: 'Falló la exportación.' });
    const service = new ExportService();

    const result = await service.exportReport('daily_sheet', 'csv', { date: '2026-10-21' });

    expect(result).toBeNull();
    expect(service.error).toBe('Falló la exportación.');
    expect(service.exporting).toBe(false);
    expect(service.activeFormat).toBeNull();
  });

  test('clearError resets the message', async () => {
    delete globalScope.window;
    const service = new ExportService();
    await service.exportReport('daily_sheet', 'csv', { date: '2026-10-21' });
    expect(service.error).not.toBeNull();

    service.clearError();

    expect(service.error).toBeNull();
  });
});
