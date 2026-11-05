# Practice

One scored run: a count-in, eight targets on F and J, then a summary with accuracy, bias, spread and an honest verdict.

## Sub-features

- `pr-start`: `Start` begins a count-in; `transport` trace event with `zero_host_ns` and targets.
- `pr-score`: hits are matched one-to-one by key and time; bias and spread in ms.
- `pr-verdict`: the verdict never hides a miss or an extra hit.
- `pr-leave`: `Leave session` ends the run; nothing is scored, no practice day.
- `pr-persist`: a finished run writes `profile.json` and one `history.jsonl` line.

## How to get to it (user POV)

Today → `Start session`, or the top bar → `Practice` → `Start`.

## Driving it with verify-learnapp

Preconditions: a launched instance (`EVID`).

1. `press.sh "$EVID" Practice`; `wait-state.sh "$EVID" screen practice`.
2. On time with an offset: `play.sh "$EVID" --offset-ms 15 &`, then `press.sh "$EVID" Start`, `wait`, `wait-state.sh "$EVID" phase summary 15`.
   Expect `last.sh "$EVID" run_end`: `matched` 8, `targets` 8, `bias_ms` 15 ± 2, `spread_ms` ≤ 2, `passed` true; play report `late` 0.
3. Misses: `play.sh "$EVID" --skip-every 4 &`, `press.sh "$EVID" Again`, `wait` → `missed` 2, verdict starts "6 of 8 in time, 2 missed".
4. Extras: `play.sh "$EVID" --extra-every 4 &`, `press.sh "$EVID" Again`, `wait` → `extras` 2, verdict names "2 extra hits".
5. Leave: from the summary press `Again` (from a fresh Practice screen: `Start`), `wait-state.sh "$EVID" phase count_in 5`, then `press.sh "$EVID" "Leave session"` → `run_left` event, screen `today`, `runs` unchanged.
6. Persistence: `data/history.jsonl` has one line per finished run; relaunch with `--keep-data "$EVID/data"` shows the same `runs`.

## Gotchas

- Synthetic keys prove capture and scoring, not human timing.
- A screenshot during a timed run can delay one event: take shots before and after runs.
