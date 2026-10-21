use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use tauri::{AppHandle, Manager, State, async_runtime::RwLock};
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::db;
use crate::error::CommandError;
use crate::state::DbState;

const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
const DB_ARCHIVE_ENTRY: &str = "housekeeping.db";
const MANIFEST_ARCHIVE_ENTRY: &str = "manifest.json";
const PHOTOS_ARCHIVE_PREFIX: &str = "photos/";
const BACKUP_FORMAT_VERSION: u32 = 1;
const SCHEMA_VERSION: u32 = 1;

const BACKUP_SCHEMA: &[(&str, &[&str])] = &[
    (
        "staff",
        &[
            "id",
            "first_name",
            "last_name",
            "birth_date",
            "phone",
            "national_id",
            "address",
            "photo_path",
            "gender",
            "role",
            "is_seasonal",
            "season_start_date",
            "season_end_date",
            "is_active",
            "notes",
            "created_at",
            "updated_at",
        ],
    ),
    ("staff_rest_day", &["staff_id", "weekday"]),
    (
        "absence",
        &[
            "id",
            "staff_id",
            "type",
            "start_date",
            "end_date",
            "reason",
            "created_at",
        ],
    ),
    (
        "place",
        &[
            "id",
            "name",
            "type",
            "status",
            "location",
            "check_in_date",
            "check_out_date",
            "is_active",
            "notes",
            "created_at",
            "updated_at",
        ],
    ),
    ("staff_place", &["staff_id", "place_id"]),
    (
        "cleaning_service",
        &[
            "id",
            "place_id",
            "date",
            "start_time",
            "end_time",
            "service_type",
            "status",
            "notes",
            "created_at",
            "updated_at",
        ],
    ),
    ("service_staff", &["service_id", "staff_id", "is_lead"]),
    (
        "incident",
        &[
            "id",
            "service_id",
            "type",
            "description",
            "photo_path",
            "status",
            "date",
            "resolved_at",
            "created_at",
        ],
    ),
    ("settings", &["key", "value", "updated_at"]),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCounts {
    pub staff: i64,
    pub places: i64,
    pub cleaning_services: i64,
    pub incidents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub app_version: String,
    pub schema_version: u32,
    pub format_version: u32,
    pub created_at: String,
    pub counts: BackupCounts,
}

#[tauri::command]
pub async fn backup_create(
    dest_path: String,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let live_db = db::db_path(&app)?;
    let live_photos = db::photos_dir(&app)?;
    let cache_root = app
        .path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir());

    backup_create_with_paths(
        state.pool(),
        &live_db,
        &live_photos,
        &cache_root,
        &dest_path,
    )
    .await
}

#[tauri::command]
pub async fn backup_restore(
    src_path: String,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<BackupManifest, CommandError> {
    let live_db = db::db_path(&app)?;
    let live_photos = db::photos_dir(&app)?;
    let cache_root = app
        .path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir());

    backup_restore_with_paths(state.pool(), &live_db, &live_photos, &cache_root, &src_path).await
}

fn unique_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn same_file_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => a == b,
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

