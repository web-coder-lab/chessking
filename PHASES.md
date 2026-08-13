# Genius Clan — 5-Phase Plan

| Phase | Name | Status |
|-------|------|--------|
| 1 | Deploy (Render free tier) | DONE |
| 2 | All-to-all pages check | DONE |
| **3** | **Database attach** | **DONE (infra) — code still SQLite** |
| 4 | Firewalls attach | Pending |
| 5 | Name change / branding / extras | Partial |

## Live URLs
- Web: https://genius-clan.onrender.com
- API: https://genius-clan-api.onrender.com
- DB dashboard: https://dashboard.render.com/d/dpg-d9uptvegekts73d1lnsg-a

## Phase 3 — Database attach

### What was attached
| Resource | Details |
|----------|---------|
| **Render Postgres** | `genius-clan-db` |
| Plan | **Free** (expires ~30 days from create — renew/upgrade before then) |
| Region | oregon |
| Version | 16 |
| Status | **available** |
| DB name | `genius_clan_db` |
| User | `genius_clan_db_user` |

### Connection
- **Internal** (from API service on Render): set as env `DATABASE_URL_POSTGRES`
- **External**: available in Render dashboard → Connection info

### Why app still uses SQLite
Backend is built on **sqlx + SQLite** (~200 references).  
Switching live traffic to Postgres needs a full dialect + type migration (`SqlitePool` → `PgPool`, migration SQL, datetime, etc.).

**Persistent disk** for SQLite needs a **paid** web instance (free cannot attach disks).

### Current data behaviour
- `DATABASE_URL=sqlite:///data/genius_clan.db` (ephemeral on free web)
- Redeploy / sleep → local SQLite resets
- Postgres is **ready** for Phase 3b (code migration)

### Phase 3b (optional follow-up)
1. sqlx features → `postgres`
2. Replace pool types across backend
3. Port `database/migrations/*.sql` to Postgres
4. Point `DATABASE_URL` to internal Postgres URL
5. Redeploy API

### Security note
Rotate DB password in dashboard if this chat is shared; connection strings appeared in deploy tooling.
