// ============ State ============
let state = 'collapsed'; // collapsed | peek | expanded
let activePeriod = 'daily';
let tasks = [];
let settings = {};
let peekTimer = null;

// Pill position state (in screen coordinates)
let pillPos = { x: 16, y: 16 };
let isDraggingPill = false;
let dragOffset = { x: 0, y: 0 };

const PILL_HEIGHT = 32;
const PILL_WIDTH = 168; // approximate
const PANEL_WIDTH = 340;
const PANEL_HEIGHT = 560;
const INSET = 16;

const appWindow = window.__TAURI__.window.getCurrentWindow();
const mainWindowLabel = 'main';

// ============ DOM refs ============
const pillContainer = document.getElementById('pill-container');
const pillDrag = document.getElementById('pill-drag');
const pillLabel = document.getElementById('pill-label');
const pillDot = document.getElementById('pill-dot');
const pillStreak = document.getElementById('pill-streak');
const pillStreakCount = document.getElementById('pill-streak-count');
const pillPeekExtras = document.getElementById('pill-peek-extras');
const pillCloseBtn = document.getElementById('pill-close-btn');

const panelContainer = document.getElementById('panel-container');
const panelDot = document.getElementById('panel-dot');
const panelTabs = document.getElementById('panel-tabs');
const panelTaskList = document.getElementById('panel-task-list');
const panelAddInput = document.getElementById('panel-add-input');
const panelAddBtn = document.getElementById('panel-add-btn');
const panelProgressFill = document.getElementById('panel-progress-fill');
const panelCount = document.getElementById('panel-count');
const panelStreak = document.getElementById('panel-streak');
const collapseBtn = document.getElementById('collapse-btn');

// ============ Init ============
async function init() {
  try {
    settings = await getSettings();
    await loadTasks();
  } catch (e) {
    console.error('overlay init error (non-fatal):', e);
  }
  updateDot();
  await updateStreakDisplay();
  updateTabCounts();
  bindEvents();
  applyOverlaySettings();
  await setState('collapsed');
}

// ============ Task Loading ============
async function loadTasks() {
  tasks = await getTasks();
  if (state === 'expanded') {
    renderTaskList();
    updateFooter();
  }
  updateDot();
  await updateStreakDisplay();
  updateTabCounts();
}

// ============ State Machine ============
async function setState(newState) {
  const prev = state;
  state = newState;

  // Clear peek timer
  if (peekTimer) { clearTimeout(peekTimer); peekTimer = null; }

  // Leaving expanded — restore pill visibility before measuring
  if (prev === 'expanded' && newState !== 'expanded') {
    panelContainer.style.display = 'none';
    pillContainer.style.display = 'flex';
  }

  pillContainer.classList.remove('state-collapsed', 'state-peek', 'state-expanded');

  if (newState === 'collapsed') {
    pillContainer.classList.add('state-collapsed');
    pillPeekExtras.style.display = 'none';
    pillContainer.style.width = '';
    applyOverlaySettings();
  } else if (newState === 'peek') {
    pillContainer.classList.add('state-peek');
    pillPeekExtras.style.display = 'flex';
    pillContainer.style.width = '';
    applyOverlaySettings();
  } else if (newState === 'expanded') {
    pillContainer.style.display = 'none';
    panelContainer.style.display = 'flex';
    pillContainer.classList.add('state-expanded');
    calculatePanelPosition();
    renderTaskList();
    updateFooter();
  }
}

function calculatePanelPosition() {
  // Get screen dimensions
  const screenW = window.innerWidth;
  const screenH = window.innerHeight;
  
  // Pill position and dimensions
  const pillX = pillPos.x;
  const pillY = pillPos.y;
  
  // Calculate available space in each direction
  const spaceAbove = pillY;
  const spaceBelow = screenH - (pillY + PILL_HEIGHT);
  const spaceLeft = pillX;
  const spaceRight = screenW - (pillX + PILL_WIDTH);
  
  let panelX = pillX;
  let panelY = pillY;
  
  // Determine horizontal position (prioritize right, then left)
  if (spaceRight >= PANEL_WIDTH) {
    // Expand to the right
    panelX = pillX + PILL_WIDTH;
  } else if (spaceLeft >= PANEL_WIDTH) {
    // Expand to the left
    panelX = pillX - PANEL_WIDTH;
  } else {
    // Not enough space on sides, position to avoid going off screen
    panelX = Math.max(INSET, Math.min(pillX, screenW - PANEL_WIDTH - INSET));
  }
  
  // Determine vertical position (prioritize down, then up)
  if (spaceBelow >= PANEL_HEIGHT) {
    // Expand downward
    panelY = pillY + PILL_HEIGHT;
  } else if (spaceAbove >= PANEL_HEIGHT) {
    // Expand upward
    panelY = pillY - PANEL_HEIGHT;
  } else {
    // Not enough space, position to avoid going off screen
    panelY = Math.max(INSET, Math.min(pillY, screenH - PANEL_HEIGHT - INSET));
  }
  
  // Apply positioning
  panelContainer.style.left = Math.round(panelX) + 'px';
  panelContainer.style.top = Math.round(panelY) + 'px';
  panelContainer.style.right = 'auto';
  panelContainer.style.bottom = 'auto';
}

