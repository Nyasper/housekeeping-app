use std::path::Path;

use sqlx::SqlitePool;

use super::model::{Incident, IncidentInput, IncidentListFilters, IncidentRow};
use super::repo;
use crate::domain::cleaning::repo as cleaning_repo;
use crate::error::CommandError;
use crate::util::dates::parse_date;
use crate::util::files::{resolve_photo_url, store_entity_photo};

const VALID_INCIDENT_TYPES: [&str; 3] = ["damage", "lost_item", "supply_shortage"];
const VALID_INCIDENT_STATUSES: [&str; 2] = ["open", "resolved"];

fn trim_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn map_row_to_incident(row: IncidentRow, data_dir: Option<&Path>) -> Incident {
    let photo_url = resolve_photo_url(data_dir, row.photo_path.as_deref());
    Incident {
        id: row.id,
        service_id: row.service_id,
        place_id: row.place_id,
        place_name: row.place_name,
        service_type: row.service_type,
        incident_type: row.incident_type,
        description: row.description,
        photo_path: row.photo_path,
        photo_url,
        status: row.status,
        date: row.date,
        resolved_at: row.resolved_at,
        created_at: row.created_at,
    }
}

pub async fn list_incidents(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    filters: IncidentListFilters,
) -> Result<Vec<Incident>, CommandError> {
    let normalized_status = trim_optional(filters.status.as_deref());
    if let Some(ref st) = normalized_status
        && !VALID_INCIDENT_STATUSES.contains(&st.as_str())
    {
        return Err(CommandError::new(
            "validation",
            "El estado de incidencia seleccionado no es válido.",
        ));
    }

    let normalized_type = trim_optional(filters.incident_type.as_deref());
    if let Some(ref tp) = normalized_type
        && !VALID_INCIDENT_TYPES.contains(&tp.as_str())
    {
        return Err(CommandError::new(
            "validation",
            "El tipo de incidencia seleccionado no es válido.",
        ));
    }

    let normalized_date = trim_optional(filters.date.as_deref());
    if let Some(ref d) = normalized_date {
        parse_date(d, "incidencia")?;
    }

    let normalized_filters = IncidentListFilters {
        status: normalized_status,
        incident_type: normalized_type,
        date: normalized_date,
        service_id: filters.service_id,
    };

    let rows = repo::list_incident_rows(pool, &normalized_filters).await?;
    Ok(rows
        .into_iter()
        .map(|row| map_row_to_incident(row, data_dir))
        .collect())
}

pub async fn get_incident(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
) -> Result<Incident, CommandError> {
    let row = repo::get_incident_row(pool, id).await?.ok_or_else(|| {
        CommandError::new("not_found", "No se encontró la incidencia solicitada.")
    })?;
    Ok(map_row_to_incident(row, data_dir))
}

pub async fn create_incident(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    input: IncidentInput,
) -> Result<Incident, CommandError> {
    let incident_type = input.incident_type.trim().to_string();
    if !VALID_INCIDENT_TYPES.contains(&incident_type.as_str()) {
        return Err(CommandError::new(
            "validation",
            "El tipo de incidencia seleccionado no es válido.",
        ));
    }

    let description = input.description.trim().to_string();
    if description.is_empty() {
        return Err(CommandError::new(
            "validation",
            "La descripción de la incidencia es obligatoria.",
        ));
    }

    let date = input.date.trim().to_string();
    parse_date(&date, "incidencia")?;

    let service = cleaning_repo::get_service_row(pool, input.service_id)
        .await?
        .ok_or_else(|| {
            CommandError::new(
                "not_found",
                "El servicio de limpieza seleccionado no existe.",
            )
        })?;

    if service.status == "cancelled" {
        return Err(CommandError::new(
            "validation",
            "No se pueden registrar incidencias sobre un servicio cancelado.",
        ));
    }

    let id =
        repo::insert_incident(pool, input.service_id, &incident_type, &description, &date).await?;

    if let (Some(base_dir), Some(source_raw)) =
        (data_dir, trim_optional(input.photo_source_path.as_deref()))
    {
        let rel_path = store_entity_photo(base_dir, "incidents", id, Path::new(&source_raw), None)?;
        repo::update_incident_photo_path(pool, id, Some(&rel_path)).await?;
    }

    get_incident(pool, data_dir, id).await
}

