use sqlx::SqlitePool;

use super::model::PlaceRow;

const PLACE_SELECT: &str = "
    SELECT
        id,
        name,
        type,
        status,
        location,
        check_in_date,
        check_out_date,
        is_active,
        cancelled_at,
        notes,
        created_at,
        updated_at
    FROM place
";

pub async fn list_place_rows(pool: &SqlitePool) -> Result<Vec<PlaceRow>, sqlx::Error> {
    let query = format!(
        "{PLACE_SELECT} ORDER BY is_active DESC, type ASC, name COLLATE NOCASE ASC, id ASC"
    );
    sqlx::query_as::<_, PlaceRow>(&query).fetch_all(pool).await
}

pub async fn get_place_row(pool: &SqlitePool, id: i64) -> Result<Option<PlaceRow>, sqlx::Error> {
    let query = format!("{PLACE_SELECT} WHERE id = ?1");
    sqlx::query_as::<_, PlaceRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_name_nocase(
    pool: &SqlitePool,
    name: &str,
) -> Result<Option<PlaceRow>, sqlx::Error> {
    let query = format!("{PLACE_SELECT} WHERE name = ?1 COLLATE NOCASE");
    sqlx::query_as::<_, PlaceRow>(&query)
        .bind(name)
        .fetch_optional(pool)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_place(
    pool: &SqlitePool,
    name: &str,
    place_type: &str,
    status: Option<&str>,
    location: Option<&str>,
    check_in_date: Option<&str>,
    check_out_date: Option<&str>,
    notes: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO place (
            name,
            type,
            status,
            location,
            check_in_date,
            check_out_date,
            notes,
            cancelled_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7,
            CASE WHEN ?3 = 'cancelled' THEN datetime('now') ELSE NULL END
        )",
    )
    .bind(name)
    .bind(place_type)
    .bind(status)
    .bind(location)
    .bind(check_in_date)
    .bind(check_out_date)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_place(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    place_type: &str,
    status: Option<&str>,
    location: Option<&str>,
    check_in_date: Option<&str>,
    check_out_date: Option<&str>,
    notes: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE place SET
            name = ?1,
            type = ?2,
            status = ?3,
            location = ?4,
            check_in_date = ?5,
            check_out_date = ?6,
            notes = ?7,
            cancelled_at = CASE
                WHEN ?3 = 'cancelled'
                     AND (status IS NULL OR status != 'cancelled')
                    THEN datetime('now')
                WHEN ?3 IS NULL OR ?3 != 'cancelled'
                    THEN NULL
                ELSE cancelled_at
            END,
            updated_at = datetime('now')
         WHERE id = ?8",
    )
    .bind(name)
    .bind(place_type)
    .bind(status)
    .bind(location)
    .bind(check_in_date)
    .bind(check_out_date)
    .bind(notes)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn remove_place_from_staff_places(
    pool: &SqlitePool,
    place_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM staff_place WHERE place_id = ?1")
        .bind(place_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_place_active(
    pool: &SqlitePool,
    id: i64,
    is_active: bool,
) -> Result<u64, sqlx::Error> {
    let result =
        sqlx::query("UPDATE place SET is_active = ?1, updated_at = datetime('now') WHERE id = ?2")
            .bind(i64::from(is_active))
            .bind(id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

pub async fn count_future_active_services_for_place(
    pool: &SqlitePool,
    place_id: i64,
    from_date: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM cleaning_service
         WHERE place_id = ?1
           AND date >= ?2
           AND status IN ('planned', 'in_progress')",
    )
    .bind(place_id)
    .bind(from_date)
    .fetch_one(pool)
    .await
}
