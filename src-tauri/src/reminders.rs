//! Reminder scheduler: sleep until the next due reminder (no 1 Hz poll).

use crate::db::Database;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

pub fn start_scheduler(app: AppHandle, db: Arc<Mutex<Database>>) {
    std::thread::spawn(move || loop {
        let sleep_for = match next_wake(&db) {
            Some(d) if d > Duration::from_secs(0) => d.min(Duration::from_secs(60)),
            Some(_) => Duration::from_secs(1),
            None => Duration::from_secs(30),
        };
        std::thread::sleep(sleep_for);
        fire_due(&app, &db);
    });
}

fn next_wake(db: &Arc<Mutex<Database>>) -> Option<Duration> {
    let db = db.lock();
    let tasks = db.list_tasks(None).ok()?;
    let now = chrono::Local::now();
    let mut best: Option<Duration> = None;
    for t in tasks {
        if t.completed_at.is_some() {
            continue;
        }
        let Some(due) = t.due_at.as_ref() else {
            continue;
        };
        let Ok(due_dt) = chrono::NaiveDateTime::parse_from_str(due, "%Y-%m-%dT%H:%M:%S") else {
            continue;
        };
        let Some(due_local) = due_dt.and_local_timezone(chrono::Local).single() else {
            continue;
        };
        let fire_at = due_local - chrono::Duration::minutes(t.remind_minutes_before);
        if fire_at > now {
            let d = (fire_at - now).to_std().unwrap_or(Duration::from_secs(1));
            best = Some(match best {
                Some(b) => b.min(d),
                None => d,
            });
        }
    }
    best
}

fn fire_due(app: &AppHandle, db: &Arc<Mutex<Database>>) {
    let due = {
        let db = db.lock();
        db.upcoming_reminders(120).unwrap_or_default()
    };
    for task in due {
        let _ = app.emit("reminder-due", &task);
        let title = format!("Todly · {}", task.title);
        let body = format!(
            "Due soon · remind {} min before",
            task.remind_minutes_before
        );
        let _ = app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show();
    }
}
