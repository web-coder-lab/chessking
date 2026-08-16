# Part 35 — Part 34 also **build_failed**

Render free Docker build cannot finish this Rust API (OOM / kill).

## Pivot
GitHub Actions workflow: `.github/workflows/build-api-image.yml`
- Builds on `ubuntu-latest` (more RAM)
- Pushes `ghcr.io/web-coder-lab/genius-clan-api:latest`

## Next steps for you / Part 36
1. After workflow succeeds, in Render Dashboard → genius-clan-api → **Settings**:
   - Switch deploy to **Docker image**: `ghcr.io/web-coder-lab/genius-clan-api:latest`
   - Or keep Dockerfile but we will try API image deploy via Render API
2. Ensure package is **public** or add GHCR pull credentials on Render

## Live (unchanged)
- `/health` ok — old process
- `/api/v1` → 404
