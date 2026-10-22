import { afterEach, describe, expect, setSystemTime, test } from 'bun:test';

import {
  capitalize,
  currentMonthIso,
  currentMonthRangeIso,
  durationMinutesBetween,
  formatDate,
  formatDurationMinutes,
  formatLongDate,
  formatMonth,
  formatNumber,
  formatTime,
  nowTimeIso,
  parseIsoDate,
  shiftIsoDate,
  todayIso,
  weekdayFromIsoDate
} from './format.js';

afterEach(() => {
  setSystemTime();
});

describe('parseIsoDate', () => {
  test('parses a valid ISO date as a local date', () => {
    const date = parseIsoDate('2026-10-21');

    expect(date).not.toBeNull();
    expect(date?.getFullYear()).toBe(2026);
    expect(date?.getMonth()).toBe(9);
    expect(date?.getDate()).toBe(21);
  });

  test('accepts a leap day in a leap year', () => {
    expect(parseIsoDate('2028-02-29')).not.toBeNull();
  });

  test('rejects malformed formats', () => {
    expect(parseIsoDate('21/10/2026')).toBeNull();
    expect(parseIsoDate('2026-1-1')).toBeNull();
    expect(parseIsoDate('')).toBeNull();
  });

  test('rejects dates that do not exist instead of rolling them over', () => {
    expect(parseIsoDate('2026-02-29')).toBeNull();
    expect(parseIsoDate('2026-02-30')).toBeNull();
    expect(parseIsoDate('2026-04-31')).toBeNull();
    expect(parseIsoDate('2026-00-10')).toBeNull();
    expect(parseIsoDate('2026-13-01')).toBeNull();
  });
});

describe('date helpers', () => {
  test('todayIso pads month and day', () => {
    expect(todayIso(new Date(2026, 0, 5))).toBe('2026-01-05');
    expect(todayIso(new Date(2026, 11, 31))).toBe('2026-12-31');
  });

  test('todayIso uses the current date by default', () => {
    setSystemTime(new Date(2026, 9, 22, 12));

    expect(todayIso()).toBe('2026-10-22');
  });

  test('currentMonthIso pads the month', () => {
    expect(currentMonthIso(new Date(2026, 0, 5))).toBe('2026-01');
    expect(currentMonthIso(new Date(2026, 11, 31))).toBe('2026-12');
  });

  test('currentMonthRangeIso returns the first and last day of the month', () => {
    expect(currentMonthRangeIso(new Date(2026, 1, 10))).toEqual({
      from: '2026-02-01',
      to: '2026-02-28'
    });
    expect(currentMonthRangeIso(new Date(2028, 1, 10))).toEqual({
      from: '2028-02-01',
      to: '2028-02-29'
    });
    expect(currentMonthRangeIso(new Date(2026, 3, 10))).toEqual({
      from: '2026-04-01',
      to: '2026-04-30'
    });
    expect(currentMonthRangeIso(new Date(2026, 11, 10))).toEqual({
      from: '2026-12-01',
      to: '2026-12-31'
    });
  });

  test('nowTimeIso pads hours and minutes', () => {
    expect(nowTimeIso(new Date(2026, 9, 22, 0, 0))).toBe('00:00');
    expect(nowTimeIso(new Date(2026, 9, 22, 9, 5))).toBe('09:05');
    expect(nowTimeIso(new Date(2026, 9, 22, 23, 59))).toBe('23:59');
  });

  test('shiftIsoDate moves across month and year boundaries', () => {
    expect(shiftIsoDate('2026-10-21', 1)).toBe('2026-10-22');
    expect(shiftIsoDate('2026-12-31', 1)).toBe('2027-01-01');
    expect(shiftIsoDate('2026-01-01', -1)).toBe('2025-12-31');
  });

  test('shiftIsoDate handles leap days', () => {
    expect(shiftIsoDate('2028-02-28', 1)).toBe('2028-02-29');
    expect(shiftIsoDate('2028-02-29', 1)).toBe('2028-03-01');
    expect(shiftIsoDate('2026-02-28', 1)).toBe('2026-03-01');
  });

  test('shiftIsoDate leaves invalid values untouched', () => {
    expect(shiftIsoDate('not-a-date', 1)).toBe('not-a-date');
    expect(shiftIsoDate('2026-02-30', -1)).toBe('2026-02-30');
  });

  test('weekdayFromIsoDate returns ISO indexes from Monday to Sunday', () => {
    expect(weekdayFromIsoDate('2026-10-19')).toBe(0);
    expect(weekdayFromIsoDate('2026-10-21')).toBe(2);
    expect(weekdayFromIsoDate('2026-10-25')).toBe(6);
    expect(weekdayFromIsoDate('2026-10-32')).toBeNull();
  });
});

