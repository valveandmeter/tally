use std::sync::Mutex;
use std::time::Duration;

use chrono::Datelike;
use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands;
use crate::db;
use crate::wrike::client::WrikeClient;
use crate::wrike::logging::{self, LogOutcome};
use crate::wrike::search::{self, SearchFilters, SearchResults};
use crate::wrike::{auth, sync};

const DEFAULT_HOST: &str = "www.wrike.com";
const QUEUE_RETRY_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WrikeAccountDto {
    pub name: String,
    pub refreshed_at: Option<String>,
}

fn account_dto(account: &db::WrikeAccount) -> WrikeAccountDto {
    WrikeAccountDto {
        name: account.name.clone(),
        refreshed_at: account.refreshed_at.map(|t| t.to_rfc3339()),
    }
}

#[tauri::command]
pub async fn wrike_status(
    state: State<'_, Mutex<Connection>>,
) -> Result<Option<WrikeAccountDto>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    Ok(db::get_wrike_account(&conn).as_ref().map(account_dto))
}

#[tauri::command]
pub async fn connect_wrike(
    state: State<'_, Mutex<Connection>>,
    token: String,
) -> Result<WrikeAccountDto, String> {
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err("Paste a Wrike permanent access token first.".to_string());
    }

    let client = WrikeClient::new(DEFAULT_HOST, &token);
    let who = client.whoami().await.map_err(|e| e.to_string())?;
    let name = format!("{} {}", who.first_name, who.last_name)
        .trim()
        .to_string();
    let name = if name.is_empty() { "Connected".to_string() } else { name };

    auth::save_token(&token)?;

    {
        let conn = state.lock().map_err(|e| e.to_string())?;
        db::set_wrike_account(&conn, DEFAULT_HOST, &name, &who.id);
    }

    // Populate the cache right away so search works immediately.
    let (clients, tasks) = sync::fetch(&client).await?;
    {
        let conn = state.lock().map_err(|e| e.to_string())?;
        sync::apply(&conn, clients, tasks);
    }

    let conn = state.lock().map_err(|e| e.to_string())?;
    let account = db::get_wrike_account(&conn).ok_or("Lost account info right after saving it")?;
    Ok(account_dto(&account))
}

#[tauri::command]
pub async fn disconnect_wrike(state: State<'_, Mutex<Connection>>) -> Result<(), String> {
    auth::clear_token();
    let conn = state.lock().map_err(|e| e.to_string())?;
    db::clear_wrike_account(&conn);
    Ok(())
}

#[tauri::command]
pub async fn refresh_wrike_cache(state: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let (host, token) = current_credentials(&state)?;
    let client = WrikeClient::new(&host, &token);
    let (clients, tasks) = sync::fetch(&client).await?;
    let conn = state.lock().map_err(|e| e.to_string())?;
    sync::apply(&conn, clients, tasks);
    Ok(())
}

#[tauri::command]
pub async fn search_wrike(
    state: State<'_, Mutex<Connection>>,
    query: String,
    client_id: Option<String>,
    assigned_to_me: bool,
    include_completed: bool,
) -> Result<SearchResults, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let account = db::get_wrike_account(&conn);
    let my_contact_id = account.as_ref().map(|a| a.contact_id.as_str());
    let filters = SearchFilters {
        client_id,
        assigned_to_me,
        include_completed,
    };
    Ok(search::search(&conn, &query, filters, my_contact_id))
}

#[tauri::command]
pub async fn set_client_pinned(
    state: State<'_, Mutex<Connection>>,
    client_id: String,
    pinned: bool,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    db::set_client_pinned(&conn, &client_id, pinned);
    Ok(())
}

fn current_credentials(state: &State<'_, Mutex<Connection>>) -> Result<(String, String), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let account = db::get_wrike_account(&conn).ok_or("Not connected to Wrike")?;
    let token = auth::load_token().ok_or("No Wrike token found in the keychain — please reconnect")?;
    Ok((account.host, token))
}

/// Resolves the task to log to and, if already known, enqueues the log and
/// removes the timer — synchronous, no network call. Returns
/// `NeedsCatchAllConfirm` instead when a client-bucket timer needs its
/// catch-all task created first (see `confirm_catch_all_task`).
#[tauri::command]
pub fn log_timer(
    app: AppHandle,
    state: State<'_, Mutex<Connection>>,
    id: String,
) -> Result<LogOutcome, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let outcome = logging::prepare_log(&conn, &id)?;
    if matches!(outcome, LogOutcome::Logged { .. }) {
        commands::broadcast(&app, &conn);
    }
    Ok(outcome)
}

