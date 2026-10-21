import { errorMessage } from '#lib/api/errors.js';
import { call } from '#lib/api/invoke.js';
import type {
  AbsencesByMonthRow,
  AverageTimesRow,
  DailySheetRow,
  DateRange,
  ServicesByStaffRow
} from '#lib/api/types.js';

export class ReportsService {
  servicesByStaff = $state<ServicesByStaffRow[]>([]);
  averageTimes = $state<AverageTimesRow[]>([]);
  absencesByMonth = $state<AbsencesByMonthRow[]>([]);
  dailySheet = $state<DailySheetRow[]>([]);

  loading = $state(false);
  error = $state<string | null>(null);

  readonly totalServicesInRange = $derived(
    this.servicesByStaff.reduce((acc, row) => acc + row.service_count, 0)
  );

  readonly totalHoursInRange = $derived(
    Math.round(this.servicesByStaff.reduce((acc, row) => acc + row.total_hours, 0) * 100) / 100
  );

  readonly totalCompletedServices = $derived(
    this.averageTimes.reduce((acc, row) => acc + row.completed_services, 0)
  );

  readonly totalAbsenceDays = $derived(
    this.absencesByMonth.reduce((acc, row) => acc + row.days, 0)
  );

  async loadServicesByStaff(range: DateRange): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.servicesByStaff = await call<ServicesByStaffRow[]>('reports_services_by_staff', {
        range
      });
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async loadAverageTimes(range: DateRange): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.averageTimes = await call<AverageTimesRow[]>('reports_average_times', {
        range
      });
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async loadRangeReports(range: DateRange): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const [byStaff, avgTimes] = await Promise.all([
        call<ServicesByStaffRow[]>('reports_services_by_staff', { range }),
        call<AverageTimesRow[]>('reports_average_times', { range })
      ]);
      this.servicesByStaff = byStaff;
      this.averageTimes = avgTimes;
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async loadAbsencesByMonth(month: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.absencesByMonth = await call<AbsencesByMonthRow[]>('reports_absences_by_month', {
        month
      });
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async loadDailySheet(date: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.dailySheet = await call<DailySheetRow[]>('reports_daily_sheet', {
        date
      });
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }
}

export const reportsService = new ReportsService();