fn remove_sqlite_files(path: &Path) -> Result<(), std::io::Error> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    for suffix in ["-wal", "-shm"] {
        let companion = with_suffix(path, suffix);
        if companion.exists() {
            fs::remove_file(companion)?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    if !src.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(src).min_depth(1) {
        let entry = entry.map_err(std::io::Error::other)?;
        let rel = entry
            .path()
            .strip_prefix(src)
            .map_err(std::io::Error::other)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }

    Ok(())
}

async fn collect_counts(pool: &SqlitePool) -> Result<BackupCounts, CommandError> {
    let staff: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM staff")
        .fetch_one(pool)
        .await?;
    let places: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM place")
        .fetch_one(pool)
        .await?;
    let cleaning_services: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cleaning_service")
        .fetch_one(pool)
        .await?;
    let incidents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incident")
        .fetch_one(pool)
        .await?;

    Ok(BackupCounts {
        staff,
        places,
        cleaning_services,
        incidents,
    })
}

async fn vacuum_into(pool: &SqlitePool, target_db: &Path) -> Result<(), CommandError> {
    let _ = remove_sqlite_files(target_db);
    let escaped = target_db.to_string_lossy().replace('\'', "''");
    let sql = format!("VACUUM INTO '{escaped}'");
    sqlx::query(&sql).execute(pool).await.map_err(|error| {
        eprintln!("VACUUM INTO error: {error}");
        CommandError::new(
            "backup_failed",
            "No se pudo generar la instantánea consistente de la base de datos.",
        )
    })?;
    Ok(())
}

/// Testable implementation of `backup_create`.
pub async fn backup_create_with_paths(
    pool_lock: &RwLock<SqlitePool>,
    live_db_path: &Path,
    live_photos_dir: &Path,
    cache_root: &Path,
    dest_path_str: &str,
) -> Result<String, CommandError> {
    let trimmed = dest_path_str.trim();
    if trimmed.is_empty() {
        return Err(CommandError::new(
            "validation",
            "Seleccioná dónde guardar el respaldo.",
        ));
    }

    let destination = PathBuf::from(trimmed);
    if same_file_path(&destination, live_db_path) {
        return Err(CommandError::new(
            "backup_same_file",
            "Elegí otra ubicación: no se puede sobrescribir la base de datos activa.",
        ));
    }

    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        return Err(CommandError::new(
            "backup_folder_missing",
            "La carpeta seleccionada para guardar el respaldo no existe.",
        ));
    }

    let staging_dir = cache_root.join(format!("backup-create-{}", unique_stamp()));
    fs::create_dir_all(&staging_dir)?;

    let result = async {
        let staged_db = staging_dir.join(DB_ARCHIVE_ENTRY);
        let counts = {
            let pool = pool_lock.read().await;
            vacuum_into(&pool, &staged_db).await?;
            collect_counts(&pool).await?
        };

        validate_sqlite_backup(&staged_db).await?;

        let manifest = BackupManifest {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version: SCHEMA_VERSION,
            format_version: BACKUP_FORMAT_VERSION,
            created_at: Utc::now().to_rfc3339(),
            counts,
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| {
            eprintln!("Manifest serialize error: {error}");
            CommandError::new(
                "backup_failed",
                "No se pudo generar el manifiesto del respaldo.",
            )
        })?;

        let temp_zip_path = with_suffix(&destination, ".tmp");
        let _ = fs::remove_file(&temp_zip_path);

        if let Err(error) =
            write_backup_zip(&temp_zip_path, &staged_db, live_photos_dir, &manifest_bytes)
        {
            let _ = fs::remove_file(&temp_zip_path);
            return Err(error);
        }

        fs::rename(&temp_zip_path, &destination).map_err(|error| {
            let _ = fs::remove_file(&temp_zip_path);
            eprintln!("Backup rename error: {error}");
            CommandError::new(
                "backup_failed",
                "No se pudo finalizar el archivo de respaldo en la ubicación seleccionada.",
            )
        })?;

        Ok(destination.to_string_lossy().into_owned())
    }
    .await;

    let _ = fs::remove_dir_all(&staging_dir);
    result
}

