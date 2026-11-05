#!/bin/bash
# Stops the instance of this evidence dir by pid (never by name). Evidence stays.
# Usage: cleanup.sh EVID
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}")
PID=$(pid_of "$EVID")
if kill -0 "$PID" 2>/dev/null; then
  CMD=$(ps -p "$PID" -o comm= || true)
  case "$CMD" in */learnapp|learnapp) kill "$PID"; echo "stopped $PID" ;; *) die "pid $PID is $CMD, not learnapp: not killed" ;; esac
else
  echo "not running: $PID"
fi
echo "evidence kept: $EVID"
