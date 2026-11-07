# INT-0009 — Reversible by default; erasure is explicit and signed

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0009
- **State:** planned
- **Work evidence:** [Sprint 3 T-004 build plan](../sprints/s3/sprint-plans/build-plan.md#t-004-specify-writing-the-working-copy-undo-restore-and-log), [Sprint 3 T-006 build plan](../sprints/s3/sprint-plans/build-plan.md#t-006-materialise-the-working-copy-record-and-resume), [Sprint 3 T-007 build plan](../sprints/s3/sprint-plans/build-plan.md#t-007-log-undo-and-restore-in-workspace), [Sprint 3 T-008 build plan](../sprints/s3/sprint-plans/build-plan.md#t-008-praps-log-praps-undo-and-praps-restore-op), [Sprint 3 T-009 build plan](../sprints/s3/sprint-plans/build-plan.md#t-009-reversibility-and-hazard-corpora-the-materialise-crash-scenario-and-the-extended-edit-corpus), [Sprint 3 T-010 build plan](../sprints/s3/sprint-plans/build-plan.md#t-010-readme-credits-and-backlog)
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
- `undo` is itself an operation. It takes H, the single op head when the
  command starts, snapshots, and then commits an op whose view is the view of
  H's parent, so the last command is undone, and so are the edits since it.
  Those edits stay reachable by `restore` through the snapshot op. With no
  parent, `undo` fails after its snapshot. A second `undo` undoes the first.
  `restore OP` returns to any earlier view. Neither removes an op (Sprint 3
  Plan, revised after its critique, C-001).
- Writing the working copy is not atomic, so it is journalled: a record in
  `.praps/`, written durably before the op that needs it, lets the next
  command finish an interrupted write instead of snapshotting a half-written
  working copy as user edits. The record is working-copy state, not a store
  pointer ([INT-0007](INT-0007-crash-safe-storage.md)).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 III §2 as corrected by R1 and R3).
- 2026-11-07: revised while `proposed` (Sprint 3 Plan): Consequences record that `undo` and `restore` are themselves operations (undo toggles; edits stay reachable) and that writing the working copy is journalled by a record in `.praps/`.
- 2026-11-07: `proposed` → `planned` (Sprint 3 build plan T-004 to T-008: AC-1 to AC-3; AC-4 and AC-5 wait for signed erasure and partial clones).
- 2026-11-07: revised while `planned` (Sprint 3 plan critique, round 1, C-001): the `undo` Consequence names H as the head at the command's start, before its snapshot.
