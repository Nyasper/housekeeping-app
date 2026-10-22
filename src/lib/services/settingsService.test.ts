import { beforeEach, describe, expect, mock, test } from 'bun:test';

import { invokeMock } from '#lib/testing/tauriMocks.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

const { DEFAULT_SETTINGS, SettingsService } = await import('./settingsService.svelte.js');

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
});

describe('SettingsService.load', () => {
  test('loads the settings returned by the backend', async () => {
    invokeMock.mockResolvedValueOnce({ recent_vacated_days: 5, upcoming_occupancy_days: 7 });
    const service = new SettingsService();

    await service.load();

    expect(service.settings).toEqual({ recent_vacated_days: 5, upcoming_occupancy_days: 7 });
    expect(service.loading).toBe(false);
    expect(service.error).toBeNull();
    expect(invokeMock).toHaveBeenCalledWith('settings_get', undefined);
  });

  test('stores the translated error when loading fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'settings_invalid', message: 'Fuera de rango.' });
    const service = new SettingsService();

    await service.load();

    expect(service.error).toBe('Fuera de rango.');
    expect(service.loading).toBe(false);
  });
});

describe('SettingsService.save', () => {
  test('saves the input and updates the settings', async () => {
    const input = { recent_vacated_days: 3, upcoming_occupancy_days: 4 };
    invokeMock.mockResolvedValueOnce(input);
    const service = new SettingsService();

    const saved = await service.save(input);

    expect(saved).toBe(true);
    expect(service.settings).toEqual(input);
    expect(service.saving).toBe(false);
    expect(service.error).toBeNull();
    expect(invokeMock).toHaveBeenCalledWith('settings_update', { input });
  });

  test('returns false and stores the error when saving fails', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'validation', message: 'Datos inválidos' });
    const service = new SettingsService();

    const saved = await service.save({ recent_vacated_days: 31, upcoming_occupancy_days: 2 });

    expect(saved).toBe(false);
    expect(service.error).toBe('Datos inválidos');
    expect(service.saving).toBe(false);
  });
});

describe('defaults', () => {
  test('the service does not mutate the exported defaults', () => {
    const service = new SettingsService();

    service.settings.recent_vacated_days = 99;

    expect(DEFAULT_SETTINGS.recent_vacated_days).toBe(2);
    expect(new SettingsService().settings.recent_vacated_days).toBe(2);
  });
});
