#!/bin/bash
# Posts keys to the app pid only. Usage: key.sh EVID f j [--gap MS] [--hold MS]
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}"); shift
"$DRIVE" key "$@" --pid "$(pid_of "$EVID")"
front_log "$EVID"
