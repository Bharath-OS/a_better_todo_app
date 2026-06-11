let currentView = 'dashboard';
let settings = {};
let streakData = { current: 0, best: 0 };

// ============ Title Bar ============
const mainWin = () => window.__TAURI__.window.getCurrentWindow();

document.getElementById('minimize-btn').onclick = async () => {
  try { await mainWin().minimize(); } catch (e) { console.error('minimize error:', e); }
};
document.getElementById('maximize-btn').onclick = async () => {
  try { await mainWin().toggleMaximize(); } catch (e) { console.error('maximize error:', e); }
};
document.getElementById('close-btn').onclick = async () => {
  try { await window.__TAURI__.core.invoke('exit_app'); } catch (e) { console.error('exit error:', e); }
};

// ============ Sidebar ============
document.querySelectorAll('.nav-item').forEach(item => {
  item.onclick = () => navigateTo(item.dataset.view);
});

function navigateTo(view) {
  currentView = view;
  document.querySelectorAll('.nav-item').forEach(n => n.classList.toggle('active', n.dataset.view === view));
  renderContent();
}

// ============ Content Rendering ============
async function renderContent() {
  const content = document.getElementById('content');
  const [tasks, streak] = await Promise.all([getTasks(), getStreak()]);
  streakData = streak;
  updateStreak(streak.current);

  switch (currentView) {
    case 'dashboard': renderDashboard(content, tasks, streak); break;
    case 'daily': renderPeriodView(content, tasks, 'daily'); break;
    case 'weekly': renderPeriodView(content, tasks, 'weekly'); break;
    case 'quarterly': renderPeriodView(content, tasks, 'quarterly'); break;
    case 'yearly': renderPeriodView(content, tasks, 'yearly'); break;
    case 'settings': renderSettings(content); break;
  }
}

function updateStreak(count) {
  document.getElementById('streak-count').textContent = count;
}

// ============ Dashboard ============
function renderDashboard(container, tasks, streak) {
  const periods = ['daily', 'weekly', 'quarterly', 'yearly'];
  const periodLabels = { daily: 'Daily', weekly: 'Weekly', quarterly: 'Quarterly', yearly: 'Yearly' };
  const periodIcons = { daily: '\u{1F4C5}', weekly: '\u{1F4C8}', quarterly: '\u{2696}', yearly: '\u{1F30E}' };

  let html = '<div class="view-header"><div class="view-title">Good to see you</div><div class="view-subtitle">Here\'s how your follow-through looks today.</div></div>';

  // Period summary cards
  html += '<div class="dashboard-grid">';
  for (const period of periods) {
    const periodTasks = tasks.filter(t => t.period === period);
    const total = periodTasks.length;
    const completed = periodTasks.filter(t => t.completed).length;
    html += `
      <div class="period-card" onclick="navigateTo('${period}')">
        <div class="period-card-header">
          <span>${periodIcons[period]}</span>
          <span>${periodLabels[period]}</span>
        </div>
        <div class="period-card-numbers">
          <span class="period-card-completed">${completed}</span>
          <span class="period-card-total">/ ${total}</span>
        </div>
        <div class="progress-bar-track"><div class="progress-bar-fill" style="width:${total > 0 ? (completed/total*100) : 0}%"></div></div>
      </div>`;
  }
  html += '</div>';

  // Chart card
  html += '<div class="chart-card"><div class="chart-header"><h3>Completion &middot; Daily</h3></div><div class="chart-subtitle">This week&rsquo;s progress.</div><div class="chart-container" id="chart-container"></div></div>';

  // Stats cards
  html += '<div class="stats-row">';
  html += `
    <div class="stat-card">
      <div class="stat-icon" style="background:rgba(245,158,11,0.1);color:var(--warning);">&#128293;</div>
      <div class="stat-info"><div class="stat-label">Current streak</div><div class="stat-subtitle">Complete all daily tasks to extend.</div></div>
      <div class="stat-value">${streak.current} days</div>
    </div>
    <div class="stat-card">
      <div class="stat-icon" style="background:rgba(59,130,246,0.1);color:var(--primary);">&#127942;</div>
      <div class="stat-info"><div class="stat-label">Best streak</div><div class="stat-subtitle">Your record. Beat it!</div></div>
      <div class="stat-value">${streak.best} days</div>
    </div>`;
  html += '</div>';

  container.innerHTML = html;

  // Render chart
  renderChart();
}

