# Today

The daily plan as a numbered list, the last timing spread, and the week ring.

## Sub-features

- `today-plan`: three rows (warm-up, review with the due count, new).
- `today-spread`: spread of the last run, "—" before the first run.
- `today-ring`: practice days of the last seven days; "N of 5 days".
- `today-done`: "Done for today." only after a finished run today (not after Leave session).
- `today-note`: "Not saving" note when the profile cannot be read.

## How to get to it (user POV)

Start the app; Today opens first. Or press `Today` in the top bar.

## Driving it with verify-learnapp

Preconditions: a launched instance (`EVID`).

1. `press.sh "$EVID" Today` → `wait-state.sh "$EVID" screen today`.
2. Before any run: `plutil -extract done_today raw -o - "$EVID/data/state.json"` prints `false`; `shot.sh "$EVID" today-fresh`.
3. After a finished run (see practice.md): `done_today` is `true`, `days_this_week` is `1`; `shot.sh "$EVID" today-done` shows "Done for today.".
4. Leave a run early (`Leave session`) on a fresh profile: `done_today` stays `false`.

## Gotchas

- `--now` changes "today" only with `--trace` and `--data-dir` (launch.sh sets both).
