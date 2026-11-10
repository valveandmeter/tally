use std::sync::Mutex;
use std::time::Duration;

use rusqlite::Connection;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Rect,
};

use crate::commands::{self, TRAY_ID};
use crate::db;

const POPOVER_LABEL: &str = "popover";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let icon = app.default_window_icon().cloned();
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Tally — no timer running")
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_popover(app, Some(rect));
            }
        });
    if let Some(icon) = icon {
        builder = builder.icon(icon);
    }
    builder.build(app)?;

    start_ticker(app.clone());
    Ok(())
}

fn toggle_popover(app: &AppHandle, tray_rect: Option<Rect>) {
    let Some(window) = app.get_webview_window(POPOVER_LABEL) else {
        return;
    };
    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        let _ = window.hide();
        return;
    }
    if let (Some(rect), Ok(win_size)) = (tray_rect, window.outer_size()) {
        let scale = window.scale_factor().unwrap_or(1.0);
        let pos = rect.position.to_physical::<i32>(scale);
        let size = rect.size.to_physical::<u32>(scale);
        let x = pos.x as f64 + (size.width as f64 / 2.0) - (win_size.width as f64 / 2.0);
        let y = pos.y as f64 + size.height as f64 + 4.0;
        let _ = window.set_position(PhysicalPosition::new(x.max(0.0) as i32, y.max(0.0) as i32));
    }
    let _ = window.show();
    let _ = window.set_focus();
}

/// Ticks once a second so the tray text and popover stay live while a timer
/// runs. Skips the broadcast entirely when nothing is running (idle, cheap).
/// Runs on a plain OS thread rather than the async runtime — the ticker body
/// is synchronous (a mutex lock + sqlite query), so there's nothing to gain
/// from async here, and it avoids depending on tokio's API directly.
fn start_ticker(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let state = app.state::<Mutex<Connection>>();
        let should_tick = state
            .lock()
            .map(|conn| db::find_running(&conn).is_some())
            .unwrap_or(false);
        if should_tick {
            commands::tick(&app, &state);
        }
    });
}
