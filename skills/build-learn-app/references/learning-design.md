# Learning design

Rules that held up in three builds (drums, keyboard improvisation, sound
design). Research for a new skill may change the numbers; keep the
principles unless the evidence says otherwise, and record why in SPEC.

## Retention decides "learned"

- An item counts as learned only after a **clean cold run on a later day**:
  no guide, no warm-up on that item, ideally in a new context (another key,
  tempo, sound). Same-session success is fluency, not storage.
- A failed cold run sends the item back to practice; it never resets the
  learner to zero. A revealed answer never counts as cold.
- Spaced reviews: a half-life model (recall = 2^(−gap/h), due below ~0.85),
  or fixed intervals 1-2-4-8-16-32 days. Early extra practice does not
  lengthen the interval. The starter's `schedule.rs` implements this.

## The daily session (15–20 minutes)

1. Warm-up from the weakest measured capacity (2–3 min).
2. Cold checks of items waiting for their gate.
3. Mixed review: 2–4 due items from different families, interleaved.
4. One new item (none when the learner struggles today).
5. A transfer or creation task (use it in your own 8 bars, recreate a
   target, make a variation).
6. Summary: what improved in measured terms, one thing for next time, a
   good stop. Offer the stop at 15 minutes; allow more.

"Done for today" appears only after a completed session, never after
leaving early. After the plan is done, extra practice becomes variations
(faster, no metronome, from memory) so the late course does not shrink to
five-minute sessions.

## A new item

Hear or see it first → copy it with help → the help fades (full → fade →
hidden; a hint ladder on request) → vary one thing → use it in a free task.
Blocked repetitions first, then interleave with confusable partners.
Attempt first, recipe after (productive failure); a worked example first only
for a truly new technique.

## Difficulty

- Keep success at ~75–90 %: adapt tempo or difficulty in small steps
  (±3 %), or 3-down-1-up staircases per axis, never two axes at once.
- Windows tighten by stage and are never tighter than 3× the learner's
  measured jitter.
- Stages unlock by demonstrated skill (e.g. 60 % of the previous stage
  learned), not by the calendar. A placement test can skip stages; skipped
  items get a short half-life and drop back on a failed check.
- Content is validated for playability (keys at once, fastest repeat per
  key, transitions between parts).

## Feedback

- During a run, mark only results outside the "good" window (bandwidth
  feedback); early and late in two distinct colours; never flash on the
  beat. Sound is the main time cue.
- After a run: measured facts (accuracy, bias in ms, spread in ms, misses,
  extras) and one next step in plain words.
- Cues name the result, not the body ("land the clap on 2 and 4", not "move
  your ring finger").
- Wrong answers are named honestly and briefly ("♭7, not 1"; "tail 220 ms
  too long"; at most two differences). A verdict never says "clean" when
  something was missed.
- For creative tasks: no creativity score. Show descriptive facts, a
  loudness-matched A/B, and let the learner predict their own rating before
  seeing the metric. Blind next-day self-rating with a hidden reference is a
  usable benchmark.

## Progress and motivation

- Show measured skill: timing spread, accuracy, keys or families covered,
  transfer to unseen material, time to start.
- No XP, no points for minutes, no badges. A weekly target (5 of 7 days) as
  a ring; a missed day never shows as a loss ("never miss twice").
- Motivation comes from the material itself (grooves, songs, fun target
  sounds) and from cheap choices (favourite genres first).
- Re-plan every 4 weeks and say it in one sentence.

## First start

A real win within 5 minutes: welcome → 2–3 favourites → a sound/input check
→ calibration if timing matters (tap along 16 clicks, median offset, MAD
outliers) → rollover check if chords matter → optional placement → the
first item. All in one consistent frame (same margins and title sizes).

## Timing (when the skill has timing)

- One clock for keys, audio and scoring (host ns). Score against calibrated
  time. Novices tap early by 20–80 ms: do not punish small negative bias
  early.
- Measure bias and spread per sound and over days; a weekly benchmark on a
  fixed easy item shows the trend.
- Synthetic input proves the path, never human timing. Say so.

## Honesty checklist (each one was a real bug)

- "Done for today" only after a finished session.
- Leaving a finished lesson's summary still commits its result exactly once.
- Leaving a session ends it; a later screen never resumes a session the
  learner left.
- Counts on buttons match the data ("Rate 10 old takes" with 3 takes was wrong).
- No raw ids, seeds, file names or validation messages on screen.
- Error messages say exactly what happened to the learner's data.
- Simulate a learner over the whole course (fast, average, sloppy) and
  check session lengths and stage timing against SPEC; report gaps instead of
  tuning numbers to hit the target.
