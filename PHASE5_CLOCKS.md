# Phase 5 — Server clocks

- Default **10+0** (`DEFAULT_CLOCK_MS`)
- On each move attempt: deduct think time from side-to-move
- Flag (0 ms) → opponent wins (`result_reason`: timeout / stored as disconnect_timeout in DB constraint)
- `board_update` / `board_sync` include `white_ms`, `black_ms`
- FE displays server times when provided; local tick still smooths UI between updates

Not yet: persist clock to SQLite across full process restart.
