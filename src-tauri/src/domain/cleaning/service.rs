use std::collections::{HashMap, HashSet};
use std::path::Path;

use chrono::{Duration, NaiveDate};
use sqlx::SqlitePool;

use super::model::{
    CleaningService, CleaningServiceInput, CleaningServiceRow, ServiceStaff, ServiceStaffInput,
    ServiceStaffRow, StaffSuggestion,
};
use super::repo;
use crate::domain::places::repo as places_repo;
use crate::domain::staff::{model::Staff, repo as staff_repo, service as staff_service};
use crate::error::CommandError;
use crate::util::availability::{
    CandidateEvalInput, is_staff_available_on, rank_candidates, times_overlap,
};
use crate::util::dates::{normalize_optional_time, parse_date, parse_time};
use crate::util::files::resolve_photo_url;

const VALID_SERVICE_TYPES: [&str; 4] = ["glass", "daily_cleaning", "deep_cleaning", "touch_up"];
const VALID_SERVICE_STATUSES: [&str; 4] = ["planned", "in_progress", "completed", "cancelled"];
pub const DEFAULT_SUGGEST_COUNT: usize = 10;

fn trim_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn map_staff_row(row: ServiceStaffRow, data_dir: Option<&Path>) -> ServiceStaff {
    let photo_url = resolve_photo_url(data_dir, row.photo_path.as_deref());
    ServiceStaff {
        staff_id: row.staff_id,
        first_name: row.first_name,
        last_name: row.last_name,
        photo_url,
        gender: row.gender,
        is_lead: row.is_lead != 0,
        is_active: row.is_active != 0,
    }
}

fn map_service_row(
    row: CleaningServiceRow,
    staff_rows: Vec<ServiceStaffRow>,
    data_dir: Option<&Path>,
    warning: Option<String>,
) -> CleaningService {
    CleaningService {
        id: row.id,
        place_id: row.place_id,
        place_name: row.place_name,
        date: row.date,
        start_time: row.start_time,
        end_time: row.end_time,
        service_type: row.service_type,
        status: row.status,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
        staff: staff_rows
            .into_iter()
            .map(|r| map_staff_row(r, data_dir))
            .collect(),
        warning,
    }
}

fn validate_staff_inputs(staff: &[ServiceStaffInput]) -> Result<(), CommandError> {
    if staff.is_empty() {
        return Err(CommandError::new(
            "validation",
            "Un servicio debe tener al menos un funcionario asignado.",
        ));
    }

    let mut seen = HashSet::new();
    let mut lead_count = 0usize;
    for item in staff {
        if !seen.insert(item.staff_id) {
            return Err(CommandError::new(
                "validation",
                "No se puede asignar dos veces al mismo funcionario en el mismo servicio.",
            ));
        }
        if item.is_lead {
            lead_count += 1;
        }
    }

    if lead_count > 1 {
        return Err(CommandError::new(
            "validation",
            "Solo puede haber un responsable principal por servicio.",
        ));
    }

    Ok(())
}

fn validate_times_and_status(
    status: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
    staff_count: usize,
) -> Result<(), CommandError> {
    if !VALID_SERVICE_STATUSES.contains(&status) {
        return Err(CommandError::new(
            "validation",
            "El estado del servicio no es válido.",
        ));
    }

    if staff_count == 0 {
        return Err(CommandError::new(
            "validation",
            "Un servicio debe tener al menos un funcionario asignado.",
        ));
    }

    if let (Some(st), Some(et)) = (start_time, end_time) {
        let start_parsed = parse_time(st, "inicio")?;
        let end_parsed = parse_time(et, "finalización")?;
        if start_parsed >= end_parsed {
            return Err(CommandError::new(
                "validation",
                "La hora de inicio debe ser anterior a la hora de finalización.",
            ));
        }
    }

    match status {
        "in_progress" if start_time.is_none() => {
            return Err(CommandError::new(
                "validation",
                "Para marcar un servicio en progreso debés indicar la hora de inicio.",
            ));
        }
        "completed" if start_time.is_none() || end_time.is_none() => {
            return Err(CommandError::new(
                "validation",
                "Para completar un servicio debés indicar la hora de inicio y la de finalización.",
            ));
        }
        _ => {}
    }

    Ok(())
}

