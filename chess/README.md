# chess/ — database `dstabase7837638362826373`

**Totally prepared durable database for Genius Clan.**

| Field | Value |
|-------|--------|
| Database ID | `dstabase7837638362826373` |
| Root folder | `chess/` |
| Repo | `web-coder-lab/chessking` |
| Engine | GitHub Contents API (JSON files) |

## Layout

```
chess/
  meta.json
  indexes/
    users_by_email.json
    users_by_username.json
    sessions_by_hash.json
    register_intents_by_email.json
  users/
  sessions/
  daily_rewards/
  inventory/
  register_intents/
  wallet/
  matches/
  _schema/          ← templates only (not live data)
```

## API connection (Render)

```
DATABASE_URL=sqlite::memory:
GITHUB_DATA_OWNER=web-coder-lab
GITHUB_DATA_REPO=chessking
GITHUB_DATA_BRANCH=main
GITHUB_DATA_ROOT=chess
GITHUB_DATA_TOKEN=<PAT with repo scope>
```

Live rows are written by the API as:
`chess/{collection}/{id}.json`

Do not put secrets in this folder. Do not edit other repo folders for DB.
