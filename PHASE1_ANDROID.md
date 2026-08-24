# Android Phase 1 — Capacitor shell

## Goal
Web Genius Clan → **native Android WebView shell** (Capacitor), better than browser tab.

## 3 phases plan
| Phase | Scope |
|-------|--------|
| **1** (this) | Capacitor config, native init, safe-area, GitHub Actions APK workflow |
| **2** | Android project polish: icon, splash, deep links, API cleartext, design scale |
| **3** | Release signing, store listing assets, optional native plugins |

## Local (when Android SDK available)
```bash
cd frontend
npm install
npm run build
npx cap add android   # once
npx cap sync android
npx cap open android  # Android Studio → Run
```

## CI
Workflow: `.github/workflows/build-android-apk.yml`  
Trigger: push to `frontend/**` or **Actions → Build Android APK → Run**  
Artifact: `genius-clan-debug-apk`

## App id
`com.geniusclan.app` · name **Genius Clan** · dark `#0F1115`
