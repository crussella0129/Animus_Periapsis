# Sprint 4 Build Plan

## Intents
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered:
  - AC-1, determinism: points and the scheduler included, and the three existing matrix digests unchanged;
  - AC-2, the crash matrix: extended to four writers and a process crash;
  - AC-3, negative controls under four writers;
  - AC-6 (new this sprint), reach: every named point reached, and the reach table regenerated and checked.

  AC-5 (network faults) waits for sync.
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — state: active; acceptance criteria covered:
  - AC-5: four writers committing ops concurrently, with no corrupt record and no lost op, under crash injection and a process crash. This is checked in simulation and on NTFS and ext4.
  - AC-1 to AC-3 re-established with several writers.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered:
  - AC-7's report: `undo`, `restore` and `new` report names with no UTF-8 form (backlog T-019);
  - AC-1 for this sprint's code.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — state: active; acceptance criteria covered: AC-1 kept through T-005's planted points and T-008's changed results.
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — through T-001 (backlog T-017), T-003 (a policy rule) and T-009 (credits) only.

## Schema Tree
- Sprint Goal: several writers coordinated by named points, and a checked table of what each scenario reaches
  - Carried forward
    - T-001: Record the Sprint 3 checkpoint's GitHub Actions runs and owner sign-offs (backlog T-017)
  - Simulation
    - T-002: Named points, the point registry and a thread-safe `SimFs`
    - T-003: The `points` policy rule
    - T-004: `sim::Processes`, the deterministic scheduler with process kills
  - Storage and working copy
    - T-005: Plant and register points in `store` and `workspace`, and move the two-writer race to a named point
    - T-006: The four-writer matrix and the four-process real-filesystem test
  - Reach
    - T-007: Reach recording, `cargo reach`, and the simulation spec
  - Working copy
    - T-008: Report skipped names from `undo`, `restore` and `new` (backlog T-019)
  - Documentation
    - T-009: INT-0005 Consequences, credits and backlog

## Execution Sequence

### T-001: Record the Sprint 3 checkpoint's GitHub Actions runs and owner sign-offs
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- **Touches:** docs/sprints/s4/sprint-tests/ci-sprint3.md, docs/intents/INT-0001-periapsis-charter.md, docs/intents/INT-0002-typed-acceptance-criteria.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0004-windows-and-linux.md, docs/intents/INT-0006-format-v1.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:**
  - the GitHub-matrix halves of Sprint 3's checkpoint: INT-0003 AC-2 to AC-4, INT-0004 AC-1, and `cargo deny` on Linux;
  - owner sign-offs: INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6, INT-0004 AC-5.
- **Success criterion (EARS):**
  - **WHEN** the record is written, **THEN** `ci-sprint3.md` **SHALL** name each of the following, all taken from `gh run view` output:
    - for every workflow run of PR #5's head and of the `main` merge commit `7b2fd59`: head SHA, event, conclusion;
    - every job, with its conclusion and runner OS;
    - the `rustc` version, the test count of each `cargo ci` leg, and the `cargo deny` result.
  - **WHEN** an owner sign-off has not been given in the conversation, **THEN** the record **SHALL** say it is pending, and backlog T-017 **SHALL** be replaced by a carried-forward task. No sign-off **SHALL** be recorded without the owner's words.

### T-002: Named points, the point registry and a thread-safe `SimFs`
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/sim/src/fs.rs, crates/sim/src/simfs.rs, crates/sim/src/lib.rs, crates/sim/src/point.rs, crates/sim/tests/contract.rs, crates/sim/tests/points.rs, test code that captures `Rc<SimFs>` in a hook (crates/store/tests/common/matrix.rs, crates/workspace/tests/)
- **Depends on:** (none)
- **Acceptance criterion:**
  - INT-0005 AC-1: determinism, including points;
  - INT-0005 AC-2: the crash model unchanged by points;
  - INT-0005 AC-6: reach is recorded per point.
