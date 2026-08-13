# Genius Clan — Final verification

## Database (GitHub private repo) — COMPLETE

**Repo:** https://github.com/web-coder-lab/genius-clan-database (private)

### Collections present
users, sessions, email_tokens, two_fa_pending, matches, match_moves,
custom_match_invites, shop_items (10 seeded), inventory, gifts,
notifications, security_events, risk_scores, referrals, ad_views,
daily_rewards, bug_reports, wallet_logs, coin_packages (3 catalog),
legal (3), config (4), indexes (13+), _payment_deferred (unused)

### Payment
**Deferred.** No gateway deposits/card data in this DB. Separate secure system later.

### Render
- No Postgres, no disk
- DATABASE_URL=sqlite::memory:
- Durable data only via GITHUB_DATA_* → private repo

### App code gap
`GitHubStore` ready; most handlers still use in-memory sqlx until migrated.

### Live
- https://genius-clan.onrender.com
- https://genius-clan-api.onrender.com/health

Phase 4 (firewalls) starts on request.
