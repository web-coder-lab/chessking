# Genius Clan — Play Store readiness

## App identity
| Field | Value |
|-------|--------|
| App name | Genius Clan |
| Package | `com.geniusclan.app` |
| Category | Games → Board |
| Content | Chess multiplayer, 13+ recommended |

## Store listing (draft)
**Short description**  
Multiplayer chess — ranked, casual, and custom matches with friends.

**Full description**  
Genius Clan is a competitive chess app with ranked matchmaking, casual games, custom invites, wallets/coins, and a dark royal gold theme. Play online, track rating, and climb the clan.

**Graphics needed (you upload in Play Console)**
- Hi-res icon 512×512 (use `frontend/resources/icon.png`)
- Feature graphic 1024×500
- Phone screenshots (2–8)

## Signing secrets (GitHub)
Repo → Settings → Secrets and variables → Actions:

| Secret | Content |
|--------|---------|
| `ANDROID_KEYSTORE_BASE64` | `base64 -w0 your.jks` |
| `ANDROID_KEYSTORE_PASSWORD` | keystore password |
| `ANDROID_KEY_ALIAS` | key alias |
| `ANDROID_KEY_PASSWORD` | key password |

Create keystore once:
```bash
keytool -genkey -v -keystore genius-clan.jks -keyalg RSA -keysize 2048 -validity 10000 -alias geniusclan
base64 -w0 genius-clan.jks > keystore.b64
```

## CI
Actions → **Build Android APK** → Run workflow → choose **release**.

## Privacy / policy (required for Play)
Host a privacy policy URL (e.g. on your site) covering account, email, and gameplay data.
