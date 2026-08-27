# Phase 5 — Polish + release APK

## Done
- **Session persist** — SharedPreferences JWT (`TokenStore`)
- **Skip login** if token already saved
- **Logout** clears tokens
- **CI** — `.github/workflows/build-native-apk.yml` (Compose, not Capacitor)
- **Release signing** via env + GitHub secrets

## Build APK
### Android Studio
1. Open `android-native/`
2. Build → Build Bundle(s) / APK(s) → Build APK(s)

### GitHub Actions
Actions → **Build Native Compose APK** → Run → `debug` or `release`  
Artifact: `genius-clan-native-debug-apk` / `genius-clan-native-release-apk`

### Release secrets
`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`

## 5 phases complete
| Phase | Result |
|-------|--------|
| 1 | API-only server |
| 2 | Compose scaffold + design |
| 3 | Auth / profile / wallet |
| 4 | Matchmaking + board WS |
| 5 | Session + native CI APK |
