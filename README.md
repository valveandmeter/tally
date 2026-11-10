# Tally — Claude Code starter (internal sister app to CopperOS)

Everything Claude Code needs to build the app: the spec, the build plan, and the design reference.

## What's inside
- `CLAUDE.md` — the full spec and working rules. Claude Code reads this automatically.
- `docs/BUILD_PLAN.md` — phases with checkboxes, testing checklist, risks.
- `design/` — screen mockups, design notes, and `tokens.css` color variables.
- `.env.example` — where your Wrike OAuth credentials go during development.

## How to start
1. Unzip this folder somewhere permanent (e.g., `~/Projects/tally`).
2. Open `CLAUDE.md` and fill in the last line ("I'm [describe your experience level]...").
3. Work through "Before you start" in `docs/BUILD_PLAN.md` (Node.js, Rust, Wrike API app).
4. Open the folder in Claude Code (desktop app: choose the folder; terminal: `cd` into it and run `claude`).
5. Say: **"Read CLAUDE.md, docs/BUILD_PLAN.md, and design/README.md, then start Phase 0."**
6. After each phase, test on both Mac and Windows before saying "Start Phase N."

Tip: run `git init` first (or let Claude Code do it) so every step is saved and reversible.

## Recommended IDE setup

This is a Tauri 2 + React + TypeScript project. If you edit in VS Code, install:
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
