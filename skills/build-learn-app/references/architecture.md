# Architecture

The starter (`templates/starter`, created with `scripts/new-app.sh`) already
compiles, passes fmt/clippy/tests, and is driven end to end by the
self-test. Grow it; do not replace its contracts.

## Workspace

```
Cargo.toml                  workspace (resolver 3, edition 2024), shared deps
crates/<slug>-core          learner model, scheduler, scoring, storage — no UI, no audio, no clock
crates/<slug>-app           eframe/egui app: screens, input, trace, platform (binary <slug>)
tools/<slug>-drive          verification driver (keys to the pid, AXPress, shots, timed play)
tools/bundle-macos.sh       universal signed .app
tools/notarize-macos.sh     notarize + staple (only after the owner's OK)
tools/install-local.sh      install into ~/Applications on this Mac
tools/install-remote.sh     install on the practice Mac over ssh
.cargo/config.toml          one build folder for every cargo command: target/shared
tools/make-icon.py          1024 px icon, stdlib only
assets/fonts                Geist family + OFL.txt (embedded with include_bytes!)
.claude/skills/verify-<slug>  the project verification skill
SPEC.md, CLAUDE.md, docs/grilling-log.md, docs/research/
```

Add crates as the SPEC needs them, for example:
- `<slug>-model` — content formats, parser, validator, embedded library
  (`build.rs` + `include_str!` over `content/**/*.toml`). Write it first: it
  is the contract other agents build against.
- `<slug>-audio` — cpal engine, voices, mixer, offline renderer.
- `<slug>-theory` / `<slug>-dsp` — pure domain logic (music theory, audio
  analysis).
- `<slug>-io` — MIDI, network sync, host integration.

**Dependency rule:** crates built in parallel never depend on each other
(core and audio both depend only on model). The app crate uses all. This
keeps one agent's broken build from blocking another.

## Stack (proven, Nov 2026)

eframe/egui 0.37 (`wgpu`, `accesskit`, custom painting), objc2 0.6 +
objc2-app-kit 0.3 for macOS bits, cpal 0.18 + rtrb 0.4 for audio, serde/
serde_json/toml for data, symphonia for decoding, egui_kittest 0.37 for UI
tests. Check for newer versions at build time, but keep egui/eframe/kittest
on the same minor version. In eframe 0.37 the app implements
`App::ui(&mut self, ui, frame)` (and optionally `logic`, `raw_input_hook`).

## Time base (timing apps)

- One clock: host time in ns = `mach_absolute_time` = NSEvent timestamp ×
  1e9 = cpal `StreamInstant`.
- Keys: an `NSEvent` local monitor (`input.rs`) sees key events before winit,
  with hardware timestamps, without any permission. Physical key codes
  (positions); labels from the active layout. It ignores repeats and
  Cmd/Ctrl/Option, pauses while text fields or system panels have focus,
  and sends key-ups for held keys on focus loss.
- Audio: the callback publishes `{frame, host_ns, sample_rate}`; a key at
  host time t is audible frame `frame + (t − host_ns) × sr / 1e9`. Measure
  the callback lead per callback (cpal's timestamp can run one buffer ahead).
- Transport ms = (hit − zero) / 1e6 − calibration offset.
- Keep the process out of App Nap while practising (`platform::NoAppNap`).

## Audio engine rules

No allocation, locks, I/O or panics in the callback (prove it with a
counting-allocator test). Commands over an SPSC ring; retired buffers come
back over a second ring and are dropped on the UI thread. One mixer shared
by the live callback and an offline renderer, so tests and previews use the
same DSP; output identical across buffer sizes. Voice pool with stealing and
short fades; peak limiter. Buffer 128 frames by default (64–512 in
settings), rebuild the stream on device change. Render preview WAVs for the
user: agents cannot judge sound. Verification runs silent: the engine must
honour `<SLUG>_NO_AUDIO=1` (render on the host clock without opening a
device) and the app reports `"audio":"off"` in its `start` trace event
(`"none"` while there is no engine, `"on"` otherwise); the verify doctor
fails on anything else. The build Mac's output may be a remote-desktop
device, and the user may be working next to it.
Never drive audio timing from the UI frame loop (macOS throttles background
repaints); schedule ahead.

## Storage

`~/Library/Application Support/<Name>/` (override `--data-dir` /
`<SLUG>_DATA_DIR`): `profile.json` (versioned, atomic temp+fsync+rename),
`history.jsonl` (one line per run, raw data so metrics can change later),
plus app folders (takes, ideas, kits). A profile that cannot be read or is
from a newer version is never overwritten: saving stops and the app shows a
permanent note (`store.rs`). Every user-visible output folder (exports,
recordings) has an env override so tests never write into the user's
folders. Licensed or private material lives outside git and outside the
bundle.

## Trace contract (the verification interface)

With `--trace`, the app appends to `trace.jsonl` (`{"kind","t_ms","data"}`)
and rewrites `state.json` on every change.

- `start` — version, data_dir, input (`native`/`egui`), background, today.
- `screen` — every screen change.
- `transport` — every scored run: `scored`, `zero_host_ns`, and the correct
  answers (`targets: [{ms, key|code, hold_ms}]`). The driver answers from
  this, like a learner. For non-timing exercises, publish the exercise rules
  in `state.json` instead (allowed items, the expected end) so the driver can
  build a correct answer.
- `hit` / input events — host time, transport ms, what was pressed.
- `run_end` — the full summary, verdict, passed, stage changes.
- `saved` / `save_failed`, `settings`, `reset`, `run_left`, `exit`.

`state.json` holds what a journey waits on: screen, phase, current item,
counts, flags (`done_today`, `saving`), the last result.

## Command line and test hooks

`--data-dir`, `--trace`, `--size WxH`, `--screen NAME` (test shortcut, never
a proof path), `--background` (verification mode), `--now YYYY-MM-DD` (test
clock; only with `--trace` and `--data-dir`), `--version`. Env:
`<SLUG>_NO_AUDIO`, `<SLUG>_NO_NATIVE_INPUT` (UI tests), `<SLUG>_SHOT_DIR`
(in-app screenshots), `<SLUG>_EXPORT_DIR`, and one env file path per native
panel the app opens (background mode reads the choice instead of opening it).

## Background mode (`--background`)

For driving the app while someone works on the same Mac: NSApplication
`activate*` and NSWindow `makeKeyAndOrderFront:`/`orderFront:` are replaced
so the app never takes focus or covers anything; accessory activation policy
(no Dock icon); eframe keeps running and painting while covered, capped at
~60 frames/s; the window reports focus to egui so posted keys count; in-app
screenshots render the next frame offscreen (`<name>.request` →
`<name>.bmp`). Finder and native panels are skipped or replaced. A proof
that used a replacement names it.

## UI tests

`tests/ui_screens.rs`: the real app rendered offscreen (egui_kittest +
wgpu) at 1440×900, driven by accessibility labels, native input off,
harnesses run one at a time (a static mutex; parallel wgpu harnesses abort).
`<SLUG>_RENDER_DIR=<dir>` saves every screen as PNG: look at them.

## Disk

One shared build folder for all agents, the gate, verification and the
bundle: `.cargo/config.toml` sets `target/shared`; the scripts use the
release profile; `debug = "line-tables-only"` in both profiles; a
free-space guard (≥ 6 GiB) before builds, launches and bundles. Per-agent build
folders filled a 1.8 TB disk twice and killed agents and the harness's own
database.
