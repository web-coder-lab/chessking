# Android Phase 2 — Icons, splash, design scale

## Brand assets
| File | Use |
|------|-----|
| `frontend/resources/icon.png` (1024) | Launcher |
| `frontend/resources/splash.png` | Splash |
| `frontend/public/icons/icon-192/512.png` | Web / PWA |
| `manifest.webmanifest` | Standalone display |

## Native design scale
`.ck-native` increases:
- Tap targets ≥ 48px
- Title 24px / body 15px
- Card radius 18px
- Safe-area top/bottom padding

## Deep links
Capacitor `allowNavigation` for:
- genius-clan.onrender.com
- genius-clan-api.onrender.com

`appUrlOpen` listener routes path into SPA history.

## CI
Workflow copies icons into `mipmap-*` after `cap sync`.
