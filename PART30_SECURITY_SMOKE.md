# Part 30 — Security smoke

## Live checks (2026-08-16)

| Check | Result |
|-------|--------|
| `GET /health` | **200** `ok` |
| `GET /.env` | **404** Genius HTML (probe blocked) |
| `GET /.git/config` | **404** Genius HTML |
| `IP_ALLOWLIST` env | empty = public mode |
| `POST /api/v1/auth/login` | **404** Genius HTML — API routes **not live** (build queue/fail) |

## Code security layers (when binary is live)
1. `probe_guard` — scanner paths → Genius 404
2. `ip_allowlist` — empty = open; non-empty = allow only listed IPs
3. `tower_governor` — rate limit on `/api/v1`
4. Auth-specific governor (~1 rps) on auth routes
5. CORS locked to frontend origins
6. Security headers (nosniff, DENY frame, etc.)
7. Fallback → Genius 404 (no stack traces)

## After green deploy re-test
```bash
curl -s https://genius-clan-api.onrender.com/api/v1/auth/login \
  -H 'Content-Type: application/json' -d '{"identifier":"x","password":"y"}'
# expect JSON error, not HTML 404

for i in 1 2 3 4 5 6 7; do
  curl -s -o /dev/null -w "%{http_code}\n" -X POST .../auth/login -d '...'
done
# expect eventual 429 rate_limited
```
