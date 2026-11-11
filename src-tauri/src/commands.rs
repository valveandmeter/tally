use std::sync::Mutex;

use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::timer_engine::{self, TimerDto};

pub const TRAY_ID: &str = "main-tray";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TimersState {
    timers: Vec<TimerDto>,
    running_id: Option<String>,
}

pub fn format_elapsed(total_seconds: i64) -> String {
    let total_seconds = total_seconds.max(0);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours}:{minutes:02}:{seconds:02}")
}

/// Recomputes current timer state and pushes it to both the popover (via
/// event) and the tray icon, so every surface always agrees. `pub(crate)`
/// so other command modules (e.g. commands_wrike's log_timer) can trigger a
/// re-broadcast after they mutate timers outside this module.
pub(crate) fn broadcast(app: &AppHandle, conn: &Connection) {
    let timers = timer_engine::list_timers(conn);
    let running = timers.iter().find(|t| t.status == "running").cloned();

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        match &running {
            Some(t) => {
                let elapsed = format_elapsed(t.elapsed_seconds);
                let _ = tray.set_title(Some(format!("{elapsed} \u{25CF}")));
                let _ = tray.set_tooltip(Some(format!("Tally — {} — {elapsed}", t.label)));
            }
            None => {
                let _ = tray.set_title(None::<&str>);
                let _ = tray.set_tooltip(Some("Tally — no timer running"));
            }
        }
    }

    let running_id = running.map(|t| t.id);
    let _ = app.emit(
        "timers://state",
        TimersState {
            timers,
            running_id,
        },
    );
}

#[tauri::command]
pub fn list_timers(state: State<'_, Mutex<Connection>>) -> Result<Vec<TimerDto>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    Ok(timer_engine::list_timers(&conn))
}

#[tauri::command]
pub fn create_timer(
    app: AppHandle,
    state: State<'_, Mutex<Connection>>,
    label: String,
    wrike_task_id: Option<String>,
    wrike_bucket_client_id: Option<String>,
) -> Result<TimerDto, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let dto = timer_engine::create_timer(
        &conn,
        &label,
        wrike_task_id.as_deref(),
        wrike_bucket_client_id.as_deref(),
    );
    if let Some(task_id) = &dto.wrike_task_id {
        crate::db::record_task_usage(&conn, task_id);
    }
    broadcast(&app, &conn);
    Ok(dto)
}

#[tauri::command]
pub fn start_timer(
    app: AppHandle,
    state: State<'_, Mutex<Connection>>,
    id: String,
) -> Result<Option<timer_engine::AutoPausedDto>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let auto_paused = timer_engine::start_timer(&conn, &id);
    broadcast(&app, &conn);
    Ok(auto_paused)
}

#[tauri::command]
pub fn pause_timer(
    app: AppHandle,
    state: State<'_, Mutex<Connection>>,
    id: String,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    timer_engine::pause_timer(&conn, &id);
    broadcast(&app, &conn);
    Ok(())
}

#[tauri::command]
pub fn rename_timer(
    app: AppHandle,
    state: State<'_, Mutex<Connection>>,
    id: String,
    label: String,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    timer_engine::rename_timer(&conn, &id, &label);
    broadcast(&app, &conn);
    Ok(())
}

#[tauri::command]
pub fn discard_timer(
    app: AppHandle,
    state: State<'_, Mutex<Connection>>,
    id: String,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    timer_engine::discard_timer(&conn, &id);
    broadcast(&app, &conn);
    Ok(())
}

/// Called once a second by the ticker in tray.rs while a timer is running.
pub fn tick(app: &AppHandle, state: &Mutex<Connection>) {
    if let Ok(conn) = state.lock() {
        broadcast(app, &conn);
    }
}
