use rusqlite::Connection;
use serde::Serialize;

use crate::db;
use crate::wrike::sync::catch_all_task_title;

/// Always rounds up to the next 5 minutes, minimum 5 — fixed, not a setting
/// (see CLAUDE.md "Wrike integration" / "Rounding").
pub fn round_up_to_5_minutes(elapsed_seconds: i64) -> i64 {
    let periods = (elapsed_seconds.max(0) + 299) / 300;
    (periods * 5).max(5)
}

pub fn today_tracked_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LogOutcome {
    Logged { label: String, minutes: i64 },
    NeedsCatchAllConfirm {
        client_id: String,
        client_name: String,
        suggested_title: String,
    },
}

/// Resolves which Wrike task a timer's time should log to, and if that's
/// already known, enqueues the log and removes the timer (both synchronous
/// — no network call happens here). Returns `NeedsCatchAllConfirm` instead
/// when the timer is a client bucket with no catch-all task cached yet; the
/// caller must create that task (a network call) and retry.
pub fn prepare_log(conn: &Connection, timer_id: &str) -> Result<LogOutcome, String> {
    let timer = db::get(conn, timer_id).ok_or("Timer not found")?;
    let minutes = round_up_to_5_minutes(timer.elapsed_seconds_at(chrono::Utc::now()));

    let task_id = if let Some(task_id) = &timer.wrike_task_id {
        task_id.clone()
    } else if let Some(client_id) = &timer.wrike_bucket_client_id {
        let client = db::get_client(conn, client_id).ok_or("That client is no longer in the cache")?;
        match client.catch_all_task_id {
            Some(task_id) => task_id,
            None => {
                return Ok(LogOutcome::NeedsCatchAllConfirm {
                    client_id: client.id,
                    client_name: client.name.clone(),
                    suggested_title: catch_all_task_title(&client.name),
                })
            }
        }
    } else {
        return Err("This timer isn't linked to a Wrike task or client yet".to_string());
    };

    db::enqueue_timelog(
        conn,
        &task_id,
        minutes,
        &today_tracked_date(),
        &timer.label,
        &timer.label,
    );
    db::delete(conn, timer_id);

    Ok(LogOutcome::Logged {
        label: timer.label,
        minutes,
    })
}
