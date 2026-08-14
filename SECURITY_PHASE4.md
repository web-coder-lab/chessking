# Genius Clan — Security Phase 4: CAPTCHA step-up

**Status:** Implemented (chess-themed CAPTCHA, no third-party vendor).

## Policy (Doc 8 §14 + Phase 4)

- After **3 failed logins** in 15 minutes → login must include captcha solution
- After **5 failures** → existing account lockout still applies
- Challenge is **chess-themed** (tap knight / move king / which side in check)
- Answers stored server-side only (ephemeral `app_config` keys), never trusted from client alone

## API

1. User posts login without captcha after ≥3 fails  
2. Response `200` with `{ requires_captcha: true, captcha: { challenge_id, kind, board_fen, prompt } }`  
3. User posts again with `captcha_challenge_id` + `captcha_answer`  
4. Wrong answer → `403 captcha_required`  
5. Correct → normal credential check continues  

Endpoints (already existed):
- `POST /api/v1/captcha/generate`
- `POST /api/v1/captcha/verify`

## Frontend

`LoginForm` shows prompt + answer field when `requires_captcha` is returned.

## Note on durability

Captcha answers live in in-memory SQLite on Render. Process restart clears challenges (user just gets a new one). Acceptable for step-up UX.

## Optional later

Cloudflare Turnstile as secondary when domain is on Cloudflare (Phase 2).
