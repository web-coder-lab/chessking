# Genius Clan — Security Phase 3: App WAF + rule hardening

**Status:** Implemented in code (no custom domain required).

## What shipped

### 1. Probe / scanner guard (`middleware/probe_guard.rs`)
Blocks common attack paths with **404** (no stack leak):

- `/.env`, `/.git`, `/.svn`, `/.aws`
- WordPress / phpMyAdmin probes
- Path traversal patterns (`../`, encoded)
- `TRACE` / `TRACK` methods

Layered on the full Axum router (including `/api/v1`).

### 2. Already active (Phase 4 original + Phase 5)
| Control | Detail |
|---------|--------|
| Global rate limit | ~3 req/s, burst 15, SmartIp |
| CORS allow-list | Frontend origins only |
| Security headers | nosniff, DENY frame, referrer, permissions |
| JWT on protected routes | require_auth |
| Login lockout | 5 fails / 15 min |
| Auth durable store | GitHub private repo |

### 3. Cloudflare (Phase 2) complementary rules
When domain is proxied, mirror probe block at edge (see SECURITY_PHASE2.md).

## Recommended rate matrix (document; partial in code)

| Route | Intent |
|-------|--------|
| POST /auth/login | App lockout + global governor |
| POST /auth/register | Global governor + GitHub index checks |
| POST /auth/forgot-password | Same as register velocity |
| GET /health | Unlimited (outside governor) |

Per-route governors can be added later without changing handlers.

## Test after deploy

```bash
curl -i https://genius-clan-api.onrender.com/.env
# expect 404

curl -i https://genius-clan-api.onrender.com/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"identifier":"x","password":"y"}'
# expect 401 JSON, not 500
```

## Next
- Phase 4: Turnstile / CAPTCHA after N failed logins
- Phase 5: Stricter per-route limits + fingerprint velocity
- Phase 6: Log aggregation / alerts
