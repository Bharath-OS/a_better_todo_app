const PERIOD_KEYS = { d: 'daily', w: 'weekly', q: 'quarterly', y: 'yearly' };
const input = document.getElementById('task-input');
const periodBtns = document.querySelectorAll('.period-btn');
const dialog = document.querySelector('.quick-add-dialog');
let selectedPeriod = 'daily';
let closing = false;

async function init() {
  const { getCurrentWindow } = window.__TAURI__.window;
  const win = getCurrentWindow();
  // Fill the full monitor for a seamless overlay
  const monitor = await win.currentMonitor;
  if (monitor) {
    await win.setSize({
      width: Math.round(monitor.size.width / monitor.scaleFactor),
      height: Math.round(monitor.size.height / monitor.scaleFactor),
    });
    await win.setPosition({
      x: Math.round(monitor.position.x / monitor.scaleFactor),
      y: Math.round(monitor.position.y / monitor.scaleFactor),
    });
  }
  // Keep interactive initially — never enable click-through until the
  // layout is stable and we've confirmed the cursor is truly outside.
  await win.setIgnoreCursorEvents(false);
  input.focus();
  // Wait one frame so the dialog layout is settled, then start polling
  requestAnimationFrame(() => pollHitTest());
}
init();

// ---------- Selective click-through ----------
// Only enable OS-level click-through (setIgnoreCursorEvents(true)) when
// the cursor has been outside the dialog for MULTIPLE consecutive polls.
// This prevents toggling during layout settling or brief cursor movement.
const POLL_MS = 80;
const OUTSIDE_THRESHOLD = 3; // require 3 consecutive "outside" readings
let consecutiveOutside = 0;
let ignoreEnabled = false;

async function pollHitTest() {
  const { getCurrentWindow } = window.__TAURI__.window;
  const win = getCurrentWindow();
  const rect = dialog.getBoundingClientRect();

  // Guard: don't enable click-through until the dialog has a valid layout
  const dialogReady = rect.width > 10 && rect.height > 10;

  try {
    const over = dialogReady && await window.__TAURI__.core.invoke('cursor_over_rect', {
      label: 'quick-add',
      rx: rect.x, ry: rect.y,
      rw: rect.width, rh: rect.height,
    });

    if (over) {
      consecutiveOutside = 0;
      if (ignoreEnabled) {
        ignoreEnabled = false;
        await win.setIgnoreCursorEvents(false);
      }
    } else if (dialogReady) {
      consecutiveOutside++;
      if (consecutiveOutside >= OUTSIDE_THRESHOLD && !ignoreEnabled) {
        ignoreEnabled = true;
        await win.setIgnoreCursorEvents(true);
      }
    }
  } catch (e) {
    // window might be closing
  }
  setTimeout(pollHitTest, POLL_MS);
}
// --------------------------------------------

periodBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    periodBtns.forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    selectedPeriod = btn.dataset.period;
  });
});

async function submit() {
  const title = input.value.trim();
  if (!title) return;
  try {
    await createTask(title, selectedPeriod);
  } catch (e) {
    console.error('createTask error:', e);
  }
  closeWindow();
}

function closeWindow() {
  if (closing) return;
  closing = true;
  try {
    const { getCurrentWindow } = window.__TAURI__.window;
    getCurrentWindow().close();
  } catch (e) {
    window.close();
  }
}

input.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') {
    e.preventDefault();
    submit();
  } else if (e.key === 'Escape') {
    e.preventDefault();
    closeWindow();
  } else if (e.ctrlKey && PERIOD_KEYS[e.key]) {
    e.preventDefault();
    selectedPeriod = PERIOD_KEYS[e.key];
    periodBtns.forEach(b => {
      b.classList.toggle('active', b.dataset.period === selectedPeriod);
    });
  }
});
