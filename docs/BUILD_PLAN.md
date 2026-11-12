# Todly — End-to-End Build Plan

**Product:** Todly — calm, always-visible desktop to-do with a top-centre island overlay  
**Long-lived integration branch:** `cursor/development-2cf5` (do **not** merge to `main` until Bharath verifies)  
**Feature branches:** `cursor/feat-<slice>-2cf5` → PR **into development only**  
**Primary targets:** Windows first, then macOS; Linux later (must not break, not prioritized)  
**Sources of truth (conflict order):** Bharath brief → product guide tabs → this plan

Guide tabs (Google Doc “Overlay To-Do App…”):

1. Competitor research  
2. Tech stack  
3. Features (MVP, Free/Pro, overlay behaviour)  
4. **UI Prompts = screen specs** (not image generation)

---

## 0. Git workflow (approved)

```
main                          # stable; README lives here; NO feature merges until Bharath says so
  └── cursor/development-2cf5 # BUILD_PLAN + integrated code; all tests must stay green
        ├── cursor/feat-scaffold-2cf5
        ├── cursor/feat-db-time-2cf5
        ├── cursor/feat-shell-tokens-2cf5
        ├── cursor/feat-today-2cf5
        ├── cursor/feat-overlay-2cf5
        ├── cursor/feat-quickadd-tray-2cf5
        ├── cursor/feat-reminders-2cf5
        ├── cursor/feat-views-settings-2cf5
        └── cursor/feat-upgrade-ui-2cf5
```

Rules:

1. Create each feature branch from the latest `cursor/development-2cf5`.
2. **TDD:** write failing tests first (happy path + edge/worst cases), then implement until green.
3. Merge a feature into development **only** when that slice’s tests pass **and** the full development suite stays green.
4. Open PRs **only** into `cursor/development-2cf5`. Never open/merge a PR into `main` until Bharath personally verifies the app.
5. Parallel agents only on **independent** slices (no shared unfinished contracts). After scaffold + DB land, overlay vs settings UI may parallelize; Today waits on DB.
6. App is **fully offline / local-first** (per product guide). No payment APIs. Upgrade UI shows a snackbar like “You paid $12 (demo)” on Pay — no network.
7. When confused: **ask Bharath** — do not guess product behaviour.

---

## 1. Feature slices (order + status)

| # | Slice | Branch | Depends on | Status |
|--:|-------|--------|------------|--------|
| 1 | Scaffold (Vite/Svelte/Tauri, `dist`/`index.html`, RGBA icons) | `cursor/feat-scaffold-2cf5` | — | **merged into development** |
| 2 | DB + local-day time + migrations + tests | `cursor/feat-db-time-2cf5` | 1 | **in progress (edge-case TDD)** |
| 3 | Design tokens + shell chrome (sidebar/nav stubs) | `cursor/feat-shell-tokens-2cf5` | 1 | **in progress on development (foundation WIP)** |
| 4 | Today view CRUD | `cursor/feat-today-2cf5` | 2, 3 | foundation present; harden via feature PR |
| 5 | Overlay mechanics (Windows-first) | `cursor/feat-overlay-2cf5` | 1, 2 | foundation present; harden via feature PR |
| 6 | Quick-add + tray + hotkey | `cursor/feat-quickadd-tray-2cf5` | 1, 2 | foundation present; harden via feature PR |
| 7 | Reminders + notifications | `cursor/feat-reminders-2cf5` | 2, 5 | foundation present; harden via feature PR |
| 8 | Week/Month/Goals/Progress/Settings stubs | `cursor/feat-views-settings-2cf5` | 3, 4 | foundation present; harden via feature PR |
| 9 | Upgrade/paywall UI (snackbar only) | `cursor/feat-upgrade-ui-2cf5` | 3 | foundation present; harden via feature PR |

**Foundation WIP note:** An initial clean rebuild was landed on `cursor/development-2cf5` so work is not lost. Subsequent work uses feature branches + TDD to harden each slice before calling it “merged complete”.

Update this table as slices merge.

---

## 2. TDD expectations per slice

- **Unit tests (Rust):** DB CRUD, migrations (incl. legacy `period`→`horizon`), local-day boundaries (midnight, day-starts-at 04:00, invalid parse), entitlements gates, hit-rect / arm state machine, reminder scheduling math.
- **Worst / edge cases:** empty DB, corrupt/legacy schema, Pro-gated horizons while Free, active-task cap (20), invalid horizon strings, missing `due_at`, rapid toggle, dwell cancel on leave, expand without arm (must not click), filesystem/DB open failure paths.
- **No fake “internet” tests** unless a feature introduces network (none planned — offline app). Notification plugin failures should degrade to overlay-only reminder UI.
- **Frontend:** `npm run build` must succeed; prefer pure TS helpers tested where logic is non-trivial.
- **CI gate before merge to development:** `npm run build`, `cargo test`, `cargo clippy` (deny warnings where practical) on the Linux VM; Windows/macOS overlay HW checks remain manual.

---

## 3. Goals and non-goals

### Goals

