# Part 38 — GHCR still private

## Status
- Image exists (Actions **success** on Part 39)
- Pull without auth → **401** (package private)
- PAT cannot change package visibility (403)
- New Actions run (Part 37 workflow) still **in_progress** (will retry public API)

## You must (1 minute)
1. Open GitHub → profile → **Packages** → **genius-clan-api**  
   Direct pattern: `https://github.com/users/web-coder-lab/packages/container/genius-clan-api`
2. **Package settings** → Danger zone / visibility → **Public**
3. Render dashboard → **genius-clan-api** → Settings → Build & Deploy:
   - Use **published image**
   - Image URL: `ghcr.io/web-coder-lab/genius-clan-api:latest`
4. Deploy

## Verify
```bash
curl -s https://genius-clan-api.onrender.com/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"identifier":"x","password":"y"}'
```
Expect JSON body, not Genius HTML 404.
