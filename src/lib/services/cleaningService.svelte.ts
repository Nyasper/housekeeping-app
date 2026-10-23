import { errorMessage } from '#lib/api/errors.js';
import { call } from '#lib/api/invoke.js';
import type {
  CleaningService,
  CleaningServiceInput,
  ServiceStaffInput,
  ServiceStatus,
  ServiceType,
  StaffSuggestion
} from '#lib/api/types.js';
import { todayIso } from '#lib/utils/format.js';

export class CleaningServiceManager {
  items = $state<CleaningService[]>([]);
  suggestions = $state<StaffSuggestion[]>([]);
  loading = $state(false);
  loadingSuggestions = $state(false);
  saving = $state(false);
  error = $state<string | null>(null);
  lastWarning = $state<string | null>(null);

  async loadByDate(date: string = todayIso()): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.items = await call<CleaningService[]>('cleaning_list_by_date', { date });
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async create(input: CleaningServiceInput): Promise<CleaningService | null> {
    this.saving = true;
    this.error = null;
    this.lastWarning = null;
    try {
      const created = await call<CleaningService>('cleaning_create', { input });
      this.lastWarning = created.warning ?? null;
      await this.loadByDate(input.date);
      return created;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async update(
    id: number,
    input: CleaningServiceInput,
    confirmReopen = false
  ): Promise<CleaningService | null> {
    this.saving = true;
    this.error = null;
    this.lastWarning = null;
    try {
      const updated = await call<CleaningService>('cleaning_update', {
        id,
        input,
        confirmReopen
      });
      this.lastWarning = updated.warning ?? null;
      await this.loadByDate(input.date);
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async setStatus(
    service: CleaningService,
    status: ServiceStatus,
    startTime?: string | null,
    endTime?: string | null,
    confirmReopen = false
  ): Promise<CleaningService | null> {
    this.saving = true;
    this.error = null;
    try {
      const updated = await call<CleaningService>('cleaning_set_status', {
        id: service.id,
        status,
        startTime: startTime ?? service.start_time,
        endTime: endTime ?? service.end_time,
        confirmReopen
      });
      await this.loadByDate(service.date);
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async setStaff(
    service: CleaningService,
    staff: ServiceStaffInput[],
    confirmReopen = false
  ): Promise<CleaningService | null> {
    this.saving = true;
    this.error = null;
    this.lastWarning = null;
    try {
      const updated = await call<CleaningService>('cleaning_set_staff', {
        id: service.id,
        staff,
        confirmReopen
      });
      this.lastWarning = updated.warning ?? null;
      await this.loadByDate(service.date);
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async suggestStaff(
    placeId: number,
    serviceType: ServiceType,
    date: string,
    startTime?: string | null,
    endTime?: string | null,
    count = 10
  ): Promise<StaffSuggestion[]> {
    this.loadingSuggestions = true;
    try {
      const list = await call<StaffSuggestion[]>('cleaning_suggest_staff', {
        placeId,
        serviceType,
        date,
        count,
        startTime: startTime || null,
        endTime: endTime || null
      });
      this.suggestions = list;
      return list;
    } catch (cause) {
      this.error = errorMessage(cause);
      this.suggestions = [];
      return [];
    } finally {
      this.loadingSuggestions = false;
    }
  }
}

export const cleaningService = new CleaningServiceManager();
