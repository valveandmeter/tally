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
    pub wrike_task_id: Option<String>,
    pub wrike_bucket_client_id: Option<String>,
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
            wrike_task_id: row.get("wrike_task_id")?,
            wrike_bucket_client_id: row.get("wrike_bucket_client_id")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ClientRow {
    pub id: String,
    pub name: String,
    pub open_task_count: i64,
}

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: String,
    pub title: String,
    pub client_id: String,
    pub client_name: String,
    pub project_path: String,
    pub status: String,
    pub responsible_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WrikeAccount {
    pub host: String,
    pub name: String,
    pub contact_id: String,
    pub refreshed_at: Option<DateTime<Utc>>,
}

fn ensure_column(conn: &Connection, table: &str, column: &str, decl: &str) {
    let has_column = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .and_then(|mut stmt| {
            stmt.query_map([], |row| row.get::<_, String>(1))?
                .collect::<Result<Vec<_>, _>>()
        })
        .map(|cols| cols.iter().any(|c| c == column))
        .unwrap_or(false);
    if !has_column {
        conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"), [])
            .expect("add column");
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
    ensure_column(&conn, "timers", "wrike_task_id", "TEXT");
    ensure_column(&conn, "timers", "wrike_bucket_client_id", "TEXT");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS wrike_account (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            host TEXT NOT NULL,
            name TEXT NOT NULL,
            contact_id TEXT NOT NULL,
            refreshed_at TEXT
        )",
        [],
    )
    .expect("could not create wrike_account table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS wrike_clients (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            open_task_count INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )
    .expect("could not create wrike_clients table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS wrike_tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            client_id TEXT NOT NULL,
            client_name TEXT NOT NULL,
            project_path TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL,
            responsible_ids TEXT NOT NULL DEFAULT ''
        )",
        [],
    )
    .expect("could not create wrike_tasks table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS wrike_pinned_clients (
            client_id TEXT PRIMARY KEY
        )",
        [],
    )
    .expect("could not create wrike_pinned_clients table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS wrike_recent_tasks (
            task_id TEXT PRIMARY KEY,
            used_at TEXT NOT NULL
        )",
        [],
    )
    .expect("could not create wrike_recent_tasks table");

    conn
}

// --- Timers ---

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

pub fn insert(
    conn: &Connection,
    label: &str,
    wrike_task_id: Option<&str>,
    wrike_bucket_client_id: Option<&str>,
) -> TimerRow {
    let row = TimerRow {
        id: uuid::Uuid::new_v4().to_string(),
        label: label.to_string(),
        status: "paused".to_string(),
        elapsed_base_seconds: 0,
        started_at: None,
        created_at: Utc::now(),
        wrike_task_id: wrike_task_id.map(String::from),
        wrike_bucket_client_id: wrike_bucket_client_id.map(String::from),
    };
    conn.execute(
        "INSERT INTO timers (id, label, status, elapsed_base_seconds, started_at, created_at, wrike_task_id, wrike_bucket_client_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            row.id,
            row.label,
            row.status,
            row.elapsed_base_seconds,
            row.started_at.map(|d| d.to_rfc3339()),
            row.created_at.to_rfc3339(),
            row.wrike_task_id,
            row.wrike_bucket_client_id,
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

// --- Wrike account ---

pub fn get_wrike_account(conn: &Connection) -> Option<WrikeAccount> {
    conn.query_row(
        "SELECT host, name, contact_id, refreshed_at FROM wrike_account WHERE id = 1",
        [],
        |row| {
            let refreshed_at: Option<String> = row.get(3)?;
            Ok(WrikeAccount {
                host: row.get(0)?,
                name: row.get(1)?,
                contact_id: row.get(2)?,
                refreshed_at: refreshed_at.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                }),
            })
        },
    )
    .ok()
}

pub fn set_wrike_account(conn: &Connection, host: &str, name: &str, contact_id: &str) {
    conn.execute(
        "INSERT INTO wrike_account (id, host, name, contact_id, refreshed_at)
         VALUES (1, ?1, ?2, ?3, NULL)
         ON CONFLICT(id) DO UPDATE SET host = ?1, name = ?2, contact_id = ?3",
        rusqlite::params![host, name, contact_id],
    )
    .expect("set wrike account");
}

pub fn touch_wrike_refreshed_at(conn: &Connection) {
    conn.execute(
        "UPDATE wrike_account SET refreshed_at = ?1 WHERE id = 1",
        [Utc::now().to_rfc3339()],
    )
    .expect("touch refreshed_at");
}

