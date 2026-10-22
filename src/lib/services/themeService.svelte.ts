export type Theme = 'light' | 'dark';

const STORAGE_KEY = 'housekeeping-theme';

export class ThemeService {
  theme = $state<Theme>('light');

  #initialized = false;

  /** Applies the persisted theme. Safe to call once from the root layout. */
  init(): void {
    if (this.#initialized || typeof window === 'undefined') {
      return;
    }
    this.#initialized = true;

    let stored: string | null = null;
    try {
      stored = window.localStorage.getItem(STORAGE_KEY);
    } catch {
      stored = null;
    }

    this.theme = stored === 'dark' ? 'dark' : 'light';
    this.apply();
  }

  toggle(): void {
    this.set(this.theme === 'dark' ? 'light' : 'dark');
  }

  set(theme: Theme): void {
    this.theme = theme;
    this.apply();
  }

  private apply(): void {
    if (typeof document === 'undefined') {
      return;
    }

    document.documentElement.dataset.theme = this.theme;

    try {
      window.localStorage.setItem(STORAGE_KEY, this.theme);
    } catch {
      // Storage can be unavailable; the theme still applies for this session.
    }
  }
}

export const themeService = new ThemeService();
