# Phase 3 — Auth, Profile, Wallet (Compose)

| Screen | API |
|--------|-----|
| Login / Register | `POST /auth/login`, `POST /auth/register` |
| Email intent | `POST /auth/register-intent` |
| Profile | `GET/PATCH /profile/me` |
| Wallet | `GET /wallet/balance` (+ profile fallback) |

Logout clears tokens and returns to Auth.
