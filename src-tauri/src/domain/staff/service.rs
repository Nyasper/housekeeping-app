use std::collections::BTreeSet;
use std::path::Path;

use sqlx::SqlitePool;

use super::model::{Absence, AbsenceInput, PlaceSummary, Staff, StaffInput, StaffRow};
use super::repo;
use crate::error::CommandError;
use crate::util::dates::{normalize_optional_date, parse_date, today_iso, validate_date_range};
use crate::util::files::{remove_relative_photo, resolve_photo_url, store_entity_photo};

const VALID_GENDERS: [&str; 2] = ["male", "female"];
const VALID_ROLES: [&str; 3] = ["maid", "window_cleaner", "common_areas"];
const VALID_ABSENCE_TYPES: [&str; 3] = ["medical_certificate", "leave", "absence"];

struct NormalizedStaffInput {
    first_name: String,
    last_name: String,
    birth_date: Option<String>,
    phone: Option<String>,
    national_id: String,
    address: Option<String>,
    gender: String,
    role: String,
    is_seasonal: bool,
    season_start_date: Option<String>,
    season_end_date: Option<String>,
    notes: Option<String>,
    photo_source_path: Option<String>,
    remove_photo: bool,
}

fn trim_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn validate_staff_input(input: StaffInput) -> Result<NormalizedStaffInput, CommandError> {
    let first_name = input.first_name.trim().to_string();
    if first_name.is_empty() {
        return Err(CommandError::new(
            "validation",
            "El nombre del funcionario es obligatorio.",
        ));
    }

    let last_name = input.last_name.trim().to_string();
    if last_name.is_empty() {
        return Err(CommandError::new(
            "validation",
            "El apellido del funcionario es obligatorio.",
        ));
    }

    let national_id = input.national_id.trim().to_string();
    if national_id.is_empty() {
        return Err(CommandError::new(
            "validation",
            "La cédula de identidad es obligatoria.",
        ));
    }

    let gender = input.gender.trim().to_string();
    if !VALID_GENDERS.contains(&gender.as_str()) {
        return Err(CommandError::new(
            "validation",
            "El género seleccionado no es válido.",
        ));
    }

    let role = input.role.trim().to_string();
    if !VALID_ROLES.contains(&role.as_str()) {
        return Err(CommandError::new(
            "validation",
            "El rol seleccionado no es válido.",
        ));
    }

    let birth_date = normalize_optional_date(input.birth_date.as_deref(), "nacimiento")?;
    let phone = trim_optional(input.phone.as_deref());
    let address = trim_optional(input.address.as_deref());
    let notes = trim_optional(input.notes.as_deref());

    let raw_season_start =
        normalize_optional_date(input.season_start_date.as_deref(), "inicio de temporada")?;
    let raw_season_end =
        normalize_optional_date(input.season_end_date.as_deref(), "fin de temporada")?;

    let (season_start_date, season_end_date) = if !input.is_seasonal {
        if raw_season_start.is_some() || raw_season_end.is_some() {
            return Err(CommandError::new(
                "validation",
                "Solo el personal zafral puede tener fechas de temporada.",
            ));
        }
        (None, None)
    } else {
        match (raw_season_start, raw_season_end) {
            (None, None) => (None, None),
            (Some(start), Some(end)) => {
                validate_date_range(&start, &end, "la temporada")?;
                (Some(start), Some(end))
            }
            _ => {
                return Err(CommandError::new(
                    "validation",
                    "Si cargás fechas de temporada, debés indicar tanto el inicio como el fin.",
                ));
            }
        }
    };

    let photo_source_path = trim_optional(input.photo_source_path.as_deref());

    Ok(NormalizedStaffInput {
        first_name,
        last_name,
        birth_date,
        phone,
        national_id,
        address,
        gender,
        role,
        is_seasonal: input.is_seasonal,
        season_start_date,
        season_end_date,
        notes,
        photo_source_path,
        remove_photo: input.remove_photo,
    })
}