fn write_backup_zip(
    zip_path: &Path,
    staged_db: &Path,
    photos_dir: &Path,
    manifest_bytes: &[u8],
) -> Result<(), CommandError> {
    let file = File::create(zip_path).map_err(|error| {
        eprintln!("Backup zip create error: {error}");
        CommandError::new(
            "backup_failed",
            "No se pudo crear el archivo ZIP del respaldo.",
        )
    })?;

    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // 1. manifest.json
    zip.start_file(MANIFEST_ARCHIVE_ENTRY, options)
        .map_err(map_zip_write_error)?;
    zip.write_all(manifest_bytes)?;

    // 2. housekeeping.db
    zip.start_file(DB_ARCHIVE_ENTRY, options)
        .map_err(map_zip_write_error)?;
    let mut db_file = File::open(staged_db)?;
    std::io::copy(&mut db_file, &mut zip)?;

    // 3. photos/ directory
    zip.add_directory(PHOTOS_ARCHIVE_PREFIX, options)
        .map_err(map_zip_write_error)?;

    if photos_dir.exists() {
        for entry in WalkDir::new(photos_dir).min_depth(1) {
            let entry = entry.map_err(|error| {
                eprintln!("WalkDir photos error: {error}");
                CommandError::new(
                    "backup_failed",
                    "No se pudieron recorrer las fotografías para el respaldo.",
                )
            })?;

            let rel = entry.path().strip_prefix(photos_dir).map_err(|error| {
                eprintln!("Strip prefix error: {error}");
                CommandError::new(
                    "backup_failed",
                    "No se pudo preparar la ruta de una fotografía en el respaldo.",
                )
            })?;

            let normalized_rel = rel
                .components()
                .filter_map(|comp| match comp {
                    Component::Normal(os) => Some(os.to_string_lossy().into_owned()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("/");

            if normalized_rel.is_empty() {
                continue;
            }

            if entry.file_type().is_dir() {
                let archive_dir = format!("{PHOTOS_ARCHIVE_PREFIX}{normalized_rel}/");
                zip.add_directory(archive_dir, options)
                    .map_err(map_zip_write_error)?;
            } else if entry.file_type().is_file() {
                let archive_file = format!("{PHOTOS_ARCHIVE_PREFIX}{normalized_rel}");
                zip.start_file(archive_file, options)
                    .map_err(map_zip_write_error)?;
                let mut photo_file = File::open(entry.path())?;
                std::io::copy(&mut photo_file, &mut zip)?;
            }
        }
    }

    zip.finish().map_err(map_zip_write_error)?;
    Ok(())
}

fn map_zip_write_error(error: zip::result::ZipError) -> CommandError {
    eprintln!("Zip write error: {error}");
    CommandError::new(
        "backup_failed",
        "No se pudo comprimir el archivo de respaldo ZIP.",
    )
}

/// Validates and extracts a backup ZIP into `extract_dir`, returning the parsed `BackupManifest`.
fn extract_and_verify_zip(
    zip_path: &Path,
    extract_dir: &Path,
) -> Result<BackupManifest, CommandError> {
    let file = File::open(zip_path).map_err(|error| {
        eprintln!("Backup open error: {error}");
        CommandError::new(
            "invalid_backup",
            "No se pudo abrir el archivo de respaldo seleccionado.",
        )
    })?;

    let mut archive = ZipArchive::new(file).map_err(|error| {
        eprintln!("Invalid zip archive: {error}");
        CommandError::new(
            "invalid_backup",
            "El archivo seleccionado no es un respaldo ZIP válido.",
        )
    })?;

    let mut has_db = false;
    let mut has_manifest = false;
    let mut has_photos_entry = false;

    let extracted_photos_dir = extract_dir.join("photos");
    fs::create_dir_all(&extracted_photos_dir)?;

    let mut manifest_bytes = Vec::new();

    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx).map_err(|error| {
            eprintln!("Zip entry error: {error}");
            CommandError::new(
                "invalid_backup",
                "El archivo ZIP del respaldo está dañado o incompleto.",
            )
        })?;

        let raw_name = entry
            .name()
            .map_err(|error| {
                eprintln!("Zip entry name error: {error}");
                CommandError::new(
                    "invalid_backup",
                    "El respaldo contiene nombres de archivo inválidos.",
                )
            })?
            .replace('\\', "/");
        if is_unsafe_zip_path(&raw_name) {
            return Err(CommandError::new(
                "invalid_backup",
                "El respaldo contiene rutas no permitidas.",
            ));
        }

        if raw_name == DB_ARCHIVE_ENTRY {
            if entry.is_dir() {
                return Err(CommandError::new(
                    "invalid_backup",
                    "La entrada de base de datos en el respaldo no es un archivo válido.",
                ));
            }
            let target_db = extract_dir.join(DB_ARCHIVE_ENTRY);
            let mut out = File::create(&target_db)?;
            std::io::copy(&mut entry, &mut out)?;
            has_db = true;
        } else if raw_name == MANIFEST_ARCHIVE_ENTRY {
            if entry.is_dir() {
                return Err(CommandError::new(
                    "invalid_backup",
                    "El manifiesto del respaldo no es un archivo válido.",
                ));
            }
            entry.read_to_end(&mut manifest_bytes)?;
            has_manifest = true;
        } else if raw_name == "photos" || raw_name == PHOTOS_ARCHIVE_PREFIX {
            has_photos_entry = true;
        } else if let Some(rel_photo) = raw_name.strip_prefix(PHOTOS_ARCHIVE_PREFIX) {
            has_photos_entry = true;
            if rel_photo.is_empty() {
                continue;
            }
            let target_path = extracted_photos_dir.join(rel_photo);
            if entry.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out = File::create(&target_path)?;
                std::io::copy(&mut entry, &mut out)?;
            }
        }
    }

    if !has_db || !has_manifest || !has_photos_entry {
        return Err(CommandError::new(
            "invalid_backup",
            "El archivo ZIP no contiene la estructura completa de un respaldo (housekeeping.db, photos/ y manifest.json).",
        ));
    }

    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
        eprintln!("Manifest parse error: {error}");
        CommandError::new(
            "invalid_backup",
            "El manifiesto del respaldo tiene un formato inválido.",
        )
    })?;

    if manifest.format_version != BACKUP_FORMAT_VERSION {
        return Err(CommandError::new(
            "invalid_backup",
            "La versión del formato de respaldo no es compatible con esta versión de la aplicación.",
        ));
    }

    Ok(manifest)
}

