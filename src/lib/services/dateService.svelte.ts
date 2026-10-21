import { formatLongDate, shiftIsoDate, todayIso } from '#lib/utils/format.js';

export class DateService {
  selectedDate = $state<string>(todayIso());

  readonly isToday = $derived(this.selectedDate === todayIso());
  readonly formattedSelectedDate = $derived(formatLongDate(this.selectedDate));

  setDate(date: string): void {
    if (date.trim()) {
      this.selectedDate = date.trim();
    }
  }

  goToToday(): void {
    this.selectedDate = todayIso();
  }

  previousDay(): void {
    this.selectedDate = shiftIsoDate(this.selectedDate, -1);
  }

  nextDay(): void {
    this.selectedDate = shiftIsoDate(this.selectedDate, 1);
  }
}

export const dateService = new DateService();
