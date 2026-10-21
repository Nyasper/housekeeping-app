/**
 * TypeScript DTOs mirroring the Rust structs. Field names stay in snake_case
 * because that is the shape serialized by the backend.
 */

export type Gender = 'male' | 'female';

export type StaffRole = 'maid' | 'window_cleaner' | 'common_areas';

export type AbsenceType = 'medical_certificate' | 'leave' | 'absence';

export type PlaceType = 'apartment' | 'common_area';

export type PlaceStatus =
  | 'occupied'
  | 'upcoming_occupancy'
  | 'vacant'
  | 'recently_vacated'
  | 'cancelled';

export type ServiceType = 'glass' | 'daily_cleaning' | 'deep_cleaning' | 'touch_up';

export type ServiceStatus = 'planned' | 'in_progress' | 'completed' | 'cancelled';

export type IncidentType = 'damage' | 'lost_item' | 'supply_shortage';

export type IncidentStatus = 'open' | 'resolved';

export interface AppSettings {
  recent_vacated_days: number;
  upcoming_occupancy_days: number;
}

export interface SettingsInput {
  recent_vacated_days: number;
  upcoming_occupancy_days: number;
}

export interface Staff {
  id: number;
  first_name: string;
  last_name: string;
  birth_date: string | null;
  phone: string | null;
  national_id: string;
  address: string | null;
  photo_path: string | null;
  photo_url: string | null;
  gender: Gender;
  role: StaffRole;
  is_seasonal: boolean;
  season_start_date: string | null;
  season_end_date: string | null;
  is_active: boolean;
  notes: string | null;
  created_at: string;
  updated_at: string;
  main_places: PlaceSummary[];
  rest_days: number[];
}

export interface StaffInput {
  first_name: string;
  last_name: string;
  birth_date: string | null;
  phone: string | null;
  national_id: string;
  address: string | null;
  gender: Gender;
  role: StaffRole;
  is_seasonal: boolean;
  season_start_date: string | null;
  season_end_date: string | null;
  notes: string | null;
}

export interface PlaceSummary {
  id: number;
  name: string;
  type: PlaceType;
  location: string | null;
}

export interface Place extends PlaceSummary {
  status: PlaceStatus | null;
  effective_status: PlaceStatus;
  check_in_date: string | null;
  check_out_date: string | null;
  is_active: boolean;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface PlaceInput {
  name: string;
  type: PlaceType;
  status: PlaceStatus | null;
  location: string | null;
  check_in_date: string | null;
  check_out_date: string | null;
  notes: string | null;
}

export interface Absence {
  id: number;
  staff_id: number;
  type: AbsenceType;
  start_date: string;
  end_date: string;
  reason: string | null;
  created_at: string;
}

export interface AbsenceInput {
  staff_id: number;
  type: AbsenceType;
  start_date: string;
  end_date: string;
  reason: string | null;
}

export interface ServiceStaff {
  staff_id: number;
  first_name: string;
  last_name: string;
  photo_url: string | null;
  gender: Gender;
  is_lead: boolean;
}

export interface CleaningService {
  id: number;
  place_id: number;
  place_name: string;
  date: string;
  start_time: string | null;
  end_time: string | null;
  service_type: ServiceType;
  status: ServiceStatus;
  notes: string | null;
  created_at: string;
  updated_at: string;
  staff: ServiceStaff[];
}

export interface CleaningServiceInput {
  place_id: number;
  date: string;
  start_time: string | null;
  end_time: string | null;
  service_type: ServiceType;
  status: ServiceStatus;
  notes: string | null;
  staff: ServiceStaffInput[];
}

export interface ServiceStaffInput {
  staff_id: number;
  is_lead: boolean;
}

export interface RecommendedTask {
  effective_status: PlaceStatus;
  recommended_service_type: ServiceType | null;
}

export interface StaffSuggestion {
  staff: Staff;
  reasons: string[];
}

export interface Incident {
  id: number;
  service_id: number;
  type: IncidentType;
  description: string;
  photo_path: string | null;
  photo_url: string | null;
  status: IncidentStatus;
  date: string;
  resolved_at: string | null;
  created_at: string;
}

export interface IncidentInput {
  service_id: number;
  type: IncidentType;
  description: string;
  date: string;
}

export interface DateRange {
  from: string;
  to: string;
}

export interface ServicesByStaffRow {
  staff_id: number;
  staff_name: string;
  service_count: number;
  total_hours: number;
}

export interface AverageTimesRow {
  service_type: ServiceType;
  average_minutes: number;
  completed_services: number;
}

export interface AbsencesByMonthRow {
  staff_id: number;
  staff_name: string;
  type: AbsenceType;
  days: number;
}
