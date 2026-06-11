const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ============ Tasks ============
async function getTasks(period) {
  return await invoke('get_tasks', { period: period || null });
}

async function createTask(title, period, dueTime, notes) {
  return await invoke('create_task', {
    title,
    period,
    dueTime: dueTime || null,
    notes: notes || null
  });
}

async function toggleTask(id) {
  return await invoke('toggle_task', { id });
}

async function updateTask(id, opts = {}) {
  return await invoke('update_task', {
    id,
    title: opts.title || null,
    completed: opts.completed !== undefined ? opts.completed : null,
    dueTime: opts.dueTime !== undefined ? opts.dueTime : null,
    notes: opts.notes !== undefined ? opts.notes : null
  });
}

async function deleteTask(id) {
  return await invoke('delete_task', { id });
}

// ============ Settings ============
async function getSettings() {
  return await invoke('get_settings');
}

async function updateSetting(key, value) {
  return await invoke('update_setting', { key, value });
}

// ============ History & Streak ============
async function getHistory() {
  return await invoke('get_history');
}

async function getStreak() {
  return await invoke('get_streak');
}

// ============ Data ============
async function resetData() {
  return await invoke('reset_data');
}

// ============ Confetti ============
async function showConfetti() {
  try {
    const { WebviewWindow } = window.__TAURI__.webviewWindow;
    const label = 'confetti-' + Date.now();
    const w = window.screen.width;
    const h = window.screen.height;
    const win = new WebviewWindow(label, {
      url: '/confetti.html',
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      width: w,
      height: h,
      x: 0,
      y: 0,
      skipTaskbar: true,
      focus: false,
    });
    // Auto-cleanup after animation
    setTimeout(async () => {
      try {
        const { getCurrentWindow } = window.__TAURI__.window;
        // We can't close another window from here easily
      } catch (e) {}
    }, 3000);
  } catch (e) {
    console.warn('Confetti window failed:', e);
  }
}

// ============ Event Listeners ============
async function onTasksChanged(callback) {
  return await listen('tasks-changed', callback);
}

async function onQuickAdd(callback) {
  return await listen('quick-add', callback);
}