// ============ Events ============
function bindEvents() {
  // Pill hover -> peek (delayed)
  let hoverTimer = null;
  let leaveTimer = null;
  pillContainer.addEventListener('mouseenter', () => {
    if (leaveTimer) { clearTimeout(leaveTimer); leaveTimer = null; }
    const delay = parseInt(settings.hover_delay_ms || 1200);
    hoverTimer = setTimeout(() => {
      if (state === 'collapsed') setState('peek');
    }, delay);
  });
  pillContainer.addEventListener('mouseleave', () => {
    if (hoverTimer) { clearTimeout(hoverTimer); hoverTimer = null; }
    if (state === 'peek') {
      leaveTimer = setTimeout(() => setState('collapsed'), 300);
    }
  });

  // Pill click -> expanded (only if not dragging)
  pillContainer.addEventListener('click', (e) => {
    if (isDraggingPill) return; // Don't expand if we were just dragging
    if (e.target.closest('.pill-close-btn')) return;
    if (state === 'peek' || state === 'collapsed') setState('expanded');
  });

  // Collapse button (panel)
  collapseBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    setState('collapsed');
  });

  // Close button in peek — show main window AND destroy overlay
  pillCloseBtn.addEventListener('click', async (e) => {
    e.stopPropagation();
    try {
      await window.__TAURI__.core.invoke('close_overlay_and_show_main');
    } catch (e) {
      console.error('close_overlay_and_show_main error:', e);
    }
  });

  // Tabs
  document.querySelectorAll('.panel-tab').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.panel-tab').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      activePeriod = tab.dataset.period;
      renderTaskList();
      updateFooter();
    });
  });

  // Add task
  panelAddBtn.addEventListener('click', addTaskFromInput);
  panelAddInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') addTaskFromInput();
  });

  // Escape key -> collapse
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && state === 'expanded') setState('collapsed');
  });

  // Pill drag (custom drag, not Tauri's)
  pillContainer.addEventListener('mousedown', startPillDrag);
  
  // Panel drag (on panel header)
  const panelHeader = document.getElementById('panel-header');
  if (panelHeader) {
    panelHeader.addEventListener('mousedown', startPanelDrag);
  }
}

// ============ Settings Sync ============
function measurePillWidth() {
  let w = 28;
  w += 6;
  w += 6;
  w += Math.max(40, pillLabel.offsetWidth || 40);
  if (streakShown()) {
    w += 6;
    w += Math.max(20, pillStreak.offsetWidth || 30);
  }
  if (pillPeekExtras.style.display !== 'none') {
    w += 6;
    w += Math.max(20, pillPeekExtras.offsetWidth || 30);
  }
  return Math.max(w, 60);
}

function streakShown() {
  return pillStreak.style.display !== 'none';
}

function applyOverlaySettings() {
  const opacity = parseFloat(settings.collapsed_opacity || '0.6');
  pillContainer.style.opacity = state !== 'expanded' ? opacity : '';
}

listen('settings-changed', async (event) => {
  const { key, value } = event.payload || {};
  if (key) settings[key] = value;
  if (!key) settings = await getSettings();
  await updateStreakDisplay();
  applyOverlaySettings();
});

// ============ Drag & Drop ============
let dragStartPos = null;

function startPillDrag(e) {
  if (e.target.closest('button')) return; // Don't drag when clicking buttons
  if (state !== 'collapsed' && state !== 'peek') return; // Only drag when not expanded
  
  isDraggingPill = true;
  dragStartPos = { x: e.clientX, y: e.clientY };
  dragOffset = {
    x: e.clientX - pillPos.x,
    y: e.clientY - pillPos.y
  };
  
  pillContainer.style.cursor = 'grabbing';
  document.addEventListener('mousemove', onPillDragMove);
  document.addEventListener('mouseup', stopPillDrag);
  e.preventDefault();
}

function onPillDragMove(e) {
  if (!isDraggingPill) return;
  
  const screenW = window.innerWidth;
  const screenH = window.innerHeight;
  
  // Calculate new position
  let newX = e.clientX - dragOffset.x;
  let newY = e.clientY - dragOffset.y;
  
  // Clamp to screen bounds
  newX = Math.max(INSET, Math.min(newX, screenW - PILL_WIDTH - INSET));
  newY = Math.max(INSET, Math.min(newY, screenH - PILL_HEIGHT - INSET));
  
  // Update pill position
  pillPos.x = newX;
  pillPos.y = newY;
  pillContainer.style.left = newX + 'px';
  pillContainer.style.top = newY + 'px';
}

function stopPillDrag() {
  if (!isDraggingPill) return;
  isDraggingPill = false;
  pillContainer.style.cursor = 'grab';
  document.removeEventListener('mousemove', onPillDragMove);
  document.removeEventListener('mouseup', stopPillDrag);
  dragStartPos = null;
}

