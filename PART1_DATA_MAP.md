# Part 1 — Data map (Genius Clan)

**Date:** 2026-08-14  
**Goal:** Exact picture of what survives a Render restart / sleep.

## Storage layers

| Layer | Tech | Survives restart? |
|-------|------|-------------------|
| **L0** | Render free process | No — sleep kills process |
| **L1** | SQLite `sqlite::memory:` (or ephemeral disk) | **No** |
| **L2** | GitHub private repo `genius-clan-database` via Contents API | **Yes** |
| **L3** | Browser localStorage + cookie (`ck_refresh_token`) | Client only — useless if server lost session |

---

## Tables (SQL schema) vs durability

| Domain | Tables / data | Today | Needed (Parts 2–4) |
|--------|---------------|-------|---------------------|
| **Users auth** | `users` password_hash, email, role | SQL + **partial GitHub** (`github_users`) | GitHub = source of truth |
| **Sessions** | `sessions` refresh_token_hash | **SQL only** | **GitHub** — login sticky |
| **Email tokens** | `email_verification_tokens`, `password_reset_tokens` | SQL only | GitHub or short TTL OK if email works |
| **2FA pending** | `two_fa_pending_verifications` | SQL only | GitHub or memory+short TTL |
| **Wallet** | `users.coin_balance`, `wallet_logs` | **SQL only** | **GitHub** |
| **Payments** | `payment_transactions` | SQL only | Deferred (Phase payment) |
| **Shop catalog** | `shop_items`, `coin_packages` | SQL seed on boot | SQL seed OK *or* GitHub catalog |
| **Inventory / equip** | `inventory`, `users.avatar_id/banner_id` | **SQL only** | **GitHub** |
| **Matches** | `matches`, hints, disconnect | SQL / memory registry | Later — in-match can stay RAM |
| **Social** | referrals, gifts | SQL | Later |
| **Daily / ads** | `daily_rewards`, `ad_views` | **SQL only** | **GitHub** |
| **Leaderboard** | snapshot / live rating | SQL | Derived from durable users |
| **Notifications** | notifications, settings | SQL | Later |
| **Security** | risk_scores, security_events, bans | SQL | Events can be RAM; bans → durable |
| **Admin / config** | app_config, static_pages, audit | SQL | support_email → durable |
| **Captcha answers** | app_config keys | SQL | RAM OK (5 min TTL) |

---

## GitHub store today (`GitHubStore`)

**Repo:** `web-coder-lab/genius-clan-database` (env override)

**API:**
- `get_json(collection, id)` → `data/{collection}/{id}.json`
- `put_json(...)` with SHA optimistic lock
- `list_ids(collection)`
- indexes: `data/indexes/{name}.json`

**Used by code:**
- `auth/github_users.rs` — save/get user, username/email indexes
- `auth/register.rs` — dual-write user on register
- `auth/login.rs` — fallback lookup if SQL miss
- `GET /health/store` — connectivity probe

**NOT used yet:** sessions, wallet, daily, inventory, shop writes.

---

## Why user-visible bugs happen

| Bug | Mechanism |
|-----|-----------|
| Login → refresh → login again | Session only in L1; process sleep → refresh token unknown |
| Coins claim → refresh → 0 + claim again | `daily_rewards` + balance only L1 |
| Equip → looks OK → reload default | `inventory` / avatar_id only L1; UI also ignores `avatar_id` |
| Email not received | SMTP path / env / silent fail (Parts 7–12) |
| API 404 on normal use | `IP_ALLOWLIST` locked to one IP (Part 5) |

---

## Target architecture (after Parts 2–4)

```
Client (cookie + localStorage refresh)
    → API
        → GitHub: users, sessions, wallet, daily, inventory, indexes
        → Memory SQL: captcha, live match state, rate counters, optional cache
```

**Rule:** Anything user notices after “refresh the page” must live on **L2 GitHub** (or later Postgres).

---

## Part 1 deliverable checklist

- [x] All CREATE TABLE domains listed
- [x] GitHub vs SQL ownership marked
- [x] Bug ↔ layer mapping
- [x] Target architecture written

## Next

**Part 2:** Durable sessions (refresh tokens on GitHub)  
**Part 3:** Durable wallet + daily  
**Part 4:** Durable inventory + equip  
**Part 5:** IP_ALLOWLIST off  
**Part 6:** Env verify  

