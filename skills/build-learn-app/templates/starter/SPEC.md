# Learnapp — specification

This file holds every product decision (from `docs/grilling-log.md`) and the
contracts between the crates. It is the only brief build agents need. Change
it only together with the code that it describes. When a build agent reports
a mismatch, the lead updates this file in the same commit.

<!-- Fill every section from the grilling log and docs/research/. Delete a
section only when it cannot apply (e.g. no timing in a reading app), and say
why in one line. No placeholder may stay when the build starts. -->

## 1. Product decisions

| # | Decision | Source |
|---|---|---|
| D1 | Who it is for, and who may get a copy (personal / friends / public) | grilling Q… |
| D2 | Platform: native macOS app in pure Rust (eframe/egui), offline, no account, no cloud, no telemetry | template |
| D3 | Daily time and pace target (e.g. 15–20 min/day; first win in 5 minutes; end state in N weeks) | grilling Q… |
| D4 | Input / instrument (which keys, which standard mapping the learner learns) | research + grilling |
| D5 | Content sources and their licences (own content, generators, licensed packs kept outside git) | research |
| D6 | "Learned" means: a clean cold run on a later day (never same-session fluency) | learning research |
| D7 | Look: the design system in `theme.rs`/`widgets.rs`, unchanged | template |
| D8 | Delivery: signed universal `.app`; notarization only after the owner's OK; install on the practice Mac | grilling Q… |
| D9 | Done means: section 10 | template |

## 2. The learner and the goal

- Mission (one sentence in the learner's words):
- Start level and placement:
- End state (a benchmark a person can pass or fail, not a feeling):
- Honest time estimate and what the learner chose when goal and time conflict:

## 3. Input and mapping

Physical key codes (positions), labels from the active layout. One fixed
mapping for the whole course (consistent mapping builds automaticity).
Playable limits (keys at once, fastest repeat) and how content is validated
against them.

## 4. Content

Formats (TOML per item under `content/`, embedded at build time), folder
layout, validation rules (each rule is a test), and the provenance of every
item. Facts written from model memory are checked by a separate research
agent with sources before they ship.

## 5. Learning design

### 5.1 Unit of practice
### 5.2 Steps of a new item (e.g. hear → copy → vary → use)
### 5.3 Stages and gates (measured thresholds per stage)
### 5.4 Scheduler (retention: cold runs on a later day, half-life review, interleaving)
### 5.5 Daily session (warm-up, cold checks, mixed review, one new item, good stop)
### 5.6 Placement
### 5.7 Exercise types (each has a pass and a fail fixture test)
### 5.8 Scoring (exact matching rules and windows)
### 5.9 Feedback (bandwidth feedback during a run, summary after, cues name the result)
### 5.10 Progress and motivation (measured skill, weekly target, no XP, no streak loss)

## 6. Sound and media

Engine rules (no allocation, locks or I/O on the audio thread), sources,
levels. What nobody has listened to yet goes on the acceptance list.

## 7. Time base

One clock for input, audio and scoring (`mach_absolute_time` ns on macOS).
Latency calibration, if timing matters.

## 8. UI

### 8.1 Design system — `theme.rs` and `widgets.rs`, unchanged
### 8.2 The main widget (the subject itself, e.g. a keyboard, a lane, a spectrogram)
### 8.3 Screens (first start, Today, the practice surface, Library, Progress, Settings)

## 9. Architecture

| Crate / tool | Owns | Depends on |
|---|---|---|
| `learnapp-core` | learner model, scheduler, scoring, storage | — |
| `learnapp-app` | UI, input, trace, platform | core (+ audio, content) |
| `tools/learnapp-drive` | verification driver | — |

Dependency rule: core never depends on the app, audio or content crates
built in parallel; parallel agents never share a crate.

Data folder `~/Library/Application Support/Learnapp/` (override
`--data-dir` / `LEARNAPP_DATA_DIR`): `profile.json` (atomic writes; never
overwritten when unreadable), `history.jsonl`. With `--trace`:
`trace.jsonl` + `state.json` (the verification interface). Every
user-visible output path has an env override so tests never write into the
learner's folders.

## 10. Verification and done

Done means all of:
1. `.claude/skills/verify-learnapp/scripts/gate.sh` prints `gate: PASS`.
2. The app starts from an empty data folder and reaches its first win.
3. `verify-learnapp` drives every feature in its map in the real app, with
   evidence, and the map says when each was proven.
4. An independent read-only review found no open real defect.
5. A signed `.app` is built from the final commit and installed where the
   learner practises.

Only the owner can accept (agents cannot prove these):
- how it feels on the real keyboard / instrument and its key rollover;
- how it sounds (preview renders are prepared);
- whether the curve fits a real beginner over days.

## 11. Open facts to check during the build