fn map_row_to_staff(
    row: StaffRow,
    data_dir: Option<&Path>,
    main_places: Vec<PlaceSummary>,
    rest_days: Vec<i64>,
    warning: Option<String>,
) -> Staff {
    let photo_url = resolve_photo_url(data_dir, row.photo_path.as_deref());
    Staff {
        id: row.id,
        first_name: row.first_name,
        last_name: row.last_name,
        birth_date: row.birth_date,
        phone: row.phone,
        national_id: row.national_id,
        address: row.address,
        photo_path: row.photo_path,
        photo_url,
        gender: row.gender,
        role: row.role,
        is_seasonal: row.is_seasonal != 0,
        season_start_date: row.season_start_date,
        season_end_date: row.season_end_date,
        is_active: row.is_active != 0,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
        main_places,
        rest_days,
        warning,
    }
}

pub async fn list_staff(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
) -> Result<Vec<Staff>, CommandError> {
    let rows = repo::list_staff_rows(pool).await?;
    let mut main_places_map = repo::get_all_main_places_map(pool).await?;
    let mut rest_days_map = repo::get_all_rest_days_map(pool).await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let main_places = main_places_map.remove(&row.id).unwrap_or_default();
            let rest_days = rest_days_map.remove(&row.id).unwrap_or_default();
            map_row_to_staff(row, data_dir, main_places, rest_days, None)
        })
        .collect())
}

pub async fn get_staff(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
) -> Result<Staff, CommandError> {
    let row = repo::get_staff_row(pool, id).await?.ok_or_else(|| {
        CommandError::new("not_found", "No se encontró el funcionario solicitado.")
    })?;
    let main_places = repo::get_main_places_for_staff(pool, id).await?;
    let rest_days = repo::get_rest_days_for_staff(pool, id).await?;
    Ok(map_row_to_staff(
        row,
        data_dir,
        main_places,
        rest_days,
        None,
    ))
}

async fn ensure_unique_national_id(
    pool: &SqlitePool,
    national_id: &str,
    exclude_id: Option<i64>,
) -> Result<(), CommandError> {
    if let Some(existing) = repo::find_by_national_id(pool, national_id).await?
        && Some(existing.id) != exclude_id
    {
        if existing.is_active == 0 {
            return Err(CommandError::new(
                "duplicate",
                format!(
                    "Ya existe un funcionario dado de baja con la cédula {} ({} {}). Podés reactivarlo desde el listado.",
                    national_id, existing.first_name, existing.last_name
                ),
            ));
        }
        return Err(CommandError::new(
            "duplicate",
            format!(
                "Ya existe un funcionario registrado con la cédula {}.",
                national_id
            ),
        ));
    }
    Ok(())
}

pub async fn create_staff(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    input: StaffInput,
) -> Result<Staff, CommandError> {
    let normalized = validate_staff_input(input)?;
    ensure_unique_national_id(pool, &normalized.national_id, None).await?;

    let id = repo::insert_staff(
        pool,
        &normalized.first_name,
        &normalized.last_name,
        normalized.birth_date.as_deref(),
        normalized.phone.as_deref(),
        &normalized.national_id,
        normalized.address.as_deref(),
        &normalized.gender,
        &normalized.role,
        normalized.is_seasonal,
        normalized.season_start_date.as_deref(),
        normalized.season_end_date.as_deref(),
        normalized.notes.as_deref(),
    )
    .await?;

    if let (Some(base_dir), Some(source_path)) = (data_dir, normalized.photo_source_path.as_deref())
    {
        let relative = store_entity_photo(base_dir, "staff", id, Path::new(source_path), None)?;
        repo::update_photo_path(pool, id, Some(&relative)).await?;
    }

    get_staff(pool, data_dir, id).await
}

pub async fn update_staff(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
    input: StaffInput,
) -> Result<Staff, CommandError> {
    let existing = repo::get_staff_row(pool, id).await?.ok_or_else(|| {
        CommandError::new("not_found", "No se encontró el funcionario solicitado.")
    })?;

    let normalized = validate_staff_input(input)?;
    ensure_unique_national_id(pool, &normalized.national_id, Some(id)).await?;

    repo::update_staff(
        pool,
        id,
        &normalized.first_name,
        &normalized.last_name,
        normalized.birth_date.as_deref(),
        normalized.phone.as_deref(),
        &normalized.national_id,
        normalized.address.as_deref(),
        &normalized.gender,
        &normalized.role,
        normalized.is_seasonal,
        normalized.season_start_date.as_deref(),
        normalized.season_end_date.as_deref(),
        normalized.notes.as_deref(),
    )
    .await?;

    if let Some(base_dir) = data_dir {
        if let Some(source_path) = normalized.photo_source_path.as_deref() {
            let relative = store_entity_photo(
                base_dir,
                "staff",
                id,
                Path::new(source_path),
                existing.photo_path.as_deref(),
            )?;
            repo::update_photo_path(pool, id, Some(&relative)).await?;
        } else if normalized.remove_photo {
            if let Some(previous) = existing.photo_path.as_deref() {
                remove_relative_photo(base_dir, previous);
            }
            repo::update_photo_path(pool, id, None).await?;
        }
    } else if normalized.remove_photo {
        repo::update_photo_path(pool, id, None).await?;
    }

    get_staff(pool, data_dir, id).await
}

