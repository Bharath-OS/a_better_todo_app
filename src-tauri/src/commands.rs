use crate::db::{Database, Task, StreakData, DailyHistory};
use std::collections::HashMap;
use tauri::{Emitter, State};
use std::sync::Mutex;

pub struct DbState(pub Mutex<Database>);

#[tauri::command]
pub fn get_tasks(db: State<DbState>, period: Option<String>) -> Result<Vec<Task>, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .get_tasks(period.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_task(
    app_handle: tauri::AppHandle,
    db: State<DbState>,
    title: String,
    period: String,
    due_time: Option<String>,
    notes: Option<String>,
) -> Result<Task, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let task = db.0.lock()
        .map_err(|e| e.to_string())?
        .create_task(&id, &title, &period, due_time.as_deref(), notes.as_deref())
        .map_err(|e| e.to_string())?;
    let _ = app_handle.emit("tasks-changed", ());
    Ok(task)
}

#[tauri::command]
pub fn toggle_task(app_handle: tauri::AppHandle, db: State<DbState>, id: String) -> Result<(), String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .toggle_task(&id)
        .map_err(|e| e.to_string())?;
    let _ = app_handle.emit("tasks-changed", ());
    Ok(())
}

#[tauri::command]
pub fn update_task(
    app_handle: tauri::AppHandle,
    db: State<DbState>,
    id: String,
    title: Option<String>,
    completed: Option<bool>,
    due_time: Option<String>,
    notes: Option<String>,
) -> Result<(), String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .update_task(&id, title.as_deref(), completed, due_time.as_deref(), notes.as_deref())
        .map_err(|e| e.to_string())?;
    let _ = app_handle.emit("tasks-changed", ());
    Ok(())
}

#[tauri::command]
pub fn delete_task(app_handle: tauri::AppHandle, db: State<DbState>, id: String) -> Result<(), String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .delete_task(&id)
        .map_err(|e| e.to_string())?;
    let _ = app_handle.emit("tasks-changed", ());
    Ok(())
}

#[tauri::command]
pub fn get_settings(db: State<DbState>) -> Result<HashMap<String, String>, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .get_settings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_setting(db: State<DbState>, key: String, value: String) -> Result<(), String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .update_setting(&key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_history(db: State<DbState>) -> Result<Vec<DailyHistory>, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .get_history()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_streak(db: State<DbState>) -> Result<StreakData, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .get_streak()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn exit_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
pub fn reset_data(app_handle: tauri::AppHandle, db: State<DbState>) -> Result<(), String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .reset_data()
        .map_err(|e| e.to_string())?;
    let _ = app_handle.emit("tasks-changed", ());
    Ok(())
}
