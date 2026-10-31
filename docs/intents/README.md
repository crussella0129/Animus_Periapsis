# Project Intents

Intent chapters use stable `INT-NNNN` identifiers and move through `proposed`,
`planned`, `active`, `deferred`, `realized`, `superseded`, or `abandoned`.
See the installed Sprint Loops `schemas/intent.md` contract before authoring one.

## Typed acceptance criteria

Every acceptance criterion in this Book is typed with a **polarity** and a
**modality**. The normative grammar and its semantics live in
[INT-0002](INT-0002-typed-acceptance-criteria.md); the short form is:

```markdown
- **AC-3** `necessary` `formal` — <a checkable claim>
  - Obligation: `Periapsis.OpMerge.conv` (or `unstated`)
  - Proxy: <how it is checked until the formal obligation is proved>
```

- `necessary` criteria are falsifiers: one failing **refutes** the intent.
  `supporting` criteria are sufficiency evidence: one failing leaves the
  intent **undetermined**.
- Modalities, strongest first: `formal` > `empirical` > `preference` >
  `judged` > `asserted`. An intent's strength is its weakest satisfied
  criterion; a `formal` criterion checked only by its proxy counts as
  `empirical` until its obligation is kernel-checked.
- `AC-n` numbers are never reused. Removing, weakening or retyping a criterion
  is a revision recorded in the chapter's Transition history.

## Evidence tags

Rationale sections cite the Sprint 0 research artifacts by tag:

| Tag | Source |
|---|---|
| I, II, III | [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md) Parts I–III (synthesis, systems research, entropy model) |
| R1-E*n* | Part IV — Round 1 correctness assessment, experiment E*n* |
| R2-E*n* | Part V — Round 2 experiments (merge, storage, sync, real suites) |
| R3 | Part VI — Round 3 specification-entropy verdict and necessity audit |
| R4 | Part VII — Round 4 proof pairing (Lean 4) |
| R5 | Part VIII — Round 5 cheat-resistant proofs, intent type theory, RR pipeline |
| Biblio | [Biblio report](../sprints/s0/sprint-research/biblio-report.md) (agent identity, lineage, Rust auditability) |

The research artifacts are provenance. Where an artifact and a chapter
disagree, the chapter is authoritative and its Transition history says why.
