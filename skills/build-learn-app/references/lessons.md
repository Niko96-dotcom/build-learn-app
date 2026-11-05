# Lessons

Every item happened in one of the three builds. Each ends with the rule.

## Questions and scope

- The user typed one message per build; everything else came through the
  question tool. → Capture every requirement in the intake and the first
  rounds; there is no later feedback to rely on.
- A skill's own question format (numbered text) conflicted with the user's
  rule (question tool). → Name the conflict, follow the user.
- A template line from another app ("timing gets better") did not fit. →
  Ask once; the user withdrew it.
- The user floated an idea ("vary the key mapping on purpose?"). Research
  showed a fixed mapping builds automaticity. → Answer open ideas with
  evidence inside the recommendation.
- "Zero to expert in a few weeks" at 15 min/day. → Give the honest estimate
  early, revise it with research (8–12 → 16–20 weeks), let the user choose
  goal or date.
- The user refused a recommended option twice (showing the creator's clips;
  the pure-Rust GPL port). → Recommendations are proposals; log the answer
  and its consequence (no clips in the app; C++ inside, GPL licence).

## Design

- Three agent-made directions and four palettes were all rejected ("they
  look incredibly vibe coded"); the user wanted the earlier app's look
  unchanged. → Start from the bundled design system; mockups only inside it.
- The user wanted to see colour options rendered, then on the laptop in
  their hands. → Show visuals as rendered pages on the user's device.
- "Continue" and "Back" overlapped at 796 px window height; a progress page
  could not scroll, so the last stage was unreachable. → Screenshot every
  screen at the minimum and default window size.
- Combo-box popups were unreliable to drive and clumsy. → Inline labelled
  choices.
- Quick Look rendered the SVG icon on white. → Render icons in code.

## Agents and builds

- Worktree isolation failed (the session root was not the repo). → Shared
  checkout, strict file ownership, one committer, or make sure the repo is
  the session root first.
- A delegate on another model stopped after writing only a WAV writer. →
  Check output objectively (`todo!()` count, tests) before relying on it.
- Seven per-agent build folders filled the disk twice; the harness database
  could not write and an agent died. → One shared build folder, small debug
  info, a disk guard in every build and launch script.
- A weekly usage limit stopped one agent mid-wave. → Re-dispatch the same
  brief to another provider after checking the tree.
- A content agent wrote 48 song progressions from memory; 7 were wrong. →
  A fact-check agent with sources for every remembered fact.
- Agents changed notation and contracts. → Fold deviations into SPEC in the
  same commit.
- The gate failed and the command chain still committed (`;`). → `gate.sh &&
  git commit`.
- A needed generator script lived in a scratch folder and went to the Trash
  in cleanup. → Move every generator into `tools/` as soon as its output is
  committed or installed.
- Python caches were committed. → `.gitignore` before the first commit of a
  new language.
- An agent read a protected drive through a symlink. → Resolve symlinks and
  give agents an explicit read-only path list.
- Batch steps silently skipped 73 items (one shell argument instead of a
  list). → Check that each batch step produced an output for every input.
- A "second pass complete" report was premature. → Count artifacts.

## Verification

- 83–86 green tests; the real app still had ~45 defects (scoring, planner,
  honest wording, layout). → Budget a long real-app loop.
- The first driver used global events and mouse clicks while the user worked
  in a DAW on the same Mac. → Background-safe driver from the start (pid
  posting, AXPress, in-app screenshots, no activation).
- The first synthetic key of each run arrived ~100 ms late. → Warm up the
  posting path; spin-wait; require `late: 0`.
- A screenshot during a timed run delayed one event 85 ms. → Shots before
  and after timed runs.
- Event timestamps in mach ticks were replaced by arrival time; in ns they
  kept the plan exactly (macOS 27). → Stamp posted events in ns; measure.
- Driver mistakes looked like app bugs (swapped key names, missing octave
  keys, major-scale degrees in a minor key). → Reproduce with a core test on
  the trace data before filing; publish exercise rules in `state.json`.
- Test exports landed in the user's real Music folder. → An env override for
  every output path.
- The native open panel swallowed typed letters as instrument keys. → Pause
  play keys while panels and text fields have focus; replace panels in
  background mode.
- The verify skill was built after integration; real-app proofs came late.
  → Build it with the app shell.
- The feature map said "not yet" after the journeys had passed. → Update
  the map in the same commit as the proof.
- A crash happened when "Close" ended a run while the same frame read it.
  → The final run clicks every exit path; a regression test per crash.

## Honesty

- "Done for today" after leaving early; "Clean run: 47 of 48" with a wrong
  note; "Rate 10 old takes" with 3 takes; "1 candidates"; an error toast
  that said "damaged and set aside" for a permission problem. → Derive every
  word and number from data; test the copy.
- Nobody heard the sounds (the build Mac's output was a remote-desktop
  device). → Preview renders for the user; never claim the sound is good.
- The beginner pace was only simulated; the Intel slice only ran under
  Rosetta. → Say exactly what ran where.

## Machine and data safety

- A macOS permission dialog appeared during the questions; the agent cannot
  click it. → Probe permissions early, ask the user to allow them as a
  question, verify with a read-only probe.
- Deleting is blocked by the user's rules; the Trash frees no space. → Ask
  once (in the allowance bundle) how build caches may be removed.
- A shared system log had another session's uncommitted changes. → Re-read
  shared files right before editing; never commit others' work.
- A push failed with a server error. → Retry later; never rewrite history
  to "fix" a remote failure.
