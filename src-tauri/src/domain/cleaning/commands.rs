use tauri::{AppHandle, State};

use super::model::{CleaningService, CleaningServiceInput, ServiceStaffInput, StaffSuggestion};
use super::service;
use crate::db;
use crate::error::CommandError;
use crate::state::DbState;

#[tauri::command]
pub async fn cleaning_list_by_date(
    date: String,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Vec<CleaningService>, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::list_services_by_date(&pool, data_dir.as_deref(), &date).await
}

#[tauri::command]
pub async fn cleaning_get(
    id: i64,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<CleaningService, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::get_service(&pool, data_dir.as_deref(), id).await
}

#[tauri::command]
pub async fn cleaning_create(
    input: CleaningServiceInput,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<CleaningService, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::create_service(&pool, data_dir.as_deref(), input).await
}

#[tauri::command]
pub async fn cleaning_update(
    id: i64,
    input: CleaningServiceInput,
    confirm_reopen: Option<bool>,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<CleaningService, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::update_service(
        &pool,
        data_dir.as_deref(),
        id,
        input,
        confirm_reopen.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub async fn cleaning_set_status(
    id: i64,
    status: String,
    start_time: Option<String>,
    end_time: Option<String>,
    confirm_reopen: Option<bool>,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<CleaningService, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::set_service_status(
        &pool,
        data_dir.as_deref(),
        id,
        &status,
        start_time.as_deref(),
        end_time.as_deref(),
        confirm_reopen.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub async fn cleaning_set_staff(
    id: i64,
    staff: Vec<ServiceStaffInput>,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<CleaningService, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::set_service_staff(&pool, data_dir.as_deref(), id, staff).await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn cleaning_suggest_staff(
    place_id: i64,
    service_type: String,
    date: String,
    count: Option<usize>,
    start_time: Option<String>,
    end_time: Option<String>,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Vec<StaffSuggestion>, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::suggest_staff(
        &pool,
        data_dir.as_deref(),
        place_id,
        &service_type,
        &date,
        count,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .await
}
