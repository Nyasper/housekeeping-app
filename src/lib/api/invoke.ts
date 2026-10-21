import { invoke } from '@tauri-apps/api/core';

import { toAppError } from './errors.js';

/** True when the page is running inside the Tauri WebView. */
export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Typed wrapper around Tauri's `invoke` that normalizes errors to `AppError`.
 * Command argument names use Tauri's default camelCase convention.
 */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (cause) {
    throw toAppError(cause);
  }
}
