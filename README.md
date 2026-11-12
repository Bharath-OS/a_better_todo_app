# Todly

Calm desktop to-do with an always-visible top-centre island overlay.

**Local-first / offline.** SQLite on device. No account. No real payments in this rebuild (Upgrade UI is demo-only).

## Stack

- **Tauri v2** (Rust) + **Svelte 5** + TypeScript + Vite  
- **rusqlite** · Motion · canvas-confetti · `tauri-plugin-notification` · global shortcut · tray  

## Branching

| Branch | Role |
|--------|------|
| `main` | Stable. Do not merge features here until Bharath verifies a build. |
| `cursor/development-2cf5` | Integration branch. Holds `docs/BUILD_PLAN.md` and passing code. |
| `cursor/feat-<slice>-2cf5` | One feature at a time. PR **into development only**. |

See [`docs/BUILD_PLAN.md`](docs/BUILD_PLAN.md) for slices, TDD rules, overlay contract, and HW test matrix.

## Develop

```bash
npm install
npm run build          # frontend → dist/
cd src-tauri && cargo test && cargo check
# Full app (Windows/macOS host):
npm run tauri dev
```

Hotkey: `Ctrl+Alt+T` (Windows) / `Cmd+Opt+T` (macOS). Tray: Show · Toggle overlay · Quit.

## Licence / payments

Mock Free/Pro only. Pay button shows a snackbar (“You paid $12 (demo)”) — no network, no processor.
