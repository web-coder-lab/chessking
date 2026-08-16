# Part 37 — GH Actions **SUCCESS**

## Image
`ghcr.io/web-coder-lab/genius-clan-api:latest`  
(SHA from Part 39 E0382 fix — build completed **success**)

## Blocker for auto-pull
GHCR package is **private** (auth required). Render free needs either:
1. Package **Public** (GitHub → Packages → genius-clan-api → Package settings → Change visibility), or  
2. Registry credential on Render for `ghcr.io`

## Dashboard steps (required once)
1. Open https://github.com/users/web-coder-lab/packages/container/package/genius-clan-api  
   (or repo → Packages)
2. Set visibility **Public**
3. Render → genius-clan-api → Settings → Build & Deploy  
   - Deploy style: **Docker image**  
   - Image: `ghcr.io/web-coder-lab/genius-clan-api:latest`  
   - Clear Dockerfile path if asked
4. Manual deploy → wait for live
5. Test: `curl -X POST .../api/v1/auth/login` → JSON not HTML

## Workflow
Next image builds will attempt to set package public via API.
