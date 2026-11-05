#!/bin/bash
# Lists the accessibility tree: role | title | description | value | frame.
# Usage: labels.sh EVID [--actions]
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}"); shift
"$DRIVE" labels "$(pid_of "$EVID")" "$@"
