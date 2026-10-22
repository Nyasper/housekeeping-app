import { describe, expect, test } from 'bun:test';

import { makeAbsence, makeStaff } from '#lib/testing/factories.js';
import { StaffService } from './staffService.svelte.js';

describe('isWithinSeasonOn', () => {
  test('non-seasonal staff are always within season', () => {
    const service = new StaffService();
    const staff = makeStaff({ is_seasonal: false });

    expect(service.isWithinSeasonOn(staff, '1999-01-01')).toBe(true);
  });

  test('seasonal staff without dates are treated as within season', () => {
    const service = new StaffService();
    const staff = makeStaff({
      is_seasonal: true,
      season_start_date: null,
      season_end_date: null
    });

    expect(service.isWithinSeasonOn(staff, '2026-10-21')).toBe(true);
  });

  test('seasonal boundaries are inclusive', () => {
    const service = new StaffService();
    const staff = makeStaff({
      is_seasonal: true,
      season_start_date: '2026-12-01',
      season_end_date: '2027-03-01'
    });

    expect(service.isWithinSeasonOn(staff, '2026-12-01')).toBe(true);
    expect(service.isWithinSeasonOn(staff, '2027-03-01')).toBe(true);
    expect(service.isWithinSeasonOn(staff, '2026-11-30')).toBe(false);
    expect(service.isWithinSeasonOn(staff, '2027-03-02')).toBe(false);
  });
});

describe('isAvailableOn', () => {
  test('inactive staff are never available', () => {
    const service = new StaffService();
    const staff = makeStaff({ is_active: false });

    expect(service.isAvailableOn(staff, '2026-10-21')).toBe(false);
  });

  test('seasonal staff are unavailable outside the season', () => {
    const service = new StaffService();
    const staff = makeStaff({
      is_seasonal: true,
      season_start_date: '2026-12-01',
      season_end_date: '2027-03-01'
    });

    expect(service.isAvailableOn(staff, '2026-10-21')).toBe(false);
    expect(service.isAvailableOn(staff, '2027-01-15')).toBe(true);
  });

  test('rest days block availability by ISO weekday', () => {
    const service = new StaffService();
    const staff = makeStaff({ rest_days: [2] }); // Wednesday

    expect(service.isAvailableOn(staff, '2026-10-21')).toBe(false);
    expect(service.isAvailableOn(staff, '2026-10-22')).toBe(true);
  });

  test('absences block availability including both boundaries', () => {
    const service = new StaffService();
    service.absences = [
      makeAbsence({ staff_id: 1, start_date: '2026-10-20', end_date: '2026-10-22' })
    ];
    const staff = makeStaff({ id: 1 });

    expect(service.isAvailableOn(staff, '2026-10-19')).toBe(true);
    expect(service.isAvailableOn(staff, '2026-10-20')).toBe(false);
    expect(service.isAvailableOn(staff, '2026-10-21')).toBe(false);
    expect(service.isAvailableOn(staff, '2026-10-22')).toBe(false);
    expect(service.isAvailableOn(staff, '2026-10-23')).toBe(true);
  });

  test('absences from another staff member do not affect availability', () => {
    const service = new StaffService();
    service.absences = [
      makeAbsence({ staff_id: 2, start_date: '2026-10-20', end_date: '2026-10-22' })
    ];

    expect(service.isAvailableOn(makeStaff({ id: 1 }), '2026-10-21')).toBe(true);
  });
});

describe('collection helpers', () => {
  test('absencesFor filters by staff id', () => {
    const service = new StaffService();
    service.absences = [
      makeAbsence({ id: 1, staff_id: 1 }),
      makeAbsence({ id: 2, staff_id: 2 }),
      makeAbsence({ id: 3, staff_id: 1 })
    ];

    expect(service.absencesFor(1).map((absence) => absence.id)).toEqual([1, 3]);
    expect(service.absencesFor(99)).toEqual([]);
  });

  test('availableOn filters active, seasonal, rest day and absence rules', () => {
    const service = new StaffService();
    const available = makeStaff({ id: 1 });
    const inactive = makeStaff({ id: 2, is_active: false });
    const resting = makeStaff({ id: 3, rest_days: [2] });
    const absent = makeStaff({ id: 4 });
    const seasonalOutside = makeStaff({
      id: 5,
      is_seasonal: true,
      season_start_date: '2026-12-01',
      season_end_date: '2027-03-01'
    });

    service.items = [available, inactive, resting, absent, seasonalOutside];
    service.absences = [
      makeAbsence({ staff_id: 4, start_date: '2026-10-20', end_date: '2026-10-22' })
    ];

    expect(service.availableOn('2026-10-21').map((staff) => staff.id)).toEqual([1]);
  });
});
