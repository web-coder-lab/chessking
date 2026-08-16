# Part 39 — API **LIVE**

## Confirmed
| Endpoint | Result |
|----------|--------|
| `GET /health` | 200 ok |
| `GET /health/email` | `smtp_configured: true` |
| `GET /health/store` | github ok |
| `POST /auth/login` (bad) | JSON `invalid_credentials` |
| `POST /auth/register` | 200 `verify_email_pending` (email_sent may be false if SMTP glitch) |
| `POST /auth/login` (new user) | **access_token + refresh_token** |
| `GET /.env` | Genius 404 |
| `POST /auth/register-intent` | **timeout** (GitHub intent write slow / hang — investigate Part 40) |

## Meaning
Render is serving a binary that includes `/api/v1` again. Core login/register path works.

## Follow-ups
1. register-intent timeout
2. SMTP email_sent false on register (configured true but send failed once)
3. GHCR still private (optional for future deploys)
