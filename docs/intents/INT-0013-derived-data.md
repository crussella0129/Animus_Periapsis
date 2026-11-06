# INT-0013 — Derived data is recomputable and versioned

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0013
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Everything computable from authoritative state (objects and op log) is derived
data and never authoritative: trees, per-file history, blame, ancestry and
segment indexes, evolution markers, op heads, intent ↔ change and actor ↔
change indexes, the Book view, the dev integration view, review-request
batches, and declared generated files such as lockfiles. Derived data is keyed
by (input hashes, algorithm id, algorithm version); it can be deleted and
recomputed at any time, and changing an algorithm version is an explicit
derived-data migration. Regenerating a declared generated file needs its
declared generator, an optional layer ([INT-0034](INT-0034-any-content.md));
where that generator is not available, the file is kept and merged like any
other file.

## Acceptance criteria
- **AC-1** `necessary` `formal` — Derived data is a function of (authoritative state, algorithm id, algorithm version): deleting all of it and recomputing under the same versions reproduces it byte for byte.
  - Obligation: `unstated` (R4 §4.4 formalization priority 4).
  - Proxy: a simulation invariant run after every scenario (INT-0005).
- **AC-2** `necessary` `empirical` — Bumping an algorithm version invalidates exactly the entries keyed by it, and recomputation writes a migration record.
- **AC-3** `supporting` `empirical` — Where its generator is available, a declared generated file takes no part in conflicts, whatever it contains; it is regenerated instead.
  - Note: R5 §C.3 measured 17.5–20% fewer textual conflicts and 8–18% lower mean latency when generated files are treated as derived data.

## Rationale
- Mononoke keeps one canonical authoritative model and computes expensive
  indexes asynchronously as derived data (II §2.3).
- Formally, D is derived iff H(D | authoritative, algorithm id, version) = 0
  (III §2.3). R1-E11 showed the version must be part of the input: the same
  diff algorithm with one setting changed produced 28 versus 30 hunks.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.3, III §2.3, R1-E11, R5 §C.3–C.4.

## Alternatives
- **Storing indexes as authoritative state.** Creates two sources of truth that
  can disagree after a crash or an algorithm change.

## Consequences
- Every derived structure needs a recompute path and an algorithm version from
  the start.
- Hash-algorithm migrations (INT-0006) become derived-data migrations instead
  of rewrites of authoritative state.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 12 and R1-E11).
- 2026-11-01: revised while `proposed`, on owner direction: a generator is an optional layer, and generated files can hold anything. The Intent says a declared generated file is kept and merged like any other file where its generator is not available, and AC-3 covers generated files whatever they contain, not only text.
