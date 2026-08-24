/**
 * Single source of truth — web SPA + Capacitor APK.
 */
const isBrowser = typeof window !== 'undefined';
const host = isBrowser ? window.location?.hostname || '' : '';
const onRender = host.includes('onrender.com');
const isNative =
  isBrowser &&
  (document.documentElement.classList.contains('ck-native'));

const PROD_API = 'https://genius-clan-api.onrender.com';
const PROD_WS = 'wss://genius-clan-api.onrender.com';
const DEV_API = 'http://localhost:8080';
const DEV_WS = 'ws://localhost:8080';

function pickApi() {
  if (import.meta.env?.VITE_API_BASE) return String(import.meta.env.VITE_API_BASE).replace(/\/$/, '');
  if (onRender || isNative) return PROD_API;
  return DEV_API;
}

function pickWs() {
  if (import.meta.env?.VITE_WS_BASE) return String(import.meta.env.VITE_WS_BASE).replace(/\/$/, '');
  if (onRender || isNative) return PROD_WS;
  return DEV_WS;
}

export const API_ORIGIN = pickApi();
export const WS_ORIGIN = pickWs();
export const API_BASE = `${API_ORIGIN}/api/v1`;
export const WS_BASE = `${WS_ORIGIN}/api/v1`;

export const ENDPOINTS = {
  health: `${API_ORIGIN}/health`,
  apiRoot: `${API_BASE}/`,
  login: `${API_BASE}/auth/login`,
  refresh: `${API_BASE}/auth/refresh`,
  matchQueueWs: `${WS_BASE}/match/queue`,
};
