---
name: build-learn-app
description: Build a complete, cool-looking native Rust learning app for macOS that teaches any skill. The user describes what they want to learn and answers a few questions; the skill then runs research, planning, a parallel multi-agent build, verification in the real running app, and a signed release. Use when someone wants an app to learn, practise or train a skill ("I want to learn X", "build me a learning/teaching/practice app", "make a trainer for drums/keys/typing/ear training/sound design/languages"), or types /build-learn-app.
argument-hint: "What do you want to learn?"
---

# Build a learning app

You turn one sentence ("I want to learn X") into a finished, verified,
signed macOS app that teaches X in short daily sessions. The method comes
from three real builds (a finger-drumming trainer, a keyboard improvisation
trainer, a sound-design course). Everything that went wrong in them is a
rule here, so the next build does not repeat it.

What the user gets:
- a git project with research notes, a grilling log and `SPEC.md`;
- a pure-Rust app (eframe/egui) in a calm, dark, typographic design;
- a project skill `.claude/skills/verify-<app>/` that drives the real app
  like a learner and maps every feature, so later changes can be checked;
- a universal, signed `.app`, installed where the learner practises;
- a final report that separates what is proven from what only the user can judge.

Stop only when Phase 9 is done. Pause only for a decision that is genuinely
the user's; finish all independent work first.

## Ground rules

1. **The user's own instructions win.** Read their CLAUDE.md / AGENTS.md and
   house rules (where files go, deletion rules, language) before acting.
   When this skill and the user's rules conflict, say so in one line and
   follow the user.
2. **Questions go through the question tool** (AskUserQuestion): at most 4
   questions per call and 2–4 options per question, the recommended option
   first and labelled "(Recommended)", one line of trade-off per option, the app's name in every question so it reads
   alone on a phone. Never ask decisions in plain text: it looks like a
   finished turn. Details: [references/questions.md](references/questions.md).
3. **Facts are your job, decisions are the user's.** Never ask for something
   you can look up, measure or have an agent research.
4. **Never delete; never touch user data.** Old files go to the Trash. Tests
   and verification use their own data folders. Read-only access to the
   user's libraries; resolve symlinks before any agent reads a folder.
5. **Outward actions need a yes**: notarization, pushes, public repos,
   installs on another Mac, logins, downloads of someone's media. Collect
   these as one allowance question at the start; ask again before anything
   not covered.
6. **Report what is proven.** Count files and run commands; do not trust an
   agent's "done". Every report has three lists: proven (with numbers and
   evidence paths), not proven, needs the user.
7. **Design starts finished.** Use the bundled design system unchanged
   ([references/design-system.md](references/design-system.md)), or the
   user's own reference app if they name one. Do not commission "bold new
   directions": users rejected every agent-made palette as "vibe coded".

## Phases

| # | Phase | Output | Gate |
|---|---|---|---|
| 0 | Intake and machine check | intake notes, project folder, `check-env: READY` | — |
| 1 | Allowances + first questions; research starts | first answers, research agents and selftest running | — |
| 2 | Research | `docs/research/*.md`, each with "Decisions for the owner" | files counted |
| 3 | Grilling rounds + confirmation | `docs/grilling-log.md`, user said "build it" | confirmed |
| 4 | Scaffold, SPEC, contracts | workspace from the starter, `SPEC.md`, stubs | `selftest: PASS`, `gate: PASS`, commit |
| 5 | Build waves (parallel agents) | crates, content, screens | `gate: PASS` per wave |
| 6 | Verify in the real app, fix, re-drive | feature map proven, `defects.md` closed | every journey proven |
| 7 | Independent review | findings fixed and re-proven | no open real defect |
| 8 | Release | signed universal `.app`, installed | bundle from final commit |
| 9 | Final report | proven / not proven / user's acceptance list | — |

Typical effort: 3–8 hours of wall clock, mostly agents in parallel.

### Phase 0 — Intake and machine check

1. Read the request against the intake checklist in
   [references/intake.md](references/intake.md). Fill every slot you can
   from the request and the standard defaults; only real gaps become
   questions. Flag lines that do not fit the topic (people reuse prompts; a
   "timing" line in a sound-design request was a leftover).
