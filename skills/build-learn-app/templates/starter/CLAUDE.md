# Learnapp — rules for agents

Read `SPEC.md` first. It is the contract. If the code and SPEC disagree,
stop and report; do not silently pick one.

## Build and commit

- One shared build folder for everyone: `target/shared` (`.cargo/config.toml`).
  Do not set a per-agent `CARGO_TARGET_DIR`: per-agent folders filled disks before.
- Before every commit: `.claude/skills/verify-learnapp/scripts/gate.sh && git commit …`
  (never `;` — a failed gate must stop the commit).
- Parallel agents edit only the files or crate they own. Only the lead commits.

## Verify

- Unit tests are not proof that the app works. Prove features in the real
  app with `.claude/skills/verify-learnapp` (background mode: it never takes
  focus, keys go only to the app's pid).
- Every bug found in the real app gets a regression test that fails without
  the fix. Re-drive the journey after the fix.
- Update `features/*.md` of the verify skill in the same commit as the proof.

## Design

- `theme.rs` and `widgets.rs` are the design system. Change usage, not tokens.
- No cards, boxes, shadows, glows or gradients. No cream colours. One accent.
  Structure comes from type, alignment, hairlines and space.
- Every interactive widget has an accessibility label equal to its visible
  text. The driver needs it.
- No ids, seeds or raw validation text on screen. Every number on screen is
  derived from data and tested.

## Data safety

- Tests and verification use `--data-dir`; never the learner's real folder.
- Never delete: move to the Trash. Never overwrite a profile that cannot be read.
- Outward actions (notarize, push, publish, install on another Mac) need the
  owner's explicit OK.
