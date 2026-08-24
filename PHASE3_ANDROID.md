# Android Phase 3 — Release APK / Play-ready

## Done
- Dual CI: **debug** (default) + **release** (workflow_dispatch)
- `scripts/android-release-prep.sh` — versionName from package.json, versionCode from run number
- Optional signing via GitHub secrets
- `PLAY_STORE.md` listing + keystore instructions
- Cleartext traffic disabled in manifest when patched

## You still need for Play upload
1. Generate `.jks` and add 4 secrets
2. Run workflow with **release**
3. Download artifact → Play Console
4. Privacy policy URL + screenshots

## 3 Android phases complete
| Phase | Result |
|-------|--------|
| 1 | Capacitor shell + CI debug |
| 2 | Icons / splash / design scale |
| 3 | Release path + Play docs |
