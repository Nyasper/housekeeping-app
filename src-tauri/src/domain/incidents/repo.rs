use sqlx::{Executor, Sqlite, SqlitePool};

use super::model::{IncidentListFilters, IncidentRow};
use crate::error::CommandError;

pub async fn list_incident_rows(
    pool: &SqlitePool,
    filters: &IncidentListFilters,
) -> Result<Vec<IncidentRow>, CommandError> {
    let rows = sqlx::query_as::<_, IncidentRow>(
        "SELECT
            i.id,
            i.service_id,
            cs.place_id,
            p.name AS place_name,
            cs.service_type,
            i.type,
            i.description,
            i.photo_path,
            i.status,
            i.date,
            i.resolved_at,
            i.created_at
         FROM incident i
         INNER JOIN cleaning_service cs ON cs.id = i.service_id
         INNER JOIN place p ON p.id = cs.place_id
         WHERE (?1 IS NULL OR i.status = ?1)
           AND (?2 IS NULL OR i.type = ?2)
           AND (?3 IS NULL OR i.date = ?3)
           AND (?4 IS NULL OR i.service_id = ?4)
         ORDER BY
            CASE WHEN i.status = 'open' THEN 0 ELSE 1 END ASC,
            i.date DESC,
            i.id DESC",
    )
    .bind(filters.status.as_deref())
    .bind(filters.incident_type.as_deref())
    .bind(filters.date.as_deref())
    .bind(filters.service_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_incident_row(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<IncidentRow>, CommandError> {
    let row = sqlx::query_as::<_, IncidentRow>(
        "SELECT
            i.id,
            i.service_id,
            cs.place_id,
            p.name AS place_name,
            cs.service_type,
            i.type,
            i.description,
            i.photo_path,
            i.status,
            i.date,
            i.resolved_at,
            i.created_at
         FROM incident i
         INNER JOIN cleaning_service cs ON cs.id = i.service_id
         INNER JOIN place p ON p.id = cs.place_id
         WHERE i.id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn insert_incident<'e, E>(
    executor: E,
    service_id: i64,
    incident_type: &str,
    description: &str,
    date: &str,
) -> Result<i64, CommandError>
where
    E: Executor<'e, Database = Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO incident (service_id, type, description, status, date)
         VALUES (?1, ?2, ?3, 'open', ?4)",
    )
    .bind(service_id)
    .bind(incident_type)
    .bind(description)
    .bind(date)
    .execute(executor)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update_incident_photo_path<'e, E>(
    executor: E,
    id: i64,
    photo_path: Option<&str>,
) -> Result<(), CommandError>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query("UPDATE incident SET photo_path = ?1 WHERE id = ?2")
        .bind(photo_path)
        .bind(id)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn update_incident_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
) -> Result<u64, CommandError> {
    let result = if status == "resolved" {
        sqlx::query(
            "UPDATE incident
             SET status = 'resolved', resolved_at = datetime('now')
             WHERE id = ?1",
        )
        .bind(id)
        .execute(pool)
        .await?
    } else {
        sqlx::query(
            "UPDATE incident
             SET status = 'open', resolved_at = NULL
             WHERE id = ?1",
        )
        .bind(id)
        .execute(pool)
        .await?
    };

    Ok(result.rows_affected())
}
