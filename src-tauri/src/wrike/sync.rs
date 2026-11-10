use std::collections::HashMap;

use rusqlite::Connection;

use crate::db::{self, ClientRow, TaskRow};
use crate::wrike::client::WrikeClient;

/// Name of the Wrike space whose top-level folders are treated as clients.
/// Matches CLAUDE.md's assumption; a picker for this lives in Settings
/// (Phase 4) — hardcoded here for now.
const CLIENTS_SPACE_NAME: &str = "Clients";

const CATCH_ALL_SUFFIX: &str = " – General time";

/// Network fetch only — no `Connection` involved, so this future stays
/// `Send` across its `.await` points regardless of how the caller is
/// holding a (non-`Sync`) sqlite connection elsewhere.
pub async fn fetch(client: &WrikeClient) -> Result<(Vec<ClientRow>, Vec<TaskRow>), String> {
    let spaces = client.list_spaces().await.map_err(|e| e.to_string())?;
    let clients_space = spaces
        .iter()
        .find(|s| s.title.eq_ignore_ascii_case(CLIENTS_SPACE_NAME))
        .ok_or_else(|| {
            format!(
                "No Wrike space named \"{CLIENTS_SPACE_NAME}\" was found. Check the space name in your Wrike account (or we'll need a Settings picker for this sooner than planned)."
            )
        })?;

    let client_folders = client
        .list_space_folders(&clients_space.id)
        .await
        .map_err(|e| e.to_string())?;
    // Only direct children of the space are "clients" — not nested folders.
    let top_level: Vec<_> = client_folders
        .into_iter()
        .filter(|f| f.parent_ids.iter().any(|p| p == &clients_space.id))
        .collect();

    let mut client_rows = Vec::new();
    let mut task_rows = Vec::new();

    for folder in &top_level {
        if folder.title.ends_with(CATCH_ALL_SUFFIX) {
            // Catch-all tasks live *inside* a client folder as a task, not
            // as a folder themselves — this guard is defensive, not
            // expected to trigger, but keeps catch-all folders (if any
            // exist from a prior naming mismatch) out of the client list.
            continue;
        }

        // One extra call per client to resolve a single level of "project"
        // breadcrumb. Deeper nesting falls back to just "Client › Task".
        let subfolders = client
            .list_folder_subfolders(&folder.id)
            .await
            .unwrap_or_default();
        let project_names: HashMap<String, String> = subfolders
            .into_iter()
            .map(|f| (f.id, f.title))
            .collect();

        let tasks = client
            .list_folder_tasks(&folder.id)
            .await
            .map_err(|e| e.to_string())?;

        let open_task_count = tasks
            .iter()
            .filter(|t| t.status == "Active" && !t.title.ends_with(CATCH_ALL_SUFFIX))
            .count() as i64;

        client_rows.push(ClientRow {
            id: folder.id.clone(),
            name: folder.title.clone(),
            open_task_count,
        });

        for task in tasks {
            if task.title.ends_with(CATCH_ALL_SUFFIX) {
                // Catch-all tasks are a logging target, not a search result.
                continue;
            }
            let project_path = task
                .parent_ids
                .iter()
                .find_map(|id| project_names.get(id).cloned())
                .unwrap_or_default();
            task_rows.push(TaskRow {
                id: task.id,
                title: task.title,
                client_id: folder.id.clone(),
                client_name: folder.title.clone(),
                project_path,
                status: task.status,
                responsible_ids: task.responsible_ids,
            });
        }
    }

    Ok((client_rows, task_rows))
}

/// Synchronous — writes a previously-fetched snapshot into the cache. Call
/// this with no `.await` anywhere near it so the sqlite connection is never
/// held across a suspension point.
pub fn apply(conn: &Connection, clients: Vec<ClientRow>, tasks: Vec<TaskRow>) {
    db::replace_clients(conn, &clients);
    db::replace_tasks(conn, &tasks);
    db::touch_wrike_refreshed_at(conn);
}
