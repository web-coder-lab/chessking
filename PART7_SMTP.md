# Part 7 — SMTP hard-check

## Changes
1. `SMTP_PASS` spaces stripped (Gmail app password format)
2. `EmailClient::is_configured()` 
3. `GET /health/email` → `{ "smtp_configured": true/false }`
4. Dockerfile Rust **1.97** (edition2024 deps)
5. Startup already logs `SMTP client configured` when ready

## Render
- SMTP_HOST=smtp.gmail.com
- SMTP_PORT=587
- SMTP_USER=newgenerationbox506@gmail.com
- SMTP_PASS=(16-char app password, no spaces)

## Verify after deploy
```bash
curl https://genius-clan-api.onrender.com/health/email
# expect smtp_configured: true
```

## Next
Part 8 — Email send errors not silent; register flow uses real send status
