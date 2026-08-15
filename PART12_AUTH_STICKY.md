# Part 12 — Login sticky polish

## Client
- Refresh token: **localStorage + cookie** (30 days)
- Silent refresh on app open
- `bootstrapping` — no flash to login while restoring
- Network blip on refresh → **retry**, not logout
- Logout only on 401 / token reuse / explicit logout
- Access refresh interval aligned closer to 15m server TTL

## Server (Part 2)
- Sessions dual-written to GitHub; `/auth/refresh` recovers after Render sleep
