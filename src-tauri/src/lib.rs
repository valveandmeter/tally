mod commands;
mod commands_wrike;
mod db;
mod timer_engine;
mod tray;
mod wrike;

use std::sync::Mutex;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const POPOVER_LABEL: &str = "popover";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_timers,
            commands::create_timer,
            commands::start_timer,
            commands::pause_timer,
            commands::rename_timer,
            commands::discard_timer,
            commands_wrike::wrike_status,
            commands_wrike::connect_wrike,
            commands_wrike::disconnect_wrike,
            commands_wrike::refresh_wrike_cache,
            commands_wrike::search_wrike,
            commands_wrike::set_client_pinned,
            commands_wrike::log_timer,
            commands_wrike::confirm_catch_all_task,
            commands_wrike::get_wrike_totals,
            commands_wrike::pending_timelog_count,
            commands_wrike::list_pending_timelogs,
        ])
        .setup(|app| {
            let conn = db::open(app.handle());
            app.manage(Mutex::new(conn));

            // Menu-bar-only app: no Dock icon, no app-switcher entry.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let popover = WebviewWindowBuilder::new(app, POPOVER_LABEL, WebviewUrl::App("index.html".into()))
                .inner_size(380.0, 520.0)
                .decorations(false)
                .resizable(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .shadow(true)
                .transparent(true)
                .visible(false)
                .build()?;

            let popover_handle = popover.clone();
            popover.on_window_event(move |event| {
                if let WindowEvent::Focused(false) = event {
                    let _ = popover_handle.hide();
                }
            });

            tray::setup(app.handle())?;
            commands_wrike::start_queue_retry_loop(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
