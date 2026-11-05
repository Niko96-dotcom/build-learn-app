#!/bin/bash
# Answers the next scored run on time from the trace's transport event.
# Start it BEFORE pressing Start (it waits for the next run). Saves the driver report.
# Usage: play.sh EVID [--offset-ms X] [--jitter-ms J] [--skip-every N] [--extra-every N] [--seed N]
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}"); shift
N=$(find "$EVID" -maxdepth 1 -name 'play-*.json' | wc -l | tr -d ' ')
OUT="$EVID/play-$((N + 1)).json"
"$DRIVE" play --pid "$(pid_of "$EVID")" --trace "$EVID/data/trace.jsonl" "$@" | tee "$OUT"
front_log "$EVID"
