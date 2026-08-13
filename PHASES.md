# Genius Clan — 5-Phase Plan

| Phase | Name | Status |
|-------|------|--------|
| 1 | Deploy (Render free tier, `genius-clan`) | Done (API + frontend live) |
| **2** | **All-to-all pages check** | **DONE** |
| 3 | Database attach (persistent) | Pending |
| 4 | Firewalls attach | Pending |
| 5 | Name change / branding / extras | Partial |

## Phase 2 report — Pages (no Docker)

### Routes registered (App.jsx)
All 26 page components have matching routes. Catch-all `*` → `/auth`.

### Bottom nav (5 tabs)
| Tab | Path | Page has BottomNav |
|-----|------|-------------------|
| Home | /dashboard | Yes |
| Wallet | /wallet | Yes |
| Play | /play | Yes |
| Leaderboard | /leaderboard | Yes |
| Profile | /profile | Yes |

### Page map + navigation integrity

| Page | Route | Key actions | Status |
|------|-------|-------------|--------|
| Splash | / | Language → /auth | OK (title → Genius Clan) |
| Auth | /auth | Login / Register / Forgot | OK |
| Reset password | /reset-password | Token + new password | OK |
| Dashboard | /dashboard | Claim reward, Play, shortcuts | OK |
| Wallet | /wallet | Packages → checkout, history | OK |
| Checkout | /wallet/checkout | Gateways + poll status | OK |
| Shop | /shop | Buy items | OK |
| Inventory | /inventory | Equip (+ ?category=avatar) | OK |
| Play | /play | Queue + custom match link | OK |
| ChessBoard | /board/:id | Moves, resign, draw, hint, gift | OK (mute disabled honestly) |
| Leaderboard | /leaderboard | Scope tabs, tap → profile | OK |
| Profile | /profile, /:username | History, gifts, send gift | OK + **Settings gear added** |
| Profile settings | /profile/settings | Edit bio, password, email | OK |
| Settings | /settings | 2FA, sessions, legal, logout | OK (now reachable from Profile) |
| 2FA / Sessions / Bug / Support / Legal | /settings/* | Full forms | OK |
| Invite | /invite | Referral link + claim | OK |
| Custom match | /custom-match | Search, invite, accept, poll | OK |
| Notifications | /notifications | Per-type deep links | OK |

### Fixes in Phase 2
1. Profile → Settings (⚙️) link was missing — **added**
2. Splash title Chess King → **Genius Clan**

### Intentionally limited (not bugs)
- Language row disabled (no i18n)
- Voice mute buttons disabled (no WebRTC UI)
- Hint engine = first legal move placeholder

### Empty onClick
None found.
