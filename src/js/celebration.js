const duration = 1200;
const end = Date.now() + duration;
const colors = ['#3B82F6', '#22C55E', '#F59E0B', '#EC4899', '#A855F7'];

// Initial burst
confetti({
  particleCount: 120,
  spread: 90,
  startVelocity: 45,
  origin: { x: 0.5, y: 0.5 },
  colors,
  zIndex: 9999,
});

function frame() {
  confetti({ particleCount: 6, angle: 60, spread: 65, startVelocity: 55, origin: { x: 0, y: 0.7 }, colors, zIndex: 9999 });
  confetti({ particleCount: 6, angle: 120, spread: 65, startVelocity: 55, origin: { x: 1, y: 0.7 }, colors, zIndex: 9999 });
  
  if (Date.now() < end) {
    requestAnimationFrame(frame);
  } else {
    // Wait for remaining particles to fall
    setTimeout(() => {
      try {
        const { getCurrentWindow } = window.__TAURI__.window;
        getCurrentWindow().close();
      } catch (e) {
        window.close();
      }
    }, 1500);
  }
}

frame();