fn is_unsafe_zip_path(name: &str) -> bool {
    if name.is_empty() || name.starts_with('/') || name.contains(':') {
        return true;
    }
    Path::new(name).components().any(|comp| {
        matches!(
            comp,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

/// Validates that `path` is a genuine SQLite database with the expected schema,
/// passing both `PRAGMA integrity_check` and `PRAGMA foreign_key_check`.
pub async fn validate_sqlite_backup(path: &Path) -> Result<(), CommandError> {
    let invalid = || {
        CommandError::new(
            "invalid_backup",
            "El respaldo no contiene una base de datos válida de Gestión de Limpieza.",
        )
    };

    let metadata = fs::metadata(path).map_err(|_| invalid())?;
    if !metadata.is_file() || metadata.len() < SQLITE_HEADER.len() as u64 {
        return Err(invalid());
    }

    let mut header = [0_u8; 16];
    File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .map_err(|_| invalid())?;
    if &header != SQLITE_HEADER {
        return Err(invalid());
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(false)
                .read_only(true)
                .busy_timeout(Duration::from_secs(5)),
        )
        .await
        .map_err(|_| invalid())?;

    let check_result = async {
        for (table, expected_columns) in BACKUP_SCHEMA {
            let pragma = format!("PRAGMA table_info({table})");
            let columns: Vec<String> = sqlx::query_scalar::<_, String>(&format!(
                "SELECT name FROM pragma_table_info('{table}')"
            ))
            .fetch_all(&pool)
            .await
            .map_err(|error| {
                eprintln!("Schema check error ({pragma}): {error}");
                invalid()
            })?;

            let column_set: HashSet<&str> = columns.iter().map(String::as_str).collect();
            if !expected_columns
                .iter()
                .all(|column| column_set.contains(column))
            {
                return Err(invalid());
            }
        }

        let integrity: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_all(&pool)
            .await
            .map_err(|_| invalid())?;
        if integrity.len() != 1 || !integrity[0].eq_ignore_ascii_case("ok") {
            return Err(CommandError::new(
                "invalid_backup",
                "El respaldo seleccionado está dañado y no se puede restaurar.",
            ));
        }

        let fk_violations = sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await
            .map_err(|_| invalid())?;
        if !fk_violations.is_empty() {
            return Err(CommandError::new(
                "invalid_backup",
                "El respaldo contiene referencias inconsistentes y no se puede restaurar.",
            ));
        }

        Ok(())
    }
    .await;

    pool.close().await;
    check_result
}

/// Testable implementation of `backup_restore`.
pub async fn backup_restore_with_paths(
    pool_lock: &RwLock<SqlitePool>,
    live_db_path: &Path,
    live_photos_dir: &Path,
    cache_root: &Path,
    src_path_str: &str,
) -> Result<BackupManifest, CommandError> {
    let trimmed = src_path_str.trim();
    if trimmed.is_empty() {
        return Err(CommandError::new(
            "validation",
            "Seleccioná un archivo de respaldo para restaurar.",
        ));
    }

    let source = PathBuf::from(trimmed);
    if !source.exists() {
        return Err(CommandError::new(
            "backup_not_found",
            "El archivo de respaldo seleccionado no existe.",
        ));
    }

    if same_file_path(&source, live_db_path) {
        return Err(CommandError::new(
            "restore_same_file",
            "El archivo seleccionado ya es la base de datos activa.",
        ));
    }

    let work_dir = cache_root.join(format!("backup-restore-{}", unique_stamp()));
    let extract_dir = work_dir.join("extracted");
    let safety_dir = work_dir.join("safety");
    fs::create_dir_all(&extract_dir)?;
    fs::create_dir_all(&safety_dir)?;

    let outcome = async {
        // 1. Validate ZIP and extract DB + photos into extract_dir
        let manifest = extract_and_verify_zip(&source, &extract_dir)?;
        let extracted_db = extract_dir.join(DB_ARCHIVE_ENTRY);
        let extracted_photos = extract_dir.join("photos");

        // 2. Validate extracted SQLite DB thoroughly before touching live files
        validate_sqlite_backup(&extracted_db).await?;

        // 3. Create safety copy of live DB and live photos
        let safety_db = safety_dir.join(DB_ARCHIVE_ENTRY);
        let safety_photos = safety_dir.join("photos");
        {
            let pool = pool_lock.read().await;
            vacuum_into(&pool, &safety_db).await.map_err(|error| {
                eprintln!("Restore safety DB backup error: {error}");
                CommandError::new(
                    "restore_safety_failed",
                    "No se pudo crear una copia de seguridad previa a la restauración.",
                )
            })?;
        }
        copy_dir_recursive(live_photos_dir, &safety_photos).map_err(|error| {
            eprintln!("Restore safety photos copy error: {error}");
            CommandError::new(
                "restore_safety_failed",
                "No se pudieron resguardar las fotografías actuales antes de restaurar.",
            )
        })?;

        // 4. Write-lock DbState, close live pool, swap DB and photos, reopen & migrate
        let mut pool_guard = pool_lock.write().await;
        pool_guard.close().await;

        let swap_result: Result<SqlitePool, CommandError> = async {
            remove_sqlite_files(live_db_path).map_err(|error| {
                eprintln!("Remove live DB error: {error}");
                CommandError::new(
                    "restore_failed",
                    "No se pudo reemplazar el archivo de base de datos activo.",
                )
            })?;

            if let Some(parent) = live_db_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&extracted_db, live_db_path).map_err(|error| {
                eprintln!("Copy extracted DB error: {error}");
                CommandError::new(
                    "restore_failed",
                    "No se pudo copiar la base de datos del respaldo.",
                )
            })?;

            if live_photos_dir.exists() {
                fs::remove_dir_all(live_photos_dir)?;
            }
            copy_dir_recursive(&extracted_photos, live_photos_dir)?;

            let reopened = db::connect(live_db_path).await.map_err(|error| {
                eprintln!("Reopen restored DB error: {error}");
                CommandError::new(
                    "restore_failed",
                    "No se pudo abrir la base de datos restaurada.",
                )
            })?;

            if let Err(error) = db::migrate(&reopened).await {
                eprintln!("Migrate restored DB error: {error}");
                reopened.close().await;
                return Err(CommandError::new(
                    "restore_failed",
                    "La base de datos del respaldo no es compatible con las migraciones actuales.",
                ));
            }

            Ok(reopened)
        }
        .await;

        match swap_result {
            Ok(restored_pool) => {
                *pool_guard = restored_pool;
                Ok(manifest)
            }
            Err(restore_error) => {
                // Automatic rollback from safety backup
                let _ = remove_sqlite_files(live_db_path);
                let _ = fs::copy(&safety_db, live_db_path);
                if live_photos_dir.exists() {
                    let _ = fs::remove_dir_all(live_photos_dir);
                }
                let _ = copy_dir_recursive(&safety_photos, live_photos_dir);

                if let Ok(recovered_pool) = db::connect(live_db_path).await {
                    let _ = db::migrate(&recovered_pool).await;
                    *pool_guard = recovered_pool;
                }

                Err(restore_error)
            }
        }
    }
    .await;

    let _ = fs::remove_dir_all(&work_dir);
    outcome
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::domain::places::{model::PlaceInput, service as places_service};
    use crate::domain::staff::{model::StaffInput, service as staff_service};

    #[test]
    fn backup_and_restore_round_trip_preserves_database_and_photos() {
        tauri::async_runtime::block_on(async {
            let temp = TempDir::new().expect("tempdir");
            let data_dir = temp.path().join("data");
            let cache_dir = temp.path().join("cache");
            let db_path = data_dir.join("housekeeping.db");
            let photos_dir = data_dir.join("photos");
            fs::create_dir_all(&photos_dir).expect("photos dir");
            fs::create_dir_all(&cache_dir).expect("cache dir");

            let pool = db::connect(&db_path).await.expect("connect");
            db::migrate(&pool).await.expect("migrate");
            let state = DbState::new(pool);

            // Seed initial staff + place + photo file
            {
                let guard = state.pool().read().await;
                places_service::create_place(
                    &guard,
                    PlaceInput {
                        name: "Apto 101".to_string(),
                        place_type: "apartment".to_string(),
                        status: None,
                        location: Some("Piso 1".to_string()),
                        check_in_date: None,
                        check_out_date: None,
                        notes: None,
                    },
                )
                .await
                .expect("create place");

                staff_service::create_staff(
                    &guard,
                    None,
                    StaffInput {
                        first_name: "Lucía".to_string(),
                        last_name: "Méndez".to_string(),
                        birth_date: None,
                        phone: None,
                        national_id: "41112223".to_string(),
                        address: None,
                        gender: "female".to_string(),
                        role: "maid".to_string(),
                        is_seasonal: false,
                        season_start_date: None,
                        season_end_date: None,
                        notes: None,
                        photo_source_path: None,
                        remove_photo: false,
                    },
                )
                .await
                .expect("create staff");
            }

            let staff_photo_dir = photos_dir.join("staff");
            fs::create_dir_all(&staff_photo_dir).expect("staff photo dir");
            let sample_photo = staff_photo_dir.join("sample.png");
            fs::write(&sample_photo, b"photo-bytes-original").expect("write photo");

            let zip_path = temp.path().join("respaldo.zip");
            let zip_str = zip_path.to_string_lossy().into_owned();

            // Create backup ZIP
            let saved_path =
                backup_create_with_paths(state.pool(), &db_path, &photos_dir, &cache_dir, &zip_str)
                    .await
                    .expect("backup_create");
            assert_eq!(saved_path, zip_str);
            assert!(zip_path.exists());

            // Modify live DB and delete the photo after the backup was taken
            {
                let guard = state.pool().read().await;
                places_service::create_place(
                    &guard,
                    PlaceInput {
                        name: "Apto 999 Posterior".to_string(),
                        place_type: "apartment".to_string(),
                        status: None,
                        location: None,
                        check_in_date: None,
                        check_out_date: None,
                        notes: None,
                    },
                )
                .await
                .expect("create second place");
            }
            fs::remove_file(&sample_photo).expect("remove photo");
            assert!(!sample_photo.exists());

            // Restore from backup ZIP
            let manifest = backup_restore_with_paths(
                state.pool(),
                &db_path,
                &photos_dir,
                &cache_dir,
                &zip_str,
            )
            .await
            .expect("backup_restore");

            assert_eq!(manifest.counts.staff, 1);
            assert_eq!(manifest.counts.places, 1);

            // Verify live DB only has the 1 original place and photo was restored
            {
                let guard = state.pool().read().await;
                let places = places_service::list_places(&guard).await.expect("places");
                assert_eq!(places.len(), 1);
                assert_eq!(places[0].name, "Apto 101");
            }
            assert!(sample_photo.exists());
            assert_eq!(
                fs::read(&sample_photo).expect("read restored photo"),
                b"photo-bytes-original"
            );

            state.pool().write().await.close().await;
        });
    }

    #[test]
    fn restore_rejects_corrupt_zip_and_keeps_live_database_intact() {
        tauri::async_runtime::block_on(async {
            let temp = TempDir::new().expect("tempdir");
            let data_dir = temp.path().join("data");
            let cache_dir = temp.path().join("cache");
            let db_path = data_dir.join("housekeeping.db");
            let photos_dir = data_dir.join("photos");
            fs::create_dir_all(&photos_dir).expect("photos dir");
            fs::create_dir_all(&cache_dir).expect("cache dir");

            let pool = db::connect(&db_path).await.expect("connect");
            db::migrate(&pool).await.expect("migrate");
            let state = DbState::new(pool);

            {
                let guard = state.pool().read().await;
                places_service::create_place(
                    &guard,
                    PlaceInput {
                        name: "Lobby Principal".to_string(),
                        place_type: "common_area".to_string(),
                        status: None,
                        location: None,
                        check_in_date: None,
                        check_out_date: None,
                        notes: None,
                    },
                )
                .await
                .expect("create place");
            }

            let corrupt_zip = temp.path().join("corrupt.zip");
            fs::write(&corrupt_zip, b"not a valid zip file").expect("write corrupt zip");

            let err = backup_restore_with_paths(
                state.pool(),
                &db_path,
                &photos_dir,
                &cache_dir,
                &corrupt_zip.to_string_lossy(),
            )
            .await
            .expect_err("must reject corrupt zip");
            assert_eq!(err.code, "invalid_backup");

            // Also test a valid ZIP whose housekeeping.db is not a valid SQLite database
            let fake_db_zip = temp.path().join("fake_db.zip");
            {
                let file = File::create(&fake_db_zip).expect("create fake zip");
                let mut zip = ZipWriter::new(file);
                let opts = SimpleFileOptions::default();
                zip.start_file("manifest.json", opts).expect("manifest");
                let manifest = BackupManifest {
                    app_version: "0.1.0".to_string(),
                    schema_version: 1,
                    format_version: 1,
                    created_at: "2026-10-10T00:00:00Z".to_string(),
                    counts: BackupCounts {
                        staff: 0,
                        places: 0,
                        cleaning_services: 0,
                        incidents: 0,
                    },
                };
                zip.write_all(&serde_json::to_vec(&manifest).expect("json"))
                    .expect("write manifest");
                zip.start_file("housekeeping.db", opts).expect("db entry");
                zip.write_all(b"corrupted sqlite content")
                    .expect("write db");
                zip.add_directory("photos/", opts).expect("photos dir");
                zip.finish().expect("finish zip");
            }

            let err2 = backup_restore_with_paths(
                state.pool(),
                &db_path,
                &photos_dir,
                &cache_dir,
                &fake_db_zip.to_string_lossy(),
            )
            .await
            .expect_err("must reject invalid sqlite inside zip");
            assert_eq!(err2.code, "invalid_backup");

            // Verify live DB still has Lobby Principal
            {
                let guard = state.pool().read().await;
                let places = places_service::list_places(&guard).await.expect("places");
                assert_eq!(places.len(), 1);
                assert_eq!(places[0].name, "Lobby Principal");
            }

            state.pool().write().await.close().await;
        });
    }
}
