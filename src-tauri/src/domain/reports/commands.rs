use tauri::State;

use super::{
    export_csv, export_pdf, export_xlsx,
    queries::{
        self, AbsencesByMonthRow, AverageTimesRow, DailySheetRow, DateRange, ExportFilters,
        ExportKind, ServicesByStaffRow,
    },
};
use crate::error::CommandError;
use crate::state::DbState;

#[tauri::command]
pub async fn reports_services_by_staff(
    range: DateRange,
    state: State<'_, DbState>,
) -> Result<Vec<ServicesByStaffRow>, CommandError> {
    let pool = state.pool().read().await.clone();
    queries::services_by_staff(&pool, &range).await
}

#[tauri::command]
pub async fn reports_average_times(
    range: DateRange,
    state: State<'_, DbState>,
) -> Result<Vec<AverageTimesRow>, CommandError> {
    let pool = state.pool().read().await.clone();
    queries::average_times(&pool, &range).await
}

#[tauri::command]
pub async fn reports_absences_by_month(
    month: String,
    state: State<'_, DbState>,
) -> Result<Vec<AbsencesByMonthRow>, CommandError> {
    let pool = state.pool().read().await.clone();
    queries::absences_by_month(&pool, &month).await
}

#[tauri::command]
pub async fn reports_daily_sheet(
    date: String,
    state: State<'_, DbState>,
) -> Result<Vec<DailySheetRow>, CommandError> {
    let pool = state.pool().read().await.clone();
    queries::daily_sheet(&pool, &date).await
}

#[tauri::command]
pub async fn export_csv(
    kind: ExportKind,
    filters: ExportFilters,
    dest_path: String,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let pool = state.pool().read().await.clone();
    export_csv::export_csv(&pool, kind, &filters, &dest_path).await
}

#[tauri::command]
pub async fn export_xlsx(
    kind: ExportKind,
    filters: ExportFilters,
    dest_path: String,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let pool = state.pool().read().await.clone();
    export_xlsx::export_xlsx(&pool, kind, &filters, &dest_path).await
}

#[tauri::command]
pub async fn export_pdf(
    kind: ExportKind,
    filters: ExportFilters,
    dest_path: String,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let pool = state.pool().read().await.clone();
    export_pdf::export_pdf(&pool, kind, &filters, &dest_path).await
}