- **Success criterion (EARS):**
  - **WHEN** code calls `Fs::point(name)` on `RealFs`, **THEN** it **SHALL** do nothing and make no syscall.
  - **WHEN** code calls `point(name)` on a `SimFs`:
    - **THEN** the call **SHALL NOT** be counted as a syscall, traced, or consume the seed;
    - `SimFs::reach()` **SHALL** report the name with its count;
    - every action attached with `SimFs::on_point(name, action)` **SHALL** run, in the order attached.
  - **WHEN** a crash has happened, **THEN** `point` **SHALL** still count, and **SHALL NOT** run actions.
  - **WHEN** the store, Snapshot and Materialise matrices run after this task, **THEN** their digests **SHALL** equal Sprint 3's:
    - `blake3:d36b8c9c…ae812`;
    - `blake3:c1e39cb1…13ea3`;
    - `blake3:6274344b…a1ac`.
  - **WHEN** a `SimFs` is shared through `Arc` by several threads, **THEN** it **SHALL** be `Send + Sync`. Its hook and point actions **SHALL** be `Send`, and `Fs` **SHALL** be implemented for `Arc<T>`.
- **Notes:**
  - `sim::Point { name, unit, sprint }` is a plain `const`-constructible struct.
  - `sprint` is the sprint that built the code the point marks, not the sprint that planted it, so the reach table's columns follow the units as they were built.
  - The `Mutex` is never held while a hook, action or switch callback runs.

### T-003: The `points` policy rule
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** crates/xtask/src/policy.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0005 AC-6: the registry is the complete table of points.
- **Success criterion (EARS):**
  - **WHEN** `cargo policy` runs, **THEN** the `points` rule **SHALL** fail and name the point and file in each of these cases, in the non-test sources of the boundary crates:
    - a `.point("…")` literal whose name no `POINTS` registry declares;
    - a registered name that no `.point("…")` literal uses;
    - a name registered twice;
    - a name that is not of the form `crate.function.stage`, in lowercase ASCII with `_`;
    - a `.point(` call whose argument is not a string literal.
  - **WHEN** the clean fixture and the real workspace are checked, **THEN** the rule **SHALL** pass.

### T-004: `sim::Processes`, the deterministic scheduler with process kills
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** crates/sim/src/processes.rs, crates/sim/src/simfs.rs, crates/sim/src/lib.rs, crates/sim/tests/processes.rs
- **Depends on:** T-002
- **Acceptance criterion:**
  - INT-0005 AC-1 for interleaved runs;
  - INT-0005's process-crash Consequence: a killed process's unsynced state stays visible to the others;
  - the means for INT-0007 AC-5.
- **Success criterion (EARS):**
  - **WHEN** `Processes::run(fs, seed, closures)` runs N closures, **THEN**:
    - exactly one **SHALL** execute at any time;
    - at every syscall and point boundary of the running process, the next runner **SHALL** be drawn from the seed among the processes that are able to run;
    - the run **SHALL** return each process's result, in input order.
  - **WHEN** two runs have the same seed, closures and `SimFs` seed, **THEN** their traces, final states and results **SHALL** be identical. **WHEN** runs over 20 seeds are compared, **THEN** at least two distinct interleavings **SHALL** appear.
  - **WHEN** an ordering `(A, B)` is given and a process reaches point B before any process has passed point A, **THEN** that process **SHALL** wait until some process passes A. **WHEN** every live process is waiting, **THEN** the run **SHALL** stop and return an ordering-deadlock error naming the waiting points.
  - **WHEN** `kill(p, k)` is set, **THEN** process *p* **SHALL** stop at its *k*-th boundary, make no further syscall, and report `Killed`. Its writes, synced or not, **SHALL** stay visible to the other processes. **WHEN** the system then crashes, **THEN** `recover` **SHALL** settle them by the seed like any pending state.
  - **WHEN** a process panics, **THEN** the run **SHALL** end without deadlock, and return that panic to the caller after every thread has joined.
  - **WHEN** a crash point is reached in an interleaved run, **THEN** every process's later syscalls **SHALL** fail with the simulated crash, and the run **SHALL** finish.
- **Notes:**
  - Each process is an OS thread, and a baton (`Mutex` and `Condvar`) lets exactly one run at a time.
  - `SimFs` calls a shared switch callback at every syscall and point boundary. It is held in an `Arc`, so several threads can be inside it, never taken out of a slot.
  - A kill unwinds the thread with `resume_unwind` and a private marker, which the runner catches.

