# Todly — End-to-End Build Plan

**Product:** Todly — calm, always-visible desktop to-do with a top-centre island overlay  
**Branch:** `cursor/todly-rebuild-2cf5`  
**Primary targets:** Windows first, then macOS; Linux later (must not break, not prioritized)  
**Sources of truth (in conflict order):** this brief → product guide tabs → this plan (updated as we build)

Guide tabs (Google Doc “Overlay To-Do App…”):

1. Competitor research  
2. Tech stack  
3. Features (MVP, Free/Pro, overlay behaviour)  
4. **UI Prompts = screen specs** (not image generation)

---

## 0. Status tracker

| Phase | Name | Status |
|------:|------|--------|
| 0 | Plan committed | **done** |
| 1 | Scaffold (Tauri v2 + Svelte 5/TS/Vite) | pending |
| 2 | Data model, migrations, local-day + DB tests | pending |
| 3 | Design system + app shell + Today | pending |
| 4 | Overlay mechanics (Windows-first) + overlay states | pending |
| 5 | Quick-add, tray, hotkey, notifications | pending |
| 6 | Remaining screens (Week…Onboarding) + mock Pro | pending |
| 7 | Verify builds/tests; PR with HW-test matrix | pending |

Update this table as phases complete. Record deviations under [Changelog](#changelog).

---

## 1. Goals and non-goals

### Goals (this rebuild)

- Clean rebuild of Todly on Tauri v2 + Svelte 5 + TypeScript + Vite + rusqlite.
- Always-visible overlay: transparent, undecorated, always-on-top, **top centre** of primary screen; click-through by default; Rust hit-testing arms after ~2 s dwell; expand → compact list with check-off + confetti; leave → collapse + restore click-through.
- Main window: **Today working**; nav stubs/screens for Week, Month, Long-term, Progress, Settings per UI Prompts.
- Light/dark design tokens from UI Prompts; **no paper texture**.
- Global quick-add: `Ctrl+Alt+T` (Win) / `Cmd+Opt+T` (mac).
- Tray: Show, Toggle overlay, Quit.
- SQLite tasks schema with migration path; day boundaries in **local time**.
- Reminders via `tauri-plugin-notification` (+ island reminder state).
- Mock free/Pro flag (no auth, no payments).
- Unit tests for local-day-boundary logic and DB layer.
- `npm run build` + frontend build + `cargo check`/`clippy` on Linux VM; PR describing what works / stubbed / HW-test needs.

### Non-goals (now)

- Real payments, accounts, sync, App Store.
- Signing/notarisation (document only).
- Full Linux polish; Wayland overlay quirks.
- Recurrence beyond horizon resets, NL quick-add, calendar ICS, AI breakdown.
- Shipping Electron (keep as documented fallback if macOS NSPanel spike fails later).

---

## 2. Bugs in the old code that must not return

Verified in the previous vanilla/Tauri tree before rebuild:

| Bug | Evidence | Fix requirement |
|-----|----------|-----------------|
| Prod build broken | `tauri.conf.json` `frontendDist: "../src"`; entry is `main.html`, not `index.html` | Vite `dist/` as `frontendDist`; multi-page entries emit proper `index.html` / named HTML per window |
| `deadline` vs `due_time` | `main.js` reads `task.deadline`; DB/API use `due_time` | Single field `due_at` end-to-end |
| UTC day boundaries | `chrono::Utc::now()` for history/streak “today” | Local-midnight helpers; tests with fixed offsets |
| CHECK disallows `monthly` | `period IN ('daily','weekly','quarterly','yearly')` | Horizons: `daily\|weekly\|monthly\|goal` + migration |
| Hover delay doesn’t gate clicks | Overlay click expands from `collapsed` without armed state | Clicks ignored until armed after dwell |
| `escHtml` misses quotes | `div.textContent` → `innerHTML` does not escape `"` for attributes | Prefer text nodes / Svelte escaping; never splice titles into attribute strings |
| Overlay corner ignored | Settings store `overlay_corner`; init always top-right | Default **top centre**; position setting applied when pinning |
| Tray quick-add / keep-on-top broken | `quick_add` only emits to main; `keep_on_top` has no handler | Tray: Show / Toggle overlay / Quit (per brief); open real quick-add window from hotkey |

Also: **no 50 ms busy poll** for click-through (`overlay.js` `setInterval(..., 50)`). Replace with event-driven / low-frequency approach (see §5).

---

## 3. Stack

| Layer | Choice |
|-------|--------|
| Shell | Tauri v2 (`tray-icon`, multi-webview) |
| Frontend | Svelte 5 + TypeScript + Vite (MPA: main, overlay, quick-add) |
| Animation | **Motion** (`motion` / svelte motion bindings) for island morph, list stagger, checkbox; CSS springs 220–320 ms |
| Delight | **canvas-confetti** inside overlay canvas (no fullscreen celebration window) |
| Optional later | Rive/Lottie only for authored check/sparkle if Motion alone is insufficient (doc notes) |
| DB | `rusqlite` (bundled) + versioned migrations |
| Time | `chrono` (+ local timezone / `Local`); day-starts-at setting |
| Plugins | `global-shortcut`, `notification`, (later) `autostart`, `single-instance` |
| Charts | Hand SVG heatmap + light bar chart (or uPlot/Chart.js if needed) |

**Windows first:** implement overlay pinning, click-through, and hit-test on Win32/WebView2 paths. macOS: abstract `OverlayPlatform` with clean `TODO(macos): NSPanel + FullScreenAuxiliary / CanJoinAllSpaces`. Linux: compile/check only.

---

## 4. Architecture

```
todly/
├── docs/BUILD_PLAN.md          # this file
├── package.json                # vite + svelte + motion + canvas-confetti + @tauri-apps/*
├── vite.config.ts              # MPA: main / overlay / quick-add
├── src/                        # Svelte frontend
│   ├── styles/tokens.css       # light/dark tokens (UI Prompts §2)
│   ├── lib/                    # api, time, entitlement, stores
│   ├── components/             # shared UI (sidebar, task row, chips…)
│   ├── routes/ or views/       # Today, Week, Month, Goals, Progress, Settings, Onboarding
│   ├── windows/
│   │   ├── main/               # App shell
│   │   ├── overlay/            # Island states S0–S8
│   │   └── quick-add/          # Hotkey popup
│   └── main.ts / overlay.ts / quick-add.ts
└── src-tauri/
    ├── tauri.conf.json         # frontendDist → ../dist; windows config
    ├── capabilities/
    └── src/
        ├── lib.rs              # setup, windows, plugins
        ├── db/                 # schema, migrations, queries, tests
        ├── time.rs             # local day boundaries + tests
        ├── overlay/            # pin, click-through, dwell arming, platform traits
        ├── reminders.rs        # sleep-until-next scheduler (no 1 Hz poll)
        ├── tray.rs
        ├── entitlements.rs     # mock free/pro
        └── commands.rs
```

**Windows (labels):**

| Label | Role |
|-------|------|
| `main` | Decorated (or custom chrome) app shell |
| `overlay` | Pill-sized transparent always-on-top; resize between states |
| `quick-add` | Pre-created small hidden window; show/hide on hotkey |

Do **not** maximise the overlay (old bug: fullscreen transparent window blocks clicks). Keep the physical window ≈ visible pill/panel.

---

## 5. Overlay mechanics (contract)

Aligned with Features tab §5 and UI Prompts 1–6. Default pin: **top centre**, ~8 px below top edge (notch Mac: below menu bar — macOS TODO).

### States

| ID | Name | Trigger | Size / look | Content | Motion |
|----|------|---------|-------------|---------|--------|
| S0 | Sliver | 30 s idle, nothing due soon | ~72×5 px, 35% opacity | — | Fade/shrink ~200 ms |
| S1 | Glimpse | Default / pointer leaves | ~320×38 px pill | Ring (done/total), next title, time chip | Spring ~250 ms |
| S1b | Count only | Setting | ~96×34 px | Ring + “N left” | Same as S1 |
| S2 | Peek (dwell) | Pointer over pill | Scale 1.03, dwell ring fills | “Hold to open” | Ring 0→100% over dwell (default **2.0 s**); cancel if leave >250 ms |
| S3 | Armed | Dwell complete (or arm hotkey) | Soft glow; **click-through OFF** | “Click to open” | Pulse |
| S4 | Expanded | Click while armed | ~380×120–440 px | Today/Week/Month segment; rows + checkbox; add; open app | Morph ~320 ms; row stagger 30 ms |
| S5 | Completion | Checkbox | Inside S4 or pill | Check, strike, slide out; ring advances | Check 180 ms, slide 240 ms; last task → confetti + S8 |
| S6 | Reminder | Due − lead (default 10 m) | ~420×52 amber pill | Title, “10 min left”, Done / Snooze | Bounce; auto-collapse ~20 s to amber dot |
| S7 | Overdue | Past due | Coral marker | “Overdue Nm” chip | Static, no shake |
| S8 | All clear | Last task done | Check ring | “All clear” | Hold ~4 s → S0 |

### Click-through + hit-testing (no 50 ms loop)

**Problem:** Tauri `set_ignore_cursor_events` is boolean only (no forward). A large transparent window steals clicks.

**Approach (justify in code comments):**

1. Keep overlay window sized to the visible pill/panel only.  
2. Default: `ignore_cursor_events = true` (click-through).  
3. **Rust-side proximity watch** (not JS 50 ms forever):
   - While collapsed/click-through: poll cursor at **low frequency** (~4–10 Hz) **only** when a cheap OS hook isn’t available; or wake on a short timer when the last sample was near the pill AABB expanded by ~24 px, else sleep longer (e.g. 250–500 ms). Prefer Windows `GetCursorPos` in a dedicated lightweight thread/timer vs invoking from the webview every frame.
   - When cursor enters the pill AABB: start dwell timer (2 s configurable).  
   - On dwell complete → **arm**: set `ignore_cursor_events = false`, emit `overlay://armed` to webview.  
   - On leave >250 ms while not expanded → disarm, restore click-through, cancel dwell.  
4. Expanded: window enlarged to panel; click-through off; collapse on Esc / outside / idle 8 s (don’t collapse while input focused / IME composing).  
5. Frontend never arms itself without Rust confirmation (fixes “hover delay doesn’t gate clicks”).

### Fullscreen / Spaces

- Setting: Hide (default) | Show above fullscreen | Reminders only.  
- **Windows:** implement hide/show using monitor/fullscreen heuristics as available; document HW test.  
- **macOS:** `TODO(macos): NSPanel + collectionBehavior FullScreenAuxiliary | CanJoinAllSpaces` (tauri-nspanel or raw objc). If not feasible in-tree, leave stub + comment linking Tauri #11488.  
- **Linux:** no-op / best-effort always-on-top.

### Performance

- No `requestAnimationFrame` / Motion loops while collapsed idle.  
- Reminder scheduler: sleep until next fire time (not 1 Hz).  
- Confetti: short burst inside overlay; dispose canvas.

---

## 6. Data model and migrations

### Schema v1 (target)

```sql
CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  horizon TEXT NOT NULL CHECK (horizon IN ('daily','weekly','monthly','goal')),
  due_at TEXT,                          -- local wall time ISO-like; NULL ok for undated goals
  remind_minutes_before INTEGER NOT NULL DEFAULT 10,
  completed_at TEXT,                    -- NULL = incomplete
  created_at TEXT NOT NULL,
  notes TEXT,
  progress INTEGER,                     -- 0–100 for long-term goals; NULL otherwise
  second_remind_minutes_before INTEGER  -- Pro; NULL = off
);

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- history / daily aggregates as needed for Progress (local dates YYYY-MM-DD)
```

### Migration path

| Version | Change |
|--------:|--------|
| 0→1 | Fresh schema above |
| Legacy detect | If old `tasks` with `period` / `due_time` / CHECK without monthly: migrate → rename columns, map `quarterly`/`yearly` → `goal`, map `due_time` → `due_at`, set `remind_minutes_before=10`, `completed_at` from old flags |

Default settings seeds: theme, dwell_ms=2000, overlay_position=`top-center`, remind lead=10, entitlement=`free`, day_starts_at=`00:00`, week_starts_on=`monday`, overlay_style=`next-task|count-only`, etc.

### Local day boundaries

- `local_day_id(now, day_starts_at) -> NaiveDate`  
- “Today’s tasks”: incomplete daily (and due-dated items falling in local day window).  
- Tests: UTC midnight vs IST (+05:30), day-starts-at 04:00, DST transition fixture if feasible.

---

## 7. Screen inventory (UI Prompts = specs)

Shared chrome (Prompts 7–11, 14): sidebar 232 px, Todly mark, nav Today / Week / Month / Long-term / Progress, Settings + Free plan card + Upgrade link. Content bg `#F7F8FA`. Tokens: see §8.

| # | Screen / state | Build priority | Acceptance (summary) |
|---|----------------|----------------|----------------------|
| 1 | Overlay Glimpse | P0 | Pill top-centre; ring; next title; time chip; click-through |
| 2 | Hover dwell / Armed | P0 | Dwell ring; arm only after dwell; click gated |
| 3 | Expanded panel | P0 | Segment Today/Week/Month; checkable rows; collapse restores CT |
| 4 | Completion moment | P0 | Motion check + strike + confetti on last |
| 5 | Reminder 10 min | P0 | Amber pill; Done / Snooze; OS toast backup |
| 6 | Idle / Count / Overdue / All clear | P0 | All four variants selectable by state machine |
| 7 | Today | P0 | Working CRUD list; matches layout intent of Prompt 7 |
| 8 | Week | P1 | Spec UI; may be read-only grouping stub then fill |
| 9 | Month | P1 | Calendar + side panel per Prompt 9 |
| 10 | Long-term goals | P1 | Goal cards + progress; Pro-gated create |
| 11 | Progress | P1 | 7-day stats; Pro stubs for heatmap/30d |
| 12 | Quick-add | P0 | Hotkey popup; horizon + due chips; Enter/Esc |
| 13 | Task details | P1 | Edit dialog; second reminder Pro tag |
| 14 | Settings | P1 | Overlay + Reminders tabs per Prompt 14 |
| 15 | Upgrade (mock) | P1 | Demo unlock writes `entitlement=pro` |
| 16 | Onboarding | P2 | 3-step welcome; Skip; sets flag |

**Horizon colors:** Daily `#4F6BED`, Weekly `#7A5AF8`, Monthly `#14A3B8`, Long-term `#D6568C` (dark variants in tokens).

---

## 8. Design tokens (from UI Prompts §2)

**Light:** bg `#F7F8FA`, surface `#FFFFFF`, subtle `#EEF1F5`, border `#E3E7EE`, text `#101828` / `#5B6574` / `#8A94A3`, accent `#4F6BED`, success `#2FAE7B`, warning `#F5A524`, danger `#E5584F`, island surface white @94%.  

**Dark:** bg `#0E1116`, surface `#171B22`, subtle `#1F2430`, border `#2A3140`, text `#F2F4F8` / `#A3ADBD`, accent `#7C8FFF`, etc.

**Type:** Inter (fallback Segoe UI Variable / SF Pro). Display 28/34 semibold; body 14–15; island 13/18; chips 11/16 tabular nums.

**Shape:** 4 px grid; radius 8 chips, 12 controls, 16 cards, 20 dialogs/expanded island, full pill; soft shadow `0 8px 24px rgba(16,24,40,0.10)` — **no paper texture, no heavy glow**.

**Motion:** springs 220–320 ms; stagger 30 ms; check 180 ms; row exit 240 ms; `prefers-reduced-motion` → ≤120 ms fades.

---

## 9. Animation approach

1. **Motion** for: pill scale/morph to panel, dwell ring progress, row enter/exit, checkbox fill.  
2. **CSS variables** for durations; disable when reduced-motion.  
3. **canvas-confetti** on last-of-day / complete-from-overlay celebration (contained).  
4. **Rive/Lottie:** optional later for authored checkmark; not required for MVP if Motion covers it.  
5. Idle: stop animations when in S0/S1 with no interaction.

---

## 10. Free vs Pro (mock)

| Feature | Free | Pro (demo unlock) |
|---------|------|-------------------|
| Horizons | Daily, Weekly | + Monthly, Long-term |
| Active tasks | Cap 20 | Unlimited |
| Overlay + reminders core | Yes | Yes (never paywalled) |
| Second reminder / per-task lead | No | Yes |
| Progress depth | 7-day | 30/90, heatmap, export |
| Display choice | Primary | Follow / pick |

`can(feature)` in Rust; Upgrade dialog sets `entitlement=pro`. No network.

---

## 11. Phases, milestones, acceptance

### Phase 1 — Scaffold

- Replace vanilla frontend with Vite + Svelte 5 + TS MPA.  
- Fix `frontendDist` → `../dist`; entries produce correct HTML.  
- Keep icons; rewrite `Cargo.toml` deps (notification plugin, etc.).  
- **Accept:** `npm run build` produces `dist/`; `cargo check` passes on Linux.

### Phase 2 — DB + time

- Migrations + CRUD commands using `due_at` / `horizon`.  
- Seed sample tasks.  
- **Accept:** `cargo test` covers DB CRUD, monthly horizon allowed, local-day boundary cases.

### Phase 3 — Shell + Today + tokens

- Sidebar chrome; Today list create/toggle/delete; light/dark.  
- **Accept:** Today matches Prompt 7 structure (logo, nav, list, add); theme toggle works.

### Phase 4 — Overlay (Windows-first)

- Create pill window; pin top-centre; state machine S0–S8 UI; Rust dwell arming; expand/collapse; confetti.  
- macOS/Linux stubs with TODOs.  
- **Accept:** Unit/integration tests for hit-rect + arm state machine where possible; manual HW checklist for Win.

### Phase 5 — Hotkey, tray, notifications

- Precreate quick-add; register platform hotkey; tray Show / Toggle overlay / Quit.  
- Reminder scheduler + notification plugin + S6 UI.  
- **Accept:** Commands invoked from unit tests/mocks; HW test for hotkey/tray/toast.

### Phase 6 — Remaining screens + Pro

- Week, Month, Long-term, Progress, Settings, Task details, Upgrade, Onboarding (P2).  
- Wire Pro gates.  
- **Accept:** Each route renders Prompt-aligned UI; Pro unlock flips gates.

### Phase 7 — Harden + PR

- Clippy, frontend build, document stubs.  
- PR lists: works / stubbed / Windows+macOS HW tests.  
- Update this plan status table.

---

## 12. Real-hardware test matrix (PR must list)

| Area | Windows | macOS | Linux |
|------|---------|-------|-------|
| Overlay click-through under browser tabs | **Required** | Required | Nice |
| Dwell arm + gated click | **Required** | Required | Nice |
| Expand/collapse resize flicker | **Required** | Required | — |
| Top-centre pin / multi-monitor | **Required** | Required | — |
| Above fullscreen / Spaces | Best-effort | **NSPanel TODO / verify** | — |
| Hotkey quick-add | **Required** | Cmd+Opt+T | — |
| Tray Show / Toggle / Quit | **Required** | Required | — |
| Notifications + reminder island | **Required** (installed build) | Required | — |
| Idle CPU ~0 with collapsed overlay | Measure | Measure | — |

---

## 13. Implementation order (execute next)

1. Scaffold Vite/Svelte/Tauri config (Phase 1).  
2. `time.rs` + `db` migrations/tests (Phase 2).  
3. Tokens + main shell + Today (Phase 3).  
4. Overlay Rust + Svelte states (Phase 4).  
5. Quick-add, tray, reminders (Phase 5).  
6. Other screens + mock Pro (Phase 6).  
7. Verify + PR (Phase 7).

---

## Changelog

| Date | Change |
|------|--------|
| 2026-11-12 | Initial plan committed on `cursor/todly-rebuild-2cf5` from brief + guide tabs 2–4; old bugs catalogued. |
