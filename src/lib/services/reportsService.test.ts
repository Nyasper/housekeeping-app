import { beforeEach, describe, expect, mock, test } from 'bun:test';

import {
  makeAbsencesByMonthRow,
  makeAverageTimesRow,
  makeServicesByStaffRow
} from '#lib/testing/factories.js';
import { invokeMock } from '#lib/testing/tauriMocks.js';
import type { DailySheetRow } from '#lib/api/types.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

const { ReportsService } = await import('./reportsService.svelte.js');

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe('ReportsService derived totals', () => {
  test('sums services and working hours', () => {
    const service = new ReportsService();
    service.servicesByStaff = [
      makeServicesByStaffRow({ service_count: 2, total_hours: 1.25 }),
      makeServicesByStaffRow({ service_count: 3, total_hours: 2.5 })
    ];

    expect(service.totalServicesInRange).toBe(5);
    expect(service.totalHoursInRange).toBe(3.75);
  });

  test('rounds total hours to two decimals', () => {
    const service = new ReportsService();
    service.servicesByStaff = [
      makeServicesByStaffRow({ total_hours: 0.1 }),
      makeServicesByStaffRow({ total_hours: 0.2 })
    ];

    expect(service.totalHoursInRange).toBe(0.3);
  });

  test('sums completed services and absence days', () => {
    const service = new ReportsService();
    service.averageTimes = [
      makeAverageTimesRow({ completed_services: 4 }),
      makeAverageTimesRow({ completed_services: 6 })
    ];
    service.absencesByMonth = [
      makeAbsencesByMonthRow({ days: 3 }),
      makeAbsencesByMonthRow({ days: 5 })
    ];

    expect(service.totalCompletedServices).toBe(10);
    expect(service.totalAbsenceDays).toBe(8);
  });

  test('returns zero for empty collections', () => {
    const service = new ReportsService();

    expect(service.totalServicesInRange).toBe(0);
    expect(service.totalHoursInRange).toBe(0);
    expect(service.totalCompletedServices).toBe(0);
    expect(service.totalAbsenceDays).toBe(0);
  });
});

describe('ReportsService loaders', () => {
  test('loads services by staff for a range', async () => {
    const row = makeServicesByStaffRow();
    invokeMock.mockResolvedValueOnce([row]);
    const service = new ReportsService();

    await service.loadServicesByStaff({ from: '2026-10-01', to: '2026-10-31' });

    expect(service.servicesByStaff).toEqual([row]);
    expect(invokeMock).toHaveBeenCalledWith('reports_services_by_staff', {
      range: { from: '2026-10-01', to: '2026-10-31' }
    });
    expect(service.loading).toBe(false);
  });

  test('loads average times for a range', async () => {
    const row = makeAverageTimesRow();
    invokeMock.mockResolvedValueOnce([row]);
    const service = new ReportsService();

    await service.loadAverageTimes({ from: '2026-10-01', to: '2026-10-31' });

    expect(service.averageTimes).toEqual([row]);
    expect(invokeMock).toHaveBeenCalledWith('reports_average_times', {
      range: { from: '2026-10-01', to: '2026-10-31' }
    });
  });

  test('loadRangeReports loads both reports', async () => {
    const byStaff = [makeServicesByStaffRow()];
    const averages = [makeAverageTimesRow()];
    invokeMock.mockResolvedValueOnce(byStaff);
    invokeMock.mockResolvedValueOnce(averages);
    const service = new ReportsService();

    await service.loadRangeReports({ from: '2026-10-01', to: '2026-10-31' });

    expect(service.servicesByStaff).toEqual(byStaff);
    expect(service.averageTimes).toEqual(averages);
    expect(invokeMock).toHaveBeenCalledTimes(2);
  });

  test('loads absences by month', async () => {
    const row = makeAbsencesByMonthRow();
    invokeMock.mockResolvedValueOnce([row]);
    const service = new ReportsService();

    await service.loadAbsencesByMonth('2026-10');

    expect(service.absencesByMonth).toEqual([row]);
    expect(invokeMock).toHaveBeenCalledWith('reports_absences_by_month', { month: '2026-10' });
  });

  test('loads the daily sheet', async () => {
    const row = {
      service_id: 1,
      place_id: 1,
      place_name: 'Apto 101',
      place_location: null,
      service_type: 'daily_cleaning' as const,
      status: 'completed' as const,
      start_time: '09:00',
      end_time: '10:00',
      duration_minutes: 60,
      staff_names: ['Ana Silva'],
      incidents_count: 0,
      open_incidents_count: 0,
      incidents_summary: [],
      notes: null
    };
    invokeMock.mockResolvedValueOnce([row]);
    const service = new ReportsService();

    await service.loadDailySheet('2026-10-21');

    expect(service.dailySheet).toEqual([row]);
    expect(invokeMock).toHaveBeenCalledWith('reports_daily_sheet', { date: '2026-10-21' });
  });

  test('stores the translated error when a report fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'database', message: 'Falló el reporte.' });
    const service = new ReportsService();

    await service.loadDailySheet('2026-10-21');

    expect(service.error).toBe('Falló el reporte.');
    expect(service.loading).toBe(false);
  });

  test('discards stale responses when the same report is reloaded', async () => {
    const stale = deferred<unknown>();
    const fresh = deferred<unknown>();
    const staleRows: DailySheetRow[] = [{ service_id: 1, place_name: 'Viejo' } as DailySheetRow];
    const freshRows: DailySheetRow[] = [{ service_id: 2, place_name: 'Nuevo' } as DailySheetRow];
    invokeMock.mockReturnValueOnce(stale.promise).mockReturnValueOnce(fresh.promise);
    const service = new ReportsService();

    const firstLoad = service.loadDailySheet('2026-10-21');
    const secondLoad = service.loadDailySheet('2026-10-22');

    fresh.resolve(freshRows);
    await secondLoad;
    stale.resolve(staleRows);
    await firstLoad;

    expect(service.dailySheet).toEqual(freshRows);
    expect(service.loading).toBe(false);
  });

  test('clearing a report invalidates in-flight responses', async () => {
    const pending = deferred<unknown>();
    invokeMock.mockReturnValueOnce(pending.promise);
    const service = new ReportsService();

    const load = service.loadDailySheet('2026-10-21');
    service.clearDailySheet();
    pending.resolve([{ service_id: 1, place_name: 'Viejo' } as DailySheetRow]);
    await load;

    expect(service.dailySheet).toEqual([]);
    expect(service.loading).toBe(false);
  });
});
