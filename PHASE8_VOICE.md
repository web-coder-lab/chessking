# Phase 8 — Real WebRTC voice

## How
- Signaling: existing match WS `webrtc_signal` relay
- STUN: Google public STUN
- Audio only via `getUserMedia`

## UI
- 🎙️ **Enable voice** (permission + offer)
- Once on: 🎙️ / 🔇 **Mute toggle** (real track.enabled)

## Notes
- Both players should tap Enable voice
- Some mobile browsers need user gesture (button tap) — already required
- Strict NATs may need TURN (not configured on free tier)
