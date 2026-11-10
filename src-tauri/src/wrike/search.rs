use serde::Serialize;

use crate::db::{self, ClientRow, TaskRow};
use rusqlite::Connection;

const RECENT_TASKS_LIMIT: i64 = 5;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientResult {
    pub id: String,
    pub name: String,
    pub open_task_count: i64,
    pub pinned: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskResult {
    pub id: String,
    pub title: String,
    pub client_id: String,
    pub client_name: String,
    pub project_path: String,
    pub status: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub clients: Vec<ClientResult>,
    pub tasks: Vec<TaskResult>,
}

#[derive(Default)]
pub struct SearchFilters {
    pub client_id: Option<String>,
    pub assigned_to_me: bool,
    pub include_completed: bool,
}

fn task_result(t: &TaskRow) -> TaskResult {
    TaskResult {
        id: t.id.clone(),
        title: t.title.clone(),
        client_id: t.client_id.clone(),
        client_name: t.client_name.clone(),
        project_path: t.project_path.clone(),
        status: t.status.clone(),
    }
}

/// Lower score = better match. `None` means the candidate doesn't match at
/// all (every query token must appear somewhere in the haystack).
fn fuzzy_score(haystack: &str, query_tokens: &[String]) -> Option<i64> {
    let haystack = haystack.to_lowercase();
    let mut score = 0i64;
    for token in query_tokens {
        let pos = haystack.find(token.as_str())?;
        let word_boundary = pos == 0 || haystack.as_bytes()[pos - 1] == b' ';
        score += pos as i64 - if word_boundary { 5 } else { 0 };
    }
    Some(score)
}

fn passes_task_filters(t: &TaskRow, filters: &SearchFilters, my_contact_id: Option<&str>) -> bool {
    if let Some(client_id) = &filters.client_id {
        if &t.client_id != client_id {
            return false;
        }
    }
    if !filters.include_completed && t.status != "Active" {
        return false;
    }
    if filters.assigned_to_me {
        match my_contact_id {
            Some(id) => {
                if !t.responsible_ids.iter().any(|r| r == id) {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

pub fn search(
    conn: &Connection,
    query: &str,
    filters: SearchFilters,
    my_contact_id: Option<&str>,
) -> SearchResults {
    let pinned_ids = db::pinned_client_ids(conn);
    let all_clients = db::list_clients(conn);
    let all_tasks = db::list_tasks(conn);

    let query = query.trim().to_lowercase();

    if query.is_empty() {
        let clients = all_clients
            .iter()
            .filter(|c| pinned_ids.iter().any(|p| p == &c.id))
            .map(|c| to_client_result(c, true))
            .collect();

        let recent_ids = db::recent_task_ids(conn, RECENT_TASKS_LIMIT);
        let tasks = recent_ids
            .iter()
            .filter_map(|id| all_tasks.iter().find(|t| &t.id == id))
            .filter(|t| passes_task_filters(t, &filters, my_contact_id))
            .map(|t| task_result(t))
            .collect();

        return SearchResults { clients, tasks };
    }

    let tokens: Vec<String> = query.split_whitespace().map(String::from).collect();

    let mut scored_clients: Vec<(i64, &ClientRow)> = all_clients
        .iter()
        .filter_map(|c| fuzzy_score(&c.name, &tokens).map(|s| (s, c)))
        .collect();
    scored_clients.sort_by_key(|(s, c)| (*s, c.name.len()));
    let clients = scored_clients
        .into_iter()
        .map(|(_, c)| to_client_result(c, pinned_ids.iter().any(|p| p == &c.id)))
        .collect();

    let mut scored_tasks: Vec<(i64, &TaskRow)> = all_tasks
        .iter()
        .filter(|t| passes_task_filters(t, &filters, my_contact_id))
        .filter_map(|t| {
            let haystack = format!("{} {} {}", t.client_name, t.project_path, t.title);
            fuzzy_score(&haystack, &tokens).map(|s| (s, t))
        })
        .collect();
    scored_tasks.sort_by_key(|(s, t)| (*s, t.title.len()));
    let tasks = scored_tasks.into_iter().map(|(_, t)| task_result(t)).collect();

    SearchResults { clients, tasks }
}

fn to_client_result(c: &ClientRow, pinned: bool) -> ClientResult {
    ClientResult {
        id: c.id.clone(),
        name: c.name.clone(),
        open_task_count: c.open_task_count,
        pinned,
    }
}
