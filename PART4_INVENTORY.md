# Part 4 — Durable inventory + equip

## Storage
- `data/inventory/{user_id}.json` — items, avatar_id, banner_id
- Also mirrors avatar_id/banner_id onto GitHub user record

## Behaviour
- List inventory → hydrate SQL from GitHub if empty (post-restart)
- Equip / unequip / purchase / register defaults → sync_from_sql → GitHub
- UI still needs avatar_id display fix (later theme/UI parts)

## Next
Part 5 — IP_ALLOWLIST off (public API)
