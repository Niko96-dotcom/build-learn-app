# Intake

The user may write one line ("I want to learn finger drumming") or a long
brain dump. Either way, fill the slots below. A slot filled by the request or
by a default is not asked again. Only real gaps become questions, and most
of those wait until research can make a recommendation.

## The standard brief (defaults)

These requirements came, word for word in spirit, from every one of the
three original requests. Treat them as the user's defaults unless the request
says otherwise.

- **Teach everyone, gradually**, with a steep but fair curve: a real win in
  the first five minutes, complex material at the end.
- **Learning science first.** Study how humans learn this kind of skill
  (motor, perceptual, generative, conceptual) and design for maximum
  efficiency per minute. At least one research agent is a learning
  researcher, and one is specific to the domain.
- **Short daily practice**: the fewest minutes a day that work, and as much
  as the learner wants on top.
- **What a practitioner must know**: research the canon of the field instead
  of inventing it; advanced chapters stay inside the field's style (for a pop
  app: no jazz polymeters).
- **Standard input mapping**: when the skill uses an instrument or a tool,
  the learner learns the standard mapping of that tool (for example
  Ableton's computer-keyboard layout), so the skill transfers.
- **Pure Rust, standalone native app** (eframe/egui), offline, no account,
  no cloud, no telemetry. If the skill needs a host program (a DAW, a
  plugin), the app works next to it; the user may lift the Rust rule.
- **Many agents**: use as many subagents and model families as help, cheap
  ones for bulk work.
- **A nice UI, no slop**: no cards, a distinctive typeface (Geist), negative
  space, calm animations, no cream colours, do not over-explain the UI. The
  bundled design system already meets this; see design-system.md.
- **Verification as a deliverable**: a project verification skill that drives
  the real app like a user, "completely tested with computer use", so later
  changes show what broke.
- **Grill before code**: ask until the project is clear, then build without
  stopping.
- **Stop only when** the app is complete, every feature works and is
  verified, and the release is installed.

## Slots

| Slot | Example (drums / keys / sound design) | If missing |
|---|---|---|
| Skill and why (mission) | "play drum grooves on the laptop keys" | ask in round 1, one line |
| Learner background | "pop producer, 10 years, clicks MIDI in" | ask (start level) |
| End state | "pretty complex pop stuff"; "as good as <creator>" | research proposes a benchmark |
| Daily time | "as few minutes as possible" | ask (minutes vs. weeks) |
| Input / tool | computer keys; Ableton + Serum | research the standard mapping |
| Content sources | "my sample library on drive X"; "his videos" | research; ask about licences |
| Reference app / design | "like Pocket, I like its colours and font" | bundled design system |
| Audience / sharing | "personal, share with friends" | ask (decides licensing) |
| Where they practise | laptop keyboard, second Mac | ask |
| Platforms | macOS | default: macOS universal |
| Allowances | drive the app, installs, logins, repo | the allowance bundle |
| Stop condition | "full working app, every feature, tested" | default above |

## Things to detect before asking

- **Leftover lines.** People reuse a prompt across apps. A line that does
  not fit the topic (a timing line in a sound-design request) is probably a
  leftover: ask once, with your reading as the recommended option, and
  "ignore it" as another option.
- **Open ideas.** "Maybe we change the mapping on purpose? You tell me" is a
  research question, not a decision. Research it; recommend with evidence.
- **Contradictions.** "Anyone from zero" vs. an answer "start at my level":
  log the conflict, follow the later answer, keep a cheap safety net (a
  placement test).
- **Hidden scope.** "Recreate any sound" in "a few weeks" at 15 min/day is
  about 10 hours of practice. Give an honest estimate early, revise it after
  research, and let the user choose "goal wins" or "date wins".
- **Named people and corpora.** A creator's videos or a course as the
  starting point: research uses them privately; the app teaches its own
  recipes and never ships their material.

## First message to the user

One short status line, then the first question call. Do not send a plan
essay. Example: "I found your earlier drum app and your sample library. Five
research agents start now; these four questions do not depend on them."
