use std::collections::{HashMap, HashSet};

use sqlx::{SqliteConnection, SqlitePool};

use super::model::{CleaningServiceRow, ServiceStaffInput, ServiceStaffRow};

const SERVICE_SELECT: &str = "
    SELECT
        cs.id,
        cs.place_id,
        p.name AS place_name,
        cs.date,
        cs.start_time,
        cs.end_time,
        cs.service_type,
        cs.status,
        cs.notes,
        cs.created_at,
        cs.updated_at
    FROM cleaning_service cs
    INNER JOIN place p ON p.id = cs.place_id
";

pub async fn list_services_by_date_rows(
    pool: &SqlitePool,
    date: &str,
) -> Result<Vec<CleaningServiceRow>, sqlx::Error> {
    let query = format!(
        "{SERVICE_SELECT}
         WHERE cs.date = ?1
         ORDER BY
            CASE cs.status
                WHEN 'in_progress' THEN 0
                WHEN 'planned' THEN 1
                WHEN 'completed' THEN 2
                ELSE 3
            END ASC,
            COALESCE(cs.start_time, '99:99') ASC,
            p.name COLLATE NOCASE ASC,
            cs.id ASC"
    );
    sqlx::query_as::<_, CleaningServiceRow>(&query)
        .bind(date)
        .fetch_all(pool)
        .await
}

