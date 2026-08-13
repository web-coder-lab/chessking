# Genius Clan — Phase 4 Firewalls / Security

## Network (Render free)

| Control | Setting |
|---------|---------|
| Public HTTPS | Render managed TLS |
| IP allow-list | `0.0.0.0/0` — **required** for a public chess API (players worldwide) |
| Health | `/health` unauthenticated, not rate-limited |

Locking IP allow-list to a single country/CIDR would break normal users. App-layer controls are the real firewall.

## Application firewall (API)

| Control | Implementation |
|---------|----------------|
| CORS | Allow-list only: `FRONTEND_BASE_URL`, localhost:5173, `https://genius-clan.onrender.com` |
| Rate limit | ~3 req/s / IP, burst 15 (`SmartIpKeyExtractor` + X-Forwarded-For) |
| JWT gate | Every protected route re-validates Bearer access token |
| Admin roles | `require_role` on admin routes |
| Security headers | `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy`, `Permissions-Policy` |
| Durable secrets | `JWT_SECRET`, `GITHUB_DATA_TOKEN` in Render env only — not in git |
| Data plane | No DB files on Render; private GitHub repo only |

## Payment

Out of scope for this phase. Future payment stack must be isolated (separate secrets, no card data in GitHub JSON DB).

## Operator checklist

1. Rotate GitHub PAT → fine-grained, only `genius-clan-database` Contents
2. Rotate `JWT_SECRET` if ever leaked
3. Keep `FRONTEND_BASE_URL` exact match to live static URL
4. Do not commit `.env` or PATs