2. Look for context you can use without asking: the user's rules files,
   earlier apps built with this skill (a `theme.rs` with `BG #0A0A0B` and
   `ACCENT #FF4D2E`), their asset libraries (read-only), tools they own.
3. Run `scripts/check-env.sh` (read-only, shows no dialogs).
4. Create the project folder where the user's rules say (else
   `~/Developer/<topic>-learn-app`), `git init`, and write a `.gitignore`
   with `target/`, `dist/`, `evidence/`, `research/raw/`, `.venv/`,
   `__pycache__/`, `.DS_Store` before the first file lands. Name the folder
   after the topic: the app gets its name later in the grilling (the drums
   folder was `fingerdrum learn app`, the app "Pocket"). Nothing goes in
   Home, Desktop or Downloads.

### Phase 1 — Allowances, first questions, research starts

Round 1 is two question calls (see the bank in
[references/questions.md](references/questions.md)):
1. the **allowances** as two multi-select questions (on this Mac /
   outward), plus who the app is for and where the learner practises;
2. daily minutes vs. goal, start level, and any leftover line from the
   request.

In the same turn, start the research agents (Phase 2) in parallel. Do not
make the user wait for research to ask what research cannot answer.

When the user allowed driving the real app, start `scripts/selftest.sh
<project-dir>/target/selftest` in the background. It builds a throwaway app
from the starter, drives it in the real window server (keys only to its own
pid, never in front) and bundles it. Its `selftest: PASS` proves Rust,
permissions and the driver on this Mac; it is part of the Phase 4 gate. If
macOS asks for Accessibility, ask the user to allow it (an agent cannot
click the dialog).

### Phase 2 — Research

Run 4–6 research agents in parallel with the briefs in
[references/research.md](references/research.md): learning science for this
skill, domain content (what a practitioner must know, the end-state
benchmark), existing apps and their anti-patterns, technology (input
mapping, latency, Rust stack, licences), and sources/assets. Each writes one
file in `docs/research/` with evidence tags and ends with "Decisions for the
owner". If the user names a creator, course or corpus as the starting point,
add the corpus pipeline (private layer, never shipped).

