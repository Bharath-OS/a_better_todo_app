const SUFFIX_MAP = { d: 'daily', w: 'weekly', q: 'quarterly', y: 'yearly' };
const input = document.getElementById('task-input');
const periodBtns = document.querySelectorAll('.period-btn');
let selectedPeriod = 'daily';

// Focus input on load
input.focus();

// Close when clicking outside the window (blur = lost focus)
setTimeout(() => {
  window.addEventListener('blur', () => closeWindow());
}, 0);

// Period button clicks
periodBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    periodBtns.forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    selectedPeriod = btn.dataset.period;
  });
});

// Parse suffix from value
function parseSuffix(value) {
  const parts = value.trim().split(/\s+/);
  const last = parts[parts.length - 1]?.toLowerCase();
  if (last && SUFFIX_MAP[last] && parts.length > 1) {
    return { period: SUFFIX_MAP[last], title: parts.slice(0, -1).join(' ').trim() };
  }
  return { period: null, title: value.trim() };
}

// Update active period based on suffix
input.addEventListener('input', () => {
  const { period } = parseSuffix(input.value);
  if (period) {
    selectedPeriod = period;
    periodBtns.forEach(b => {
      b.classList.toggle('active', b.dataset.period === period);
    });
  }
});

async function submit() {
  const { period, title } = parseSuffix(input.value);
  if (!title) return;
  const p = period || selectedPeriod;
  try {
    await createTask(title, p);
  } catch (e) {
    console.error('createTask error:', e);
  }
  closeWindow();
}

function closeWindow() {
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
  }
});