async function renderChart() {
  const chartEl = document.getElementById('chart-container');
  if (!chartEl) return;
  const history = await getHistory();
  const isEmpty = history.length === 0 || history.every(h => h.total === 0);
  if (isEmpty) {
    chartEl.innerHTML = '<div class="chart-empty">Complete some tasks to see your progress here.</div>';
    return;
  }
  const days = [];
  const today = new Date();
  const dayLabels = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

  for (let i = 6; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(d.getDate() - i);
    const ds = d.toISOString().slice(0, 10);
    const entry = history.find(h => h.date === ds);
    const total = entry ? entry.total : 0;
    const completed = entry ? entry.completed : 0;
    const pct = total > 0 ? (completed / total * 100) : 0;
    days.push({ label: dayLabels[d.getDay()], pct, completed, total });
  }

  const maxPct = Math.max(...days.map(d => d.pct), 100);

  chartEl.innerHTML = days.map(d => `
    <div class="chart-bar-wrapper">
      <div class="chart-bar" style="height:${d.pct / maxPct * 100}%;min-height:${d.pct > 0 ? '2px' : '0'}"
           title="${d.pct.toFixed(0)}% (${d.completed}/${d.total})"></div>
      <div class="chart-bar-label">${d.label}</div>
    </div>
  `).join('');
}

// ============ Period Views ============
function renderPeriodView(container, tasks, period) {
  const labels = { daily: 'Daily', weekly: 'Weekly', quarterly: 'Quarterly', yearly: 'Yearly' };
  const periodTasks = tasks.filter(t => t.period === period);
  const incomplete = periodTasks.filter(t => !t.completed);
  const completed = periodTasks.filter(t => t.completed);
  const total = periodTasks.length;
  const done = completed.length;

  let html = `
    <div class="view-header-row">
      <div>
        <div class="view-title">${labels[period]} Tasks</div>
        <div class="view-subtitle">${done} of ${total} completed</div>
      </div>
      <button class="btn-primary" onclick="showAddDialog('${period}')">+ Add task</button>
    </div>`;

  if (periodTasks.length === 0) {
    html += `
      <div class="task-card">
        <div class="empty-state">
          <div class="empty-icon">&#10003;</div>
          <div class="empty-text">No ${labels[period].toLowerCase()} tasks yet. Click "Add task" to create one.</div>
        </div>
      </div>`;
  } else {
    html += '<div class="task-card"><div class="task-list">';
    for (const task of [...incomplete, ...completed]) {
      html += renderTaskItem(task);
    }
    html += '</div></div>';
  }

  container.innerHTML = html;
}

function renderTaskItem(task) {
  const created = formatDate(task.created_at);
  const deadlineHtml = task.deadline ? `<span class="task-deadline">${formatDeadline(task.deadline)}</span>` : '';
  const checked = task.completed ? 'checked' : '';
  const completedClass = task.completed ? 'completed' : '';

  return `
    <div class="task-item">
      <div class="task-checkbox ${checked}" onclick="handleToggle('${task.id}')"></div>
      <div class="task-title-area">
        <div class="task-title ${completedClass}">${escHtml(task.title)}</div>
        <div class="task-created">${created}</div>
      </div>
      ${deadlineHtml}
      <div class="task-actions">
        <button class="task-btn" onclick="showEditDialog('${task.id}', '${escHtml(task.title)}')" title="Edit">&#9998;</button>
        <button class="task-btn delete" onclick="handleDelete('${task.id}')" title="Delete">&#10005;</button>
      </div>
    </div>`;
}

