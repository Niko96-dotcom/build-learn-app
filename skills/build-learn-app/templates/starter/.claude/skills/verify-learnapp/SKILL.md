---
name: verify-learnapp
description: Drive the Learnapp macOS app (Rust/egui learning app) like a learner, in the background, while someone works on the same Mac. Real key events posted only to the app's pid, controls pressed through the accessibility tree, in-app screenshots, trace/state/profile evidence. Use to prove any Learnapp feature works in the running app (Today, Practice runs and scoring, Progress, Settings, persistence), not just in unit tests, and to check what a change broke.
---

# verify-learnapp

Drives the real `learnapp` binary from an isolated data folder and proves
behaviour with evidence. Written for an agent that has never seen the app.

## Hard rules (do not disturb the person at this Mac)

- Keys go only to the app's pid (`CGEventPostToPid`); the driver refuses any
  other process. Controls are pressed with `AXPress`; the pointer never moves.
- The app runs with `--background`: it never activates, stays behind other
  windows, paints while covered, and has no Dock icon. Every helper logs the
  frontmost app to `front.log`; doctor fails if the app was ever in front.
- Screenshots come from the app's own renderer (`shot.sh`), never from the
  screen. Audio is off in verification: `launch.sh` sets
  `LEARNAPP_NO_AUDIO=1`, the app reports `"audio":"off"` (or `"none"` while
  it has no engine) in its start event, and doctor fails otherwise.
- Never write `profile.json` or `state.json` by hand to set up a journey.
  Use the real user path, or `--keep-data DIR` with data an earlier run made,
  or the test clock `--now YYYY-MM-DD`.

Permissions: the calling process (terminal / agent host) needs Accessibility
(System Settings → Privacy & Security → Accessibility). `learnapp-drive shot`
(window capture) also needs Screen Recording; `shot.sh` does not.

## Launch

```bash
EVID=$(.claude/skills/verify-learnapp/scripts/launch.sh my-run [--keep-data DIR] [--now 2026-11-06] [--screen practice])
```

Builds `learnapp` and `learnapp-drive` (release, shared folder
`target/shared`), refuses below 6 GiB free disk, creates
`evidence/<name>/` with an empty `data/`, writes `build.txt` (git head,
dirty files, binary sha), starts the app with `--data-dir EVID/data --trace
--background --size 1440x900`, and prints the evidence dir. Ready means:
`state.json` exists and the window is in the AX tree. Several instances can
run side by side (one evidence dir each).

## Doctor

```bash
.claude/skills/verify-learnapp/scripts/doctor.sh "$EVID"   # ends "doctor: OK" or "doctor: FAIL (…)"
```

Read-only: process is ours, binary not rebuilt since launch, data folder is
the isolated one, input is the native NSEvent monitor (needed for timing),
current screen/phase/runs, window count, frontmost app. Run it first whenever
anything looks off, and at the end of every proof.

## Drive

| Helper | What it does |
|---|---|
| `press.sh EVID "Label"` | AXPress on the element with this label |
| `action.sh EVID "Label" AXAction` | any AX action (`AXIncrement`, `AXDecrement`, `AXScrollToVisible`) |
| `labels.sh EVID [--actions]` | the accessibility tree (find labels here) |
| `key.sh EVID f j [--gap MS] [--hold MS]` | keys posted to the app pid |
| `play.sh EVID [--offset-ms X] [--jitter-ms J] [--skip-every N] [--extra-every N]` | answers the next scored run on time from the trace (start it before pressing Start) |
| `wait-state.sh EVID KEY VALUE [T]` | waits on `state.json` (never fixed sleeps) |
| `last.sh EVID KIND` | last trace event of a kind |
| `shot.sh EVID NAME` | in-app screenshot → `EVID/shots/NAME.png` |
| `cleanup.sh EVID` | stops this instance by pid; evidence stays |
| `gate.sh` | fmt + clippy -D warnings + tests; `gate: PASS` / `gate: FAIL (…)` |

Stable labels: nav `Today`, `Practice`, `Progress`, `Settings`; Today
`Start session` (or `Practise more` after a finished run); Practice `Start`,
`Leave session`, `Again`, `Back to Today`; Settings `Reduce motion`,
`Level`, `Reset progress`, `Move to Trash`, `Keep it`.

`state.json` keys: `screen`, `phase` (`ready`, `count_in`, `playing`,
`summary`), `item`, `today`, `runs`, `done_today`, `days_this_week`,
`item_stage`, `reduce_motion`, `reset_asked`, `saving`, `note`,
`background`, `last.{accuracy,bias_ms,spread_ms,matched,targets,missed,extras,verdict,passed}`.
Read one with `plutil -extract KEY raw -o - "$EVID/data/state.json"`
(nested: `last.verdict`).

Trace kinds: `start` (version, data_dir, input, audio, background, today,
profile), `screen`, `transport` (zero time + correct answers), `hit`,
`run_end`, `run_left`, `saved`, `save_failed`, `settings`, `reset`,
`reset_failed`, `exit`.

### Typical journey (scored run with a known offset)

```bash
S=.claude/skills/verify-learnapp/scripts
EVID=$($S/launch.sh scored-run)
$S/doctor.sh "$EVID"
$S/press.sh "$EVID" Practice && $S/wait-state.sh "$EVID" screen practice
$S/play.sh "$EVID" --offset-ms 15 > /dev/null &   # waits for the run
sleep 0.5; $S/press.sh "$EVID" Start
wait; $S/wait-state.sh "$EVID" phase summary 15
$S/last.sh "$EVID" run_end      # expect matched 8 of 8, bias_ms ≈ 15, passed true
$S/shot.sh "$EVID" summary
$S/doctor.sh "$EVID" && $S/cleanup.sh "$EVID"
```

## Evidence

Everything stays in `evidence/<name>/` (git-ignored): `data/trace.jsonl`
(ground truth), `data/state.json`, `data/profile.json`,
`data/history.jsonl`, `shots/*.png`, `play-N.json` (driver reports:
`"late":0` required), `app.log`, `build.txt`, `front.log`.

Proof standards:
- Drive the real user path (nav, buttons, keys). `--screen` is a test
  shortcut: say so if a proof used it.
- A screenshot alone never proves behaviour: pair it with the trace event or
  state value that proves the claim.
- Check side effects in a second place (history line, profile values, a
  relaunch with `--keep-data`).
- A step that cannot run (permission missing, precondition unmet) is reported
  as not verified, with the command and the reason. Never prove it through
  another path and call it verified.
- Synthetic keys prove the input and scoring path, not human timing or the
  feel of a real keyboard. Say that in every timing report.

## Cleanup

`cleanup.sh EVID` stops only the pid of that evidence dir and keeps the
evidence. Old evidence dirs go to the Trash, never `rm -rf`.

## Gotchas

- The AX tree is empty until the first AX query switches AccessKit on; the
  helpers wait up to 2 s for it.
- Start `play.sh` before pressing Start (or Again): it only answers runs
  that begin after it started. Its report must contain `"late":0`.
- egui shortcuts (Space, Escape) are not delivered in background mode: press
  the buttons instead.
- A rebuilt binary is not the launched one: doctor says so; relaunch before
  a proof.
- A cold run cannot happen on the day its lesson passed: use `--now` with a
  later day and `--keep-data` from the earlier run.

## Feature map

`features/README.md` lists every user-facing feature with its journey and
the date it was last proven. After any change to the app, update the map in
the same commit as the proof, and add a file when a feature is added.
