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
  backup_failed: 'No se pudo crear el archivo de respaldo.',
  backup_folder_missing: 'La carpeta seleccionada para guardar el respaldo no existe.',
  backup_not_found: 'El archivo de respaldo seleccionado no existe.',
  backup_same_file: 'Elegí otra ubicación: no se puede sobrescribir la base de datos activa.',
  blocked: 'La operación está bloqueada por una regla de la aplicación.',
  constraint: 'Los datos no cumplen una regla de la base de datos.',
  database: 'No se pudo completar la operación en la base de datos.',
  database_open: 'No se pudo abrir la base de datos de la aplicación.',
  duplicate: 'Ya existe un registro con esos datos.',
  export_failed: 'No se pudo generar el archivo exportado.',
  filesystem: 'No se pudo acceder a un archivo de la aplicación.',
  foreign_key: 'La operación hace referencia a datos que no existen.',
  invalid_backup: 'El archivo seleccionado no es un respaldo válido de Gestión de Limpieza.',
  not_found: 'No se encontró el registro solicitado.',
  required: 'Faltan datos obligatorios para completar la operación.',
  restore_failed: 'No se pudo restaurar el respaldo.',
  restore_safety_failed: 'No se pudo crear la copia de seguridad previa a la restauración.',
  restore_same_file: 'El archivo seleccionado ya es la base de datos activa.',
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