fn validate_status_transition(
    current_status: &str,
    next_status: &str,
    confirm_reopen: bool,
) -> Result<(), CommandError> {
    if current_status == "completed" && !confirm_reopen {
        return Err(CommandError::new(
            "blocked",
            "El servicio ya está completado. Confirmá explícitamente si querés reabrirlo o modificarlo.",
        ));
    }

    if current_status == next_status {
        return Ok(());
    }

    let allowed = match current_status {
        "planned" => matches!(next_status, "in_progress" | "completed" | "cancelled"),
        "in_progress" => matches!(next_status, "completed" | "cancelled" | "planned"),
        "completed" => confirm_reopen,
        "cancelled" => next_status == "planned" && confirm_reopen,
        _ => false,
    };

    if !allowed {
        return Err(CommandError::new(
            "validation",
            "La transición de estado solicitada no es válida para este servicio.",
        ));
    }

    Ok(())
}

async fn check_double_assignment_warnings(
    pool: &SqlitePool,
    date: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
    staff: &[ServiceStaffInput],
    exclude_service_id: Option<i64>,
) -> Result<Option<String>, CommandError> {
    let mut warnings = Vec::new();

    for entry in staff {
        let Some(staff_row) = staff_repo::get_staff_row(pool, entry.staff_id).await? else {
            return Err(CommandError::new(
                "not_found",
                "Uno de los funcionarios seleccionados no existe.",
            ));
        };

        let others = repo::list_other_services_for_staff_on_date(
            pool,
            entry.staff_id,
            date,
            exclude_service_id,
        )
        .await?;

        if others.is_empty() {
            continue;
        }

        let full_name = format!("{} {}", staff_row.first_name, staff_row.last_name);
        let has_complete_times = start_time.is_some() && end_time.is_some();
        let mut overlapped = false;
        let mut other_without_times = false;

        for (_, other_st, other_et) in &others {
            match (other_st.as_deref(), other_et.as_deref()) {
                (Some(ost), Some(oet)) => {
                    if let (Some(st), Some(et)) = (start_time, end_time)
                        && times_overlap(st, et, ost, oet)
                    {
                        overlapped = true;
                        break;
                    }
                }
                _ => other_without_times = true,
            }
        }

        if overlapped {
            warnings.push(format!(
                "{full_name} tiene otro servicio con horario superpuesto ese día"
            ));
        } else if !has_complete_times || other_without_times {
            warnings.push(format!(
                "{full_name} ya tiene {} servicio(s) asignado(s) ese día",
                others.len()
            ));
        }
    }

    if warnings.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!(
            "Aviso de asignación: {}.",
            warnings.join("; ")
        )))
    }
}

pub async fn list_services_by_date(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    date: &str,
) -> Result<Vec<CleaningService>, CommandError> {
    let parsed_date = parse_date(date, "del tablero")?;
    let iso_date = parsed_date.format("%Y-%m-%d").to_string();

    let rows = repo::list_services_by_date_rows(pool, &iso_date).await?;
    let mut staff_map = repo::get_service_staff_by_date_map(pool, &iso_date).await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let staff_rows = staff_map.remove(&row.id).unwrap_or_default();
            map_service_row(row, staff_rows, data_dir, None)
        })
        .collect())
}

pub async fn get_service(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
) -> Result<CleaningService, CommandError> {
    let row = repo::get_service_row(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró el servicio solicitado."))?;
    let staff_rows = repo::get_service_staff_for_service(pool, id).await?;
    Ok(map_service_row(row, staff_rows, data_dir, None))
}

pub async fn create_service(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    input: CleaningServiceInput,
) -> Result<CleaningService, CommandError> {
    let service_type = input.service_type.trim().to_string();
    if !VALID_SERVICE_TYPES.contains(&service_type.as_str()) {
        return Err(CommandError::new(
            "validation",
            "El tipo de servicio seleccionado no es válido.",
        ));
    }

    let status = input.status.trim().to_string();
    let parsed_date = parse_date(&input.date, "del servicio")?;
    let iso_date = parsed_date.format("%Y-%m-%d").to_string();
    let start_time = normalize_optional_time(input.start_time.as_deref(), "inicio")?;
    let end_time = normalize_optional_time(input.end_time.as_deref(), "finalización")?;
    let notes = trim_optional(input.notes.as_deref());

    validate_staff_inputs(&input.staff)?;
    validate_times_and_status(
        &status,
        start_time.as_deref(),
        end_time.as_deref(),
        input.staff.len(),
    )?;

    let place = places_repo::get_place_row(pool, input.place_id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "El lugar seleccionado no existe."))?;
    if place.is_active == 0 {
        return Err(CommandError::new(
            "validation",
            "No se puede crear un servicio en un lugar desactivado.",
        ));
    }

    let warning = check_double_assignment_warnings(
        pool,
        &iso_date,
        start_time.as_deref(),
        end_time.as_deref(),
        &input.staff,
        None,
    )
    .await?;

    let id = repo::insert_service_with_staff(
        pool,
        input.place_id,
        &iso_date,
        start_time.as_deref(),
        end_time.as_deref(),
        &service_type,
        &status,
        notes.as_deref(),
        &input.staff,
    )
    .await?;

    let mut created = get_service(pool, data_dir, id).await?;
    created.warning = warning;
    Ok(created)
}

