//! Local SQLite persistence: messages, sessions, settings.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StoredMessage {
    pub id: i64,
    pub session: String,
    pub sender: String,
    pub text: String,
    pub kind: String,
    pub ts: i64,
    pub msgid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRow {
    pub name: String,
    pub kind: String,
    pub last_ts: i64,
    pub muted: bool,
}

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session TEXT NOT NULL,
                sender TEXT NOT NULL,
                text TEXT NOT NULL,
                kind TEXT NOT NULL,
                ts INTEGER NOT NULL,
                msgid TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_messages_session_ts ON messages(session, ts);
            CREATE TABLE IF NOT EXISTS sessions (
                name TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                last_ts INTEGER NOT NULL DEFAULT 0,
                muted INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn append_message(
        &self,
        session: &str,
        sender: &str,
        text: &str,
        kind: &str,
        ts: i64,
        msgid: Option<&str>,
    ) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (session, sender, text, kind, ts, msgid) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![session, sender, text, kind, ts, msgid],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn history(
        &self,
        session: &str,
        before_ts: Option<i64>,
        limit: u32,
    ) -> rusqlite::Result<Vec<StoredMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session, sender, text, kind, ts, msgid FROM messages
             WHERE session = ?1 AND (?2 IS NULL OR ts < ?2)
             ORDER BY ts DESC, id DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(rusqlite::params![session, before_ts, limit], |row| {
            Ok(StoredMessage {
                id: row.get(0)?,
                session: row.get(1)?,
                sender: row.get(2)?,
                text: row.get(3)?,
                kind: row.get(4)?,
                ts: row.get(5)?,
                msgid: row.get(6)?,
            })
        })?;
        let mut out: Vec<StoredMessage> = rows.collect::<Result<_, _>>()?;
        out.reverse();
        Ok(out)
    }

    pub fn search(
        &self,
        session: &str,
        query: &str,
        limit: u32,
    ) -> rusqlite::Result<Vec<StoredMessage>> {
        let conn = self.conn.lock().unwrap();
        let escaped = query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        let mut stmt = conn.prepare(
            "SELECT id, session, sender, text, kind, ts, msgid FROM messages
             WHERE session = ?1 AND text LIKE ?2 ESCAPE '\\'
             ORDER BY ts DESC, id DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(rusqlite::params![session, pattern, limit], |row| {
            Ok(StoredMessage {
                id: row.get(0)?,
                session: row.get(1)?,
                sender: row.get(2)?,
                text: row.get(3)?,
                kind: row.get(4)?,
                ts: row.get(5)?,
                msgid: row.get(6)?,
            })
        })?;
        let mut out: Vec<StoredMessage> = rows.collect::<Result<_, _>>()?;
        out.reverse();
        Ok(out)
    }

    pub fn touch_session(&self, name: &str, kind: &str, ts: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (name, kind, last_ts) VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET last_ts = ?3",
            rusqlite::params![name, kind, ts],
        )?;
        Ok(())
    }

    pub fn sessions(&self) -> rusqlite::Result<Vec<SessionRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT name, kind, last_ts, muted FROM sessions ORDER BY last_ts DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(SessionRow {
                name: row.get(0)?,
                kind: row.get(1)?,
                last_ts: row.get(2)?,
                muted: row.get::<_, i64>(3)? != 0,
            })
        })?;
        rows.collect()
    }

    pub fn set_muted(&self, name: &str, muted: bool) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET muted = ?2 WHERE name = ?1",
            rusqlite::params![name, muted as i64],
        )?;
        Ok(())
    }

    pub fn delete_session(&self, name: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM sessions WHERE name = ?1",
            rusqlite::params![name],
        )?;
        Ok(())
    }

    pub fn setting_get(&self, key: &str) -> rusqlite::Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(rusqlite::params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn setting_set(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }
}
