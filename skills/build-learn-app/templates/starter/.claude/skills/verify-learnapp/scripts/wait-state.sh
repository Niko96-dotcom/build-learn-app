#!/bin/bash
# Waits until state.json has KEY == VALUE (no fixed sleeps in journeys).
# Usage: wait-state.sh EVID KEY VALUE [TIMEOUT_S]
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}"); KEY="${2:?key}"; VALUE="${3:?value}"; T="${4:-20}"
for _ in $(seq 1 $((T * 10))); do
  [ "$(json_get "$EVID/data/state.json" "$KEY")" = "$VALUE" ] && { echo "$KEY=$VALUE"; exit 0; }
  sleep 0.1
done
die "timeout: $KEY is '$(json_get "$EVID/data/state.json" "$KEY")', wanted '$VALUE'"
