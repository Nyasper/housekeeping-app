use tauri::{AppHandle, State};

use super::model::{Incident, IncidentInput, IncidentListFilters};
use super::service;
use crate::db;
use crate::error::CommandError;
use crate::state::DbState;

#[tauri::command]
pub async fn incidents_list(
    filters: Option<IncidentListFilters>,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Vec<Incident>, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::list_incidents(&pool, data_dir.as_deref(), filters.unwrap_or_default()).await
}

#[tauri::command]
pub async fn incidents_create(
    input: IncidentInput,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Incident, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::create_incident(&pool, data_dir.as_deref(), input).await
}

#[tauri::command]
pub async fn incidents_set_status(
    id: i64,
    status: String,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Incident, CommandError> {
    let data_dir = db::data_dir(&app).ok();
    let pool = state.pool().read().await.clone();
    service::set_incident_status(&pool, data_dir.as_deref(), id, &status).await
}