function startPanelDrag(e) {
  if (e.target.closest('button')) return;
  dragStartPos = { x: e.screenX, y: e.screenY };
  document.addEventListener('mousemove', onPanelDragMove);
  document.addEventListener('mouseup', stopPanelDrag);
}

async function onPanelDragMove(e) {
  if (!dragStartPos) return;
  const dx = e.screenX - dragStartPos.x;
  const dy = e.screenY - dragStartPos.y;
  if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
    document.removeEventListener('mousemove', onPanelDragMove);
    document.removeEventListener('mouseup', stopPanelDrag);
    dragStartPos = null;
    try {
      await appWindow.startDragging();
    } catch (e) {
      console.error('drag error:', e);
    }
  }
}

function stopPanelDrag() {
  dragStartPos = null;
  document.removeEventListener('mousemove', onPanelDragMove);
  document.removeEventListener('mouseup', stopPanelDrag);
}

// ============ Task List Rendering ============
function renderTaskList() {
  const periodTasks = tasks.filter(t => t.period === activePeriod);
  const incomplete = periodTasks.filter(t => !t.completed);
  const completed = periodTasks.filter(t => t.completed);

  if (periodTasks.length === 0) {
    panelTaskList.innerHTML = '<div class="panel-empty">No tasks yet. Add one below!</div>';
    return;
  }

  panelTaskList.innerHTML = [...incomplete, ...completed].map(task => {
    const checked = task.completed ? 'checked' : '';
    const completedClass = task.completed ? 'completed' : '';
    return `
      <div class="panel-task-item" data-id="${task.id}">
        <div class="panel-task-checkbox ${checked}" onclick="handleOverlayToggle('${task.id}')"></div>
        <div class="panel-task-title ${completedClass}">${escHtml(task.title)}</div>
        <button class="panel-task-delete" onclick="handleOverlayDelete('${task.id}')">&#10005;</button>
      </div>`;
  }).join('');
}

function updateTabCounts() {
  document.querySelectorAll('.panel-tab').forEach(tab => {
    const period = tab.dataset.period;
    const count = tasks.filter(t => t.period === period).length;
    tab.textContent = `${period.charAt(0).toUpperCase() + period.slice(1)} (${count})`;
  });
}

function updateFooter() {
  const periodTasks = tasks.filter(t => t.period === activePeriod);
  const total = periodTasks.length;
  const completed = periodTasks.filter(t => t.completed).length;
  const pct = total > 0 ? (completed / total * 100) : 0;
  panelProgressFill.style.width = pct + '%';
  panelCount.textContent = `${completed}/${total}`;
}

function updateDot() {
  const dailyTasks = tasks.filter(t => t.period === 'daily');
  const incomplete = dailyTasks.filter(t => !t.completed).length;
  if (dailyTasks.length === 0) {
    [pillDot, panelDot].forEach(el => { if (el) el.style.backgroundColor = 'var(--muted-fg, #9CA3AF)'; });
    pillLabel.textContent = 'No tasks';
  } else {
    const color = incomplete > 0 ? 'var(--warning, #F59E0B)' : 'var(--success, #22C55E)';
    [pillDot, panelDot].forEach(el => { if (el) el.style.backgroundColor = color; });
    pillLabel.textContent = incomplete > 0 ? `${incomplete} left` : 'All done!';
  }
}

async function updateStreakDisplay() {
  const s = await getStreak();
  const show = settings.show_streak === 'true';
  if (show) {
    pillStreak.style.display = 'inline';
    pillStreakCount.textContent = s.current;
    panelStreak.style.display = 'inline';
    panelStreak.innerHTML = `&#128293; ${s.current}`;
  } else {
    pillStreak.style.display = 'none';
    panelStreak.style.display = 'none';
  }
}

// ============ Task Operations ============
async function addTaskFromInput() {
  const title = panelAddInput.value.trim();
  if (!title) return;
  await createTask(title, activePeriod);
  panelAddInput.value = '';
  await loadTasks();
  renderTaskList();
  updateFooter();
  updateTabCounts();
}

async function handleOverlayToggle(id) {
  const task = tasks.find(t => t.id === id);
  const wasIncomplete = task && !task.completed;
  await toggleTask(id);
  if (task) task.completed = !task.completed;
  if (wasIncomplete) celebrate();
  await loadTasks();
  updateDot();
}

async function handleOverlayDelete(id) {
  await deleteTask(id);
  await loadTasks();
  updateDot();
}

// Make functions accessible from HTML onclick
window.handleOverlayToggle = handleOverlayToggle;
window.handleOverlayDelete = handleOverlayDelete;

// ============ Listen for external updates ============
onQuickAdd(async (title) => {
  await createTask(title, 'daily');
  await loadTasks();
  updateDot();
  if (state === 'expanded') {
    renderTaskList();
    updateFooter();
    updateTabCounts();
  }
});

onTasksChanged(async () => {
  await loadTasks();
  if (state === 'expanded') {
    renderTaskList();
    updateFooter();
    updateTabCounts();
  }
  updateDot();
  await updateStreakDisplay();
});

// ============ Utility ============
function escHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// ============ Boot ============
init();
