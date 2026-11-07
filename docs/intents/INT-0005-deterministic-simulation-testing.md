# INT-0005 — Deterministic simulation testing from the first storage code

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0005
- **State:** active
- **Work evidence:** [T-001 build plan](../sprints/s0/sprint-plans/build-plan.md#t-001-workspace-guardrails-policy-checker-and-credits), [Sprint 1 T-002 build plan](../sprints/s1/sprint-plans/build-plan.md#t-002-filesystem-and-randomness-traits-in-the-sim-crate), [Sprint 1 T-003 build plan](../sprints/s1/sprint-plans/build-plan.md#t-003-filesystem-boundary-only-sims-real-half-touches-the-real-filesystem), [Sprint 1 T-005 build plan](../sprints/s1/sprint-plans/build-plan.md#t-005-crash-matrix-with-negative-controls), [T-011 backlog](../work/tasks.md)
- **Completion evidence:** [T-001 completion](../work/completed-tasks.md#t-001-sprint-0), [Sprint 1 T-002 completion](../work/completed-tasks.md#t-002-sprint-1), [Sprint 1 T-003 completion](../work/completed-tasks.md#t-003-sprint-1), [Sprint 1 T-005 completion](../work/completed-tasks.md#t-005-sprint-1)
- **Code evidence:** [clippy.toml](../../clippy.toml), [crates/sim](../../crates/sim/src/lib.rs), [crash matrix](../../crates/store/tests/common/matrix.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md)
- **Documentation evidence:** none

## Intent
All filesystem I/O, clocks and randomness in Periapsis go through traits
(crate `sim`) with a production implementation and a deterministic one,
so any whole-system run is reproducible from a seed. That includes crashes at
every syscall, torn or lost unsynced writes, and later network partitions and
hostile peers. Serialized or ordering-sensitive code uses ordered maps or
seeded hashers, never randomly seeded hash maps. The simulation suite, with
negative controls, runs in CI on every change to storage, concurrency or sync
code.

Boundaries: network simulation arrives with the sync work
([INT-0024](INT-0024-staged-sync.md)); this intent owns the traits, the
filesystem and clock simulation, the crash matrix and the determinism rules.

## Acceptance criteria
- **AC-1** `necessary` `formal` — Determinism: two simulator runs with the same seed and inputs produce byte-identical storage and identical event traces.
  - Obligation: `unstated`.
  - Proxy: CI runs every scenario twice and compares content hashes.
- **AC-2** `necessary` `empirical` — Crash matrix: referential integrity, durability of acknowledged ops and atomicity (the recovered op chain is a prefix of the intended one) hold with a crash injected at every syscall, under lenient and strict-POSIX directory-entry semantics.
  - Sample: at least 40 seeds per scenario. Threshold: 0 violations (R2-E23 measured 0 over 2,600 scenarios).
- **AC-3** `necessary` `empirical` — Negative controls: every known-bad storage variant (pointer file written in place; objects not synced before the op; directory not synced at init) produces at least one violation in the same harness.
  - Note: R2-E23 measured 84, 1,457 and 48 violations; a harness that cannot fail is not evidence.
- **AC-4** `necessary` `empirical` — No randomly seeded `HashMap` or `HashSet` exists in serialized or ordering-sensitive code paths, enforced by Clippy `disallowed-types` in those crates.
- **AC-5** `supporting` `empirical` — Network faults (partitions, reordering, a hostile peer) are simulated for sync with the same seed discipline (delivered with INT-0024).

## Rationale
- Deterministic simulation testing is the established method for storage and
  sync engines: TigerBeetle simulates months of faults in minutes with
  byte-identical storage determinism; turmoil and madsim provide simulated
  network, filesystem shims and controlled time (II §2.9).
- It paid off immediately here: the prototype harness found a real durability
  bug, a missing directory fsync at init (48 violations under strict POSIX),
  that ordinary tests missed (R2-E23).
- Concurrent `O_APPEND` writers did not corrupt records on ext4, but POSIX does
  not guarantee it, so portable designs need per-op files (R2-E27).
- Randomized `HashMap` seeds are a known way to lose determinism (II §2.9).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.9, R2-E23, R2-E27.

## Alternatives
- **Tests against the real filesystem only.** Crashes cannot be placed at
  every syscall and failures do not reproduce.
- **Model checking alone (Loom, TLA+).** Valuable for the concurrency spec
  (INT-0010), but it does not model torn writes or directory durability.

## Consequences
- Every I/O call site goes through a trait, which costs some ergonomics and
  forbids casual `std::fs` use in core crates.
- The harness must grow with the format: each new durable structure needs
  invariants and at least one negative control.
- The simulator's lenient mode models an ordered journal: when a file's own
  entry becomes durable, so does every earlier entry change in its directory.
  Promoting the one entry alone let an older change to the same name reappear
  after a crash, an impossible state (Sprint 1 code review). Each pending entry
  change acts only on the file it was made for.
- The matrix models whole-system crashes. A process crash with the system
  still running (unsynced state visible to the next process) is not
  simulated yet; it belongs with multi-process writers (INT-0007 AC-5).
- Each durability measure needs its own probe: in Sprint 1 the matrix at
  first could not tell `put_object`'s sync from `commit_op`'s re-sync, because
  each masked the other. Acknowledged objects are now checked, and a
  two-writer scenario exercises the re-sync on its own.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §2.9 and R2-E23).
- 2026-10-31: `proposed` → `planned` (Sprint 0 build plan T-001). The first plan critique noted that `codec` is the first serialised code path, so AC-4's Clippy guard against randomly seeded hash collections lands now, workspace-wide; the simulation work itself stays with the first storage code.
- 2026-10-31: `planned` → `active` (Sprint 0 build, T-001 starts: the Clippy hash-collection guard).
- 2026-11-06: revised while `active` (Sprint 1 Loop): Consequences record the lenient mode's ordered-journal model, the unsimulated process crash, and the lesson that each durability measure needs its own probe. AC-1 to AC-4 are met by the Sprint 1 test report; AC-5 (network faults) remains, so the chapter stays `active`.
