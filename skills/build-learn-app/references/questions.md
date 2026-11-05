# Questions

The grilling method: map the decisions as a tree, ask the whole frontier
each round, recommend an answer for every question, find facts yourself,
and do not build until the user confirms a shared understanding. The format
is the question tool, not numbered text.

## Rules

- AskUserQuestion: at most 4 questions per call, 2–4 options per question
  (the user can always type their own answer). Two calls back to back are
  fine when a round has more.
- Recommended option first, labelled "(Recommended)". Each option has one
  line: what it means and what it costs.
- Put the reason in the question: a research fact or a measured value
  ("Bluetooth keyboards add about 15 ms", "only 13 GiB free"). The user is
  often on a phone without the thread in view.
- Name the app or project in every question.
- Multi-select where choices combine (allowances, families, extras).
- A question whose answer depends on another open question waits for the
  next round.
- When the user answers in free text, treat it as the decision and log it
  verbatim. "Make a quick mockup so I can decide" and "open it on my laptop"
  are answers too: do it, then ask again.
- Budget: 12–20 questions in 4–6 calls. If you have more, you are asking for
  facts or for things you should decide yourself (stack, file layout,
  data formats: decide and state them for objection).

## Round 1: the allowances (two multi-select questions)

Ask once at the start so nothing blocks later. Two questions of at most 4
options each; adapt the options to the project and drop what does not apply.

**"<App>: what may I do on this Mac?"** (multi-select)
- **Drive the real app** — post key events only to the app and press its
  controls through the accessibility tree; in-app screenshots; macOS may ask
  once for Accessibility. Needed for verification.
- **Install free tools** — open-source tools inside the project (Rust
  targets, uv/.venv, Homebrew formulas); no purchases, no accounts.
- **Drive a host program** (DAW, plugin) — throwaway test documents only,
  never your projects; quit it after.
- **Remove build caches for good** — only agent build folders, when the disk
  runs low (moving them to the Trash frees no space).

**"<App>: what may I do outside this Mac?"** (multi-select)
- **Create a private git remote** — secret scan before each push.
- **Install on your practice Mac** — over ssh; the old app goes to that
  Mac's Trash; your data stays.
- **Use your login for research** — only if a public listing is incomplete;
  only that site's session; nothing posted, liked or followed.
- **Download free assets/media** — named sources only, licences recorded.

Notarization stays separate: ask right before it (it uploads to Apple).

## Round 1: questions research cannot answer

- **Audience** — Personal, share with friends (Recommended): local only;
  licensed assets stay outside the build; friends add their own. / Only me /
  Public release (only own or CC0 content, notarized, onboarding).
- **Time vs. goal** — e.g. "15–20 min a day; the goal wins, I estimate N
  weeks (Recommended)" / fixed weeks, cut scope / more minutes a day.
- **Start level** — Zero with a 5-minute placement (Recommended) / start at
  my level / pure zero.
- **Where you practise** — e.g. "the laptop's built-in keyboard (Recommended)
  / an external keyboard / both". Facts about the devices go in the question.

## Round 2: after research (examples from earlier builds)

Each came from a research file's "Decisions for the owner":
- the input mapping (fixed standard layout vs. remap; research: a fixed
  mapping builds automaticity);
- the course shape (skill stages with the learner's favourite genres first);
- the daily session shape (e.g. 3 min ear drill, 3 min knob match, 8 min
  main task, 1 min review);
- what "learned" means (a clean cold run on a later day);
- content sources (generators + templates + own material; real songs only by
  name and chord progression);
- the hardest tier (what is in, what is out of style);
- extras (auto dynamics, scale lock, MIDI export, tempo sync) and their
  licence consequences (Ableton Link is GPL);
- the app name (4 proposals with one line of meaning each).

## Design questions

Default: no question. State "the look is the bundled design system,
unchanged" as an assumption. If the user asks for options, build mockups in
the same system (same tokens, different layout or main widget), render them,
and open them where the user can see them. Never offer a new palette as the
recommended option.

## The shared-understanding summary (before the confirmation call)

Post it as a message, then ask "Is this the shared understanding for <App>,
so I write SPEC.md and build without stopping?" — "Yes, build it
(Recommended)" / "Change something first". Add open delivery permissions to
the same call.

```
<App>: shared understanding
1. Product — who, platform, offline, sharing, language.
2. Input — the mapping and why.
3. Content — sources, amount, licences, what is out of scope.
4. Learning — the daily session, what "learned" means, stages, pace estimate.
5. Feedback and progress — what is measured, no XP/streak loss.
6. Look — the design system, the main widget.
7. Verification — the verify skill, real input, background-safe.
8. Delivery — signed universal app, install target, notarize only after OK.
Only you can check: <feel on your keyboard, sound, the curve over days>.
Rule breaks so far: <none / what and why>.
```
