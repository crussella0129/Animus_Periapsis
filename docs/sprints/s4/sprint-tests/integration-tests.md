# Sprint 4 Integration Test Results

These are the results at tested head `5dfb418`, which holds the [code review](code-review.md) fixes. Every run is in [test-matrix-log.md](test-matrix-log.md), under the section named here. The two legs ran at the same time on one 24-thread host, which slowed both; the durations below are under that load.

### Canonical runner on the Windows host and in WSL
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-1 to AC-4, AC-8 to AC-10
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-4, AC-6
- **Scope:** these are the local halves only. The GitHub matrix runs at this sprint's checkpoint, and the next sprint records it (backlog T-020), as T-001 did for Sprint 3.

| Run | System | Result |
|---|---|---|
| `int_ci_windows` | Windows 11 Pro 10.0.26300, NTFS working checkout, rustc 1.99.1 | `cargo ci`: PASS for fmt, clippy, test, reach, policy and book; exit 0 in 2,142 s; 473 tests passed; 0 tracked files changed |
| `e2e fresh_clone_ci` | Ubuntu 26.04.1 (WSL2), fresh ext4 clone, rustc 1.99.1 | cold `cargo ci`: all six steps PASS, exit 0 in 1,351 s; 474 tests passed; 0 tracked files changed |
| `int_reach_windows`, `int_reach_linux` | both | `cargo reach` after the test run rewrote `docs/reach.md` byte for byte: 0 changes on either system. The same table is therefore generated on both, and the `reach` step of `cargo ci` passed on both. All five scenario files were present after `cargo ci`. |
| `int_deny_windows`, `int_deny_linux` | both, cargo-deny 0.20.2 | advisories, bans, licenses and sources ok, exit 0. cargo-deny was installed in WSL this sprint, which closes Sprint 3's critique C-003. No dependency was added. |
| `cargo policy` (inside `cargo ci`) | both | every rule PASS, including the new `points` rule |

### The four-writer matrix
- **Intents:**
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1 to AC-3, AC-5
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1 to AC-3
- **Where:** sections `int_writers_matrix_windows` and `int_writers_matrix_linux`. Every count and the digest are identical on both systems.

**The scenario.**
1. `init` and a base op.
2. Four `sim::Processes` share one `SimFs`, each with its own `Store`.
3. Each process commits an op on the base:
   - process 1 commits its op twice, a retry that reaches `store.commit_op.present_synced`;
   - processes 2 and 3 list the ops, then commit a second op that names their first op and every head they listed.

**Seeds.**
- **Odd seeds force the parent race with three orderings.** Process 2's second op names process 0's op while that op is visible and not yet durable.
- **Even seeds run without orderings.** The seed alone interleaves them.

**Cells.** 40 seeds × 2 modes × every global crash step × {no kill, one kill}. Kills:
- in odd seeds, process 0 is killed at its held rename;
- in even seeds, a seeded process is killed at a seeded step within its counted steps.

**The oracle, per cell:**
- torn and closure;
- every acknowledged op and its objects present;
- every present op attempted;
- `store.heads()` equal to the heads of the logged ops that are present;
- layout and stray names;
- `fsck` clean, for the correct store.

| Variant (strict mode) | Cells | Result |
|---|---:|---|
| Correct, both modes | 44,836 (11,616 + 10,802 per mode) | 0 violations of any kind, 0 `fsck` problems; 6,552 cells killed a process |
| `PointerFile` | 23,886 | caught: heads 16,302, stray 23,064 |
| `UnsyncedObjects` | 20,172 | caught: closure 52,481, durability 141,608, torn 144,176 |
| `UnsyncedInit` | 20,832 | caught: closure 5,982, durability 207,914, layout 17,753 |
| `WriteInPlace` | 20,427 | caught: torn 6,758 |
| `UnsyncedReferences` | 20,012 | caught: closure 12. In 100 odd cells, process 0's op was not durable at process 2's rename. |

Each count above is the number of violations, summed over cells; one cell can hold several.

