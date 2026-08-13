const WS_BASE = (import.meta.env?.VITE_WS_BASE || 'ws://localhost:8080') + '/api/v1';

/**
 * Thin wrapper around the backend's match WebSocket (game/websocket.rs).
 * One connection handles queueing, move play, and WebRTC signal relay via
 * a `type` field in each message (see backend for the full protocol).
 */
export class GameSocket {
  constructor(accessToken) {
    this.accessToken = accessToken;
    this.ws = null;
    this.listeners = {};
  }

  connect(path = '/match/queue') {
    this.ws = new WebSocket(`${WS_BASE}${path}?token=${this.accessToken}`);
    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        this._emit(msg.type, msg);
      } catch {
        // ignore malformed frames
      }
    };
    this.ws.onclose = () => this._emit('__closed', {});
    this.ws.onerror = () => this._emit('__error', {});
    return new Promise((resolve) => {
      this.ws.onopen = () => resolve();
    });
  }

  on(type, callback) {
    if (!this.listeners[type]) this.listeners[type] = [];
    this.listeners[type].push(callback);
  }

  _emit(type, payload) {
    (this.listeners[type] || []).forEach((cb) => cb(payload));
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

  webrtcSignal(matchId, payload) {
    this.send({ type: 'webrtc_signal', match_id: matchId, payload });
  }

  close() {
    this.ws?.close();
  }
}