pub async fn deactivate_staff(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
) -> Result<Staff, CommandError> {
    let _existing = repo::get_staff_row(pool, id).await?.ok_or_else(|| {
        CommandError::new("not_found", "No se encontró el funcionario solicitado.")
    })?;

    let today = today_iso();
    let future_services = repo::list_future_active_service_counts(pool, id, &today).await?;

    let solo_count = future_services
        .iter()
        .filter(|(_, total)| *total <= 1)
        .count();
    if solo_count > 0 {
        return Err(CommandError::new(
            "blocked",
            format!(
                "No se puede dar de baja al funcionario porque es el único asignado en {solo_count} servicio(s) futuro(s). Reasigná o cancelá esos servicios primero."
            ),
        ));
    }

    let shared_count = future_services.len();
    repo::set_staff_active(pool, id, false).await?;

    let mut staff = get_staff(pool, data_dir, id).await?;
    if shared_count > 0 {
        staff.warning = Some(format!(
            "El funcionario fue dado de baja. Tenía {shared_count} servicio(s) futuro(s) con más personal asignado que quedaron para revisión."
        ));
    }
    Ok(staff)
}

pub async fn reactivate_staff(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
) -> Result<Staff, CommandError> {
    let affected = repo::set_staff_active(pool, id, true).await?;
    if affected == 0 {
        return Err(CommandError::new(
            "not_found",
            "No se encontró el funcionario solicitado.",
        ));
    }
    get_staff(pool, data_dir, id).await
}

pub async fn list_available_apartments(
    pool: &SqlitePool,
) -> Result<Vec<PlaceSummary>, CommandError> {
    Ok(repo::list_active_apartments(pool).await?)
}

pub async fn set_staff_places(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    staff_id: i64,
    place_ids: Vec<i64>,
) -> Result<Staff, CommandError> {
    let _existing = repo::get_staff_row(pool, staff_id).await?.ok_or_else(|| {
        CommandError::new("not_found", "No se encontró el funcionario solicitado.")
    })?;

    let unique_ids: Vec<i64> = place_ids
        .into_iter()
        .collect::<BTreeSet<i64>>()
        .into_iter()
        .collect();

    let valid_count = repo::count_valid_active_apartments(pool, &unique_ids).await?;
    if valid_count != unique_ids.len() {
        return Err(CommandError::new(
            "validation",
            "Los apartamentos principales deben ser apartamentos activos existentes.",
        ));
    }

    repo::replace_staff_places(pool, staff_id, &unique_ids).await?;
    get_staff(pool, data_dir, staff_id).await
}

pub async fn get_rest_days(pool: &SqlitePool, staff_id: i64) -> Result<Vec<i64>, CommandError> {
    let _existing = repo::get_staff_row(pool, staff_id).await?.ok_or_else(|| {
        CommandError::new("not_found", "No se encontró el funcionario solicitado.")
    })?;
    Ok(repo::get_rest_days_for_staff(pool, staff_id).await?)
}

pub async fn set_rest_days(
    pool: &SqlitePool,
    staff_id: i64,
    weekdays: Vec<i64>,
) -> Result<Vec<i64>, CommandError> {
    let _existing = repo::get_staff_row(pool, staff_id).await?.ok_or_else(|| {
        CommandError::new("not_found", "No se encontró el funcionario solicitado.")
    })?;

    let mut unique: BTreeSet<i64> = BTreeSet::new();
    for day in weekdays {
        if !(0..=6).contains(&day) {
            return Err(CommandError::new(
                "validation",
                "Los días de descanso deben estar entre 0 (lunes) y 6 (domingo).",
            ));
        }
        unique.insert(day);
    }

    let sorted: Vec<i64> = unique.into_iter().collect();
    repo::replace_rest_days(pool, staff_id, &sorted).await?;
    Ok(sorted)
}

