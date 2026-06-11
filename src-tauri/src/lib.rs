mod commands;
mod db;
mod tray;

use commands::DbState;
use db::Database;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use std::sync::atomic::{AtomicBool, Ordering};

static OVERLAY_PENDING: AtomicBool = AtomicBool::new(false);

fn create_overlay(app_handle: &tauri::AppHandle) {
    if app_handle.get_webview_window("overlay").is_some() {
        return;
    }
    let _ = tauri::WebviewWindowBuilder::new(
        app_handle,
        "overlay",
        tauri::WebviewUrl::App("overlay.html".into()),
    )
    .title("Todly Overlay")
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
    // Check if a quick-add or confetti window is active — don't create overlay during brief focus changes
    let has_transient = app_handle.get_webview_window("quick-add").is_some()
        || app_handle.get_webview_window("overlay").is_some();
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
    if show_overlay && !has_transient {
        // Debounce: only create overlay on next event loop iteration
        if OVERLAY_PENDING.swap(true, Ordering::SeqCst) == false {
            let h = app_handle.clone();
            let _ = app_handle.run_on_main_thread(move || {
                OVERLAY_PENDING.store(false, Ordering::SeqCst);
                create_overlay(&h);
            });
        }
    } else {
        OVERLAY_PENDING.store(false, Ordering::SeqCst);
        destroy_overlay(app_handle);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    eprintln!("Shortcut event: key={:?} state={:?}", shortcut, event.state);
                    if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        return;
                    }
                    if app.get_webview_window("quick-add").is_some() {
                        eprintln!("Quick-add window already exists, skipping");
                        return;
                    }
                    eprintln!("Creating quick-add window...");
                    match tauri::WebviewWindowBuilder::new(
                        app,
                        "quick-add",
                        tauri::WebviewUrl::App("quick-add.html".into()),
                    )
                    .title("Quick Add")
                    .decorations(false)
                    .transparent(true)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .inner_size(420.0, 114.0)
                    .visible(true)
                    .focused(true)
                    .center()
                    .build() {
                        Ok(_) => eprintln!("Quick-add window created successfully"),
                        Err(e) => eprintln!("Failed to create quick-add window: {:?}", e),
                    }
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

            // Manage overlay based on window state
            let app_handle = app.handle().clone();
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
                        tauri::WindowEvent::Focused(_) => {
                            update_overlay(&win_handle);
                        }
                        _ => {}
                    }
                });
            }

            // One-shot initial check after window is fully initialized on the event loop
            let init_handle = app.handle().clone();
            let _ = app_handle.run_on_main_thread(move || {
                update_overlay(&init_handle);
            });

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


