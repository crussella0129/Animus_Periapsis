# Sprint 4 Research Report

## Intents Reviewed
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — **selected and to be revised**; current state: active.
  - **The owner's proposal.** Planted, coded breakpoints that do nothing until switched on, collected in a table, and used to coordinate integration testing. Each sprint's regression run goes back through every earlier unit's points, so the runs and units form a growing triangle.
  - **What the simulator already does.** Every syscall is a numbered crash point, enumerated by the matrices.
  - **What it lacks:**
    - Named points inside logic.
    - A record of which points each scenario reached.
    - A way to coordinate several writers by name rather than by syscall number or trace text.
  - **The revision.** A new acceptance criterion for reach, and a Consequence replacing "a process crash is not simulated yet".
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — **selected**; current state: active. AC-5 asks that at least four processes writing ops concurrently produce no corrupt records and no lost ops under crash injection (backlog T-011). Coordinating those writers is exactly what named points are for.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — **selected for backlog T-019**; current state: active. AC-7's "skipped without a report" holds only for `status`. `undo`, `restore` and `new` drop their snapshot's report.
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md) — **selected for backlog T-017 only**: record the Sprint 3 checkpoint's GitHub Actions runs and the pending sign-offs. Current states: proposed and active.
- [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) — **reviewed, not selected**; current state: proposed.
  - Several store writers leave several op heads. That is legal in the store, which derives heads from op files.
  - Merging those heads is INT-0010's canonical merge, Phase 2. Until it exists, workspace commands keep stopping on several heads.
  - This sprint's writers therefore work at the store level.

## 1. Sprint Goal
Make the simulator coordinate several writers by named points, and show which earlier code every scenario reaches:
- **Named points:** code places a dormant, named point (`fs.point("store.commit.renamed")`).
  - On the real filesystem it does nothing.
  - In the simulator it is recorded, and a test can attach an action to it by name.
  - Every point is declared once in a registry, with its unit and the sprint that added it.
- **Several processes:** a deterministic scheduler runs several simulated processes over one `SimFs`, one at a time. It switches between them at syscalls and points, by seed.
  - A process can be killed while the system keeps running. The next process then sees its unsynced state until a system crash settles it (INT-0005 Consequences).
- **Four writers:** a matrix of four store writers under crash injection, including a process crash, with negative controls (INT-0007 AC-5). It is backed by a real-filesystem run of four OS processes.
- **The reach table:** a generated table with every scenario as a row and every unit's points as columns, grouped by sprint. Every registered point must be reached by some scenario. This turns the owner's triangle diagram into checked evidence.
- **Carry forward:** T-017 (the Sprint 3 CI record and sign-offs) and T-019 (report skipped names from `undo`, `restore` and `new`).

**Out of scope:**
- merging several op heads (INT-0010);
- several workspaces (INT-0014 AC-3);
- network simulation;
- selecting which tests to run from the reach table. The table makes this possible later; this sprint only records reach.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| `crates/sim/src/fs.rs` | high | The `Fs` trait: 14 syscalls plus two non-syscall capabilities. A `point` method fits as a third non-syscall method with a no-op default. |
| `crates/sim/src/simfs.rs` | high | State in a `RefCell`. One `Hook` runs at every syscall boundary, keyed by the next syscall number. Neither can cross threads, so the scheduler needs a `Mutex` and a `Send` hook. |
| `crates/sim/src/real.rs` | medium | `RealFs`: `point` is a no-op that compiles away. |
| `crates/sim/tests/contract.rs` | medium | Determinism checks compare traces; points must not change syscall numbering or traces. |
| `crates/store/tests/common/matrix.rs` | high | The two-writer race. `install_race` finds its moment by matching trace text (`starts_with("rename ")`), a hand-placed breakpoint keyed to output. A named point replaces it. |
| `crates/store/src/lib.rs` | high | `put_object` and `commit_op` (write, sync, rename, re-sync), the stages to name. |
| `crates/workspace/src/lib.rs` | high | snapshot, materialise, record and resume stages to name. `undo`, `restore_view` and `new_change` drop `Snapshot::reported` (T-019). |
| `crates/workspace/tests/crash.rs`, `materialise_crash.rs` | high | The Snapshot and Materialise matrices. Their digests (`c1e39cb1…`, `6274344b…`) and the store's (`d36b8c9c…`) must stay unchanged, which is the regression row for earlier units. |
| `crates/praps/src/main.rs` | medium | Printing the skipped names (T-019). |
| `crates/xtask/src/policy.rs` | high | The fixture-tested policy rules. A `points` rule fits here: each point named in code is registered exactly once, and each registered point is used. |
| `docs/prior-art.md` | medium | Credits: fail-rs, RocksDB sync points, FoundationDB BUGGIFY, shuttle/PCT. |
| `docs/work/tasks.md` | high | T-011, T-017, T-019. |

