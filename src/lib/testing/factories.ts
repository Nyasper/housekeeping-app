import type {
  Absence,
  AbsencesByMonthRow,
  AverageTimesRow,
  CleaningService,
  CleaningServiceInput,
  Incident,
  Place,
  ServiceStaffInput,
  ServicesByStaffRow,
  Staff,
  StaffInput
} from '#lib/api/types.js';

export function makeStaff(overrides: Partial<Staff> = {}): Staff {
  return {
    id: 1,
    first_name: 'Ana',
    last_name: 'Silva',
    birth_date: null,
    phone: null,
    national_id: '12345678',
    address: null,
    photo_path: null,
    photo_url: null,
    gender: 'female',
    role: 'maid',
    is_seasonal: false,
    season_start_date: null,
    season_end_date: null,
    is_active: true,
    notes: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    main_places: [],
    rest_days: [],
    ...overrides
  };
}

export function makeStaffInput(overrides: Partial<StaffInput> = {}): StaffInput {
  return {
    first_name: 'Ana',
    last_name: 'Silva',
    birth_date: null,
    phone: null,
    national_id: '12345678',
    address: null,
    gender: 'female',
    role: 'maid',
    is_seasonal: false,
    season_start_date: null,
    season_end_date: null,
    notes: null,
    ...overrides
  };
}

export function makeAbsence(overrides: Partial<Absence> = {}): Absence {
  return {
    id: 1,
    staff_id: 1,
    type: 'leave',
    start_date: '2026-10-20',
    end_date: '2026-10-22',
    reason: null,
    created_at: '2026-10-01T00:00:00Z',
    ...overrides
  };
}

export function makePlace(overrides: Partial<Place> = {}): Place {
  return {
    id: 1,
    name: 'Apto 101',
    type: 'apartment',
    location: null,
    status: null,
    effective_status: 'vacant',
    recommended_service_type: null,
    check_in_date: null,
    check_out_date: null,
    is_active: true,
    notes: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides
  };
}

export function makeIncident(overrides: Partial<Incident> = {}): Incident {
  return {
    id: 1,
    service_id: 1,
    place_id: 1,
    place_name: 'Apto 101',
    service_type: 'daily_cleaning',
    type: 'damage',
    description: 'Lámpara rota',
    photo_path: null,
    photo_url: null,
    status: 'open',
    date: '2026-10-21',
    resolved_at: null,
    created_at: '2026-10-21T00:00:00Z',
    ...overrides
  };
}

export function makeServiceStaffInput(
  overrides: Partial<ServiceStaffInput> = {}
): ServiceStaffInput {
  return {
    staff_id: 1,
    is_lead: true,
    ...overrides
  };
}

export function makeCleaningService(overrides: Partial<CleaningService> = {}): CleaningService {
  return {
    id: 1,
    place_id: 1,
    place_name: 'Apto 101',
    date: '2026-10-21',
    start_time: null,
    end_time: null,
    service_type: 'daily_cleaning',
    status: 'planned',
    notes: null,
    created_at: '2026-10-21T00:00:00Z',
    updated_at: '2026-10-21T00:00:00Z',
    staff: [],
    ...overrides
  };
}

export function makeCleaningServiceInput(
  overrides: Partial<CleaningServiceInput> = {}
): CleaningServiceInput {
  return {
    place_id: 1,
    date: '2026-10-21',
    start_time: null,
    end_time: null,
    service_type: 'daily_cleaning',
    status: 'planned',
    notes: null,
    staff: [makeServiceStaffInput()],
    ...overrides
  };
}

export function makeServicesByStaffRow(
  overrides: Partial<ServicesByStaffRow> = {}
): ServicesByStaffRow {
  return {
    staff_id: 1,
    staff_name: 'Ana Silva',
    service_count: 1,
    total_hours: 1,
    ...overrides
  };
}

export function makeAverageTimesRow(overrides: Partial<AverageTimesRow> = {}): AverageTimesRow {
  return {
    service_type: 'daily_cleaning',
    average_minutes: 60,
    completed_services: 1,
    ...overrides
  };
}

export function makeAbsencesByMonthRow(
  overrides: Partial<AbsencesByMonthRow> = {}
): AbsencesByMonthRow {
  return {
    staff_id: 1,
    staff_name: 'Ana Silva',
    type: 'leave',
    days: 1,
    ...overrides
  };
}
