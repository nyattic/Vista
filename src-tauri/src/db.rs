use crate::error::{AppError, AppResult};
use crate::hitomi::Gallery;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const HISTORY_LIMIT: i64 = 300;
const SCHEMA_VERSION: i64 = 1;

const SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS favorites (id INTEGER PRIMARY KEY, data TEXT NOT NULL, added_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS history (id INTEGER PRIMARY KEY, data TEXT NOT NULL, viewed_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS progress (id INTEGER PRIMARY KEY, page INTEGER NOT NULL, total INTEGER NOT NULL, updated_at INTEGER NOT NULL);
";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub id: i64,
    pub page: i64,
    pub total: i64,
}

pub struct Db {
    conn: Mutex<Connection>,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn migrate(conn: &Connection) -> AppResult<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version < 1 {
        conn.execute_batch(SCHEMA_V1)?;
    }
    if version != SCHEMA_VERSION {
        conn.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))?;
    }
    Ok(())
}

fn to_json(g: &Gallery) -> AppResult<String> {
    serde_json::to_string(g).map_err(|e| AppError::Decode(e.to_string()))
}

fn from_json(s: &str) -> Option<Gallery> {
    serde_json::from_str(s).ok()
}

impl Db {
    pub fn open(path: &Path) -> AppResult<Db> {
        let conn = Connection::open(path)?;
        migrate(&conn)?;
        Ok(Db {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn toggle_favorite(&self, g: &Gallery) -> AppResult<bool> {
        let conn = self.lock();
        let exists = conn
            .query_row("SELECT 1 FROM favorites WHERE id = ?1", [g.id], |_| Ok(()))
            .optional()?
            .is_some();
        if exists {
            conn.execute("DELETE FROM favorites WHERE id = ?1", [g.id])?;
            Ok(false)
        } else {
            conn.execute(
                "INSERT INTO favorites (id, data, added_at) VALUES (?1, ?2, ?3)",
                params![g.id, to_json(g)?, now()],
            )?;
            Ok(true)
        }
    }

    pub fn remove_favorite(&self, id: i64) -> AppResult<()> {
        self.lock().execute("DELETE FROM favorites WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn favorite_ids(&self) -> AppResult<Vec<i64>> {
        let conn = self.lock();
        let mut stmt = conn.prepare("SELECT id FROM favorites")?;
        let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn list_favorites(&self) -> AppResult<Vec<Gallery>> {
        let conn = self.lock();
        let mut stmt = conn.prepare("SELECT data FROM favorites ORDER BY added_at DESC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(Result::ok).filter_map(|s| from_json(&s)).collect())
    }

    pub fn record_view(&self, g: &Gallery) -> AppResult<()> {
        let conn = self.lock();
        conn.execute(
            "INSERT OR REPLACE INTO history (id, data, viewed_at) VALUES (?1, ?2, ?3)",
            params![g.id, to_json(g)?, now()],
        )?;
        conn.execute(
            "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY viewed_at DESC LIMIT ?1)",
            [HISTORY_LIMIT],
        )?;
        Ok(())
    }

    pub fn list_history(&self) -> AppResult<Vec<Gallery>> {
        let conn = self.lock();
        let mut stmt = conn.prepare("SELECT data FROM history ORDER BY viewed_at DESC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(Result::ok).filter_map(|s| from_json(&s)).collect())
    }

    pub fn remove_history(&self, id: i64) -> AppResult<()> {
        self.lock().execute("DELETE FROM history WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn clear_history(&self) -> AppResult<()> {
        self.lock().execute("DELETE FROM history", [])?;
        Ok(())
    }

    pub fn set_progress(&self, id: i64, page: i64, total: i64) -> AppResult<()> {
        self.lock().execute(
            "INSERT OR REPLACE INTO progress (id, page, total, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, page, total, now()],
        )?;
        Ok(())
    }

    pub fn all_progress(&self) -> AppResult<Vec<Progress>> {
        let conn = self.lock();
        let mut stmt = conn.prepare("SELECT id, page, total FROM progress")?;
        let rows = stmt.query_map([], |row| {
            Ok(Progress {
                id: row.get(0)?,
                page: row.get(1)?,
                total: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }
}
