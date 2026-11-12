use crate::db::{Database, DayStats, Task, TaskPatch};
use crate::entitlements::Plan;
use crate::overlay::{self, OverlayController};
use crate::time_util::today_local_day_id;
use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct DbState(pub Arc<Mutex<Database>>);

#[tauri::command]
pub fn get_tasks(db: State<DbState>, horizon: Option<String>) -> Result<Vec<Task>, String> {
    db.0.lock().list_tasks(horizon.as_deref())
}

#[tauri::command]
pub fn get_today_tasks(db: State<DbState>) -> Result<Vec<Task>, String> {
    db.0.lock().tasks_for_local_today()
}

#[tauri::command]
pub fn create_task(
    app: AppHandle,
    db: State<DbState>,
    title: String,
    horizon: String,
    due_at: Option<String>,
    notes: Option<String>,
    remind_minutes_before: Option<i64>,
) -> Result<Task, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let task = db.0.lock().create_task(
        &id,
        &title,
        &horizon,
        due_at.as_deref(),
        notes.as_deref(),
        remind_minutes_before,
    )?;
    let _ = app.emit("tasks-changed", ());
    Ok(task)
}

#[tauri::command]
pub fn toggle_task(app: AppHandle, db: State<DbState>, id: String) -> Result<Task, String> {
    let task = db.0.lock().toggle_task(&id)?;
    let _ = app.emit("tasks-changed", ());
    Ok(task)
}

#[derive(serde::Deserialize)]
pub struct UpdateTaskArgs {
    pub id: String,
    pub title: Option<String>,
    pub horizon: Option<String>,
    pub due_at: Option<String>,
    pub clear_due: Option<bool>,
    pub notes: Option<String>,
    pub remind_minutes_before: Option<i64>,
    pub progress: Option<i64>,
}

#[tauri::command]
pub fn update_task(app: AppHandle, db: State<DbState>, args: UpdateTaskArgs) -> Result<(), String> {
    let patch = TaskPatch {
        title: args.title.as_deref(),
        horizon: args.horizon.as_deref(),
        due_at: args.due_at.as_deref(),
        clear_due: args.clear_due.unwrap_or(false),
        notes: args.notes.as_deref(),
        remind_minutes_before: args.remind_minutes_before,
        progress: args.progress,
    };
    db.0.lock().update_task(&args.id, patch)?;
    let _ = app.emit("tasks-changed", ());
    Ok(())
}

#[tauri::command]
pub fn delete_task(app: AppHandle, db: State<DbState>, id: String) -> Result<(), String> {
    db.0.lock().delete_task(&id)?;
    let _ = app.emit("tasks-changed", ());
    Ok(())
}

#[tauri::command]
pub fn get_settings(db: State<DbState>) -> Result<HashMap<String, String>, String> {
    db.0.lock().get_settings()
}

#[tauri::command]
pub fn update_setting(
    app: AppHandle,
    db: State<DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    db.0.lock().update_setting(&key, &value)?;
    if key == "dwell_ms" {
        if let Ok(ms) = value.parse::<u64>() {
            if let Some(ctrl) = app.try_state::<Arc<OverlayController>>() {
                ctrl.set_dwell_ms(ms);
            }
        }
    }
    if key == "overlay_visible" {
        if let Some(win) = app.get_webview_window("overlay") {
            if value == "true" {
                let _ = win.show();
            } else {
                let _ = win.hide();
            }
        }
        if let Some(ctrl) = app.try_state::<Arc<OverlayController>>() {
            ctrl.set_visible(value == "true");
        }
    }
    let _ = app.emit("settings-changed", ());
    Ok(())
}

#[tauri::command]
pub fn get_day_stats(db: State<DbState>, days: Option<i64>) -> Result<Vec<DayStats>, String> {
    db.0.lock().day_stats_last_n(days.unwrap_or(7))
}

#[tauri::command]
pub fn get_local_today(db: State<DbState>) -> Result<String, String> {
    let settings = db.0.lock().get_settings()?;
    let day_starts = settings
        .get("day_starts_at")
        .map(|s| s.as_str())
        .unwrap_or("00:00");
    Ok(today_local_day_id(day_starts))
}

#[derive(Serialize)]
pub struct EntitlementInfo {
    pub plan: String,
    pub active_tasks: i64,
    pub active_cap: Option<i64>,
}

#[tauri::command]
pub fn get_entitlement(db: State<DbState>) -> Result<EntitlementInfo, String> {
    let db = db.0.lock();
    let settings = db.get_settings()?;
    let plan = Plan::from_setting(settings.get("entitlement").map(|s| s.as_str()).unwrap_or("free"));
    let active = db.active_task_count()?;
    Ok(EntitlementInfo {
        plan: plan.as_str().to_string(),
        active_tasks: active,
        active_cap: if plan == Plan::Free { Some(20) } else { None },
    })
}

#[tauri::command]
pub fn can_feature(db: State<DbState>, feature: String) -> Result<bool, String> {
    let settings = db.0.lock().get_settings()?;
    let plan = Plan::from_setting(settings.get("entitlement").map(|s| s.as_str()).unwrap_or("free"));
    Ok(plan.can(&feature))
}

#[tauri::command]
pub fn unlock_pro_demo(app: AppHandle, db: State<DbState>) -> Result<(), String> {
    db.0.lock().update_setting("entitlement", "pro")?;
    let _ = app.emit("settings-changed", ());
    Ok(())
}

#[tauri::command]
pub fn show_quick_add(app: AppHandle) -> Result<(), String> {
    crate::ensure_quick_add(&app)?;
    if let Some(win) = app.get_webview_window("quick-add") {
        let _ = win.show();
        let _ = win.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub fn hide_quick_add(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("quick-add") {
        let _ = win.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn set_overlay_expanded(
    app: AppHandle,
    ctrl: State<'_, Arc<OverlayController>>,
    expanded: bool,
) -> Result<(), String> {
    ctrl.set_expanded(expanded);
    if let Some(win) = app.get_webview_window("overlay") {
        // Expanded → interactive; collapsed → click-through until dwell arms again.
        let _ = win.set_ignore_cursor_events(!expanded);
    }
    debug_assert_eq!(ctrl.is_expanded(), expanded);
    overlay::resize_overlay(&app, expanded)?;
    Ok(())
}

#[tauri::command]
pub fn pin_overlay(app: AppHandle) -> Result<(), String> {
    overlay::pin_top_center(&app, 340.0, 48.0)
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub fn exit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}
