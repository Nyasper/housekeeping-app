use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use tauri::{AppHandle, Manager};

use crate::error::CommandError;

pub type Db = SqlitePool;

const DATABASE_FILE_NAME: &str = "housekeeping.db";
const PHOTOS_DIRECTORY_NAME: &str = "photos";

/// Opens the SQLite pool with the settings required by the application:
/// one serialized connection, WAL journal, foreign keys and a 10 second busy
/// timeout. The parent directory is created when missing.
pub async fn connect(path: &Path) -> Result<Db, sqlx::Error> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
    }

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .foreign_keys(true)
                .busy_timeout(Duration::from_secs(10))
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .pragma("temp_store", "2")
                .pragma("cache_size", "-8000"),
        )
        .await
}

/// Applies every versioned migration. SQLx records applied migrations in
/// `_sqlx_migrations`, so calling this again is safe.
pub async fn migrate(pool: &Db) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

/// Directory where the database and photos live.
pub fn data_dir(app: &AppHandle) -> Result<PathBuf, CommandError> {
    app.path().app_data_dir().map_err(|error| {
        eprintln!("App data directory error: {error}");
        CommandError::new(
            "app_data_dir",
            "No se pudo resolver el directorio de datos de la aplicación.",
        )
    })
}

pub fn db_path(app: &AppHandle) -> Result<PathBuf, CommandError> {
    Ok(data_dir(app)?.join(DATABASE_FILE_NAME))
}

pub fn photos_dir(app: &AppHandle) -> Result<PathBuf, CommandError> {
    Ok(data_dir(app)?.join(PHOTOS_DIRECTORY_NAME))
}

/// Creates the data and photos directories during startup.
pub fn prepare_data_dirs(app: &AppHandle) -> Result<(), CommandError> {
    std::fs::create_dir_all(data_dir(app)?)?;
    std::fs::create_dir_all(photos_dir(app)?)?;
    Ok(())
}

/// Opens and migrates the database. On success returns the pool and an optional
/// notice for the frontend when a damaged database had to be quarantined.
pub fn open_with_recovery(path: &Path) -> Result<(Db, Option<String>), CommandError> {
    match open_and_migrate(path) {
        Ok(pool) => Ok((pool, None)),
        Err(error) => {
            eprintln!("Database open error: {error}");
            if !path.exists() || !should_quarantine(&error) {
                return Err(CommandError::new(
                    "database_open",
                    format!("No se pudo abrir la base de datos: {error}"),
                ));
            }

            let quarantined = quarantine_database(path).map_err(|quarantine_error| {
                CommandError::new(
                    "quarantine_failed",
                    format!("No se pudo apartar la base de datos dañada: {quarantine_error}"),
                )
            })?;

            let pool = open_and_migrate(path).map_err(|recreate_error| {
                CommandError::new(
                    "database_recreate",
                    format!("No se pudo crear una base de datos nueva: {recreate_error}"),
                )
            })?;

            let name = quarantined
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            Ok((
                pool,
                Some(format!(
                    "La base de datos no se pudo abrir y se apartó como «{name}». Se inició con una base vacía; podés restaurar un respaldo desde Ajustes."
                )),
            ))
        }
    }
}

fn open_and_migrate(path: &Path) -> Result<Db, sqlx::Error> {
    tauri::async_runtime::block_on(async {
        let pool = connect(path).await?;
        if let Err(error) = migrate(&pool).await {
            // Release the file handle before a possible quarantine on Windows.
            pool.close().await;
            return Err(error);
        }
        Ok(pool)
    })
}

/// Moves a database that could not be opened next to the original file, along
/// with its WAL/SHM companions. Returns the new path.
pub fn quarantine_database(path: &Path) -> Result<PathBuf, std::io::Error> {
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No existe el archivo de base de datos.",
        ));
    }

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let quarantined = with_suffix(path, &format!(".corrupt-{stamp}"));

    std::fs::rename(path, &quarantined)?;
    for suffix in ["-wal", "-shm"] {
        let source = with_suffix(path, suffix);
        if source.exists() {
            let destination = with_suffix(&quarantined, suffix);
            let _ = std::fs::rename(source, destination);
        }
    }

    Ok(quarantined)
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