When the user floats a pedagogical idea ("maybe change the key mapping on
purpose?"), have research test it; answer with evidence in the grilling.

### Phase 3 — Grilling rounds and confirmation

Merge the "Decisions for the owner" sections. Decide yourself where research
agrees; ask where it conflicts or where taste decides. Rounds of ≤ 4
questions, 12–20 questions in total. Users accepted the recommendation in
about 85 % of questions, so the recommendation must be the best answer with
its reason (a research fact) in the question text.

- Design: do not ask about palettes. The look is the bundled system. If the
  user wants to see options, render them **in that system** and open them on
  the device they are holding.
- Name the things only the user can accept (feel on their keyboard, sound,
  the curve for a real beginner) and put them in SPEC now.
- End with one confirmation call: the shared-understanding summary (format in
  [references/questions.md](references/questions.md)) plus any delivery
  permission still open (install target, notarization later).

Log every round in `docs/grilling-log.md` as you go.

### Phase 4 — Scaffold, SPEC, contracts

1. `scripts/new-app.sh <project-dir> <slug> "<Display Name>" <bundle-prefix>`
   copies [templates/starter](templates/starter) into the project folder
   (it keeps the existing `docs/`, `research/` and `.gitignore` entries
   untouched): a compiling workspace with
   the design system, Geist fonts, key input with hardware timestamps, trace
   and state files, safe storage, a review scheduler, honest scoring, a
   background-safe driver, the verify skill, release scripts and an icon
   renderer. Read [references/architecture.md](references/architecture.md).
2. Write `SPEC.md` from the grilling log and the research (template in the
   starter). Every decision gets an id (D1…). SPEC is the only brief build
   agents get.
3. Write the contracts before any agent starts: shared data types in the
   core crate, compiling stubs (`todo!()`) for every crate an agent will
   fill, and the dependency rule (crates built in parallel never depend on
   each other).
4. `selftest.sh` printed `selftest: PASS` and `gate.sh` prints
   `gate: PASS`. Commit.

### Phase 5 — Build waves

Follow [references/build-waves.md](references/build-waves.md). In short:
waves of parallel agents, each owning one crate or folder; one shared build
folder; no agent runs state-changing git; the lead runs the gate, writes
agent deviations back into SPEC, and commits. Check every agent's output
objectively (remaining `todo!()`, tests, files) before relying on it. Build
the verification driver and the first feature-map entries with the app
shell in wave 1, not at the end.

Content rules: content is data (TOML) validated by tests; generators are
seeded; anything an agent wrote from memory (songs, facts, values) is
checked by a separate fact-check agent with sources. Learning rules: see
[references/learning-design.md](references/learning-design.md).

### Phase 6 — Verify in the real app

Unit tests and headless tests passed in every build, and the real app still
had 20–45 defects each time. Plan for a long drive → fix → re-drive loop.
Follow [references/verification.md](references/verification.md):

- Extend `.claude/skills/verify-<slug>/` with helpers and one feature file
  per user-facing feature; drive every journey in the list there (first
  start, later days with the test clock, every exercise with a pass and a
  fail, every exit path, settings, persistence, error states).
- The driver is background-safe: keys only to the app's pid, AXPress for
  controls, in-app screenshots, no focus steal. The user may be working on
  the same Mac.
- Keep `docs/verification/defects.md` with ids. Each fix gets a regression
  test that fails without it, then the journey is driven again.
- Review every screenshot against the design rules: overlaps at the
  smallest window size, raw ids or developer text on screen, numbers that do
  not match the data, dishonest verdicts.

### Phase 7 — Independent review

Give a read-only reviewer (another model or vendor if available) the repo
and this brief: "Find real defects only, ranked by severity, each with
file:line, a concrete failure scenario and a minimal fix; at most 20; then
list what you checked and found sound." Focus: timing path, real-time audio
safety, state machines (finish, leave, resume), persistence and data safety.
Fix each finding with a regression test and re-prove it in the running app.

### Phase 8 — Release

Follow [references/release.md](references/release.md): render and look at
the icon, bundle from the final commit (`tools/bundle-macos.sh`), launch the
bundle once in background mode with a test data folder, install where the
learner practises: this Mac (`tools/install-local.sh`) or another Mac over
ssh (`tools/install-remote.sh`, if allowed). Notarization uploads the app to Apple: ask
right before, with the reason (friends see a warning otherwise).

### Phase 9 — Final report

Use [references/final-report.md](references/final-report.md). Three lists:
proven (numbers, evidence paths), not proven, needs the user (with the
artefacts prepared for them: installed app, audio previews). Then how to
start, how to verify after changes (`/verify-<slug>`), and open items.

## Resuming after an interruption

Read, in order: `docs/grilling-log.md`, `SPEC.md`, `git log --oneline`,
`docs/verification/defects.md`, the verify skill's `features/README.md`.
Check running agents by their output files. Continue at the first phase
whose gate is not met.

## Files in this skill

- [references/intake.md](references/intake.md) — the standard brief and intake checklist
- [references/questions.md](references/questions.md) — question rules, allowance bundle, question bank, confirmation summary
- [references/research.md](references/research.md) — research agent briefs, corpus pipeline, evidence tags
- [references/learning-design.md](references/learning-design.md) — learning rules that worked
- [references/architecture.md](references/architecture.md) — workspace, contracts, time base, audio, storage, trace
- [references/design-system.md](references/design-system.md) — tokens, components, layout, motion, copy, anti-slop
- [references/build-waves.md](references/build-waves.md) — parallel agents, briefs, file ownership, commits
- [references/verification.md](references/verification.md) — verify skill, driver, journeys, defect loop
- [references/release.md](references/release.md) — bundle, sign, notarize, install, licences
- [references/lessons.md](references/lessons.md) — every pitfall from the three builds, as rules
- [references/final-report.md](references/final-report.md) — report template
- [references/case-studies.md](references/case-studies.md) — the three builds in short
- `templates/starter/` — the compiling workspace; `scripts/` — `new-app.sh`, `check-env.sh`, `selftest.sh`
