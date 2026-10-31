# INT-0025 — Git bridge: import, export and mirroring

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0025
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Periapsis imports Git repositories (full history, branches and tags) and
exports public history to Git for mirroring on GitHub, Codeberg and other
forges. The bridge is written against the Git object-format specification, not
on top of a Git library, and maintains the SHA-1/SHA-256 ↔ BLAKE3 mapping as
derived data. Periapsis-only metadata (change IDs, intent references,
signatures, confirmations) travels through a documented mapping onto commit
trailers or notes. The bridge ships before any public release.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Import followed by export reproduces identical Git commit and tree object ids for at least three real repositories, including this one.
- **AC-2** `necessary` `empirical` — Incrementally exported public changes form Git commits that stock `git fsck` accepts and a forge accepts on push.
- **AC-3** `necessary` `asserted` — The bridge is released before any public release of Periapsis.
  - Signers: project owner.
- **AC-4** `supporting` `empirical` — Change IDs, intent references and signatures survive a Periapsis → Git → Periapsis round trip through the documented trailer/notes mapping.

## Rationale
- Adoption without Git is the main non-technical risk; a Git bridge before any
  public release is the mitigation (I.6, II §5).
- Writing it against the format spec keeps the from-scratch rule and the GPL
  licence position clean (I.1, II §4 Phase 4 step 29).
- Jujutsu's change IDs live in non-standard Git headers that plain Git drops on
  rebase, so the mapping must use durable carriers such as trailers or notes
  (Biblio §2).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  I.6, II §4 Phase 4; [Biblio report](../sprints/s0/sprint-research/biblio-report.md) §2.

## Alternatives
- **gitoxide or libgit2 as the Git layer.** Faster to build, but breaks the
  from-scratch rule and, for libgit2, adds C.
- **No bridge.** Rejected: blocks adoption and the INT-0027 port path.

## Consequences
- Git's hash algorithms must be implemented or depended on for the bridge only
  (SHA-1 and SHA-256 are needed to reproduce Git ids).
- The ↔ mapping must be rebuilt deterministically from both histories, which
  makes it derived data under INT-0013.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 33 and II §4 Phase 4).
