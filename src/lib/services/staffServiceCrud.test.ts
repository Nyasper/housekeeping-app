import { beforeEach, describe, expect, mock, test } from 'bun:test';

import {
  makeAbsence,
  makeStaff,
  makeStaffInput
} from '#lib/testing/factories.js';
import { invokeMock } from '#lib/testing/tauriMocks.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

const { StaffService } = await import('./staffService.svelte.js');

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
});

function mockLoad(staff: ReturnType<typeof makeStaff>[], absences = []) {
  invokeMock.mockResolvedValueOnce(staff); // staff_list
  invokeMock.mockResolvedValueOnce(absences); // staff_absences_list
  invokeMock.mockResolvedValueOnce([]); // staff_available_apartments
}

describe('StaffService.create', () => {
  test('creates staff without optional relations and reloads', async () => {
    const created = makeStaff();
    invokeMock.mockResolvedValueOnce(created);
    mockLoad([created]);
    const service = new StaffService();

    const result = await service.create(makeStaffInput(), [], []);

    expect(result).toEqual(created);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'staff_create', {
      input: expect.objectContaining({ national_id: '12345678' }),
      restDays: [],
      mainPlaceIds: []
    });
    expect(invokeMock).not.toHaveBeenCalledWith('staff_rest_days_set', expect.anything());
    expect(invokeMock).not.toHaveBeenCalledWith('staff_places_set', expect.anything());
    expect(service.items).toEqual([created]);
    expect(service.saving).toBe(false);
  });

  test('sets rest days and main places when provided', async () => {
    const created = makeStaff({ id: 7 });
    const updated = makeStaff({ id: 7, rest_days: [0, 6] });
    invokeMock.mockResolvedValueOnce(created);
    mockLoad([updated]);
    const service = new StaffService();

    const result = await service.create(makeStaffInput(), [0, 6], [1]);

    expect(result).toEqual(updated);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'staff_create', {
      input: expect.objectContaining({ national_id: '12345678' }),
      restDays: [0, 6],
      mainPlaceIds: [1]
    });
    expect(invokeMock).not.toHaveBeenCalledWith('staff_rest_days_set', expect.anything());
    expect(invokeMock).not.toHaveBeenCalledWith('staff_places_set', expect.anything());
  });

  test('returns null and stores the error when creation fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'duplicate', message: 'Cédula duplicada.' });
    const service = new StaffService();

    const result = await service.create(makeStaffInput(), [], []);

    expect(result).toBeNull();
    expect(service.error).toBe('Cédula duplicada.');
    expect(service.saving).toBe(false);
  });
});

describe('StaffService.update', () => {
  test('updates staff and replaces rest days and places', async () => {
    const updated = makeStaff({ first_name: 'Carla' });
    invokeMock.mockResolvedValueOnce(updated);
    mockLoad([updated]);
    const service = new StaffService();

    const result = await service.update(1, makeStaffInput({ first_name: 'Carla' }), [2], [3]);

    expect(result).toEqual(updated);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'staff_update', {
      id: 1,
      input: expect.objectContaining({ first_name: 'Carla' }),
      restDays: [2],
      mainPlaceIds: [3]
    });
    expect(invokeMock).not.toHaveBeenCalledWith('staff_rest_days_set', expect.anything());
    expect(invokeMock).not.toHaveBeenCalledWith('staff_places_set', expect.anything());
  });
});

describe('StaffService status changes', () => {
  test('deactivate stores the warning returned by the backend', async () => {
    const deactivated = makeStaff({ is_active: false, warning: 'Tiene servicios futuros.' });
    invokeMock.mockResolvedValueOnce(deactivated);
    mockLoad([]);
    const service = new StaffService();

    const result = await service.deactivate(1);

    expect(result).toEqual(deactivated);
    expect(service.lastWarning).toBe('Tiene servicios futuros.');
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'staff_deactivate', { id: 1 });
  });

  test('reactivate updates the staff member', async () => {
    const reactivated = makeStaff({ is_active: true });
    invokeMock.mockResolvedValueOnce(reactivated);
    mockLoad([reactivated]);
    const service = new StaffService();

    const result = await service.reactivate(1);

    expect(result).toEqual(reactivated);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'staff_reactivate', { id: 1 });
  });

  test('deactivate failures return null and store the error', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'blocked', message: 'No se puede dar de baja.' });
    const service = new StaffService();

    const result = await service.deactivate(1);

    expect(result).toBeNull();
    expect(service.error).toBe('No se puede dar de baja.');
  });
});

describe('StaffService absences', () => {
  test('creates an absence and refreshes the absence list', async () => {
    const absence = makeAbsence();
    const input = {
      staff_id: 1,
      type: 'leave' as const,
      start_date: '2026-10-20',
      end_date: '2026-10-22',
      reason: null
    };
    invokeMock.mockResolvedValueOnce(absence);
    invokeMock.mockResolvedValueOnce([absence]);
    const service = new StaffService();

    const result = await service.createAbsence(input);

    expect(result).toEqual(absence);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'staff_absences_create', { input });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'staff_absences_list', { staffId: null });
    expect(service.absences).toEqual([absence]);
  });

  test('updates an absence and refreshes the absence list', async () => {
    const absence = makeAbsence({ end_date: '2026-10-23' });
    invokeMock.mockResolvedValueOnce(absence);
    invokeMock.mockResolvedValueOnce([absence]);
    const service = new StaffService();

    const result = await service.updateAbsence(1, {
      staff_id: 1,
      type: 'leave',
      start_date: '2026-10-20',
      end_date: '2026-10-23',
      reason: null
    });

    expect(result).toEqual(absence);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'staff_absences_update', {
      id: 1,
      input: expect.objectContaining({ end_date: '2026-10-23' })
    });
  });

  test('deletes an absence and refreshes the absence list', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    invokeMock.mockResolvedValueOnce([]);
    const service = new StaffService();

    const result = await service.deleteAbsence(1);

    expect(result).toBe(true);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'staff_absences_delete', { id: 1 });
    expect(service.absences).toEqual([]);
  });

  test('delete failures return false and store the error', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'not_found', message: 'No existe.' });
    const service = new StaffService();

    const result = await service.deleteAbsence(1);

    expect(result).toBe(false);
    expect(service.error).toBe('No existe.');
    expect(service.saving).toBe(false);
  });
});
