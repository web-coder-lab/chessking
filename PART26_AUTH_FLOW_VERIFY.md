# Part 26 — Auth flow verification (code)

## Flow (implemented)
1. `POST /api/v1/auth/register-intent` `{ email }` → durable intent + complete-signup email
2. Link: `{FRONTEND}/complete-signup?token=...`
3. `POST /api/v1/auth/complete-signup` `{ token, username, password }` → user + tokens
4. Frontend `setSession` → `/dashboard`

Alternate full register:
1. `POST /auth/register` → verify email link
2. `/verify-email?token=` → tokens → dashboard

## Live status (2026-08-16)
- `/health` → ok
- `/api/v1/*` was returning Genius 404 (likely stale process + IP allowlist or failed builds)
- New deploys queued with clearCache; need **live** status before E2E email test

## Checklist after green deploy
- [ ] `curl .../health/email` → smtp_configured true
- [ ] register-intent returns email_sent
- [ ] complete-signup returns access_token
- [ ] FE: session persists on refresh
