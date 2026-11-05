#!/bin/bash
# End-to-end self-test of this skill on this Mac. One command, one verdict:
# scaffolds a throwaway app from templates/starter, runs its gate, drives the
# real app in background mode with its generated verify skill (scored run with
# a known offset, misses, leave, settings, persistence after relaunch), takes
# an in-app screenshot, and builds an ad-hoc signed bundle.
# Prints "selftest: PASS" or "selftest: FAIL (<step>)".
#
# Usage: selftest.sh [workdir]   (default: $TMPDIR/build-learn-app-selftest)
# Needs: Rust, Accessibility for the calling process. First run ~5–10 min (full build).
set -uo pipefail
SKILL="$(cd "$(dirname "$0")/.." && pwd)"
WORK="${1:-${TMPDIR:-/tmp}/build-learn-app-selftest}"
APPDIR="$WORK/pulse"
export PULSE_TARGET="$WORK/build"          # shared, reused by later self-tests
fail() { echo "selftest: FAIL ($*)"; exit 1; }
step() { echo; echo "== $*"; }
discard() { if command -v trash >/dev/null 2>&1; then trash "$@"; else for p in "$@"; do mv "$p" "$HOME/.Trash/$(basename "$p").$(date +%s)"; done; fi; }
near() { awk -v v="$1" -v want="$2" -v tol="$3" 'BEGIN { d = v - want; if (d < 0) d = -d; exit !(d <= tol) }'; }

mkdir -p "$WORK"
[ -e "$APPDIR" ] && discard "$APPDIR"

step "scaffold"
"$SKILL/scripts/new-app.sh" "$APPDIR" pulse "Pulse" com.example.selftest || fail "new-app.sh"
S="$APPDIR/.claude/skills/verify-pulse/scripts"
cd "$APPDIR" || fail "cd"
git init -q && git add -A && git -c user.name=selftest -c user.email=selftest@example.com commit -qm "scaffold" || fail "git"

step "gate (fmt, clippy -D warnings, tests)"
GATE=$("$S/gate.sh") ; echo "$GATE"
case "$GATE" in "gate: PASS"*) ;; *) fail "gate" ;; esac

step "verify: launch + doctor"
EVID=$("$S/launch.sh" selftest) || fail "launch"
echo "$EVID"
"$S/doctor.sh" "$EVID" || { "$S/cleanup.sh" "$EVID"; fail "doctor after launch"; }
"$S/shot.sh" "$EVID" 01-today >/dev/null || { "$S/cleanup.sh" "$EVID"; fail "in-app screenshot"; }

run_scored() { # $1 = shot name, rest = play args
  local name="$1"; shift
  "$S/play.sh" "$EVID" "$@" >"$EVID/$name.play.json" 2>/dev/null &
  local player=$!
  sleep 0.5
  if [ "$(plutil -extract phase raw -o - "$EVID/data/state.json")" = summary ]; then
    "$S/press.sh" "$EVID" Again >/dev/null
  else
    "$S/press.sh" "$EVID" Start >/dev/null
  fi
  wait "$player" || return 1
  "$S/wait-state.sh" "$EVID" phase summary 15 >/dev/null || return 1
  "$S/shot.sh" "$EVID" "$name" >/dev/null
}
get() { plutil -extract "$1" raw -o - "$EVID/data/state.json"; }
cleanup_fail() { "$S/cleanup.sh" "$EVID" >/dev/null; fail "$*"; }

step "verify: scored run, every press 15 ms late"
"$S/press.sh" "$EVID" Practice >/dev/null && "$S/wait-state.sh" "$EVID" screen practice >/dev/null || cleanup_fail "open Practice"
run_scored 02-on-time --offset-ms 15 || cleanup_fail "scored run"
cat "$EVID/02-on-time.play.json"
echo "matched $(get last.matched)/$(get last.targets) bias $(get last.bias_ms) spread $(get last.spread_ms) — $(get last.verdict)"
grep -q '"late":0' "$EVID/02-on-time.play.json" || cleanup_fail "driver posted late events"
[ "$(get last.matched)" = 8 ] || cleanup_fail "expected 8 of 8 matched"
near "$(get last.bias_ms)" 15 2 || cleanup_fail "bias not 15 ± 2 ms"
[ "$(get last.passed)" = true ] || cleanup_fail "run not passed"

step "verify: misses and extras are reported honestly"
run_scored 03-misses --skip-every 4 --extra-every 3 || cleanup_fail "run with misses"
echo "matched $(get last.matched) missed $(get last.missed) extras $(get last.extras) — $(get last.verdict)"
[ "$(get last.missed)" = 2 ] || cleanup_fail "expected 2 missed"
[ "$(get last.extras)" = 2 ] || cleanup_fail "expected 2 extras"
case "$(get last.verdict)" in "6 of 8 in time, 2 missed, 2 extra hits"*) ;; *) cleanup_fail "verdict hides a miss" ;; esac

step "verify: leave session scores nothing"
RUNS=$(get runs)
"$S/press.sh" "$EVID" Again >/dev/null && "$S/wait-state.sh" "$EVID" phase count_in 5 >/dev/null || cleanup_fail "start for leave"
"$S/press.sh" "$EVID" "Leave session" >/dev/null && "$S/wait-state.sh" "$EVID" screen today >/dev/null || cleanup_fail "leave"
[ "$(get runs)" = "$RUNS" ] || cleanup_fail "leave changed the run count"
"$S/last.sh" "$EVID" run_left >/dev/null || cleanup_fail "no run_left event"
[ "$(get done_today)" = true ] || cleanup_fail "done_today should be true after two finished runs"
"$S/shot.sh" "$EVID" 04-today-done >/dev/null

step "verify: settings persist"
"$S/press.sh" "$EVID" Settings >/dev/null && "$S/wait-state.sh" "$EVID" screen settings >/dev/null || cleanup_fail "open Settings"
"$S/press.sh" "$EVID" "Reduce motion" >/dev/null && "$S/wait-state.sh" "$EVID" reduce_motion true >/dev/null || cleanup_fail "reduce motion"
grep -q '"reduce_motion": true' "$EVID/data/profile.json" || cleanup_fail "reduce motion not saved"
"$S/doctor.sh" "$EVID" >/dev/null || cleanup_fail "doctor at the end"
"$S/cleanup.sh" "$EVID" >/dev/null

step "verify: relaunch with the same data"
EVID2=$("$S/launch.sh" selftest-relaunch --keep-data "$EVID/data") || fail "relaunch"
R2=$(plutil -extract runs raw -o - "$EVID2/data/state.json")
H2=$(wc -l <"$EVID2/data/history.jsonl" | tr -d ' ')
"$S/cleanup.sh" "$EVID2" >/dev/null
[ "$R2" = 2 ] && [ "$H2" = 2 ] || fail "persistence: runs $R2, history lines $H2 (expected 2 and 2)"
echo "runs $R2, history lines $H2"
ls "$EVID/shots" | grep -q 04-today-done.png || fail "evidence removed by cleanup"

step "bundle (ad-hoc signed, host architecture, shared build folder)"
CARGO_TARGET_DIR="$PULSE_TARGET" tools/bundle-macos.sh --adhoc --host-only >"$WORK/bundle.log" 2>&1 || fail "bundle (see $WORK/bundle.log)"
codesign --verify --deep --strict dist/Pulse.app || fail "bundle signature"
dist/Pulse.app/Contents/MacOS/pulse --version || fail "bundled binary"

echo
echo "evidence: $EVID"
echo "selftest: PASS"
