mod db;
mod error;
mod settings;
mod state;

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
        ])
        .run(tauri::generate_context!())
        .expect("No se pudo iniciar la aplicación");
}
