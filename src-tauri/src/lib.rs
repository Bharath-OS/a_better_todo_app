mod commands;
mod db;
mod tray;

use commands::DbState;
use db::Database;
use tauri::Manager;
fn create_overlay(app_handle: &tauri::AppHandle) {
    if app_handle.get_webview_window("overlay").is_some() {
        return;
    }
    let _ = tauri::WebviewWindowBuilder::new(
        app_handle,
        "overlay",
        tauri::WebviewUrl::App("overlay.html".into()),
    )
    .title("PTC Overlay")
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(true)
    .focused(false)
    .inner_size(168.0, 32.0)
    .build();
}

fn destroy_overlay(app_handle: &tauri::AppHandle) {
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        let _ = overlay.close();
    }
}

fn update_overlay(app_handle: &tauri::AppHandle) {
    let show_overlay = {
        if let Some(main) = app_handle.get_webview_window("main") {
            let visible = main.is_visible().unwrap_or(true);
            let minimized = main.is_minimized().unwrap_or(false);
            let focused = main.is_focused().unwrap_or(true);
            !(visible && !minimized && focused)
        } else {
            false
        }
    };
    if show_overlay {
        create_overlay(app_handle);
    } else {
        destroy_overlay(app_handle);
    }
}

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

            tray::setup_tray(app)?;

            // Track main window focus and manage overlay lifecycle
            let app_handle = app.handle().clone();

            if let Some(main_window) = app.get_webview_window("main") {
                main_window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            api.prevent_close();
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.hide();
                            }
                            create_overlay(&app_handle);
                        }
                        tauri::WindowEvent::Focused(_) => {
                            update_overlay(&app_handle);
                        }
                        _ => {}
                    }
                });
            }

            // Initial check: if main is not visible/focused, create overlay
            update_overlay(app.handle());

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
            commands::close_overlay_and_show_main,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Don't exit on window close
            }
        });
}


