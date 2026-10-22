import { beforeEach, describe, expect, mock, setSystemTime, afterEach, test } from 'bun:test';

import { makePlace } from '#lib/testing/factories.js';
import { invokeMock } from '#lib/testing/tauriMocks.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

const { PlacesService } = await import('./placesService.svelte.js');

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
});

afterEach(() => {
  setSystemTime();
});

describe('PlacesService.load', () => {
  test('stores the reference date and passes it to the backend', async () => {
    const place = makePlace();
    invokeMock.mockResolvedValueOnce([place]);
    const service = new PlacesService();

    await service.load('2026-10-21');

    expect(service.referenceDate).toBe('2026-10-21');
    expect(service.items).toEqual([place]);
    expect(service.loading).toBe(false);
    expect(invokeMock).toHaveBeenCalledWith('places_list', { date: '2026-10-21' });
  });

  test('reuses the stored reference date when no date is given', async () => {
    const service = new PlacesService();
    invokeMock.mockResolvedValueOnce([]);
    await service.load('2026-10-21');

    invokeMock.mockResolvedValueOnce([]);
    await service.load();

    expect(invokeMock).toHaveBeenLastCalledWith('places_list', { date: '2026-10-21' });
  });

  test('sends null when there is no reference date', async () => {
    invokeMock.mockResolvedValueOnce([]);
    const service = new PlacesService();

    await service.load();

    expect(invokeMock).toHaveBeenCalledWith('places_list', { date: null });
  });

  test('stores the translated error when loading fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'database', message: 'Falló la consulta.' });
    const service = new PlacesService();

    await service.load('2026-10-21');

    expect(service.error).toBe('Falló la consulta.');
    expect(service.loading).toBe(false);
  });
});

describe('PlacesService mutations', () => {
  test('creates a place and reloads the list', async () => {
    const place = makePlace();
    invokeMock.mockResolvedValueOnce(place);
    invokeMock.mockResolvedValueOnce([place]);
    const service = new PlacesService();

    const created = await service.create({
      name: 'Apto 101',
      type: 'apartment',
      status: null,
      location: null,
      check_in_date: null,
      check_out_date: null,
      notes: null
    });

    expect(created).toEqual(place);
    expect(service.items).toEqual([place]);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'places_create', {
      input: expect.objectContaining({ name: 'Apto 101' })
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'places_list', { date: null });
    expect(service.saving).toBe(false);
  });

  test('updates a place and reloads the list', async () => {
    const place = makePlace({ name: 'Apto 202' });
    invokeMock.mockResolvedValueOnce(place);
    invokeMock.mockResolvedValueOnce([place]);
    const service = new PlacesService();

    const updated = await service.update(1, {
      name: 'Apto 202',
      type: 'apartment',
      status: null,
      location: null,
      check_in_date: null,
      check_out_date: null,
      notes: null
    });

    expect(updated).toEqual(place);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'places_update', {
      id: 1,
      input: expect.objectContaining({ name: 'Apto 202' })
    });
  });

  test('setStatusOverride maps the place fields into the update input', async () => {
    const place = makePlace({
      name: 'Apto 303',
      type: 'apartment',
      location: 'Piso 3',
      check_in_date: '2026-10-19',
      check_out_date: '2026-10-25',
      notes: 'Vista al mar'
    });
    invokeMock.mockResolvedValueOnce(place);
    invokeMock.mockResolvedValueOnce([place]);
    const service = new PlacesService();

    await service.setStatusOverride(place, 'cancelled');

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'places_update', {
      id: 1,
      input: {
        name: 'Apto 303',
        type: 'apartment',
        status: 'cancelled',
        location: 'Piso 3',
        check_in_date: '2026-10-19',
        check_out_date: '2026-10-25',
        notes: 'Vista al mar'
      }
    });
  });

  test('deactivates and reactivates a place', async () => {
    invokeMock.mockResolvedValue(makePlace());
    const service = new PlacesService();

    await service.deactivate(1);
    expect(invokeMock).toHaveBeenCalledWith('places_deactivate', { id: 1 });

    await service.reactivate(1);
    expect(invokeMock).toHaveBeenCalledWith('places_reactivate', { id: 1 });
  });

  test('returns the recommended task and defaults to today', async () => {
    setSystemTime(new Date(2026, 9, 21, 12));
    const task = {
      effective_status: 'occupied' as const,
      recommended_service_type: 'daily_cleaning' as const
    };
    invokeMock.mockResolvedValueOnce(task);
    const service = new PlacesService();

    const result = await service.recommendedTask(1);

    expect(result).toEqual(task);
    expect(invokeMock).toHaveBeenCalledWith('places_recommended_task', {
      placeId: 1,
      date: '2026-10-21'
    });
  });

  test('stores errors and returns null when a mutation fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'duplicate', message: 'Ya existe.' });
    const service = new PlacesService();

    const created = await service.create({
      name: 'Apto 101',
      type: 'apartment',
      status: null,
      location: null,
      check_in_date: null,
      check_out_date: null,
      notes: null
    });

    expect(created).toBeNull();
    expect(service.error).toBe('Ya existe.');
    expect(service.saving).toBe(false);
  });
});
