#!/bin/bash
# Prints the last trace event of a kind (run_end, transport, hit, screen, saved, …).
# Usage: last.sh EVID KIND
source "$(dirname "$0")/common.sh"
EVID=$(evid "${1:-}"); KIND="${2:?kind}"
grep "\"kind\":\"$KIND\"" "$EVID/data/trace.jsonl" | tail -1 || die "no $KIND event"
