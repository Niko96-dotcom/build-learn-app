#!/bin/bash
# Presses a control by its accessibility label (AXPress; the pointer never moves).
# Usage: press.sh EVID "Label"
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}"); LABEL="${2:?label required}"
"$DRIVE" press "$(pid_of "$EVID")" "$LABEL"
front_log "$EVID"