pub async fn set_incident_status(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
    status: &str,
) -> Result<Incident, CommandError> {
    let normalized_status = status.trim();
    if !VALID_INCIDENT_STATUSES.contains(&normalized_status) {
        return Err(CommandError::new(
            "validation",
            "El estado de la incidencia no es válido.",
        ));
    }

    let affected = repo::update_incident_status(pool, id, normalized_status).await?;
    if affected == 0 {
        return Err(CommandError::new(
            "not_found",
            "No se encontró la incidencia solicitada.",
        ));
    }

    get_incident(pool, data_dir, id).await
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::db;
    use crate::domain::cleaning::{
        model::{CleaningServiceInput, ServiceStaffInput},
        service as cleaning_service,
    };
    use crate::domain::places::{model::PlaceInput, service as places_service};
    use crate::domain::staff::{model::StaffInput, service as staff_service};

    async fn test_pool() -> SqlitePool {
        let pool = db::connect(Path::new(":memory:"))
            .await
            .expect("in-memory db");
        db::migrate(&pool).await.expect("migrations");
        pool
    }

    async fn seed_service(pool: &SqlitePool) -> i64 {
        let place = places_service::create_place(
            pool,
            PlaceInput {
                name: "Apto 401".to_string(),
                place_type: "apartment".to_string(),
                status: None,
                location: Some("Piso 4".to_string()),
                check_in_date: None,
                check_out_date: None,
                notes: None,
            },
        )
        .await
        .expect("place");

        let staff = staff_service::create_staff(
            pool,
            None,
            StaffInput {
                first_name: "Elena".to_string(),
                last_name: "Gómez".to_string(),
                birth_date: None,
                phone: None,
                national_id: "40001112".to_string(),
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
        .expect("staff");

        let service = cleaning_service::create_service(
            pool,
            None,
            CleaningServiceInput {
                place_id: place.id,
                date: "2026-10-21".to_string(),
                start_time: Some("09:00".to_string()),
                end_time: Some("10:00".to_string()),
                service_type: "daily_cleaning".to_string(),
                status: "completed".to_string(),
                notes: None,
                staff: vec![ServiceStaffInput {
                    staff_id: staff.id,
                    is_lead: true,
                }],
            },
        )
        .await
        .expect("service");

        service.id
    }

    #[test]
    fn creates_lists_resolves_and_rejects_cancelled_service_incidents() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;
            let service_id = seed_service(&pool).await;

            // Empty description rejected
            let empty_err = create_incident(
                &pool,
                None,
                IncidentInput {
                    service_id,
                    incident_type: "damage".to_string(),
                    description: "   ".to_string(),
                    date: "2026-10-21".to_string(),
                    photo_source_path: None,
                },
            )
            .await
            .expect_err("empty description");
            assert_eq!(empty_err.code, "validation");

            // Create valid incident on completed service
            let inc = create_incident(
                &pool,
                None,
                IncidentInput {
                    service_id,
                    incident_type: "damage".to_string(),
                    description: "Lámpara de mesa rota.".to_string(),
                    date: "2026-10-21".to_string(),
                    photo_source_path: None,
                },
            )
            .await
            .expect("create incident");
            assert_eq!(inc.status, "open");
            assert_eq!(inc.place_name, "Apto 401");
            assert!(inc.resolved_at.is_none());

            // Resolve incident
            let resolved = set_incident_status(&pool, None, inc.id, "resolved")
                .await
                .expect("resolve");
            assert_eq!(resolved.status, "resolved");
            assert!(resolved.resolved_at.is_some());

            // Reopen incident
            let reopened = set_incident_status(&pool, None, inc.id, "open")
                .await
                .expect("reopen");
            assert_eq!(reopened.status, "open");
            assert!(reopened.resolved_at.is_none());

            // Cancel service -> new incident rejected
            sqlx::query("UPDATE cleaning_service SET status = 'cancelled' WHERE id = ?1")
                .bind(service_id)
                .execute(&pool)
                .await
                .expect("cancel service");

            let cancelled_err = create_incident(
                &pool,
                None,
                IncidentInput {
                    service_id,
                    incident_type: "lost_item".to_string(),
                    description: "Cargador olvidado.".to_string(),
                    date: "2026-10-21".to_string(),
                    photo_source_path: None,
                },
            )
            .await
            .expect_err("cancelled service");
            assert_eq!(cancelled_err.code, "validation");

            pool.close().await;
        });
    }
}
