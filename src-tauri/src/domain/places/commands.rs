use tauri::State;

use super::model::{Place, PlaceInput, RecommendedTask};
use super::service;
use crate::error::CommandError;
use crate::state::DbState;

#[tauri::command]
pub async fn places_list(
    date: Option<String>,
    state: State<'_, DbState>,
) -> Result<Vec<Place>, CommandError> {
    let pool = state.pool().read().await.clone();
    service::list_places_for_date(&pool, date.as_deref()).await
}

#[tauri::command]
pub async fn places_get(id: i64, state: State<'_, DbState>) -> Result<Place, CommandError> {
    let pool = state.pool().read().await.clone();
    service::get_place(&pool, id).await
}

#[tauri::command]
pub async fn places_create(
    input: PlaceInput,
    state: State<'_, DbState>,
) -> Result<Place, CommandError> {
    let pool = state.pool().read().await.clone();
    service::create_place(&pool, input).await
}

#[tauri::command]
pub async fn places_update(
    id: i64,
    input: PlaceInput,
    state: State<'_, DbState>,
) -> Result<Place, CommandError> {
    let pool = state.pool().read().await.clone();
    service::update_place(&pool, id, input).await
}

#[tauri::command]
pub async fn places_deactivate(id: i64, state: State<'_, DbState>) -> Result<Place, CommandError> {
    let pool = state.pool().read().await.clone();
    service::deactivate_place(&pool, id).await
}

#[tauri::command]
pub async fn places_reactivate(id: i64, state: State<'_, DbState>) -> Result<Place, CommandError> {
    let pool = state.pool().read().await.clone();
    service::reactivate_place(&pool, id).await
}

#[tauri::command]
pub async fn places_recommended_task(
    place_id: i64,
    date: String,
    state: State<'_, DbState>,
) -> Result<RecommendedTask, CommandError> {
    let pool = state.pool().read().await.clone();
    service::recommended_task(&pool, place_id, &date).await
}
