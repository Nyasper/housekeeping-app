import { errorMessage } from '#lib/api/errors.js';
import { call } from '#lib/api/invoke.js';
import type {
  Absence,
  AbsenceInput,
  PlaceSummary,
  Staff,
  StaffInput
} from '#lib/api/types.js';
import { todayIso, weekdayFromIsoDate } from '#lib/utils/format.js';

export class StaffService {
  items = $state<Staff[]>([]);
  absences = $state<Absence[]>([]);
  availableApartments = $state<PlaceSummary[]>([]);
  loading = $state(false);
  saving = $state(false);
  error = $state<string | null>(null);
  lastWarning = $state<string | null>(null);
  private requestToken = 0;

  async load(): Promise<void> {
    const token = ++this.requestToken;
    this.loading = true;
    this.error = null;
    try {
      const [staffList, absenceList, apartments] = await Promise.all([
        call<Staff[]>('staff_list'),
        call<Absence[]>('staff_absences_list', { staffId: null }),
        call<PlaceSummary[]>('staff_available_apartments')
      ]);
      if (token === this.requestToken) {
        this.items = staffList;
        this.absences = absenceList;
        this.availableApartments = apartments;
      }
    } catch (cause) {
      if (token === this.requestToken) {
        this.error = errorMessage(cause);
      }
    } finally {
      if (token === this.requestToken) {
        this.loading = false;
      }
    }
  }

  async create(
    input: StaffInput,
    restDays: number[],
    mainPlaceIds: number[]
  ): Promise<Staff | null> {
    this.saving = true;
    this.error = null;
    try {
      const created = await call<Staff>('staff_create', {
        input,
        restDays,
        mainPlaceIds
      });
      await this.load();
      return this.items.find((item) => item.id === created.id) ?? created;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async update(
    id: number,
    input: StaffInput,
    restDays: number[],
    mainPlaceIds: number[]
  ): Promise<Staff | null> {
    this.saving = true;
    this.error = null;
    try {
      const updated = await call<Staff>('staff_update', {
        id,
        input,
        restDays,
        mainPlaceIds
      });
      await this.load();
      return this.items.find((item) => item.id === id) ?? updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async deactivate(id: number): Promise<Staff | null> {
    this.saving = true;
    this.error = null;
    this.lastWarning = null;
    try {
      const updated = await call<Staff>('staff_deactivate', { id });
      this.lastWarning = updated.warning ?? null;
      await this.load();
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async reactivate(id: number): Promise<Staff | null> {
    this.saving = true;
    this.error = null;
    try {
      const updated = await call<Staff>('staff_reactivate', { id });
      await this.load();
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async createAbsence(input: AbsenceInput): Promise<Absence | null> {
    this.saving = true;
    this.error = null;
    try {
      const created = await call<Absence>('staff_absences_create', { input });
      this.absences = await call<Absence[]>('staff_absences_list', { staffId: null });
      return created;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async updateAbsence(id: number, input: AbsenceInput): Promise<Absence | null> {
    this.saving = true;
    this.error = null;
    try {
      const updated = await call<Absence>('staff_absences_update', { id, input });
      this.absences = await call<Absence[]>('staff_absences_list', { staffId: null });
      return updated;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.saving = false;
    }
  }

  async deleteAbsence(id: number): Promise<boolean> {
    this.saving = true;
    this.error = null;
    try {
      await call<void>('staff_absences_delete', { id });
      this.absences = await call<Absence[]>('staff_absences_list', { staffId: null });
      return true;
    } catch (cause) {
      this.error = errorMessage(cause);
      return false;
    } finally {
      this.saving = false;
    }
  }

  absencesFor(staffId: number): Absence[] {
    return this.absences.filter((absence) => absence.staff_id === staffId);
  }

  isWithinSeasonOn(staff: Staff, date: string = todayIso()): boolean {
    if (!staff.is_seasonal) {
      return true;
    }
    if (!staff.season_start_date || !staff.season_end_date) {
      return true;
    }
    return date >= staff.season_start_date && date <= staff.season_end_date;
  }

  isAvailableOn(staff: Staff, date: string = todayIso()): boolean {
    if (!staff.is_active) {
      return false;
    }
    if (!this.isWithinSeasonOn(staff, date)) {
      return false;
    }
    const weekday = weekdayFromIsoDate(date);
    if (weekday !== null && staff.rest_days.includes(weekday)) {
      return false;
    }
    const hasAbsence = this.absences.some(
      (absence) =>
        absence.staff_id === staff.id &&
        absence.start_date <= date &&
        absence.end_date >= date
    );
    return !hasAbsence;
  }

  availableOn(date: string = todayIso()): Staff[] {
    return this.items.filter((staff) => this.isAvailableOn(staff, date));
  }
}

export const staffService = new StaffService();
