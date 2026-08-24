# Phase 4 — Matchmaking + Chess board (Compose)

| Component | Role |
|-----------|------|
| `GameSocket` | OkHttp WebSocket → queue + match |
| `PlayScreen` | Casual / Ranked queue |
| `BoardScreen` | 8×8 FEN board, tap moves, resign/draw |
| `Fen` | Parse FEN for UI glyphs |

Server remains move authority (`illegal_move` / `board_update`).
