#!/bin/bash
# Notarizes dist/Learnapp.app with Apple and staples the ticket.
# OUTWARD-FACING: this uploads the app to Apple. Run it only after the owner said yes.
# Needs a notarytool keychain profile (create once:
#   xcrun notarytool store-credentials <profile> --apple-id <id> --team-id <team>)
# Usage: LEARNAPP_NOTARY_PROFILE=<profile> tools/notarize-macos.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/dist/Learnapp.app"
PROFILE="${LEARNAPP_NOTARY_PROFILE:?set LEARNAPP_NOTARY_PROFILE to your notarytool keychain profile}"
ZIP="$ROOT/dist/Learnapp-notarize.zip"
discard() { if command -v trash >/dev/null 2>&1; then trash "$@"; else for p in "$@"; do mv "$p" "$HOME/.Trash/$(basename "$p").$(date +%s)"; done; fi; }
[ -d "$APP" ] || { echo "missing $APP — run tools/bundle-macos.sh first"; exit 1; }
codesign --verify --deep --strict "$APP"
[ -e "$ZIP" ] && discard "$ZIP"
ditto -c -k --keepParent "$APP" "$ZIP"
xcrun notarytool submit "$ZIP" --keychain-profile "$PROFILE" --wait --timeout 30m
xcrun stapler staple "$APP"
xcrun stapler validate "$APP"
spctl -a -vv "$APP"
discard "$ZIP"
OUT="$ROOT/dist/Learnapp.zip"
[ -e "$OUT" ] && discard "$OUT"
ditto -c -k --keepParent "$APP" "$OUT"
echo "==> notarized: $APP"
echo "==> share: $OUT"
