use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone)]
pub struct TimerRow {
    pub id: String,
    pub label: String,
    pub status: String, // "running" | "paused"
    pub elapsed_base_seconds: i64,
    pub started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TimerRow {
    /// Elapsed time as of `now`, derived from wall-clock time rather than an
    /// in-memory counter so it stays correct across app restarts and sleep.
    pub fn elapsed_seconds_at(&self, now: DateTime<Utc>) -> i64 {
        let running_delta = match (&self.status[..], self.started_at) {
            ("running", Some(started_at)) => (now - started_at).num_seconds().max(0),
            _ => 0,
        };
        self.elapsed_base_seconds + running_delta
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let started_at: Option<String> = row.get("started_at")?;
        let created_at: String = row.get("created_at")?;
        Ok(TimerRow {
            id: row.get("id")?,
            label: row.get("label")?,
            status: row.get("status")?,
            elapsed_base_seconds: row.get("elapsed_base_seconds")?,
            started_at: started_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

pub fn open(app: &AppHandle) -> Connection {
    let dir = app
        .path()
        .app_data_dir()
        .expect("could not resolve app data dir");
    std::fs::create_dir_all(&dir).expect("could not create app data dir");
    let conn = Connection::open(dir.join("tally.db")).expect("could not open tally.db");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS timers (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('running', 'paused')),
            elapsed_base_seconds INTEGER NOT NULL DEFAULT 0,
            started_at TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )
    .expect("could not create timers table");
    conn
}

pub fn list(conn: &Connection) -> Vec<TimerRow> {
    let mut stmt = conn
        .prepare("SELECT * FROM timers ORDER BY created_at ASC")
        .expect("prepare list");
    stmt.query_map([], TimerRow::from_row)
        .expect("query list")
        .filter_map(Result::ok)
        .collect()
}

pub fn get(conn: &Connection, id: &str) -> Option<TimerRow> {
    conn.query_row("SELECT * FROM timers WHERE id = ?1", [id], TimerRow::from_row)
        .ok()
}

pub fn find_running(conn: &Connection) -> Option<TimerRow> {
    conn.query_row(
        "SELECT * FROM timers WHERE status = 'running' LIMIT 1",
        [],
        TimerRow::from_row,
    )
    .ok()
}

pub fn insert(conn: &Connection, label: &str) -> TimerRow {
    let row = TimerRow {
        id: uuid::Uuid::new_v4().to_string(),
        label: label.to_string(),
        status: "paused".to_string(),
        elapsed_base_seconds: 0,
        started_at: None,
        created_at: Utc::now(),
    };
    conn.execute(
        "INSERT INTO timers (id, label, status, elapsed_base_seconds, started_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            row.id,
            row.label,
            row.status,
            row.elapsed_base_seconds,
            row.started_at.map(|d| d.to_rfc3339()),
            row.created_at.to_rfc3339(),
        ],
    )
    .expect("insert timer");
    row
}

pub fn set_running(conn: &Connection, id: &str, now: DateTime<Utc>) {
    conn.execute(
        "UPDATE timers SET status = 'running', started_at = ?2 WHERE id = ?1",
        rusqlite::params![id, now.to_rfc3339()],
    )
    .expect("set running");
}

pub fn set_paused(conn: &Connection, id: &str, elapsed_base_seconds: i64) {
    conn.execute(
        "UPDATE timers SET status = 'paused', started_at = NULL, elapsed_base_seconds = ?2 WHERE id = ?1",
        rusqlite::params![id, elapsed_base_seconds],
    )
    .expect("set paused");
}

pub fn rename(conn: &Connection, id: &str, label: &str) {
    conn.execute(
        "UPDATE timers SET label = ?2 WHERE id = ?1",
        rusqlite::params![id, label],
    )
    .expect("rename timer");
}

pub fn delete(conn: &Connection, id: &str) {
    conn.execute("DELETE FROM timers WHERE id = ?1", [id])
        .expect("delete timer");
}
