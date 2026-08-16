# Part 23 — Smooth refresh

- `/auth/refresh` uses `skipAuth` (no stale bearer needed)
- Splash: if language chosen + session cookie → `/dashboard`
- AuthScreen: bootstrap “Restoring session…” then redirect if logged in
- Part 12: network blips do not clear tokens
