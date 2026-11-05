# Research

Research runs in parallel with the first questions and decides most of the
later ones. Every agent writes one file in `docs/research/`, tags every claim,
cites sources, and ends with "Decisions for the owner" (options, recommended
first, one line each). Raw downloads go to `research/raw/` (git-ignored).

## Evidence tags

[S] strong (replicated studies, official docs) · [M] moderate · [W] weak ·
[P] practitioner opinion · [L] measured locally (command + number) ·
[I] inference. A recommendation that rests on [W] or [I] says so.

## Standard agents

Start these four or five in one message. Use a strong model for learning
science and domain synthesis; a cheaper one for inventories.

### 1. Learning science for this skill → `learning-science.md`

> How do humans learn <skill> (motor / perceptual / generative /
> conceptual parts)? Give design rules as a table "Rule | Mechanic in the
> app | Evidence", with citations. Cover: retention vs. in-session fluency
> (gates on later-day performance), spacing and session length, blocked
> then interleaved practice, the challenge point (~75–90 % success),
> feedback (bandwidth, summary, fading guidance), retrieval from memory,
> part-whole practice, external focus cues, motivation without points
> (measured skill, learner choice), and what the evidence does NOT support
> (list weak or conflicting findings and the decision you would take).
> Answer the user's open ideas: <ideas>. Estimate the time to the end state
> at <minutes>/day and mark it as opinion. End with Decisions for the owner.

### 2. Domain content → `domain.md`

> What should a <practitioner> know and be able to do, in order? Build a
> skill inventory, frequency data where it exists, the canonical material
> (generic names, own transcriptions; no copied third-party files), an
> end-state benchmark a person can pass or fail (e.g. "random brief → 8-bar
> part in 60 s"), a stage skeleton, and an explicit out-of-scope list.
> Note legal limits for content (what may be transcribed, what not).

### 3. Existing apps and pedagogy → `apps-and-pedagogy.md`

> Benchmark the apps and courses that teach <skill>: what each does well,
> and the anti-patterns (scores that measure time spent, streak loss,
> theory before doing, "improvise" only at the end, one opaque mastery
> score, raising tempo as the only difficulty, the tool doing the learner's
> work). List exercise types worth copying.

### 4. Technology → `tech.md`

> Input: the standard mapping of <tool> (and how it differs across tools),
> physical key codes vs. layouts (QWERTZ/AZERTY), rollover limits, input
> latency and timestamps on macOS (NSEvent local monitor needs no
> permission). Output: audio stack for Rust (cpal + own engine; measure the
> buffer latency on this Mac), synthesis vs. samples. GUI: confirm eframe/egui
> with AccessKit (needed by the verification driver). Licences of every
> dependency that would change the app's licence (e.g. Ableton Link is GPL).
> Measure, do not guess: [L] numbers with the command used.

### 5. Sources and assets → `sources.md` (when the skill needs media)

> Inventory the user's libraries (read-only; resolve symlinks first; never
> write there) and free sources (CC0, licensed bundles). Record the licence
> of each: what may ship inside the app, what may never leave it as a file,
> what must stay out of git.

## Fact checks

Anything a content agent writes from memory (song progressions, values,
historical facts) is checked by a separate agent with sources. In one build
7 of 48 progressions were wrong.

## Corpus pipeline (when the user names a creator or course)

Only with the user's allowance, and only for personal study.

1. Find the real account (handles differ across platforms; check followers,
   bio, cross-platform matches). List the catalogue; ask once if a login is
   needed, and use only that site's session.
2. Preprocess with scripts before any agent looks: transcripts with word
   times, contact sheets, keyframes kept on screen change, voice removal,
   spectrograms, level/pitch/onset features. Resumable, skip finished steps.
3. Pilot the analysis brief on 1–2 items, read the output, tighten the brief.
4. Fan out with cheap agents: one agent per video for deep items (agents
   lose older images after ~100 per conversation), ≤ 70 frames chosen by
   change, the sharpest source copy, notes written to disk every ~10 frames,
   a `keyframes_read / keyframes_total` count in the output.
5. Count output files; replace hung agents by checking for missing files,
   not by waiting.
6. Synthesise with a strong model into a technique catalogue that explains
   *why* each move works (checked against the field, not only the creator's
   words) and gives a path with the user's own tools.
7. Privacy: media, frames, transcripts and reference clips stay in
   `research/raw/` and the app's private data folder, never in git, never
   shipped. Quote at most ~15 words; paraphrase the rest. Commit only
   summaries without media.

## Done when

All files exist, each has a Decisions section, conflicts between files are
listed, and you counted the files yourself.
