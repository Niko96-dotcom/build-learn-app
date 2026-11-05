# Case studies

The three builds this skill comes from (November 2026). Personal paths,
machines and accounts are left out.

## Pocket — finger drumming on the computer keyboard

- **Request**: teach everyone to play drum grooves of all genres on the
  computer keys, steep curve, any BPM, timing that improves over time; pure
  Rust; use my sample library (copy, never change); grill me first; stop only
  at a fully verified app.
- **Questions**: 18 in 6 calls, answered from a phone in 45 minutes; 16 of 18
  recommendations taken. Research answered the user's open idea (vary the
  key mapping?) with "keep one fixed layout" (Ableton's computer keyboard at
  C1 = General MIDI).
- **Build**: SPEC with decisions D1–D18, a shared model crate and compiling
  stubs first; then parallel agents for the learning core, groove content
  (57 grooves, 6 stages, 25 genres), sample kits (copied byte-identical) and
  the audio engine (cpal, 64-frame buffers, ~8 ms key to sound).
- **Verification**: a native driver posted real key events and clicked AX
  labels; the app's trace proved timing (+15 ms played → +14.98 ms
  measured). Real-input runs found the late first event, the open panel
  eating keys, a crash on close mid-frame. A cross-vendor review found 8 real
  defects (a risk of overwriting the learner's profile among them).
- **Release**: universal, Developer ID, notarized after the user's OK the
  next morning; installed on the practice laptop with 22 kits.
- **Open for the user**: feel and rollover of the laptop keyboard, the
  sound (nobody heard it), real use over days.

## Homerow — pop keyboard improvisation on laptop keys

- **Request**: a producer who can replay pieces but not improvise wants to
  play riffs, chords, basslines and melodies on the fly on MacBook keys;
  "like Pocket's design"; "take earlier work as inspiration but make it
  better".
- **Questions**: 23 in 9 calls; 20 recommendations taken. Rejected: three
  agent-made UI directions and four palettes ("make it look like Pocket").
  The user chose the proven C++ Ableton Link wrapper over an unproven Rust
  port, which made the app GPL.
- **Learning design**: moves in scale-degree numbers; Hear → Copy →
  Transpose → Vary → Use; "known" only after a cold pass in a later session
  in a random key; generators + templates + own takes; a "Room" for timed
  creative briefs; a Jam mode that keeps ideas.
- **Build**: six crates in waves of parallel agents across two vendors; a
  usage limit and a full disk (twice) interrupted it; the fixes became the
  shared-build-folder and disk-guard rules.
- **Verification**: a background-safe driver (keys to the pid, AXPress,
  in-app screenshots, never frontmost: 0 of 206 samples) while the user
  worked in DAWs on the same Mac. ~45 defects found and fixed in the real
  app after 86 green tests.

## Partial — sound design with a DAW and a synth plugin

- **Request**: recreate any sound with stock DAW devices and one synth,
  starting from a creator's free short videos (analyse every second, audio
  and picture), maximum learning efficiency, few minutes a day, "like
  Pocket"; the Rust rule was lifted.
- **Questions**: 20 in 8 calls; an allowance bundle first ("so no blocker
  comes later"); research-independent questions while 291 videos were
  processed; a leftover template line withdrawn by the user.
- **Research**: scripted preprocessing (transcripts, keyframes, voice
  removal, spectrograms), one cheap agent per video after a pilot, a
  technique catalogue of 82 moves with stock-tool paths; the creator's
  media stays private and is never shipped.
- **Learning design**: a 15-minute day (ear drill, knob match on the hosted
  synth, an 8-minute recreation sprint in the DAW, a debrief); targets
  compared by a calibrated similarity score with at most two named
  differences; blind next-day self-rating; a benchmark of 12 unseen targets.
- **Status at extraction**: research, SPEC and five crates done (analysis
  engine, audio, learner model, an AUv2 host for the synth, DAW capture in
  progress); screens, driver, verify skill and release still to come.
