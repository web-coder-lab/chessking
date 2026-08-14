# Part 3 — Durable wallet + daily claim

## Storage
- User `coin_balance` on GitHub `data/users/{id}.json` (via github_wallet::sync_balance)
- Daily claims: `data/daily_rewards/{user_id}.json`

## Behaviour
- Claim → SQL ledger + GitHub balance + GitHub daily file
- Status → SQL or GitHub claimed_today
- GET /wallet/balance → hydrate from GitHub if SQL differs (post-restart)

## Next
Part 4 — durable inventory / equip
