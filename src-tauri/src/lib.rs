mod db;
pub mod domain;
mod error;
mod settings;
mod state;
pub mod util;

use domain::cleaning::commands as cleaning_commands;
use domain::incidents::commands as incidents_commands;
use domain::places::commands as places_commands;
use domain::reports::commands as reports_commands;
use domain::staff::commands as staff_commands;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Single instance must be registered before any other plugin.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(state::StartupNotice::default());

            db::prepare_data_dirs(app.handle()).map_err(|error| error.message)?;
            let path = db::db_path(app.handle()).map_err(|error| error.message)?;
            let (pool, notice) = db::open_with_recovery(&path).map_err(|error| error.message)?;

            if let Some(message) = notice {
                app.state::<state::StartupNotice>().set(message);
            }

            app.manage(state::DbState::new(pool));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            settings::settings_get,
            settings::settings_update,
            state::take_startup_notice,
            staff_commands::staff_list,
            staff_commands::staff_get,
            staff_commands::staff_create,
            staff_commands::staff_update,
            staff_commands::staff_deactivate,
            staff_commands::staff_reactivate,
            staff_commands::staff_available_apartments,
            staff_commands::staff_places_set,
            staff_commands::staff_rest_days_get,
            staff_commands::staff_rest_days_set,
            staff_commands::staff_absences_list,
            staff_commands::staff_absences_create,
            staff_commands::staff_absences_update,
            staff_commands::staff_absences_delete,
            places_commands::places_list,
            places_commands::places_get,
            places_commands::places_create,
            places_commands::places_update,
            places_commands::places_deactivate,
            places_commands::places_reactivate,
            places_commands::places_recommended_task,
            cleaning_commands::cleaning_list_by_date,
            cleaning_commands::cleaning_get,
            cleaning_commands::cleaning_create,
            cleaning_commands::cleaning_update,
            cleaning_commands::cleaning_set_status,
            cleaning_commands::cleaning_set_staff,
            cleaning_commands::cleaning_suggest_staff,
            incidents_commands::incidents_list,
            incidents_commands::incidents_create,
            incidents_commands::incidents_set_status,
            reports_commands::reports_services_by_staff,
            reports_commands::reports_average_times,
            reports_commands::reports_absences_by_month,
            reports_commands::reports_daily_sheet,
        ])
        .run(tauri::generate_context!())
        .expect("No se pudo iniciar la aplicación");
}