pub async fn list_absences(
    pool: &SqlitePool,
    staff_id: Option<i64>,
) -> Result<Vec<Absence>, CommandError> {
    Ok(repo::list_absences(pool, staff_id).await?)
}

struct NormalizedAbsenceInput {
    staff_id: i64,
    absence_type: String,
    start_date: String,
    end_date: String,
    reason: Option<String>,
}

fn validate_absence_input(input: AbsenceInput) -> Result<NormalizedAbsenceInput, CommandError> {
    let absence_type = input.absence_type.trim().to_string();
    if !VALID_ABSENCE_TYPES.contains(&absence_type.as_str()) {
        return Err(CommandError::new(
            "validation",
            "El tipo de ausencia seleccionado no es válido.",
        ));
    }

    let start_parsed = parse_date(&input.start_date, "inicio de la ausencia")?;
    let end_parsed = parse_date(&input.end_date, "fin de la ausencia")?;
    if start_parsed > end_parsed {
        return Err(CommandError::new(
            "validation",
            "La fecha de inicio de la ausencia no puede ser posterior a la fecha de fin.",
        ));
    }

    Ok(NormalizedAbsenceInput {
        staff_id: input.staff_id,
        absence_type,
        start_date: start_parsed.format("%Y-%m-%d").to_string(),
        end_date: end_parsed.format("%Y-%m-%d").to_string(),
        reason: trim_optional(input.reason.as_deref()),
    })
}

