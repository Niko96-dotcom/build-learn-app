#!/bin/bash
# Screenshot from the app's own renderer (works while the window is covered;
# never raises it). Usage: shot.sh EVID NAME   → EVID/shots/NAME.png
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}"); NAME="${2:?name}"
[ -e "$EVID/shots/$NAME.png" ] && die "screenshot name used already: $NAME"
touch "$EVID/.shots/$NAME.request"
for _ in $(seq 1 100); do
  if [ -s "$EVID/.shots/$NAME.bmp" ]; then
    sips -s format png "$EVID/.shots/$NAME.bmp" --out "$EVID/shots/$NAME.png" >/dev/null
    # Empty the large scratch BMP in place (no Trash: hundreds of shots would fill it).
    : >"$EVID/.shots/$NAME.bmp"
    echo "$EVID/shots/$NAME.png"
    exit 0
  fi
  sleep 0.1
done
die "no screenshot after 10 s (is the app in background mode with LEARNAPP_SHOT_DIR?)"