pub async fn update_service(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
    input: CleaningServiceInput,
    confirm_reopen: bool,
) -> Result<CleaningService, CommandError> {
    let existing = repo::get_service_row(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró el servicio solicitado."))?;

    let service_type = input.service_type.trim().to_string();
    if !VALID_SERVICE_TYPES.contains(&service_type.as_str()) {
        return Err(CommandError::new(
            "validation",
            "El tipo de servicio seleccionado no es válido.",
        ));
    }

    let status = input.status.trim().to_string();
    validate_status_transition(&existing.status, &status, confirm_reopen)?;

    let parsed_date = parse_date(&input.date, "del servicio")?;
    let iso_date = parsed_date.format("%Y-%m-%d").to_string();
    let start_time = normalize_optional_time(input.start_time.as_deref(), "inicio")?;
    let end_time = normalize_optional_time(input.end_time.as_deref(), "finalización")?;
    let notes = trim_optional(input.notes.as_deref());

    validate_staff_inputs(&input.staff)?;
    validate_times_and_status(
        &status,
        start_time.as_deref(),
        end_time.as_deref(),
        input.staff.len(),
    )?;

    let _place = places_repo::get_place_row(pool, input.place_id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "El lugar seleccionado no existe."))?;

    let warning = check_double_assignment_warnings(
        pool,
        &iso_date,
        start_time.as_deref(),
        end_time.as_deref(),
        &input.staff,
        Some(id),
    )
    .await?;

    repo::update_service_with_staff(
        pool,
        id,
        input.place_id,
        &iso_date,
        start_time.as_deref(),
        end_time.as_deref(),
        &service_type,
        &status,
        notes.as_deref(),
        &input.staff,
    )
    .await?;

    let mut updated = get_service(pool, data_dir, id).await?;
    updated.warning = warning;
    Ok(updated)
}

