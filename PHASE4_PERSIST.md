# Phase 4 — Persist + resume

## Backend
- Every legal move: `UPDATE matches SET pgn = ?` (UCI list)
- Resume: `ensure_session_loaded` rebuilds `GameState` from DB pgn if RAM empty
- On resume WS: `board_sync` { fen, pgn, color }

## Frontend
- Listens for `board_sync` and reloads FEN

## Test
1. Start match, make moves
2. Refresh board page / reconnect WS
3. Position should match last moves
