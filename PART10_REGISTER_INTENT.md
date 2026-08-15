# Part 10 — Register intent (email only)

## API
`POST /api/v1/auth/register-intent`
```json
{ "email": "user@example.com" }
```
Response:
```json
{ "status": "intent_sent", "email_sent": true, "message": "..." }
```

## Storage
- `data/register_intents/{token_hash}.json` on GitHub
- Token valid 30 minutes, single use

## Behaviour
- No user row until complete-signup (Part 11)
- Existing email → generic response (no enumeration)
- Full `/auth/register` still available

## Next
Part 11 — CompleteSignup page + `POST /auth/complete-signup`
