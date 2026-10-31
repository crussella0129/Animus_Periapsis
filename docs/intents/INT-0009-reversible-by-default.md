# INT-0009 — Reversible by default; erasure is explicit and signed

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0009
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Every operation that is not an explicit erasure is injective on repository
state, so the op log is a complete history and `undo` and `op restore` can
return to any earlier state. Every operation that overwrites the working copy
snapshots it first. Information leaves the repository only through explicit
erasure operations — garbage collection of unreachable objects, redaction,
history truncation, op-log abandonment — which are signed by an authorised
principal, gated by repository policy, and record the erased object ids and
byte counts.

## Acceptance criteria
- **AC-1** `necessary` `formal` — Injectivity: for every non-erasure op `F` and states `σ₁ ≠ σ₂` in its domain, `F(σ₁) ≠ F(σ₂)`; equivalently, every prior state is recoverable from the op log.
  - Obligation: `unstated` (R4 §4.4 formalization priority 3).
  - Proxy: property tests over random pairs of pre-states.
- **AC-2** `necessary` `empirical` — Snapshot-before-overwrite: over at least 200 pairs of pre-states that differ only in uncommitted working-copy content, every working-copy-overwriting command leaves the pairs distinguishable.
  - Threshold: 0 collapsed pairs (R1-E2: 0/200 with auto-snapshot, 200/200 without).
- **AC-3** `necessary` `empirical` — `undo` and `op restore <op>` return views and working copy to the exact earlier state for every op type in the test corpus.
- **AC-4** `necessary` `formal` — Only erasure op types remove objects or ops from storage; each requires a signature from a principal authorised by repository policy and records the erased ids and bytes.
  - Obligation: `unstated`.
  - Proxy: the storage delete API is reachable only from the erasure path (checked by tests and review).
- **AC-5** `supporting` `asserted` — Shallow and partial clones record which history they lack.
  - Signers: human reviewer.

## Rationale
- Bits destroyed by a deterministic map F on a prior X equal H(X|F(X)), which
  is 0 exactly when F is injective (Landauer/Bennett, III §2; identity
  confirmed numerically in R1-E1). An append-only op log is Bennett's history
  tape, which is why jj-style undo works.
- The injectivity claim holds only with auto-snapshot: without it, uncommitted
  edits were destroyed in 200 of 200 cases (R1-E2).
- The R3 audit removed the per-op `bits_destroyed` field because it is 0 for
  every op except erasure ops and so duplicates the op type; the principle
  stays (R3 §4.1).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  III §2, R1-E1, R1-E2, R3 §4.1.

## Alternatives
- **A Landauer ledger: `bits_destroyed_ub` on every op.** Removed as redundant
  (R3 §4.1); see [INT-0028](INT-0028-entropy-layer.md).
- **Git-style reflog with expiry.** Lossy by design; recovery depends on
  timing.

## Consequences
- Storage grows until an explicit, signed GC runs, so GC policy and signing
  rights must exist before long-lived repositories do.
- Redacting sensitive data is itself a visible, attributed erasure operation.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 III §2 as corrected by R1 and R3).
