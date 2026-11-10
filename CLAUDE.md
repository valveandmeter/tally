# Tally — Project Instructions for Claude Code

The app is called **Tally**. It's an internal-only tool for our team and a sister app to CopperOS. Use the name Tally for the window title, tray/menu bar tooltip, notifications, and installers.

You are building a lightweight, cross-platform desktop time tracker that logs time directly to Wrike: a TinyStopwatch-style menu bar app (macOS) / system tray app (Windows).

Read these before writing code:
- `docs/BUILD_PLAN.md` — phases, tasks, "done when" criteria, testing checklist, risks.
- `design/README.md` — screen-by-screen design reference, colors, and how to read the mockup files in `design/screens/`.
- `design/tokens.css` — the color and type tokens to use as the app's theme variables.

## Working rules
- Work one phase at a time, in order (see "Phases" below). At the start of each phase, summarize what you'll build and wait for my OK.
- At the end of each phase, tell me exactly how to run and test it on macOS and Windows, list anything I must set up manually, and update the checkboxes in `docs/BUILD_PLAN.md`.
- Verify Wrike API endpoints and field names against the current Wrike API v4 docs before using them. Do not assume.
- Never commit secrets. `.env` is for local development only; the app stores Wrike tokens in the OS keychain.
- Keep the Wrike client, timer engine, and storage in separate modules.
- Use the colors and tokens from `design/tokens.css` as CSS variables from day one. No hard-coded colors in components.
- Commit to git at the end of each working step with a clear message.

## Goals
- Minimal, fast, always-available timers in the macOS menu bar and the Windows system tray.
- Keep several timers on Wrike tasks or client buckets, with only one running at a time, then submit each as a Wrike timelog with one click.
- One codebase for macOS (Apple Silicon + Intel) and Windows 10/11.

## Tech stack
- Tauri 2 (Rust backend + lightweight web frontend). If another option is meaningfully better, explain why before starting.
- Frontend: TypeScript with Svelte or React, plain CSS.
- Local storage: SQLite for timers, history, cached Wrike data, and an offline queue.
- Store Wrike credentials in the OS keychain (macOS Keychain / Windows Credential Manager).
- Keep timer state in the backend so the menu bar, popover, and mini timer always agree.

## Core timer features
- macOS: show elapsed time as text in the menu bar with a small dot while running; click to open a popover.
- Windows: elapsed time in the tray tooltip, plus an optional always-on-top floating mini timer (task path, elapsed time, pause button, open-full-timer button), draggable anywhere.
- Pause, resume, reset, and manually edit elapsed time before submitting.
- Global keyboard shortcuts (configurable): start/pause the most recently used timer, and open search.
- Idle detection: after N idle minutes, a prompt to keep, discard, or subtract idle time from the running timer.
- Reminder if no timer runs for X minutes during work hours.
- Launch at login (optional).
- Time format setting: 12-hour (default, e.g., 2:30 PM) or 24-hour (e.g., 14:30), applied to every clock time shown (work hours, entry times, notifications, history). Elapsed durations like 0:42:13 are unaffected.
- Timers persist across restarts and sleep.

## Multiple timers (only one running)
- Several timers can exist at once, each tied to its own task or client bucket with its own elapsed time and note, but ONLY ONE can run at a time. Starting or resuming a timer automatically pauses the one that was running.
- Popover shows a list: each row has the Client › Project path, task name, elapsed time, and Pause/Resume, Log to Wrike, and discard (confirm if over a minute). The running row is highlighted; paused rows are muted. Summary at top ("1 running · 2 paused").
- "+ New timer" opens search.
- When a timer is auto-paused, show a brief notice in the popover ("Paused [task] at 0:42:13"). No OS notification and no confirmation needed.
- Menu bar: shows the running timer's time; with none running, a paused icon.
- Windows mini timer shows the running timer, or the most recently paused one when none is running.

## Popover layout (~380px wide)
1. Header: "Timers" and a settings button.
2. Auto-pause notice when shown.
3. Timer list.
4. "+ New timer" button.
5. Footer: today's total, this week's total, pending count when the offline queue isn't empty.

After logging, show a brief confirmation ("Logged 0:45 to Wrike · [task]") and remove that timer. No timelog category picker and no rounded-amount preview.

