use chrono::Utc;
use rusqlite::Connection;
use serde::Serialize;

use crate::db::{self, TimerRow};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimerDto {
    pub id: String,
    pub label: String,
    pub status: String,
    pub elapsed_seconds: i64,
    pub wrike_task_id: Option<String>,
    pub wrike_bucket_client_id: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutoPausedDto {
    pub label: String,
    pub elapsed_seconds: i64,
}

fn to_dto(row: &TimerRow, now: chrono::DateTime<Utc>) -> TimerDto {
    TimerDto {
        id: row.id.clone(),
        label: row.label.clone(),
        status: row.status.clone(),
        elapsed_seconds: row.elapsed_seconds_at(now),
        wrike_task_id: row.wrike_task_id.clone(),
        wrike_bucket_client_id: row.wrike_bucket_client_id.clone(),
    }
}

pub fn list_timers(conn: &Connection) -> Vec<TimerDto> {
    let now = Utc::now();
    db::list(conn).iter().map(|r| to_dto(r, now)).collect()
}

pub fn create_timer(
    conn: &Connection,
    label: &str,
    wrike_task_id: Option<&str>,
    wrike_bucket_client_id: Option<&str>,
) -> TimerDto {
    let row = db::insert(conn, label, wrike_task_id, wrike_bucket_client_id);
    to_dto(&row, Utc::now())
}

/// Starts or resumes `id`. If another timer was running, it is paused first
/// and its label + elapsed time at the moment of pause is returned so the
/// caller can show the auto-pause notice.
pub fn start_timer(conn: &Connection, id: &str) -> Option<AutoPausedDto> {
    let now = Utc::now();
    let auto_paused = match db::find_running(conn) {
        Some(running) if running.id != id => {
            let elapsed = running.elapsed_seconds_at(now);
            db::set_paused(conn, &running.id, elapsed);
            Some(AutoPausedDto {
                label: running.label,
                elapsed_seconds: elapsed,
            })
        }
        _ => None,
    };
    db::set_running(conn, id, now);
    auto_paused
}

pub fn pause_timer(conn: &Connection, id: &str) {
    if let Some(row) = db::get(conn, id) {
        if row.status == "running" {
            let elapsed = row.elapsed_seconds_at(Utc::now());
            db::set_paused(conn, id, elapsed);
        }
    }
}

pub fn rename_timer(conn: &Connection, id: &str, label: &str) {
    db::rename(conn, id, label);
}

pub fn discard_timer(conn: &Connection, id: &str) {
    db::delete(conn, id);
}
