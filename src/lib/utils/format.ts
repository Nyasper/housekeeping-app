const longDateFormatter = new Intl.DateTimeFormat('es-UY', {
  weekday: 'long',
  day: 'numeric',
  month: 'long',
  year: 'numeric'
});

const shortDateFormatter = new Intl.DateTimeFormat('es-UY', {
  day: 'numeric',
  month: 'long',
  year: 'numeric'
});

const monthFormatter = new Intl.DateTimeFormat('es-UY', {
  month: 'long',
  year: 'numeric'
});

const timeFormatter = new Intl.DateTimeFormat('es-UY', {
  hour: '2-digit',
  minute: '2-digit',
  hour12: false
});

const numberFormatter = new Intl.NumberFormat('es-UY');

export function capitalize(value: string): string {
  if (!value) {
    return value;
  }
  return value.charAt(0).toLocaleUpperCase('es-UY') + value.slice(1);
}

/**
 * Parses `YYYY-MM-DD` as a local date. `new Date('YYYY-MM-DD')` would parse it
 * as UTC and can display the previous day in negative time zones.
 */
export function parseIsoDate(value: string): Date | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) {
    return null;
  }
  const year = Number(match[1]);
  const month = Number(match[2]) - 1;
  const day = Number(match[3]);
  const date = new Date(year, month, day);
  // `Date` rolls invalid values over (2026-02-30 becomes 2026-03-02), so the
  // parsed components must survive the round trip.
  if (
    Number.isNaN(date.getTime()) ||
    date.getFullYear() !== year ||
    date.getMonth() !== month ||
    date.getDate() !== day
  ) {
    return null;
  }
  return date;
}

function parseTime(value: string): { hours: number; minutes: number } | null {
  const match = /^(\d{2}):(\d{2})$/.exec(value);
  if (!match) {
    return null;
  }
  const hours = Number(match[1]);
  const minutes = Number(match[2]);
  if (hours > 23 || minutes > 59) {
    return null;
  }
  return { hours, minutes };
}

export function todayIso(now: Date = new Date()): string {
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

export function currentMonthIso(now: Date = new Date()): string {
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  return `${year}-${month}`;
}

export function currentMonthRangeIso(now: Date = new Date()): { from: string; to: string } {
  const year = now.getFullYear();
  const monthIndex = now.getMonth();
  const month = String(monthIndex + 1).padStart(2, '0');
  const lastDay = new Date(year, monthIndex + 1, 0).getDate();
  const lastDayPadded = String(lastDay).padStart(2, '0');
  return {
    from: `${year}-${month}-01`,
    to: `${year}-${month}-${lastDayPadded}`
  };
}

export function nowTimeIso(now: Date = new Date()): string {
  const hours = String(now.getHours()).padStart(2, '0');
  const minutes = String(now.getMinutes()).padStart(2, '0');
  return `${hours}:${minutes}`;
}

export function shiftIsoDate(value: string, deltaDays: number): string {
  const parsed = parseIsoDate(value);
  if (!parsed) {
    return value;
  }
  parsed.setDate(parsed.getDate() + deltaDays);
  return todayIso(parsed);
}

export function durationMinutesBetween(
  startTime: string | null | undefined,
  endTime: string | null | undefined
): number | null {
  if (!startTime || !endTime) {
    return null;
  }
  const start = parseTime(startTime);
  const end = parseTime(endTime);
  if (!start || !end) {
    return null;
  }
  const startMinutes = start.hours * 60 + start.minutes;
  const endMinutes = end.hours * 60 + end.minutes;
  return endMinutes > startMinutes ? endMinutes - startMinutes : null;
}

/** Returns ISO weekday index (`0 = Monday … 6 = Sunday`) for `YYYY-MM-DD`. */
export function weekdayFromIsoDate(value: string): number | null {
  const date = parseIsoDate(value);
  if (!date) {
    return null;
  }
  return (date.getDay() + 6) % 7;
}

export function formatLongDate(value: Date | string = new Date(), fallback = 'Sin datos'): string {
  const date = typeof value === 'string' ? parseIsoDate(value) : value;
  if (!date || Number.isNaN(date.getTime())) {
    return fallback;
  }
  return capitalize(longDateFormatter.format(date));
}

export function formatDate(value: string | null | undefined, fallback = 'Sin datos'): string {
  if (!value) {
    return fallback;
  }
  const date = parseIsoDate(value);
  return date ? shortDateFormatter.format(date) : fallback;
}

/**
 * Parses a SQLite `datetime('now')` timestamp (UTC, `YYYY-MM-DD HH:MM:SS`)
 * and formats its local date. ISO timestamps with an explicit offset are kept.
 */
export function formatUtcDateTimeDate(
  value: string | null | undefined,
  fallback = 'Sin datos'
): string {
  if (!value) {
    return fallback;
  }
  const normalized = value.includes('T') ? value : value.replace(' ', 'T');
  const withZone = /(?:Z|[+-]\d{2}:?\d{2})$/.test(normalized) ? normalized : `${normalized}Z`;
  const date = new Date(withZone);
  if (Number.isNaN(date.getTime())) {
    return fallback;
  }
  return shortDateFormatter.format(date);
}

export function formatMonth(value: string, fallback = 'Sin datos'): string {
  const date = parseIsoDate(`${value}-01`);
  return date ? capitalize(monthFormatter.format(date)) : fallback;
}

export function formatTime(value: string | null | undefined, fallback = 'Sin datos'): string {
  if (!value) {
    return fallback;
  }
  const time = parseTime(value);
  if (!time) {
    return fallback;
  }
  const date = new Date(2000, 0, 1, time.hours, time.minutes);
  return timeFormatter.format(date);
}

export function formatNumber(value: number): string {
  return numberFormatter.format(value);
}

export function formatDurationMinutes(minutes: number): string {
  if (!Number.isFinite(minutes) || minutes < 0) {
    return 'Sin datos';
  }
  // Round the total first so values like 59.6 become 1 h instead of 60 min.
  const totalMinutes = Math.round(minutes);
  const hours = Math.floor(totalMinutes / 60);
  const remaining = totalMinutes % 60;
  if (hours === 0) {
    return `${formatNumber(remaining)} min`;
  }
  if (remaining === 0) {
    return `${formatNumber(hours)} h`;
  }
  return `${formatNumber(hours)} h ${formatNumber(remaining)} min`;
}
