import { WS_BASE } from '../config/endpoints.js';

/**
 * Match WebSocket — queue + in-match moves.
 */
export class GameSocket {
  constructor(accessToken) {
    this.accessToken = accessToken;
    this.ws = null;
    this.listeners = {};
  }

  connect(path = '/match/queue') {
    return new Promise((resolve, reject) => {
      if (!this.accessToken) {
        reject(new Error('Not signed in. Please log in again.'));
        return;
      }
      const url = `${WS_BASE}${path}?token=${encodeURIComponent(this.accessToken)}`;
      let settled = false;
      try {
        this.ws = new WebSocket(url);
      } catch (e) {
        reject(new Error('Could not open matchmaking connection.'));
        return;
      }

      const timer = setTimeout(() => {
        if (settled) return;
        settled = true;
        try { this.ws.close(); } catch (_) {}
        reject(new Error('Matchmaking connection timed out. Server may be waking up — try again.'));
      }, 20000);

      this.ws.onopen = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        resolve();
      };

      this.ws.onerror = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        reject(new Error('Matchmaking failed to connect. Check login and try again.'));
      };

      this.ws.onclose = (ev) => {
        clearTimeout(timer);
        this._emit('__closed', { code: ev.code, reason: ev.reason });
        if (!settled) {
          settled = true;
          reject(new Error(
            ev.code === 1006
              ? 'Connection closed. Server may be down or token expired — log in again.'
              : `Connection closed (${ev.code}).`
          ));
        }
      };

      this.ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          this._emit(msg.type, msg);
        } catch {
          // ignore
        }
      };
    });
  }

  on(type, callback) {
    if (!this.listeners[type]) this.listeners[type] = [];
    this.listeners[type].push(callback);
  }

  off(type, callback) {
    if (!this.listeners[type]) return;
    this.listeners[type] = this.listeners[type].filter((cb) => cb !== callback);
  }

  _emit(type, payload) {
    (this.listeners[type] || []).forEach((cb) => {
      try { cb(payload); } catch (_) {}
    });
  }

  send(message) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  joinQueue(matchType) {
    this.send({ type: 'join_queue', match_type: matchType });
  }

  resumeMatch(matchId) {
    this.send({ type: 'resume_match', match_id: matchId });
  }

  move(matchId, from, to, promotion) {
    this.send({ type: 'move', match_id: matchId, from, to, promotion: promotion || null });
  }

  resign(matchId) {
    this.send({ type: 'resign', match_id: matchId });
  }

  offerDraw(matchId) {
    this.send({ type: 'offer_draw', match_id: matchId });
  }

  acceptDraw(matchId) {
    this.send({ type: 'accept_draw', match_id: matchId });
  }

  declineDraw(matchId) {
    this.send({ type: 'decline_draw', match_id: matchId });
  }

  webrtcSignal(matchId, payload) {
    this.send({ type: 'webrtc_signal', match_id: matchId, payload });
  }

  close() {
    try {
      if (this.ws && this.ws.readyState <= 1) this.ws.close();
    } catch (_) {}
    this.ws = null;
  }
}
