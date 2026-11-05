#!/bin/bash
# Read-only check of the build Mac before a learning-app build. Changes nothing.
# Prints one line per fact and a summary: what is ready, what needs the owner.
# Usage: check-env.sh [project-dir]
set -uo pipefail
DIR="${1:-$PWD}"
ok() { printf '  ok    %s\n' "$*"; }
need() { printf '  NEED  %s\n' "$*"; NEEDS=$((NEEDS + 1)); }
info() { printf '  info  %s\n' "$*"; }
NEEDS=0

echo "build machine"
info "host $(hostname -s), macOS $(sw_vers -productVersion 2>/dev/null || echo ?), $(uname -m)"
[ "$(uname -s)" = Darwin ] || need "macOS is required for the input, driver and release parts"

echo "rust"
if xcode-select -p >/dev/null 2>&1; then ok "Xcode Command Line Tools ($(xcode-select -p))"; else need "Xcode Command Line Tools (xcode-select --install): the linker, lipo, python3"; fi
if command -v cargo >/dev/null; then ok "$(rustc --version)"; else need "Rust (https://rustup.rs)"; fi
RUSTUP="$(command -v rustup || true)"
[ -z "$RUSTUP" ] && [ -x /opt/homebrew/opt/rustup/bin/rustup ] && RUSTUP=/opt/homebrew/opt/rustup/bin/rustup
TARGETS="$([ -n "$RUSTUP" ] && "$RUSTUP" target list --installed --toolchain stable 2>/dev/null || true)"
if grep -qx aarch64-apple-darwin <<<"$TARGETS" && grep -qx x86_64-apple-darwin <<<"$TARGETS"; then
  ok "rustup with aarch64 and x86_64 targets (universal builds)"
else
  info "universal builds need: rustup target add aarch64-apple-darwin x86_64-apple-darwin (else host architecture only)"
fi

echo "disk"
FREE=$(df -g "$DIR" 2>/dev/null | awk 'NR==2 {print $4}')
if [ "${FREE:-0}" -ge 30 ]; then ok "${FREE} GiB free"; else need "only ${FREE:-?} GiB free: a Rust GUI build with agents needs ~10–30 GiB"; fi

echo "permissions (for driving the real app)"
# AXIsProcessTrusted asks nothing and shows no dialog; it answers for the
# app that runs this shell (the terminal or agent host).
if xcode-select -p >/dev/null 2>&1 && python3 -I -c 'import ctypes,sys; a=ctypes.cdll.LoadLibrary("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices"); sys.exit(0 if a.AXIsProcessTrusted() else 1)' 2>/dev/null; then
  ok "Accessibility for this terminal / agent host"
else
  need "Accessibility for this terminal / agent host (System Settings → Privacy & Security → Accessibility)"
fi

echo "signing"
ID=$(security find-identity -v -p codesigning 2>/dev/null | grep -c "Developer ID Application" || true)
if [ "${ID:-0}" -gt 0 ]; then ok "Developer ID Application identity found ($ID)"; else info "no Developer ID: bundles are ad-hoc signed (other Macs warn on first open)"; fi
info "notarization needs a notarytool keychain profile and the owner's OK (outward-facing)"

echo "tools"
for t in git python3 sips iconutil lipo codesign plutil rsync; do
  if command -v "$t" >/dev/null; then ok "$t"; else need "$t"; fi
done
command -v trash >/dev/null && ok "trash (deletions go to the Trash)" || info "no trash command: scripts move files into ~/.Trash"
command -v gh >/dev/null && info "gh: $(gh auth status 2>&1 | grep -m1 -o 'Logged in to [^ ]* account [^ ]*' || echo 'not logged in')"

echo "audio output"
OUT=$(system_profiler SPAudioDataType 2>/dev/null | awk '/^        [^ ].*:$/ {name=$0} /Default Output Device: Yes/ {sub(/^ +/, "", name); sub(/:$/, "", name); print name; exit}')
info "default output: ${OUT:-unknown} (remote/virtual devices are not audible here: keep verification silent)"

echo
if [ "$NEEDS" -eq 0 ]; then echo "check-env: READY"; else echo "check-env: $NEEDS item(s) need the owner"; fi