**Named tests:**
- `writers_matrix_finds_no_violations`;
- `writers_forced_race_runs`: every odd seed in both modes;
- `writers_overlap_in_some_even_seed`: since CR-01, the overlap is read from the trace;
- `writers_kills_happen`;
- `writers_retry_reaches_present`: 196 cells crashed during the retry, and the op stayed acknowledged and present;
- `writers_catch_*`, the five controls above;
- `writers_matrix_is_reproducible`: digest `blake3:253b1eabc4054f7aebfb6edd2e907fdaccbc76549158410a47fedc610ba0855c` over 44,836 cells, asserted, and equal on both systems.

**Budget.** The plan allowed 10 minutes for the `writers` binary on the Windows host:

| System | Binary wall time | Correct matrix alone |
|---|---:|---:|
| Windows | 133 s | 96 s |
| Linux | 176 s | 123 s |

Both are well inside the budget, even with the other leg running. Seeds run in parallel worker threads, and cells are sorted into canonical order before the digest.

**What a pass shows, and what it does not.** The `UnsyncedReferences` control loses closure in the forced-race cells, which its test requires. So the parent re-sync in `commit_op` is load-bearing for several writers, and the odd-seed orderings are what reach it on purpose. INT-0007's Consequences record this. The test does not split the 12 losses by seed parity, so nothing here claims the even seeds found none.

The matrix shows the store is safe for four writers on one store, through a simulated `SimFs`. On a real filesystem it is evidenced by `real_four_processes_lose_nothing` ([e2e-tests.md](e2e-tests.md)), which is not reproducible.

### The existing matrices, re-run with points planted
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1, AC-3
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1 to AC-4
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1
- **Where:** sections `int_crash_matrix_*`, `int_snapshot_matrix_*` and `int_materialise_matrix_*`.
- **Why re-run:** T-002 moved `SimFs` to a `Mutex`, and T-006 planted 21 points in the code these matrices run. A point is not a syscall, so every count and every digest must be Sprint 3's. They are, on both systems:

| Matrix | Cells | Digest | Correct | Controls |
|---|---:|---|---|---|
| Store | 32,840 | `blake3:d36b8c9c…ae812` | 0 violations in both modes; 280 view races and 40 parent races, as in Sprint 3 | `PointerFile`, `UnsyncedInit` (strict), `UnsyncedObjects`, `UnsyncedReferences`, `WriteInPlace`: Sprint 3's counts |
| Snapshot | 25,360 | `blake3:c1e39cb1…13ea3` | 0 violations | `UnsyncedObjects`: Sprint 3's counts |
| Materialise | 40,960 | `blake3:6274344b…a1ac` | 0 violations in both modes; 7,998 cells resumed a record; the second-crash pass gave 118,306 cells and 0 violations; the Windows-rules variant gave 0 violations | `NoRecord`: 2,800 and 2,820 working-copy violations |

`two_writer_race_fires_at_the_named_point` (store `crash.rs`) shows the Sprint 1 race now fires from a named point, in every seed, with the store digest unchanged.

### The reach table
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-6
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-9, AC-10
- **What it is:** `docs/reach.md`, generated from the five scenario files the matrices wrote during `cargo ci`'s test step.
- **Result:**
  - All 21 registered points are reached.
  - Each later scenario reaches every earlier unit it calls into. This is the runs × units triangle:

| Scenario | Reaches |
|---|---|
| store (S1), two writers (S1) | the four S1 store units |
| Snapshot (S2) | the S1 store units and S2 `workspace::snapshot` |
| Materialise (S3) | S1, S2 and the three S3 units |
| four writers (S4) | the four S1 store units, for example 542,894 hits on `store::commit_op` and 1,963,196 on `store::put_object` |

The four-writer row reaches no workspace unit, as expected: it drives the store directly.

- **Probes:**
  - **Unregistered point:** the Windows probe planted an unregistered point, and `cargo policy` exited 1.
  - **Target directory outside the checkout:** the WSL probe ran `cargo reach` with `CARGO_TARGET_DIR` outside the checkout. It exited 1, saying that `xtask` was not inside the checkout's target directory.
  - **Stale table:** this is not a probe. It is the unit test `reach_check_fails_on_a_stale_table`, which replaced a probe that would have rerun all of `cargo ci`.

### The `sim` tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Where:** sections `int_sim_points_and_processes_*`.
- **Result:** `points.rs` (10), `processes.rs` (16) and `threads.rs` (3) passed on both systems.