describe('durationMinutesBetween', () => {
  test('computes whole minutes between two valid times', () => {
    expect(durationMinutesBetween('09:00', '10:30')).toBe(90);
    expect(durationMinutesBetween('00:00', '23:59')).toBe(1439);
  });

  test('returns null when an endpoint is missing', () => {
    expect(durationMinutesBetween(null, '10:00')).toBeNull();
    expect(durationMinutesBetween('10:00', undefined)).toBeNull();
    expect(durationMinutesBetween('', '10:00')).toBeNull();
  });

  test('returns null for invalid or non-positive ranges', () => {
    expect(durationMinutesBetween('9:00', '10:00')).toBeNull();
    expect(durationMinutesBetween('09:00', '9:00')).toBeNull();
    expect(durationMinutesBetween('09:00', '09:00')).toBeNull();
    expect(durationMinutesBetween('10:00', '09:00')).toBeNull();
    expect(durationMinutesBetween('24:00', '25:00')).toBeNull();
    expect(durationMinutesBetween('09:60', '10:00')).toBeNull();
    expect(durationMinutesBetween('99:99', '10:00')).toBeNull();
  });
});

describe('formatters', () => {
  test('capitalize only changes the first character', () => {
    expect(capitalize('octubre')).toBe('Octubre');
    expect(capitalize('')).toBe('');
  });

  test('formatLongDate formats strings and dates in es-UY', () => {
    expect(formatLongDate('2026-10-21')).toBe('Miércoles, 21 de octubre de 2026');
    expect(formatLongDate(new Date(2026, 9, 21))).toBe('Miércoles, 21 de octubre de 2026');
  });

  test('formatLongDate falls back for invalid values', () => {
    expect(formatLongDate('2026-02-30')).toBe('Sin datos');
    expect(formatLongDate('invalid', 'Sin fecha')).toBe('Sin fecha');
    expect(formatLongDate(new Date(Number.NaN))).toBe('Sin datos');
  });

  test('formatDate handles valid and missing values', () => {
    expect(formatDate('2026-10-21')).toBe('21 de octubre de 2026');
    expect(formatDate(null)).toBe('Sin datos');
    expect(formatDate(undefined)).toBe('Sin datos');
    expect(formatDate('')).toBe('Sin datos');
    expect(formatDate('2026-02-30', 'Sin fecha')).toBe('Sin fecha');
  });

  test('formatMonth handles valid and invalid months', () => {
    expect(formatMonth('2026-10')).toBe('Octubre de 2026');
    expect(formatMonth('2026-13')).toBe('Sin datos');
    expect(formatMonth('2026-1')).toBe('Sin datos');
    expect(formatMonth('', 'Sin mes')).toBe('Sin mes');
  });

  test('formatTime handles valid and invalid times', () => {
    expect(formatTime('09:05')).toBe('09:05');
    expect(formatTime('23:59')).toBe('23:59');
    expect(formatTime(null)).toBe('Sin datos');
    expect(formatTime(undefined, 'Sin hora')).toBe('Sin hora');
    expect(formatTime('9:05')).toBe('Sin datos');
    expect(formatTime('24:00')).toBe('Sin datos');
    expect(formatTime('09:60')).toBe('Sin datos');
    expect(formatTime('99:99')).toBe('Sin datos');
  });

  test('formatNumber uses the es-UY separators', () => {
    expect(formatNumber(0)).toBe('0');
    expect(formatNumber(1234.5)).toBe('1.234,5');
    expect(formatNumber(-1234.5)).toBe('-1.234,5');
  });

  test('formatDurationMinutes formats hours and minutes', () => {
    expect(formatDurationMinutes(0)).toBe('0 min');
    expect(formatDurationMinutes(45)).toBe('45 min');
    expect(formatDurationMinutes(60)).toBe('1 h');
    expect(formatDurationMinutes(90)).toBe('1 h 30 min');
    expect(formatDurationMinutes(125)).toBe('2 h 5 min');
  });

  test('formatDurationMinutes rounds the total before splitting hours', () => {
    expect(formatDurationMinutes(59.4)).toBe('59 min');
    expect(formatDurationMinutes(59.6)).toBe('1 h');
    expect(formatDurationMinutes(119.6)).toBe('2 h');
  });

  test('formatDurationMinutes rejects invalid values', () => {
    expect(formatDurationMinutes(-1)).toBe('Sin datos');
    expect(formatDurationMinutes(Number.NaN)).toBe('Sin datos');
    expect(formatDurationMinutes(Number.POSITIVE_INFINITY)).toBe('Sin datos');
  });
});
