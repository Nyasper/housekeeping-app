use std::fmt;

use serde::Serialize;

/// Serializable command error. The frontend receives `{ code, message }` and
/// uses the stable `code` to decide how to react; the message is always in
/// Spanish and can be shown directly to the user.
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

impl From<sqlx::Error> for CommandError {
    fn from(error: sqlx::Error) -> Self {
        eprintln!("Database error: {error}");
        if let sqlx::Error::Database(database_error) = &error
            && let Some(code) = database_error.code()
            && let Ok(number) = code.parse::<i32>()
        {
            return match number {
                // SQLITE_CONSTRAINT_UNIQUE / SQLITE_CONSTRAINT_PRIMARYKEY
                2067 | 1555 => Self::new("duplicate", "Ya existe un registro con esos datos."),
                // SQLITE_CONSTRAINT_FOREIGNKEY
                787 => Self::new(
                    "foreign_key",
                    "La operación hace referencia a datos que no existen.",
                ),
                // SQLITE_CONSTRAINT_CHECK
                275 => Self::new(
                    "constraint",
                    "Los datos no cumplen una regla de la base de datos.",
                ),
                // SQLITE_CONSTRAINT_NOTNULL
                1299 => Self::new(
                    "required",
                    "Faltan datos obligatorios para completar la operación.",
                ),
                // SQLITE_CONSTRAINT_TRIGGER
                1811 => Self::new(
                    "blocked",
                    "La operación está bloqueada por una regla de la aplicación.",
                ),
                _ => Self::new(
                    "database",
                    "No se pudo completar la operación en la base de datos.",
                ),
            };
        }
        Self::new(
            "database",
            "No se pudo completar la operación en la base de datos.",
        )
    }
}

impl From<std::io::Error> for CommandError {
    fn from(error: std::io::Error) -> Self {
        eprintln!("File system error: {error}");
        Self::new(
            "filesystem",
            "No se pudo acceder a un archivo de la aplicación.",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_code_and_message() {
        let error = CommandError::new("settings_invalid", "Valor fuera de rango.");
        let json = serde_json::to_value(&error).expect("serializable");
        assert_eq!(json["code"], "settings_invalid");
        assert_eq!(json["message"], "Valor fuera de rango.");
    }

    #[test]
    fn maps_unique_constraint_to_duplicate_code() {
        tauri::async_runtime::block_on(async {
            let pool = crate::db::connect(std::path::Path::new(":memory:"))
                .await
                .expect("in-memory database");
            sqlx::query("CREATE TABLE unique_test (name TEXT NOT NULL UNIQUE)")
                .execute(&pool)
                .await
                .expect("table");
            sqlx::query("INSERT INTO unique_test (name) VALUES ('Limpieza')")
                .execute(&pool)
                .await
                .expect("first row");
            let error = sqlx::query("INSERT INTO unique_test (name) VALUES ('Limpieza')")
                .execute(&pool)
                .await
                .expect_err("duplicate");
            assert_eq!(CommandError::from(error).code, "duplicate");
            pool.close().await;
        });
    }
}
