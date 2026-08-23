# Phase 2 — Matchmaking solid

## Backend
- Wider ranked band (200 + 25/s) + **mutual** band check
- **Casual sweep** every 2s (pair waiting players)
- Zombie / self-duplicate queue cleanup
- Non-initiator waits up to ~10s for DB + **in-memory registry**
- `match_found` includes **opponent_username**

## Frontend
- Search timeout / cancel (Phase 1–2)
- Heartbeat while searching
- Found screen shows opponent name when provided

## How to test (2 devices)
1. Login account A → Play → Casual
2. Login account B → Play → Casual
3. Both should get Match found → board

Single player will correctly time out (no fake AI opponent in this phase).
