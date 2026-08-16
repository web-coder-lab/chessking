# Part 35 — Deploy result check (2026-08-16)

## Render free Docker
| Commit | Result |
|--------|--------|
| Part 31–34 | **build_failed** (OOM / resource) |
| Live process | `/health` + `/health/store` only; `/api/v1` → 404 |

## Bypass: GHCR via GitHub Actions
Workflow: `.github/workflows/build-api-image.yml`  
Image: `ghcr.io/web-coder-lab/genius-clan-api`

| Run | Status |
|-----|--------|
| Part 35 first image build | **failure** |
| Part 39 E0382 fix | **in_progress** (cargo compile on Actions) |

## After image is on GHCR
1. Make package public **or** add Render deploy key for private pull
2. Point Render service to Docker image `ghcr.io/web-coder-lab/genius-clan-api:latest` (not Dockerfile build)
3. Confirm:
```bash
curl https://genius-clan-api.onrender.com/api/v1/auth/login -H 'Content-Type: application/json' -d '{"identifier":"x","password":"y"}'
# JSON error, not HTML 404
```
