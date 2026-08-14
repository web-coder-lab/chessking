# Part 6 — Env verify

## API (Render) — all present
| Key | Status |
|-----|--------|
| JWT_SECRET | OK |
| FRONTEND_BASE_URL | https://genius-clan.onrender.com |
| GITHUB_DATA_* | OK (token, owner, repo, branch) |
| SMTP_* | OK (gmail 587) |
| IP_ALLOWLIST | empty (public) |
| DATABASE_URL | sqlite::memory: |

## Frontend
| VITE_API_BASE | https://genius-clan-api.onrender.com |
| VITE_WS_BASE | wss://genius-clan-api.onrender.com |

## Live probes
- /health → ok
- /health/store → github reachable

## Note
Parts 2–4 deploys reported build_failed on Render (fast fail). Dockerfile pinned to rust:1.83-bookworm for stable builds; re-deploy after this commit.
