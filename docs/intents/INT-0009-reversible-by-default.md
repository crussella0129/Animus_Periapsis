# INT-0009 — Reversible by default; erasure is explicit and signed

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0009
- **State:** active
- **Work evidence:** [Sprint 3 T-004 build plan](../sprints/s3/sprint-plans/build-plan.md#t-004-specify-writing-the-working-copy-undo-restore-and-log), [Sprint 3 T-006 build plan](../sprints/s3/sprint-plans/build-plan.md#t-006-materialise-the-working-copy-record-and-resume), [Sprint 3 T-007 build plan](../sprints/s3/sprint-plans/build-plan.md#t-007-log-undo-and-restore-in-workspace), [Sprint 3 T-008 build plan](../sprints/s3/sprint-plans/build-plan.md#t-008-praps-log-praps-undo-and-praps-restore-op), [Sprint 3 T-009 build plan](../sprints/s3/sprint-plans/build-plan.md#t-009-reversibility-and-hazard-corpora-the-materialise-crash-scenario-and-the-extended-edit-corpus), [Sprint 3 T-010 build plan](../sprints/s3/sprint-plans/build-plan.md#t-010-readme-credits-and-backlog), [Sprint 6 T-005 build plan](../sprints/s6/sprint-plans/build-plan.md#t-005-undo-and-restore-scan-the-working-copy-once), [Sprint 6 T-009 build plan](../sprints/s6/sprint-plans/build-plan.md#t-009-log-lists-changes-and-ops-lists-operations), [Sprint 6 T-010 build plan](../sprints/s6/sprint-plans/build-plan.md#t-010-describe-sets-the-description-of-the-current-change), [Sprint 6 T-012 build plan](../sprints/s6/sprint-plans/build-plan.md#t-012-a-write-leaves-a-directory-it-cannot-remove)
- **Completion evidence:** [Sprint 3 T-004 completion](../work/completed-tasks.md#t-004-sprint-3), [Sprint 3 T-006 completion](../work/completed-tasks.md#t-006-sprint-3), [Sprint 3 T-007 completion](../work/completed-tasks.md#t-007-sprint-3), [Sprint 3 T-008 completion](../work/completed-tasks.md#t-008-sprint-3), [Sprint 3 T-009 completion](../work/completed-tasks.md#t-009-sprint-3), [Sprint 3 T-010 completion](../work/completed-tasks.md#t-010-sprint-3)
- **Code evidence:** [crates/workspace](../../crates/workspace/src/lib.rs), [materialise.rs](../../crates/workspace/src/materialise.rs), [record.rs](../../crates/workspace/src/record.rs), [praps](../../crates/praps/src/main.rs)
- **Test evidence:** [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md)
- **Documentation evidence:** [working-copy spec](../spec/working-copy.md#writing-the-working-copy)

## Intent
Every operation that is not an explicit erasure is injective on repository
state, so the op log is a complete history and `undo` and `op restore` can
return to any earlier state. Every operation that overwrites the working copy
snapshots it first. Information leaves the repository only through explicit
erasure operations — garbage collection of unreachable objects, redaction,
history truncation, op-log abandonment — which are signed by an authorised
principal, gated by repository policy, and record the erased object ids and
byte counts.

With many writers, three rules keep this true
([INT-0035](INT-0035-scale-and-asynchrony.md)):
- **Undo is scoped.** `undo` takes back the last operation this workspace
  made, and `restore` returns this workspace to its state at an earlier
  operation. Neither changes a view entry that another actor owns
  ([INT-0016](INT-0016-actors-and-agent-lifecycle.md)).
- **Operations are local or shared.** A local operation records the working
  life of one workspace: its snapshots, undos and restores. It stays on its
  replica. A shared operation records what others may build on: a published
  lane head, a rewrite of a published change, an event record, a promotion.
  Only shared operations are synced
  ([INT-0024](INT-0024-staged-sync.md)). A replica's own log holds both, so
  each of its own earlier states stays recoverable.
- **Packing is not erasure.** Storing operations or objects in less space
  ([INT-0008](INT-0008-compact-history-storage.md)) removes nothing and needs
  no signature.

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
- **AC-6** `necessary` `empirical` — Undo is scoped: with two writers interleaved, `undo` and `restore` by one leave every view entry the other owns exactly as the merge of op heads left it.
  - Sample: at least 200 seeded interleavings.
  - Threshold: 0 entries of the other writer changed.
- **AC-7** `necessary` `formal` — No local operation leaves its replica: none is sent to a peer or written to a bundle.
  - Obligation: `unstated`.
  - Proxy: property tests over push and bundle creation, with [INT-0024](INT-0024-staged-sync.md) AC-4.

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
- As built in Sprint 3, `undo` and `restore` put back a whole view, and "the
  last op" is the single head, whoever made it. With one writer and one
  workspace that is the writer's own state, so AC-1 to AC-3 hold. With
  several writers it would undo their work too. The scoped form (AC-6)
  arrives with the merge of op heads
  ([INT-0010](INT-0010-lock-free-concurrency.md)).
- Every snapshot is an operation, so thousands of agents fill a log fast.
  Local operations and packing keep that from costing every replica: a
  replica stores its own working operations and the shared ones it follows.
- Since Sprint 6 the command that lists operations is `ops`, and `log` lists
  changes ([INT-0037](INT-0037-command-set.md)).
- A command that overwrites the working copy records it once, just before it
  plans the write (Sprint 6, T-005). As built in Sprint 3, `undo` and
  `restore` recorded it twice, which read every file a second time. Once is
  enough for AC-2: an edit made before the command is in that snapshot, and
  the write reads each path again just before it removes or replaces it and
  leaves a path that has changed since
  ([INT-0014](INT-0014-working-copy-and-workspaces.md)).
- `describe` is an operation like any other, so `undo` after it gives back
  the description the change had before, and `restore` can return to either
  (Sprint 6, T-010).
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
- 2026-11-07: `planned` → `active` (Sprint 3 build, T-004 starts: the working-copy spec for undo, restore and log).
- 2026-11-10: revised while `active` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents): the Intent scopes `undo` and `restore` to what the actor owns, separates local from shared operations, and says packing is not erasure. AC-6 and AC-7 are added. Consequences record that Sprint 3's `undo` and `restore` act on the whole view, which is right only for one writer, and the rename of the operation list to `ops`. The Completion and Code evidence fields, left at `none` since Sprint 3, now name that sprint's tasks and code.
- 2026-11-10: revised while `active` (Sprint 6 plan critique, round 1, C-002; `editorial`: no criterion changes): Consequences record that a command which overwrites the working copy snapshots it once, with the reason one snapshot keeps AC-2, and that `describe` is an operation `undo` and `restore` reverse. Work evidence names Sprint 6's T-005, T-009 and T-010.
- 2026-11-10: evidence while `active` (Sprint 6 plan critique, round 2, C-005 and C-009): Work evidence names Sprint 6's T-012. A write that cannot remove a directory it emptied leaves it ([INT-0014](INT-0014-working-copy-and-workspaces.md)); AC-3's "exact earlier state" is of tracked files, and directories are not tracked.
