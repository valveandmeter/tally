use std::sync::Mutex;

use rusqlite::Connection;
use serde::Serialize;
use tauri::State;

use crate::db;
use crate::wrike::client::WrikeClient;
use crate::wrike::search::{self, SearchFilters, SearchResults};
use crate::wrike::{auth, sync};

const DEFAULT_HOST: &str = "www.wrike.com";

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
