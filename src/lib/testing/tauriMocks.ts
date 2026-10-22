import { mock } from 'bun:test';

/** Shared Tauri doubles so every test file sees the same mock instances. */
export const invokeMock = mock(
  async (_command: string, _args?: Record<string, unknown>): Promise<unknown> => undefined
);

export const saveDialogMock = mock(async (_options?: unknown): Promise<string | null> => null);

export const openDialogMock = mock(
  async (_options?: unknown): Promise<string | string[] | null> => null
);
