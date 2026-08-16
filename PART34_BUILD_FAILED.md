# Part 34 — Part 31 result: **build_failed**

## Live
- `/health` + `/health/store` → 200 (old binary still running)
- `/api/v1/*` → Genius 404

## Actions this part
1. Release profile: no LTO, opt-level 2, strip, no incremental
2. Dockerfile: `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`
3. Still on Rust 1.97 (edition2024 crates)

## If this deploy also fails
Render free build RAM cannot compile this Rust API reliably.
**Options:** paid build, CI-built image (GHCR), or smaller host with more RAM.
