# INT-0027 — A lossless port path for git-based Animus_Sprint_Loops projects

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0027
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
A project that uses Git with Animus_Sprint_Loops (Book schema v2) moves to
Periapsis in one command without losing history or meaning, and keeps working
with Git-based teammates during the transition. The port:

1. imports Git history through the bridge
   ([INT-0025](INT-0025-git-bridge.md));
2. converts every `docs/intents/INT-NNNN-*.md` chapter into an intent record —
   untyped criteria lift to `necessary` `asserted` under the port rule of
   [INT-0002](INT-0002-typed-acceptance-criteria.md), and typed criteria keep
   their polarity, modality and details;
3. converts `docs/work/` ledgers into per-task event records and
   `docs/sprints/sN/` into orbit records, sprint N becoming orbit N
   ([INT-0026](INT-0026-sprint-loops-on-periapsis.md)), that keep their
   provenance links;
4. maps the remote profile's `base`/`work` branches onto `main` (the public
   phase) and `dev` (the derived integration view);
5. re-projects the Book as Markdown compatible with Book v2 for Git mirrors.

On the Sprint Loops side, the intent schema gains optional typed acceptance
criteria (the INT-0002 grammar), validated by `check-book.sh` when present, so
a git-based project can type its intents before it ports. That change lands in
the Animus_Sprint_Loops repository.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Importing the Animus_Sprint_Loops repository itself yields one intent record per chapter, one task record per ledger entry and one orbit record per `docs/sprints/sN/`.
  - Threshold: 0 dropped records or fields, shown by a count reconciliation report.
- **AC-2** `necessary` `empirical` — Re-projecting an imported Book produces Markdown that Sprint Loops' `check-book.sh` validates and that has an empty diff against the source after the documented normalisations.
- **AC-3** `necessary` `empirical` — Untyped criteria import as `necessary` `asserted`, and typed criteria keep polarity, modality and details.
  - Threshold: 100% over a fixture corpus covering every modality.
- **AC-4** `necessary` `asserted` — Animus_Sprint_Loops ships the optional typed-acceptance extension to its intent schema, and every existing Book still validates under it.
  - Signers: the Animus_Sprint_Loops maintainer.
- **AC-5** `supporting` `empirical` — A mixed team, one member on a Git + Sprint Loops clone and one on Periapsis, completes an orbit, a sprint on the Git side, with the bridge keeping both views in sync.

## Rationale
- The owner's projects already run Sprint Loops on Git (Animus_Sprint_Loops
  itself is past its 22nd sprint), so a lossless path is the difference between
  migrating and starting over.
- Sprint Loops' Book v2 is already the right shape — intents, work and sprints
  with derived phase — so the port is a projection change, not a
  reinterpretation (II §1; R4 §2.4).
- The conservative lift (untyped → `necessary` `asserted`) means nothing claims
  more strength after the port than it had before.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §1, R4 §2.4; the Animus_Sprint_Loops repository's Book v2 schema.

## Alternatives
- **A manual migration guide.** Error-prone at 20+ sprints of history and
  loses the guarantee of AC-1.
- **Typed intents only inside Periapsis.** Forces every project to retype its
  Book at migration time, without being able to validate the result
  beforehand.

## Consequences
- The Markdown Book becomes a format with two implementations — the Sprint
  Loops shell validator and the Periapsis importer — and they must agree on the
  grammar.
- The Sprint Loops change belongs to that repository's own sprint loop, not to
  this one.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; from the owner's sprint request and Master Report v6 II §1 / R4 §2.4).
- 2026-11-01: revised while `proposed`, on owner direction: sprints import as orbits ([INT-0026](INT-0026-sprint-loops-on-periapsis.md)). Step 3, AC-1 and AC-5 are reworded.
