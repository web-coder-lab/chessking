# Phase 12 — 2-device E2E checklist (Genius Clan chess)

**App:** https://genius-clan.onrender.com  
**API:** https://genius-clan-api.onrender.com  

Use **two accounts** (or two browsers / one normal + one incognito).

---

## 0. Preflight
- [ ] FE loads (not blank) — hard refresh if needed  
- [ ] API `/health` → `ok`  
- [ ] Register / login works on both devices  
- [ ] After login, dashboard shows coins / profile  

## 1. Move authority (Phase 1)
- [ ] Board shows pieces after match starts  
- [ ] Only your turn allows selecting your pieces  
- [ ] Illegal / not-your-turn does not corrupt board  

## 2. Matchmaking (Phase 2)
- [ ] Device A: Play → **Casual** → Searching  
- [ ] Device B: Play → **Casual** → both get **Match found**  
- [ ] Opponent username appears when available  
- [ ] Single player: times out with clear message (no fake bot)  

## 3. Board UX (Phase 3)
- [ ] Clocks visible (10:00 style)  
- [ ] Check → king square red  
- [ ] Pawn to last rank → promotion picker (Q/R/B/N)  

## 4. Persist + resume (Phase 4)
- [ ] Play several moves  
- [ ] Refresh board URL / reconnect  
- [ ] Position still matches (PGN/FEN sync)  

## 5. Server clocks (Phase 5)
- [ ] After moves, clock values update from server fields  
- [ ] (Optional long test) flag at 0 → opponent wins timeout  

## 6. Draw (Phase 6)
- [ ] A: **Offer draw**  
- [ ] B: sees banner → **Decline** → continue  
- [ ] A offers again → B **Accept** → draw end screen  

## 7. Sounds (Phase 7)
- [ ] Quiet move beep  
- [ ] Capture different sound  
- [ ] Check sound  
- [ ] End-of-game sound  

## 8. Voice (Phase 8)
- [ ] Both tap 🎙️ → allow mic  
- [ ] Mute toggles real track  
- [ ] (NAT may block some networks without TURN)  

## 9. Timing anti-cheat (Phase 9)
- [ ] Normal human play: no issue  
- [ ] (Dev) ultra-fast automated spam → server logs `impossible_move_timing`  

## 10. Ranked rating (Phase 10)
- [ ] Ranked match complete → end screen shows **rating + delta**  
- [ ] Profile rating updated after refresh  
- [ ] Casual match does **not** change rating  

## 11. PGN (Phase 11)
- [ ] History → **Copy PGN** works  
- [ ] End → **Share / Copy PGN** has headers + moves  

## 12. Resign / disconnect
- [ ] Resign → opponent wins  
- [ ] Optional: disconnect banner / grace behavior  

---

## Known limits (honest)
| Item | Note |
|------|------|
| API Docker build | Free Render often OOM — need live binary with Phases 2–10 code |
| Voice | Needs both players + permissive network |
| Clocks across full server restart | In-memory unless redeployed with persist |
| 1 player | Cannot complete online match alone |

## Smoke (automated 2026-08-24)
- FE HTTP 200  
- API health 200  
- GitHub store ok  
- Login token ok  