### T-005: Plant and register points in `store` and `workspace`, and move the two-writer race to a named point
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/store/src/lib.rs, crates/store/src/points.rs, crates/workspace/src/lib.rs, crates/workspace/src/points.rs, crates/store/tests/common/matrix.rs
- **Depends on:** T-002, T-003
- **Acceptance criterion:**
  - INT-0005 AC-1: the digests are unchanged by planting points;
  - INT-0005 AC-6: the points exist and are registered;
  - INT-0007 AC-1 to AC-3: the two-writer race is preserved.
- **Success criterion (EARS):**
  - **WHEN** `put_object`, `commit_op` or `init` reaches a stage, **THEN** it **SHALL** call the registered point. The stages are: temporary file written, data synced, renamed into place, directory synced, and cross-directory re-sync done.
  - **WHEN** a snapshot commits, a working-copy record is written or removed, a path is materialised, or resume starts, **THEN** `workspace` **SHALL** call the registered point.
  - **WHEN** the two-writer scenario runs, **THEN** writer B **SHALL** be fired by `on_point` at A's `rename` point, with no inspection of the trace. The store matrix digest **SHALL** equal Sprint 3's.
  - **WHEN** all three existing matrices run, **THEN** their digests **SHALL** be unchanged.
  - **WHEN** `cargo policy` runs, **THEN** the `points` rule **SHALL** pass, with every planted point registered under its unit and building sprint: store points sprint 1, snapshot points sprint 2, materialise and resume points sprint 3.

### T-006: The four-writer matrix and the four-process real-filesystem test
- **Intent:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/store/tests/common/writers.rs, crates/store/tests/writers.rs, crates/store/tests/real_writers.rs
- **Depends on:** T-004, T-005
- **Acceptance criterion:**
  - INT-0007 AC-5: at least four processes, no corrupt records, no lost ops, under crash injection;
  - INT-0007 AC-1 to AC-3 with several writers;
  - INT-0005 AC-2 and AC-3.
- **Success criterion (EARS):**
  - **WHEN** the scenario runs:
    - `init` is followed by a base op;
    - then four processes, each with its own `Store` and seed, each record a state and commit two ops;
    - each process's first op has the base as parent;
    - each second op has as parents the process's own first op and every other first op that its store lists at that moment.
  - **THEN** every cell of 40 seeds × 2 modes × every global crash step × {no kill, one kill} **SHALL** satisfy:
    - closure;
    - survival of every op acknowledged to any process, killed or not;
    - absence or full validity of every unacknowledged op;
    - recovered heads equal to the heads of the acknowledged ops, extended only by in-flight ops;
    - `fsck` with no fault.

    The killed process and its step are drawn from the seed.
  - **WHEN** an uncrashed four-writer run is examined, **THEN** at least one seed **SHALL** show an interleaving in which two processes' commits overlap: one renames between another's rename and directory sync.
  - **WHEN** each of the five store variants (`PointerFile`, `UnsyncedObjects`, `UnsyncedInit`, `WriteInPlace`, `NoResync`) runs the four-writer matrix, **THEN** each **SHALL** produce at least one violation.
  - **WHEN** the matrix runs twice, **THEN** the digest of every cell's recovered state, trace and outcome **SHALL** be equal. The digest **SHALL** be equal on Windows and Linux.
  - **WHEN** the real test launches four OS processes (the test binary re-run as a child), each committing 25 ops to one temporary store on the real filesystem, **THEN**:
    - every op a child reports as acknowledged **SHALL** be present and valid;
    - `fsck` **SHALL** report no fault;
    - heads **SHALL** equal those derived from all ops.
- **Notes:**
  - The counts are fixed here, before any measurement: 40 seeds, 2 modes, 2 kill settings.
  - If the run exceeds 10 minutes on either system, the seed count is reduced only by a recorded change to this plan's critique responses, never silently.

