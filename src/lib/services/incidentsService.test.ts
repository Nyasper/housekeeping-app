import { beforeEach, describe, expect, mock, test } from 'bun:test';

import { makeIncident } from '#lib/testing/factories.js';
import { invokeMock } from '#lib/testing/tauriMocks.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

const { IncidentsService } = await import('./incidentsService.svelte.js');

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
});

describe('IncidentsService.openItems', () => {
  test('keeps only open incidents', () => {
    const service = new IncidentsService();
    service.items = [
      makeIncident({ id: 1, status: 'open' }),
      makeIncident({ id: 2, status: 'resolved' }),
      makeIncident({ id: 3, status: 'open' })
    ];

    expect(service.openItems.map((incident) => incident.id)).toEqual([1, 3]);
  });

  test('returns an empty list when there are no incidents', () => {
    const service = new IncidentsService();

    expect(service.openItems).toEqual([]);
  });
});

describe('IncidentsService.load', () => {
  test('stores the filters and sends them to the backend', async () => {
    const incident = makeIncident();
    invokeMock.mockResolvedValueOnce([incident]);
    const service = new IncidentsService();

    await service.load({ status: 'open', date: '2026-10-21' });

    expect(service.items).toEqual([incident]);
    expect(service.lastFilters).toEqual({ status: 'open', date: '2026-10-21' });
    expect(invokeMock).toHaveBeenCalledWith('incidents_list', {
      filters: { status: 'open', date: '2026-10-21' }
    });
    expect(service.loading).toBe(false);
  });

  test('reuses the last filters when none are given', async () => {
    invokeMock.mockResolvedValue([]);
    const service = new IncidentsService();
    await service.load({ status: 'resolved' });

    await service.load();

    expect(invokeMock).toHaveBeenLastCalledWith('incidents_list', {
      filters: { status: 'resolved' }
    });
  });

  test('stores the translated error when loading fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'database', message: 'Falló.' });
    const service = new IncidentsService();

    await service.load();

    expect(service.error).toBe('Falló.');
    expect(service.loading).toBe(false);
  });
});

describe('IncidentsService mutations', () => {
  test('creates an incident and reloads the list', async () => {
    const incident = makeIncident();
    const input = {
      service_id: 1,
      type: 'damage' as const,
      description: 'Lámpara rota',
      date: '2026-10-21'
    };
    invokeMock.mockResolvedValueOnce(incident);
    invokeMock.mockResolvedValueOnce([incident]);
    const service = new IncidentsService();

    const result = await service.create(input);

    expect(result).toEqual(incident);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'incidents_create', { input });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'incidents_list', {
      filters: {}
    });
    expect(service.saving).toBe(false);
  });

  test('sets the status and reloads the list', async () => {
    const incident = makeIncident({ status: 'resolved' });
    invokeMock.mockResolvedValueOnce(incident);
    invokeMock.mockResolvedValueOnce([incident]);
    const service = new IncidentsService();

    const result = await service.setStatus(1, 'resolved');

    expect(result).toEqual(incident);
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'incidents_set_status', {
      id: 1,
      status: 'resolved'
    });
  });

  test('returns null and stores the error when a mutation fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'validation', message: 'Servicio cancelado.' });
    const service = new IncidentsService();

    const result = await service.setStatus(1, 'resolved');

    expect(result).toBeNull();
    expect(service.error).toBe('Servicio cancelado.');
    expect(service.saving).toBe(false);
  });
});
