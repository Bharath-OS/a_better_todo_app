//! Overlay window lifecycle: pin top-centre, click-through, dwell arming.
//!
//! ## Click-through strategy (replaces the old 50 ms JS poll)
//!
//! Tauri only supports boolean `set_ignore_cursor_events` (no "forward" mode).
//! A maximised transparent window therefore steals clicks across the whole screen.
//!
//! We instead:
//! 1. Keep the overlay window sized to the visible pill/panel only.
//! 2. Default to click-through (`ignore_cursor_events = true`).
//! 3. Run a **low-frequency** Rust timer (~250 ms far away, ~100 ms when near the
//!    pill AABB expanded by 24 px). This is event-ish / adaptive, not a busy 50 ms
//!    webview loop, and does not wake the WebView until arm/disarm transitions.
//! 4. After `dwell_ms` (default 2000) of continuous hover, arm: disable click-through
//!    and emit `overlay-armed`. Clicks before arm are impossible (window still
//!    click-through) — fixing the old bug where hover delay did not gate clicks.
//! 5. Leaving the AABB for >250 ms cancels dwell / disarms when collapsed.
//!
//! ## Platform notes
//! - **Windows (priority):** pin to primary monitor work area top-centre.
//! - **macOS:** same pinning for now; `TODO(macos): NSPanel + FullScreenAuxiliary |
//!   CanJoinAllSpaces` so the island can appear over fullscreen Spaces.
//! - **Linux:** best-effort always-on-top; Wayland limits documented in BUILD_PLAN.

use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

const PILL_W: f64 = 340.0;
const PILL_H: f64 = 48.0;
const PANEL_W: f64 = 400.0;
const PANEL_H: f64 = 440.0;
const TOP_INSET: f64 = 8.0;
const NEAR_PAD: f64 = 24.0;
const LEAVE_DEBOUNCE_MS: u64 = 250;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmState {
    ClickThrough,
    Dwelling,
    Armed,
    Expanded,
}

#[derive(Default)]
struct OverlayInner {
    arm: ArmState,
    dwell_started: Option<Instant>,
    leave_started: Option<Instant>,
    dwell_ms: u64,
    visible: bool,
}

impl Default for ArmState {
    fn default() -> Self {
        ArmState::ClickThrough
    }
}

pub struct OverlayController {
    inner: Mutex<OverlayInner>,
    running: AtomicBool,
}

impl OverlayController {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(OverlayInner {
                arm: ArmState::ClickThrough,
                dwell_started: None,
                leave_started: None,
                dwell_ms: 2000,
                visible: true,
            }),
            running: AtomicBool::new(false),
        })
    }

    pub fn set_dwell_ms(&self, ms: u64) {
        self.inner.lock().dwell_ms = ms.max(500);
    }

    pub fn set_expanded(&self, expanded: bool) {
        let mut g = self.inner.lock();
        if expanded {
            g.arm = ArmState::Expanded;
            g.dwell_started = None;
            g.leave_started = None;
        } else {
            g.arm = ArmState::ClickThrough;
            g.dwell_started = None;
            g.leave_started = None;
        }
    }

    pub fn set_visible(&self, visible: bool) {
        self.inner.lock().visible = visible;
    }

    pub fn is_expanded(&self) -> bool {
        self.inner.lock().arm == ArmState::Expanded
    }
}

#[derive(Clone, Serialize)]
struct ArmPayload {
    armed: bool,
    dwelling: bool,
    progress: f64,
}

pub fn create_overlay_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window("overlay").is_some() {
        return Ok(());
    }
    let win = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(true)
        .focused(false)
        .inner_size(PILL_W, PILL_H)
        .build()
        .map_err(|e| e.to_string())?;

    let _ = win.set_ignore_cursor_events(true);
    pin_top_center(app, PILL_W, PILL_H)?;

    // TODO(macos): convert this window to an NSPanel with collection behaviors
    // NSWindowCollectionBehaviorFullScreenAuxiliary | CanJoinAllSpaces so the
    // island can appear over fullscreen Spaces. See Tauri #11488 / tauri-nspanel.
    #[cfg(target_os = "macos")]
    {
        let _ = win; // placeholder for future NSPanel wiring
    }

    Ok(())
}

pub fn pin_top_center(app: &AppHandle, width: f64, height: f64) -> Result<(), String> {
    let win = app
        .get_webview_window("overlay")
        .ok_or_else(|| "overlay missing".to_string())?;
    let monitor = win
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| win.current_monitor().ok().flatten())
        .ok_or_else(|| "no monitor".to_string())?;
    let scale = monitor.scale_factor();
    let size = monitor.size();
    let pos = monitor.position();
    let logical_w = size.width as f64 / scale;
    let x = pos.x as f64 / scale + (logical_w - width) / 2.0;
    let y = pos.y as f64 / scale + TOP_INSET;
    let _ = win.set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)));
    let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
    Ok(())
}

pub fn resize_overlay(app: &AppHandle, expanded: bool) -> Result<(), String> {
    let (w, h) = if expanded {
        (PANEL_W, PANEL_H)
    } else {
        (PILL_W, PILL_H)
    };
    pin_top_center(app, w, h)?;
    Ok(())
}

