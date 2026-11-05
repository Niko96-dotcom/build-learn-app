# Build waves

Parallel agents build the app in waves. The lead (you) owns SPEC, the
contracts, the gate, integration and every commit.

## Before the first wave

- SPEC.md is written and committed.
- The contract crate (formats, shared types) compiles and has tests.
- Every crate an agent will fill exists as compiling stubs (`todo!()` bodies,
  documented signatures). The workspace root and `Cargo.toml` members are
  final, so no agent edits shared files.
- `gate.sh` prints `gate: PASS`.

## Wave plan (typical)

| Wave | Agents in parallel | Depends on |
|---|---|---|
| 1 | domain logic (theory/dsp), audio engine, IO, app shell with the design system + main widget + input + trace + driver/verify skill skeleton | contracts |
| 2 | content (formats → files + generators + validation tests), core part A (scoring, learner model, storage) | wave-1 domain logic |
| 3 | core part B (curriculum, planner, session flow, simulation), app integration 1 (practice surface, library, settings), fact check of content | wave 2 |
| 4 | app integration 2 (first start, placement, session flow, progress), hardening (gaps from the integration reports) | wave 3 |
| 5 | full verification: drive every journey, fix, re-drive (one strong agent, or two on disjoint features) | wave 4 |

Start the next agent as soon as its inputs land; do not wait for the whole
wave.

## Rules for every agent (put them in every brief)

- Edit only the files or crate you own. If another crate does not compile,
  wait and retry; never fix someone else's code.
- Build in the shared folder `target/shared` (release profile). No private
  target folders: they filled disks.
- Run no state-changing git (no commit, checkout, stash, reset). The lead
  commits.
- Temp files only under your own folder in `target/`.
- Decide anything SPEC leaves open in its spirit; write the decision as a
  doc comment and list it in your report.
- Report contract gaps (a validator that misses a case, a type that does not
  fit) as a list. Do not work around them silently.
- Prove what you claim with a command and its output. Say what you did not
  check ("I did not listen to the sounds").
- Safety: never write to the user's real data or library folders; never
  quit or change apps the user is using; no network calls beyond what the
  brief allows.
- Audio agents: the engine honours `<SLUG>_NO_AUDIO=1` (silent, host clock)
  and the app's `start` trace event reports `"audio":"off"` then; the
  verify doctor checks it.

## Brief template

```
You build <crate/folder> of <App>, a native macOS app (Rust, eframe/egui)
that teaches <skill>. You have no other history: this brief is all you get.

Read first: SPEC.md §<sections>; docs/research/<files>; <contract files>.
Own: <paths>. Edit nothing else.
Shared checkout rules: other agents build <crates> at the same time. Build
with CARGO_TARGET_DIR=<abs>/target/shared and --release. No git commands
that change state. If another crate fails to compile, wait and retry.

GOAL: <one sentence>.
REQUIREMENTS:
1. <capability with a measurable limit, e.g. "render < 150 ms per comparison">
2. <…>
3. Tests: <what must be tested, e.g. one pass + one fail fixture per type,
   no allocation on the audio thread (counting allocator)>.
Decide anything unspecified in the spirit of SPEC and document it.

FINISH with a report of at most 25 lines: files changed, tests (command +
result), decisions taken, contract gaps, what you did not check.
```

## Checking an agent's work

- Count what is left: `grep -rn "todo!()" crates/<crate>`; run its tests in
  the shared folder; read the diff.
- One delegate "finished" after writing only a WAV writer. Another changed
  the content notation. Do not trust the report; trust the tree.
- Fold every deviation into SPEC in the same commit as the code.
- Then `gate.sh && git commit -m "<what>"`.

## Models and failures

- Strongest model for real-time code, the core learner model, integration
  and verification. Cheaper models for inventories, bulk media analysis and
  fact checks. Another vendor's model for the independent review, if the
  harness offers one.
- Split a big component into disjoint files so two agents can work on it
  (sound design vs. engine).
- On a rate limit or a provider failure: check the working tree for partial
  changes, then send the same brief to another model.
- Detect a hung agent by its missing output files, not by waiting.

## Disk and machine

- Check free disk before each wave (`df -g`); stop data production below
  the guard instead of letting the disk fill.
- The user may work on the same Mac: agents never steal focus, never post
  global input, never drive a program the user has open.
- At the end, move build folders the user does not need to the Trash (or
  ask how), keep evidence.
