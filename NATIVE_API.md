# Genius Clan — Native Android API contract

**Base:** `https://genius-clan-api.onrender.com`  
**Prefix:** `/api/v1`  
**Auth header:** `Authorization: Bearer <access_token>`  
**WebSocket:** `wss://genius-clan-api.onrender.com/api/v1/...`

> Native apps (OkHttp / Ktor / Retrofit) **do not use CORS**. CORS only affects browsers.

Website is **not** part of this product path. Server is **API-only**.

---

## Health
| Method | Path |
|--------|------|
| GET | `/health` → plain `ok` |
| GET | `/health/store` |
| GET | `/api/v1/` → JSON route map |

## Auth
| Method | Path | Body (JSON) |
|--------|------|-------------|
| POST | `/auth/register-intent` | `{ "email": "..." }` |
| POST | `/auth/complete-signup` | `{ "token", "username", "password" }` |
| POST | `/auth/login` | `{ "identifier", "password" }` |
| POST | `/auth/login/2fa` | `{ "pending_login_id", "code" }` |
| POST | `/auth/refresh` | `{ "refresh_token" }` |
| POST | `/auth/logout` | — |
| POST | `/auth/forgot-password` | `{ "email" }` |
| POST | `/auth/reset-password` | `{ "token", "new_password" }` |
| POST | `/auth/2fa/enable` | password + codes |
| POST | `/auth/2fa/disable` | password + code |

Login response typically includes `access_token`, `refresh_token`, or `requires_2fa`.

## Profile
| Method | Path |
|--------|------|
| GET | `/profile/me` |
| PATCH | `/profile/me` | `{ "bio": "..." }` |
| POST | `/profile/me/password` |
| POST | `/profile/me/email` |
| GET | `/profile/:username` |

## Wallet / shop
| Method | Path |
|--------|------|
| GET | `/wallet/balance` |
| POST | deposit initiate (see wallet routes) |
| GET | `/shop/items` |
| POST | `/shop/purchase` |
| POST | inventory equip |

## Game (real-time)
| | |
|--|--|
| Queue | `WSS /api/v1/match/queue?token=<access>` |
| Match | `WSS /api/v1/ws/match/{id}?token=<access>` |
| REST | `GET /match/{id}` |

### Client → server (JSON text frames)
`join_queue`, `resume_match`, `move`, `resign`, `offer_draw`, `accept_draw`, `decline_draw`, `heartbeat`, `webrtc_signal`

### Server → client
`match_found`, `board_update`, `board_sync`, `match_ended`, `draw_offered`, `draw_declined`, `error`, `opponent_disconnected`, `opponent_reconnected`

---

## Android client (Phase 2+)
- Language: **Kotlin**
- UI: **Jetpack Compose**
- Network: Retrofit/Ktor + OkHttp WebSocket
- **No WebView**, no Capacitor wrap
