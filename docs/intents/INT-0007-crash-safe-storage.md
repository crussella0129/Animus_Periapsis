# INT-0007 — Crash-safe storage with op heads derived from the op log

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0007
- **State:** active
- **Work evidence:** [Sprint 1 T-004 build plan](../sprints/s1/sprint-plans/build-plan.md#t-004-object-and-op-file-store-with-derived-heads-and-fsck-in-the-store-crate), [Sprint 1 T-005 build plan](../sprints/s1/sprint-plans/build-plan.md#t-005-crash-matrix-with-negative-controls), [Sprint 1 T-006 build plan](../sprints/s1/sprint-plans/build-plan.md#t-006-fsck-corruption-corpus-and-the-praps-fsck-command), [Sprint 1 T-007 build plan](../sprints/s1/sprint-plans/build-plan.md#t-007-windows-durability-strategy-and-real-filesystem-evidence), [T-011 backlog](../work/tasks.md), [Sprint 2 T-003 build plan](../sprints/s2/sprint-plans/build-plan.md#t-003-the-operation-names-its-view-closure-through-references-inside-objects), [Sprint 2 T-007 build plan](../sprints/s2/sprint-plans/build-plan.md#t-007-snapshot-crash-scenario-and-edit-corpora), [Sprint 3 T-006 build plan](../sprints/s3/sprint-plans/build-plan.md#t-006-materialise-the-working-copy-record-and-resume), [Sprint 3 T-009 build plan](../sprints/s3/sprint-plans/build-plan.md#t-009-reversibility-and-hazard-corpora-the-materialise-crash-scenario-and-the-extended-edit-corpus), [Sprint 4 T-005 build plan](../sprints/s4/sprint-plans/build-plan.md#t-005-simprocesses-the-deterministic-scheduler-with-orderings-and-kills), [Sprint 4 T-006 build plan](../sprints/s4/sprint-plans/build-plan.md#t-006-plant-and-register-points-in-store-and-workspace-and-move-the-two-writer-race-to-a-named-point), [Sprint 4 T-007 build plan](../sprints/s4/sprint-plans/build-plan.md#t-007-the-four-writer-matrix-and-the-four-process-real-filesystem-test), [Sprint 4 T-010 build plan](../sprints/s4/sprint-plans/build-plan.md#t-010-consequences-credits-readme-and-backlog)
- **Completion evidence:** [Sprint 1 T-004 completion](../work/completed-tasks.md#t-004-sprint-1), [Sprint 1 T-005 completion](../work/completed-tasks.md#t-005-sprint-1), [Sprint 1 T-006 completion](../work/completed-tasks.md#t-006-sprint-1), [Sprint 1 T-007 completion](../work/completed-tasks.md#t-007-sprint-1), [Sprint 2 T-003 completion](../work/completed-tasks.md#t-003-sprint-2), [Sprint 2 T-007 completion](../work/completed-tasks.md#t-007-sprint-2)
- **Code evidence:** [crates/store](../../crates/store/src/lib.rs), [crates/praps](../../crates/praps/src/main.rs), [references.rs](../../crates/codec/src/references.rs)
- **Test evidence:** [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md), [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md), [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md)
- **Documentation evidence:** [Storage durability](../spec/storage-durability.md)

## Intent
The object store and op log never lose an operation acknowledged to its caller
and never expose a partially written one, whatever the crash point, on Windows
and Linux. Each operation is one file (the portable default; a checksummed
append-only log is an optional local fast path). Objects are durable before the
operation that references them. Op heads are derived from the checksummed op
files; there is no HEAD or pointer file. Repository initialisation creates its
files and makes the directory entries durable. `praps fsck` verifies checksums
and referential integrity.

## Acceptance criteria
- **AC-1** `necessary` `formal` — After any crash, the recovered set of ops is closed under ancestry, and every recovered op's referenced objects are present.
  - Obligation: `unstated`.
  - Proxy: the INT-0005 crash matrix.
- **AC-2** `necessary` `empirical` — Every op acknowledged to the caller survives a crash at any later syscall.
  - Threshold: 0 violations across the crash matrix.
- **AC-3** `necessary` `empirical` — After recovery, op heads recomputed from the op files equal the heads of the last acknowledged state, extended only by in-flight ops (ops whose commit began but was not acknowledged before the crash, each of which is either wholly present and valid or absent), and the store contains no HEAD or pointer file.
- **AC-4** `necessary` `empirical` — `fsck` detects every injected corruption (truncated op file, flipped bit in an object, missing referenced object) and reports nothing on healthy repositories.
  - Threshold: 100% detection, 0 false reports, over a corruption corpus.
- **AC-5** `supporting` `empirical` — At least four processes writing ops concurrently produce no corrupt records and no lost ops, under crash injection.

## Rationale
- The prototype store had 0 violations across 2,600 crash scenarios, while
  negative controls (pointer file written in place; objects not synced before
  the op) produced 84 and 1,457 violations (R2-E23).
- Strict POSIX semantics exposed a real bug, the missing directory fsync at
  init (48 violations), fixed to 0 (R2-E23).
- Deriving heads from the checksummed op log was equally crash-safe and needed
  37% fewer syscalls than a HEAD file (R2-E23b).
- Concurrent `O_APPEND` writers were fine on ext4 (61,600 appends, 0 corrupt)
  but POSIX does not guarantee it, so per-op files are the portable default
  (R2-E27).
- Multi-writer crash consistency was left open (R2 §4), hence AC-5.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  R2-E23, R2-E23b, R2-E27, R3 §4.1.

## Alternatives
- **A HEAD/pointer file.** Removed: redundant with the op log and a source of
  crash bugs (R2-E23b).
- **Single append-only pack log as the only format.** Kept only as an optional
  fast path because its interleaving guarantees are not portable.
- **An embedded database (SQLite, LMDB).** Rejected: a C dependency, and it
  hides the durability semantics this intent must prove.

## Consequences
- Per-op files mean many small files; compaction and packing become later work.
- Windows has no POSIX directory fsync, so the durability strategy there needs
  its own design and evidence (INT-0004 AC-5).
- On-disk names cannot use the `HashRef` text form `<name>:<hex>`: on NTFS a
  colon names an alternate data stream. Object and op files are named
  `<algorithm id as two hex digits>-<digest hex>`, which keeps equal digests
  under different algorithms apart and is legal on Windows and Linux.
- An object or op that is already present is still synced, with its
  directory, before an op that depends on it is acknowledged, because another
  writer may have renamed it without syncing yet.
- Opening a store for writing syncs its layout, as `init` does: a process
  that died inside `init` can leave directories that exist but were never
  made durable. Checking a store (`fsck`) opens it read-only and syncs
  nothing, so it works on read-only media.
- The directory holding a root's entry is its parent, or `root/..` for a bare
  relative name such as `repo` or `.`, whose parent path is empty.
- Closure follows references inside objects (op → view → changesets →
  blobs). The store refuses an object or op whose direct references are not
  present, valid and of the kind it must be. Closure holds by induction over
  acknowledged puts, for two reasons. A writer syncs a file's data before
  renaming it. Every object lives in `objects/`, and each put ends with a
  sync of that directory, which also makes durable a name that another
  writer renamed but has not synced. An unacknowledged object that survives
  a crash with a lost reference is unreachable from every op; `fsck` lists
  it as a leftover, not a fault.
- A writer may name another writer's view or parent op that is present but
  not yet durable, so `commit_op` re-syncs the view with `objects/`, and each
  parent op with `ops/`, before writing the op. The parent re-sync is needed
  even within one directory, because a crash between the new op's rename and
  its own directory sync can keep the in-flight op and lose an unsynced
  parent. Both re-syncs share one negative control, exercised by a race of
  each kind.

- Power loss on real hardware is out of scope: there is no power-cut rig, and
  the simulator's crash model (INT-0005), which assumes only what POSIX
  promises in strict mode, stands in for it (Sprint 4 plan critique, C-021).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 8 and R2 §3).
- 2026-11-01: revised while `proposed`, on owner direction: the command is `praps`, short for periapsis, replacing `peri` ([INT-0003](INT-0003-engineering-baseline.md)).
- 2026-11-06: AC-3 revised while `proposed`, after the Sprint 1 plan critique (round 1, C-005): an op whose commit was interrupted after its file became durable but before it was acknowledged may survive, so the recovered heads are the last acknowledged heads extended only by such in-flight ops, each wholly present and valid or absent. Consequences gain the on-disk name form (C-007) and the rule that an already-present file is synced before a dependent op is acknowledged (C-004).
- 2026-11-06: `proposed` → `planned` (Sprint 1 build plan T-004 to T-007: the store, the crash matrix, `fsck` and the Windows durability note; AC-5, four writing processes, is deferred to a later sprint).
- 2026-11-06: `planned` → `active` (Sprint 1 build, T-004 starts: the store crate).
- 2026-11-06: revised while `active` (Sprint 1 Loop): Consequences record that opening for writing syncs the layout, that `fsck` opens read-only, and how a relative root's parent is found (Sprint 1 code review and test critiques). AC-1 to AC-4 are met by the Sprint 1 test report; AC-5 (four writing processes) remains, so the chapter stays `active`.
- 2026-11-07: revised while `active` (Sprint 2 Plan): Consequences record that closure follows references inside objects and holds by induction, because an object is written only after its direct references are present, valid and synced.
- 2026-11-07: revised while `active` (Sprint 2 plan critique, round 1): the closure-by-induction Consequence now says which re-syncs are needed: none for references inside `objects/`, which the object's own directory sync covers, and `commit_op`'s cross-directory re-sync of the view and parents.
- 2026-11-07: revised while `active` (Sprint 2 plan critique, round 2): the closure Consequence is restated through reachability from ops, with unreachable objects reported as leftovers, and the re-sync Consequence names both the view and the same-directory parent re-sync, each probed by its own race.
- 2026-11-07: revised while `active` (Sprint 2 code review, CR-11 and CR-14): references must also be of the kind they name (a view's heads and workspaces are changesets, a changeset's puts are blobs), and the closure Consequence rests on data-before-names and the shared `objects/` directory sync rather than on acknowledgement order alone.
- 2026-11-07: evidence while `active` (Sprint 2 Loop): AC-1 to AC-4 hold again with closure through references and reference kinds (Sprint 2 test report). AC-5 (four writing processes) remains in backlog T-011.
- 2026-11-08: revised while `active` (Sprint 4 plan critique, round 1): Consequences record that power loss on real hardware is out of scope and the crash model stands in.
