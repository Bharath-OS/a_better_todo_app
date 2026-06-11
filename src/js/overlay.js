// ============ State ============
let state = 'collapsed'; // collapsed | peek | expanded
let activePeriod = 'daily';
let tasks = [];
let settings = {};
let peekTimer = null;


const PILL_WIDTH = 168;
const PILL_HEIGHT = 32;
const PEEK_WIDTH = 204;
const PANEL_WIDTH = 340;
const PANEL_HEIGHT = 560;

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
  updateStreakDisplay();
  updateTabCounts();
  bindEvents();
  try {
    await setWindowSize(PILL_WIDTH, PILL_HEIGHT);
  } catch (e) {
    console.error('overlay setWindowSize error (non-fatal):', e);
  }
}

// ============ Task Loading ============
async function loadTasks() {
  tasks = await getTasks();
  if (state === 'expanded') {
    renderTaskList();
    updateFooter();
  }
  updateDot();
  updateStreakDisplay();
  updateTabCounts();
}

// ============ State Machine ============
async function setState(newState) {
  const prev = state;
  state = newState;

  // Clear peek timer
  if (peekTimer) { clearTimeout(peekTimer); peekTimer = null; }

  pillContainer.classList.remove('state-collapsed', 'state-peek', 'state-expanded');
  if (newState === 'collapsed') {
    pillContainer.classList.add('state-collapsed');
    pillPeekExtras.style.display = 'none';
    pillContainer.style.width = PILL_WIDTH + 'px';
    await setWindowSize(PILL_WIDTH, PILL_HEIGHT);
  } else if (newState === 'peek') {
    pillContainer.classList.add('state-peek');
    pillPeekExtras.style.display = 'flex';
    pillContainer.style.width = PEEK_WIDTH + 'px';
    await setWindowSize(PEEK_WIDTH, PILL_HEIGHT);
  } else if (newState === 'expanded') {
    pillContainer.style.display = 'none';
    panelContainer.style.display = 'flex';
    pillContainer.classList.add('state-expanded');
    await setWindowSize(PANEL_WIDTH, PANEL_HEIGHT);
    renderTaskList();
    updateFooter();
  }

  // Leaving expanded
  if (prev === 'expanded' && newState !== 'expanded') {
    panelContainer.style.display = 'none';
    pillContainer.style.display = 'flex';
  }
}

async function setWindowSize(w, h) {
    try {
      await appWindow.setResizable(true);
      const monitor = await appWindow.currentMonitor();
      const inset = 16;
      if (monitor) {
        const pos = monitor.position;
        const size = monitor.size;
        const scale = monitor.scaleFactor;
        const mx = pos.x, my = pos.y, mw = size.width, mh = size.height;
        let targetX, targetY;
        if (settings.overlay_corner === 'tr' || !settings.overlay_corner) {
          targetX = (mx + mw - w - inset) / scale;
          targetY = (my + inset) / scale;
        } else if (settings.overlay_corner === 'tl') {
          targetX = (mx + inset) / scale;
          targetY = (my + inset) / scale;
        } else if (settings.overlay_corner === 'br') {
          targetX = (mx + mw - w - inset) / scale;
          targetY = (my + mh - h - inset) / scale;
        } else if (settings.overlay_corner === 'bl') {
          targetX = (mx + inset) / scale;
          targetY = (my + mh - h - inset) / scale;
        }
        await appWindow.setPosition({ type: 'Logical', x: targetX, y: targetY });
      } else {
        await appWindow.setPosition({ type: 'Logical', x: 16, y: 16 });
      }
      await appWindow.setSize({ type: 'Logical', width: w, height: h });
      await appWindow.setResizable(false);
  } catch (e) {
    console.error('window resize error:', e);
  }
}

// ============ Events ============
function bindEvents() {
  // Pill hover -> peek (delayed)
  let hoverTimer = null;
  pillContainer.addEventListener('mouseenter', () => {
    const delay = parseInt(settings.hover_delay_ms || 1200);
    hoverTimer = setTimeout(() => {
      if (state === 'collapsed') setState('peek');
    }, delay);
  });
  pillContainer.addEventListener('mouseleave', () => {
    if (hoverTimer) { clearTimeout(hoverTimer); hoverTimer = null; }
    if (state === 'peek') setState('collapsed');
  });

  // Pill click -> expanded
  pillContainer.addEventListener('click', (e) => {
    if (e.target.closest('.pill-close-btn')) return;
    if (state === 'peek' || state === 'collapsed') setState('expanded');
  });

  // Collapse button
  collapseBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    setState('collapsed');
  });

  // Close button (peek + panel)
  pillCloseBtn.addEventListener('click', async (e) => {
    e.stopPropagation();
    await restoreMainWindow();
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

  // Drag (on pill drag area or panel header)
  const dragTargets = [pillDrag, document.getElementById('panel-header')];
  dragTargets.forEach(el => {
    if (!el) return;
    el.addEventListener('mousedown', startDrag);
  });
}

// ============ Drag ============
let dragStartPos = null;

function startDrag(e) {
  if (e.target.closest('button')) return;
  dragStartPos = { x: e.screenX, y: e.screenY };
  document.addEventListener('mousemove', onDragMove);
  document.addEventListener('mouseup', stopDrag);
}

async function onDragMove(e) {
  if (!dragStartPos) return;
  const dx = e.screenX - dragStartPos.x;
  const dy = e.screenY - dragStartPos.y;
  if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
    document.removeEventListener('mousemove', onDragMove);
    document.removeEventListener('mouseup', stopDrag);
    dragStartPos = null;
    try {
      await appWindow.startDragging();
    } catch (e) {
      console.error('drag error:', e);
    }
  }
}

function stopDrag() {
  dragStartPos = null;
  document.removeEventListener('mousemove', onDragMove);
  document.removeEventListener('mouseup', stopDrag);
}

// ============ Restore Main Window ============
async function restoreMainWindow() {
  try {
    const all = await window.__TAURI__.window.getAllWindows();
    const main = all.find(w => w.label === mainWindowLabel);
    if (main) {
      await main.show();
      await main.setFocus();
    }
  } catch (e) {
    console.error('restoreMainWindow error:', e);
  }
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
  const incomplete = tasks.filter(t => !t.completed && t.period === 'daily').length;
  const color = incomplete > 0 ? 'var(--warning, #F59E0B)' : 'var(--success, #22C55E)';
  [pillDot, panelDot].forEach(el => { if (el) el.style.backgroundColor = color; });
  pillLabel.textContent = incomplete > 0 ? `${incomplete} left` : 'All done!';
}

function updateStreakDisplay() {
  getStreak().then(s => {
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
  });
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
  updateStreakDisplay();
});

// ============ Utility ============
function escHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// ============ Boot ============
init();
