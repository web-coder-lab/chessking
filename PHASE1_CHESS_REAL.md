# 12 phases — Real chess (not simulation)

| Phase | Goal |
|-------|------|
| **1** | Move authority: server FEN, board sync, turn lock, honest voice UI |
| **2** | Matchmaking 2-player reliable + queue leave |
| **3** | Board UX: clocks, check highlight, promotion picker |
| **4** | Persist moves to DB every ply + reconnect resume |
| **5** | Clocks (time control) server-side |
| **6** | Draw offers / takeback rules |
| **7** | Sound + animations (real events only) |
| **8** | WebRTC voice (real mic) or remove |
| **9** | Anti-cheat move timing |
| **10** | Ranked rating updates verify |
| **11** | Spectator / share PGN |
| **12** | E2E live 2-device test checklist |

## Phase 1 done
- Backend `GameState::fen()` + `board_update.fen`
- ChessBoard: load match PGN, turn lock, server fen apply, no fake mic enable
- Illegal / not_your_turn toasts
