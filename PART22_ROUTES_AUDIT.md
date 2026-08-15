# Part 22 — Routes / buttons audit

## Registered routes (App.jsx)
Splash, Auth, Reset password, Complete signup, **Verify email** (added),
Dashboard, Wallet, Checkout, Shop, Inventory, Play, Board, Leaderboard,
Profile, Settings (+ 2FA, sessions, bug, support, legal), Invite, Custom match, Notifications, 404.

## Navigation sources
- BottomNav → dashboard, wallet, play, leaderboard, profile
- Settings rows → all have handlers (language = alert only — intentional)
- Email deep links: `/verify-email?token=`, `/complete-signup?token=`, `/reset-password?token=`

## Fix this part
- Added missing `/verify-email` page (was 404 for verification emails)
