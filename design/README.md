# Design reference

Visual source of truth: https://claude.ai/artifact/55y8qmxZ2NxfeWf3bQKrfQ (the design canvas; may require signing in as its owner).

> **Note:** the `screens/` mockup files described below were lost in an earlier session (an overly aggressive scaffold command cleared this directory before the project files were committed to git). The live canvas at the link above is still the real source of truth — open it yourself to see the actual mockups. The table below documents what each screen covered, reconstructed from project notes, as a map of what to look for on the canvas.

The files in `screens/` (when present) are the canvas mockups' source. They use a canvas-only template format (`{{holes}}`, `<sc-for>`, `<sc-if>`, a `support.js` runtime), so they won't render correctly if opened directly in a browser. Read them as a precise reference for layout, spacing, sizes, copy, and colors (all styles are inline). Rebuild the screens in the app's own framework; don't copy the template syntax.

Colors and tokens: `tokens.css`.

## Screens

| File | What it shows | Notes |
| --- | --- | --- |
| `MultiTimers.dc.html` | **Main popover**: timer list, one running, auto-pause notice, "+ New timer", today/week footer | The current main screen. Starting a timer pauses the running one. |
| `Main.dc.html` | Earlier single-timer popover and menu bar | Superseded by MultiTimers; useful for the menu bar item, current-task card, and note field styling. Category/"Will log" were removed. |
| `Search.dc.html` | Search: client filter chip, Clients and Tasks groups, "Track time" on clients, "Track to [Client] (no task)", filters, keyboard hint | Interactive logic in its script block shows the intended filtering behavior. |
| `Settings.dc.html` | Settings: Wrike connection, Clients space, cache refresh, rounding (read-only, 5 min), idle, reminders, work hours (8:00 AM–6:00 PM), time format (12/24-hour), Appearance link, shortcuts, launch at login | |
| `Appearance.dc.html` | Customize colors: live preview, theme toggle, accent swatches + custom hex, menu bar display | Client colors were intentionally removed. |
| `DarkMode.dc.html` | Dark theme of the timer list and search | Shows dark token usage, including #D9A47E for accent text. |
| `Windows.dc.html` | Windows floating mini timer, tray icon, tooltip | Mini timer shows the running timer only. |

Client and task names in the mockups (Acme Corp, Northwind Outfitters, etc.) are sample data.
