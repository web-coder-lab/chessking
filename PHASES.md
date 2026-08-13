# Genius Clan — 5-Phase Plan

| Phase | Name | Status |
|-------|------|--------|
| **1** | **Deploy** (Render free tier, name `genius-clan`) | **IN PROGRESS** |
| 2 | All-to-all pages check | Pending |
| 3 | Database attach (persistent) | Pending |
| 4 | Firewalls attach | Pending |
| 5 | Name change / branding / extras | Pending |

## Phase 1 — Deploy

**Services (free plan):**
- `genius-clan-api` — Rust API (Docker)
- `genius-clan` — React static site

**Files added:**
- `Dockerfile` (repo root)
- `render.yaml` (Blueprint)
- CORS on API for browser clients

**You must provide:** Render API token (or Blueprint connect via GitHub)

**Limits (free tier):**
- Services spin down after idle ~15 min
- SQLite on ephemeral disk = data lost on restart (fixed in Phase 3)
- Cold start can take 30–60s

## After Phase 1 goes live

1. Set `FRONTEND_BASE_URL` on API to the static site URL  
2. Set `VITE_API_BASE` / `VITE_WS_BASE` on static site to API URL  
3. Redeploy frontend once  

Then → Phase 2 (pages check).
