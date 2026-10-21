import type {
  AbsenceType,
  Gender,
  IncidentStatus,
  IncidentType,
  PlaceStatus,
  PlaceType,
  ServiceStatus,
  ServiceType,
  StaffRole
} from './types.js';

export const genderLabels: Record<Gender, string> = {
  male: 'Hombre',
  female: 'Mujer'
};

export const staffRoleLabels: Record<StaffRole, string> = {
  maid: 'Mucama',
  window_cleaner: 'Vidriero',
  common_areas: 'Áreas comunes'
};

export const absenceTypeLabels: Record<AbsenceType, string> = {
  medical_certificate: 'Certificación médica',
  leave: 'Licencia',
  absence: 'Falta'
};

export const placeTypeLabels: Record<PlaceType, string> = {
  apartment: 'Apartamento',
  common_area: 'Área común'
};

export const placeStatusLabels: Record<PlaceStatus, string> = {
  occupied: 'Ocupado',
  upcoming_occupancy: 'Próxima ocupación',
  vacant: 'Desocupado',
  recently_vacated: 'Desocupado reciente',
  cancelled: 'Cancelado'
};

export const serviceTypeLabels: Record<ServiceType, string> = {
  glass: 'Vidrios',
  daily_cleaning: 'Limpieza diaria',
  deep_cleaning: 'Limpieza general',
  touch_up: 'Repaso'
};

export const serviceStatusLabels: Record<ServiceStatus, string> = {
  planned: 'Planificado',
  in_progress: 'En progreso',
  completed: 'Completado',
  cancelled: 'Cancelado'
};

export const incidentTypeLabels: Record<IncidentType, string> = {
  damage: 'Daño',
  lost_item: 'Objeto olvidado',
  supply_shortage: 'Faltante de insumos'
};

export const incidentStatusLabels: Record<IncidentStatus, string> = {
  open: 'Abierta',
  resolved: 'Resuelta'
};

/** 0 = Monday … 6 = Sunday, matching chrono::Weekday. */
export const weekdayLabels = ['Lunes', 'Martes', 'Miércoles', 'Jueves', 'Viernes', 'Sábado', 'Domingo'] as const;

export type LabelValue = string;

export function toOptions<T extends LabelValue>(labels: Record<T, string>): Array<{ value: T; label: string }> {
  return (Object.entries(labels) as Array<[T, string]>).map(([value, label]) => ({ value, label }));
}

export function labelOf<T extends LabelValue>(
  labels: Record<T, string>,
  value: T | null | undefined,
  fallback = 'Sin datos'
): string {
  if (value === null || value === undefined) {
    return fallback;
  }
  return labels[value] ?? fallback;
}