/// Creates the "[Client] – General time" catch-all task after the user has
/// confirmed it (CLAUDE.md: "Create it on first use after asking me to
/// confirm"). Call `log_timer` again afterward — it'll find the task cached.
#[tauri::command]
pub async fn confirm_catch_all_task(
    state: State<'_, Mutex<Connection>>,
    client_id: String,
) -> Result<(), String> {
    let (host, token) = current_credentials(&state)?;
    let title = {
        let conn = state.lock().map_err(|e| e.to_string())?;
        let client = db::get_client(&conn, &client_id).ok_or("That client is no longer in the cache")?;
        sync::catch_all_task_title(&client.name)
    };
    let client = WrikeClient::new(&host, &token);
    let task = client
        .create_task(&client_id, &title)
        .await
        .map_err(|e| e.to_string())?;
    let conn = state.lock().map_err(|e| e.to_string())?;
    db::set_catch_all_task_id(&conn, &client_id, &task.id);
    Ok(())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TotalsDto {
    pub today_hours: f64,
    pub week_hours: f64,
    pub pending_count: i64,
}

fn monday_of_this_week() -> String {
    let today = chrono::Local::now().date_naive();
    let days_since_monday = today.weekday().num_days_from_monday();
    (today - chrono::Duration::days(days_since_monday as i64))
        .format("%Y-%m-%d")
        .to_string()
}

/// Pulled from Wrike directly (not summed locally) per CLAUDE.md — this
/// reflects whatever's actually logged in the account, not just what this
/// app has submitted.
#[tauri::command]
pub async fn get_wrike_totals(state: State<'_, Mutex<Connection>>) -> Result<TotalsDto, String> {
    let (host, token) = current_credentials(&state)?;
    let client = WrikeClient::new(&host, &token);
    let today = logging::today_tracked_date();
    let week_start = monday_of_this_week();
    let today_hours = client
        .sum_my_timelog_hours(&today, &today)
        .await
        .map_err(|e| e.to_string())?;
    let week_hours = client
        .sum_my_timelog_hours(&week_start, &today)
        .await
        .map_err(|e| e.to_string())?;
    let pending_count = {
        let conn = state.lock().map_err(|e| e.to_string())?;
        db::count_pending_timelogs(&conn)
    };
    Ok(TotalsDto {
        today_hours,
        week_hours,
        pending_count,
    })
}

#[tauri::command]
pub fn pending_timelog_count(state: State<'_, Mutex<Connection>>) -> Result<i64, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    Ok(db::count_pending_timelogs(&conn))
}

/// Not wired into the UI yet (no queue-inspector screen until Phase 4's
/// Settings), but useful right now for checking why a specific entry hasn't
/// gone through yet — `attempts`/`lastError` carry the retry history.
#[tauri::command]
pub fn list_pending_timelogs(state: State<'_, Mutex<Connection>>) -> Result<Vec<db::PendingTimelog>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    Ok(db::list_pending_timelogs(&conn))
}

/// Retries queued timelogs every `QUEUE_RETRY_INTERVAL` while any are
/// pending. No server-side idempotency exists for this endpoint (verified
/// against Wrike's docs — a retried POST can't be told apart from a fresh
/// one), so a request that times out after Wrike actually accepted it is a
/// known, unresolved double-log risk; this only retries on a clear error
/// response, not on ambiguous network timeouts within the same tick.
pub fn start_queue_retry_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(QUEUE_RETRY_INTERVAL);
        loop {
            interval.tick().await;
            let state = app.state::<Mutex<Connection>>();

            let pending = {
                let Ok(conn) = state.lock() else { continue };
                db::list_pending_timelogs(&conn)
            };
            if pending.is_empty() {
                continue;
            }

            let account = {
                let Ok(conn) = state.lock() else { continue };
                db::get_wrike_account(&conn)
            };
            let (Some(account), Some(token)) = (account, auth::load_token()) else {
                continue;
            };
            let client = WrikeClient::new(&account.host, &token);

            let mut any_succeeded = false;
            for entry in pending {
                let hours = entry.minutes as f64 / 60.0;
                let result = client
                    .create_timelog(&entry.wrike_task_id, hours, &entry.tracked_date, &entry.comment)
                    .await;
                let Ok(conn) = state.lock() else { continue };
                match result {
                    Ok(()) => {
                        db::remove_pending_timelog(&conn, &entry.id);
                        any_succeeded = true;
                    }
                    Err(e) => {
                        db::mark_pending_timelog_attempt(&conn, &entry.id, &e.to_string());
                    }
                }
            }
            if any_succeeded {
                let _ = app.emit("queue://changed", ());
            }
        }
    });
}
