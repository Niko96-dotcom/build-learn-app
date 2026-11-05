#!/bin/bash
# Shared helpers for the verify-learnapp scripts. Source it; do not run it.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
# One shared build folder for every agent, the gate and verification (per-agent
# folders filled the disk twice in earlier builds). Release profile only.
export CARGO_TARGET_DIR="${LEARNAPP_TARGET:-$ROOT/target/shared}"
APP_BIN="$CARGO_TARGET_DIR/release/learnapp"
DRIVE="$CARGO_TARGET_DIR/release/learnapp-drive"
MIN_FREE_GIB="${LEARNAPP_MIN_FREE_GIB:-6}"

die() { echo "$*" >&2; exit 1; }

# Refuses to build or launch when the disk is nearly full (full disks kill
# agents, databases and the harness itself).
disk_guard() {
  local free
  free=$(df -g "$ROOT" | awk 'NR==2 {print $4}')
  [ "${free:-0}" -ge "$MIN_FREE_GIB" ] || die "disk guard: only ${free} GiB free (need $MIN_FREE_GIB)"
}

# The evidence dir of a run: absolute, must exist.
evid() {
  local e="${1:-}"
  [ -n "$e" ] || die "EVID required (the path launch.sh printed)"
  [ -d "$e" ] || die "no evidence dir $e"
  (cd "$e" && pwd)
}

pid_of() { cat "$1/pid" 2>/dev/null || die "no pid file in $1 (launch first)"; }

# Reads a key path ("last.verdict") from a JSON file with plutil (no python needed).
json_get() { plutil -extract "$2" raw -o - "$1" 2>/dev/null || true; }

# Moves our own scratch files to the Trash (never deletes).
discard() { if command -v trash >/dev/null 2>&1; then trash "$@"; else for p in "$@"; do [ -e "$p" ] && mv "$p" "$HOME/.Trash/$(basename "$p").$(date +%s)"; done; fi; }

# Logs the frontmost app; the app under test must never be in front.
front_log() {
  local e="$1" line
  line=$("$DRIVE" front 2>/dev/null || echo "? unknown")
  echo "$(date +%H:%M:%S) $line" >>"$e/front.log"
  case "$line" in *" learnapp"*|*" Learnapp"*) echo "WARNING: learnapp is frontmost" | tee -a "$e/front.log" >&2 ;; esac
}
