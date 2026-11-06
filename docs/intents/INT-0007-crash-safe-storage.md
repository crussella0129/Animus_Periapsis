# INT-0007 — Crash-safe storage with op heads derived from the op log

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0007
- **State:** active
- **Work evidence:** [Sprint 1 T-004 build plan](../sprints/s1/sprint-plans/build-plan.md#t-004-object-and-op-file-store-with-derived-heads-and-fsck-in-the-store-crate), [Sprint 1 T-005 build plan](../sprints/s1/sprint-plans/build-plan.md#t-005-crash-matrix-with-negative-controls), [Sprint 1 T-006 build plan](../sprints/s1/sprint-plans/build-plan.md#t-006-fsck-corruption-corpus-and-the-praps-fsck-command), [Sprint 1 T-007 build plan](../sprints/s1/sprint-plans/build-plan.md#t-007-windows-durability-strategy-and-real-filesystem-evidence)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

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

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 8 and R2 §3).
- 2026-11-01: revised while `proposed`, on owner direction: the command is `praps`, short for periapsis, replacing `peri` ([INT-0003](INT-0003-engineering-baseline.md)).
- 2026-11-06: AC-3 revised while `proposed`, after the Sprint 1 plan critique (round 1, C-005): an op whose commit was interrupted after its file became durable but before it was acknowledged may survive, so the recovered heads are the last acknowledged heads extended only by such in-flight ops, each wholly present and valid or absent. Consequences gain the on-disk name form (C-007) and the rule that an already-present file is synced before a dependent op is acknowledged (C-004).
- 2026-11-06: `proposed` → `planned` (Sprint 1 build plan T-004 to T-007: the store, the crash matrix, `fsck` and the Windows durability note; AC-5, four writing processes, is deferred to a later sprint).
- 2026-11-06: `planned` → `active` (Sprint 1 build, T-004 starts: the store crate).