pub async fn set_service_status(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
    status: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
    confirm_reopen: bool,
) -> Result<CleaningService, CommandError> {
    let existing = repo::get_service_row(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró el servicio solicitado."))?;
    let staff_rows = repo::get_service_staff_for_service(pool, id).await?;

    let next_status = status.trim();
    validate_status_transition(&existing.status, next_status, confirm_reopen)?;

    let normalized_start = match start_time {
        Some(raw) => normalize_optional_time(Some(raw), "inicio")?,
        None => existing.start_time.clone(),
    };
    let normalized_end = match end_time {
        Some(raw) => normalize_optional_time(Some(raw), "finalización")?,
        None => existing.end_time.clone(),
    };

    validate_times_and_status(
        next_status,
        normalized_start.as_deref(),
        normalized_end.as_deref(),
        staff_rows.len(),
    )?;

    repo::update_service_status_and_times(
        pool,
        id,
        next_status,
        normalized_start.as_deref(),
        normalized_end.as_deref(),
    )
    .await?;

    get_service(pool, data_dir, id).await
}

pub async fn set_service_staff(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    id: i64,
    staff: Vec<ServiceStaffInput>,
) -> Result<CleaningService, CommandError> {
    let existing = repo::get_service_row(pool, id)
        .await?
        .ok_or_else(|| CommandError::new("not_found", "No se encontró el servicio solicitado."))?;

    validate_staff_inputs(&staff)?;

    let warning = check_double_assignment_warnings(
        pool,
        &existing.date,
        existing.start_time.as_deref(),
        existing.end_time.as_deref(),
        &staff,
        Some(id),
    )
    .await?;

    repo::replace_service_staff(pool, id, &staff).await?;

    let mut updated = get_service(pool, data_dir, id).await?;
    updated.warning = warning;
    Ok(updated)
}

#[allow(clippy::too_many_arguments)]
pub async fn suggest_staff(
    pool: &SqlitePool,
    data_dir: Option<&Path>,
    place_id: i64,
    service_type: &str,
    date: &str,
    count: Option<usize>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<StaffSuggestion>, CommandError> {
    let st_trimmed = service_type.trim();
    if !VALID_SERVICE_TYPES.contains(&st_trimmed) {
        return Err(CommandError::new(
            "validation",
            "El tipo de servicio seleccionado no es válido.",
        ));
    }

    let ref_date: NaiveDate = parse_date(date, "del servicio")?;
    let iso_date = ref_date.format("%Y-%m-%d").to_string();
    let window_start = (ref_date - Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    let norm_start = normalize_optional_time(start_time, "inicio")?;
    let norm_end = normalize_optional_time(end_time, "finalización")?;

    let all_staff = staff_service::list_staff(pool, data_dir).await?;
    let all_absences = staff_repo::list_absences(pool, None).await?;
    let recent_counts = repo::get_recent_service_counts_map(pool, &window_start, &iso_date).await?;

    let mut absence_map: HashMap<i64, Vec<(NaiveDate, NaiveDate)>> = HashMap::new();
    for abs in all_absences {
        if let (Ok(s), Ok(e)) = (
            parse_date(&abs.start_date, "inicio ausencia"),
            parse_date(&abs.end_date, "fin ausencia"),
        ) {
            absence_map.entry(abs.staff_id).or_default().push((s, e));
        }
    }

    let mut staff_by_id: HashMap<i64, Staff> = HashMap::new();
    let mut eval_inputs = Vec::new();

    for member in all_staff {
        let abs_ranges = absence_map
            .get(&member.id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        if !is_staff_available_on(
            member.is_active,
            member.is_seasonal,
            member.season_start_date.as_deref(),
            member.season_end_date.as_deref(),
            &member.rest_days,
            abs_ranges,
            ref_date,
        ) {
            continue;
        }

        let same_day =
            repo::list_other_services_for_staff_on_date(pool, member.id, &iso_date, None).await?;

        let mut has_time_overlap = false;
        if let (Some(st), Some(et)) = (norm_start.as_deref(), norm_end.as_deref()) {
            for (_, ost, oet) in &same_day {
                if let (Some(o_start), Some(o_end)) = (ost.as_deref(), oet.as_deref())
                    && times_overlap(st, et, o_start, o_end)
                {
                    has_time_overlap = true;
                    break;
                }
            }
        }

        let is_main_place = member.main_places.iter().any(|p| p.id == place_id);
        let recent_service_count = recent_counts.get(&member.id).copied().unwrap_or(0);

        eval_inputs.push(CandidateEvalInput {
            staff_id: member.id,
            role: member.role.clone(),
            is_main_place,
            recent_service_count,
            same_day_service_count: same_day.len(),
            has_time_overlap,
        });

        staff_by_id.insert(member.id, member);
    }

    let limit = count.unwrap_or(DEFAULT_SUGGEST_COUNT);
    let ranked = rank_candidates(eval_inputs, st_trimmed, limit);

    Ok(ranked
        .into_iter()
        .filter_map(|rc| {
            staff_by_id
                .remove(&rc.staff_id)
                .map(|staff| StaffSuggestion {
                    staff,
                    reasons: rc.reasons,
                })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::db;
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

    fn make_staff_input(first: &str, last: &str, national_id: &str, role: &str) -> StaffInput {
        StaffInput {
            first_name: first.to_string(),
            last_name: last.to_string(),
            birth_date: None,
            phone: None,
            national_id: national_id.to_string(),
            address: None,
            gender: "female".to_string(),
            role: role.to_string(),
            is_seasonal: false,
            season_start_date: None,
            season_end_date: None,
            notes: None,
            photo_source_path: None,
            remove_photo: false,
        }
    }

    #[test]
    fn service_crud_mandatory_staff_and_transitions() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            let apt = places_service::create_place(
                &pool,
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

            let s1 = staff_service::create_staff(
                &pool,
                None,
                make_staff_input("Ana", "Silva", "50000001", "maid"),
            )
            .await
            .expect("s1");
            let s2 = staff_service::create_staff(
                &pool,
                None,
                make_staff_input("Lucía", "Pérez", "50000002", "maid"),
            )
            .await
            .expect("s2");

            // Empty staff rejected
            let empty_err = create_service(
                &pool,
                None,
                CleaningServiceInput {
                    place_id: apt.id,
                    date: "2026-10-21".to_string(),
                    start_time: None,
                    end_time: None,
                    service_type: "daily_cleaning".to_string(),
                    status: "planned".to_string(),
                    notes: None,
                    staff: vec![],
                },
            )
            .await
            .expect_err("empty staff");
            assert_eq!(empty_err.code, "validation");

            // Two leads rejected
            let two_leads_err = create_service(
                &pool,
                None,
                CleaningServiceInput {
                    place_id: apt.id,
                    date: "2026-10-21".to_string(),
                    start_time: None,
                    end_time: None,
                    service_type: "daily_cleaning".to_string(),
                    status: "planned".to_string(),
                    notes: None,
                    staff: vec![
                        ServiceStaffInput {
                            staff_id: s1.id,
                            is_lead: true,
                        },
                        ServiceStaffInput {
                            staff_id: s2.id,
                            is_lead: true,
                        },
                    ],
                },
            )
            .await
            .expect_err("two leads");
            assert_eq!(two_leads_err.code, "validation");

            // Valid planned service with 1 staff
            let service = create_service(
                &pool,
                None,
                CleaningServiceInput {
                    place_id: apt.id,
                    date: "2026-10-21".to_string(),
                    start_time: None,
                    end_time: None,
                    service_type: "daily_cleaning".to_string(),
                    status: "planned".to_string(),
                    notes: Some("Prioridad".to_string()),
                    staff: vec![ServiceStaffInput {
                        staff_id: s1.id,
                        is_lead: true,
                    }],
                },
            )
            .await
            .expect("create service");
            assert_eq!(service.staff.len(), 1);
            assert_eq!(service.staff[0].staff_id, s1.id);

            // Replace sole staff member s1 with s2 (must not trip trg_service_staff_keep_last)
            let reassigned = set_service_staff(
                &pool,
                None,
                service.id,
                vec![ServiceStaffInput {
                    staff_id: s2.id,
                    is_lead: true,
                }],
            )
            .await
            .expect("reassign sole staff");
            assert_eq!(reassigned.staff.len(), 1);
            assert_eq!(reassigned.staff[0].staff_id, s2.id);

            // Transition to in_progress requires start_time
            let no_start_err =
                set_service_status(&pool, None, service.id, "in_progress", None, None, false)
                    .await
                    .expect_err("in_progress without start_time");
            assert_eq!(no_start_err.code, "validation");

            let in_prog = set_service_status(
                &pool,
                None,
                service.id,
                "in_progress",
                Some("09:00"),
                None,
                false,
            )
            .await
            .expect("in_progress");
            assert_eq!(in_prog.status, "in_progress");
            assert_eq!(in_prog.start_time.as_deref(), Some("09:00"));

            // Transition to completed requires end_time > start_time
            let bad_end_err = set_service_status(
                &pool,
                None,
                service.id,
                "completed",
                Some("09:00"),
                Some("08:30"),
                false,
            )
            .await
            .expect_err("end_time < start_time");
            assert_eq!(bad_end_err.code, "validation");

            let completed = set_service_status(
                &pool,
                None,
                service.id,
                "completed",
                Some("09:00"),
                Some("10:15"),
                false,
            )
            .await
            .expect("completed");
            assert_eq!(completed.status, "completed");

            // Reopening completed without confirmation is blocked
            let reopen_blocked =
                set_service_status(&pool, None, service.id, "in_progress", None, None, false)
                    .await
                    .expect_err("completed terminal without confirmation");
            assert_eq!(reopen_blocked.code, "blocked");

            // Reopening with confirmation succeeds
            let reopened =
                set_service_status(&pool, None, service.id, "in_progress", None, None, true)
                    .await
                    .expect("reopen with confirmation");
            assert_eq!(reopened.status, "in_progress");

            pool.close().await;
        });
    }

    #[test]
    fn double_assignment_warnings_only_for_overlaps_or_missing_times() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            let apt = places_service::create_place(
                &pool,
                PlaceInput {
                    name: "Apto 402".to_string(),
                    place_type: "apartment".to_string(),
                    status: None,
                    location: None,
                    check_in_date: None,
                    check_out_date: None,
                    notes: None,
                },
            )
            .await
            .expect("place");

            let staff = staff_service::create_staff(
                &pool,
                None,
                make_staff_input("Marta", "López", "50000003", "maid"),
            )
            .await
            .expect("staff");

            let make_input = |start: Option<&str>, end: Option<&str>| CleaningServiceInput {
                place_id: apt.id,
                date: "2026-10-21".to_string(),
                start_time: start.map(ToString::to_string),
                end_time: end.map(ToString::to_string),
                service_type: "daily_cleaning".to_string(),
                status: "planned".to_string(),
                notes: None,
                staff: vec![ServiceStaffInput {
                    staff_id: staff.id,
                    is_lead: true,
                }],
            };

            let first = create_service(&pool, None, make_input(Some("09:00"), Some("10:00")))
                .await
                .expect("first service");
            assert!(first.warning.is_none());

            let non_overlapping =
                create_service(&pool, None, make_input(Some("14:00"), Some("15:00")))
                    .await
                    .expect("non-overlapping service");
            assert!(non_overlapping.warning.is_none());

            let overlapping = create_service(&pool, None, make_input(Some("09:30"), Some("10:30")))
                .await
                .expect("overlapping service");
            assert!(
                overlapping
                    .warning
                    .as_deref()
                    .unwrap_or_default()
                    .contains("horario superpuesto")
            );

            let without_times = create_service(&pool, None, make_input(None, None))
                .await
                .expect("service without times");
            assert!(
                without_times
                    .warning
                    .as_deref()
                    .unwrap_or_default()
                    .contains("ya tiene")
            );

            let after_missing_times =
                create_service(&pool, None, make_input(Some("16:00"), Some("17:00")))
                    .await
                    .expect("service after missing times");
            assert!(
                after_missing_times
                    .warning
                    .as_deref()
                    .unwrap_or_default()
                    .contains("ya tiene")
            );

            pool.close().await;
        });
    }

    #[test]
    fn assisted_staff_suggestions_respect_availability_and_ranking() {
        tauri::async_runtime::block_on(async {
            let pool = test_pool().await;

            let apt = places_service::create_place(
                &pool,
                PlaceInput {
                    name: "Apto 501".to_string(),
                    place_type: "apartment".to_string(),
                    status: None,
                    location: None,
                    check_in_date: None,
                    check_out_date: None,
                    notes: None,
                },
            )
            .await
            .expect("apt");

            // Staff 1: Maid, assigned to Apto 501 as main apartment
            let s_main = staff_service::create_staff(
                &pool,
                None,
                make_staff_input("Carla", "Méndez", "60000001", "maid"),
            )
            .await
            .expect("s_main");
            staff_service::set_staff_places(&pool, None, s_main.id, vec![apt.id])
                .await
                .expect("set main place");

            // Staff 2: Maid, not main apartment
            let s_other = staff_service::create_staff(
                &pool,
                None,
                make_staff_input("Elena", "Ríos", "60000002", "maid"),
            )
            .await
            .expect("s_other");

            // Staff 3: Maid on absence on 2026-10-21 -> excluded
            let s_absent = staff_service::create_staff(
                &pool,
                None,
                make_staff_input("Sofía", "Castro", "60000003", "maid"),
            )
            .await
            .expect("s_absent");
            staff_service::create_absence(
                &pool,
                AbsenceInput {
                    staff_id: s_absent.id,
                    absence_type: "leave".to_string(),
                    start_date: "2026-10-20".to_string(),
                    end_date: "2026-10-22".to_string(),
                    reason: None,
                },
            )
            .await
            .expect("absence");

            // Staff 4: Window cleaner
            let s_glass = staff_service::create_staff(
                &pool,
                None,
                make_staff_input("Diego", "Vega", "60000004", "window_cleaner"),
            )
            .await
            .expect("s_glass");

            let suggestions = suggest_staff(
                &pool,
                None,
                apt.id,
                "daily_cleaning",
                "2026-10-21",
                Some(10),
                None,
                None,
            )
            .await
            .expect("suggestions");

            let suggested_ids: Vec<i64> = suggestions.iter().map(|s| s.staff.id).collect();
            // s_absent is excluded; s_main comes first (role + main place), then s_other (role), then s_glass
            assert_eq!(suggested_ids, vec![s_main.id, s_other.id, s_glass.id]);

            pool.close().await;
        });
    }
}
