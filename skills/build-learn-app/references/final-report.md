# Final report

Short, factual, in the user's language. No progress ritual. The user may
read it on a phone.

```
<App> is built, verified and installed on <machine>.

Proven (in the running app, with evidence)
- <feature>: <journey> → <measured result> (evidence/<run>/…)
- Timing: driver played +15 ms → app measured +14.98 ms; spread 0.4 ms; late 0
  (synthetic input: proves the path, not human timing)
- Tests: <N> pass, clippy clean (gate: PASS at <sha>)
- Review: <model> found <n> real defects; all fixed with regression tests and re-proven
- Release: universal (arm64 + x86_64), Developer ID, installed at ~/Applications on <host>

Not proven
- <what could not run and why: e.g. MIDI to the DAW, pending while the DAW ran>
- <architectures only run under Rosetta>

Only you can check
- How it feels on your keyboard, and whether chords like A+S+T+O register
- How it sounds: <n> previews in <path>
- Whether the curve fits you over the next days (first cold check: tomorrow)

How to use
- Open <App> from ~/Applications. First start takes ~5 minutes.
- After changes: /verify-<slug> (feature map in .claude/skills/verify-<slug>/features)

Open
- Notarize for friends? (uploads the app to Apple) → asked below
```

Then ask any remaining outward decision (notarization, publishing) with the
question tool, recommended option first.
