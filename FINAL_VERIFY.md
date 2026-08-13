# Genius Clan — Final verification (2026-08-13)

## Database connection

| Layer | Config | Status |
|-------|--------|--------|
| Durable store | Private repo `web-coder-lab/genius-clan-database` via GitHub Contents API | **Connected (HTTP 200 on indexes)** |
| Local on Render | `DATABASE_URL=sqlite::memory:` | **No disk files** |
| Render Postgres | Deleted | **None** |
| Probe | `GET /health/store` | Returns GitHub reachability JSON |

Env required on API:
`GITHUB_DATA_TOKEN`, `GITHUB_DATA_OWNER`, `GITHUB_DATA_REPO`, `GITHUB_DATA_BRANCH`

## Critical architecture gap (honest)

Business logic (auth, wallet, matches, …) still mostly uses **sqlx SqlitePool**.
That pool is **in-memory** on Render → data **does not survive** restart until each module is migrated to `GitHubStore`.

`GitHubStore` API is ready (`get_json` / `put_json` / indexes). Migration of all handlers = next engineering phase.

## Files removed / cleaned

- `backend/Dockerfile` (duplicate; root `Dockerfile` is source of truth)
- Local `frontend/node_modules` (gitignored; not in repo)
- Render Postgres instance (already deleted)

## Frontend routes — verified earlier (Phase 2)

All App.jsx routes resolve; BottomNav OK; Profile→Settings OK; no empty onClick.

## Known remaining issues (not fixed in this pass)

1. Auth/wallet/game not yet reading/writing GitHub JSON (only store layer exists)
2. Payment gateways are stubs (TODO in gateway.rs)
3. Voice chat UI disabled
4. Free Render sleep + cold start
5. GitHub API rate limits for high traffic
6. PAT in env should be fine-grained + rotated

## Live

- https://genius-clan.onrender.com
- https://genius-clan-api.onrender.com/health → `ok`