## 3. External Sources
- [fail-rs (`fail` crate)](https://docs.rs/crate/fail): named fail points, compiled out unless the `failpoints` feature is on. They are configured by name, from code or the `FAILPOINTS` environment variable, with actions such as panic, sleep, print or pause.
  - Periapsis takes the idea of dormant points configured by name.
  - It does not take the crate:
    - the configuration is process-global and racy across parallel tests (hence the `fail-parallel` fork);
    - it uses real threads and time, which the deterministic simulator must own;
    - a new dependency would have to justify itself against a method on `Fs`.
- [RocksDB `sync_point.h`](https://sources.debian.org/src/rocksdb/9.11.2-1/test_util/sync_point.h): `TEST_SYNC_POINT("name")` markers, a no-op in release builds. `LoadDependency({{A, B}})` blocks any thread at B until some thread has passed A, so a test can force an interleaving by table. This is the owner's "table of coded breakpoints used to coordinate integration testing", already proven in production. Periapsis takes the table form for its scheduler: an ordering of named points.
- [FoundationDB BUGGIFY (as described for client testing)](https://apple.github.io/foundationdb/client-testing.html): fault sites marked in code. They are always false in production, and each is switched on at random per simulation run. This supports seeding which points act, rather than enumerating every combination.
- [shuttle (AWS Labs)](https://github.com/awslabs/shuttle): randomized concurrency testing for Rust. One logical thread runs at a time under a seeded scheduler, which includes PCT (Burckhardt et al., ASPLOS 2010) with a bounded number of preemptions, and failing schedules replay from the seed.
  - Periapsis's scheduler follows the same model, one runner at a time with seeded switches. It is limited to the simulator's own switch points (syscalls and named points), so it needs no replacement of std's synchronisation types.
  - shuttle is credited, not depended on.

## 4. Risks, Unknowns, Dependencies
- **Points must not move crash points.** A point is not a syscall: it is neither counted nor traced. All three existing matrix digests must stay unchanged, which also shows that no earlier unit changed behaviour.
- **`SimFs` must be shared across threads.**
  - Simulated processes need real stacks to stop mid-operation. Rust has no stable coroutines, and `unsafe` is forbidden.
  - So each process is an OS thread, and a baton lets exactly one run at a time. `SimFs` moves from `RefCell` to `Mutex`, and its `Hook` gains `Send`, so test code that captures `Rc` in a hook moves to `Arc`.
  - Determinism holds because only the baton holder runs and every switch is drawn from the seed.
  - A panic in one process must end the run cleanly, not deadlock the baton.
- **What a process crash means.** A killed process stops at a switch point. Its files, synced or not, stay visible to the others, because the page cache survives. Its in-memory state is gone.
  - A later system crash settles all pending state by the seed.
  - Invariants:
    - every op acknowledged to a live or killed process survives;
    - the killed process's unacknowledged op is absent or wholly valid;
    - `fsck` reports leftovers, never faults.
- **Matrix size.** With four writers, each op takes about 15 syscalls, so a run makes roughly 120 syscalls plus points. With 40 seeds × 2 modes × every crash step × a few schedules, the matrix must stay within CI time. Sprint 3's `cargo ci` already took 31 minutes on Windows. The plan needs to fix the seed and schedule counts before measuring.
- **A real-filesystem run.** A test helper binary in `store`'s tests, run as four OS processes against one temporary directory, checks no corruption and no lost ops without crashes. The real kernel decides the interleavings, so this run is evidence, not reproducible.
- **Reach must be computed, not curated.**
  - The table is generated from the scenarios' recorded point counts and the registry, then compared with a committed file, so a stale table fails the check.
  - The `points` policy rule needs negative fixtures: an unregistered point, an unused registration, and a duplicate.
- **Dependencies:** none new. `std::thread`, `std::sync::Mutex` and `Condvar` only.

## 5. Recommended Approach
1. **T-017:** record the Sprint 3 checkpoint's GitHub runs. Carry forward any sign-offs not given.
2. **`sim` points and the registry:**
   - `Fs::point(&self, name: &'static str)` with a no-op default.
   - `SimFs` counts each point and runs actions attached by name. Points are not syscalls.
   - `sim::Point { name, unit, sprint }`, with each boundary crate exporting `POINTS`.
   - A `points` policy rule in `xtask`, with fixtures.
3. **The `sim` scheduler:**
   - `SimFs` becomes `Sync`.
   - `sim::Processes` runs N closures under a baton, switching at syscalls and points by seed. It also takes a list of named-point orderings, as in RocksDB.
   - A process crash (kill) at a chosen step.
   - Determinism and contract tests, with the existing digests unchanged.
4. **Plant points** at the stages of `store` (`put_object`, `commit_op`) and `workspace` (snapshot, record written or removed, materialise, resume). Move the two-writer race from trace matching to a named point; the store digest must stay unchanged.
5. **The four-writer matrix** (INT-0007 AC-5):
   - four store writers, each committing ops from a shared base, over seeds, modes and every crash step, with and without a process crash;
   - closure, durability, atomicity and `fsck` invariants;
   - the existing store negative controls must be caught again with four writers;
   - plus the real four-process run on NTFS and ext4.
6. **The reach table:**
   - every matrix records its point counts;
   - `cargo reach` writes `docs/sprints/s4/sprint-tests/reach.md`, with scenarios as rows and units grouped by sprint as columns;
   - a test fails on any registered point no scenario reaches, or on a stale table.
7. **T-019:** `undo`, `restore` and `new` return their snapshot's report, and `praps` prints it, with a CLI test per command.
8. **Docs:**
   - a spec page, `docs/spec/simulation.md`, covering points, the scheduler, the process crash and the reach table;
   - INT-0005 revised;
   - credits;
   - the backlog. Test selection from reach becomes a new backlog item.

## Artifacts
- This report is the only artifact. The sources are linked above.
