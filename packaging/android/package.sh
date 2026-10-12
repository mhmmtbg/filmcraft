#!/usr/bin/env bash
# Build and package FilmCraft for Android phones and tablets (64-bit ARM, Android 8.0+):
#
#   $DIST/filmcraft-<version>-android-arm64.apk
#
# Usage: packaging/android/package.sh
#
# Needs: rustup's aarch64-linux-android target; cargo-apk (cargo install cargo-apk --locked);
# the Android SDK (ANDROID_HOME, with build-tools and an android-34 platform) and NDK
# (ANDROID_NDK_ROOT); ImageMagick for the launcher icons; keytool (any JDK).
#
# Signing: CARGO_APK_RELEASE_KEYSTORE (path) and CARGO_APK_RELEASE_KEYSTORE_PASSWORD. Without
# them a throwaway key is generated; such an APK installs fine, but a later APK signed with a
# different key cannot update it in place (uninstall first). Keep one keystore for releases.
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
CRATE="$ROOT/apps/filmcraft-android"
TARGET=aarch64-linux-android

echo "FilmCraft $VERSION for Android arm64 ($TARGET)"

# The crate is its own workspace; its version follows the root one (versionName/versionCode).
sed -i "0,/^version = \".*\"/s//version = \"$VERSION\"/" "$CRATE/Cargo.toml"
# Start from the root lock file, so every crate the desktop app also uses gets the exact version
# the desktop gates test; cargo then only adds the Android-only crates (gitignored copy).
cp "$ROOT/Cargo.lock" "$CRATE/Cargo.lock"

# Launcher icons from the app icon (generated, gitignored: no new asset files in the repo).
MAGICK="$(command -v magick || command -v convert || true)"
if [ -z "$MAGICK" ]; then
  echo "error: ImageMagick (magick or convert) is needed for the launcher icons" >&2
  exit 1
fi
for density in mdpi:48 hdpi:72 xhdpi:96 xxhdpi:144 xxxhdpi:192; do
  dir="$CRATE/res/mipmap-${density%%:*}"
  mkdir -p "$dir"
  "$MAGICK" "$ROOT/assets/app-icon/filmcraft-1024.png" -resize "${density##*:}x${density##*:}" "$dir/ic_launcher.png"
done

if [ -z "${CARGO_APK_RELEASE_KEYSTORE:-}" ]; then
  echo "warning: no CARGO_APK_RELEASE_KEYSTORE; signing with a throwaway key" >&2
  KS="$CARGO_TARGET_DIR/android-throwaway.keystore"
  mkdir -p "$CARGO_TARGET_DIR"
  rm -f "$KS"
  keytool -genkeypair -noprompt -keystore "$KS" -storepass android -keypass android -alias filmcraft \
    -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=FilmCraft"
  export CARGO_APK_RELEASE_KEYSTORE="$KS" CARGO_APK_RELEASE_KEYSTORE_PASSWORD=android
fi

(cd "$CRATE" && cargo apk build --release --target "$TARGET")

APK="$(find "$CARGO_TARGET_DIR/release/apk" "$CRATE/target/release/apk" -maxdepth 1 -name '*.apk' -print -quit 2>/dev/null || true)"
if [ -z "$APK" ]; then
  echo "error: cargo apk produced no .apk under $CARGO_TARGET_DIR/release/apk" >&2
  exit 1
fi
OUT="$DIST/filmcraft-$VERSION-android-arm64.apk"
cp "$APK" "$OUT"
ls -l "$OUT"
