# Genius Clan — 5-Phase Plan

| Phase | Name | Status |
|-------|------|--------|
| 1 | Deploy (Render free tier) | DONE |
| 2 | All-to-all pages check | DONE |
| 3 | Database attach | **SUPERSEDED** |
| 3′ | **Zero data on Render + private DB repo + Firebase** | **DONE (policy + infra)** |
| 4 | Firewalls attach | Pending |
| 5 | Name change / branding | Partial |

## Data policy (mandatory)

**Render pe koi bhi durable app data nahi.**

| What | Where |
|------|--------|
| API + static site | Render (compute only) |
| User / match / wallet data | **Firebase Firestore** (live) |
| Schema SQL archive | Private repo only |

### Private database repo
https://github.com/web-coder-lab/genius-clan-database (**private**)

Contains:
- `schema/migrations/*` (legacy SQL reference)
- `POLICY.md` — zero data on Render
- Firebase collection map

### Render Postgres
**Deleted.** No Render DB instances for Genius Clan.

### Env on API
- `DATABASE_URL` cleared (no local SQLite file store on Render)
- Next: `FIREBASE_*` service account env vars when Firestore is wired

### Code note
Backend still has SQLite sqlx code paths. Until Firestore is wired, API must not rely on Render-local files. Firestore migration is the next engineering step after Phase 4 or on request.
