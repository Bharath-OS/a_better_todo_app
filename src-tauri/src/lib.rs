mod commands;
mod db;
mod tray;

use commands::DbState;
use db::Database;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

fn create_overlay(app_handle: &tauri::AppHandle) {
    if app_handle.get_webview_window("overlay").is_some() {
        return;
    }
    let _ = tauri::WebviewWindowBuilder::new(
        app_handle,
        "overlay",
        tauri::WebviewUrl::App("overlay.html".into()),
    )
    .title("")
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .fullscreen(true)
    .visible(true)
    .focused(false)
    .build();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        return;
                    }
                    if app.get_webview_window("quick-add").is_some() {
                        return;
                    }
                    // Defer window creation to next event loop iteration so the
                    // shortcut handler returns immediately and doesn't block.
                    let h = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        if h.get_webview_window("quick-add").is_some() {
                            return;
                        }
                        let _ = tauri::WebviewWindowBuilder::new(
                            &h,
                            "quick-add",
                            tauri::WebviewUrl::App("quick-add.html".into()),
                        )
                        .title("Quick Add")
                        .decorations(false)
                        .transparent(true)
                        .always_on_top(true)
                        .skip_taskbar(true)
                        .fullscreen(true)
                        .visible(true)
                        .focused(true)
                        .build();
                    });
                })
                .build(),
        )
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let database =
                Database::new(&app_dir).expect("Failed to initialize database");
            let settings = database.get_settings().unwrap_or_default();
            let hotkey = settings
                .get("quick_add_hotkey")
                .cloned()
                .unwrap_or_else(|| "Ctrl+Alt+T".into());
            app.manage(DbState(std::sync::Mutex::new(database)));

            tray::setup_tray(app)?;

            // Register global shortcut with stored hotkey
            let handle = app.handle().clone();
            if handle.global_shortcut().register(hotkey.as_str()).is_err() {
                eprintln!("Failed to register hotkey '{}', falling back to Ctrl+Alt+T", hotkey);
                let _ = handle.global_shortcut().register("Ctrl+Alt+T");
            }

            // Create overlay at startup and keep it visible
            let overlay_handle = app.handle().clone();
            let _ = app.handle().run_on_main_thread(move || {
                create_overlay(&overlay_handle);
            });

            // Handle main window close (prevent close, just hide)
            if let Some(main_window) = app.get_webview_window("main") {
                let win_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            api.prevent_close();
                            if let Some(window) = win_handle.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        _ => {}
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
            commands::close_overlay_and_show_main,
            commands::cursor_over_rect,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Don't exit on window close
            }
        });
}


