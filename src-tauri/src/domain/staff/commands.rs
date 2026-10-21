use tauri::{AppHandle, State};

use super::model::{Absence, AbsenceInput, PlaceSummary, Staff, StaffInput};
use super::service;
use crate::db;
use crate::error::CommandError;
use crate::state::DbState;

#[tauri::command]
pub async fn staff_list(
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Vec<Staff>, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::list_staff(&pool, data_dir.as_deref()).await
}

#[tauri::command]
pub async fn staff_get(
    id: i64,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Staff, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::get_staff(&pool, data_dir.as_deref(), id).await
}

#[tauri::command]
pub async fn staff_create(
    input: StaffInput,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Staff, CommandError> {
    let data_dir = db::data_dir(&app)?;
    let pool = state.pool().read().await.clone();
    service::create_staff(&pool, Some(&data_dir), input).await
}

#[tauri::command]
pub async fn staff_update(
    id: i64,
    input: StaffInput,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Staff, CommandError> {
    let data_dir = db::data_dir(&app)?;
    let pool = state.pool().read().await.clone();
    service::update_staff(&pool, Some(&data_dir), id, input).await
}

#[tauri::command]
pub async fn staff_deactivate(
    id: i64,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Staff, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::deactivate_staff(&pool, data_dir.as_deref(), id).await
}

#[tauri::command]
pub async fn staff_reactivate(
    id: i64,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Staff, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::reactivate_staff(&pool, data_dir.as_deref(), id).await
}

#[tauri::command]
pub async fn staff_available_apartments(
    state: State<'_, DbState>,
) -> Result<Vec<PlaceSummary>, CommandError> {
    let pool = state.pool().read().await.clone();
    service::list_available_apartments(&pool).await
}

#[tauri::command]
pub async fn staff_places_set(
    staff_id: i64,
    place_ids: Vec<i64>,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Staff, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::set_staff_places(&pool, data_dir.as_deref(), staff_id, place_ids).await
}

#[tauri::command]
pub async fn staff_rest_days_get(
    staff_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<i64>, CommandError> {
    let pool = state.pool().read().await.clone();
    service::get_rest_days(&pool, staff_id).await
}

#[tauri::command]
pub async fn staff_rest_days_set(
    staff_id: i64,
    weekdays: Vec<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<i64>, CommandError> {
    let pool = state.pool().read().await.clone();
    service::set_rest_days(&pool, staff_id, weekdays).await
}

#[tauri::command]
pub async fn staff_absences_list(
    staff_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<Absence>, CommandError> {
    let pool = state.pool().read().await.clone();
    service::list_absences(&pool, staff_id).await
}

#[tauri::command]
pub async fn staff_absences_create(
    input: AbsenceInput,
    state: State<'_, DbState>,
) -> Result<Absence, CommandError> {
    let pool = state.pool().read().await.clone();
    service::create_absence(&pool, input).await
}

#[tauri::command]
pub async fn staff_absences_update(
    id: i64,
    input: AbsenceInput,
    state: State<'_, DbState>,
) -> Result<Absence, CommandError> {
    let pool = state.pool().read().await.clone();
    service::update_absence(&pool, id, input).await
}

#[tauri::command]
pub async fn staff_absences_delete(id: i64, state: State<'_, DbState>) -> Result<(), CommandError> {
    let pool = state.pool().read().await.clone();
    service::delete_absence(&pool, id).await
}
