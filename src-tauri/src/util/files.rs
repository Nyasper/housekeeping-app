use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::CommandError;

const ALLOWED_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "webp", "gif"];

/// Copies a source image into `{data_dir}/photos/{category}/{entity_id}/` with a
/// unique file name and returns the relative path (`photos/{category}/{entity_id}/{file}`).
pub fn store_entity_photo(
    data_dir: &Path,
    category: &str,
    entity_id: i64,
    source_path: &Path,
    previous_relative_path: Option<&str>,
) -> Result<String, CommandError> {
    if !source_path.exists() || !source_path.is_file() {
        return Err(CommandError::new(
            "validation",
            "El archivo de imagen seleccionado no existe.",
        ));
    }

    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .filter(|ext| ALLOWED_EXTENSIONS.contains(&ext.as_str()))
        .ok_or_else(|| {
            CommandError::new(
                "validation",
                "Formato de imagen no admitido. Usá JPG, PNG, WEBP o GIF.",
            )
        })?;

    let relative_dir = format!("photos/{category}/{entity_id}");
    let target_dir = data_dir.join(&relative_dir);
    std::fs::create_dir_all(&target_dir)?;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let file_name = format!("photo-{nanos}.{extension}");
    let target_path = target_dir.join(&file_name);

    if let Err(error) = std::fs::copy(source_path, &target_path) {
        let _ = std::fs::remove_file(&target_path);
        return Err(error.into());
    }

    if let Some(previous) = previous_relative_path {
        remove_relative_photo(data_dir, previous);
    }

    Ok(format!("{relative_dir}/{file_name}"))
}

/// Resolves a stored relative path only when it points to an existing file
/// inside `data_dir/photos`. Rejects absolute paths, drive prefixes and parent
/// traversal so a manipulated database cannot reference or delete arbitrary
/// files on the machine.
fn resolve_contained_photo_path(data_dir: &Path, relative_path: &str) -> Option<PathBuf> {
    let trimmed = relative_path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let relative = Path::new(trimmed);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }

    let full_path = data_dir.join(relative);
    let photos_root = data_dir.join("photos");
    let canonical_root = photos_root.canonicalize().ok()?;
    let canonical_target = full_path.canonicalize().ok()?;
    if canonical_target.starts_with(&canonical_root) {
        Some(full_path)
    } else {
        None
    }
}

/// Removes a stored photo file if it exists inside `data_dir/photos`.
pub fn remove_relative_photo(data_dir: &Path, relative_path: &str) {
    if let Some(full_path) = resolve_contained_photo_path(data_dir, relative_path)
        && full_path.is_file()
    {
        let _ = std::fs::remove_file(full_path);
    }
}

/// Resolves a relative photo path stored in the database to an absolute path
/// string suitable for the frontend (`convertFileSrc`).
pub fn resolve_photo_url(data_dir: Option<&Path>, relative_path: Option<&str>) -> Option<String> {
    let base = data_dir?;
    let rel = relative_path?;
    resolve_contained_photo_path(base, rel).map(|path| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_replaces_and_resolves_photo() {
        let dir = tempfile::tempdir().expect("tempdir");
        let data_dir = dir.path().join("appdata");
        std::fs::create_dir_all(&data_dir).expect("appdata");

        let src = dir.path().join("sample.png");
        std::fs::write(&src, b"fake-png-bytes").expect("write source");

        let rel1 = store_entity_photo(&data_dir, "staff", 7, &src, None).expect("store 1");
        assert!(rel1.starts_with("photos/staff/7/photo-"));
        assert!(rel1.ends_with(".png"));

        let resolved1 = resolve_photo_url(Some(&data_dir), Some(&rel1)).expect("resolved 1");
        assert!(Path::new(&resolved1).exists());

        let rel2 = store_entity_photo(&data_dir, "staff", 7, &src, Some(&rel1)).expect("store 2");
        assert_ne!(rel1, rel2);
        assert!(resolve_photo_url(Some(&data_dir), Some(&rel1)).is_none());
        assert!(resolve_photo_url(Some(&data_dir), Some(&rel2)).is_some());
    }

    #[test]
    fn rejects_unsupported_extension() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("malware.exe");
        std::fs::write(&src, b"nope").expect("write source");

        let err = store_entity_photo(dir.path(), "staff", 1, &src, None).expect_err("bad ext");
        assert_eq!(err.code, "validation");
    }

    #[test]
    fn only_resolves_and_removes_files_inside_the_photos_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let data_dir = dir.path().join("appdata");
        std::fs::create_dir_all(data_dir.join("photos")).expect("photos");

        let database = data_dir.join("housekeeping.db");
        std::fs::write(&database, b"database").expect("database");
        let outside = dir.path().join("outside.txt");
        std::fs::write(&outside, b"outside").expect("outside");

        remove_relative_photo(&data_dir, "housekeeping.db");
        assert!(database.exists());
        assert!(resolve_photo_url(Some(&data_dir), Some("housekeeping.db")).is_none());

        let absolute = outside.to_string_lossy();
        remove_relative_photo(&data_dir, &absolute);
        assert!(outside.exists());
        assert!(resolve_photo_url(Some(&data_dir), Some(&absolute)).is_none());

        remove_relative_photo(&data_dir, "../outside.txt");
        assert!(outside.exists());
    }
}
