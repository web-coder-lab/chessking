# Part 36 — GHCR image deploy (waiting on build)

## Blocker
No GHCR package yet (`genius-clan-api` 404) until Actions succeeds.

## Compile error (fixed in Part 39)
`rewards.rs` E0382: match moved `date` then `last.is_none()`.
Fix: `match &last` + `date == &today`.

## Current Actions
`Build API image` on Part 39 commit — **in_progress** (~7+ min cargo).

## When success
1. Package appears: `ghcr.io/web-coder-lab/genius-clan-api:latest`
2. Set package visibility / Render pull credentials
3. Switch Render service from Dockerfile → **Docker image** URL
4. Verify `/api/v1/auth/login` returns JSON

## Render image settings (dashboard)
- Runtime: Docker
- Image: `ghcr.io/web-coder-lab/genius-clan-api:latest`
- Do **not** use “build from Dockerfile” on free tier
