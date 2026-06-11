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
  await invoke('update_setting', { key, value });
  try { await window.__TAURI__.event.emit('settings-changed', { key, value }); } catch (e) {}
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
function celebrate() {
  // Fire in-window confetti (always works)
  try {
    if (typeof confetti !== 'undefined') {
      const duration = 1200;
      const end = Date.now() + duration;
      const colors = ['#3B82F6', '#22C55E', '#F59E0B', '#EC4899', '#A855F7'];
      confetti({
        particleCount: 120,
        spread: 90,
        startVelocity: 45,
        origin: { x: 0.5, y: 0.5 },
        colors,
        zIndex: 9999,
      });
      function frame() {
        confetti({
          particleCount: 6,
          angle: 60,
          spread: 65,
          startVelocity: 55,
          origin: { x: 0, y: 0.7 },
          colors,
          zIndex: 9999,
        });
        confetti({
          particleCount: 6,
          angle: 120,
          spread: 65,
          startVelocity: 55,
          origin: { x: 1, y: 0.7 },
          colors,
          zIndex: 9999,
        });
        if (Date.now() < end) requestAnimationFrame(frame);
      }
      frame();
    }
  } catch (e) {}

  // Also try fullscreen window (best-effort)
  try {
    const { WebviewWindow } = window.__TAURI__.window;
    const label = 'confetti-' + Date.now();
    new WebviewWindow(label, {
      url: '/confetti.html',
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      focus: false,
    });
    setTimeout(async () => {
      try {
        const w = WebviewWindow.getByLabel(label);
        if (w) await w.close();
      } catch (e) {}
    }, 4000);
  } catch (e) {}
}

// ============ Event Listeners ============
async function onTasksChanged(callback) {
  return await listen('tasks-changed', callback);
}

async function onQuickAdd(callback) {
  return await listen('quick-add', callback);
}
