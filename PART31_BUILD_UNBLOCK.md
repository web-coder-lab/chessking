# Part 31 — Build unblock

## Diagnosis
- Service is Docker, `dockerfilePath: ./Dockerfile`, context `.`
- Recent deploys: **`build_failed`** (often ~30–60s → likely **OOM** / killed on free tier, not a 10‑min compile)
- `/health` still serves old process; `/api/v1/*` → Genius 404

## Mitigations applied
1. Dockerfile: `CARGO_BUILD_JOBS=1`, `cargo build -j 1`
2. `pkg-config` + `libssl-dev` for native crates
3. `strip` binary after build
4. Cargo.lock copied explicitly

## If still failing
- Upgrade Render plan for more build RAM, **or**
- Build binary in CI → ship minimal runtime image, **or**
- Temporary: run API elsewhere (Fly/Railway) with more RAM

## Re-test after live
```bash
curl https://genius-clan-api.onrender.com/health
curl -X POST https://genius-clan-api.onrender.com/api/v1/auth/login \
  -H 'Content-Type: application/json' -d '{"identifier":"x","password":"y"}'
# expect JSON, not HTML 404
```