pub async fn get_service_row(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<CleaningServiceRow>, sqlx::Error> {
    let query = format!("{SERVICE_SELECT} WHERE cs.id = ?1");
    sqlx::query_as::<_, CleaningServiceRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_service_staff_for_service(
    pool: &SqlitePool,
    service_id: i64,
) -> Result<Vec<ServiceStaffRow>, sqlx::Error> {
    sqlx::query_as::<_, ServiceStaffRow>(
        "SELECT
            ss.service_id,
            ss.staff_id,
            s.first_name,
            s.last_name,
            s.photo_path,
            s.gender,
            ss.is_lead,
            s.is_active
         FROM service_staff ss
         INNER JOIN staff s ON s.id = ss.staff_id
         WHERE ss.service_id = ?1
         ORDER BY ss.is_lead DESC, s.last_name COLLATE NOCASE ASC, s.first_name COLLATE NOCASE ASC, s.id ASC",
    )
    .bind(service_id)
    .fetch_all(pool)
    .await
}

pub async fn get_service_staff_by_date_map(
    pool: &SqlitePool,
    date: &str,
) -> Result<HashMap<i64, Vec<ServiceStaffRow>>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ServiceStaffRow>(
        "SELECT
            ss.service_id,
            ss.staff_id,
            s.first_name,
            s.last_name,
            s.photo_path,
            s.gender,
            ss.is_lead,
            s.is_active
         FROM service_staff ss
         INNER JOIN cleaning_service cs ON cs.id = ss.service_id
         INNER JOIN staff s ON s.id = ss.staff_id
         WHERE cs.date = ?1
         ORDER BY ss.service_id ASC, ss.is_lead DESC, s.last_name COLLATE NOCASE ASC, s.first_name COLLATE NOCASE ASC, s.id ASC",
    )
    .bind(date)
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<i64, Vec<ServiceStaffRow>> = HashMap::new();
    for row in rows {
        map.entry(row.service_id).or_default().push(row);
    }
    Ok(map)
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_service_with_staff(
    pool: &SqlitePool,
    place_id: i64,
    date: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
    service_type: &str,
    status: &str,
    notes: Option<&str>,
    staff: &[ServiceStaffInput],
) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query(
        "INSERT INTO cleaning_service (
            place_id,
            date,
            start_time,
            end_time,
            service_type,
            status,
            notes
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(place_id)
    .bind(date)
    .bind(start_time)
    .bind(end_time)
    .bind(service_type)
    .bind(status)
    .bind(notes)
    .execute(&mut *tx)
    .await?;

    let service_id = result.last_insert_rowid();

    for entry in staff {
        sqlx::query(
            "INSERT INTO service_staff (service_id, staff_id, is_lead) VALUES (?1, ?2, ?3)",
        )
        .bind(service_id)
        .bind(entry.staff_id)
        .bind(i64::from(entry.is_lead))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(service_id)
}

/// Safely replaces the staff list of a service inside an active transaction
/// without violating `trg_service_staff_keep_last` or `idx_service_staff_one_lead`.
pub async fn replace_service_staff_in_tx(
    conn: &mut SqliteConnection,
    service_id: i64,
    staff: &[ServiceStaffInput],
) -> Result<(), sqlx::Error> {
    // 1. Clear lead flag first so unique lead index never collides during upsert.
    sqlx::query("UPDATE service_staff SET is_lead = 0 WHERE service_id = ?1")
        .bind(service_id)
        .execute(&mut *conn)
        .await?;

    // 2. Upsert all desired staff members first so count >= 2 before any removal.
    for entry in staff {
        sqlx::query(
            "INSERT INTO service_staff (service_id, staff_id, is_lead)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(service_id, staff_id) DO UPDATE SET
                 is_lead = excluded.is_lead",
        )
        .bind(service_id)
        .bind(entry.staff_id)
        .bind(i64::from(entry.is_lead))
        .execute(&mut *conn)
        .await?;
    }

    // 3. Remove any previously assigned staff not in the new list.
    let existing_ids =
        sqlx::query_scalar::<_, i64>("SELECT staff_id FROM service_staff WHERE service_id = ?1")
            .bind(service_id)
            .fetch_all(&mut *conn)
            .await?;

    let keep_ids: HashSet<i64> = staff.iter().map(|s| s.staff_id).collect();
    for existing_id in existing_ids {
        if !keep_ids.contains(&existing_id) {
            sqlx::query("DELETE FROM service_staff WHERE service_id = ?1 AND staff_id = ?2")
                .bind(service_id)
                .bind(existing_id)
                .execute(&mut *conn)
                .await?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_service_with_staff(
    pool: &SqlitePool,
    id: i64,
    place_id: i64,
    date: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
    service_type: &str,
    status: &str,
    notes: Option<&str>,
    staff: &[ServiceStaffInput],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE cleaning_service SET
            place_id = ?1,
            date = ?2,
            start_time = ?3,
            end_time = ?4,
            service_type = ?5,
            status = ?6,
            notes = ?7,
            updated_at = datetime('now')
         WHERE id = ?8",
    )
    .bind(place_id)
    .bind(date)
    .bind(start_time)
    .bind(end_time)
    .bind(service_type)
    .bind(status)
    .bind(notes)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    replace_service_staff_in_tx(&mut tx, id, staff).await?;

    tx.commit().await
}

pub async fn replace_service_staff(
    pool: &SqlitePool,
    service_id: i64,
    staff: &[ServiceStaffInput],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    replace_service_staff_in_tx(&mut tx, service_id, staff).await?;
    sqlx::query("UPDATE cleaning_service SET updated_at = datetime('now') WHERE id = ?1")
        .bind(service_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

pub async fn update_service_status_and_times(
    pool: &SqlitePool,
    id: i64,
    status: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE cleaning_service SET
            status = ?1,
            start_time = ?2,
            end_time = ?3,
            updated_at = datetime('now')
         WHERE id = ?4",
    )
    .bind(status)
    .bind(start_time)
    .bind(end_time)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Returns other non-cancelled services on `date` for `staff_id`, excluding `exclude_service_id`.
pub async fn list_other_services_for_staff_on_date(
    pool: &SqlitePool,
    staff_id: i64,
    date: &str,
    exclude_service_id: Option<i64>,
) -> Result<Vec<(i64, Option<String>, Option<String>)>, sqlx::Error> {
    if let Some(exclude_id) = exclude_service_id {
        sqlx::query_as::<_, (i64, Option<String>, Option<String>)>(
            "SELECT cs.id, cs.start_time, cs.end_time
             FROM cleaning_service cs
             INNER JOIN service_staff ss ON ss.service_id = cs.id
             WHERE ss.staff_id = ?1
               AND cs.date = ?2
               AND cs.id != ?3
               AND cs.status != 'cancelled'",
        )
        .bind(staff_id)
        .bind(date)
        .bind(exclude_id)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, (i64, Option<String>, Option<String>)>(
            "SELECT cs.id, cs.start_time, cs.end_time
             FROM cleaning_service cs
             INNER JOIN service_staff ss ON ss.service_id = cs.id
             WHERE ss.staff_id = ?1
               AND cs.date = ?2
               AND cs.status != 'cancelled'",
        )
        .bind(staff_id)
        .bind(date)
        .fetch_all(pool)
        .await
    }
}

/// Counts non-cancelled services for each staff member within `[from_date, to_date]`.
pub async fn get_recent_service_counts_map(
    pool: &SqlitePool,
    from_date: &str,
    to_date: &str,
) -> Result<HashMap<i64, i64>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64)>(
        "SELECT ss.staff_id, COUNT(*) AS cnt
         FROM service_staff ss
         INNER JOIN cleaning_service cs ON cs.id = ss.service_id
         WHERE cs.date >= ?1
           AND cs.date <= ?2
           AND cs.status != 'cancelled'
         GROUP BY ss.staff_id",
    )
    .bind(from_date)
    .bind(to_date)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().collect())
}
