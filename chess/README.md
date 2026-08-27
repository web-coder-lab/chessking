# chess/ — durable database (GitHub JSON)

**Database ID:** `dstabase7837638362826373`

Render API process holds **no permanent user data**.  
All durable rows are JSON files under this folder:

```
chess/
  meta.json
  users/{user_id}.json
  sessions/{session_id}.json
  sessions_by_hash/{hash}.json
  wallet/{user_id}.json
  inventory/{user_id}.json
  register_intents/{id}.json
  matches/{match_id}.json
  indexes/*.json
```

API uses GitHub Contents API with path prefix **`chess/`**.

Env (Render):
- `GITHUB_DATA_TOKEN` — PAT with `repo` scope
- `GITHUB_DATA_OWNER` — e.g. `web-coder-lab`
- `GITHUB_DATA_REPO` — e.g. `chessking`
- `GITHUB_DATA_BRANCH` — `main`
- `GITHUB_DATA_ROOT` — `chess` (default)
