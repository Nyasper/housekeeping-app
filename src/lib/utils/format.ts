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
  return Number.isNaN(date.getTime()) ? null : date;
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

export function formatMonth(value: string, fallback = 'Sin datos'): string {
  const date = parseIsoDate(`${value}-01`);
  return date ? capitalize(monthFormatter.format(date)) : fallback;
}

export function formatTime(value: string | null | undefined, fallback = 'Sin datos'): string {
  if (!value) {
    return fallback;
  }
  const match = /^(\d{2}):(\d{2})$/.exec(value);
  if (!match) {
    return fallback;
  }
  const date = new Date(2000, 0, 1, Number(match[1]), Number(match[2]));
  return timeFormatter.format(date);
}

export function formatNumber(value: number): string {
  return numberFormatter.format(value);
}

export function formatDurationMinutes(minutes: number): string {
  if (!Number.isFinite(minutes) || minutes < 0) {
    return 'Sin datos';
  }
  const hours = Math.floor(minutes / 60);
  const remaining = Math.round(minutes % 60);
  if (hours === 0) {
    return `${formatNumber(remaining)} min`;
  }
  if (remaining === 0) {
    return `${formatNumber(hours)} h`;
  }
  return `${formatNumber(hours)} h ${formatNumber(remaining)} min`;
}
