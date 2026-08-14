# Part 2 — Durable sessions (GitHub)

## What
Refresh sessions dual-written to private GitHub repo so login survives Render sleep/restart.

## Storage
- `data/sessions/{session_id}.json`
- `data/indexes/sessions_by_hash.json` (refresh hash → session id)

## Code
- `auth/github_sessions.rs` — save / find_by_hash / deactivate
- `create_session(..., gh)` — SQL + GitHub
- `rotate_refresh_token(..., gh)` — SQL first, else GitHub recovery, then dual-write new hash

## Client
Still sends refresh via localStorage/cookie; server must find session after restart (this part).

## Next
Part 3 — durable wallet + daily claim
