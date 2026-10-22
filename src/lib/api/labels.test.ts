import { describe, expect, test } from 'bun:test';

import {
  absenceTypeLabels,
  genderLabels,
  incidentStatusLabels,
  incidentTypeLabels,
  labelOf,
  placeStatusLabels,
  placeTypeLabels,
  serviceStatusLabels,
  serviceTypeLabels,
  staffRoleLabels,
  toOptions,
  weekdayLabels
} from './labels.js';
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

describe('labelOf', () => {
  test('returns the Spanish label for a known value', () => {
    expect(labelOf(genderLabels, 'female')).toBe('Mujer');
    expect(labelOf(serviceStatusLabels, 'in_progress')).toBe('En progreso');
  });

  test('returns the fallback for null, undefined or unknown values', () => {
    expect(labelOf(genderLabels, null)).toBe('Sin datos');
    expect(labelOf(genderLabels, undefined)).toBe('Sin datos');
    expect(labelOf(genderLabels, 'unknown' as Gender)).toBe('Sin datos');
    expect(labelOf(genderLabels, null, 'Sin género')).toBe('Sin género');
  });
});

describe('toOptions', () => {
  test('maps labels to option objects preserving order', () => {
    expect(toOptions(genderLabels)).toEqual([
      { value: 'male', label: 'Hombre' },
      { value: 'female', label: 'Mujer' }
    ]);
  });

  test('works with an empty record', () => {
    expect(toOptions({} as Record<string, string>)).toEqual([]);
  });
});

describe('weekdayLabels', () => {
  test('starts on Monday and has seven entries', () => {
    expect(weekdayLabels).toHaveLength(7);
    expect(weekdayLabels[0]).toBe('Lunes');
    expect(weekdayLabels[6]).toBe('Domingo');
  });
});

describe('label completeness', () => {
  test('every enum value has a label', () => {
    const genders: Gender[] = ['male', 'female'];
    const roles: StaffRole[] = ['maid', 'window_cleaner', 'common_areas'];
    const absenceTypes: AbsenceType[] = ['medical_certificate', 'leave', 'absence'];
    const placeTypes: PlaceType[] = ['apartment', 'common_area'];
    const placeStatuses: PlaceStatus[] = [
      'occupied',
      'upcoming_occupancy',
      'vacant',
      'recently_vacated',
      'cancelled'
    ];
    const serviceTypes: ServiceType[] = ['glass', 'daily_cleaning', 'deep_cleaning', 'touch_up'];
    const serviceStatuses: ServiceStatus[] = ['planned', 'in_progress', 'completed', 'cancelled'];
    const incidentTypes: IncidentType[] = ['damage', 'lost_item', 'supply_shortage'];
    const incidentStatuses: IncidentStatus[] = ['open', 'resolved'];

    for (const value of genders) expect(genderLabels[value]).toBeTruthy();
    for (const value of roles) expect(staffRoleLabels[value]).toBeTruthy();
    for (const value of absenceTypes) expect(absenceTypeLabels[value]).toBeTruthy();
    for (const value of placeTypes) expect(placeTypeLabels[value]).toBeTruthy();
    for (const value of placeStatuses) expect(placeStatusLabels[value]).toBeTruthy();
    for (const value of serviceTypes) expect(serviceTypeLabels[value]).toBeTruthy();
    for (const value of serviceStatuses) expect(serviceStatusLabels[value]).toBeTruthy();
    for (const value of incidentTypes) expect(incidentTypeLabels[value]).toBeTruthy();
    for (const value of incidentStatuses) expect(incidentStatusLabels[value]).toBeTruthy();
  });
});
