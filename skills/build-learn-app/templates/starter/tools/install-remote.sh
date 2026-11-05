#!/bin/bash
# Installs dist/Learnapp.app on another Mac over ssh (the Mac the learner
# practises on). Never deletes: an old Learnapp.app goes to that Mac's Trash.
# The learner's data folder is never touched.
# Usage: tools/install-remote.sh <ssh-host>
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOST="${1:?ssh host required (an alias from ~/.ssh/config)}"
APP="$ROOT/dist/Learnapp.app"
[ -d "$APP" ] || { echo "missing $APP — run tools/bundle-macos.sh first"; exit 1; }
SSH="ssh -o ConnectTimeout=8 -o BatchMode=yes $HOST"
$SSH 'if pgrep -x learnapp >/dev/null; then echo "Learnapp runs there — quit it first"; exit 1; fi
mkdir -p "$HOME/Applications"
if [ -e "$HOME/Applications/Learnapp.app" ]; then
  if command -v trash >/dev/null; then trash "$HOME/Applications/Learnapp.app"; else mv "$HOME/Applications/Learnapp.app" "$HOME/.Trash/Learnapp-$(date +%s).app"; fi
fi'
rsync -a "$APP" "$HOST:Applications/"
$SSH 'codesign --verify --deep --strict "$HOME/Applications/Learnapp.app" && echo "signature ok"; lipo -archs "$HOME/Applications/Learnapp.app/Contents/MacOS/learnapp"; "$HOME/Applications/Learnapp.app/Contents/MacOS/learnapp" --version'
echo "Installed on $HOST: ~/Applications/Learnapp.app"
