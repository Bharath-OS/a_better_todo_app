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
  // Start with cursor events enabled so the dialog is interactive
  await win.setIgnoreCursorEvents(false);
  input.focus();
  // Poll cursor position to toggle click-through on transparent areas only
  pollHitTest();
}
init();

// Poll every 100ms: enable click-through when cursor is outside the dialog,
// disable it when cursor is over the dialog
let lastIgnore = false;

async function pollHitTest() {
  const { getCurrentWindow } = window.__TAURI__.window;
  const win = getCurrentWindow();
  const rect = dialog.getBoundingClientRect();
  try {
    const over = await window.__TAURI__.core.invoke('cursor_over_rect', {
      label: 'quick-add',
      rx: rect.x, ry: rect.y,
      rw: rect.width, rh: rect.height,
    });
    const shouldIgnore = !over;
    if (shouldIgnore !== lastIgnore) {
      lastIgnore = shouldIgnore;
      await win.setIgnoreCursorEvents(shouldIgnore);
    }
  } catch (e) {
    // window might be closing
  }
  setTimeout(pollHitTest, 100);
}

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
