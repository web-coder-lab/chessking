# Genius Clan — 5-Phase Plan

| Phase | Name | Status |
|-------|------|--------|
| 1 | Deploy | DONE |
| 2 | Pages check | DONE |
| 3 | GitHub JSON database | DONE |
| 4 | Firewalls / security | DONE |
| **5** | **Branding + durable auth (GitHub users)** | **DONE** |

## Phase 5 notes
- Branding: Chess King → Genius Clan (email, invite share)
- Auth register dual-writes user to private GitHub DB + indexes
- Auth login falls back to GitHub user store and hydrates memory SQL
- Free tier: email_verified=true on GitHub users when SMTP unset
- Register frontend accepts `verify_email_sent` status