pub async fn create_absence(
    pool: &SqlitePool,
    input: AbsenceInput,
) -> Result<Absence, CommandError> {
    let normalized = validate_absence_input(input)?;
    let _staff = repo::get_staff_row(pool, normalized.staff_id)
        .await?
        .ok_or_else(|| {
            CommandError::new("not_found", "No se encontró el funcionario solicitado.")
        })?;

    if repo::has_overlapping_absence(
        pool,
        normalized.staff_id,
        &normalized.start_date,
        &normalized.end_date,
        None,
    )
    .await?
    {
        return Err(CommandError::new(
            "duplicate",
            "El funcionario ya tiene una ausencia registrada que se superpone con esas fechas.",
        ));
    }

    let id = repo::insert_absence(
        pool,
        normalized.staff_id,
        &normalized.absence_type,
        &normalized.start_date,
        &normalized.end_date,
        normalized.reason.as_deref(),
    )
    .await?;

    repo::get_absence(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se pudo recuperar la ausencia creada."))
}

pub async fn update_absence(
    pool: &SqlitePool,
    id: i64,
    input: AbsenceInput,
) -> Result<Absence, CommandError> {
    let _existing = repo::get_absence(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró la ausencia solicitada."))?;

    let normalized = validate_absence_input(input)?;
    let _staff = repo::get_staff_row(pool, normalized.staff_id)
        .await?
        .ok_or_else(|| {
            CommandError::new("not_found", "No se encontró el funcionario solicitado.")
        })?;

    if repo::has_overlapping_absence(
        pool,
        normalized.staff_id,
        &normalized.start_date,
        &normalized.end_date,
        Some(id),
    )
    .await?
    {
        return Err(CommandError::new(
            "duplicate",
            "El funcionario ya tiene otra ausencia que se superpone con esas fechas.",
        ));
    }

    repo::update_absence(
        pool,
        id,
        normalized.staff_id,
        &normalized.absence_type,
        &normalized.start_date,
        &normalized.end_date,
        normalized.reason.as_deref(),
    )
    .await?;

    repo::get_absence(pool, id).await?.ok_or_else(|| {
        CommandError::new("not_found", "No se pudo recuperar la ausencia actualizada.")
    })
}

pub async fn delete_absence(pool: &SqlitePool, id: i64) -> Result<(), CommandError> {
    let affected = repo::delete_absence(pool, id).await?;
    if affected == 0 {
        return Err(CommandError::new(
            "not_found",
            "No se encontró la ausencia solicitada.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::db;

    async fn test_pool() -> SqlitePool {
        let pool = db::connect(Path::new(":memory:"))
            .await
            .expect("in-memory db");
        db::migrate(&pool).await.expect("migrations");
        pool
    }

    fn sample_input(national_id: &str) -> StaffInput {
        StaffInput {
            first_name: "María".to_string(),
            last_name: "Gómez".to_string(),
            birth_date: Some("1990-05-14".to_string()),
            phone: Some("099123456".to_string()),
            national_id: national_id.to_string(),
            address: Some("Av. 18 de Julio 1234".to_string()),
            gender: "female".to_string(),
            role: "maid".to_string(),
            is_seasonal: false,
            season_start_date: None,
            season_end_date: None,
            notes: Some("Puntual".to_string()),
            photo_source_path: None,
            remove_photo: false,
        }
    }

    #[test]
    fn staff_crud_and_national_id_uniqueness() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            let created = create_staff(&pool, None, sample_input("41234567"))
                .await
                .expect("create staff");
            assert_eq!(created.first_name, "María");
            assert!(created.is_active);
            assert!(!created.is_seasonal);

            // Duplicate national_id while active
            let dup_err = create_staff(&pool, None, sample_input("41234567"))
                .await
                .expect_err("duplicate national_id");
            assert_eq!(dup_err.code, "duplicate");

            // Deactivate and try creating with same national_id -> error suggests reactivation
            let deactivated = deactivate_staff(&pool, None, created.id)
                .await
                .expect("deactivate");
            assert!(!deactivated.is_active);

            let inactive_dup_err = create_staff(&pool, None, sample_input("41234567"))
                .await
                .expect_err("duplicate inactive national_id");
            assert_eq!(inactive_dup_err.code, "duplicate");
            assert!(inactive_dup_err.message.contains("reactivarlo"));

            let reactivated = reactivate_staff(&pool, None, created.id)
                .await
                .expect("reactivate");
            assert!(reactivated.is_active);

            pool.close().await;
        });
    }

    #[test]
    fn seasonal_validation_rules() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            // Non-seasonal with dates -> rejected
            let mut bad_non_seasonal = sample_input("10000001");
            bad_non_seasonal.is_seasonal = false;
            bad_non_seasonal.season_start_date = Some("2026-12-01".to_string());
            bad_non_seasonal.season_end_date = Some("2027-03-01".to_string());
            let err = create_staff(&pool, None, bad_non_seasonal)
                .await
                .expect_err("non-seasonal with dates");
            assert_eq!(err.code, "validation");

            // Seasonal with only one date -> rejected
            let mut partial_seasonal = sample_input("10000002");
            partial_seasonal.is_seasonal = true;
            partial_seasonal.season_start_date = Some("2026-12-01".to_string());
            let err = create_staff(&pool, None, partial_seasonal)
                .await
                .expect_err("partial season");
            assert_eq!(err.code, "validation");

            // Seasonal with start > end -> rejected
            let mut inverted_seasonal = sample_input("10000003");
            inverted_seasonal.is_seasonal = true;
            inverted_seasonal.season_start_date = Some("2027-03-10".to_string());
            inverted_seasonal.season_end_date = Some("2026-12-01".to_string());
            let err = create_staff(&pool, None, inverted_seasonal)
                .await
                .expect_err("inverted season");
            assert_eq!(err.code, "validation");

            // Seasonal without dates -> allowed (informative flag)
            let mut info_seasonal = sample_input("10000004");
            info_seasonal.is_seasonal = true;
            let staff_info = create_staff(&pool, None, info_seasonal)
                .await
                .expect("informative seasonal");
            assert!(staff_info.is_seasonal);
            assert!(staff_info.season_start_date.is_none());

            // Seasonal with valid dates -> allowed
            let mut valid_seasonal = sample_input("10000005");
            valid_seasonal.is_seasonal = true;
            valid_seasonal.season_start_date = Some("2026-12-01".to_string());
            valid_seasonal.season_end_date = Some("2027-03-01".to_string());
            let staff_valid = create_staff(&pool, None, valid_seasonal)
                .await
                .expect("valid seasonal");
            assert_eq!(staff_valid.season_start_date.as_deref(), Some("2026-12-01"));

            pool.close().await;
        });
    }

    #[test]
    fn rest_days_and_main_places_and_absences_rules() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;
            let staff = create_staff(&pool, None, sample_input("22223333"))
                .await
                .expect("staff");

            // Rest days
            let days = set_rest_days(&pool, staff.id, vec![6, 0, 6])
                .await
                .expect("set rest days");
            assert_eq!(days, vec![0, 6]);

            let bad_day = set_rest_days(&pool, staff.id, vec![7])
                .await
                .expect_err("weekday 7");
            assert_eq!(bad_day.code, "validation");

            // Places: apartment vs common_area
            sqlx::query("INSERT INTO place (id, name, type, is_active) VALUES (1, 'Apto 101', 'apartment', 1), (2, 'Lobby', 'common_area', 1), (3, 'Apto 102', 'apartment', 0)")
                .execute(&pool)
                .await
                .expect("seed places");

            let updated = set_staff_places(&pool, None, staff.id, vec![1])
                .await
                .expect("assign active apartment");
            assert_eq!(updated.main_places.len(), 1);
            assert_eq!(updated.main_places[0].name, "Apto 101");

            // Common area rejected
            let err_common = set_staff_places(&pool, None, staff.id, vec![2])
                .await
                .expect_err("common area rejected");
            assert_eq!(err_common.code, "validation");

            // Inactive apartment rejected
            let err_inactive = set_staff_places(&pool, None, staff.id, vec![3])
                .await
                .expect_err("inactive apartment rejected");
            assert_eq!(err_inactive.code, "validation");

            // Absences without overlaps
            let abs1 = create_absence(
                &pool,
                AbsenceInput {
                    staff_id: staff.id,
                    absence_type: "leave".to_string(),
                    start_date: "2026-11-10".to_string(),
                    end_date: "2026-11-15".to_string(),
                    reason: Some("Vacaciones".to_string()),
                },
            )
            .await
            .expect("create absence");

            // Overlapping absence (touching end date 2026-11-15) -> rejected
            let overlap_err = create_absence(
                &pool,
                AbsenceInput {
                    staff_id: staff.id,
                    absence_type: "medical_certificate".to_string(),
                    start_date: "2026-11-15".to_string(),
                    end_date: "2026-11-18".to_string(),
                    reason: None,
                },
            )
            .await
            .expect_err("overlapping absence");
            assert_eq!(overlap_err.code, "duplicate");

            // Updating abs1 itself with extended end_date -> allowed
            let abs1_updated = update_absence(
                &pool,
                abs1.id,
                AbsenceInput {
                    staff_id: staff.id,
                    absence_type: "leave".to_string(),
                    start_date: "2026-11-10".to_string(),
                    end_date: "2026-11-16".to_string(),
                    reason: Some("Vacaciones extendidas".to_string()),
                },
            )
            .await
            .expect("update absence");
            assert_eq!(abs1_updated.end_date, "2026-11-16");

            delete_absence(&pool, abs1.id)
                .await
                .expect("delete absence");
            assert!(
                list_absences(&pool, Some(staff.id))
                    .await
                    .expect("list")
                    .is_empty()
            );

            pool.close().await;
        });
    }

    #[test]
    fn deactivate_blocks_when_sole_staff_on_future_service_and_warns_when_shared() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;
            let s1 = create_staff(&pool, None, sample_input("30000001"))
                .await
                .expect("s1");
            let s2 = create_staff(&pool, None, sample_input("30000002"))
                .await
                .expect("s2");

            sqlx::query("INSERT INTO place (id, name, type, is_active) VALUES (10, 'Apto 201', 'apartment', 1)")
                .execute(&pool)
                .await
                .expect("place");

            let future_date = "2099-12-31";
            sqlx::query("INSERT INTO cleaning_service (id, place_id, date, service_type, status) VALUES (100, 10, ?1, 'daily_cleaning', 'planned')")
                .bind(future_date)
                .execute(&pool)
                .await
                .expect("service");

            sqlx::query(
                "INSERT INTO service_staff (service_id, staff_id, is_lead) VALUES (100, ?1, 1)",
            )
            .bind(s1.id)
            .execute(&pool)
            .await
            .expect("assign s1");

            // Sole assignee -> blocked
            let blocked = deactivate_staff(&pool, None, s1.id)
                .await
                .expect_err("sole assignee blocked");
            assert_eq!(blocked.code, "blocked");

            // Co-assign s2 -> allowed with warning
            sqlx::query(
                "INSERT INTO service_staff (service_id, staff_id, is_lead) VALUES (100, ?1, 0)",
            )
            .bind(s2.id)
            .execute(&pool)
            .await
            .expect("assign s2");

            let deactivated = deactivate_staff(&pool, None, s1.id)
                .await
                .expect("deactivated with warning");
            assert!(!deactivated.is_active);
            assert!(deactivated.warning.is_some());

            pool.close().await;
        });
    }
}
