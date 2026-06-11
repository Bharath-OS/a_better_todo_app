use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub period: String,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub due_time: Option<String>,
    pub notes: Option<String>,
    pub recurring: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailyHistory {
    pub date: String,
    pub total: i64,
    pub completed: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreakData {
    pub current: i64,
    pub best: i64,
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(app_dir).ok();
        let db_path = app_dir.join("ptc.db");
        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                period TEXT NOT NULL CHECK(period IN ('daily','weekly','quarterly','yearly')),
                completed INTEGER NOT NULL DEFAULT 0,
                completed_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                due_time TEXT,
                notes TEXT,
                recurring INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS daily_history (
                date TEXT PRIMARY KEY,
                total INTEGER NOT NULL DEFAULT 0,
                completed INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    pub fn get_tasks(&self, period: Option<&str>) -> Result<Vec<Task>> {
        let conn = self.conn.lock().unwrap();
        let query = match period {
            Some(_) => "SELECT * FROM tasks WHERE period = ?1 ORDER BY completed ASC, created_at DESC",
            None => "SELECT * FROM tasks ORDER BY period, completed ASC, created_at DESC",
        };
        let mut stmt = conn.prepare(query)?;

        let tasks = if let Some(p) = period {
            stmt.query_map(params![p], Self::map_task)?
        } else {
            stmt.query_map([], Self::map_task)?
        };

        tasks.collect::<Result<Vec<_>>>()
    }

    pub fn create_task(
        &self,
        id: &str,
        title: &str,
        period: &str,
        due_time: Option<&str>,
        notes: Option<&str>,
    ) -> Result<Task> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        conn.execute(
            "INSERT INTO tasks (id, title, period, completed, created_at, due_time, notes, recurring)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6, 0)",
            params![id, title, period, now, due_time, notes],
        )?;
        Ok(Task {
            id: id.to_string(),
            title: title.to_string(),
            period: period.to_string(),
            completed: false,
            completed_at: None,
            created_at: now,
            due_time: due_time.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
            recurring: false,
        })
    }

    pub fn toggle_task(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let (completed, _completed_at): (bool, Option<String>) = conn
            .query_row(
                "SELECT completed, completed_at FROM tasks WHERE id = ?1",
                params![id],
                |row| {
                    let comp: i64 = row.get(0)?;
                    Ok((comp != 0, row.get(1)?))
                },
            )
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let new_completed = !completed;
        let new_completed_at = if new_completed { Some(&now) } else { None };

        conn.execute(
            "UPDATE tasks SET completed = ?1, completed_at = ?2 WHERE id = ?3",
            params![new_completed as i64, new_completed_at, id],
        )?;

        drop(conn);
        self.update_daily_history()?;
        Ok(())
    }

    pub fn update_task(
        &self,
        id: &str,
        title: Option<&str>,
        completed: Option<bool>,
        due_time: Option<&str>,
        notes: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(t) = title {
            conn.execute("UPDATE tasks SET title = ?1 WHERE id = ?2", params![t, id])?;
        }
        if let Some(c) = completed {
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
            conn.execute(
                "UPDATE tasks SET completed = ?1, completed_at = ?2 WHERE id = ?3",
                params![c as i64, if c { Some(&now) } else { None }, id],
            )?;
        }
        if let Some(t) = due_time {
            conn.execute(
                "UPDATE tasks SET due_time = ?1 WHERE id = ?2",
                params![t, id],
            )?;
        }
        if let Some(n) = notes {
            conn.execute(
                "UPDATE tasks SET notes = ?1 WHERE id = ?2",
                params![n, id],
            )?;
        }
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        drop(conn);
        self.update_daily_history()?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<std::collections::HashMap<String, String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (k, v) = row?;
            map.insert(k, v);
        }

        if !map.contains_key("overlay_corner") {
            map.insert("overlay_corner".into(), "tr".into());
        }
        if !map.contains_key("collapsed_opacity") {
            map.insert("collapsed_opacity".into(), "0.60".into());
        }
        if !map.contains_key("hover_delay_ms") {
            map.insert("hover_delay_ms".into(), "1200".into());
        }
        if !map.contains_key("show_streak") {
            map.insert("show_streak".into(), "true".into());
        }
        if !map.contains_key("keep_on_top") {
            map.insert("keep_on_top".into(), "true".into());
        }
        Ok(map)
    }

    pub fn update_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_history(&self) -> Result<Vec<DailyHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT date, total, completed FROM daily_history ORDER BY date DESC LIMIT 365",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DailyHistory {
                date: row.get(0)?,
                total: row.get(1)?,
                completed: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>>>()
    }

    pub fn get_streak(&self) -> Result<StreakData> {
        let conn = self.conn.lock().unwrap();
        let mut current: i64 = 0;
        let mut best: i64 = 0;

        for i in 0..365 {
            let date = (chrono::Utc::now() - chrono::Duration::days(i))
                .format("%Y-%m-%d")
                .to_string();
            let row: Result<(i64, i64)> = conn.query_row(
                "SELECT COALESCE(total,0), COALESCE(completed,0) FROM daily_history WHERE date = ?1",
                params![date],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );

            match row {
                Ok((total, completed)) => {
                    let all_done = total > 0 && completed >= total;
                    if all_done {
                        current += 1;
                        if current > best {
                            best = current;
                        }
                    } else if i == 0 {
                        continue;
                    } else {
                        break;
                    }
                }
                Err(_) => {
                    if i == 0 {
                        continue;
                    }
                    break;
                }
            }
        }

        Ok(StreakData { current, best })
    }

    pub fn reset_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM tasks; DELETE FROM daily_history;",
        )?;
        drop(conn);
        self.seed_data()?;
        Ok(())
    }

    pub fn seed_data(&self) -> Result<()> {
        let tasks = vec![
            ("Review morning emails", "daily"),
            ("Deep work block: 2 hours", "daily"),
            ("Exercise 30 min", "daily"),
            ("Ship v2 of landing page", "weekly"),
            ("Prepare team presentation", "weekly"),
            ("Code review backlog", "weekly"),
            ("Hire 2 engineers", "quarterly"),
            ("Review Q2 OKRs", "quarterly"),
            ("Read 24 books", "yearly"),
            ("Visit 3 new countries", "yearly"),
        ];

        let now = chrono::Utc::now();
        for (i, (title, period)) in tasks.iter().enumerate() {
            let id = uuid::Uuid::new_v4().to_string();
            let created = (now - chrono::Duration::hours(i as i64 * 2))
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string();
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT OR IGNORE INTO tasks (id, title, period, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, title, period, created],
            )?;
        }
        Ok(())
    }

    fn update_daily_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let (total, completed): (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(CASE WHEN completed THEN 1 ELSE 0 END), 0)
                 FROM tasks WHERE period = 'daily'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;

        conn.execute(
            "INSERT INTO daily_history (date, total, completed) VALUES (?1, ?2, ?3)
             ON CONFLICT(date) DO UPDATE SET total = ?2, completed = ?3",
            params![today, total, completed],
        )?;
        Ok(())
    }

    fn map_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            period: row.get(2)?,
            completed: {
                let v: i64 = row.get(3)?;
                v != 0
            },
            completed_at: row.get(4)?,
            created_at: row.get(5)?,
            due_time: row.get(6)?,
            notes: row.get(7)?,
            recurring: {
                let v: i64 = row.get(8)?;
                v != 0
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                period TEXT NOT NULL CHECK(period IN ('daily','weekly','quarterly','yearly')),
                completed INTEGER NOT NULL DEFAULT 0,
                completed_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                due_time TEXT,
                notes TEXT,
                recurring INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS daily_history (
                date TEXT PRIMARY KEY,
                total INTEGER NOT NULL DEFAULT 0,
                completed INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )
        .unwrap();
        Database {
            conn: Mutex::new(conn),
        }
    }

    #[test]
    fn test_create_and_get_tasks() {
        let db = test_db();
        let task = db
            .create_task("test-1", "Test task", "daily", None, None)
            .unwrap();
        assert_eq!(task.title, "Test task");
        assert_eq!(task.period, "daily");
        assert!(!task.completed);

        let tasks = db.get_tasks(None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Test task");
    }

    #[test]
    fn test_get_tasks_filtered_by_period() {
        let db = test_db();
        db.create_task("t1", "Daily task", "daily", None, None).unwrap();
        db.create_task("t2", "Weekly task", "weekly", None, None).unwrap();

        let daily = db.get_tasks(Some("daily")).unwrap();
        assert_eq!(daily.len(), 1);
        assert_eq!(daily[0].title, "Daily task");

        let weekly = db.get_tasks(Some("weekly")).unwrap();
        assert_eq!(weekly.len(), 1);
        assert_eq!(weekly[0].title, "Weekly task");
    }

    #[test]
    fn test_toggle_task() {
        let db = test_db();
        db.create_task("t1", "Togglable", "daily", None, None).unwrap();

        db.toggle_task("t1").unwrap();
        let tasks = db.get_tasks(Some("daily")).unwrap();
        assert!(tasks[0].completed);

        db.toggle_task("t1").unwrap();
        let tasks = db.get_tasks(Some("daily")).unwrap();
        assert!(!tasks[0].completed);
    }

    #[test]
    fn test_update_task() {
        let db = test_db();
        db.create_task("t1", "Original", "daily", None, None).unwrap();

        db.update_task("t1", Some("Updated"), None, None, None).unwrap();
        let tasks = db.get_tasks(None).unwrap();
        assert_eq!(tasks[0].title, "Updated");

        db.update_task("t1", None, Some(true), None, None).unwrap();
        let tasks = db.get_tasks(None).unwrap();
        assert!(tasks[0].completed);
    }

    #[test]
    fn test_delete_task() {
        let db = test_db();
        db.create_task("t1", "Delete me", "daily", None, None).unwrap();
        assert_eq!(db.get_tasks(None).unwrap().len(), 1);

        db.delete_task("t1").unwrap();
        assert_eq!(db.get_tasks(None).unwrap().len(), 0);
    }

    #[test]
    fn test_settings_crud() {
        let db = test_db();
        let settings = db.get_settings().unwrap();
        assert_eq!(settings.get("overlay_corner").unwrap(), "tr");

        db.update_setting("overlay_corner", "bl").unwrap();
        let settings = db.get_settings().unwrap();
        assert_eq!(settings.get("overlay_corner").unwrap(), "bl");
    }

    #[test]
    fn test_streak_empty() {
        let db = test_db();
        let streak = db.get_streak().unwrap();
        assert_eq!(streak.current, 0);
        assert_eq!(streak.best, 0);
    }

    #[test]
    fn test_reset_data() {
        let db = test_db();
        db.create_task("t1", "Task", "daily", None, None).unwrap();
        assert_eq!(db.get_tasks(None).unwrap().len(), 1);

        db.reset_data().unwrap();
        let tasks = db.get_tasks(None).unwrap();
        assert!(tasks.len() >= 3);
        assert!(tasks.iter().any(|t| t.title.contains("emails")));
    }
}
