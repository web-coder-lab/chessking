# Genius Clan — Mobile API (API-only server)

**Base URL:** `https://genius-clan-api.onrender.com`  
**Prefix:** `/api/v1`  
**Auth:** `Authorization: Bearer <access_token>`  
**WebSocket:** `wss://genius-clan-api.onrender.com/api/v1/...`

This host serves **JSON + WebSocket only**. No website HTML.

---

## Health
| Method | Path | Auth |
|--------|------|------|
| GET | `/health` | no |
| GET | `/health/store` | no |
| GET | `/health/email` | no |
| GET | `/api/v1/` | no — API index |

## Auth
| Method | Path | Auth |
|--------|------|------|
| POST | `/api/v1/auth/register-intent` | no |
| POST | `/api/v1/auth/complete-signup` | no |
| POST | `/api/v1/auth/login` | no |
| POST | `/api/v1/auth/refresh` | no |
| POST | `/api/v1/auth/forgot-password` | no |
| POST | `/api/v1/auth/reset-password` | no |
| POST | `/api/v1/auth/logout` | yes |
| GET | `/api/v1/auth/sessions` | yes |

## Profile / social
| Method | Path | Auth |
|--------|------|------|
| GET | `/api/v1/profile/me` | yes |
| PATCH | `/api/v1/profile/me` | yes |
| GET | `/api/v1/users/:username` | yes |

## Wallet / shop
| Method | Path | Auth |
|--------|------|------|
| GET | `/api/v1/wallet/balance` | yes |
| POST | `/api/v1/wallet/claim` (or rewards daily) | yes |
| GET | `/api/v1/shop/items` | yes |
| POST | `/api/v1/shop/purchase` | yes |
| POST | `/api/v1/inventory/equip` | yes |

## Game
| Method | Path | Auth |
|--------|------|------|
| WS | `/api/v1/match/queue?token=` | token query |
| WS | `/api/v1/ws/match/:id?token=` | token query |
| GET | `/api/v1/match/:match_id` | yes |
| POST | `/api/v1/match/:match_id/hint` | yes |
| POST | `/api/v1/custom-match/invite` | yes |

### WS client messages
`join_queue`, `resume_match`, `move`, `resign`, `offer_draw`, `accept_draw`, `decline_draw`, `heartbeat`, `webrtc_signal`

### WS server messages
`match_found`, `board_update`, `board_sync`, `match_ended`, `draw_offered`, `draw_declined`, `error`, `opponent_disconnected`, `opponent_reconnected`

## Capacitor / APK
```js
// production
VITE_API_BASE=https://genius-clan-api.onrender.com
VITE_WS_BASE=wss://genius-clan-api.onrender.com
```

CORS allows: web SPA + `capacitor://localhost` + `https://localhost`.

## Frontend removal
`render.yaml` no longer deploys static site with the API.  
Optional web: deploy `frontend/` separately or use APK only.
