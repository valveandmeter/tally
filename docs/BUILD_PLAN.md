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

- [x] Menu bar item showing elapsed time on macOS; tray icon with tooltip on Windows (Windows untested so far).
- [x] Popover with the timer list: Pause/Resume, discard, and "+ New timer" (placeholder task for now).
- [x] Multiple timers, each with its own time and note; only one runs at a time, and starting or resuming one pauses the running one.
- [x] Brief in-popover notice when a timer is auto-paused ("Paused Homepage hero revisions at 0:42:13").
- [x] Menu bar always shows the running timer's time, or a paused icon when none is running.
- [x] Timers persist across restarts and sleep (SQLite).
- [x] All colors defined as theme variables, with Light and Dark working from the start.

**Done when:** you can switch between three timers with only one ever running, quit the app, reopen it, and find them intact on both platforms. **Confirmed:** tray icon + popover working on Mac. **Still to verify yourself:** multi-timer switching, discard confirmation, persistence across a full quit/relaunch, and Light/Dark following system — worth a pass before Phase 3 (logging) once there's real data at stake. Windows not tested yet.

Known simplifications from the spec, to revisit in Phase 4 polish:
- No separate "paused" tray icon yet — same icon throughout, state shown via the title/tooltip text only.
- Popover dismiss-on-click-away uses window focus-loss, not a native macOS panel — works, but may feel slightly different from a true NSPopover.
- No footer totals/pending count yet (needs real Wrike data — Phase 3).

## Phase 2: Wrike connection and search

- [x] Connect via permanent token (built, not yet verified against a real account — no token available yet). OAuth not started; permanent token confirmed sufficient to start.
- [x] Load the Clients space and its folders as the client list; cache locally (built, unverified).
- [x] Type-ahead search with Clients and Tasks groups, client filter chip, "Assigned to me" and "Include completed" filters (built, unverified).
- [x] "Track time" on client rows and "Track to [Client] (no task)" in a filtered list (built, unverified).
- [x] Pinned clients and recent tasks before typing; keyboard navigation (built, unverified).

**Done when:** you can find any real task or client in under a second and start a timer on it. **Not yet verifiable** — needs a real Wrike permanent access token and confirmation that your account has a "Clients" space structured as CLAUDE.md assumes. Everything compiles and the UI flow works end-to-end against an empty/disconnected state; the actual Wrike API calls (endpoint paths, response field names) are implemented from verified current API v4 docs but untested against live data.

Known simplifications to revisit later:
- `project_path` only resolves one folder level under each client (the immediate subfolder) — deeper nesting falls back to "Client › Task" with no project segment.
- Host for permanent-token connections defaults to `www.wrike.com`; if your account actually lives on a different Wrike data center, the first connect attempt will fail clearly and we'll adjust.
- The "Clients" space name is still hardcoded (no Settings picker yet — that's Phase 4).

## Phase 3: Logging time

- [x] "Log to Wrike" creates a timelog on the task, rounded up to the next 5 minutes (minimum 5) (built, unverified — see note below).
- [x] Client bucket time logs to the catch-all task, creating it after confirmation if missing (built, unverified).
- [x] Offline queue with automatic retry (every 10s while entries are pending) and a pending count in the footer (built, unverified).
- [x] Today and this week totals pulled from Wrike, not summed locally (built, unverified).
- [x] Clear errors for expired tokens (401) and rate limits (429, respects Retry-After) — surfaced as plain-English messages from WrikeError (built, unverified).

**Done when:** a logged entry appears correctly in Wrike, and an entry logged offline arrives once you reconnect. **Not yet verifiable** — same blocker as Phase 2: no live Wrike token yet. Everything compiles and the UI flow works end-to-end (log button appears only on Wrike-linked timers, confirmation banner, catch-all confirm card, offline queue persists across restarts since it's a SQLite table).

Known limitation, inherent to Wrike's API (not a bug to fix later): **no server-side idempotency** for the timelog-creation endpoint. If a submission times out on our end after Wrike actually received it, a retry will double-log. The retry loop only retries on a clear error response, not on ambiguous timeouts, to reduce — not eliminate — this risk. Flagging now since it only shows up under real network flakiness, which is hard to test for deliberately.

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
