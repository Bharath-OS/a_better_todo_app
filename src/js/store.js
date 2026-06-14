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
  try {
    invoke('spawn_celebration_window');
  } catch (e) {
    console.error('Failed to spawn celebration window:', e);
  }
}

// ============ Event Listeners ============
async function onTasksChanged(callback) {
  return await listen('tasks-changed', callback);
}

async function onQuickAdd(callback) {
  return await listen('quick-add', callback);
}
