use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder},
    Emitter, Manager,
};

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "Show main window").build(app)?;
    let quick_add = MenuItemBuilder::with_id("quick_add", "Quick add...").build(app)?;
    let keep_on_top = MenuItemBuilder::with_id("keep_on_top", "Keep on top")
        .accelerator("CmdOrCtrl+T")
        .build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit PTC").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&quick_add)
        .item(&keep_on_top)
        .separator()
        .item(&quit)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("overlay") {
                        window.hide().ok();
                    }
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
                "quick_add" => {
                    // Quick add functionality - could emit an event to the main window
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                        window.emit("quick-add", ()).ok();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("overlay") {
                    window.hide().ok();
                }
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
        })
        .build(app)?;

    Ok(())
}
