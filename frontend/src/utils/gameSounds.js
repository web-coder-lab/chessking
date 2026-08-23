/**
 * Lightweight Web Audio beeps — no external assets.
 * Only call on real server-confirmed events.
 */

let ctx = null;

function getCtx() {
  if (typeof window === 'undefined') return null;
  if (!ctx) {
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return null;
    ctx = new AC();
  }
  return ctx;
}

function beep(freq, duration, type = 'sine', gain = 0.08) {
  const c = getCtx();
  if (!c) return;
  try {
    if (c.state === 'suspended') c.resume();
    const osc = c.createOscillator();
    const g = c.createGain();
    osc.type = type;
    osc.frequency.value = freq;
    g.gain.value = gain;
    osc.connect(g);
    g.connect(c.destination);
    const t = c.currentTime;
    g.gain.setValueAtTime(gain, t);
    g.gain.exponentialRampToValueAtTime(0.001, t + duration);
    osc.start(t);
    osc.stop(t + duration);
  } catch (_) {
    /* ignore autoplay blocks */
  }
}

export function soundMove() {
  beep(420, 0.06, 'triangle', 0.06);
}

export function soundCapture() {
  beep(280, 0.05, 'square', 0.05);
  setTimeout(() => beep(180, 0.08, 'square', 0.04), 40);
}

export function soundCheck() {
  beep(520, 0.08, 'sawtooth', 0.05);
  setTimeout(() => beep(660, 0.1, 'sawtooth', 0.05), 70);
}

export function soundGameEnd() {
  beep(330, 0.12, 'sine', 0.07);
  setTimeout(() => beep(440, 0.15, 'sine', 0.06), 100);
  setTimeout(() => beep(550, 0.2, 'sine', 0.05), 220);
}

export function soundDrawOffer() {
  beep(480, 0.1, 'sine', 0.05);
}
