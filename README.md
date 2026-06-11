# Todly

A perpetual desktop todo app built with **Tauri v2** (Rust backend) and vanilla HTML/CSS/JS — designed to stay out of your way while keeping you on track.

## Why another todo app?

Most todo apps either disappear into a tab you forget about or demand your full attention in a bloated interface. Todly was built to live **on your desktop**, always one keystroke away, without ever interrupting your flow.

The core idea: a **persistent overlay** that peeks into your tasks without taking over your screen, a **global hotkey** for instant capture, and streaks to nudge you toward consistency — not urgency.

## Features

- **Floating overlay** — Always-on-top pill that shows your current task count (collapsed) and expands into a full panel (peek → expand). Appears automatically when the main window is hidden.
- **Global quick-add** — Press `Ctrl+Alt+T` (configurable in Settings) from anywhere to open a minimal popup and instantly log a task.
- **Quarterly & Yearly views** — Beyond daily/weekly, plan across quarters and years with dedicated tabs.
- **Streak tracking** — Daily/weekly/quarterly/yearly streaks to encourage consistency. Displayed in the overlay pill.
- **Charts** — Completion trends and streak charts per period.
- **Confetti** — A small celebration on every task completion.
- **Interactive hotkey recorder** — Set your own quick-add shortcut in Settings.
- **System tray** — Minimize to tray, quick-add and show/hide from tray menu.
- **Fully keyboard-friendly** — Quick-add supports `Ctrl+D/W/Q/Y` to switch periods, `Enter` to save, `Esc` to close.

## Folder structure

```
todly/
├── src/                          # Frontend (vanilla HTML/CSS/JS)
│   ├── index.html                # Main window entry point
│   ├── overlay.html              # Overlay window
│   ├── quick-add.html            # Quick-add popup
│   ├── css/
│   │   ├── tokens.css            # Design tokens (colors, spacing, fonts)
│   │   ├── main.css              # Main window styles
│   │   ├── overlay.css           # Overlay styles
│   │   └── quick-add.css         # Quick-add popup styles
│   ├── js/
│   │   ├── main.js               # App logic, routing, CRUD, settings
│   │   ├── overlay.js            # Overlay lifecycle and display
│   │   ├── quick-add.js          # Quick-add popup logic
│   │   ├── store.js              # Tauri invoke wrappers + event bus
│   │   └── canvas-confetti.min.js
│   └── assets/
│       └── fonts/                # Inter font (subset)
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs                # Tauri app setup, plugin wiring, overlay lifecycle
│   │   ├── commands.rs           # IPC command handlers (CRUD, settings)
│   │   ├── db.rs                 # SQLite schema, queries, streak math
│   │   └── tray.rs               # System tray menu
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Tauri window & build configuration
│   └── capabilities/
│       └── default.json          # Tauri capability permissions
├── dev_server.py                 # Dev HTTP server (serves frontend)
└── package.json
```

## How to run

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021, tested with 1.96+)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- Windows, macOS, or Linux (tested on Windows)

### Setup

```bash
# Install Tauri CLI
npm install -g @tauri-apps/cli

# Run in development mode
cd todly
cargo tauri dev
```

This starts the dev server (`python dev_server.py` on port 1422) and launches the Tauri app with hot-reload for the frontend.

### Build for production

```bash
cargo tauri build
```

The bundled installer will be in `src-tauri/target/release/bundle/`.

## Tech stack

| Layer  | Technology |
|--------|-----------|
| Backend | Rust, Tauri v2 |
| Frontend | Vanilla HTML/CSS/JS (no framework) |
| Database | SQLite via `rusqlite` |
| Shortcuts | `tauri-plugin-global-shortcut` |
| Font | [Inter](https://rsms.me/inter/) |
| Confetti | `canvas-confetti` (local copy) |
