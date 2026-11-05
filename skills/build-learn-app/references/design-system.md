# Design system

The look of all three apps, and the reason it is not up for debate by
default: when an agent proposed three "bold" directions and four palettes,
the user rejected all of them ("I hate the other ones, they look incredibly
vibe coded") and asked for the first app's design unchanged. Since then every
app started from it, and the only design work left was usage: layout, the
main widget, copy.

The code is in the starter: `crates/<slug>-app/src/theme.rs` (tokens) and
`widgets.rs` (hand-drawn controls). Change usage, not tokens. If the user
names their own reference app, port its theme instead and keep the same
rules.

## Principles

- Dark, calm, typographic. **No cards**: no boxes, panels, shadows, glows,
  gradients. Structure comes from type size and weight, alignment, 1 px
  hairlines and space.
- **One accent** (`#FF4D2E`): the playhead, the one primary action per
  screen, "live". Everything else is ink on near-black.
- **Negative space**: 72 px side gutters, generous vertical rhythm, content
  left-aligned.
- **The subject is the main widget**: a keyboard, a timing lane, a piano
  roll folded to the key, a spectrogram with an A/B loop. Not a dashboard.
- **Do not over-explain**: one meta line, one title, one cue line. Labels
  name the thing; no tutorials in the chrome.
- **Sound is the time cue**; visuals move continuously and never flash on
  the beat.

## Tokens

| Token | Hex | Use |
|---|---|---|
| BG | `#0A0A0B` | window background |
| WASH | `#131315` | hover wash of rows; never a filled panel |
| LINE | `#1D1D21` | hairlines, beat lines |
| LINE_STRONG | `#2B2B31` | bar lines, outlines |
| INK | `#F2F2F3` | primary text |
| INK_2 | `#A4A4AC` | secondary text |
| INK_3 | `#62626A` | labels, targets |
| INK_4 | `#38383E` | disabled, faint marks |
| ACCENT | `#FF4D2E` (hover `#FF6E52`) | signal colour |
| EARLY | `#5CA4FF` | early (timing) |
| LATE | `#FFB547` | late (timing) |
| GOOD | `#9EF07A` | passed / tight |

Type: Geist (Light/Regular/Medium/SemiBold), Geist Mono, Geist Pixel
(Square/Circle) for big numerals; SIL OFL 1.1, the licence text ships in the
bundle. Scale: hero 52 SemiBold, display 44 SemiBold, title 26 SemiBold,
heading 17 Medium, body 15, small 13, label 11 Geist Mono UPPERCASE with
1.2 tracking, mono 13, readouts in Geist Pixel at any size.

Spacing: 4, 8, 12, 16, 24, 32, 48, 64, 96. Gutter 72. Top bar 64.
Window 1440×900 default, 1080×720 minimum, title bar hidden (traffic lights
float over the content), corner radius 0, no shadows.

Motion: hover 0.12 s, press 0.06 s, choice dot 0.16 s with `ease_out_back`,
screen enter 0.32 s `ease_out_cubic` (10 px rise + fade), segmented
underline slides 0.22 s, key glow decays with a 0.09 s half-life, the
playhead is never eased. A "Reduce motion" setting turns enter animations
and extras off. Repaint continuously only while something moves.

## Components (`widgets.rs`)

`primary_button` (the one accent pill per screen, 44 px, with its key hint
"Start SPACE"), `text_button` (quiet, underline grows on hover),
`outline_button`, `choice` (a word with a dot), `segmented` (options with a
sliding underline), `slider` (1 px track, AX increment/decrement/set-value),
`switch`, `row` (full-width hover wash), `rule` (hairline), `caps` / `caps_at`
(mono eyebrow labels), `readout` (Pixel numeral + caps unit), `keycap` and
`key_chip` (laptop keys with press glow and countdown fill). Every
interactive widget reports an accessibility label equal to its visible text:
the verification driver finds controls by it.

## Layout patterns

- Top bar: lowercase wordmark + accent dot on the left; nav words on the
  right with a sliding accent underline under the active one.
- A screen: mono caps meta line ("WARM-UP · STEADY PULSE · 8 TARGETS"), a
  big title, one cue line, then the main widget, then one action row.
- Today: a numbered typographic plan list on the left (01 WARM-UP Title …
  2 MIN), measured numbers on the right (big Pixel readouts, a week ring).
- Results: three readouts (accuracy %, bias ms, spread ms) and one verdict
  sentence.
- Lists instead of popups: egui combo boxes were unreliable to drive; use
  inline, labelled choices ("Use <kit> for <genre>").
- Empty states are calm sentences ("No runs yet. Your first run shows up
  here."), never blank areas or zeros that look like data.

## Copy

Measured, plain, short. Numbers come from data. Name what happened to the
learner's data exactly. Singular/plural correct. No developer words (ids,
seeds, "candidates", raw validation text). One word for one meaning across
the app (a glossary in SPEC).

## Icon

`tools/make-icon.py` renders a 1024 px PNG with a transparent margin from
the same palette (stdlib Python, signed-distance anti-aliasing). Draw the
app's subject in a few marks (a 4×4 groove grid with one accent hit; an
octave of keycaps with the tonic in accent). No glow, no gradient. Do not
render SVG through Quick Look (white background). Look at the PNG at 1024
and 64 px before bundling.

## Review checklist for every screenshot

- Nothing overlaps or clips at 1080×720 and at 1440×900.
- One accent element per screen (plus live marks).
- No boxes, shadows or gradients crept in.
- Every number matches the trace or the profile.
- No raw ids, file names, seeds or debug text.
- Empty, error and long-content states look intentional.
