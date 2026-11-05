#!/bin/bash
# Builds and starts one isolated, traced instance in background mode.
# Usage: launch.sh [name] [--keep-data DIR] [--now YYYY-MM-DD] [--screen NAME]
# Prints the evidence dir (EVID). Ready = state.json exists and the window is in the AX tree.
source "$(dirname "$0")/common.sh"
NAME="run-$(date +%Y%m%d-%H%M%S)"
KEEP="" NOW="" SCREEN=""
while [ $# -gt 0 ]; do
  case "$1" in
    --keep-data) KEEP="$2"; shift 2 ;;
    --now) NOW="$2"; shift 2 ;;
    --screen) SCREEN="$2"; shift 2 ;;
    -*) die "unknown option $1" ;;
    *) NAME="$1"; shift ;;
  esac
done
disk_guard
cd "$ROOT"
cargo build --release -p learnapp-app -p learnapp-drive >&2
EVID="$ROOT/evidence/$NAME"
[ -e "$EVID" ] && die "evidence dir exists: $EVID (pick another name)"
mkdir -p "$EVID/data" "$EVID/shots" "$EVID/.shots"
if [ -n "$KEEP" ]; then
  # The learner data of an earlier run, without its trace and state.
  rsync -a --exclude trace.jsonl --exclude state.json "$KEEP/" "$EVID/data/"
fi
{
  echo "git: $(git rev-parse --short HEAD 2>/dev/null || echo none) dirty: $(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
  echo "binary: $APP_BIN"
  echo "sha: $(shasum -a 256 "$APP_BIN" | cut -d' ' -f1)"
  echo "launched: $(date '+%Y-%m-%d %H:%M:%S')"
  echo "now: ${NOW:-real clock}"
} >"$EVID/build.txt"
ARGS=(--data-dir "$EVID/data" --trace --background --size 1440x900)
[ -n "$NOW" ] && ARGS+=(--now "$NOW")
[ -n "$SCREEN" ] && ARGS+=(--screen "$SCREEN")
LEARNAPP_SHOT_DIR="$EVID/.shots" LEARNAPP_NO_AUDIO=1 \
  nohup "$APP_BIN" "${ARGS[@]}" >"$EVID/app.log" 2>&1 &
echo $! >"$EVID/pid"
PID=$(cat "$EVID/pid")
[ -x "$DRIVE" ] || { kill "$PID" 2>/dev/null; die "driver missing: $DRIVE"; }
for _ in $(seq 1 120); do
  if [ -f "$EVID/data/state.json" ] && [ "$("$DRIVE" window-count "$PID" 2>/dev/null || echo 0)" -gt 0 ]; then
    front_log "$EVID"
    echo "$EVID"
    exit 0
  fi
  kill -0 "$PID" 2>/dev/null || die "app exited during launch (see $EVID/app.log)"
  sleep 0.5
done
kill "$PID" 2>/dev/null
die "not ready after 60 s; stopped $PID (see $EVID/app.log)"
