use chrono::{Local, NaiveDate};
use sqlx::SqlitePool;

use super::model::{Place, PlaceInput, PlaceRow, RecommendedTask};
use super::repo;
use crate::error::CommandError;
use crate::settings::{self, AppSettings};
use crate::util::availability::compute_recommended_task;
use crate::util::dates::{normalize_optional_date, parse_date, today_iso, validate_date_range};

const VALID_PLACE_TYPES: [&str; 2] = ["apartment", "common_area"];
const VALID_PLACE_STATUSES: [&str; 5] = [
    "occupied",
    "upcoming_occupancy",
    "vacant",
    "recently_vacated",
    "cancelled",
];

struct NormalizedPlaceInput {
    name: String,
    place_type: String,
    status: Option<String>,
    location: Option<String>,
    check_in_date: Option<String>,
    check_out_date: Option<String>,
    notes: Option<String>,
}

fn trim_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn validate_place_input(input: PlaceInput) -> Result<NormalizedPlaceInput, CommandError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(CommandError::new(
            "validation",
            "El nombre del lugar es obligatorio.",
        ));
    }

    let place_type = input.place_type.trim().to_string();
    if !VALID_PLACE_TYPES.contains(&place_type.as_str()) {
        return Err(CommandError::new(
            "validation",
            "El tipo de lugar seleccionado no es válido.",
        ));
    }

    let raw_status = trim_optional(input.status.as_deref());
    if let Some(ref st) = raw_status
        && !VALID_PLACE_STATUSES.contains(&st.as_str())
    {
        return Err(CommandError::new(
            "validation",
            "El estado seleccionado para el apartamento no es válido.",
        ));
    }

    let check_in_date = normalize_optional_date(input.check_in_date.as_deref(), "de ingreso")?;
    let check_out_date = normalize_optional_date(input.check_out_date.as_deref(), "de egreso")?;

    if place_type == "common_area"
        && (raw_status.is_some() || check_in_date.is_some() || check_out_date.is_some())
    {
        return Err(CommandError::new(
            "validation",
            "Las áreas comunes no admiten estado de ocupación ni fechas de ingreso o egreso.",
        ));
    }

    if let (Some(start), Some(end)) = (check_in_date.as_deref(), check_out_date.as_deref()) {
        validate_date_range(start, end, "de la estadía del apartamento")?;
    }

    Ok(NormalizedPlaceInput {
        name,
        place_type,
        status: raw_status,
        location: trim_optional(input.location.as_deref()),
        check_in_date,
        check_out_date,
        notes: trim_optional(input.notes.as_deref()),
    })
}