/// Only corruption and "not a database" errors justify moving the file away.
/// Locked or inaccessible databases must not be quarantined.
fn should_quarantine(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database_error) => {
            if let Some(code) = database_error.code()
                && let Ok(number) = code.parse::<i32>()
            {
                let primary_code = number & 0xFF;
                // SQLITE_CORRUPT (11) and SQLITE_NOTADB (26).
                if primary_code == 11 || primary_code == 26 {
                    return true;
                }
            }
            looks_corrupt(database_error.message())
        }
        // A migration failure may carry the database error as text.
        sqlx::Error::Migrate(migration_error) => looks_corrupt(&migration_error.to_string()),
        _ => false,
    }
}

fn looks_corrupt(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("malformed")
        || message.contains("file is not a database")
        || message.contains("database disk image is malformed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_creates_parent_directories() {
        tauri::async_runtime::block_on(async {
            let directory = tempfile::tempdir().expect("tempdir");
            let path = directory.path().join("nested").join("housekeeping.db");
            let pool = connect(&path).await.expect("database");
            migrate(&pool).await.expect("migrations");
            assert!(path.exists());
            pool.close().await;
        });
    }

    #[test]
    fn migrations_create_the_expected_schema() {
        tauri::async_runtime::block_on(async {
            let directory = tempfile::tempdir().expect("tempdir");
            let pool = connect(&directory.path().join("housekeeping.db"))
                .await
                .expect("database");
            migrate(&pool).await.expect("migrations");

            let tables = sqlx::query_scalar::<_, String>(
                "SELECT name FROM sqlite_master
                 WHERE type = 'table'
                   AND name NOT LIKE 'sqlite_%'
                   AND name NOT LIKE '_sqlx%'
                 ORDER BY name",
            )
            .fetch_all(&pool)
            .await
            .expect("table list");

            for expected in [
                "absence",
                "cleaning_service",
                "incident",
                "place",
                "service_staff",
                "settings",
                "staff",
                "staff_place",
                "staff_rest_day",
            ] {
                assert!(tables.iter().any(|table| table == expected), "{expected}");
            }

            let settings_count: i64 = sqlx::query_scalar("SELECT count(*) FROM settings")
                .fetch_one(&pool)
                .await
                .expect("settings count");
            assert_eq!(settings_count, 2);

            let trigger_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM sqlite_master
                 WHERE type = 'trigger' AND name = 'trg_service_staff_keep_last'",
            )
            .fetch_one(&pool)
            .await
            .expect("trigger count");
            assert_eq!(trigger_count, 1);

            pool.close().await;
        });
    }

    #[test]
    fn valid_database_reopens_without_notice() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("housekeeping.db");

        let (pool, notice) = open_with_recovery(&path).expect("first open");
        assert!(notice.is_none());
        tauri::async_runtime::block_on(pool.close());

        let (pool, notice) = open_with_recovery(&path).expect("second open");
        assert!(notice.is_none());
        tauri::async_runtime::block_on(pool.close());
    }

    #[test]
    fn corrupt_database_is_quarantined_and_recreated() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("housekeeping.db");
        std::fs::write(&path, b"this is not a sqlite database").expect("corrupt file");

        let (pool, notice) = open_with_recovery(&path).expect("recovered database");
        let notice = notice.expect("recovery notice");
        assert!(notice.contains("corrupt"));
        assert!(notice.contains("Ajustes"));

        let has_settings: i64 = tauri::async_runtime::block_on(
            sqlx::query_scalar(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'settings'",
            )
            .fetch_one(&pool),
        )
        .expect("settings table");
        assert_eq!(has_settings, 1);
        tauri::async_runtime::block_on(pool.close());

        assert!(
            std::fs::read_dir(directory.path())
                .expect("directory")
                .filter_map(Result::ok)
                .any(|entry| entry.file_name().to_string_lossy().contains(".corrupt-"))
        );
    }
}
