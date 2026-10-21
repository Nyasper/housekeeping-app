use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

use crate::error::CommandError;
use crate::state::DbState;

pub const RECENT_VACATED_DAYS_KEY: &str = "recent_vacated_days";
pub const UPCOMING_OCCUPANCY_DAYS_KEY: &str = "upcoming_occupancy_days";
pub const DEFAULT_RECENT_VACATED_DAYS: i64 = 2;
pub const DEFAULT_UPCOMING_OCCUPANCY_DAYS: i64 = 2;
pub const MIN_DAYS: i64 = 0;
pub const MAX_DAYS: i64 = 30;

/// Global settings exposed to the frontend. Values are always within their
/// valid range: invalid stored values fall back to the defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AppSettings {
    pub recent_vacated_days: i64,
    pub upcoming_occupancy_days: i64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            recent_vacated_days: DEFAULT_RECENT_VACATED_DAYS,
            upcoming_occupancy_days: DEFAULT_UPCOMING_OCCUPANCY_DAYS,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SettingsInput {
    pub recent_vacated_days: i64,
    pub upcoming_occupancy_days: i64,
}

impl SettingsInput {
    fn validate(self) -> Result<AppSettings, CommandError> {
        validate_days(
            self.recent_vacated_days,
            "días para considerar un apartamento desocupado recientemente",
        )?;
        validate_days(
            self.upcoming_occupancy_days,
            "días de anticipación para una próxima ocupación",
        )?;
        Ok(AppSettings {
            recent_vacated_days: self.recent_vacated_days,
            upcoming_occupancy_days: self.upcoming_occupancy_days,
        })
    }
}

fn validate_days(value: i64, description: &str) -> Result<(), CommandError> {
    if !(MIN_DAYS..=MAX_DAYS).contains(&value) {
        return Err(CommandError::new(
            "settings_invalid",
            format!("El valor de {description} debe estar entre {MIN_DAYS} y {MAX_DAYS} días."),
        ));
    }
    Ok(())
}

/// Reads the settings, falling back to the defaults when a key is missing or
/// contains an invalid value.
pub async fn get_settings(pool: &SqlitePool) -> Result<AppSettings, CommandError> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM settings WHERE key IN (?1, ?2)",
    )
    .bind(RECENT_VACATED_DAYS_KEY)
    .bind(UPCOMING_OCCUPANCY_DAYS_KEY)
    .fetch_all(pool)
    .await?;

    let mut settings = AppSettings::default();
    for (key, value) in rows {
        let parsed = value.parse::<i64>().ok();
        match key.as_str() {
            RECENT_VACATED_DAYS_KEY => {
                settings.recent_vacated_days = parsed
                    .filter(|value| (MIN_DAYS..=MAX_DAYS).contains(value))
                    .unwrap_or(DEFAULT_RECENT_VACATED_DAYS);
            }
            UPCOMING_OCCUPANCY_DAYS_KEY => {
                settings.upcoming_occupancy_days = parsed
                    .filter(|value| (MIN_DAYS..=MAX_DAYS).contains(value))
                    .unwrap_or(DEFAULT_UPCOMING_OCCUPANCY_DAYS);
            }
            _ => {}
        }
    }

    Ok(settings)
}

/// Validates and upserts both settings in a single transaction.
pub async fn save_settings(
    pool: &SqlitePool,
    input: SettingsInput,
) -> Result<AppSettings, CommandError> {
    let settings = input.validate()?;
    let mut transaction = pool.begin().await?;

    for (key, value) in [
        (RECENT_VACATED_DAYS_KEY, settings.recent_vacated_days),
        (
            UPCOMING_OCCUPANCY_DAYS_KEY,
            settings.upcoming_occupancy_days,
        ),
    ] {
        sqlx::query(
            "INSERT INTO settings (key, value)
             VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET
                 value = excluded.value,
                 updated_at = datetime('now')",
        )
        .bind(key)
        .bind(value.to_string())
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(settings)
}

#[tauri::command]
pub async fn settings_get(state: State<'_, DbState>) -> Result<AppSettings, CommandError> {
    let pool = state.pool().read().await.clone();
    get_settings(&pool).await
}

#[tauri::command]
pub async fn settings_update(
    input: SettingsInput,
    state: State<'_, DbState>,
) -> Result<AppSettings, CommandError> {
    let pool = state.pool().read().await.clone();
    save_settings(&pool, input).await
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::db;

    async fn test_database() -> SqlitePool {
        let pool = db::connect(Path::new(":memory:"))
            .await
            .expect("in-memory database");
        db::migrate(&pool).await.expect("migrations");
        pool
    }

    #[test]
    fn fresh_database_returns_defaults() {
        tauri::async_runtime::block_on(async {
            let pool = test_database().await;
            let settings = get_settings(&pool).await.expect("settings");
            assert_eq!(settings, AppSettings::default());
            pool.close().await;
        });
    }

    #[test]
    fn settings_are_saved_and_read_back() {
        tauri::async_runtime::block_on(async {
            let pool = test_database().await;
            let saved = save_settings(
                &pool,
                SettingsInput {
                    recent_vacated_days: 5,
                    upcoming_occupancy_days: 7,
                },
            )
            .await
            .expect("save settings");
            assert_eq!(saved.recent_vacated_days, 5);
            assert_eq!(saved.upcoming_occupancy_days, 7);

            let reloaded = get_settings(&pool).await.expect("reload settings");
            assert_eq!(reloaded, saved);
            pool.close().await;
        });
    }

    #[test]
    fn out_of_range_values_are_rejected() {
        tauri::async_runtime::block_on(async {
            let pool = test_database().await;
            let error = save_settings(
                &pool,
                SettingsInput {
                    recent_vacated_days: 31,
                    upcoming_occupancy_days: 2,
                },
            )
            .await
            .expect_err("invalid settings");
            assert_eq!(error.code, "settings_invalid");
            pool.close().await;
        });
    }

    #[test]
    fn invalid_stored_values_fall_back_to_defaults() {
        tauri::async_runtime::block_on(async {
            let pool = test_database().await;
            sqlx::query("UPDATE settings SET value = 'not-a-number' WHERE key = ?1")
                .bind(RECENT_VACATED_DAYS_KEY)
                .execute(&pool)
                .await
                .expect("invalid value");
            sqlx::query("UPDATE settings SET value = '99' WHERE key = ?1")
                .bind(UPCOMING_OCCUPANCY_DAYS_KEY)
                .execute(&pool)
                .await
                .expect("out of range value");

            let settings = get_settings(&pool).await.expect("settings");
            assert_eq!(settings, AppSettings::default());
            pool.close().await;
        });
    }

    #[test]
    fn missing_keys_fall_back_to_defaults() {
        tauri::async_runtime::block_on(async {
            let pool = test_database().await;
            sqlx::query("DELETE FROM settings")
                .execute(&pool)
                .await
                .expect("delete settings");
            let settings = get_settings(&pool).await.expect("settings");
            assert_eq!(settings, AppSettings::default());
            pool.close().await;
        });
    }
}
