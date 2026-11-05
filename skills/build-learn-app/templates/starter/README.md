# Learnapp

<!-- First line: one sentence of purpose. Keep the sections below current. -->
A native macOS app that teaches <skill> in short daily sessions.

- Schedule: none. Nothing runs when the app is closed.
- Output: the learner's progress in `~/Library/Application Support/Learnapp/`.
- Turn off / remove: quit the app; move `Learnapp.app` and the data folder to the Trash.

## Run

```bash
cargo run --release -p learnapp-app --bin learnapp
cargo run --release -p learnapp-app --bin learnapp -- --data-dir /tmp/learnapp-try --trace
```

## Check

```bash
.claude/skills/verify-learnapp/scripts/gate.sh        # fmt + clippy + tests → gate: PASS / FAIL
```

Prove features in the running app with the project skill
`.claude/skills/verify-learnapp/` (see its `SKILL.md` and `features/`).

## Release

```bash
tools/make-icon.py assets/icon/icon_1024.png           # look at the PNG
tools/bundle-macos.sh                                  # dist/Learnapp.app, universal, signed
LEARNAPP_NOTARY_PROFILE=<profile> tools/notarize-macos.sh   # only with the owner's OK
tools/install-local.sh                                 # install into ~/Applications here
tools/install-remote.sh <ssh-host>                     # or on the practice Mac
```

## Layout

| Path | What |
|---|---|
| `SPEC.md` | every product decision and contract |
| `docs/grilling-log.md` | the questions and answers that made SPEC |
| `docs/research/` | research notes with sources |
| `crates/learnapp-core` | learner model, scheduler, scoring, storage |
| `crates/learnapp-app` | the app: UI, input, trace |
| `tools/learnapp-drive` | verification driver |
| `assets/fonts` | Geist (SIL OFL 1.1, `OFL.txt`) |

Fonts: Geist by Vercel, SIL Open Font License 1.1.