- Clean rebuild: Tauri v2 + Svelte 5 + TypeScript + Vite + rusqlite.
- Overlay: transparent, undecorated, always-on-top, **top centre**; click-through by default; Rust hit-testing arms after ~2 s dwell; expand → list + confetti; leave → collapse + click-through. **No 50 ms busy poll.**
- Main: Today working; Week/Month/Long-term/Progress/Settings per UI Prompts (stubs OK until slice 8 done).
- Light/dark tokens; **no paper texture**.
- Hotkey `Ctrl+Alt+T` (Win) / `Cmd+Opt+T` (mac); tray Show / Toggle overlay / Quit.
- SQLite + migrations; **local** day boundaries; reminders via `tauri-plugin-notification`.
- Mock Free/Pro; Upgrade Pay → snackbar only.

### Non-goals

- Real payments, accounts, sync, App Store.
- Merging to `main` without Bharath’s OK.
- Full Linux polish; Wayland overlay.
- Electron unless macOS NSPanel proves infeasible later.

---

## 4. Bugs that must not return

| Bug | Fix |
|-----|-----|
| `frontendDist: ../src` + `main.html` | Vite `dist/` + `index.html` / MPA entries |
| `deadline` vs `due_time` | Single field `due_at` |
| UTC day boundaries | Local-day helpers + tests |
| CHECK without `monthly` | `daily\|weekly\|monthly\|goal` + migration |
| Hover delay doesn’t gate clicks | Click-through until Rust arms after dwell |
| `escHtml` quote holes | Svelte text binding; no attribute string splicing |
| Overlay corner ignored | Default **top-centre**; honour `overlay_position` |
| Tray quick-add / keep-on-top broken | Tray: Show / Toggle overlay / Quit; hotkey opens real quick-add |
| 50 ms click-through poll | Adaptive low-frequency Rust hit-test (~100–250 ms) |

---

## 5. Stack

| Layer | Choice |
|-------|--------|
| Shell | Tauri v2 (`tray-icon`, multi-webview) |
| Frontend | Svelte 5 + TypeScript + Vite (MPA: main, overlay, quick-add) |
| Animation | Motion for morph/stagger; canvas-confetti in overlay |
| DB | rusqlite + versioned migrations |
| Time | chrono Local; `day_starts_at` |
| Plugins | global-shortcut, notification |

Windows first. macOS: `TODO(macos): NSPanel + FullScreenAuxiliary | CanJoinAllSpaces`. Linux: compile/check only.

---

## 6. Overlay mechanics (contract)

States S0–S8 per Features tab / UI Prompts 1–6: Sliver, Glimpse, Count-only, Peek (dwell), Armed, Expanded, Completion, Reminder, Overdue, All clear.

**Click-through:** window sized to pill/panel; default ignore cursor; Rust dwell (2 s) then arm; leave >250 ms cancels. Frontend must not expand until armed.

**Fullscreen:** setting Hide | Show above | Reminders only. macOS Spaces = TODO.

---

## 7. Data model

```sql
tasks (
  id, title,
  horizon CHECK IN ('daily','weekly','monthly','goal'),
  due_at,                          -- local wall time
  remind_minutes_before DEFAULT 10,
  completed_at, created_at, notes, progress, second_remind_minutes_before
)
settings (key, value)
schema_migrations (version, applied_at)
```

Legacy migrate: `period`/`due_time`/`quarterly|yearly` → `horizon`/`due_at`/`goal`.

---

## 8. Screen inventory (UI Prompts)

1–6 Overlay states · 7 Today · 8 Week · 9 Month · 10 Long-term · 11 Progress · 12 Quick-add · 13 Task details · 14 Settings · 15 Upgrade (snackbar pay) · 16 Onboarding.

---

## 9. Design tokens

Light/dark tokens from UI Prompts §2 (`#F7F8FA` / `#0E1116`, accent `#4F6BED` / `#7C8FFF`, Inter, 4 px grid, soft shadows, no paper texture). Motion 220–320 ms; reduced-motion ≤120 ms.

---

## 10. Free vs Pro (mock)

Free: Daily/Weekly, 20 active tasks, 7-day progress, overlay+reminders.  
Pro (demo unlock / fake Pay snackbar): Monthly, Long-term, unlimited, second reminder, deep progress.  
Never paywall overlay/reminders reliability.

---

## 11. Hardware test matrix (Windows first)

| Area | Windows | macOS | Linux |
|------|---------|-------|-------|
| Overlay click-through under browser tabs | **Required** | Required | Nice |
| Dwell arm + gated click | **Required** | Required | Nice |
| Expand/collapse resize | **Required** | Required | — |
| Top-centre pin / multi-monitor | **Required** | Required | — |
| Above fullscreen / Spaces | Best-effort | **NSPanel TODO** | — |
| Hotkey / tray / notifications | **Required** | Required | — |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-11-12 | Initial plan on rebuild branch. |
| 2026-11-12 | Workflow rewritten: `cursor/development-2cf5`, feature branches, TDD, no merge to main until Bharath verifies; foundation WIP saved on development. |
