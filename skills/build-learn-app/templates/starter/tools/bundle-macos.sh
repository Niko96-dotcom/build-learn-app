#!/bin/bash
# Builds dist/Learnapp.app: universal (arm64 + x86_64 when a rustup toolchain
# with the x86_64 std is installed), with icon, licences and Info.plist,
# signed with a Developer ID (hardened runtime) or ad-hoc when there is none.
# Notarization is a separate step (tools/notarize-macos.sh) and needs the
# owner's explicit OK: it sends the app to Apple.
# Usage: tools/bundle-macos.sh [--no-sign | --adhoc] [--host-only]
# Env: LEARNAPP_SIGN_IDENTITY="Developer ID Application: Name (TEAMID)" (default: first one found)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SIGN=1
ADHOC=0
HOST_ONLY=0
for a in "$@"; do
  case "$a" in
    --no-sign) SIGN=0 ;;
    --adhoc) ADHOC=1 ;;
    --host-only | --arm64-only) HOST_ONLY=1 ;;
    *) echo "unknown option: $a"; exit 2 ;;
  esac
done
# Never delete: old bundles go to the Trash.
discard() { if command -v trash >/dev/null 2>&1; then trash "$@"; else for p in "$@"; do mv "$p" "$HOME/.Trash/$(basename "$p").$(date +%s)"; done; fi; }
VERSION="$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')"
BUILD="$(git rev-parse --short HEAD 2>/dev/null || echo dev)"
# The shared build folder (.cargo/config.toml), unless the caller set one.
TD="${CARGO_TARGET_DIR:-$ROOT/target/shared}"
FREE=$(df -g "$ROOT" | awk 'NR==2 {print $4}')
[ "${FREE:-0}" -ge "${LEARNAPP_MIN_FREE_GIB:-6}" ] || { echo "disk guard: only ${FREE} GiB free"; exit 1; }
mkdir -p "$ROOT/dist"
BIN_OUT="$ROOT/dist/.learnapp-bin"

# Universal build: one rustup toolchain with both std targets builds both
# slices with the same compiler. Otherwise: the host architecture only.
RUSTUP="$(command -v rustup || true)"
[ -z "$RUSTUP" ] && [ -x /opt/homebrew/opt/rustup/bin/rustup ] && RUSTUP=/opt/homebrew/opt/rustup/bin/rustup
TARGETS="$([ -n "$RUSTUP" ] && "$RUSTUP" target list --installed --toolchain stable 2>/dev/null || true)"
if [ "$HOST_ONLY" = 0 ] && grep -qx aarch64-apple-darwin <<<"$TARGETS" && grep -qx x86_64-apple-darwin <<<"$TARGETS"; then
  RBIN="$(dirname "$RUSTUP")"
  for T in aarch64-apple-darwin x86_64-apple-darwin; do
    echo "==> $T"
    CARGO_TARGET_DIR="$TD" RUSTUP_TOOLCHAIN=stable PATH="$RBIN:$PATH" MACOSX_DEPLOYMENT_TARGET=13.0 \
      cargo build --release --locked -p learnapp-app --bin learnapp --target "$T"
  done
  lipo -create -output "$BIN_OUT" \
    "$TD/aarch64-apple-darwin/release/learnapp" \
    "$TD/x86_64-apple-darwin/release/learnapp"
else
  echo "==> host architecture only (for universal: rustup target add aarch64-apple-darwin x86_64-apple-darwin)"
  CARGO_TARGET_DIR="$TD" MACOSX_DEPLOYMENT_TARGET=13.0 cargo build --release --locked -p learnapp-app --bin learnapp
  cp "$TD/release/learnapp" "$BIN_OUT"
fi

APP="$ROOT/dist/Learnapp.app"
STAGE="$ROOT/dist/.Learnapp.app.new"
[ -e "$STAGE" ] && discard "$STAGE"
mkdir -p "$STAGE/Contents/MacOS" "$STAGE/Contents/Resources"
mv "$BIN_OUT" "$STAGE/Contents/MacOS/learnapp"

# Icon: tools/make-icon.py renders assets/icon/icon_1024.png (look at it before bundling).
SRC="$ROOT/assets/icon/icon_1024.png"
[ -f "$SRC" ] || python3 "$ROOT/tools/make-icon.py" "$SRC"
ICONSET="$TD/Learnapp.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z $s $s "$SRC" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  d=$((s * 2))
  sips -z $d $d "$SRC" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$STAGE/Contents/Resources/Learnapp.icns"

# Licences shipped inside the app.
cp "$ROOT/assets/fonts/OFL.txt" "$STAGE/Contents/Resources/Fonts-Geist-OFL.txt"
if [ -f "$ROOT/LICENSE" ]; then cp "$ROOT/LICENSE" "$STAGE/Contents/Resources/LICENSE.txt"; fi

cat > "$STAGE/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Learnapp</string>
  <key>CFBundleDisplayName</key><string>Learnapp</string>
  <key>CFBundleIdentifier</key><string>com.example.learnapp</string>
  <key>CFBundleVersion</key><string>${VERSION}.${BUILD}</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleExecutable</key><string>learnapp</string>
  <key>CFBundleIconFile</key><string>Learnapp</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.education</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
  <key>NSHumanReadableCopyright</key><string>Learnapp. Fonts: Geist, SIL OFL 1.1.</string>
</dict>
</plist>
PLIST
plutil -lint "$STAGE/Contents/Info.plist" >/dev/null

if [ "$SIGN" = 1 ]; then
  IDENTITY=""
  [ "$ADHOC" = 0 ] && IDENTITY="${LEARNAPP_SIGN_IDENTITY:-$(security find-identity -v -p codesigning 2>/dev/null | sed -n 's/.*"\(Developer ID Application:[^"]*\)".*/\1/p' | head -1)}"
  if [ -n "$IDENTITY" ]; then
    echo "==> sign: Developer ID (hardened runtime)"
    codesign --force --options runtime --timestamp --sign "$IDENTITY" "$STAGE/Contents/MacOS/learnapp"
    codesign --force --options runtime --timestamp --sign "$IDENTITY" "$STAGE"
  else
    echo "==> sign: ad-hoc (other Macs warn on first open)"
    codesign --force --sign - "$STAGE/Contents/MacOS/learnapp"
    codesign --force --sign - "$STAGE"
  fi
  codesign --verify --deep --strict --verbose=2 "$STAGE"
fi

[ -e "$APP" ] && discard "$APP"
mv "$STAGE" "$APP"
echo "==> $APP"
lipo -archs "$APP/Contents/MacOS/learnapp"
