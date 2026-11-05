#!/bin/bash
# Performs any AX action on the element with this label
# (AXIncrement, AXDecrement, AXScrollToVisible, AXPress, …).
# Usage: action.sh EVID "Label" AXAction
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}"); LABEL="${2:?label required}"; ACTION="${3:?AX action required}"
"$DRIVE" action "$(pid_of "$EVID")" "$LABEL" "$ACTION"
front_log "$EVID"
