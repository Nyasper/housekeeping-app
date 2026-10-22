import { afterEach, beforeEach, describe, expect, spyOn, test } from 'bun:test';

import { AppError, errorMessage, toAppError } from './errors.js';

let errorSpy: ReturnType<typeof spyOn>;

beforeEach(() => {
  errorSpy = spyOn(console, 'error').mockImplementation(() => {});
});

afterEach(() => {
  errorSpy.mockRestore();
});

describe('AppError', () => {
  test('keeps the code and message', () => {
    const error = new AppError('validation', 'Revisá los datos.');

    expect(error).toBeInstanceOf(Error);
    expect(error.name).toBe('AppError');
    expect(error.code).toBe('validation');
    expect(error.message).toBe('Revisá los datos.');
  });
});

describe('toAppError', () => {
  test('returns the same instance when it is already an AppError', () => {
    const original = new AppError('duplicate', 'Ya existe.');

    expect(toAppError(original)).toBe(original);
  });

  test('maps a command error payload', () => {
    const error = toAppError({ code: 'duplicate', message: 'Ya existe un registro.' });

    expect(error.code).toBe('duplicate');
    expect(error.message).toBe('Ya existe un registro.');
  });

  test('trims payload messages', () => {
    const error = toAppError({ code: 'custom', message: '  Detalle  ' });

    expect(error.code).toBe('custom');
    expect(error.message).toBe('Detalle');
  });

  test('uses the code message when the payload message is blank', () => {
    expect(toAppError({ code: 'validation', message: '' }).message).toBe(
      'Revisá los datos ingresados.'
    );
    expect(toAppError({ code: 'duplicate', message: '   ' }).message).toBe(
      'Ya existe un registro con esos datos.'
    );
  });

  test('falls back when the code is unknown and the message is blank', () => {
    const error = toAppError({ code: 'not_a_real_code', message: '' });

    expect(error.code).toBe('not_a_real_code');
    expect(error.message).toBe('Ocurrió un error inesperado. Volvé a intentarlo.');
  });

  test('maps string causes to an unexpected error', () => {
    const error = toAppError('boom');

    expect(error.code).toBe('unexpected');
    expect(error.message).toBe('boom');
  });

  test('falls back for blank strings', () => {
    const error = toAppError('   ');

    expect(error.code).toBe('unexpected');
    expect(error.message).toBe('Ocurrió un error inesperado. Volvé a intentarlo.');
    expect(errorSpy).toHaveBeenCalled();
  });

  test('falls back for unknown values', () => {
    for (const cause of [null, undefined, 42, {}, []]) {
      const error = toAppError(cause);

      expect(error.code).toBe('unexpected');
      expect(error.message).toBe('Ocurrió un error inesperado. Volvé a intentarlo.');
    }
    expect(errorSpy).toHaveBeenCalled();
  });
});

describe('errorMessage', () => {
  test('returns the normalized message', () => {
    expect(errorMessage({ code: 'validation', message: 'Datos inválidos' })).toBe('Datos inválidos');
    expect(errorMessage(new AppError('x', 'Mensaje'))).toBe('Mensaje');
  });
});
