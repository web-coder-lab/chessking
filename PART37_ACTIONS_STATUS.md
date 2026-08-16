# Part 37 — Actions status

## Build API image
| Step | Status |
|------|--------|
| Checkout | success |
| Login GHCR | success |
| **Build and push** | **in_progress** (Rust compile on GH runner) |

Not failed — compile simply takes time (10–25+ min).

## When conclusion = success
1. Open https://github.com/web-coder-lab/chessking/pkgs/container/genius-clan-api  
2. Set visibility **Public** (Settings)  
3. Render → genius-clan-api → deploy from  
   `ghcr.io/web-coder-lab/genius-clan-api:latest`  
4. Smoke:
   ```bash
   curl -X POST https://genius-clan-api.onrender.com/api/v1/auth/login \
     -H 'Content-Type: application/json' \
     -d '{"identifier":"x","password":"y"}'
   ```

## Current API host
Still old binary (`/api/v1` → 404 HTML).
