use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::error::CommandError;
use crate::util::dates::{parse_date, parse_time, validate_date_range};

const SERVICE_TYPES_ORDER: [&str; 4] = ["daily_cleaning", "deep_cleaning", "touch_up", "glass"];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DateRange {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServicesByStaffRow {
    pub staff_id: i64,
    pub staff_name: String,
    pub service_count: i64,
    pub total_hours: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AverageTimesRow {
    pub service_type: String,
    pub average_minutes: f64,
    pub completed_services: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsencesByMonthRow {
    pub staff_id: i64,
    pub staff_name: String,
    #[serde(rename = "type")]
    pub absence_type: String,
    pub days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailySheetRow {
    pub service_id: i64,
    pub place_id: i64,
    pub place_name: String,
    pub place_location: Option<String>,
    pub service_type: String,
    pub status: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub duration_minutes: Option<i64>,
    pub staff_names: Vec<String>,
    pub incidents_count: i64,
    pub open_incidents_count: i64,
    pub incidents_summary: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, FromRow)]
struct StaffServiceDbRow {
    staff_id: i64,
    first_name: String,
    last_name: String,
    service_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
}

#[derive(Debug, FromRow)]
struct CompletedServiceTimeDbRow {
    service_type: String,
    start_time: String,
    end_time: String,
}

#[derive(Debug, FromRow)]
struct AbsenceDbRow {
    staff_id: i64,
    first_name: String,
    last_name: String,
    absence_type: String,
    start_date: String,
    end_date: String,
}

#[derive(Debug, FromRow)]
struct DailyServiceDbRow {
    service_id: i64,
    place_id: i64,
    place_name: String,
    place_location: Option<String>,
    service_type: String,
    status: String,
    start_time: Option<String>,
    end_time: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, FromRow)]
struct DailyStaffDbRow {
    service_id: i64,
    first_name: String,
    last_name: String,
    is_lead: i64,
}

#[derive(Debug, FromRow)]
struct DailyIncidentDbRow {
    service_id: i64,
    incident_type: String,
    description: String,
    status: String,
}

/// Computes the duration in minutes between two `HH:MM` timestamps when `end > start`.
pub fn duration_minutes(start_time: Option<&str>, end_time: Option<&str>) -> Option<i64> {
    let start_str = start_time?;
    let end_str = end_time?;
    let start = parse_time(start_str, "inicio").ok()?;
    let end = parse_time(end_str, "fin").ok()?;
    let diff = (end - start).num_minutes();
    if diff > 0 { Some(diff) } else { None }
}

/// Parses a `YYYY-MM` month string and returns its inclusive `(first_day, last_day)`.
pub fn parse_month_bounds(month: &str) -> Result<(NaiveDate, NaiveDate), CommandError> {
    let trimmed = month.trim();
    let parts: Vec<&str> = trimmed.split('-').collect();
    if parts.len() != 2 || parts[0].len() != 4 || parts[1].len() != 2 {
        return Err(CommandError::new(
            "validation",
            "El mes seleccionado no es válido (formato esperado: AAAA-MM).",
        ));
    }

    let first_day_str = format!("{trimmed}-01");
    let first_day = NaiveDate::parse_from_str(&first_day_str, "%Y-%m-%d").map_err(|_| {
        CommandError::new(
            "validation",
            "El mes seleccionado no es válido (formato esperado: AAAA-MM).",
        )
    })?;

    let (next_year, next_month) = if first_day.month() == 12 {
        (first_day.year() + 1, 1)
    } else {
        (first_day.year(), first_day.month() + 1)
    };

    let next_month_first = NaiveDate::from_ymd_opt(next_year, next_month, 1).ok_or_else(|| {
        CommandError::new(
            "validation",
            "No se pudo calcular el fin del mes seleccionado.",
        )
    })?;
    let last_day = next_month_first.pred_opt().ok_or_else(|| {
        CommandError::new(
            "validation",
            "No se pudo calcular el fin del mes seleccionado.",
        )
    })?;

    Ok((first_day, last_day))
}

fn incident_type_label_es(code: &str) -> &str {
    match code {
        "damage" => "Daño",
        "lost_item" => "Objeto olvidado",
        "supply_shortage" => "Faltante de insumos",
        _ => code,
    }
}

fn incident_status_label_es(code: &str) -> &str {
    match code {
        "open" => "Abierta",
        "resolved" => "Resuelta",
        _ => code,
    }
}

/// Report 1: Services and total hours per staff member in `[from, to]`.
/// Excludes `cancelled` services. Includes all active staff plus any inactive
/// staff who performed at least one non-cancelled service in the range.
pub async fn services_by_staff(
    pool: &SqlitePool,
    range: &DateRange,
) -> Result<Vec<ServicesByStaffRow>, CommandError> {
    let (start_date, end_date) = validate_date_range(&range.from, &range.to, "reporte")?;
    let from_iso = start_date.format("%Y-%m-%d").to_string();
    let to_iso = end_date.format("%Y-%m-%d").to_string();

    let rows = sqlx::query_as::<_, StaffServiceDbRow>(
        "SELECT
            s.id AS staff_id,
            s.first_name,
            s.last_name,
            cs.id AS service_id,
            cs.start_time,
            cs.end_time
         FROM staff s
         LEFT JOIN service_staff ss ON ss.staff_id = s.id
         LEFT JOIN cleaning_service cs
             ON cs.id = ss.service_id
            AND cs.date >= ?1
            AND cs.date <= ?2
            AND cs.status != 'cancelled'
         WHERE s.is_active = 1 OR cs.id IS NOT NULL
         ORDER BY s.last_name COLLATE NOCASE ASC, s.first_name COLLATE NOCASE ASC, s.id ASC",
    )
    .bind(&from_iso)
    .bind(&to_iso)
    .fetch_all(pool)
    .await?;

    struct Accum {
        staff_id: i64,
        staff_name: String,
        service_count: i64,
        total_minutes: i64,
    }

    let mut order: Vec<i64> = Vec::new();
    let mut map: HashMap<i64, Accum> = HashMap::new();

    for row in rows {
        let entry = map.entry(row.staff_id).or_insert_with(|| {
            order.push(row.staff_id);
            Accum {
                staff_id: row.staff_id,
                staff_name: format!("{} {}", row.first_name, row.last_name),
                service_count: 0,
                total_minutes: 0,
            }
        });

        if row.service_id.is_some() {
            entry.service_count += 1;
            if let Some(mins) = duration_minutes(row.start_time.as_deref(), row.end_time.as_deref())
            {
                entry.total_minutes += mins;
            }
        }
    }

    let mut result: Vec<(i64, ServicesByStaffRow)> = order
        .into_iter()
        .filter_map(|id| map.remove(&id))
        .map(|acc| {
            let hours = ((acc.total_minutes as f64 / 60.0) * 100.0).round() / 100.0;
            (
                acc.total_minutes,
                ServicesByStaffRow {
                    staff_id: acc.staff_id,
                    staff_name: acc.staff_name,
                    service_count: acc.service_count,
                    total_hours: hours,
                },
            )
        })
        .collect();

    result.sort_by(|(mins_a, row_a), (mins_b, row_b)| {
        row_b
            .service_count
            .cmp(&row_a.service_count)
            .then_with(|| mins_b.cmp(mins_a))
            .then_with(|| row_a.staff_name.cmp(&row_b.staff_name))
            .then_with(|| row_a.staff_id.cmp(&row_b.staff_id))
    });

    Ok(result.into_iter().map(|(_, row)| row).collect())
}

/// Report 2: Average duration in minutes per service type in `[from, to]`.
/// Only considers `completed` services that have both `start_time` and `end_time`.
pub async fn average_times(
    pool: &SqlitePool,
    range: &DateRange,
) -> Result<Vec<AverageTimesRow>, CommandError> {
    let (start_date, end_date) = validate_date_range(&range.from, &range.to, "reporte")?;
    let from_iso = start_date.format("%Y-%m-%d").to_string();
    let to_iso = end_date.format("%Y-%m-%d").to_string();

    let rows = sqlx::query_as::<_, CompletedServiceTimeDbRow>(
        "SELECT
            service_type,
            start_time,
            end_time
         FROM cleaning_service
         WHERE date >= ?1
           AND date <= ?2
           AND status = 'completed'
           AND start_time IS NOT NULL
           AND end_time IS NOT NULL",
    )
    .bind(&from_iso)
    .bind(&to_iso)
    .fetch_all(pool)
    .await?;

    let mut totals: HashMap<String, (i64, i64)> = HashMap::new();
    for row in rows {
        if let Some(mins) = duration_minutes(Some(&row.start_time), Some(&row.end_time)) {
            let entry = totals.entry(row.service_type).or_insert((0, 0));
            entry.0 += mins;
            entry.1 += 1;
        }
    }

    let result = SERVICE_TYPES_ORDER
        .iter()
        .map(|&service_type| {
            let (sum_mins, count) = totals.get(service_type).copied().unwrap_or((0, 0));
            let avg = if count > 0 {
                ((sum_mins as f64 / count as f64) * 10.0).round() / 10.0
            } else {
                0.0
            };
            AverageTimesRow {
                service_type: service_type.to_string(),
                average_minutes: avg,
                completed_services: count,
            }
        })
        .collect();

    Ok(result)
}

/// Report 3: Absence days per staff member and absence type within the given month (`YYYY-MM`).
/// Ranges crossing month boundaries are prorated by counting only the days inside the month.
pub async fn absences_by_month(
    pool: &SqlitePool,
    month: &str,
) -> Result<Vec<AbsencesByMonthRow>, CommandError> {
    let (month_start, month_end) = parse_month_bounds(month)?;
    let month_start_iso = month_start.format("%Y-%m-%d").to_string();
    let month_end_iso = month_end.format("%Y-%m-%d").to_string();

    let rows = sqlx::query_as::<_, AbsenceDbRow>(
        "SELECT
            a.staff_id,
            s.first_name,
            s.last_name,
            a.type AS absence_type,
            a.start_date,
            a.end_date
         FROM absence a
         INNER JOIN staff s ON s.id = a.staff_id
         WHERE a.start_date <= ?2
           AND a.end_date >= ?1
         ORDER BY s.last_name COLLATE NOCASE ASC, s.first_name COLLATE NOCASE ASC, s.id ASC, a.type ASC",
    )
    .bind(&month_start_iso)
    .bind(&month_end_iso)
    .fetch_all(pool)
    .await?;

    let mut order: Vec<(i64, String)> = Vec::new();
    let mut map: HashMap<(i64, String), AbsencesByMonthRow> = HashMap::new();

    for row in rows {
        let start = parse_date(&row.start_date, "inicio de ausencia")?;
        let end = parse_date(&row.end_date, "fin de ausencia")?;
        let effective_start = start.max(month_start);
        let effective_end = end.min(month_end);
        let days = (effective_end - effective_start).num_days() + 1;
        if days <= 0 {
            continue;
        }

        let key = (row.staff_id, row.absence_type.clone());
        let entry = map.entry(key.clone()).or_insert_with(|| {
            order.push(key);
            AbsencesByMonthRow {
                staff_id: row.staff_id,
                staff_name: format!("{} {}", row.first_name, row.last_name),
                absence_type: row.absence_type,
                days: 0,
            }
        });
        entry.days += days;
    }

    Ok(order
        .into_iter()
        .filter_map(|key| map.remove(&key))
        .collect())
}

/// Report 4: Daily sheet for a specific date (`YYYY-MM-DD`).
/// Includes place, location, service type, assigned staff, hours, status, and incidents.
pub async fn daily_sheet(
    pool: &SqlitePool,
    date: &str,
) -> Result<Vec<DailySheetRow>, CommandError> {
    let parsed_date = parse_date(date, "planilla diaria")?;
    let date_iso = parsed_date.format("%Y-%m-%d").to_string();

    let services = sqlx::query_as::<_, DailyServiceDbRow>(
        "SELECT
            cs.id AS service_id,
            cs.place_id,
            p.name AS place_name,
            p.location AS place_location,
            cs.service_type,
            cs.status,
            cs.start_time,
            cs.end_time,
            cs.notes
         FROM cleaning_service cs
         INNER JOIN place p ON p.id = cs.place_id
         WHERE cs.date = ?1
         ORDER BY
            CASE WHEN cs.status = 'cancelled' THEN 1 ELSE 0 END ASC,
            COALESCE(cs.start_time, '99:99') ASC,
            p.name COLLATE NOCASE ASC,
            cs.id ASC",
    )
    .bind(&date_iso)
    .fetch_all(pool)
    .await?;

    let staff_rows = sqlx::query_as::<_, DailyStaffDbRow>(
        "SELECT
            ss.service_id,
            s.first_name,
            s.last_name,
            ss.is_lead
         FROM service_staff ss
         INNER JOIN cleaning_service cs ON cs.id = ss.service_id
         INNER JOIN staff s ON s.id = ss.staff_id
         WHERE cs.date = ?1
         ORDER BY ss.service_id ASC, ss.is_lead DESC, s.last_name COLLATE NOCASE ASC, s.first_name COLLATE NOCASE ASC, s.id ASC",
    )
    .bind(&date_iso)
    .fetch_all(pool)
    .await?;

    let mut staff_by_service: HashMap<i64, Vec<String>> = HashMap::new();
    for s in staff_rows {
        let label = if s.is_lead != 0 {
            format!("{} {} (Responsable)", s.first_name, s.last_name)
        } else {
            format!("{} {}", s.first_name, s.last_name)
        };
        staff_by_service
            .entry(s.service_id)
            .or_default()
            .push(label);
    }

    let incident_rows = sqlx::query_as::<_, DailyIncidentDbRow>(
        "SELECT
            i.service_id,
            i.type AS incident_type,
            i.description,
            i.status
         FROM incident i
         INNER JOIN cleaning_service cs ON cs.id = i.service_id
         WHERE cs.date = ?1
         ORDER BY
            CASE WHEN i.status = 'open' THEN 0 ELSE 1 END ASC,
            i.id ASC",
    )
    .bind(&date_iso)
    .fetch_all(pool)
    .await?;

    let mut incidents_by_service: HashMap<i64, Vec<DailyIncidentDbRow>> = HashMap::new();
    for inc in incident_rows {
        incidents_by_service
            .entry(inc.service_id)
            .or_default()
            .push(inc);
    }

    let result = services
        .into_iter()
        .map(|svc| {
            let duration = duration_minutes(svc.start_time.as_deref(), svc.end_time.as_deref());
            let staff_names = staff_by_service.remove(&svc.service_id).unwrap_or_default();
            let incs = incidents_by_service
                .remove(&svc.service_id)
                .unwrap_or_default();
            let incidents_count = incs.len() as i64;
            let open_incidents_count = incs.iter().filter(|i| i.status == "open").count() as i64;
            let incidents_summary = incs
                .into_iter()
                .map(|i| {
                    format!(
                        "{}: {} ({})",
                        incident_type_label_es(&i.incident_type),
                        i.description,
                        incident_status_label_es(&i.status)
                    )
                })
                .collect();

            DailySheetRow {
                service_id: svc.service_id,
                place_id: svc.place_id,
                place_name: svc.place_name,
                place_location: svc.place_location,
                service_type: svc.service_type,
                status: svc.status,
                start_time: svc.start_time,
                end_time: svc.end_time,
                duration_minutes: duration,
                staff_names,
                incidents_count,
                open_incidents_count,
                incidents_summary,
                notes: svc.notes,
            }
        })
        .collect();

    Ok(result)
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
    use crate::domain::incidents::{model::IncidentInput, service as incidents_service};
    use crate::domain::places::{model::PlaceInput, service as places_service};
    use crate::domain::staff::{
        model::{AbsenceInput, StaffInput},
        service as staff_service,
    };

    async fn test_pool() -> SqlitePool {
        let pool = db::connect(Path::new(":memory:"))
            .await
            .expect("in-memory db");
        db::migrate(&pool).await.expect("migrations");
        pool
    }

    #[test]
    fn services_by_staff_and_average_times_exclude_cancelled_and_compute_metrics() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            let place = places_service::create_place(
                &pool,
                PlaceInput {
                    name: "Apto 501".to_string(),
                    place_type: "apartment".to_string(),
                    status: None,
                    location: Some("Piso 5".to_string()),
                    check_in_date: None,
                    check_out_date: None,
                    notes: None,
                },
            )
            .await
            .expect("place");

            let maid_a = staff_service::create_staff(
                &pool,
                None,
                StaffInput {
                    first_name: "Ana".to_string(),
                    last_name: "Silva".to_string(),
                    birth_date: None,
                    phone: None,
                    national_id: "50011112".to_string(),
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
            .expect("maid a");

            let maid_b = staff_service::create_staff(
                &pool,
                None,
                StaffInput {
                    first_name: "Beatriz".to_string(),
                    last_name: "Luna".to_string(),
                    birth_date: None,
                    phone: None,
                    national_id: "50022223".to_string(),
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
            .expect("maid b");

            // Completed daily cleaning: 90 minutes (1.5 hours) for Ana
            let svc1 = cleaning_service::create_service(
                &pool,
                None,
                CleaningServiceInput {
                    place_id: place.id,
                    date: "2026-10-10".to_string(),
                    start_time: Some("08:00".to_string()),
                    end_time: Some("09:30".to_string()),
                    service_type: "daily_cleaning".to_string(),
                    status: "completed".to_string(),
                    notes: Some("Todo en orden".to_string()),
                    staff: vec![ServiceStaffInput {
                        staff_id: maid_a.id,
                        is_lead: true,
                    }],
                },
            )
            .await
            .expect("svc1");

            // Completed daily cleaning: 30 minutes (0.5 hours) for Ana & Beatriz
            cleaning_service::create_service(
                &pool,
                None,
                CleaningServiceInput {
                    place_id: place.id,
                    date: "2026-10-12".to_string(),
                    start_time: Some("10:00".to_string()),
                    end_time: Some("10:30".to_string()),
                    service_type: "daily_cleaning".to_string(),
                    status: "completed".to_string(),
                    notes: None,
                    staff: vec![
                        ServiceStaffInput {
                            staff_id: maid_a.id,
                            is_lead: true,
                        },
                        ServiceStaffInput {
                            staff_id: maid_b.id,
                            is_lead: false,
                        },
                    ],
                },
            )
            .await
            .expect("svc2");

            // Cancelled service for Beatriz -> must be excluded from counts and averages
            cleaning_service::create_service(
                &pool,
                None,
                CleaningServiceInput {
                    place_id: place.id,
                    date: "2026-10-15".to_string(),
                    start_time: Some("11:00".to_string()),
                    end_time: Some("13:00".to_string()),
                    service_type: "daily_cleaning".to_string(),
                    status: "cancelled".to_string(),
                    notes: None,
                    staff: vec![ServiceStaffInput {
                        staff_id: maid_b.id,
                        is_lead: true,
                    }],
                },
            )
            .await
            .expect("svc3 cancelled");

            let range = DateRange {
                from: "2026-10-01".to_string(),
                to: "2026-10-31".to_string(),
            };

            let by_staff = services_by_staff(&pool, &range)
                .await
                .expect("services_by_staff");
            assert_eq!(by_staff.len(), 2);
            assert_eq!(by_staff[0].staff_id, maid_a.id);
            assert_eq!(by_staff[0].service_count, 2);
            assert!((by_staff[0].total_hours - 2.0).abs() < f64::EPSILON);

            assert_eq!(by_staff[1].staff_id, maid_b.id);
            assert_eq!(by_staff[1].service_count, 1);
            assert!((by_staff[1].total_hours - 0.5).abs() < f64::EPSILON);

            let avg = average_times(&pool, &range).await.expect("average_times");
            let daily_avg = avg
                .iter()
                .find(|r| r.service_type == "daily_cleaning")
                .expect("daily_cleaning row");
            assert_eq!(daily_avg.completed_services, 2);
            // (90 + 30) / 2 = 60.0 minutes
            assert!((daily_avg.average_minutes - 60.0).abs() < f64::EPSILON);

            // Add an incident to svc1 and check daily_sheet
            incidents_service::create_incident(
                &pool,
                None,
                IncidentInput {
                    service_id: svc1.id,
                    incident_type: "damage".to_string(),
                    description: "Espejo rayado".to_string(),
                    date: "2026-10-10".to_string(),
                    photo_source_path: None,
                },
            )
            .await
            .expect("incident");

            let sheet = daily_sheet(&pool, "2026-10-10").await.expect("daily_sheet");
            assert_eq!(sheet.len(), 1);
            assert_eq!(sheet[0].place_name, "Apto 501");
            assert_eq!(sheet[0].duration_minutes, Some(90));
            assert_eq!(sheet[0].open_incidents_count, 1);
            assert_eq!(sheet[0].staff_names, vec!["Ana Silva (Responsable)"]);

            pool.close().await;
        });
    }

    #[test]
    fn absences_by_month_prorates_cross_month_ranges() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            let staff = staff_service::create_staff(
                &pool,
                None,
                StaffInput {
                    first_name: "Carlos".to_string(),
                    last_name: "Ríos".to_string(),
                    birth_date: None,
                    phone: None,
                    national_id: "60011119".to_string(),
                    address: None,
                    gender: "male".to_string(),
                    role: "window_cleaner".to_string(),
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

            // Absence crossing September -> October: 2026-09-28 to 2026-10-03
            // September has 30 days -> 3 days in Sept (28, 29, 30) and 3 days in Oct (1, 2, 3)
            staff_service::create_absence(
                &pool,
                AbsenceInput {
                    staff_id: staff.id,
                    absence_type: "leave".to_string(),
                    start_date: "2026-09-28".to_string(),
                    end_date: "2026-10-03".to_string(),
                    reason: Some("Vacaciones".to_string()),
                },
            )
            .await
            .expect("absence 1");

            // Another leave in October: 2026-10-20 to 2026-10-21 (2 days)
            staff_service::create_absence(
                &pool,
                AbsenceInput {
                    staff_id: staff.id,
                    absence_type: "leave".to_string(),
                    start_date: "2026-10-20".to_string(),
                    end_date: "2026-10-21".to_string(),
                    reason: None,
                },
            )
            .await
            .expect("absence 2");

            let sept = absences_by_month(&pool, "2026-09")
                .await
                .expect("september");
            assert_eq!(sept.len(), 1);
            assert_eq!(sept[0].days, 3);

            let oct = absences_by_month(&pool, "2026-10").await.expect("october");
            assert_eq!(oct.len(), 1);
            assert_eq!(oct[0].absence_type, "leave");
            assert_eq!(oct[0].days, 5); // 3 + 2

            pool.close().await;
        });
    }
}
