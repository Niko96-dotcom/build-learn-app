#!/bin/bash
# Read-only check: is this instance worth driving? Ends with "doctor: OK" or "doctor: FAIL (…)".
# Usage: doctor.sh EVID
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}")
PID=$(pid_of "$EVID")
FAIL=()
if kill -0 "$PID" 2>/dev/null; then
  CMD=$(ps -p "$PID" -o comm= || true)
  case "$CMD" in */learnapp|learnapp) echo "process: $PID $CMD" ;; *) FAIL+=("pid $PID is $CMD") ;; esac
else
  FAIL+=("process $PID is not running")
fi
WANT=$(sed -n 's/^sha: //p' "$EVID/build.txt")
HAVE=$(shasum -a 256 "$APP_BIN" | cut -d' ' -f1)
[ "$WANT" = "$HAVE" ] && echo "binary: launched build" || echo "binary: REBUILT since launch (relaunch before a proof)"
START=$(grep -m1 '"kind":"start"' "$EVID/data/trace.jsonl" || true)
[ -n "$START" ] || FAIL+=("no start event in the trace")
case "$START" in *"\"data_dir\":\"$EVID/data\""*) echo "data: isolated ($EVID/data)" ;; *) FAIL+=("data dir is not $EVID/data") ;; esac
case "$START" in *'"input":"native"'*) echo "input: native NSEvent monitor" ;; *) FAIL+=("input is not native (timing proofs need it)") ;; esac
case "$START" in
  *'"audio":"off"'*|*'"audio":"none"'*) echo "audio: silent" ;;
  *) FAIL+=("audio is not off: the app must honour LEARNAPP_NO_AUDIO=1 and report audio off in the start event") ;;
esac
echo "state: screen=$(json_get "$EVID/data/state.json" screen) phase=$(json_get "$EVID/data/state.json" phase) runs=$(json_get "$EVID/data/state.json" runs) saving=$(json_get "$EVID/data/state.json" saving)"
echo "windows: $("$DRIVE" window-count "$PID" 2>/dev/null || echo 0)"
front_log "$EVID"
grep -q "WARNING" "$EVID/front.log" && FAIL+=("learnapp was frontmost (see front.log)")
if [ ${#FAIL[@]} -eq 0 ]; then echo "doctor: OK"; else echo "doctor: FAIL (${FAIL[*]})"; exit 1; fi
