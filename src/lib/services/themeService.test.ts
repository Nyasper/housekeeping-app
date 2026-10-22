import { afterEach, describe, expect, test } from 'bun:test';

import { ThemeService } from './themeService.svelte.js';

const globalScope = globalThis as unknown as { window?: unknown; document?: unknown };
const originalWindow = globalScope.window;
const originalDocument = globalScope.document;

interface FakeStorage {
  data: Map<string, string>;
  storage: {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
  };
}

function createStorage(
  initial: Record<string, string> = {},
  options: { throwOnGet?: boolean; throwOnSet?: boolean } = {}
): FakeStorage {
  const data = new Map(Object.entries(initial));
  return {
    data,
    storage: {
      getItem(key: string): string | null {
        if (options.throwOnGet) {
          throw new Error('storage unavailable');
        }
        return data.get(key) ?? null;
      },
      setItem(key: string, value: string): void {
        if (options.throwOnSet) {
          throw new Error('storage unavailable');
        }
        data.set(key, value);
      },
      removeItem(key: string): void {
        data.delete(key);
      }
    }
  };
}

function installDom(storage: FakeStorage['storage']): Record<string, string> {
  const dataset: Record<string, string> = {};
  globalScope.window = { localStorage: storage };
  globalScope.document = { documentElement: { dataset } };
  return dataset;
}

function restoreDom(): void {
  if (originalWindow === undefined) {
    delete globalScope.window;
  } else {
    globalScope.window = originalWindow;
  }
  if (originalDocument === undefined) {
    delete globalScope.document;
  } else {
    globalScope.document = originalDocument;
  }
}

afterEach(() => {
  restoreDom();
});

describe('ThemeService', () => {
  test('defaults to light when there is nothing stored', () => {
    const { storage } = createStorage();
    const dataset = installDom(storage);
    const service = new ThemeService();

    service.init();

    expect(service.theme).toBe('light');
    expect(dataset.theme).toBe('light');
  });

  test('reads the persisted dark theme', () => {
    const { storage } = createStorage({ 'housekeeping-theme': 'dark' });
    const dataset = installDom(storage);
    const service = new ThemeService();

    service.init();

    expect(service.theme).toBe('dark');
    expect(dataset.theme).toBe('dark');
  });

  test('ignores invalid stored values', () => {
    const { storage } = createStorage({ 'housekeeping-theme': 'sepia' });
    const dataset = installDom(storage);
    const service = new ThemeService();

    service.init();

    expect(service.theme).toBe('light');
    expect(dataset.theme).toBe('light');
  });

  test('toggle switches and persists the theme', () => {
    const { data, storage } = createStorage();
    const dataset = installDom(storage);
    const service = new ThemeService();
    service.init();

    service.toggle();
    expect(service.theme).toBe('dark');
    expect(dataset.theme).toBe('dark');
    expect(data.get('housekeeping-theme')).toBe('dark');

    service.toggle();
    expect(service.theme).toBe('light');
    expect(dataset.theme).toBe('light');
    expect(data.get('housekeeping-theme')).toBe('light');
  });

  test('set applies and persists an explicit theme', () => {
    const { data, storage } = createStorage();
    const dataset = installDom(storage);
    const service = new ThemeService();

    service.set('dark');

    expect(service.theme).toBe('dark');
    expect(dataset.theme).toBe('dark');
    expect(data.get('housekeeping-theme')).toBe('dark');
  });

  test('init only runs once', () => {
    const { data, storage } = createStorage({ 'housekeeping-theme': 'light' });
    installDom(storage);
    const service = new ThemeService();

    service.init();
    data.set('housekeeping-theme', 'dark');
    service.init();

    expect(service.theme).toBe('light');
  });

  test('survives storage read failures', () => {
    const { storage } = createStorage({}, { throwOnGet: true });
    const dataset = installDom(storage);
    const service = new ThemeService();

    service.init();

    expect(service.theme).toBe('light');
    expect(dataset.theme).toBe('light');
  });

  test('survives storage write failures', () => {
    const { storage } = createStorage({}, { throwOnSet: true });
    const dataset = installDom(storage);
    const service = new ThemeService();

    service.set('dark');

    expect(service.theme).toBe('dark');
    expect(dataset.theme).toBe('dark');
  });

  test('works without a DOM', () => {
    delete globalScope.window;
    delete globalScope.document;
    const service = new ThemeService();

    expect(() => service.init()).not.toThrow();
    expect(() => service.set('dark')).not.toThrow();
    expect(service.theme).toBe('dark');
  });
});
