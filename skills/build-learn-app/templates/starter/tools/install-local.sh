#!/bin/bash
# Installs dist/Learnapp.app into ~/Applications on this Mac (when the learner
# practises on the build Mac). Never deletes: an old copy goes to the Trash.
# The learner's data folder is never touched.
# Usage: tools/install-local.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/dist/Learnapp.app"
DEST="$HOME/Applications/Learnapp.app"
[ -d "$APP" ] || { echo "missing $APP — run tools/bundle-macos.sh first"; exit 1; }
if pgrep -x learnapp >/dev/null; then
  echo "a learnapp process runs (the installed app or a verification instance) — quit it first"; exit 1
fi
mkdir -p "$HOME/Applications"
if [ -e "$DEST" ]; then
  if command -v trash >/dev/null 2>&1; then trash "$DEST"; else mv "$DEST" "$HOME/.Trash/Learnapp-$(date +%s).app"; fi
fi
ditto "$APP" "$DEST"
codesign --verify --deep --strict "$DEST" && echo "signature ok"
lipo -archs "$DEST/Contents/MacOS/learnapp"
"$DEST/Contents/MacOS/learnapp" --version
echo "Installed: $DEST"
