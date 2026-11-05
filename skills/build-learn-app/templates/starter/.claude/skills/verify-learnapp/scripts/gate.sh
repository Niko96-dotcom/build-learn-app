#!/bin/bash
# The one pass/fail command before every commit: fmt, clippy -D warnings, all
# tests, in the shared release build folder (a second debug tree would double
# the disk use). Prints "gate: PASS (…)" or "gate: FAIL (<step>)".
# Commit with: gate.sh && git commit …   (never with ";").
source "$(dirname "$0")/common.sh"
cd "$ROOT"
disk_guard
mkdir -p "$ROOT/target"
cargo fmt --all -- --check >/dev/null 2>&1 || { echo "gate: FAIL (cargo fmt --check)"; exit 1; }
cargo clippy --release --workspace --all-targets -- -D warnings >"$ROOT/target/gate-clippy.log" 2>&1 \
  || { echo "gate: FAIL (clippy, see target/gate-clippy.log)"; exit 1; }
if cargo test --release --workspace --no-fail-fast >"$ROOT/target/gate-test.log" 2>&1; then
  echo "gate: PASS ($(grep -c '^test result: ok' "$ROOT/target/gate-test.log") test binaries ok)"
else
  echo "gate: FAIL (tests, see target/gate-test.log)"; exit 1
fi
