# Genius Clan ♟️

Full-stack multiplayer chess platform (project codename: Chess King → brand: **Genius Clan**).

**Phase 1:** Render free tier — `genius-clan` + `genius-clan-api` (see `PHASES.md`, `render.yaml`).

## Stack
- **Backend**: Rust (Axum + SQLx + SQLite) — auth, wallet, shop, gifts, matchmaking, anti-cheat, admin
- **Frontend**: React + Vite + React Router
- **Chess engine**: shakmaty (rules + legality)

## Features
- Ranked / Casual / Custom matchmaking
- Real-time WebSocket moves
- Wallet (JazzCash / EasyPaisa / Google Pay stubs)
- Shop + Inventory + Gifts with tiered animations
- 2FA + device approval flow
- Anti-cheat risk scoring + ban escalation
- Daily rewards, referrals, leaderboards
- Email verification / password reset (SMTP)

## Quick Start

### Backend
```bash
cd backend
cp .env.example .env
# edit JWT_SECRET at minimum
cargo run
# listens on :8080
```

### Frontend
```bash
cd frontend
npm install
npm run dev
# http://localhost:5173
```

Set `VITE_API_BASE=http://localhost:8080` if needed.

## Notes
- Payment gateways are **stubs** — replace in `wallet/gateway.rs` with real API calls.
- Hint system currently returns first legal move (placeholder).
- WebRTC voice chat signaling exists on backend; frontend peer connection not implemented yet.
- See `BUGFIX_REPORT.md` for the full audit history of fixes.

## License
Educational / private use.
