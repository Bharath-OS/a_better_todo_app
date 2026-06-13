const PERIOD_KEYS = { d: 'daily', w: 'weekly', q: 'quarterly', y: 'yearly' };
const input = document.getElementById('task-input');
const periodBtns = document.querySelectorAll('.period-btn');
let selectedPeriod = 'daily';
let closing = false;

input.focus();

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
