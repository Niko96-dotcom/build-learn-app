# Research

One file per research question, written by the research agents. Every file:

- states the question in its first line;
- tags each claim with its evidence strength: [S] strong (replicated
  studies, official docs), [M] moderate, [W] weak, [P] practitioner opinion,
  [L] measured locally (command and number), [I] inference;
- gives sources (links or local paths) for every fact;
- lists conflicts with other research and the decision taken;
- ends with "Decisions for the owner": options, recommended first, one line
  of trade-off each. These become the grilling questions.

Typical files: `learning-science.md`, `domain.md` (what a practitioner must
know, canonical material, end-state benchmark), `apps-and-pedagogy.md`
(existing apps, anti-patterns), `tech.md` (input, audio, Rust stack,
measured latency), `sources-and-licences.md`.

Raw material (downloads, transcripts, media) goes to `research/raw/`
(git-ignored), never into git and never into the app.