// ============ Task Operations ============
async function handleToggle(id) {
  const tasks = await getTasks();
  const task = tasks.find(t => t.id === id);
  const wasIncomplete = task && !task.completed;
  await toggleTask(id);
  if (wasIncomplete) celebrate();
  renderContent();
}

async function handleDelete(id) {
  if (confirm('Delete this task?')) {
    await deleteTask(id);
    renderContent();
  }
}

// ============ Dialogs ============
function showAddDialog(period) {
  const overlay = document.createElement('div');
  overlay.className = 'dialog-overlay';
  overlay.innerHTML = `
    <div class="dialog">
      <h2>New task</h2>
      <p>Add a task to your ${period} list.</p>
      <div class="dialog-field">
        <label>Task title</label>
        <input type="text" id="dialog-title" placeholder="What needs to get done?" autofocus>
      </div>
      <div class="dialog-field" id="period-field" style="display:${period ? 'none' : 'block'}">
        <label>Period</label>
        <select id="dialog-period">
          <option value="daily" ${period === 'daily' ? 'selected' : ''}>Daily</option>
          <option value="weekly" ${period === 'weekly' ? 'selected' : ''}>Weekly</option>
          <option value="quarterly" ${period === 'quarterly' ? 'selected' : ''}>Quarterly</option>
          <option value="yearly" ${period === 'yearly' ? 'selected' : ''}>Yearly</option>
        </select>
      </div>
      <div class="dialog-footer">
        <button class="btn-outline" onclick="this.closest('.dialog-overlay').remove()">Cancel</button>
        <button class="btn-primary" id="dialog-submit">Add task</button>
      </div>
    </div>`;

  document.body.appendChild(overlay);
  const input = overlay.querySelector('#dialog-title');
  const submit = overlay.querySelector('#dialog-submit');
  input.focus();

  function submitTask() {
    const title = input.value.trim();
    if (!title) return;
    const p = period || overlay.querySelector('#dialog-period').value;
    createTask(title, p);
    overlay.remove();
    renderContent();
  }

  submit.onclick = submitTask;
  input.onkeydown = (e) => { if (e.key === 'Enter') submitTask(); if (e.key === 'Escape') overlay.remove(); };
  overlay.onclick = (e) => { if (e.target === overlay) overlay.remove(); };
}

function showEditDialog(id, currentTitle) {
  const overlay = document.createElement('div');
  overlay.className = 'dialog-overlay';
  overlay.innerHTML = `
    <div class="dialog">
      <h2>Edit task</h2>
      <p>Update the task title.</p>
      <div class="dialog-field">
        <label>Task title</label>
        <input type="text" id="dialog-title" value="${escHtml(currentTitle)}" autofocus>
      </div>
      <div class="dialog-footer">
        <button class="btn-outline" onclick="this.closest('.dialog-overlay').remove()">Cancel</button>
        <button class="btn-primary" id="dialog-submit">Save</button>
      </div>
    </div>`;

  document.body.appendChild(overlay);
  const input = overlay.querySelector('#dialog-title');
  input.focus();
  input.setSelectionRange(input.value.length, input.value.length);

  function submitEdit() {
    const title = input.value.trim();
    if (!title) return;
    updateTask(id, { title });
    overlay.remove();
    renderContent();
  }

  overlay.querySelector('#dialog-submit').onclick = submitEdit;
  input.onkeydown = (e) => { if (e.key === 'Enter') submitEdit(); if (e.key === 'Escape') overlay.remove(); };
  overlay.onclick = (e) => { if (e.target === overlay) overlay.remove(); };
}

