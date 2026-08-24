import { useEffect, useState } from 'react';
import { API_ORIGIN } from '../../config/endpoints.js';
import './ServerGate.css';

const PING_MS = 4000;
const HINT_AFTER_MS = 2500;

/**
 * If API is down (cold start), show connecting animation.
 * If already online, render children immediately (no flash).
 */
export default function ServerGate({ children }) {
  const [status, setStatus] = useState('checking'); // checking | online | offline
  const [showHint, setShowHint] = useState(false);

  useEffect(() => {
    let cancelled = false;
    let timer;
    let hintTimer;

    async function ping() {
      try {
        const ctrl = new AbortController();
        const t = setTimeout(() => ctrl.abort(), 8000);
        const res = await fetch(`${API_ORIGIN}/health`, {
          method: 'GET',
          cache: 'no-store',
          signal: ctrl.signal,
        });
        clearTimeout(t);
        if (!cancelled && res.ok) {
          setStatus('online');
          return true;
        }
      } catch (_) {}
      return false;
    }

    async function loop() {
      const ok = await ping();
      if (cancelled) return;
      if (ok) {
        setStatus('online');
        return;
      }
      setStatus('offline');
      timer = setTimeout(loop, PING_MS);
    }

    // First ping — if fast OK, skip UI
    (async () => {
      const ok = await ping();
      if (cancelled) return;
      if (ok) {
        setStatus('online');
      } else {
        setStatus('offline');
        hintTimer = setTimeout(() => setShowHint(true), HINT_AFTER_MS);
        timer = setTimeout(loop, PING_MS);
      }
    })();

    return () => {
      cancelled = true;
      clearTimeout(timer);
      clearTimeout(hintTimer);
    };
  }, []);

  if (status === 'online') return children;

  // checking: minimal blank/brand (avoid full connecting if server already warm)
  if (status === 'checking') {
    return (
      <div className="ck-server-gate ck-server-gate--quiet">
        <div className="ck-server-gate__logo">♚</div>
      </div>
    );
  }

  return (
    <div className="ck-server-gate">
      <div className="ck-server-gate__orb" aria-hidden>
        <span className="ck-server-gate__ring" />
        <span className="ck-server-gate__ring ck-server-gate__ring--2" />
        <span className="ck-server-gate__king">♚</span>
      </div>
      <h1 className="ck-server-gate__title">Connecting server…</h1>
      {showHint && (
        <p className="ck-server-gate__hint">
          Please wait 30–60 seconds
          <br />
          <span className="text-secondary">Free server may be waking up</span>
        </p>
      )}
      <div className="ck-server-gate__dots" aria-hidden>
        <span /><span /><span />
      </div>
    </div>
  );
}
