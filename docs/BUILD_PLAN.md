# Tally — Build Plan

Build a TinyStopwatch-style desktop timer for macOS and Windows that logs time straight to Wrike, in five phases using Claude Code. The full spec is in `CLAUDE.md`; the design reference is in `design/`.

## Before you start

- [ ] Install Claude Code (desktop app or terminal).
- [ ] Install Node.js (LTS) and Rust (via rustup). Claude Code can walk you through this.
- [ ] macOS: install Xcode Command Line Tools (`xcode-select --install`).
- [ ] Windows: install Microsoft C++ Build Tools and WebView2 (usually already on Windows 11).
- [ ] Have access to both a Mac and a Windows PC (or a Windows VM) for testing.
- [ ] Create a GitHub repository for the project so both machines share the code.
- [ ] In Wrike, register an API app (Apps & Integrations → API) for an OAuth client ID and secret. A permanent access token from the same page is enough to start.
- [ ] Confirm your Wrike account has a space named "Clients" with one folder per client.
- [ ] Decide on the catch-all task name for client buckets (default "[Client] – General time").
- [ ] Optional, internal only: an Apple Developer account ($99/year) and a Windows code signing certificate, only if you want to avoid first-launch security warnings for teammates.

## Tech stack

| Layer | Choice | Why |
| --- | --- | --- |
| App shell | Tauri 2 (Rust) | Tray/menu bar, global shortcuts, notifications, launch at login, auto-updates via official plugins |
| UI | TypeScript + Svelte (or React), plain CSS | Small and fast; colors as CSS variables for themes and custom accents |
| Local data | SQLite | Timers, history, cached clients and tasks, offline queue |
| Credentials | OS keychain | Wrike tokens never stored in plain files |
| Wrike | REST API v4 | OAuth 2.0 plus permanent token option; use the account's "host" from the token response |

## Phase 0: Project setup

- [x] Review Claude Code's proposed project structure.
- [x] Scaffold the Tauri 2 project and confirm a blank window runs on Mac.
- [ ] Confirm a blank window runs on Windows.
- [ ] Push to GitHub (deferred — repo is local-only for now, by choice).

**Done when:** the empty app launches on both machines from the same code.

## Phase 1: Local timers

- [ ] Menu bar item showing elapsed time on macOS; tray icon with tooltip on Windows.
- [ ] Popover with the timer list: Pause/Resume, discard, and "+ New timer" (placeholder task for now).
- [ ] Multiple timers, each with its own time and note; only one runs at a time, and starting or resuming one pauses the running one.
- [ ] Brief in-popover notice when a timer is auto-paused ("Paused Homepage hero revisions at 0:42:13").
- [ ] Menu bar always shows the running timer's time, or a paused icon when none is running.
- [ ] Timers persist across restarts and sleep (SQLite).
- [ ] All colors defined as theme variables, with Light and Dark working from the start.

**Done when:** you can switch between three timers with only one ever running, quit the app, reopen it, and find them intact on both platforms.

## Phase 2: Wrike connection and search

- [ ] Connect via permanent token first, then OAuth; store credentials in the keychain.
- [ ] Load the Clients space and its folders as the client list; cache locally.
- [ ] Type-ahead search with Clients and Tasks groups, client filter chip, "Assigned to me" and "Include completed" filters.
- [ ] "Track time" on client rows and "Track to [Client] (no task)" in a filtered list.
- [ ] Pinned clients and recent tasks before typing; keyboard navigation.

**Done when:** you can find any real task or client in under a second and start a timer on it.

## Phase 3: Logging time

- [ ] "Log to Wrike" creates a timelog on the task, rounded up to the next 5 minutes (minimum 5).
- [ ] Client bucket time logs to the catch-all task, creating it after confirmation if missing.
- [ ] Offline queue with automatic retry and a pending count.
- [ ] Today and this week totals pulled from Wrike.
- [ ] Clear errors for expired tokens and rate limits.

**Done when:** a logged entry appears correctly in Wrike, and an entry logged offline arrives once you reconnect.

## Phase 4: Polish

- [ ] Idle detection for the running timer.
- [ ] Reminders when no timer runs during work hours (default Mon–Fri 8:00 AM–6:00 PM).
- [ ] Global shortcuts for start/pause and open search.
- [ ] Windows floating mini timer showing the running timer.
- [ ] Settings screen, including 12/24-hour time format (default 12-hour).
- [ ] Customize colors screen: theme, accent swatches (default #955F3A), custom hex with contrast warning, menu bar display options.
- [ ] Launch at login.

**Done when:** everything in `design/` works in the real app.

## Phase 5: Packaging and release

- [ ] .dmg for macOS (Apple Silicon and Intel) and .msi or .exe for Windows.
- [ ] Decide whether code signing is worth it for internal use (unsigned apps show a one-time security warning).
- [ ] Auto-updates via a private GitHub repo's Releases or a shared drive.
- [ ] Short install guide for teammates.

**Done when:** a teammate can install Tally from the internal link and start tracking.

## Testing checklist

Run on both a Mac and a Windows PC; use a test task in Wrike so real timesheets stay clean.

| Scenario | Expected result | Mac | Windows |
| --- | --- | --- | --- |
| Start a second timer while one runs | The first timer pauses automatically and a brief notice says so | | |
| Quit and reopen with timers running | All timers return with correct elapsed time | | |
| Computer sleeps for 30 minutes | Idle prompt appears on wake; choice applies to the running timer | | |
| Log 0:42:13 | Wrike shows 0:45 on the task | | |
| Log 0:02:00 | Wrike shows 0:05 | | |
| Log while offline, then reconnect | Pending count shows 1, then entry appears in Wrike | | |
| Track to a client with no catch-all task | Asks to create it, then logs there | | |
| Search "acme web" | Finds Acme Corp › Website Redesign tasks | | |
| Add a new client folder in Wrike | Appears after refresh | | |
| Switch to 24-hour format | Work hours show 08:00–18:00 | | |
| Switch to Dark and pick a custom accent | Every screen updates without restart; low-contrast accent shows a warning | | |
| Expired or revoked Wrike token | Clear message and a reconnect button | | |

## Risks and open questions

| Risk | Fallback |
| --- | --- |
| Wrike timelogs may only attach to tasks, not folders | Catch-all "General time" task per client |
| Wrike API rate limits during search | Search the local cache first; debounce typing; refresh in the background |
| Windows tray can't show live text | Tooltip plus floating mini timer |
| Unsigned apps trigger security warnings | Acceptable for internal use; include "right-click › Open" steps in the install guide |
| Many open timers clutter the popover | Collapse paused timers into a "Paused" section if the list grows |

- [ ] Will teammates use it, or only you? Decides whether OAuth is needed or a personal token is enough.
- [ ] Does your team already have a catch-all task convention (e.g., "Admin") to reuse?
- [ ] Should catch-all tasks be assigned to anyone, or left unassigned?
