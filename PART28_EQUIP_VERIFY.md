# Part 28 — Equip / inventory verify (code)

## Server
| Endpoint | Behaviour |
|----------|-----------|
| `GET /inventory` | hydrate SQL from GitHub if empty |
| `POST /inventory/:id/equip` | SQL equip + `sync_from_sql` → GitHub |
| `POST /inventory/:id/unequip` | same sync |
| Purchase / register defaults | sync inventory + balance |

Storage: `data/inventory/{user_id}.json` (+ avatar_id/banner_id on user)

## Client
| Action | Behaviour |
|--------|-----------|
| Equip | API → local equipped flag → `refreshUser()` |
| TopBar / Profile | `avatar_id` / `banner_id` → emoji via `utils/avatar.js` |

## Live checklist (after API live)
- [ ] Equip item → gold border on card
- [ ] TopBar emoji changes without full logout
- [ ] Soft refresh → same equipped item still marked
