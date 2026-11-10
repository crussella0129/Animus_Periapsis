# INT-0005 — Deterministic simulation testing from the first storage code

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0005
- **State:** active
- **Work evidence:** [T-001 build plan](../sprints/s0/sprint-plans/build-plan.md#t-001-workspace-guardrails-policy-checker-and-credits), [Sprint 1 T-002 build plan](../sprints/s1/sprint-plans/build-plan.md#t-002-filesystem-and-randomness-traits-in-the-sim-crate), [Sprint 1 T-003 build plan](../sprints/s1/sprint-plans/build-plan.md#t-003-filesystem-boundary-only-sims-real-half-touches-the-real-filesystem), [Sprint 1 T-005 build plan](../sprints/s1/sprint-plans/build-plan.md#t-005-crash-matrix-with-negative-controls), [T-011 backlog](../work/tasks.md), [Sprint 2 T-003 build plan](../sprints/s2/sprint-plans/build-plan.md#t-003-the-operation-names-its-view-closure-through-references-inside-objects), [Sprint 2 T-004 build plan](../sprints/s2/sprint-plans/build-plan.md#t-004-a-scan-syscall-an-executable-bit-capability-and-operating-system-randomness-in-sim), [Sprint 2 T-007 build plan](../sprints/s2/sprint-plans/build-plan.md#t-007-snapshot-crash-scenario-and-edit-corpora), [T-014 backlog](../work/tasks.md), [Sprint 3 T-002 build plan](../sprints/s3/sprint-plans/build-plan.md#t-002-ban-the-remaining-routes-to-operating-system-randomness), [Sprint 3 T-003 build plan](../sprints/s3/sprint-plans/build-plan.md#t-003-remove_dir-set_executable-and-name_rules-in-sim), [Sprint 3 T-009 build plan](../sprints/s3/sprint-plans/build-plan.md#t-009-reversibility-and-hazard-corpora-the-materialise-crash-scenario-and-the-extended-edit-corpus), [Sprint 4 T-002 build plan](../sprints/s4/sprint-plans/build-plan.md#t-002-a-thread-safe-simfs-with-the-sprint-3-digests-asserted), [Sprint 4 T-003 build plan](../sprints/s4/sprint-plans/build-plan.md#t-003-named-points-and-the-point-registry), [Sprint 4 T-004 build plan](../sprints/s4/sprint-plans/build-plan.md#t-004-the-points-policy-rule), [Sprint 4 T-005 build plan](../sprints/s4/sprint-plans/build-plan.md#t-005-simprocesses-the-deterministic-scheduler-with-orderings-and-kills), [Sprint 4 T-006 build plan](../sprints/s4/sprint-plans/build-plan.md#t-006-plant-and-register-points-in-store-and-workspace-and-move-the-two-writer-race-to-a-named-point), [Sprint 4 T-007 build plan](../sprints/s4/sprint-plans/build-plan.md#t-007-the-four-writer-matrix-and-the-four-process-real-filesystem-test), [Sprint 4 T-008 build plan](../sprints/s4/sprint-plans/build-plan.md#t-008-reach-recording-cargo-reach-the-reach-step-in-cargo-ci-and-the-simulation-spec), [Sprint 4 T-010 build plan](../sprints/s4/sprint-plans/build-plan.md#t-010-consequences-credits-readme-and-backlog), [Sprint 5 T-002 build plan](../sprints/s5/sprint-plans/build-plan.md#t-002-optimised-test-builds-with-every-sprint-4-result-unchanged), [Sprint 5 T-003 build plan](../sprints/s5/sprint-plans/build-plan.md#t-003-run-test-targets-in-parallel-inside-cargo-ci), [Sprint 6 T-002 build plan](../sprints/s6/sprint-plans/build-plan.md#t-002-remove-duplicated-helpers-with-every-matrix-digest-unchanged), [Sprint 6 T-003 build plan](../sprints/s6/sprint-plans/build-plan.md#t-003-a-command-reads-the-op-log-once), [Sprint 6 T-004 build plan](../sprints/s6/sprint-plans/build-plan.md#t-004-open-checks-the-op-log-before-it-resumes-an-interrupted-write), [Sprint 6 T-005 build plan](../sprints/s6/sprint-plans/build-plan.md#t-005-undo-and-restore-scan-the-working-copy-once), [Sprint 6 T-010 build plan](../sprints/s6/sprint-plans/build-plan.md#t-010-describe-sets-the-description-of-the-current-change), [Sprint 6 T-012 build plan](../sprints/s6/sprint-plans/build-plan.md#t-012-a-write-leaves-a-directory-it-cannot-remove)
- **Completion evidence:** [T-001 completion](../work/completed-tasks.md#t-001-sprint-0), [Sprint 1 T-002 completion](../work/completed-tasks.md#t-002-sprint-1), [Sprint 1 T-003 completion](../work/completed-tasks.md#t-003-sprint-1), [Sprint 1 T-005 completion](../work/completed-tasks.md#t-005-sprint-1), [Sprint 2 T-004 completion](../work/completed-tasks.md#t-004-sprint-2), [Sprint 2 T-007 completion](../work/completed-tasks.md#t-007-sprint-2)
- **Code evidence:** [clippy.toml](../../clippy.toml), [crates/sim](../../crates/sim/src/lib.rs), [crash matrix](../../crates/store/tests/common/matrix.rs), [workspace crash matrix](../../crates/workspace/tests/crash.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md), [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md), [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md), [Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md), [Sprint 5 test report](../sprints/s5/sprint-tests/test-report.md), [Sprint 6 test report](../sprints/s6/sprint-tests/test-report.md)
- **Documentation evidence:** [Simulation: named points, several processes and reach](../spec/simulation.md)

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
- **AC-6** `supporting` `empirical` — Reach: every named point registered in a boundary crate is reached by at least one simulation scenario, and the committed reach table (scenarios × units, each grouped by the sprint that built it) equals the one generated from the latest run.
  - Threshold: 100% of registered points reached; `cargo ci` fails on a stale table.
  - Note: a *unit* is a function or module of a boundary crate that a point marks (`crate::name`); its sprint is the sprint whose build plan delivered it.
- **AC-7** `necessary` `empirical` — Counts of cost: for any run, the simulator reports the syscalls made, the flushes made and the bytes read and written, and two runs from one seed report the same counts.
  - Threshold: identical counts in 100% of repeated runs.
  - Note: these counts are the measure for the cost criteria of [INT-0035](INT-0035-scale-and-asynchrony.md) and for [INT-0036](INT-0036-measured-speed.md) AC-6.
- **AC-8** `supporting` `empirical` — Scale of the simulator: a scenario of at least 1,000 simulated processes over one filesystem, and one of at least 100 simulated replicas that exchange over a simulated network, each run from a seed in under 60 seconds on the development host and give identical results twice.

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
- The matrices model whole-system crashes, and since Sprint 4 process
  crashes too. `sim::Processes` runs several simulated processes over one
  filesystem, one at a time, switching at syscalls and named points by the
  seed. A killed process stops at a boundary; its writes, synced or not,
  stay visible to the others until a system crash settles them by the seed
  (`docs/spec/simulation.md`). The four-writer matrix kills process 0
  between its op's rename and its `ops/` sync while another process names
  that op (INT-0007 AC-5).
- Each durability measure needs its own probe: in Sprint 1 the matrix at
  first could not tell `put_object`'s sync from `commit_op`'s re-sync, because
  each masked the other. Acknowledged objects are now checked, and a
  two-writer scenario exercises the re-sync on its own.

- Each new durable unit registers at least one named point, as each new durable
  structure needs a negative control; otherwise the reach table cannot show a
  later scenario passing through it (Sprint 4 plan critique, C-014).
- A point is not a syscall, so adding one changes no single-process run. Under
  several simulated processes, though, every point is a scheduling and kill
  boundary: adding a point changes the four-writer interleavings and digest.
  The expected response is to re-record that digest in the sprint's test
  evidence after confirming the forced races still run and every negative
  control is still caught (C-015).
- An oracle must not read the state it checks. In Sprint 2 the edit corpus
  took "tracked before" from the workspace, so a `new` that dropped tracked
  files passed; the corpus now keeps an independent model of the tracked paths
  and modes (test critique round 1, C-001). Mutation runs by the reviewers are
  the check that an oracle can fail.
- Randomness reaches a boundary crate only through `sim::os_seed`, enforced
  for `getrandom` by `cargo policy`. Since Sprint 3 the rule also bans
  `std::hash::RandomState` and the transitive sources `rand`, `uuid` and
  `fastrand`.
- No clock and no network trait exists yet, because no code reads the time
  or opens a connection. Nothing bans `std::time` or `std::net` in the core
  crates as `std::fs` is banned. Both traits, and both bans, arrive with the
  first code that needs them, and
  [INT-0035](INT-0035-scale-and-asynchrony.md) AC-7 requires the ban on time.
- A matrix has one crash cell for every syscall, reads included, and each
  cell's digest covers the trace. So removing a read from `store` or
  `workspace` changes the cell count and digest of every scenario that made
  it. The response is the one given above for a new point: confirm the
  oracles pass and every negative control is still caught, then record the
  new literals in the sprint's test evidence. A crash at a read leaves the
  same files as a crash at the write before it, so fewer repeated reads in
  the code also make shorter matrices (Sprint 6 code audit).
- `sim::Processes` runs each simulated process as an operating-system thread
  and rescans every process at every syscall. That reaches four writers, not
  a thousand. Its scheduler is plain data, so the same seeded draws can drive
  processes written as resumable steps; AC-8 needs that change (Sprint 6
  code audit, S-01 and S-02; backlog T-032).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §2.9 and R2-E23).
- 2026-10-31: `proposed` → `planned` (Sprint 0 build plan T-001). The first plan critique noted that `codec` is the first serialised code path, so AC-4's Clippy guard against randomly seeded hash collections lands now, workspace-wide; the simulation work itself stays with the first storage code.
- 2026-10-31: `planned` → `active` (Sprint 0 build, T-001 starts: the Clippy hash-collection guard).
- 2026-11-06: revised while `active` (Sprint 1 Loop): Consequences record the lenient mode's ordered-journal model, the unsimulated process crash, and the lesson that each durability measure needs its own probe. AC-1 to AC-4 are met by the Sprint 1 test report; AC-5 (network faults) remains, so the chapter stays `active`.
- 2026-11-07: revised while `active` (Sprint 2 Loop): Consequences record that an oracle must not read the state it checks, and the randomness routes not yet banned (T-014). AC-1 to AC-4 hold again for the new syscall and the Snapshot matrix; AC-5 (network faults) remains, so the chapter stays `active`.
- 2026-11-08: revised while `active` (Sprint 4 Plan, on the owner's proposal of dormant coded breakpoints collected in a table, and of regression runs that pass back through every earlier unit): AC-6 adds reach. Named points that are not syscalls, a registry of them, and a generated reach table make that triangle of runs × units checked evidence.
- 2026-11-08: revised while `active` (Sprint 4 plan critique, round 1): AC-6 defines a unit; Consequences require a point for each new durable unit and record that points are scheduling boundaries under several processes.
- 2026-11-08: revised while `active` (Sprint 4 plan critique, round 2, C-009): a unit may be a module as well as a function.
- 2026-11-09: revised while `active` (Sprint 4 build, T-010): the Consequence on process crashes records the model built (several simulated processes, kills, the four-writer matrix) in place of "not simulated yet".
- 2026-11-09: evidence while `active` (Sprint 4 Loop): AC-6 is met. All 21 registered points are reached, each scenario reaches the points expected of it, and the generated table is identical on Windows and Linux. AC-1 to AC-4 hold for several processes, and the three earlier digests are unchanged ([Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md)). AC-5 (network faults) stays with INT-0024.
- 2026-11-09: evidence while `active` (Sprint 5 Loop): AC-1 to AC-3 and AC-6 hold unchanged under optimised builds and parallel test files. The four digests, the 12 controls and the reach table pass, and every matrix line is identical to Sprint 4's on Windows and Linux. AC-5 (network faults) remains, so the chapter stays `active` ([Sprint 5 test report](../sprints/s5/sprint-tests/test-report.md)).
- 2026-11-10: revised while `active` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents): AC-7 asks the simulator for exact counts of cost, which [INT-0035](INT-0035-scale-and-asynchrony.md) and [INT-0036](INT-0036-measured-speed.md) measure by, and AC-8 for scenarios of 1,000 processes and 100 replicas. Consequences record that there is no clock or network trait yet, what a removed syscall does to a matrix, and why `sim::Processes` does not reach a thousand. A stale sentence is corrected (the remaining randomness routes were banned in Sprint 3), and Documentation evidence names the simulation spec.