### T-007: Reach recording, `cargo reach`, and the simulation spec
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/sim/src/point.rs, crates/store/tests/common/, crates/workspace/tests/, crates/xtask/src/reach.rs, crates/xtask/src/main.rs, crates/xtask/src/ci.rs, .cargo/config.toml, docs/reach.md, docs/spec/simulation.md, docs/SUMMARY.md
- **Depends on:** T-005, T-006
- **Acceptance criterion:** INT-0005 AC-6.
- **Success criterion (EARS):**
  - **WHEN** a matrix finishes, **THEN** it **SHALL** write its scenario name, the sprint that added the scenario, and the summed point counts over all its cells to `$CARGO_TARGET_TMPDIR/reach/<scenario>.txt`. The scenarios are: store, two writers, four writers, Snapshot, Materialise.
  - **WHEN** `cargo reach` runs after the tests, **THEN** it **SHALL** write `docs/reach.md`:
    - one row per scenario, ordered by sprint;
    - one column per unit, grouped and ordered by the sprint that built it;
    - each cell holding the number of point hits;
    - a final list of every point with its total.
  - **WHEN** a registered point has no hits in any scenario, **THEN** `cargo reach` **SHALL** fail and name it.
  - **WHEN** a scenario's file is missing, **THEN** `cargo reach` **SHALL** fail and name it.
  - **WHEN** `docs/reach.md` differs from what would be generated, **THEN** `cargo reach --check` **SHALL** fail.
  - **WHEN** `cargo ci` runs, **THEN** it **SHALL**:
    - clear the reach directory before `test`;
    - run `reach --check` as a sixth step after `test`.
  - **WHEN** the generated table is read, **THEN** the Sprint 4 four-writer row **SHALL** reach the Sprint 1 store units, and the Sprint 3 Materialise row **SHALL** reach the Sprint 1 store and Sprint 2 snapshot units.
  - **WHEN** the spec is written, **THEN** `docs/spec/simulation.md` **SHALL** state:
    - what a point is and is not (not a syscall);
    - the registry and its policy rule;
    - the scheduler's switching, orderings and kills;
    - the process-crash model;
    - how the reach table is generated and checked.
- **Notes:** `cargo reach` takes no flags except `--check`. Like `ci` and `policy`, it is an xtask alias, not a `praps` command.

### T-008: Report skipped names from `undo`, `restore` and `new` (backlog T-019)
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/workspace/src/lib.rs, crates/praps/src/main.rs, crates/praps/tests/cli.rs, crates/workspace/tests/
- **Depends on:** T-005
- **Acceptance criterion:** INT-0004 AC-7: a name with no UTF-8 form is never "skipped without a report".
- **Success criterion (EARS):**
  - **WHEN** `undo`, `restore` or `new_change` snapshots a working copy holding entries it reports and leaves alone, **THEN** it **SHALL** return those entries with its result, in the same sorted form `status` returns.
  - **WHEN** `praps undo`, `praps restore OP` or `praps new` runs over such an entry, **THEN** it **SHALL** print each one in `status`'s format, after its own line, and keep its exit code.
  - **WHEN** no entry is reported, **THEN** the output **SHALL** be unchanged from Sprint 3.

### T-009: INT-0005 Consequences, credits and backlog
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** docs/intents/INT-0005-deterministic-simulation-testing.md, docs/intents/INT-0007-crash-safe-storage.md, docs/prior-art.md, docs/work/tasks.md, README.md
- **Depends on:** T-001 to T-008
- **Acceptance criterion:** INT-0003 AC-5 (credits); INT-0005 and INT-0007 Consequences.
- **Success criterion (EARS):**
  - **WHEN** the docs are updated:
    - **THEN** `docs/prior-art.md` **SHALL** credit fail-rs, RocksDB sync points, FoundationDB BUGGIFY, and shuttle with PCT (Burckhardt et al., 2010), and record the owner's proposal of coded breakpoints and the run × unit triangle.
    - INT-0005's Consequences **SHALL** replace "a process crash is not simulated yet" with the model built.
    - INT-0007's Consequences **SHALL** record what several writers showed.
  - **WHEN** the backlog is updated, **THEN**:
    - T-011 and T-019 **SHALL** move to completed;
    - a new task **SHALL** record test selection from the reach table;
    - T-017 **SHALL** be replaced by its carried-forward successor.
