# Phase 9 — Anti-cheat move timing

## Server
- Tracks per-side think times (`white_move_times_ms` / `black_move_times_ms`)
- After ply > 4: if **4 consecutive** moves under **80ms** → `timing_anomaly`
- Logs `impossible_move_timing` via `security_events` / risk score
- Does **not** auto-void on timing alone (Doc 8: need ≥2 independent signals)

## Thresholds
| Constant | Value |
|----------|--------|
| FAST_MOVE_MS | 80 |
| FAST_MOVE_STREAK_FOR_ANOMALY | 4 |
| Opening ignore | first 4 plies |

## Next hardening (later)
- Engine similarity signal
- Screen / bot-tool signal
- Combined mid-match pause flow