pub fn start_hit_test_loop(app: AppHandle, ctrl: Arc<OverlayController>) {
    if ctrl.running.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        loop {
            if !ctrl.running.load(Ordering::SeqCst) {
                break;
            }
            let sleep_for = tick_hit_test(&app, &ctrl).unwrap_or(Duration::from_millis(250));
            std::thread::sleep(sleep_for);
        }
    });
}

fn tick_hit_test(app: &AppHandle, ctrl: &OverlayController) -> Result<Duration, String> {
    let Some(win) = app.get_webview_window("overlay") else {
        return Ok(Duration::from_millis(500));
    };
    let snapshot = {
        let g = ctrl.inner.lock();
        (g.visible, g.arm, g.dwell_ms, g.dwell_started, g.leave_started)
    };
    let (visible, arm, dwell_ms, _, _) = snapshot;
    if !visible {
        return Ok(Duration::from_millis(500));
    }
    if arm == ArmState::Expanded {
        // Expanded: keep interactive; frontend collapses and calls set_expanded(false).
        return Ok(Duration::from_millis(250));
    }

    let cursor = win.cursor_position().map_err(|e| e.to_string())?;
    let outer_pos = win.outer_position().map_err(|e| e.to_string())?;
    let outer_size = win.outer_size().map_err(|e| e.to_string())?;
    let scale = win.scale_factor().unwrap_or(1.0);

    // Convert to logical pixels relative to the overlay window.
    let cx = cursor.x as f64 / scale;
    let cy = cursor.y as f64 / scale;
    let wx = outer_pos.x as f64 / scale;
    let wy = outer_pos.y as f64 / scale;
    let ww = outer_size.width as f64 / scale;
    let wh = outer_size.height as f64 / scale;

    let near = point_near_rect(cx, cy, wx - NEAR_PAD, wy - NEAR_PAD, ww + NEAR_PAD * 2.0, wh + NEAR_PAD * 2.0);
    let over = point_in_rect(cx, cy, wx, wy, ww, wh);

    let mut g = ctrl.inner.lock();
    let now = Instant::now();

    if over {
        g.leave_started = None;
        match g.arm {
            ArmState::ClickThrough => {
                g.arm = ArmState::Dwelling;
                g.dwell_started = Some(now);
                let _ = app.emit(
                    "overlay-arm",
                    ArmPayload {
                        armed: false,
                        dwelling: true,
                        progress: 0.0,
                    },
                );
            }
            ArmState::Dwelling => {
                let started = g.dwell_started.unwrap_or(now);
                let elapsed = now.duration_since(started).as_millis() as u64;
                let progress = (elapsed as f64 / dwell_ms as f64).min(1.0);
                if elapsed >= dwell_ms {
                    g.arm = ArmState::Armed;
                    let _ = win.set_ignore_cursor_events(false);
                    let _ = app.emit(
                        "overlay-arm",
                        ArmPayload {
                            armed: true,
                            dwelling: false,
                            progress: 1.0,
                        },
                    );
                } else {
                    let _ = app.emit(
                        "overlay-arm",
                        ArmPayload {
                            armed: false,
                            dwelling: true,
                            progress,
                        },
                    );
                }
            }
            ArmState::Armed | ArmState::Expanded => {}
        }
        Ok(Duration::from_millis(100))
    } else {
        if let Some(left_at) = g.leave_started {
            if now.duration_since(left_at) >= Duration::from_millis(LEAVE_DEBOUNCE_MS) {
                if g.arm != ArmState::ClickThrough && g.arm != ArmState::Expanded {
                    g.arm = ArmState::ClickThrough;
                    g.dwell_started = None;
                    let _ = win.set_ignore_cursor_events(true);
                    let _ = app.emit(
                        "overlay-arm",
                        ArmPayload {
                            armed: false,
                            dwelling: false,
                            progress: 0.0,
                        },
                    );
                }
            }
        } else if g.arm == ArmState::Dwelling || g.arm == ArmState::Armed {
            g.leave_started = Some(now);
        }
        Ok(if near {
            Duration::from_millis(100)
        } else {
            Duration::from_millis(250)
        })
    }
}

pub fn point_in_rect(cx: f64, cy: f64, rx: f64, ry: f64, rw: f64, rh: f64) -> bool {
    cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh
}

fn point_near_rect(cx: f64, cy: f64, rx: f64, ry: f64, rw: f64, rh: f64) -> bool {
    point_in_rect(cx, cy, rx, ry, rw, rh)
}

#[allow(dead_code)]
pub fn physical_placeholder() -> (PhysicalPosition<i32>, PhysicalSize<u32>) {
    (PhysicalPosition::new(0, 0), PhysicalSize::new(1, 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_test_inside_outside() {
        assert!(point_in_rect(150.0, 20.0, 100.0, 8.0, 340.0, 48.0));
        assert!(!point_in_rect(50.0, 20.0, 100.0, 8.0, 340.0, 48.0));
    }

    #[test]
    fn arm_state_defaults_click_through() {
        let c = OverlayController::new();
        assert_eq!(c.inner.lock().arm, ArmState::ClickThrough);
        c.set_expanded(true);
        assert_eq!(c.inner.lock().arm, ArmState::Expanded);
        c.set_expanded(false);
        assert_eq!(c.inner.lock().arm, ArmState::ClickThrough);
    }
}
