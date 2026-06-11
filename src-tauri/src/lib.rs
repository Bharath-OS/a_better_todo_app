mod commands;
mod db;
mod tray;

use commands::DbState;
use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let database =
                Database::new(&app_dir).expect("Failed to initialize database");
            app.manage(DbState(std::sync::Mutex::new(database)));

            // Position overlay at top-right corner
            if let Some(overlay) = app.get_webview_window("overlay") {
                if let Some(monitor) = overlay.current_monitor().ok().flatten() {
                    let size = monitor.size();
                    let scale = monitor.scale_factor();
                    let x = size.width as f64 / scale - 168.0 - 16.0;
                    let y = 16.0;
                    let _ = overlay.set_position(tauri::LogicalPosition::new(x, y));
                }
            }

            tray::setup_tray(app)?;

            // Listen for window close events (hide instead of close)
            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window("main") {
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::create_task,
            commands::toggle_task,
            commands::update_task,
            commands::delete_task,
            commands::get_settings,
            commands::update_setting,
            commands::get_history,
            commands::get_streak,
            commands::reset_data,
            commands::resize_overlay,
            commands::exit_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Don't exit on window close
            }
        });
}


