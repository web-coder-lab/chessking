#!/usr/bin/env bash
# Phase 3 — patch Capacitor Android project for versioning + optional release signing.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_GRADLE="$ROOT/android/app/build.gradle"
MANIFEST="$ROOT/android/app/src/main/AndroidManifest.xml"

if [ ! -f "$APP_GRADLE" ]; then
  echo "android project missing — run cap add android first"
  exit 1
fi

# Version from package.json
VERSION_NAME=$(node -p "require('$ROOT/package.json').version" 2>/dev/null || echo "1.0.0")
VERSION_CODE=${VERSION_CODE:-1}

# Ensure defaultConfig has versionName/versionCode
if grep -q "versionName" "$APP_GRADLE"; then
  sed -i -E "s/versionName .*/versionName \"$VERSION_NAME\"/" "$APP_GRADLE" || true
  sed -i -E "s/versionCode .*/versionCode $VERSION_CODE/" "$APP_GRADLE" || true
else
  # Insert after applicationId line if present
  if grep -q "applicationId" "$APP_GRADLE"; then
    sed -i "/applicationId/a\        versionCode $VERSION_CODE\n        versionName \"$VERSION_NAME\"" "$APP_GRADLE"
  fi
fi

# Network security / cleartext off is default; ensure usesCleartextTraffic false
if [ -f "$MANIFEST" ]; then
  if ! grep -q "usesCleartextTraffic" "$MANIFEST"; then
    sed -i 's/<application/<application android:usesCleartextTraffic="false"/' "$MANIFEST" || true
  fi
fi

# Optional release signing via env (CI secrets)
# KEYSTORE_PATH, KEYSTORE_PASSWORD, KEY_ALIAS, KEY_PASSWORD
if [ -n "${KEYSTORE_PATH:-}" ] && [ -f "${KEYSTORE_PATH}" ]; then
  cat >> "$APP_GRADLE" << GRADLE

android {
    signingConfigs {
        release {
            storeFile file("${KEYSTORE_PATH}")
            storePassword "${KEYSTORE_PASSWORD}"
            keyAlias "${KEY_ALIAS}"
            keyPassword "${KEY_PASSWORD}"
        }
    }
    buildTypes {
        release {
            signingConfig signingConfigs.release
            minifyEnabled false
        }
    }
}
GRADLE
  echo "Release signing config appended"
else
  echo "No KEYSTORE_PATH — release builds will be unsigned or debug-signed"
fi

echo "android-release-prep done versionName=$VERSION_NAME versionCode=$VERSION_CODE"
