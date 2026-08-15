# Part 11 — Complete signup

## API
`POST /api/v1/auth/complete-signup`
```json
{ "token": "...", "username": "...", "password": "...", "device_fingerprint": "..." }
```
→ `{ "access_token", "refresh_token" }` (auto login)

## Frontend
Route: `/complete-signup?token=...`
Page: username + password → session → `/dashboard`

## Flow end-to-end
1. `POST /auth/register-intent` { email }
2. Email link → `/complete-signup?token=`
3. `POST /auth/complete-signup` → durable user + tokens
