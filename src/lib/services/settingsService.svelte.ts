import { errorMessage } from '#lib/api/errors.js';
import { call } from '#lib/api/invoke.js';
import type { AppSettings, SettingsInput } from '#lib/api/types.js';

export const DEFAULT_SETTINGS: AppSettings = {
  recent_vacated_days: 2,
  upcoming_occupancy_days: 2
};

export class SettingsService {
  settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });
  loading = $state(false);
  saving = $state(false);
  error = $state<string | null>(null);

  async load(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.settings = await call<AppSettings>('settings_get');
    } catch (cause) {
      this.error = errorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async save(input: SettingsInput): Promise<boolean> {
    this.saving = true;
    this.error = null;
    try {
      this.settings = await call<AppSettings>('settings_update', { input });
      return true;
    } catch (cause) {
      this.error = errorMessage(cause);
      return false;
    } finally {
      this.saving = false;
    }
  }
}

export const settingsService = new SettingsService();
