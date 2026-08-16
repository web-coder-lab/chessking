# Part 25 — Admin update paths (API)

All routes need `Authorization: Bearer <admin_access_token>` and an account with the right role (`super_admin` / section role).

## Support email
```bash
# Read
curl -H "Authorization: Bearer $TOK" \
  https://genius-clan-api.onrender.com/api/v1/admin/content/support_email

# Update
curl -X POST -H "Authorization: Bearer $TOK" -H "Content-Type: application/json" \
  https://genius-clan-api.onrender.com/api/v1/admin/content \
  -d '{"key":"support_email","content":"workn8312@gmail.com"}'
```
Also valid keys: `privacy_policy`, `terms_of_service`, `about`.

## Shop items
```bash
# Create
curl -X POST -H "Authorization: Bearer $TOK" -H "Content-Type: application/json" \
  https://genius-clan-api.onrender.com/api/v1/admin/shop/items \
  -d '{"category":"avatar","name":"...","price_coins":50,"icon_emoji":"👑"}'

# Update / deactivate
curl -X PATCH .../admin/shop/items/{id} -d '{...}'
curl -X POST  .../admin/shop/items/{id}/deactivate
```

## SMTP test
```bash
curl -X POST -H "Authorization: Bearer $TOK" -H "Content-Type: application/json" \
  https://genius-clan-api.onrender.com/api/v1/admin/config/smtp/test \
  -d '{"to":"you@example.com"}'
```

## Promote a user to admin (SQL / future tool)
Role is on `users.role`. First admin is operational bootstrap (env/SQL) — not self-serve from public UI this phase.

## Public app
Support page reads `GET /api/v1/support/info` → shows seeded `workn8312@gmail.com`.
