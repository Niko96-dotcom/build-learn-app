# Progress

Measured skill only: runs, practice days, best spread, spread per run, the last runs as a list.

## Sub-features

- `pg-empty`: "No runs yet." before the first run.
- `pg-readouts`: runs, practice days, best spread.
- `pg-chart`: spread per run, the last point in the accent colour.
- `pg-list`: the last eight runs with accuracy and spread.

## How to get to it (user POV)

Top bar → `Progress`.

## Driving it with verify-learnapp

Preconditions: a launched instance; for the filled state, at least two finished runs (see practice.md).

1. Fresh profile: `press.sh "$EVID" Progress`, `shot.sh "$EVID" progress-empty` shows "No runs yet.".
2. After two runs: `shot.sh "$EVID" progress-two-runs`; compare the numbers with `data/history.jsonl`.

## Gotchas

- Progress reads `history.jsonl`; a read-only store shows the old history and saves nothing.
