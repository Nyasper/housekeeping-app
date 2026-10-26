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
  private servicesToken = 0;
  private averageTimesToken = 0;
  private absencesToken = 0;
  private dailySheetToken = 0;
  private pendingRequests = 0;

  private beginRequest(): void {
    this.pendingRequests += 1;
    this.loading = true;
    this.error = null;
  }

  private endRequest(): void {
    this.pendingRequests = Math.max(0, this.pendingRequests - 1);
    this.loading = this.pendingRequests > 0;
  }

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
    const token = ++this.servicesToken;
    this.beginRequest();
    try {
      const rows = await call<ServicesByStaffRow[]>('reports_services_by_staff', { range });
      if (token === this.servicesToken) {
        this.servicesByStaff = rows;
      }
    } catch (cause) {
      if (token === this.servicesToken) {
        this.error = errorMessage(cause);
      }
    } finally {
      this.endRequest();
    }
  }

  async loadAverageTimes(range: DateRange): Promise<void> {
    const token = ++this.averageTimesToken;
    this.beginRequest();
    try {
      const rows = await call<AverageTimesRow[]>('reports_average_times', { range });
      if (token === this.averageTimesToken) {
        this.averageTimes = rows;
      }
    } catch (cause) {
      if (token === this.averageTimesToken) {
        this.error = errorMessage(cause);
      }
    } finally {
      this.endRequest();
    }
  }

  async loadRangeReports(range: DateRange): Promise<void> {
    await Promise.all([this.loadServicesByStaff(range), this.loadAverageTimes(range)]);
  }

  async loadAbsencesByMonth(month: string): Promise<void> {
    const token = ++this.absencesToken;
    this.beginRequest();
    try {
      const rows = await call<AbsencesByMonthRow[]>('reports_absences_by_month', { month });
      if (token === this.absencesToken) {
        this.absencesByMonth = rows;
      }
    } catch (cause) {
      if (token === this.absencesToken) {
        this.error = errorMessage(cause);
      }
    } finally {
      this.endRequest();
    }
  }

  async loadDailySheet(date: string): Promise<void> {
    const token = ++this.dailySheetToken;
    this.beginRequest();
    try {
      const rows = await call<DailySheetRow[]>('reports_daily_sheet', { date });
      if (token === this.dailySheetToken) {
        this.dailySheet = rows;
      }
    } catch (cause) {
      if (token === this.dailySheetToken) {
        this.error = errorMessage(cause);
      }
    } finally {
      this.endRequest();
    }
  }

  clearRangeReports(): void {
    this.servicesToken += 1;
    this.averageTimesToken += 1;
    this.servicesByStaff = [];
    this.averageTimes = [];
    this.error = null;
  }

  clearAbsencesByMonth(): void {
    this.absencesToken += 1;
    this.absencesByMonth = [];
    this.error = null;
  }

  clearDailySheet(): void {
    this.dailySheetToken += 1;
    this.dailySheet = [];
    this.error = null;
  }
}

export const reportsService = new ReportsService();
