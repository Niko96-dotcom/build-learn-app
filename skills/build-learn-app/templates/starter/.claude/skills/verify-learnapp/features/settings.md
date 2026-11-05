# Settings

Reduce motion, output level, the data folder, and a two-step reset that moves files to the Trash.

## Sub-features

- `st-motion`: `Reduce motion` switch, saved in the profile.
- `st-level`: `Level` slider (AXIncrement / AXDecrement steps of 5 %).
- `st-folder`: data folder path shown.
- `st-reset`: `Reset progress` → `Move to Trash` moves `profile.json` and `history.jsonl` to the Trash; `Keep it` cancels.

## How to get to it (user POV)

Top bar → `Settings`.

## Driving it with verify-learnapp

Preconditions: a launched instance.

1. `press.sh "$EVID" Settings`; `press.sh "$EVID" "Reduce motion"` → `settings` trace event with `reduce_motion: true`; `data/profile.json` has `"reduce_motion": true`.
2. `action.sh "$EVID" Level AXIncrement` → `settings` event with level 0.85; `data/profile.json` has `"level": 0.85`.
3. Reset: finish one run first, then `press.sh "$EVID" "Reset progress"`, `press.sh "$EVID" "Move to Trash"` → `reset` event lists the moved files; `runs` is 0; the files are in the Trash, not deleted.

## Gotchas

- Reset touches only the evidence data folder in verification runs (never the real profile).
