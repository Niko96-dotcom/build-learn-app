# Verification

"Completely tested with computer use" means: the real binary, driven by real
OS input, from an empty data folder, with evidence for every claim. Unit
tests and headless UI tests are necessary and not enough: in every build
they passed while the real app still had 20–45 defects.

## The verify skill

The starter ships `.claude/skills/verify-<slug>/`. It follows a simple
shape a later agent can use cold:

- **Launch** — build, isolated `evidence/<run>/data`, `--trace --background`,
  ready = `state.json` exists and the window is in the AX tree.
- **Doctor** — read-only, ends `doctor: OK` / `doctor: FAIL (…)`.
- **Drive** — helpers with real labels and commands from this app.
- **Evidence** — what to capture and where; proof standards.
- **Cleanup** — stop by pid, never by name; evidence stays.
- **Gotchas** — every surprise from real runs.
- **Feature map** — `features/README.md` plus one file per user-facing
  feature with exactly four sections: `Sub-features` (stable ids),
  `How to get to it (user POV)`, `Driving it with verify-<slug>`
  (preconditions; each action paired with a command and an observable
  result), `Gotchas`. The README table records when each feature was last
  proven end to end.

Grow it with the app: add a feature file in the same commit as the
feature; add helpers when a journey needs them (e.g. `session-step.sh` that
reads the current exercise rules from `state.json` and answers like a
learner); update the "proven" column in the same commit as the proof. Prove
the skill itself once early (launch → doctor → one journey → cleanup →
evidence still there).

## Driver rules (background-safe)

The user may be working on the same Mac, in a DAW or an editor.

- Keys only to the app's pid (`CGEventPostToPid`), from a private event
  source, with the planned time as the event timestamp (ns). The driver
  refuses any pid that is not the app binary.
- Controls via `AXPress` on the element with the label; the pointer never
  moves; no mouse events.
- The app runs with `--background` (never activates, stays behind, paints
  while covered). Screenshots from the app's own renderer.
- Log the frontmost app around every action; doctor fails if the app was
  ever in front.
- Audio silent; anything that reaches other apps (MIDI out, network sync,
  a host program) runs only when those apps are not running, or after an
  idle check, and is otherwise reported as pending with the time checked.
- Warm up the posting path (one harmless F19 press ~100 ms before the first
  timed key); sleep then spin for on-time posting; report `late` and require 0.
- Do not take screenshots during a timed run (one event came 85 ms late).

## Proof standards

- The real user path: navigation, buttons, keys. Never internal setters,
  never hand-written `profile.json` or `state.json`. `--screen` shortcuts are
  named when used.
- A screenshot alone proves nothing about behaviour: pair it with the trace
  event or state value.
- Side effects checked in a second place: history lines, profile values,
  exported files (parse them), a relaunch with `--keep-data`.
- Wait on state, not on sleeps.
- A step that cannot run is reported as not verified with the command and
  the reason. Never "verified another way".
- Synthetic input proves capture and scoring, not human timing, keyboard
  rollover or feel. Every timing report says so.
- Report the build that was tested (`build.txt`: git head, dirty files,
  binary sha). Relaunch after a rebuild.

## Journeys to drive (adapt to the app)

1. First start as a beginner: every first-start step, then the first item;
   measure the time to the first win (target < 5 min).
2. First start as an experienced learner: placement puts them later.
3. A whole first session: every exercise type that appears, at least one
   pass and one fail each, the summary, Today afterwards.
4. Later days with the test clock (`--now`, `--keep-data`): cold checks on
   day 2, a review a week later, a monthly event if the app has one.
5. Free practice: every control on the practice surface (tempo, guide
   levels, metronome modes, record, export; parse the exported file).
6. Every exit path: leave mid-run, leave from a summary, close the window
   during a run, switch screens during a count-in.
7. Settings: each setting changes behaviour and persists across relaunch;
   reset moves data to the Trash.
8. Error states: an unreadable profile (the file stays byte-identical,
   saving stops, a permanent note shows), a missing device, a full export
   folder.
9. Layout: every screen at 1080×720 and 1440×900; long content (the
   hardest item, the longest name).
10. Anything that talks to other programs: only when allowed and idle.

## The defect loop

- Keep `docs/verification/defects.md`: id (UI-1, SCORE-3…), what you saw,
  the evidence path, status, fix commit.
- Before filing an app bug, rule out the driver: replay the trace data in a
  core test. Driver bugs looked like app bugs four times.
- Fix with the smallest correct change and a regression test that fails
  without it. Re-drive the journey. Commit each fix (`gate.sh && git commit`).
- Review every screenshot against design-system.md's checklist and the
  first screenshots of the app (consistency over time).
- Never weaken a journey to make it pass.

## Done

Every feature in the map is proven with a date and an evidence path; the
defect list has no open real defects; `docs/verification/<date>-full-run.md`
lists feature → journey → result → evidence, the bugs fixed with commits,
and what only the user can accept.
