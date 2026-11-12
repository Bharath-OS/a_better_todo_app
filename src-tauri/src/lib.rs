mod commands;
mod db;
mod entitlements;
mod overlay;
mod reminders;
mod time_util;
mod tray;

use commands::DbState;
use db::Database;
use overlay::OverlayController;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn ensure_quick_add(app: &tauri::AppHandle) -> Result<(), String> {
    if app.get_webview_window("quick-add").is_some() {
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "quick-add", WebviewUrl::App("quick-add.html".into()))
        .title("Quick Add")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .inner_size(560.0, 280.0)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let overlay_ctrl = OverlayController::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let handle = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        let _ = ensure_quick_add(&handle);
                        if let Some(win) = handle.get_webview_window("quick-add") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    });
                })
                .build(),
        )
        .manage(overlay_ctrl.clone())
        .setup(move |app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let database = Database::open(&app_dir).expect("Failed to initialize database");
            let _ = database.seed_demo_if_empty();
            let settings = database.get_settings().unwrap_or_default();
            if let Some(dwell) = settings.get("dwell_ms").and_then(|s| s.parse().ok()) {
                overlay_ctrl.set_dwell_ms(dwell);
            }

            let db_arc = Arc::new(Mutex::new(database));
            app.manage(DbState(db_arc.clone()));

            tray::setup_tray(app)?;

            // Hotkey: Windows Ctrl+Alt+T, macOS Cmd+Opt+T
            #[cfg(target_os = "macos")]
            let hotkey = settings
                .get("quick_add_hotkey_mac")
                .cloned()
                .unwrap_or_else(|| "Command+Option+T".into());
            #[cfg(not(target_os = "macos"))]
            let hotkey = settings
                .get("quick_add_hotkey_win")
                .cloned()
                .unwrap_or_else(|| "Ctrl+Alt+T".into());

            let handle = app.handle().clone();
            if handle.global_shortcut().register(hotkey.as_str()).is_err() {
                eprintln!("Failed to register hotkey '{hotkey}', trying Ctrl+Alt+T");
                let _ = handle.global_shortcut().register("Ctrl+Alt+T");
            }

            let overlay_handle = app.handle().clone();
            let quick_handle = app.handle().clone();
            let hit_handle = app.handle().clone();
            let ctrl = overlay_ctrl.clone();
            let remind_handle = app.handle().clone();
            let remind_db = db_arc.clone();

            let _ = app.handle().run_on_main_thread(move || {
                let _ = overlay::create_overlay_window(&overlay_handle);
                let _ = ensure_quick_add(&quick_handle);
                overlay::start_hit_test_loop(hit_handle, ctrl);
                reminders::start_scheduler(remind_handle, remind_db);
            });

            if let Some(main_window) = app.get_webview_window("main") {
                let win_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(window) = win_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::get_today_tasks,
            commands::create_task,
            commands::toggle_task,
            commands::update_task,
            commands::delete_task,
            commands::get_settings,
            commands::update_setting,
            commands::get_day_stats,
            commands::get_local_today,
            commands::get_entitlement,
            commands::can_feature,
            commands::unlock_pro_demo,
            commands::show_quick_add,
            commands::hide_quick_add,
            commands::set_overlay_expanded,
            commands::pin_overlay,
            commands::show_main_window,
            commands::exit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Todly");
}
