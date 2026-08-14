# Genius Clan — Security Phase 6: Monitoring & operator checklist

## Structured log target: `security`

Filter Render logs with:
```text
target:security
```
or search event names below.

| Event | Level | When |
|-------|--------|------|
| `probe_blocked` | warn | Path scanner hit (Phase 3) |
| `captcha_required` | info | Login step-up after 3 fails |
| `captcha_failed` | warn | Wrong captcha answer |
| `account_lockout` | warn | 5+ fails / 15 min |
| `auth_rate_limited` | warn | (governor / velocity) |
| `github_store_error` | error | Durable DB API failure |
| `hourly_summary` | info | Every 15 min: counts last 1h |
| `high_severity_alert` | warn | severity ≥ 8 events in last 1h |

## Periodic job

`anticheat::monitor::spawn_periodic_security_summary` — every **15 minutes** emits `hourly_summary` from ephemeral `security_events` (this process only; resets on redeploy).

## Admin API (already existed)

- `GET /api/v1/admin/security/events/:user_id` — per-user timeline  
- Anticheat dashboard risk tiers / pending review  

Requires admin JWT + role.

## Operator checklist (weekly)

- [ ] Render API logs: any `high_severity_alert` or `github_store_error`?
- [ ] Spike in `login_failed` / `probe_blocked`?
- [ ] GitHub PAT still valid; rotate if shared
- [ ] SMTP still sending (test register email)
- [ ] Cloudflare (if domain): Bot Fight + probe rules still on
- [ ] No secrets committed in `chessking` repo
- [ ] Payment still deferred — no card data in GitHub JSON DB

## Alerting (free tier)

Render free has no native PagerDuty. Practical options:
1. Manually check logs after `hourly_summary`
2. Later: webhook on high_severity (Phase 6.1) to Discord/email
3. Upgrade: external log drain (Better Stack, Axiom, etc.)

## Limitation

In-memory SQLite → security_events **do not survive** API restart. Durable user rows are on GitHub; event history is process-local unless migrated later.
