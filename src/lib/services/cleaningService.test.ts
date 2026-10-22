import { afterEach, beforeEach, describe, expect, mock, setSystemTime, test } from 'bun:test';

import {
  makeCleaningService,
  makeCleaningServiceInput,
  makeServiceStaffInput
} from '#lib/testing/factories.js';
import { invokeMock } from '#lib/testing/tauriMocks.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

const { CleaningServiceManager } = await import('./cleaningService.svelte.js');

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
});

afterEach(() => {
  setSystemTime();
});

describe('CleaningServiceManager.loadByDate', () => {
  test('defaults to today', async () => {
    setSystemTime(new Date(2026, 9, 21, 12));
    const service = makeCleaningService();
    invokeMock.mockResolvedValueOnce([service]);
    const manager = new CleaningServiceManager();

    await manager.loadByDate();

    expect(manager.items).toEqual([service]);
    expect(invokeMock).toHaveBeenCalledWith('cleaning_list_by_date', { date: '2026-10-21' });
    expect(manager.loading).toBe(false);
  });

  test('stores the translated error when loading fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'database', message: 'Falló la consulta.' });
    const manager = new CleaningServiceManager();

    await manager.loadByDate('2026-10-21');

    expect(manager.error).toBe('Falló la consulta.');
    expect(manager.loading).toBe(false);
  });
});

describe('CleaningServiceManager.create', () => {
  test('creates the service, stores its warning and reloads the date', async () => {
    const input = makeCleaningServiceInput();
    const created = makeCleaningService({ warning: 'Queda sin responsable' });
    invokeMock.mockResolvedValueOnce(created);
    invokeMock.mockResolvedValueOnce([created]);
    const manager = new CleaningServiceManager();

    const result = await manager.create(input);

    expect(result).toEqual(created);
    expect(manager.lastWarning).toBe('Queda sin responsable');
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'cleaning_create', { input });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'cleaning_list_by_date', {
      date: '2026-10-21'
    });
    expect(manager.saving).toBe(false);
  });

  test('returns null and stores the error when creation fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'validation', message: 'Sin personal.' });
    const manager = new CleaningServiceManager();

    const result = await manager.create(makeCleaningServiceInput());

    expect(result).toBeNull();
    expect(manager.error).toBe('Sin personal.');
    expect(manager.saving).toBe(false);
  });
});

describe('CleaningServiceManager.update and setStatus', () => {
  test('passes confirmReopen and stores the warning', async () => {
    const input = makeCleaningServiceInput({ status: 'in_progress' });
    const updated = makeCleaningService({ status: 'in_progress', warning: 'Reabierto' });
    invokeMock.mockResolvedValueOnce(updated);
    invokeMock.mockResolvedValueOnce([updated]);
    const manager = new CleaningServiceManager();

    const result = await manager.update(1, input, true);

    expect(result).toEqual(updated);
    expect(manager.lastWarning).toBe('Reabierto');
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'cleaning_update', {
      id: 1,
      input,
      confirmReopen: true
    });
  });

  test('falls back to the service times when no new times are provided', async () => {
    const service = makeCleaningService({ start_time: '09:00', end_time: '10:00' });
    invokeMock.mockResolvedValueOnce(makeCleaningService({ status: 'completed' }));
    invokeMock.mockResolvedValueOnce([]);
    const manager = new CleaningServiceManager();

    await manager.setStatus(service, 'completed');

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'cleaning_set_status', {
      id: 1,
      status: 'completed',
      startTime: '09:00',
      endTime: '10:00',
      confirmReopen: false
    });
  });

  test('uses provided times over the service times', async () => {
    const service = makeCleaningService({ start_time: '09:00', end_time: '10:00' });
    invokeMock.mockResolvedValueOnce(makeCleaningService({ status: 'completed' }));
    invokeMock.mockResolvedValueOnce([]);
    const manager = new CleaningServiceManager();

    await manager.setStatus(service, 'completed', '11:00', '12:30', true);

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'cleaning_set_status', {
      id: 1,
      status: 'completed',
      startTime: '11:00',
      endTime: '12:30',
      confirmReopen: true
    });
  });
});

describe('CleaningServiceManager.setStaff', () => {
  test('sets the staff, stores the warning and reloads', async () => {
    const service = makeCleaningService();
    const staff = [makeServiceStaffInput({ staff_id: 2 })];
    const updated = makeCleaningService({ warning: 'Advertencia' });
    invokeMock.mockResolvedValueOnce(updated);
    invokeMock.mockResolvedValueOnce([updated]);
    const manager = new CleaningServiceManager();

    const result = await manager.setStaff(service, staff);

    expect(result).toEqual(updated);
    expect(manager.lastWarning).toBe('Advertencia');
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'cleaning_set_staff', { id: 1, staff });
  });
});

describe('CleaningServiceManager.suggestStaff', () => {
  test('maps empty times to null and uses the default count', async () => {
    const suggestion = { staff: {} as never, reasons: [] };
    invokeMock.mockResolvedValueOnce([suggestion]);
    const manager = new CleaningServiceManager();

    const result = await manager.suggestStaff(1, 'daily_cleaning', '2026-10-21', '', '');

    expect(result).toEqual([suggestion]);
    expect(manager.suggestions).toEqual([suggestion]);
    expect(manager.loadingSuggestions).toBe(false);
    expect(invokeMock).toHaveBeenCalledWith('cleaning_suggest_staff', {
      placeId: 1,
      serviceType: 'daily_cleaning',
      date: '2026-10-21',
      count: 10,
      startTime: null,
      endTime: null
    });
  });

  test('passes explicit times and count', async () => {
    invokeMock.mockResolvedValueOnce([]);
    const manager = new CleaningServiceManager();

    await manager.suggestStaff(2, 'deep_cleaning', '2026-10-22', '09:00', '11:00', 3);

    expect(invokeMock).toHaveBeenCalledWith('cleaning_suggest_staff', {
      placeId: 2,
      serviceType: 'deep_cleaning',
      date: '2026-10-22',
      count: 3,
      startTime: '09:00',
      endTime: '11:00'
    });
  });

  test('clears suggestions and stores the error on failure', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'database', message: 'Falló.' });
    const manager = new CleaningServiceManager();
    manager.suggestions = [{ staff: {} as never, reasons: [] }];

    const result = await manager.suggestStaff(1, 'daily_cleaning', '2026-10-21');

    expect(result).toEqual([]);
    expect(manager.suggestions).toEqual([]);
    expect(manager.error).toBe('Falló.');
    expect(manager.loadingSuggestions).toBe(false);
  });
});
