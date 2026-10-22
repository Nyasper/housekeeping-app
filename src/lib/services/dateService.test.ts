import { afterEach, describe, expect, setSystemTime, test } from 'bun:test';

import { DateService } from './dateService.svelte.js';

afterEach(() => {
  setSystemTime();
});

describe('DateService', () => {
  test('starts on the current date', () => {
    setSystemTime(new Date(2026, 9, 21, 12));
    const service = new DateService();

    expect(service.selectedDate).toBe('2026-10-21');
    expect(service.isToday).toBe(true);
    expect(service.formattedSelectedDate).toBe('Miércoles, 21 de octubre de 2026');
  });

  test('setDate trims the value and ignores blanks', () => {
    setSystemTime(new Date(2026, 9, 21, 12));
    const service = new DateService();

    service.setDate(' 2026-11-05 ');
    expect(service.selectedDate).toBe('2026-11-05');
    expect(service.isToday).toBe(false);

    service.setDate('   ');
    expect(service.selectedDate).toBe('2026-11-05');
  });

  test('previousDay and nextDay move one day at a time', () => {
    setSystemTime(new Date(2026, 9, 21, 12));
    const service = new DateService();

    service.previousDay();
    expect(service.selectedDate).toBe('2026-10-20');

    service.nextDay();
    expect(service.selectedDate).toBe('2026-10-21');
  });

  test('moves across month and year boundaries', () => {
    const service = new DateService();

    service.setDate('2026-12-31');
    service.nextDay();
    expect(service.selectedDate).toBe('2027-01-01');

    service.setDate('2026-01-01');
    service.previousDay();
    expect(service.selectedDate).toBe('2025-12-31');
  });

  test('handles leap days', () => {
    const service = new DateService();

    service.setDate('2028-02-28');
    service.nextDay();
    expect(service.selectedDate).toBe('2028-02-29');

    service.nextDay();
    expect(service.selectedDate).toBe('2028-03-01');
  });

  test('goToToday returns to the current date', () => {
    setSystemTime(new Date(2026, 9, 21, 12));
    const service = new DateService();

    service.setDate('2025-01-01');
    service.goToToday();

    expect(service.selectedDate).toBe('2026-10-21');
    expect(service.isToday).toBe(true);
  });

  test('isToday updates when the selected date changes', () => {
    setSystemTime(new Date(2026, 9, 21, 12));
    const service = new DateService();

    expect(service.isToday).toBe(true);

    service.nextDay();
    expect(service.isToday).toBe(false);

    service.previousDay();
    expect(service.isToday).toBe(true);
  });

  test('formattedSelectedDate follows the selected date', () => {
    const service = new DateService();

    service.setDate('2026-01-01');

    expect(service.formattedSelectedDate).toBe('Jueves, 1 de enero de 2026');
  });
});
