use std::collections::HashMap;

use sqlx::SqlitePool;

use super::model::{Absence, PlaceSummary, StaffRow};

const STAFF_SELECT: &str = "
    SELECT
        id,
        first_name,
        last_name,
        birth_date,
        phone,
        national_id,
        address,
        photo_path,
        gender,
        role,
        is_seasonal,
        season_start_date,
        season_end_date,
        is_active,
        notes,
        created_at,
        updated_at
    FROM staff
";

pub async fn list_staff_rows(pool: &SqlitePool) -> Result<Vec<StaffRow>, sqlx::Error> {
    let query = format!(
        "{STAFF_SELECT} ORDER BY is_active DESC, last_name COLLATE NOCASE ASC, first_name COLLATE NOCASE ASC, id ASC"
    );
    sqlx::query_as::<_, StaffRow>(&query).fetch_all(pool).await
}

pub async fn get_staff_row(pool: &SqlitePool, id: i64) -> Result<Option<StaffRow>, sqlx::Error> {
    let query = format!("{STAFF_SELECT} WHERE id = ?1");
    sqlx::query_as::<_, StaffRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_national_id(
    pool: &SqlitePool,
    national_id: &str,
) -> Result<Option<StaffRow>, sqlx::Error> {
    let query = format!("{STAFF_SELECT} WHERE national_id = ?1");
    sqlx::query_as::<_, StaffRow>(&query)
        .bind(national_id)
        .fetch_optional(pool)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_staff(
    pool: &SqlitePool,
    first_name: &str,
    last_name: &str,
    birth_date: Option<&str>,
    phone: Option<&str>,
    national_id: &str,
    address: Option<&str>,
    gender: &str,
    role: &str,
    is_seasonal: bool,
    season_start_date: Option<&str>,
    season_end_date: Option<&str>,
    notes: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO staff (
            first_name,
            last_name,
            birth_date,
            phone,
            national_id,
            address,
            gender,
            role,
            is_seasonal,
            season_start_date,
            season_end_date,
            notes
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )
    .bind(first_name)
    .bind(last_name)
    .bind(birth_date)
    .bind(phone)
    .bind(national_id)
    .bind(address)
    .bind(gender)
    .bind(role)
    .bind(i64::from(is_seasonal))
    .bind(season_start_date)
    .bind(season_end_date)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_staff(
    pool: &SqlitePool,
    id: i64,
    first_name: &str,
    last_name: &str,
    birth_date: Option<&str>,
    phone: Option<&str>,
    national_id: &str,
    address: Option<&str>,
    gender: &str,
    role: &str,
    is_seasonal: bool,
    season_start_date: Option<&str>,
    season_end_date: Option<&str>,
    notes: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE staff SET
            first_name = ?1,
            last_name = ?2,
            birth_date = ?3,
            phone = ?4,
            national_id = ?5,
            address = ?6,
            gender = ?7,
            role = ?8,
            is_seasonal = ?9,
            season_start_date = ?10,
            season_end_date = ?11,
            notes = ?12,
            updated_at = datetime('now')
         WHERE id = ?13",
    )
    .bind(first_name)
    .bind(last_name)
    .bind(birth_date)
    .bind(phone)
    .bind(national_id)
    .bind(address)
    .bind(gender)
    .bind(role)
    .bind(i64::from(is_seasonal))
    .bind(season_start_date)
    .bind(season_end_date)
    .bind(notes)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn update_photo_path(
    pool: &SqlitePool,
    id: i64,
    photo_path: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE staff SET photo_path = ?1, updated_at = datetime('now') WHERE id = ?2")
        .bind(photo_path)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_staff_active(
    pool: &SqlitePool,
    id: i64,
    is_active: bool,
) -> Result<u64, sqlx::Error> {
    let result =
        sqlx::query("UPDATE staff SET is_active = ?1, updated_at = datetime('now') WHERE id = ?2")
            .bind(i64::from(is_active))
            .bind(id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

/// Returns `(service_id, assigned_staff_count)` for every future (`date >= from_date`)
/// `planned` or `in_progress` service where `staff_id` is assigned.
pub async fn list_future_active_service_counts(
    pool: &SqlitePool,
    staff_id: i64,
    from_date: &str,
) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64)>(
        "SELECT
            cs.id,
            (SELECT COUNT(*) FROM service_staff ss2 WHERE ss2.service_id = cs.id) AS total_staff
         FROM cleaning_service cs
         INNER JOIN service_staff ss ON ss.service_id = cs.id
         WHERE ss.staff_id = ?1
           AND cs.date >= ?2
           AND cs.status IN ('planned', 'in_progress')
         ORDER BY cs.date ASC, cs.id ASC",
    )
    .bind(staff_id)
    .bind(from_date)
    .fetch_all(pool)
    .await
}

pub async fn get_rest_days_for_staff(
    pool: &SqlitePool,
    staff_id: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT weekday FROM staff_rest_day WHERE staff_id = ?1 ORDER BY weekday ASC",
    )
    .bind(staff_id)
    .fetch_all(pool)
    .await
}

pub async fn get_all_rest_days_map(
    pool: &SqlitePool,
) -> Result<HashMap<i64, Vec<i64>>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64)>(
        "SELECT staff_id, weekday FROM staff_rest_day ORDER BY staff_id ASC, weekday ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<i64, Vec<i64>> = HashMap::new();
    for (staff_id, weekday) in rows {
        map.entry(staff_id).or_default().push(weekday);
    }
    Ok(map)
}

pub async fn replace_rest_days(
    pool: &SqlitePool,
    staff_id: i64,
    weekdays: &[i64],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM staff_rest_day WHERE staff_id = ?1")
        .bind(staff_id)
        .execute(&mut *tx)
        .await?;

    for weekday in weekdays {
        sqlx::query("INSERT INTO staff_rest_day (staff_id, weekday) VALUES (?1, ?2)")
            .bind(staff_id)
            .bind(*weekday)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await
}

pub async fn get_main_places_for_staff(
    pool: &SqlitePool,
    staff_id: i64,
) -> Result<Vec<PlaceSummary>, sqlx::Error> {
    sqlx::query_as::<_, PlaceSummary>(
        "SELECT p.id, p.name, p.type, p.location
         FROM staff_place sp
         INNER JOIN place p ON p.id = sp.place_id
         WHERE sp.staff_id = ?1
           AND p.is_active = 1
           AND p.type = 'apartment'
         ORDER BY p.name COLLATE NOCASE ASC, p.id ASC",
    )
    .bind(staff_id)
    .fetch_all(pool)
    .await
}

pub async fn get_all_main_places_map(
    pool: &SqlitePool,
) -> Result<HashMap<i64, Vec<PlaceSummary>>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct StaffPlaceJoin {
        staff_id: i64,
        id: i64,
        name: String,
        #[sqlx(rename = "type")]
        place_type: String,
        location: Option<String>,
    }

    let rows = sqlx::query_as::<_, StaffPlaceJoin>(
        "SELECT sp.staff_id, p.id, p.name, p.type, p.location
         FROM staff_place sp
         INNER JOIN place p ON p.id = sp.place_id
         WHERE p.is_active = 1
           AND p.type = 'apartment'
         ORDER BY sp.staff_id ASC, p.name COLLATE NOCASE ASC, p.id ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<i64, Vec<PlaceSummary>> = HashMap::new();
    for row in rows {
        map.entry(row.staff_id).or_default().push(PlaceSummary {
            id: row.id,
            name: row.name,
            place_type: row.place_type,
            location: row.location,
        });
    }
    Ok(map)
}

pub async fn list_active_apartments(pool: &SqlitePool) -> Result<Vec<PlaceSummary>, sqlx::Error> {
    sqlx::query_as::<_, PlaceSummary>(
        "SELECT id, name, type, location
         FROM place
         WHERE is_active = 1 AND type = 'apartment'
         ORDER BY name COLLATE NOCASE ASC, id ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn count_valid_active_apartments(
    pool: &SqlitePool,
    place_ids: &[i64],
) -> Result<usize, sqlx::Error> {
    if place_ids.is_empty() {
        return Ok(0);
    }
    let mut valid_count = 0usize;
    for place_id in place_ids {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM place WHERE id = ?1 AND type = 'apartment' AND is_active = 1",
        )
        .bind(*place_id)
        .fetch_one(pool)
        .await?;
        if count > 0 {
            valid_count += 1;
        }
    }
    Ok(valid_count)
}

pub async fn replace_staff_places(
    pool: &SqlitePool,
    staff_id: i64,
    place_ids: &[i64],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM staff_place WHERE staff_id = ?1")
        .bind(staff_id)
        .execute(&mut *tx)
        .await?;

    for place_id in place_ids {
        sqlx::query("INSERT INTO staff_place (staff_id, place_id) VALUES (?1, ?2)")
            .bind(staff_id)
            .bind(*place_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await
}

pub async fn list_absences(
    pool: &SqlitePool,
    staff_id: Option<i64>,
) -> Result<Vec<Absence>, sqlx::Error> {
    if let Some(staff_id) = staff_id {
        sqlx::query_as::<_, Absence>(
            "SELECT id, staff_id, type, start_date, end_date, reason, created_at
             FROM absence
             WHERE staff_id = ?1
             ORDER BY start_date DESC, id DESC",
        )
        .bind(staff_id)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Absence>(
            "SELECT id, staff_id, type, start_date, end_date, reason, created_at
             FROM absence
             ORDER BY start_date DESC, id DESC",
        )
        .fetch_all(pool)
        .await
    }
}

pub async fn get_absence(pool: &SqlitePool, id: i64) -> Result<Option<Absence>, sqlx::Error> {
    sqlx::query_as::<_, Absence>(
        "SELECT id, staff_id, type, start_date, end_date, reason, created_at
         FROM absence
         WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn has_overlapping_absence(
    pool: &SqlitePool,
    staff_id: i64,
    start_date: &str,
    end_date: &str,
    exclude_id: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let count: i64 = if let Some(exclude_id) = exclude_id {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM absence
             WHERE staff_id = ?1
               AND id != ?2
               AND start_date <= ?4
               AND end_date >= ?3",
        )
        .bind(staff_id)
        .bind(exclude_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM absence
             WHERE staff_id = ?1
               AND start_date <= ?3
               AND end_date >= ?2",
        )
        .bind(staff_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?
    };
    Ok(count > 0)
}

pub async fn insert_absence(
    pool: &SqlitePool,
    staff_id: i64,
    absence_type: &str,
    start_date: &str,
    end_date: &str,
    reason: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO absence (staff_id, type, start_date, end_date, reason)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(staff_id)
    .bind(absence_type)
    .bind(start_date)
    .bind(end_date)
    .bind(reason)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update_absence(
    pool: &SqlitePool,
    id: i64,
    staff_id: i64,
    absence_type: &str,
    start_date: &str,
    end_date: &str,
    reason: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE absence SET
            staff_id = ?1,
            type = ?2,
            start_date = ?3,
            end_date = ?4,
            reason = ?5
         WHERE id = ?6",
    )
    .bind(staff_id)
    .bind(absence_type)
    .bind(start_date)
    .bind(end_date)
    .bind(reason)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_absence(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM absence WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
