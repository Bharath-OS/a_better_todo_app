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
    const { WebviewWindow } = window.__TAURI__.window;
    const label = 'confetti-' + Date.now();
    const win = new WebviewWindow(label, {
      url: '/confetti.html',
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      focus: false,
    });
    win.once('tauri://created', async () => {
      try {
        const monitor = await win.currentMonitor();
        if (monitor) {
          const { width, height } = monitor.size;
          const scale = monitor.scaleFactor;
          await win.setPosition({ type: 'Logical', x: 0, y: 0 });
          await win.setSize({ type: 'Logical', width: width / scale, height: height / scale });
        }
      } catch (e) {}
    });
    setTimeout(async () => {
      try { await win.close(); } catch (e) {}
    }, 4000);
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