## Wrike integration
- Wrike REST API v4. Verify endpoint paths and field names against the current docs rather than assuming.
- Auth: OAuth 2.0, plus an option to paste a permanent access token. Use the "host" value from the token response for all calls.
- Submitting: create a timelog on the task with hours, tracked date, and the note as comment. No timelog categories.
- Rounding: always round up to the next 5 minutes on submit (minimum 5). Fixed, not a setting.
- Offline queue with automatic retry and a visible pending indicator.
- Today and this week totals from Wrike.
- Handle rate limits and token refresh with clear messages.

## Search: client buckets and tasks
My Wrike account has a space called "Clients". Each client is a folder directly inside it; that client's projects, subfolders, and tasks live inside its folder. A "client bucket" means one of these top-level folders.
- One search bar with type-ahead (debounced ~250ms) and fuzzy matching ("acme web" finds "Acme Corp › Website Redesign").
- Results grouped into "Clients" and "Tasks".
- Each client result shows its open task count and has two actions: clicking the name filters to that client's tasks (removable chip in the search bar), and a "Track time" button selects the client bucket as the timer target.
- Task results show Client › Project › Task plus status; the current selection is highlighted.
- Filters: assigned to me; include completed (default off).
- Before typing: pinned clients and recently tracked tasks.
- Keyboard: arrows to move, Enter to select, Esc to clear the client filter.

Implementation:
- On connect, find the "Clients" space (selectable in Settings, default "Clients") and load its top-level folders as clients; new folders appear on next refresh.
- Fetch a client's tasks including all descendants.
- Cache clients and recent tasks locally; search the cache first; refresh in the background and on demand. Don't call the API on every keystroke.

## Client buckets (no task)
- When a client filter is active, show "Track to [Client] (no task)" at the top of its task list.
- Timelogs attach to tasks, not folders (verify), so bucket time logs to a catch-all task in the client's folder, default name "[Client] – General time" (configurable). Create it on first use after asking me to confirm.
- Exclude catch-all tasks from normal search results.
- Show the bucket as "[Client] (no task)" on timers.

## Settings
- Wrike: connected as [name/email] with Disconnect; Clients space picker; catch-all task name; cache refresh interval plus "Refresh now".
- Tracking: rounding shown read-only ("Always rounds up to 5 min"); idle threshold; reminder interval; work hours (default Mon–Fri 8:00 AM–6:00 PM); time format (12-hour default / 24-hour).
- Appearance: link to Customize colors, showing current accent and theme.
- Shortcuts & startup: start/pause, open search, launch at login, show floating mini timer (Windows).

## Customize colors (Settings › Appearance)
- Live preview of a mini menu bar and timer row that updates instantly.
- Theme: Light, Dark, Match system (default).
- Accent: six swatches (brown #955F3A default, blue #1D4ED8, violet #6D28D9, rose #BE123C, orange #C2410C, slate #334155) plus custom hex; warn if white text on it is below 4.5:1 contrast.
- Menu bar display (macOS): Time only, Time + task name, or Icon only; on Windows this sets the mini timer's compact mode.
- "Reset to defaults". Changes apply everywhere instantly and persist.

## Visual style
See `design/tokens.css` and `design/README.md`.
- Inter with system fallbacks; popover with soft shadow and 14px corners; gray secondary text.
- Light: white panels; accent #955F3A for primary buttons, running indicators, selection borders, with light warm tints for selected rows; near-black #262626 for primary text and the "Log to Wrike" button.
- Dark: #262626 panels on a darker backdrop, #2E2E2E unselected rows, #404040 borders, #F5F5F5 text, #A3A3A3 secondary text; keep #955F3A for button fills and selection borders, with light tan #D9A47E for accent-colored text and links; "Log to Wrike" becomes #F5F5F5 with #262626 text. For custom accents, derive a lighter text tint the same way.
- No splash screens or unnecessary animations.

## Phases
0. Propose the project structure and confirm the stack. Scaffold Tauri 2 and confirm a blank app runs. Wait for my OK.
1. Local multi-timer app in the tray/menu bar on both platforms, with persistence, one-running-timer switching, and Light/Dark themes (no Wrike yet).
2. Wrike auth, client list, search, and client buckets.
3. Logging timelogs, 5-minute rounding, catch-all tasks, offline queue, totals.
4. Idle detection, reminders, shortcuts, Windows mini timer, Settings, Customize colors, launch at login.
5. Installers for internal use: .dmg for macOS and .msi/.exe for Windows, plus auto-updates. This is internal only, so no public distribution; ask me before setting up paid code signing, and explain the trade-off (unsigned apps show security warnings on first launch).

## About me
I'm [describe your experience level], so explain anything platform-specific I'll need to handle myself.
