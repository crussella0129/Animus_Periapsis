Finalized - DO NOT EDIT

# Sprint 4 Build Plan

## Intents
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered:
  - AC-1, determinism: with points and interleavings, and the three Sprint 3 digests asserted unchanged;
  - AC-2, the crash matrix: extended to four writers and a process crash;
  - AC-3, negative controls under four writers;
  - AC-6 (new), reach: every registered point reached; the reach table regenerated and checked.

  AC-5 (network faults) waits for sync.
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — state: active; acceptance criteria covered:
  - AC-5: four writers committing ops concurrently, with no corrupt record and no lost op, under crash injection and a process crash. Checked in simulation, and on NTFS and ext4;
  - AC-1 to AC-3 with several writers.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered:
  - AC-7's report: every command that snapshots reports names with no UTF-8 form (backlog T-019);
  - AC-1 for this sprint's code.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — state: active; acceptance criteria covered: AC-1, kept through the planted points and the changed results.
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: active; acceptance criteria covered: AC-9 and AC-10, now with `cargo reach` as the fourth flagless project command; AC-5 credits.
- [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — through T-001 (backlog T-017) only.

## Schema Tree
- Sprint Goal: several writers coordinated by named points, and a checked table of what each scenario reaches
  - Carried forward
    - T-001: Record the Sprint 3 checkpoint's GitHub Actions runs and owner sign-offs (backlog T-017)
  - Simulation
    - T-002: A thread-safe `SimFs`, with the Sprint 3 digests asserted
    - T-003: Named points and the point registry
    - T-004: The `points` policy rule
    - T-005: `sim::Processes`, the deterministic scheduler with orderings and kills
  - Storage and working copy
    - T-006: Plant and register points in `store` and `workspace`, and move the two-writer race to a named point
    - T-007: The four-writer matrix and the four-process real-filesystem test
  - Reach
    - T-008: Reach recording, `cargo reach`, the `reach` step in `cargo ci`, and the simulation spec
  - Working copy
    - T-009: Report skipped names from every command that snapshots (backlog T-019)
  - Documentation
    - T-010: Consequences, credits, README and backlog

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
    - for every workflow run of PR #5's last head `4f05dd1` and of the `main` merge commit `7b2fd59`: head SHA, event, conclusion;
    - every job, with its conclusion and runner OS;
    - the `rustc` version, the test count of each `cargo ci` leg, and the `cargo deny` result.
  - **WHEN** an owner sign-off has not been given in the conversation, **THEN** the record **SHALL** say it is pending, and backlog T-017 **SHALL** be replaced by a carried-forward task. No sign-off **SHALL** be recorded without the owner's words.

### T-002: A thread-safe `SimFs`, with the Sprint 3 digests asserted
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/sim/src/simfs.rs, crates/sim/src/fs.rs, crates/sim/tests/contract.rs, crates/store/tests/crash.rs, crates/store/tests/common/matrix.rs, crates/workspace/tests/crash.rs, crates/workspace/tests/materialise_crash.rs, other tests that capture `Rc<SimFs>` in a hook
- **Depends on:** (none)
- **Acceptance criterion:** INT-0005 AC-1, determinism unchanged by the refactor (critique C-008, C-018).
- **Success criterion (EARS):**
  - **WHEN** a `SimFs` is shared through `Arc` by several threads, **THEN**:
    - it **SHALL** be `Send + Sync`;
    - its `Hook` **SHALL** be `Send`;
    - `Fs` **SHALL** be implemented for `Arc<T>`.
  - **WHEN** a thread panics while it holds the state lock, **THEN** later lock calls **SHALL** recover the state, not panic on the poison, so the first panic is the one reported.
  - **WHEN** the reproducibility tests run, **THEN** they **SHALL** assert the full Sprint 3 digests as constants, not only print them:
    - store `blake3:d36b8c9c77744d485a362ef3d94cebe40dcb67ca7260eb409f5430c2a13ae812` over 32,840 cells;
    - Snapshot `blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3` over 25,360 cells;
    - Materialise `blake3:6274344b358b15cb24d182a681c592ccad66a82564fd40c5cf45e4fcb666a1ac` over 40,960 cells.
- **Notes:**
  - State moves from `RefCell` to `Mutex`. The lock is never held while a hook runs.
  - The hook stays taken out of its slot while it runs, so its own syscalls do not run it again.
  - No behaviour changes. This task's only evidence is the three asserted digests.

### T-003: Named points and the point registry
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/sim/src/fs.rs, crates/sim/src/simfs.rs, crates/sim/src/real.rs, crates/sim/src/point.rs, crates/sim/src/lib.rs, crates/sim/tests/points.rs, crates/workspace/tests/real_edits.rs (its `Under` wrapper)
- **Depends on:** T-002
- **Acceptance criterion:**
  - INT-0005 AC-1 and AC-2: points change neither determinism nor the crash model;
  - INT-0005 AC-6: reach is counted per point.
- **Success criterion (EARS):**
  - **WHEN** `Fs` is implemented, **THEN** `point(&self, name: &'static str)` **SHALL** have no default body, so every implementation and forwarder states what it does (critique C-007):
    - `RealFs` and test wrappers do nothing;
    - `&T`, `Rc<T>` and `Arc<T>` forward.
  - **WHEN** code calls `point(name)` on a `SimFs`, **THEN**:
    - the call **SHALL NOT** be counted as a syscall, traced, or consume the seed;
    - `SimFs::reach()` **SHALL** report the name with its count, sorted by name;
    - each action attached with `SimFs::on_point(name, action)` **SHALL** run, in the order attached.
  - **WHEN** an action for a name is running on a thread and code on that thread reaches the same name again, **THEN** the point **SHALL** be counted and its actions **SHALL NOT** run again.
  - **WHEN** a crash has happened, **THEN** `point` **SHALL** count and **SHALL NOT** run actions.
  - **WHEN** `recover()` builds the restarted filesystem, **THEN** it **SHALL** have no hook, no actions, no switch callback and zero reach.
  - **WHEN** the three Sprint 3 matrices run after this task, **THEN** their asserted digests **SHALL** still hold.
- **Notes:**
  - Actions are `Fn + Send + Sync` in an `Arc`. Any state they keep (such as a "fired" flag) is atomic.
  - The `Hook` keeps running only at syscall boundaries, as now.
  - An action's own syscalls are ordinary syscalls of the process that reached the point.
  - `sim::Point { name, unit, sprint }` is `const`-constructible. Its fields:
    - `unit` is the function or module of a boundary crate the point marks, as `crate::name`;
    - `sprint` is the sprint whose build plan delivered that unit (INT-0005 defines "unit").

### T-004: The `points` policy rule
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** crates/xtask/src/policy.rs
- **Depends on:** T-003
- **Acceptance criterion:** INT-0005 AC-6: the registry is the complete table of points.
- **Success criterion (EARS):**
  - **WHEN** `cargo policy` runs, **THEN** the `points` rule **SHALL** read the `src/` files of the boundary crates other than `sim`, with comments removed, so a point in a comment or in rustdoc does not count. The trait and its forwarders live in `sim`.
  - **WHEN** it reads them, **THEN** it **SHALL** fail and name the point and file in each of these cases:
    - a `.point("…")` literal (whitespace and line breaks inside the parentheses allowed) whose name no registry declares;
    - a registered name that no literal uses;
    - a name registered twice;
    - a name not of the form `crate.function.stage`, in lowercase ASCII with `_`, whose first segment is not the crate's name;
    - a `.point(` call whose argument is not a string literal;
    - a registry entry not of the one accepted form, `Point { name: "…", unit: "…", sprint: N }`, inside `pub const POINTS: &[Point] = &[…];` in the crate's `src/points.rs`;
    - a `sprint` greater than the highest `docs/sprints/sN` or less than 1.
  - **WHEN** the clean fixture and the real workspace are checked, **THEN** the rule **SHALL** pass.

### T-005: `sim::Processes`, the deterministic scheduler with orderings and kills
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** crates/sim/src/processes.rs, crates/sim/src/simfs.rs, crates/sim/src/lib.rs, crates/sim/tests/processes.rs
- **Depends on:** T-003
- **Acceptance criterion:**
  - INT-0005 AC-1 for interleaved runs;
  - INT-0005's process-crash Consequence: a killed process's unsynced state stays visible to the others;
  - the means for INT-0007 AC-5.
- **Success criterion (EARS):**
  - **WHEN** `Processes::run(fs, seed, closures)` runs N closures, **THEN**:
    - exactly one **SHALL** execute at any time;
    - each thread **SHALL** start only once it holds the baton;
    - the next runner **SHALL** be drawn from the seed among the processes able to run. A draw happens at each of these:
      - every syscall boundary and point boundary of the running process;
      - its return;
      - its kill;
      - its wait on an ordering.
    - the run **SHALL** return each process's result in input order: its value, `Killed`, or its panic.
  - **WHEN** two runs have the same seed, closures and `SimFs` seed, **THEN** their traces, final states and results **SHALL** be identical. **WHEN** 20 seeds are run, **THEN** at least two distinct interleavings **SHALL** appear.
  - **WHEN** an ordering is given, **THEN** process *q*, on reaching its *m*-th point B, **SHALL** wait. The predecessor is either a point, `((p, A, n), (q, B, m))`, or the end of a process, `((p, End), (q, B, m))`:
    - for a point, *q* waits until process *p* has reached its *n*-th point A. A process counts as having reached a point as soon as it arrives, even if it then waits there;
    - for an end, *q* waits until process *p* has ended.
  - **WHEN** a process arrives at a point under `Processes`, **THEN** these **SHALL** happen in this order:
    1. the point is counted;
    2. a kill set for that arrival takes effect;
    3. the process waits on every ordering whose successor it is;
    4. once released, the point's actions run.

    So an action sees the state as it is when the process goes on (`point_events_are_ordered`).
  - **WHEN** the system has crashed, or the predecessor's process has ended (returned, killed or panicked) without reaching A, **THEN** the wait **SHALL** be released. After a crash the waiter's next syscall fails.
  - **WHEN** every live process waits on a live process, **THEN** the run **SHALL** stop with an ordering-deadlock error naming the waiting points. This is a test bug, never a matrix outcome.
  - **WHEN** a kill is set for process *p*, either at its *k*-th boundary or at its *n*-th arrival at point A, **THEN** *p* **SHALL** stop there, make no further syscall, and report `Killed`. Its writes, synced or not, **SHALL** stay visible to the others. **WHEN** the system then crashes, **THEN** `recover` **SHALL** settle them by the seed like any pending state.
  - **WHEN** a process panics, **THEN** every other process **SHALL** be unwound at its next boundary with the kill marker. The run **SHALL** return the first panic after every thread has joined, with no deadlock.
  - **WHEN** a crash point is reached in an interleaved run, **THEN** every process's later syscalls **SHALL** fail with the simulated crash, and the run **SHALL** finish.
- **Notes:**
  - Each process is an OS thread. A baton (`Mutex` and `Condvar`) lets exactly one run at a time.
  - `SimFs` calls a shared switch callback, held in an `Arc`, at every syscall and point boundary.
  - Kills and unwinds use `resume_unwind` with a private marker, caught by the runner.
  - Point orderings are the RocksDB `LoadDependency` form, keyed by process and occurrence, so a code change elsewhere cannot shift them.

### T-006: Plant and register points in `store` and `workspace`, and move the two-writer race to a named point
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/store/src/lib.rs, crates/store/src/points.rs, crates/store/tests/store.rs, crates/store/tests/common/matrix.rs, crates/workspace/src/lib.rs, crates/workspace/src/materialise.rs, crates/workspace/src/record.rs, crates/workspace/src/points.rs, crates/workspace/tests/workspace.rs
- **Depends on:** T-003, T-004
- **Acceptance criterion:**
  - INT-0005 AC-1: digests unchanged by planting points;
  - INT-0005 AC-6: the points exist and are registered;
  - INT-0007 AC-1 to AC-3: the two-writer race preserved.
- **Success criterion (EARS):**
  - **WHEN** the store runs, **THEN** it **SHALL** call these points (unit and building sprint in brackets):
    - `store.init.layout_created`, after `init` creates its directories (`store::init`, 1);
    - `store.sync_layout.synced`, after the layout sync that `init` and `open` share (`store::sync_layout`, 1);
    - `store.commit_op.references_resynced`, after `commit_op`'s re-sync of the view and parents (`store::commit_op`, 1);
    - inside the shared `write_file`, with the name chosen by whether it writes an object or an op (`store::put_object` and `store::commit_op`, 1):
      - `store.put_object.present_synced` and `store.commit_op.present_synced`, after a file already present is synced again;
      - `….temp_written`, after the data is written to the temporary file;
      - `….temp_synced`, after it is synced;
      - `….renamed`, after the rename;
      - `….dir_synced`, after the directory sync.

    Each point is a string literal at its call site.
  - **WHEN** the working copy is written or snapshotted, **THEN** `workspace` **SHALL** call:
    - `workspace.snapshot.scanned` and `workspace.snapshot.committed` (`workspace::snapshot`, 2);
    - `workspace.record.written` and `workspace.record.dropped` (`workspace::record`, a module, 3);
    - `workspace.apply.path_written` and `workspace.apply.path_removed` (`workspace::materialise`, a module, 3);
    - `workspace.resume.started` and `workspace.resume.dropped` (`workspace::resume`, 3).
  - **WHEN** the plan is locked, **THEN** each registered point's expected scenario **SHALL** be as listed:
    - store: every `store.*` point except `commit_op.present_synced`. ForkMerge reaches `put_object.present_synced`;
    - four writers: `store.commit_op.present_synced`, through process 1's retry;
    - Snapshot: `workspace.snapshot.*`;
    - Materialise: `workspace.record.*`, `workspace.apply.*` and `workspace.resume.*`. `resume.dropped` is reached where a crash falls between the record's write and the target op's commit.

    **WHEN** a build run finds a point unreached, **THEN** a new scenario **SHALL** be extended to reach it. The point **SHALL** be removed instead only if its unit keeps at least one reached point (INT-0005 Consequences). Either change is recorded in the build notes. No Sprint 3 scenario is changed, so its digest stays fixed.
  - **WHEN** `put_object` or `commit_op` writes a new file in the correct store, **THEN** it **SHALL** reach `temp_written`, `temp_synced`, `renamed` and `dir_synced` once each, in that order. **WHEN** the file is already present, **THEN** it **SHALL** reach only `present_synced`.
  - **WHEN** the two-writer scenario runs, **THEN** writer B **SHALL** be fired by an action on `store.put_object.renamed` (the view race) or `store.commit_op.renamed` (the parent race). The action re-checks that the target is visible and not durable, as `install_race` does now, and reads no trace. The point sits immediately after `rename` returns, with no syscall in between, so B's syscalls take the same numbers as before. The store digest **SHALL** equal its asserted constant.
  - **WHEN** all three Sprint 3 matrices run, **THEN** their asserted digests **SHALL** hold.
  - **WHEN** `cargo policy` runs, **THEN** the `points` rule **SHALL** pass.

### T-007: The four-writer matrix and the four-process real-filesystem test
- **Intent:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/store/tests/common/writers.rs, crates/store/tests/writers.rs, crates/store/tests/real_writers.rs, crates/xtask/src/ci.rs
- **Depends on:** T-005, T-006
- **Acceptance criterion:**
  - INT-0007 AC-5: at least four processes, no corrupt records, no lost ops, under crash injection;
  - INT-0007 AC-1 to AC-3 with several writers;
  - INT-0005 AC-2 and AC-3.
- **Success criterion (EARS):**
  - **The scenario.**
    - `init` and a base op, then four processes over one `SimFs`, each opening its own `Store` with its own seed.
    - Each process records a state of its own (distinct content) and commits a first op whose parent is the base.
    - Processes 2 and 3 then list the ops their store sees and commit a second op. Its parents are their own first op and every other op they listed that no listed op names as a parent.
    - Process 1 commits its first op twice, as a retry after a lost acknowledgement would. The second commit takes the already-present path, so it reaches `store.commit_op.present_synced`.
    - In odd seeds, orderings force the parent race:
      - (process 1's end), (process 3's end) and (process 2's first `store.commit_op.dir_synced`) before (process 0's first `store.commit_op.temp_written`). So no other `open` or `ops/` sync can make process 0's op durable during the hold, and no second op of process 3 can take process 0's op out of the heads;
      - (process 0's first `store.commit_op.renamed`) before (process 2's `test.writers.listing`);
      - (process 2's second `store.commit_op.renamed`) before (process 0's first `store.commit_op.renamed`).

      So process 0 is held between its op's rename and its `ops/` sync while process 2 lists and names that op. In the correct store, process 2's parent re-sync then makes the op durable. `UnsyncedReferences` skips that re-sync, so a crash between process 2's rename and its `ops/` sync can keep process 2's op and lose process 0's.
    - The listing point uses the reserved `test.` prefix. Points with that prefix are never registered and are dropped when reach is recorded.
    - Even seeds use no orderings.
  - **The log.**
    - It lives outside the process threads and survives kills.
    - An op is *attempted* when a process logs it before calling `commit_op`, and *acknowledged* when `commit_op` returned `Ok`. Objects are logged the same way.
    - The *in-flight* ops are those attempted but not acknowledged, in any process, including a killed one.
    - All three are sets of op ids: in-flight = attempted ids − acknowledged ids. An op acknowledged once stays acknowledged even if a retry of it is then interrupted.
  - **WHEN** a cell runs, over 40 seeds × 2 modes × every global crash step × {no kill, one kill}, **THEN** the recovered store **SHALL** satisfy:
    - closure;
    - acknowledged ⊆ present ⊆ acknowledged ∪ in-flight (plus the base);
    - every acknowledged object present and valid;
    - `store.heads()` equal to the heads computed from the logged ops (ids and parents) restricted to those present;
    - `fsck` with no fault, where leftovers are allowed.
  - **The kill.**
    - In even seeds, the killed process and its step *k* are drawn from the seed. *k* lies within that process's boundary count in the seed's uncrashed run.
    - In odd seeds, process 0 is killed on arrival at its held `store.commit_op.renamed`. A later process then sees the dead one's unsynced op.
    - **WHEN** a kill run is uncrashed, **THEN** it **SHALL** report `Killed` for the chosen process. The number of cells that killed a process **SHALL** be printed.
  - **WHEN** an odd seed's uncrashed run is examined in the correct store, **THEN**:
    - process 2's second op **SHALL** name process 0's op;
    - when process 2 is released from `test.writers.listing`, an action there **SHALL** find process 0's op name visible and not durable;
    - the number of odd cells in which it is still not durable at process 2's second `renamed` **SHALL** be printed per variant, and **SHALL** be non-zero for `UnsyncedReferences`. **WHEN** any even seed's uncrashed run is examined, **THEN** at least one of them **SHALL** show two commits overlapping.
  - **WHEN** each store variant runs the matrix in strict mode, **THEN** it **SHALL** produce semantic violations (closure, durability, heads or atomicity), as Sprints 1 to 3 required:
    - `PointerFile`, `UnsyncedObjects`, `UnsyncedInit`, `WriteInPlace`: at least one;
    - `UnsyncedObjects`: durability > 0 and torn > 0;
    - `UnsyncedReferences`: closure > 0 in the odd-seed (forced-race) cells.
  - **WHEN** the correct matrix runs twice, **THEN** every cell's recovered state, trace and outcome **SHALL** be equal:
    - one of the two runs is the one `writers_matrix_finds_no_violations` uses, shared through a once-initialised cache, so the matrix runs twice, not three times;
    - cells **SHALL** be ordered canonically by (mode, seed, kill setting, crash step) before digesting, whatever order they ran in;
    - one digest over all cells **SHALL** be printed, compared across Windows and Linux, and asserted as a constant in T-008.
  - **The real filesystem.** The test binary re-runs itself as four child processes, each committing 25 ops of its own distinct content to one temporary store; each op's parents are its own previous op and the base.
    - The child mode needs both an exact test-name argument and the variable `PERI_WRITER_CHILD`. `cargo ci` scrubs that variable as it scrubs `PERI_GOLDEN_CREATE`.
    - Each child prints the ids and parents of its acknowledged ops.
    - **WHEN** every child has exited, **THEN**:
      - every reported op **SHALL** be present and valid;
      - `fsck` **SHALL** report no fault;
      - `store.heads()` **SHALL** equal the heads computed from the children's reports.
    - Children share no object, so no file is renamed over one another reads. A sharing error is a test failure, not retried.
- **Notes (cost, critique C-001):**
  - About 200 syscalls a run. 40 seeds × 2 modes × about 200 steps × 2 kill settings ≈ 32,000 correct cells, run twice. The five variants run strict only, about 16,000 cells each.
  - The budget is 10 minutes wall-clock for the `writers` test binary on the Windows host, recorded in the test evidence.
  - If it is exceeded, these controls apply in order:
    1. run seeds in parallel threads within a test, each with its own `SimFs`;
    2. reuse process threads across cells;
    3. stop a variant's run once its required violation kinds are all found, which still records that they were found.
  - The seed count never drops below INT-0005 AC-2's 40.
  - The budget is recorded, not asserted. If all three controls still miss it, the budget is raised to the measured time, and the raise is recorded in the test report and the test critique's responses.

### T-008: Reach recording, `cargo reach`, the `reach` step in `cargo ci`, and the simulation spec
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** crates/sim/src/reach.rs, crates/store/tests/, crates/workspace/tests/, crates/xtask/src/reach.rs, crates/xtask/src/main.rs, crates/xtask/src/ci.rs, .cargo/config.toml, docs/reach.md, docs/spec/simulation.md, docs/SUMMARY.md
- **Depends on:** T-006, T-007
- **Acceptance criterion:** INT-0005 AC-6; INT-0003 AC-9.
- **Success criterion (EARS):**
  - **What a scenario records.** Each scenario's reach file comes from one designated test, which runs the correct store only:
    - store (`crash_matrix_finds_no_violations`);
    - two writers (the same test, as its own scenario);
    - four writers (`writers_matrix_finds_no_violations`);
    - Snapshot (`snapshot_matrix_finds_no_violations`);
    - Materialise (`materialise_matrix_finds_no_violations`, Unix rules).

    **WHEN** that test finishes, **THEN** it **SHALL** write the scenario's name, the sprint that added it, and the point counts summed over its cells to `$CARGO_TARGET_TMPDIR/reach/<scenario>.txt`, through a temporary file and a rename.
  - **The recorder** is one function in `sim` (`sim::reach::record`, writing through `RealFs`), shared by the store and workspace tests and tested once in `sim` (`reach_record_round_trips`).
  - **Which runs count:**
    - reach is read from the scenario's own `SimFs` after its run;
    - for a scenario that models a next command on the recovered filesystem, reach is also read from that copy right after the command returns, before the oracle runs any check. The command is `init` and `snapshot` for Snapshot, and the resuming `open` for Materialise. Store, two writers and four writers model none;
    - the oracle's checks (`fsck`, inspection, its own `init`, `open` or `snapshot`) never count;
    - names with the reserved `test.` prefix may appear in a reach file. `cargo reach` ignores them and fails on any other unregistered name (`reach_ignores_test_points`).
  - **WHEN** `cargo reach` runs, **THEN** it **SHALL** take no arguments and write `docs/reach.md`:
    - one row per scenario, ordered by sprint;
    - one column per unit, grouped and ordered by the sprint that built it;
    - each cell holding the number of point hits;
    - a final list of every point with its total.

    Two runs over the same files **SHALL** write identical bytes. Like `ci`, it **SHALL** refuse to run out of tree.
  - **WHEN** any of these holds, **THEN** `cargo reach` **SHALL** fail and name it, writing nothing:
    - a registered point has no hits;
    - a scenario's file is missing;
    - a file names an unregistered point outside the reserved `test.` prefix.
  - **WHEN** `cargo ci` runs, **THEN** it **SHALL**:
    - clear the reach directory before `test`;
    - after `test`, run a `reach` step that generates the table in memory, compares it with `docs/reach.md`, fails on any difference, and writes nothing;
    - have six steps: fmt, clippy, test, reach, policy, book.
  - **WHEN** `docs/reach.md` is read by `reach_table_shows_the_triangle`, **THEN**:
    - the four-writer row **SHALL** have non-zero hits in the Sprint 1 `store::commit_op` and `store::put_object` columns;
    - the Materialise row **SHALL** have non-zero hits in the Sprint 1 store columns and the Sprint 2 `workspace::snapshot` column.
  - **WHEN** the spec is written, **THEN** `docs/spec/simulation.md` **SHALL** state:
    - what a point is and is not;
    - the registry and its policy rule;
    - the scheduler's draws, orderings and kills;
    - the process-crash model;
    - that adding a point changes multi-process interleavings, and so the four-writer digest, but no single-process digest, with the expected response (critique C-015);
    - how the reach table is generated and checked.
  - **WHEN** the four-writer digest is known on both systems, **THEN** it **SHALL** be asserted as a constant in `writers_matrix_is_reproducible`.

### T-009: Report skipped names from every command that snapshots (backlog T-019)
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/workspace/src/lib.rs, crates/praps/src/main.rs, crates/praps/tests/cli.rs, crates/workspace/tests/history.rs
- **Depends on:** T-006
- **Acceptance criterion:** INT-0004 AC-7: a name with no UTF-8 form is never "skipped without a report".
- **Success criterion (EARS):**
  - **WHEN** `log`, `undo`, `restore` or `new_change` snapshots a working copy holding entries it reports and leaves alone, **THEN** it **SHALL** return them with its result, in `status`'s sorted form. They come from the command's first snapshot only, so `undo`'s second snapshot in `restore_view` adds no duplicate.
  - **WHEN** such a command fails after its snapshot (`NothingToUndo`, hazards, a blocking entry, an unknown op), **THEN** its error **SHALL** carry the same entries.
  - **WHEN** `praps log`, `praps undo`, `praps restore OP` or `praps new` runs over such an entry, **THEN** it **SHALL** print each one to stdout in `status`'s format, after its own output, on success and on failure, and keep its exit code.
  - **WHEN** no entry is reported, **THEN** every command's output **SHALL** be unchanged from Sprint 3.

### T-010: Consequences, credits, README and backlog
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** docs/intents/INT-0005-deterministic-simulation-testing.md, docs/intents/INT-0007-crash-safe-storage.md, docs/prior-art.md, docs/work/tasks.md, README.md, crates/xtask/src/docs.rs, crates/xtask/Cargo.toml
- **Depends on:** T-001 to T-009
- **Acceptance criterion:** INT-0003 AC-5 and AC-10; INT-0005 and INT-0007 Consequences.
- **Success criterion (EARS):**
  - **WHEN** the docs are updated, **THEN**:
    - `docs/prior-art.md` **SHALL** credit fail-rs, RocksDB sync points, FoundationDB BUGGIFY, and shuttle with PCT (Burckhardt et al., ASPLOS 2010), and **SHALL** record the owner's proposal of coded breakpoints and the run × unit triangle;
    - INT-0005's Consequences **SHALL** replace "a process crash is not simulated yet" with the model built;
    - INT-0007's Consequences **SHALL** record what several writers showed;
    - the README **SHALL** document `cargo reach` among the four flagless project commands. Its no-flags sentence becomes "`cargo ci`, `cargo policy`, `cargo book` and `cargo reach` take no flags." `readme_problems` in `crates/xtask/src/docs.rs` **SHALL** require that sentence and a description of `cargo reach`;
    - the `xtask` crate's description and its row in the README's crate table **SHALL** name all four commands.
  - **WHEN** the backlog is updated, **THEN**:
    - T-011 and T-019 **SHALL** move to completed;
    - new tasks **SHALL** record test selection from the reach table, and the move of the remaining trace-text hooks (`crates/workspace/tests/workspace.rs`, `materialise.rs`) to named points (critique C-020);
    - T-017 **SHALL** be replaced by its carried-forward successor.