fn map_row_to_place(row: PlaceRow, reference_date: NaiveDate, settings: AppSettings) -> Place {
    let rec = compute_recommended_task(
        &row.place_type,
        row.status.as_deref(),
        row.check_in_date.as_deref(),
        row.check_out_date.as_deref(),
        reference_date,
        settings,
    );

    Place {
        id: row.id,
        name: row.name,
        place_type: row.place_type,
        location: row.location,
        status: row.status,
        effective_status: rec.effective_status,
        recommended_service_type: rec.recommended_service_type,
        check_in_date: row.check_in_date,
        check_out_date: row.check_out_date,
        is_active: row.is_active != 0,
        cancelled_at: row.cancelled_at,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn normalize_place_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

async fn ensure_unique_place_name(
    pool: &SqlitePool,
    name: &str,
    exclude_id: Option<i64>,
) -> Result<(), CommandError> {
    let normalized_name = normalize_place_name(name);
    let existing = repo::list_place_rows(pool)
        .await?
        .into_iter()
        .find(|place| normalize_place_name(&place.name) == normalized_name);

    if let Some(existing) = existing
        && Some(existing.id) != exclude_id
    {
        if existing.is_active == 0 {
            return Err(CommandError::new(
                "duplicate",
                format!(
                    "Ya existe un lugar desactivado con el nombre «{}». Podés reactivarlo desde el listado.",
                    existing.name
                ),
            ));
        }
        return Err(CommandError::new(
            "duplicate",
            format!("Ya existe un lugar con el nombre «{}».", existing.name),
        ));
    }
    Ok(())
}

pub async fn list_places(pool: &SqlitePool) -> Result<Vec<Place>, CommandError> {
    list_places_for_date(pool, None).await
}

pub async fn list_places_for_date(
    pool: &SqlitePool,
    date: Option<&str>,
) -> Result<Vec<Place>, CommandError> {
    let rows = repo::list_place_rows(pool).await?;
    let app_settings = settings::get_settings(pool).await?;
    let reference_date = match date.map(str::trim).filter(|s| !s.is_empty()) {
        Some(raw) => parse_date(raw, "de referencia")?,
        None => Local::now().date_naive(),
    };

    Ok(rows
        .into_iter()
        .map(|row| map_row_to_place(row, reference_date, app_settings))
        .collect())
}

pub async fn get_place(pool: &SqlitePool, id: i64) -> Result<Place, CommandError> {
    let row = repo::get_place_row(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró el lugar solicitado."))?;
    let app_settings = settings::get_settings(pool).await?;
    let today = Local::now().date_naive();
    Ok(map_row_to_place(row, today, app_settings))
}

pub async fn create_place(pool: &SqlitePool, input: PlaceInput) -> Result<Place, CommandError> {
    let normalized = validate_place_input(input)?;
    ensure_unique_place_name(pool, &normalized.name, None).await?;

    let id = repo::insert_place(
        pool,
        &normalized.name,
        &normalized.place_type,
        normalized.status.as_deref(),
        normalized.location.as_deref(),
        normalized.check_in_date.as_deref(),
        normalized.check_out_date.as_deref(),
        normalized.notes.as_deref(),
    )
    .await?;

    get_place(pool, id).await
}

pub async fn update_place(
    pool: &SqlitePool,
    id: i64,
    input: PlaceInput,
) -> Result<Place, CommandError> {
    let _existing = repo::get_place_row(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró el lugar solicitado."))?;

    let normalized = validate_place_input(input)?;
    ensure_unique_place_name(pool, &normalized.name, Some(id)).await?;

    let mut transaction = pool.begin().await?;
    let affected = repo::update_place(
        &mut *transaction,
        id,
        &normalized.name,
        &normalized.place_type,
        normalized.status.as_deref(),
        normalized.location.as_deref(),
        normalized.check_in_date.as_deref(),
        normalized.check_out_date.as_deref(),
        normalized.notes.as_deref(),
    )
    .await?;
    if affected == 0 {
        return Err(CommandError::new(
            "not_found",
            "No se encontró el lugar solicitado.",
        ));
    }

    if normalized.place_type != "apartment" {
        repo::remove_place_from_staff_places(&mut *transaction, id).await?;
    }

    transaction.commit().await?;
    get_place(pool, id).await
}

pub async fn deactivate_place(pool: &SqlitePool, id: i64) -> Result<Place, CommandError> {
    let _existing = repo::get_place_row(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró el lugar solicitado."))?;

    let today = today_iso();
    let future_count = repo::count_future_active_services_for_place(pool, id, &today).await?;
    if future_count > 0 {
        return Err(CommandError::new(
            "blocked",
            format!(
                "No se puede desactivar el lugar porque tiene {future_count} servicio(s) futuro(s) planificados o en progreso. Cancelalos o reasignalos primero."
            ),
        ));
    }

    repo::set_place_active(pool, id, false).await?;
    get_place(pool, id).await
}

pub async fn reactivate_place(pool: &SqlitePool, id: i64) -> Result<Place, CommandError> {
    let affected = repo::set_place_active(pool, id, true).await?;
    if affected == 0 {
        return Err(CommandError::new(
            "not_found",
            "No se encontró el lugar solicitado.",
        ));
    }
    get_place(pool, id).await
}

pub async fn recommended_task(
    pool: &SqlitePool,
    place_id: i64,
    date: &str,
) -> Result<RecommendedTask, CommandError> {
    let row = repo::get_place_row(pool, place_id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró el lugar solicitado."))?;
    let ref_date = parse_date(date, "de referencia")?;
    let app_settings = settings::get_settings(pool).await?;

    let rec = compute_recommended_task(
        &row.place_type,
        row.status.as_deref(),
        row.check_in_date.as_deref(),
        row.check_out_date.as_deref(),
        ref_date,
        app_settings,
    );

    Ok(RecommendedTask {
        effective_status: rec.effective_status,
        recommended_service_type: rec.recommended_service_type,
    })
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

    fn sample_apartment(name: &str) -> PlaceInput {
        PlaceInput {
            name: name.to_string(),
            place_type: "apartment".to_string(),
            status: None,
            location: Some("Piso 1".to_string()),
            check_in_date: Some("2026-10-19".to_string()),
            check_out_date: Some("2026-10-25".to_string()),
            notes: Some("Vista al mar".to_string()),
        }
    }

    #[test]
    fn place_crud_and_case_insensitive_uniqueness() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            let apt = create_place(&pool, sample_apartment("Apto 101"))
                .await
                .expect("create apartment");
            assert_eq!(apt.name, "Apto 101");
            assert!(apt.is_active);

            // Case-insensitive duplicate rejected
            let dup = create_place(&pool, sample_apartment("apto 101"))
                .await
                .expect_err("case-insensitive duplicate");
            assert_eq!(dup.code, "duplicate");

            // Unicode case-insensitive duplicate rejected
            create_place(&pool, sample_apartment("Ático"))
                .await
                .expect("accented place");
            let accented_dup = create_place(&pool, sample_apartment("ático"))
                .await
                .expect_err("unicode case-insensitive duplicate");
            assert_eq!(accented_dup.code, "duplicate");

            // Common area cannot have dates or status
            let bad_common = create_place(
                &pool,
                PlaceInput {
                    name: "Lobby".to_string(),
                    place_type: "common_area".to_string(),
                    status: Some("occupied".to_string()),
                    location: Some("Planta baja".to_string()),
                    check_in_date: None,
                    check_out_date: None,
                    notes: None,
                },
            )
            .await
            .expect_err("common area with status");
            assert_eq!(bad_common.code, "validation");

            // Valid common area
            let lobby = create_place(
                &pool,
                PlaceInput {
                    name: "Lobby".to_string(),
                    place_type: "common_area".to_string(),
                    status: None,
                    location: Some("Planta baja".to_string()),
                    check_in_date: None,
                    check_out_date: None,
                    notes: None,
                },
            )
            .await
            .expect("valid common area");
            assert_eq!(lobby.place_type, "common_area");

            pool.close().await;
        });
    }

    #[test]
    fn recommended_task_and_cancelled_status_and_deactivation_block() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            let apt = create_place(&pool, sample_apartment("Apto 302"))
                .await
                .expect("create apt");

            // Occupied on 2026-10-21 -> daily_cleaning
            let rec = recommended_task(&pool, apt.id, "2026-10-21")
                .await
                .expect("rec occupied");
            assert_eq!(rec.effective_status, "occupied");
            assert_eq!(
                rec.recommended_service_type.as_deref(),
                Some("daily_cleaning")
            );

            // Checkout on 2026-10-25 -> recently_vacated + deep_cleaning on 2026-10-25 and 2026-10-26
            let rec_co = recommended_task(&pool, apt.id, "2026-10-26")
                .await
                .expect("rec checkout");
            assert_eq!(rec_co.effective_status, "recently_vacated");
            assert_eq!(
                rec_co.recommended_service_type.as_deref(),
                Some("deep_cleaning")
            );

            // Override with cancelled -> no recommendation
            let mut cancelled_input = sample_apartment("Apto 302");
            cancelled_input.status = Some("cancelled".to_string());
            let updated = update_place(&pool, apt.id, cancelled_input)
                .await
                .expect("set cancelled");
            assert_eq!(updated.effective_status, "cancelled");
            assert!(updated.recommended_service_type.is_none());
            assert!(updated.cancelled_at.is_some());

            let rec_cancelled = recommended_task(&pool, apt.id, "2026-10-21")
                .await
                .expect("rec cancelled");
            assert_eq!(rec_cancelled.effective_status, "cancelled");
            assert!(rec_cancelled.recommended_service_type.is_none());

            // Clearing the manual status clears the cancellation timestamp
            let mut restored_input = sample_apartment("Apto 302");
            restored_input.status = None;
            let restored = update_place(&pool, apt.id, restored_input)
                .await
                .expect("clear cancelled");
            assert!(restored.cancelled_at.is_none());

            // Future active service blocks deactivation
            sqlx::query("INSERT INTO cleaning_service (id, place_id, date, service_type, status) VALUES (1, ?1, '2099-12-31', 'daily_cleaning', 'planned')")
                .bind(apt.id)
                .execute(&pool)
                .await
                .expect("future service");

            let blocked = deactivate_place(&pool, apt.id)
                .await
                .expect_err("blocked by future service");
            assert_eq!(blocked.code, "blocked");

            // Cancel future service -> deactivation succeeds
            sqlx::query("UPDATE cleaning_service SET status = 'cancelled' WHERE id = 1")
                .execute(&pool)
                .await
                .expect("cancel service");

            let deactivated = deactivate_place(&pool, apt.id)
                .await
                .expect("deactivate place");
            assert!(!deactivated.is_active);

            let reactivated = reactivate_place(&pool, apt.id)
                .await
                .expect("reactivate place");
            assert!(reactivated.is_active);

            pool.close().await;
        });
    }
}
