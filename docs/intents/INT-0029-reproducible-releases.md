# INT-0029 — Reproducible, attested releases

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0029
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Every Periapsis release is reproducible — two independent builders using the
pinned toolchain, locked dependencies and normalised paths produce
bit-identical artifacts — and attested: each release ships a signed build
attestation recorded in Periapsis's own ledger. For users who want the
strongest assurance, a release may also be built with an mrustc-bootstrapped
toolchain.

Non-goal: "verifiable down to the compiler"; rustc's backend and the
independent bootstrap path are C++ (Biblio §5).

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Two independent builders (for example the Windows host and the Debian host Nighthawk, or two CI runners) produce bit-identical release artifacts for the same tag and target.
- **AC-2** `necessary` `empirical` — Every release has a signed build attestation recorded in Periapsis's ledger, verifiable with `praps`.
- **AC-3** `supporting` `empirical` — A release built with an mrustc-bootstrapped toolchain reproduces the official artifact.

## Rationale
- Rust programs are often reproducible by default with a version-matched
  toolchain and normalised build paths, while rustc's own reproducibility is a
  never-closing tracking issue and a full bootstrap through rustc 1.93 needs
  about 310 GB (Biblio §5).
- Self-verification: the tool that records provenance should carry its own
  (Biblio §5 recommendation 6; I.5 step 39).
- Sources: [Biblio report](../sprints/s0/sprint-research/biblio-report.md) §5;
  [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md) I.5 step 39.

## Alternatives
- **Signed but unreproducible releases.** A signature proves who built the
  binary, not that it matches the source.

## Consequences
- Release builds need path remapping and fixed environments on every platform
  in the INT-0004 matrix.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from the Biblio report §5 and Master Report v6 I.5 step 39).
- 2026-11-01: revised while `proposed`, on owner direction: the command is `praps`, short for periapsis, replacing `peri` ([INT-0003](INT-0003-engineering-baseline.md)). AC-2 is reworded for the new name only.
