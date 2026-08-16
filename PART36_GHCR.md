# Part 36 — GH Actions image build (in progress)

## Status
Workflow **Build API image** was `in_progress` on GitHub Actions (not instant-fail — good sign vs Render OOM).

Image (when green):
`ghcr.io/web-coder-lab/genius-clan-api:latest`

## Render: switch to image deploy
Dashboard → **genius-clan-api** → Settings → Build & Deploy:

1. Runtime: **Docker**
2. Image URL: `ghcr.io/web-coder-lab/genius-clan-api:latest`
3. If private package: add registry credential (GitHub PAT with `read:packages`)
4. Prefer **public** package: GitHub → Packages → genius-clan-api → Package settings → Change visibility → Public

Keep existing env vars (JWT, SMTP, GITHUB_DATA_*, FRONTEND_BASE_URL).

## Verify after switch
```bash
curl https://genius-clan-api.onrender.com/health
curl -X POST https://genius-clan-api.onrender.com/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"identifier":"x","password":"y"}'
# expect JSON body, not Genius HTML 404
```
