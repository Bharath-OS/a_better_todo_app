//! SQLite persistence with versioned migrations.
//!
//! Schema uses `horizon` (`daily|weekly|monthly|goal`) and `due_at` (local wall time).
//! Legacy DBs with `period` / `due_time` are migrated in place.

use crate::time_util::{format_day_id, local_day_id, parse_day_starts_at};
use chrono::Local;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub type DbResult<T> = Result<T, String>;

#[derive(Debug, Default)]
pub struct TaskPatch<'a> {
    pub title: Option<&'a str>,
    pub horizon: Option<&'a str>,
    pub due_at: Option<&'a str>,
    pub clear_due: bool,
    pub notes: Option<&'a str>,
    pub remind_minutes_before: Option<i64>,
    pub progress: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub horizon: String,
    pub due_at: Option<String>,
    pub remind_minutes_before: i64,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub notes: Option<String>,
    pub progress: Option<i64>,
    pub second_remind_minutes_before: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DayStats {
    pub date: String,
    pub total: i64,
    pub completed: i64,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(app_dir: &Path) -> DbResult<Self> {
        std::fs::create_dir_all(app_dir).map_err(|e| e.to_string())?;
        let path = app_dir.join("todly.db");
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> DbResult<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .map_err(|e| e.to_string())?;

        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // Detect legacy schema (no migrations, but old tasks table exists).
        let has_tasks = table_exists(&conn, "tasks")?;
        let has_horizon = has_tasks && column_exists(&conn, "tasks", "horizon")?;
        let has_period = has_tasks && column_exists(&conn, "tasks", "period")?;

        if version < 1 {
            if has_period && !has_horizon {
                migrate_legacy_v1(&conn)?;
            } else if !has_tasks {
                conn.execute_batch(SCHEMA_V1).map_err(|e| e.to_string())?;
            } else if has_horizon {
                // Already on v1-shaped schema without migration row
            } else {
                conn.execute_batch(SCHEMA_V1).map_err(|e| e.to_string())?;
            }
            mark_migration(&conn, 1)?;
        }

        seed_default_settings(&conn)?;
        Ok(())
    }

    pub fn list_tasks(&self, horizon: Option<&str>) -> DbResult<Vec<Task>> {
        let conn = self.conn.lock();
        let mut out = Vec::new();
        if let Some(h) = horizon {
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, horizon, due_at, remind_minutes_before, completed_at,
                            created_at, notes, progress, second_remind_minutes_before
                     FROM tasks WHERE horizon = ?1
                     ORDER BY completed_at IS NOT NULL, due_at IS NULL, due_at ASC, created_at DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![h], map_task)
                .map_err(|e| e.to_string())?;
            for row in rows {
                out.push(row.map_err(|e| e.to_string())?);
            }
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, horizon, due_at, remind_minutes_before, completed_at,
                            created_at, notes, progress, second_remind_minutes_before
                     FROM tasks
                     ORDER BY horizon, completed_at IS NOT NULL, due_at IS NULL, due_at ASC, created_at DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], map_task).map_err(|e| e.to_string())?;
            for row in rows {
                out.push(row.map_err(|e| e.to_string())?);
            }
        }
        Ok(out)
    }

    pub fn tasks_for_local_today(&self) -> DbResult<Vec<Task>> {
        let settings = self.get_settings()?;
        let day_starts = settings
            .get("day_starts_at")
            .map(|s| s.as_str())
            .unwrap_or("00:00");
        let mins = parse_day_starts_at(day_starts);
        let today = format_day_id(local_day_id(Local::now(), mins));
        let all = self.list_tasks(Some("daily"))?;
        // Include weekly/monthly items due today as well for the Today view.
        let mut extra = self.list_tasks(None)?;
        extra.retain(|t| {
            t.horizon != "daily"
                && t.completed_at.is_none()
                && t.due_at
                    .as_ref()
                    .map(|d| d.starts_with(&today))
                    .unwrap_or(false)
        });
        let mut daily = all;
        daily.extend(extra);
        Ok(daily)
    }

    pub fn create_task(
        &self,
        id: &str,
        title: &str,
        horizon: &str,
        due_at: Option<&str>,
        notes: Option<&str>,
        remind_minutes_before: Option<i64>,
    ) -> DbResult<Task> {
        validate_horizon(horizon)?;
        let entitlement = self
            .get_settings()?
            .get("entitlement")
            .cloned()
            .unwrap_or_else(|| "free".into());
        if entitlement != "pro" {
            if horizon == "monthly" || horizon == "goal" {
                return Err("Monthly and Long-term goals require Pro (demo unlock in Upgrade)."
                    .into());
            }
            let active = self.active_task_count()?;
            if active >= 20 {
                return Err("Free plan allows up to 20 active tasks. Unlock Pro (demo) to continue."
                    .into());
            }
        }

        let now = local_now_iso();
        let remind = remind_minutes_before.unwrap_or(10);
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO tasks (id, title, horizon, due_at, remind_minutes_before, completed_at, created_at, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7)",
            params![id, title, horizon, due_at, remind, now, notes],
        )
        .map_err(|e| e.to_string())?;
        Ok(Task {
            id: id.to_string(),
            title: title.to_string(),
            horizon: horizon.to_string(),
            due_at: due_at.map(|s| s.to_string()),
            remind_minutes_before: remind,
            completed_at: None,
            created_at: now,
            notes: notes.map(|s| s.to_string()),
            progress: None,
            second_remind_minutes_before: None,
        })
    }

    pub fn toggle_task(&self, id: &str) -> DbResult<Task> {
        let conn = self.conn.lock();
        let mut task = conn
            .query_row(
                "SELECT id, title, horizon, due_at, remind_minutes_before, completed_at,
                        created_at, notes, progress, second_remind_minutes_before
                 FROM tasks WHERE id = ?1",
                params![id],
                map_task,
            )
            .map_err(|e| e.to_string())?;
        if task.completed_at.is_some() {
            conn.execute(
                "UPDATE tasks SET completed_at = NULL WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
            task.completed_at = None;
        } else {
            let now = local_now_iso();
            conn.execute(
                "UPDATE tasks SET completed_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| e.to_string())?;
            task.completed_at = Some(now);
        }
        Ok(task)
    }

    pub fn update_task(&self, id: &str, patch: TaskPatch<'_>) -> DbResult<()> {
        if let Some(h) = patch.horizon {
            validate_horizon(h)?;
        }
        let conn = self.conn.lock();
        if let Some(t) = patch.title {
            conn.execute("UPDATE tasks SET title = ?1 WHERE id = ?2", params![t, id])
                .map_err(|e| e.to_string())?;
        }
        if let Some(h) = patch.horizon {
            conn.execute(
                "UPDATE tasks SET horizon = ?1 WHERE id = ?2",
                params![h, id],
            )
            .map_err(|e| e.to_string())?;
        }
        if patch.clear_due {
            conn.execute("UPDATE tasks SET due_at = NULL WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())?;
        } else if let Some(due) = patch.due_at {
            conn.execute(
                "UPDATE tasks SET due_at = ?1 WHERE id = ?2",
                params![due, id],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(n) = patch.notes {
            conn.execute("UPDATE tasks SET notes = ?1 WHERE id = ?2", params![n, id])
                .map_err(|e| e.to_string())?;
        }
        if let Some(r) = patch.remind_minutes_before {
            conn.execute(
                "UPDATE tasks SET remind_minutes_before = ?1 WHERE id = ?2",
                params![r, id],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(p) = patch.progress {
            conn.execute(
                "UPDATE tasks SET progress = ?1 WHERE id = ?2",
                params![p, id],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> DbResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn active_task_count(&self) -> DbResult<i64> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE completed_at IS NULL",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())
    }

    pub fn get_settings(&self) -> DbResult<HashMap<String, String>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        let mut map = HashMap::new();
        for row in rows {
            let (k, v) = row.map_err(|e| e.to_string())?;
            map.insert(k, v);
        }
        Ok(map)
    }

    pub fn update_setting(&self, key: &str, value: &str) -> DbResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn day_stats_last_n(&self, n: i64) -> DbResult<Vec<DayStats>> {
        let settings = self.get_settings()?;
        let day_starts = settings
            .get("day_starts_at")
            .map(|s| s.as_str())
            .unwrap_or("00:00");
        let mins = parse_day_starts_at(day_starts);
        let today = local_day_id(Local::now(), mins);
        let mut out = Vec::new();
        let conn = self.conn.lock();
        for i in (0..n).rev() {
            let day = today - chrono::Duration::days(i);
            let date = format_day_id(day);
            // Count daily tasks completed on that local date (by completed_at prefix).
            let completed: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM tasks
                     WHERE horizon = 'daily' AND completed_at IS NOT NULL AND substr(completed_at, 1, 10) = ?1",
                    params![date],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            let total: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM tasks WHERE horizon = 'daily'
                     AND (substr(created_at, 1, 10) <= ?1)
                     AND (completed_at IS NULL OR substr(completed_at, 1, 10) >= ?1 OR substr(created_at, 1, 10) = ?1)",
                    params![date],
                    |r| r.get(0),
                )
                .unwrap_or(completed);
            out.push(DayStats {
                date,
                total: total.max(completed),
                completed,
            });
        }
        Ok(out)
    }

    pub fn seed_demo_if_empty(&self) -> DbResult<()> {
        let conn = self.conn.lock();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if count > 0 {
            return Ok(());
        }
        drop(conn);
        let samples = [
            ("Review morning emails", "daily"),
            ("Deep work block: 2 hours", "daily"),
            ("Send invoice to Acme Studio", "daily"),
            ("Ship landing page v2", "weekly"),
            ("Prepare team presentation", "weekly"),
        ];
        for (title, horizon) in samples {
            let id = uuid::Uuid::new_v4().to_string();
            let _ = self.create_task(&id, title, horizon, None, None, Some(10))?;
        }
        Ok(())
    }

    pub fn upcoming_reminders(&self, within_minutes: i64) -> DbResult<Vec<Task>> {
        let now = Local::now();
        let tasks = self.list_tasks(None)?;
        let mut out = Vec::new();
        for t in tasks {
            if t.completed_at.is_some() {
                continue;
            }
            let Some(due) = t.due_at.as_ref() else {
                continue;
            };
            if let Ok(due_dt) = chrono::NaiveDateTime::parse_from_str(due, "%Y-%m-%dT%H:%M:%S") {
                if let Some(due_local) = due_dt.and_local_timezone(Local).single() {
                    let lead = chrono::Duration::minutes(t.remind_minutes_before);
                    let fire_at = due_local - lead;
                    let delta = (fire_at - now).num_minutes();
                    if delta <= 0 && (due_local - now).num_minutes() <= within_minutes {
                        out.push(t);
                    }
                }
            }
        }
        Ok(out)
    }
}

const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    horizon TEXT NOT NULL CHECK (horizon IN ('daily','weekly','monthly','goal')),
    due_at TEXT,
    remind_minutes_before INTEGER NOT NULL DEFAULT 10,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    notes TEXT,
    progress INTEGER,
    second_remind_minutes_before INTEGER
);
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

fn mark_migration(conn: &Connection, version: i64) -> DbResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
        params![version, local_now_iso()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn migrate_legacy_v1(conn: &Connection) -> DbResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE tasks_new (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            horizon TEXT NOT NULL CHECK (horizon IN ('daily','weekly','monthly','goal')),
            due_at TEXT,
            remind_minutes_before INTEGER NOT NULL DEFAULT 10,
            completed_at TEXT,
            created_at TEXT NOT NULL,
            notes TEXT,
            progress INTEGER,
            second_remind_minutes_before INTEGER
        );
        INSERT INTO tasks_new (id, title, horizon, due_at, remind_minutes_before, completed_at, created_at, notes)
        SELECT
            id,
            title,
            CASE period
                WHEN 'quarterly' THEN 'goal'
                WHEN 'yearly' THEN 'goal'
                ELSE period
            END,
            due_time,
            10,
            CASE WHEN completed != 0 THEN COALESCE(completed_at, created_at) ELSE NULL END,
            created_at,
            notes
        FROM tasks;
        DROP TABLE tasks;
        ALTER TABLE tasks_new RENAME TO tasks;
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn seed_default_settings(conn: &Connection) -> DbResult<()> {
    let defaults = [
        ("theme", "system"),
        ("day_starts_at", "00:00"),
        ("week_starts_on", "monday"),
        ("dwell_ms", "2000"),
        ("overlay_position", "top-center"),
        ("overlay_style", "next-task"),
        ("overlay_visible", "true"),
        ("remind_minutes_before", "10"),
        ("entitlement", "free"),
        ("quick_add_hotkey_win", "Ctrl+Alt+T"),
        ("quick_add_hotkey_mac", "Command+Option+T"),
        ("onboarding_done", "false"),
        ("fullscreen_behaviour", "hide"),
    ];
    for (k, v) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            params![k, v],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn validate_horizon(h: &str) -> DbResult<()> {
    match h {
        "daily" | "weekly" | "monthly" | "goal" => Ok(()),
        _ => Err(format!("invalid horizon '{h}'")),
    }
}

fn local_now_iso() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        horizon: row.get(2)?,
        due_at: row.get(3)?,
        remind_minutes_before: row.get(4)?,
        completed_at: row.get(5)?,
        created_at: row.get(6)?,
        notes: row.get(7)?,
        progress: row.get(8)?,
        second_remind_minutes_before: row.get(9)?,
    })
}

fn table_exists(conn: &Connection, name: &str) -> DbResult<bool> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            params![name],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(n > 0)
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> DbResult<bool> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;
    for row in rows {
        if row.map_err(|e| e.to_string())? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list_toggle_delete() {
        let db = Database::open_in_memory().unwrap();
        let t = db
            .create_task("a1", "Hello", "daily", None, None, Some(10))
            .unwrap();
        assert_eq!(t.title, "Hello");
        assert!(t.completed_at.is_none());
        assert_eq!(db.list_tasks(None).unwrap().len(), 1);
        let toggled = db.toggle_task("a1").unwrap();
        assert!(toggled.completed_at.is_some());
        db.delete_task("a1").unwrap();
        assert!(db.list_tasks(None).unwrap().is_empty());
    }

    #[test]
    fn monthly_horizon_allowed_in_schema() {
        let db = Database::open_in_memory().unwrap();
        db.update_setting("entitlement", "pro").unwrap();
        let t = db
            .create_task("m1", "Month goal", "monthly", None, None, None)
            .unwrap();
        assert_eq!(t.horizon, "monthly");
    }

    #[test]
    fn free_blocks_monthly_and_caps_active() {
        let db = Database::open_in_memory().unwrap();
        let err = db
            .create_task("m1", "Month", "monthly", None, None, None)
            .unwrap_err();
        assert!(err.to_lowercase().contains("pro"));
        for i in 0..20 {
            db.create_task(&format!("t{i}"), "x", "daily", None, None, None)
                .unwrap();
        }
        let err = db
            .create_task("t21", "overflow", "daily", None, None, None)
            .unwrap_err();
        assert!(err.contains("20"));
    }

    #[test]
    fn due_at_round_trips() {
        let db = Database::open_in_memory().unwrap();
        db.create_task(
            "d1",
            "Due",
            "daily",
            Some("2026-11-12T17:00:00"),
            None,
            Some(10),
        )
        .unwrap();
        let tasks = db.list_tasks(None).unwrap();
        assert_eq!(tasks[0].due_at.as_deref(), Some("2026-11-12T17:00:00"));
        // Ensure we never expose a legacy `deadline` field name via serde keys
        let json = serde_json::to_value(&tasks[0]).unwrap();
        assert!(json.get("due_at").is_some());
        assert!(json.get("deadline").is_none());
    }

    #[test]
    fn settings_defaults_include_top_center() {
        let db = Database::open_in_memory().unwrap();
        let s = db.get_settings().unwrap();
        assert_eq!(s.get("overlay_position").map(|x| x.as_str()), Some("top-center"));
        assert_eq!(s.get("dwell_ms").map(|x| x.as_str()), Some("2000"));
    }

    #[test]
    fn rejects_invalid_horizon() {
        let db = Database::open_in_memory().unwrap();
        let err = db
            .create_task("bad", "x", "quarterly", None, None, None)
            .unwrap_err();
        assert!(err.contains("invalid horizon"), "{err}");
    }

    #[test]
    fn empty_title_still_stores_row() {
        // Product may validate in UI; DB layer currently accepts empty string —
        // document current contract so a later required-title change is intentional.
        let db = Database::open_in_memory().unwrap();
        let t = db.create_task("e1", "", "daily", None, None, None).unwrap();
        assert_eq!(t.title, "");
    }

    #[test]
    fn clear_due_removes_due_at() {
        let db = Database::open_in_memory().unwrap();
        db.create_task(
            "d2",
            "Has due",
            "daily",
            Some("2026-11-12T17:00:00"),
            None,
            Some(10),
        )
        .unwrap();
        db.update_task(
            "d2",
            TaskPatch {
                clear_due: true,
                ..TaskPatch::default()
            },
        )
        .unwrap();
        let tasks = db.list_tasks(None).unwrap();
        assert!(tasks[0].due_at.is_none());
    }

    #[test]
    fn delete_missing_id_is_ok() {
        let db = Database::open_in_memory().unwrap();
        db.delete_task("no-such-id").unwrap();
    }

    #[test]
    fn toggle_missing_id_errors() {
        let db = Database::open_in_memory().unwrap();
        assert!(db.toggle_task("missing").is_err());
    }

    #[test]
    fn pro_unlock_allows_goal_horizon() {
        let db = Database::open_in_memory().unwrap();
        db.update_setting("entitlement", "pro").unwrap();
        let t = db
            .create_task("g1", "Ship Todly 1.0", "goal", None, None, None)
            .unwrap();
        assert_eq!(t.horizon, "goal");
    }

    #[test]
    fn seed_demo_is_idempotent_when_nonempty() {
        let db = Database::open_in_memory().unwrap();
        db.create_task("keep", "Keep me", "daily", None, None, None)
            .unwrap();
        db.seed_demo_if_empty().unwrap();
        assert_eq!(db.list_tasks(None).unwrap().len(), 1);
    }

    #[test]
    fn migration_from_legacy_period_table() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                period TEXT NOT NULL CHECK(period IN ('daily','weekly','quarterly','yearly')),
                completed INTEGER NOT NULL DEFAULT 0,
                completed_at TEXT,
                created_at TEXT NOT NULL,
                due_time TEXT,
                notes TEXT,
                recurring INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO tasks (id, title, period, completed, created_at, due_time)
            VALUES ('1', 'Old', 'quarterly', 0, '2026-01-01T00:00:00', '2026-03-01T12:00:00');
            CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);",
        )
        .unwrap();
        // Use Database by wrapping — re-run migrate path via open_in_memory won't have legacy.
        // Call migrate_legacy directly then construct.
        migrate_legacy_v1(&conn).unwrap();
        let horizon: String = conn
            .query_row("SELECT horizon FROM tasks WHERE id='1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(horizon, "goal");
        let due: String = conn
            .query_row("SELECT due_at FROM tasks WHERE id='1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(due, "2026-03-01T12:00:00");
        // monthly must be insertable after migration
        conn.execute(
            "INSERT INTO tasks (id, title, horizon, remind_minutes_before, created_at)
             VALUES ('2', 'M', 'monthly', 10, '2026-01-01T00:00:00')",
            [],
        )
        .unwrap();
    }
}
