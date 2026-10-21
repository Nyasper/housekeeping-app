/**
 * Errors crossing the Tauri bridge arrive as `{ code, message }`. Everything
 * else is converted into a Spanish, user-facing `AppError`.
 */

export interface CommandErrorPayload {
  code: string;
  message: string;
}

export class AppError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'AppError';
    this.code = code;
  }
}

const FALLBACK_MESSAGE = 'Ocurrió un error inesperado. Volvé a intentarlo.';

const codeMessages: Record<string, string> = {
  app_data_dir: 'No se pudo acceder a la carpeta de datos de la aplicación.',
  blocked: 'La operación está bloqueada por una regla de la aplicación.',
  constraint: 'Los datos no cumplen una regla de la base de datos.',
  database: 'No se pudo completar la operación en la base de datos.',
  database_open: 'No se pudo abrir la base de datos de la aplicación.',
  duplicate: 'Ya existe un registro con esos datos.',
  filesystem: 'No se pudo acceder a un archivo de la aplicación.',
  foreign_key: 'La operación hace referencia a datos que no existen.',
  not_found: 'No se encontró el registro solicitado.',
  required: 'Faltan datos obligatorios para completar la operación.',
  settings_invalid: 'Los valores de configuración están fuera del rango permitido.',
  validation: 'Revisá los datos ingresados.'
};

function isCommandErrorPayload(cause: unknown): cause is CommandErrorPayload {
  if (typeof cause !== 'object' || cause === null) {
    return false;
  }
  const value = cause as Record<string, unknown>;
  return typeof value.code === 'string' && typeof value.message === 'string';
}

export function toAppError(cause: unknown): AppError {
  if (cause instanceof AppError) {
    return cause;
  }

  if (isCommandErrorPayload(cause)) {
    const message = cause.message.trim() || codeMessages[cause.code] || FALLBACK_MESSAGE;
    return new AppError(cause.code, message);
  }

  if (typeof cause === 'string' && cause.trim()) {
    return new AppError('unexpected', cause);
  }

  console.error('Unexpected application error:', cause);
  return new AppError('unexpected', FALLBACK_MESSAGE);
}

export function errorMessage(cause: unknown): string {
  return toAppError(cause).message;
}