pub fn clear_wrike_account(conn: &Connection) {
    conn.execute("DELETE FROM wrike_account WHERE id = 1", [])
        .expect("clear wrike account");
    conn.execute("DELETE FROM wrike_clients", []).expect("clear clients");
    conn.execute("DELETE FROM wrike_tasks", []).expect("clear tasks");
    conn.execute("DELETE FROM wrike_pinned_clients", [])
        .expect("clear pins");
    conn.execute("DELETE FROM wrike_recent_tasks", [])
        .expect("clear recents");
}

// --- Wrike cache ---

pub fn replace_clients(conn: &Connection, clients: &[ClientRow]) {
    let tx = conn.unchecked_transaction().expect("tx");
    tx.execute("DELETE FROM wrike_clients", []).expect("clear clients");
    for c in clients {
        tx.execute(
            "INSERT INTO wrike_clients (id, name, open_task_count) VALUES (?1, ?2, ?3)",
            rusqlite::params![c.id, c.name, c.open_task_count],
        )
        .expect("insert client");
    }
    tx.commit().expect("commit clients");
}

pub fn replace_tasks(conn: &Connection, tasks: &[TaskRow]) {
    let tx = conn.unchecked_transaction().expect("tx");
    tx.execute("DELETE FROM wrike_tasks", []).expect("clear tasks");
    for t in tasks {
        tx.execute(
            "INSERT INTO wrike_tasks (id, title, client_id, client_name, project_path, status, responsible_ids)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                t.id,
                t.title,
                t.client_id,
                t.client_name,
                t.project_path,
                t.status,
                t.responsible_ids.join(","),
            ],
        )
        .expect("insert task");
    }
    tx.commit().expect("commit tasks");
}

pub fn list_clients(conn: &Connection) -> Vec<ClientRow> {
    let mut stmt = conn
        .prepare("SELECT id, name, open_task_count FROM wrike_clients ORDER BY name ASC")
        .expect("prepare clients");
    stmt.query_map([], |row| {
        Ok(ClientRow {
            id: row.get(0)?,
            name: row.get(1)?,
            open_task_count: row.get(2)?,
        })
    })
    .expect("query clients")
    .filter_map(Result::ok)
    .collect()
}

pub fn list_tasks(conn: &Connection) -> Vec<TaskRow> {
    let mut stmt = conn
        .prepare("SELECT id, title, client_id, client_name, project_path, status, responsible_ids FROM wrike_tasks")
        .expect("prepare tasks");
    stmt.query_map([], |row| {
        let responsible_ids: String = row.get(6)?;
        Ok(TaskRow {
            id: row.get(0)?,
            title: row.get(1)?,
            client_id: row.get(2)?,
            client_name: row.get(3)?,
            project_path: row.get(4)?,
            status: row.get(5)?,
            responsible_ids: if responsible_ids.is_empty() {
                vec![]
            } else {
                responsible_ids.split(',').map(String::from).collect()
            },
        })
    })
    .expect("query tasks")
    .filter_map(Result::ok)
    .collect()
}

// --- Pins & recents ---

pub fn pinned_client_ids(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT client_id FROM wrike_pinned_clients")
        .expect("prepare pins");
    stmt.query_map([], |row| row.get(0))
        .expect("query pins")
        .filter_map(Result::ok)
        .collect()
}

pub fn set_client_pinned(conn: &Connection, client_id: &str, pinned: bool) {
    if pinned {
        conn.execute(
            "INSERT OR IGNORE INTO wrike_pinned_clients (client_id) VALUES (?1)",
            [client_id],
        )
        .expect("pin client");
    } else {
        conn.execute("DELETE FROM wrike_pinned_clients WHERE client_id = ?1", [client_id])
            .expect("unpin client");
    }
}

pub fn record_task_usage(conn: &Connection, task_id: &str) {
    conn.execute(
        "INSERT INTO wrike_recent_tasks (task_id, used_at) VALUES (?1, ?2)
         ON CONFLICT(task_id) DO UPDATE SET used_at = ?2",
        rusqlite::params![task_id, Utc::now().to_rfc3339()],
    )
    .expect("record recent task");
}

pub fn recent_task_ids(conn: &Connection, limit: i64) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT task_id FROM wrike_recent_tasks ORDER BY used_at DESC LIMIT ?1")
        .expect("prepare recents");
    stmt.query_map([limit], |row| row.get(0))
        .expect("query recents")
        .filter_map(Result::ok)
        .collect()
}
