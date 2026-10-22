import { afterEach, beforeEach, describe, expect, mock, test } from 'bun:test';

import { invokeMock } from '#lib/testing/tauriMocks.js';

mock.module('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

const { call, isTauriRuntime } = await import('./invoke.js');
const { AppError } = await import('./errors.js');

const globalScope = globalThis as unknown as { window?: unknown };
const originalWindow = globalScope.window;

function restoreWindow(): void {
  if (originalWindow === undefined) {
    delete globalScope.window;
  } else {
    globalScope.window = originalWindow;
  }
}

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
});

afterEach(() => {
  restoreWindow();
});

describe('isTauriRuntime', () => {
  test('returns false without a window', () => {
    delete globalScope.window;

    expect(isTauriRuntime()).toBe(false);
  });

  test('returns false when the Tauri internals are missing', () => {
    globalScope.window = {};

    expect(isTauriRuntime()).toBe(false);
  });

  test('returns true when the Tauri internals are present', () => {
    globalScope.window = { __TAURI_INTERNALS__: {} };

    expect(isTauriRuntime()).toBe(true);
  });
});

describe('call', () => {
  test('invokes the command and returns its result', async () => {
    invokeMock.mockResolvedValueOnce({ id: 7 });

    const result = await call<{ id: number }>('staff_get', { id: 7 });

    expect(result).toEqual({ id: 7 });
    expect(invokeMock).toHaveBeenCalledWith('staff_get', { id: 7 });
  });

  test('passes undefined arguments through', async () => {
    await call('staff_list');

    expect(invokeMock).toHaveBeenCalledWith('staff_list', undefined);
  });

  test('normalizes a command error payload', async () => {
    invokeMock.mockRejectedValueOnce({ code: 'validation', message: 'Datos inválidos' });

    const error = await call('staff_create', {}).catch((cause: unknown) => cause);

    expect(error).toBeInstanceOf(AppError);
    expect((error as InstanceType<typeof AppError>).code).toBe('validation');
    expect((error as InstanceType<typeof AppError>).message).toBe('Datos inválidos');
  });

  test('normalizes string errors', async () => {
    invokeMock.mockRejectedValueOnce('falló');

    const error = await call('staff_list').catch((cause: unknown) => cause);

    expect(error).toBeInstanceOf(AppError);
    expect((error as InstanceType<typeof AppError>).code).toBe('unexpected');
    expect((error as InstanceType<typeof AppError>).message).toBe('falló');
  });
});
