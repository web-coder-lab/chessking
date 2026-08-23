# Phase 10 — Ranked rating verify

## Rules
- Elo **K=32**, only `match_type == ranked`
- Casual / custom: ratings unchanged
- Floor **100**
- Single finalize wins race (`status != completed`) — no double apply

## Broadcast `match_ended`
- `white_rating_before/after`, `black_*`, `white_delta`, `black_delta`, `match_type`

## UI
- End screen shows your new rating + delta (ranked only)

## Sanity
Equal 1200, white wins → ~+16 / -16
