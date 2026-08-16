# Part 27 — Claim / coins verify (code)

## Server path
1. `GET /api/v1/rewards/daily` → `claimed_today` from SQL **or GitHub** (`github_wallet::claimed_today`)
2. `POST /api/v1/rewards/daily/claim` → SQL ledger + `record_claim` + `sync_balance` on GitHub
3. `GET /api/v1/wallet/balance` → hydrate `coin_balance` from GitHub if SQL empty

## Client path
- Claim → toast + `refreshUser` + re-fetch daily status
- Tab visibility → re-sync balance/claim

## Live
Blocked until API deploy is **live** (recent builds were `build_failed`).
Dockerfile simplified (single `cargo build --release`, Rust 1.97).