// ============ Settings ============
async function renderSettings(container) {
  settings = await getSettings();

  container.innerHTML = `
    <div class="view-header"><div class="view-title">Settings</div></div>

    <div class="settings-card">
      <h3>Overlay</h3>
      <div class="settings-row">
        <span class="settings-label">Screen position</span>
        <div class="settings-control">
          <select id="setting-corner">
            <option value="tr" ${settings.overlay_corner === 'tr' ? 'selected' : ''}>Top-right</option>
            <option value="tl" ${settings.overlay_corner === 'tl' ? 'selected' : ''}>Top-left</option>
            <option value="br" ${settings.overlay_corner === 'br' ? 'selected' : ''}>Bottom-right</option>
            <option value="bl" ${settings.overlay_corner === 'bl' ? 'selected' : ''}>Bottom-left</option>
          </select>
        </div>
      </div>
      <div class="settings-row">
        <span class="settings-label">Collapsed opacity</span>
        <div class="settings-control">
          <input type="range" id="setting-opacity" min="20" max="100" value="${Math.round(parseFloat(settings.collapsed_opacity || 0.6) * 100)}" step="5">
          <span class="range-value" id="opacity-value">${Math.round(parseFloat(settings.collapsed_opacity || 0.6) * 100)}%</span>
        </div>
      </div>
      <div class="settings-row">
        <span class="settings-label">Hover activation delay</span>
        <div class="settings-control">
          <input type="range" id="setting-delay" min="0" max="2000" value="${settings.hover_delay_ms || 1200}" step="100">
          <span class="range-value" id="delay-value">${((settings.hover_delay_ms || 1200) / 1000).toFixed(1)}s</span>
        </div>
      </div>
      <div class="settings-row">
        <span class="settings-label">Show streak</span>
        <div class="settings-control">
          <label class="toggle-switch">
            <input type="checkbox" id="setting-streak" ${settings.show_streak === 'true' ? 'checked' : ''}>
            <div class="toggle-track"></div>
            <div class="toggle-thumb"></div>
          </label>
        </div>
      </div>
    </div>

    <div class="settings-card">
      <h3>Hotkeys</h3>
      <div class="hotkey-grid">
        <div class="hotkey-chip"><kbd>Win</kbd> + <kbd>T</kbd> Toggle overlay</div>
        <div class="hotkey-chip"><kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>T</kbd> Quick add</div>
      </div>
    </div>

    <div class="settings-card">
      <h3>Data</h3>
      <button class="btn-outline" onclick="handleReset()">Reset demo data</button>
    </div>`;

  // Bind settings events
  document.getElementById('setting-corner').onchange = (e) => updateSetting('overlay_corner', e.target.value);
  document.getElementById('setting-opacity').oninput = (e) => {
    document.getElementById('opacity-value').textContent = e.target.value + '%';
    updateSetting('collapsed_opacity', (parseInt(e.target.value) / 100).toFixed(2));
  };
  document.getElementById('setting-delay').oninput = (e) => {
    document.getElementById('delay-value').textContent = (parseInt(e.target.value) / 1000).toFixed(1) + 's';
    updateSetting('hover_delay_ms', e.target.value);
  };
  document.getElementById('setting-streak').onchange = (e) => updateSetting('show_streak', e.target.checked ? 'true' : 'false');
}

async function handleReset() {
  if (confirm('Reset all data? This will re-seed demo tasks.')) {
    await resetData();
    renderContent();
  }
}

// ============ Utilities ============
function formatDate(dateStr) {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  const now = new Date();
  const diff = Math.floor((now - d) / (1000 * 60 * 60 * 24));
  if (diff === 0) return 'Today';
  if (diff === 1) return 'Yesterday';
  return `${d.getMonth() + 1}/${d.getDate()}/${d.getFullYear()}`;
}

function formatDeadline(dateStr) {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  const now = new Date();
  const diff = d - now;
  if (diff < 0) return 'Overdue!';
  const hours = Math.floor(diff / (1000 * 60 * 60));
  if (hours < 1) return Math.floor(diff / (1000 * 60)) + 'm left';
  if (hours < 24) return hours + 'h left';
  const days = Math.floor(hours / 24);
  if (days === 1) return 'Tomorrow';
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

function escHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// ============ Init ============
renderContent();
