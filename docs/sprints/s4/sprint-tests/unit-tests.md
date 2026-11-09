# Sprint 4 Unit Test Results

- **Tested head:** `9fa59fb` on `dev`, committed, with a clean working tree. It holds the fixes for the Test-phase [code review](code-review.md) (CR-01 to CR-07, in `5dfb418`) and for the [round-1 test critique](critique-round1.md) (C-001 to C-011, in `9fa59fb`). Both legs first ran at `5dfb418` and then again in full at `9fa59fb`. Only the second runs are recorded in [test-matrix-log.md](test-matrix-log.md).
- **Runner:** the `test` step of `cargo ci` (`cargo test --workspace --locked`). It ran in the Windows working checkout and in a fresh WSL ext4 clone (sections `int_ci_windows` and `e2e fresh_clone_ci`).
- **Result:**
  - 0 failed and 0 ignored on both systems;
  - 477 passed on Windows and 478 on Linux; Sprint 3 ended with 407 and 408;
  - the extra Linux test is still `real_symlinked_directories_are_not_followed`, which is compiled for Unix only.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13):
  - `x86_64-pc-windows-msvc`, on Windows 11 Pro 10.0.26300;
  - `x86_64-unknown-linux-gnu`, on Ubuntu 26.04.1 LTS (WSL2 kernel 6.6.114.1).
- **Randomness:**
  - Every new test that varies a run takes an explicit seed through `sim::Rng`. That includes `sim::Processes`, whose seed draws the next runner at every boundary.
  - Points take no seed and draw nothing.
  - The scheduler tests that loop over seeds also require at least one seed of the interleaving they need, such as a waiter arriving first. A change in the draw therefore cannot make them pass vacuously.

## New tests by file

70 new tests, counted by `#[test]` against Sprint 3's tested head `ad22319`.

| File | Sprint 3 | Sprint 4 |
|------|---------:|---------:|
| `sim/tests/points.rs` (new) | 0 | 10 |
| `sim/tests/processes.rs` (new) | 0 | 18 |
| `sim/tests/threads.rs` (new) | 0 | 3 |
| `store/tests/writers.rs` (new) | 0 | 12 |
| `store/tests/real_writers.rs` (new; one test is the child entry point) | 0 | 2 |
| `store/tests/crash.rs` | 7 | 8 |
| `store/tests/store.rs` | 23 | 24 |
| `workspace/tests/materialise.rs` | 26 | 27 |
| `workspace/tests/history.rs` | 12 | 14 |
| `praps/tests/e2e.rs` | 4 | 5 |
| `xtask`: `reach.rs` (new), `points.rs` (new), `main.rs`, `ci.rs`, `policy.rs` | 70 in all | 89 in all (+11, +4, +2, +1, +1) |

## Departures from the test plan

Every test the [test plan](../sprint-plans/test-plan.md) names exists and passed on both systems, except for the two renames below. Some of the tests check something different from the plan's description. Each departure is listed here with the test that covers the plan's wording.

| Plan | What runs instead |
|---|---|
| T-009: six CLI tests, `praps_log_reports_skipped_names`, `praps_undo_reports_skipped_names`, `praps_restore_reports_skipped_names`, `praps_new_reports_skipped_names`, `praps_failed_undo_reports_skipped_names` and `praps_output_unchanged_without_reports` | One test, `praps_commands_report_skipped_names`, mapped under T-009 below. The T-009 completion entry describes it, but did not say it replaces six planned names; this table does. |
| T-009: `failed_undo_carries_reports` | Renamed `failed_commands_carry_reports`. It now covers all four failure kinds the EARS clause names (round-1 C-006). |
| T-005: `processes_return_results_in_input_order` with "values, `Killed` and a panic" | Values and `Killed` only, because a panic ends the run with `Stopped::Panicked` and returns no outcomes. Recorded in the T-005 entry. The panic is `panic_in_a_process_ends_the_run`. |
| T-003: `points_do_not_change_traces` over "the contract scenario" | The `every_kind` scenario, which makes a call of every syscall kind, over 40 seeds × 2 modes. |
| T-006: `workspace_points_mark_each_stage` with "an `undo`" | `restore_view`, the function `undo` and `restore` both call. `undo` itself reaches the same points in the Materialise matrix. |
| T-008: `ci_steps_are_ordered_and_locked` "extended" to the reach directory and `PERI_WRITER_CHILD` | It checks the six steps in order. The clearing is `test_step_clears_the_recorded_reach`. The scrubbing is `test_step_removes_golden_create` and `proptest_variables_are_never_inherited`. |

## By build task

### T-002: `SimFs` shared across threads
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1
- `simfs_is_send_and_sync`: a compile-time check, and an `Arc<SimFs>` used from two threads.
- `poisoned_state_is_recovered`: there are two panics, and later calls keep working after both:
  - one in a hook that holds no lock;
  - one with the state locked, through the doc-hidden `panic_holding_the_lock`.
- `a_hook_that_unwinds_is_put_back` (CR-06): a hook that panics is back in its slot for the next syscall.
- The three Sprint 3 reproducibility tests now assert their digest constants. Those constants are unchanged on both systems, even with the `Mutex` state and the planted points:

  | Test | Cells | Digest |
  |---|---:|---|
  | `matrix_runs_are_reproducible` | 32,840 | `blake3:d36b8c9c…ae812` |
  | `snapshot_matrix_is_reproducible` | 25,360 | `blake3:c1e39cb1…13ea3` |
  | `materialise_matrix_is_reproducible` | 40,960 | `blake3:6274344b…a1ac` |

### T-003: named points
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1, AC-6
- `points_are_not_syscalls`: for every syscall kind, points before, between and after the syscalls leave three things as they were without them:
  - `syscalls()`;
  - the trace;
  - every crash outcome.
- `points_do_not_change_traces`: the `every_kind` scenario gives equal traces and states with and without points, over 40 seeds × 2 modes.
- The reach and action tests:
  - `reach_counts_points`;
  - `on_point_actions_run_in_order`;
  - `on_point_action_is_not_reentered`;
  - `points_count_but_do_not_act_after_a_crash`;
  - `recover_drops_actions_and_reach`;
  - `points_reach_through_every_forwarder` (`&SimFs`, `Rc<SimFs>`, `Arc<SimFs>`);
  - `real_fs_point_is_a_no_op`;
  - `reach_record_round_trips`.

### T-004: the `points` policy rule
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-6
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `policy_points_rule` has eleven failing fixtures. Each must yield one problem with three parts: it starts with the offending file, it names the offending point, unit or call, and it carries the rule's text (round-1 C-008). The fixtures:
  - the plan's eight:
    - an unregistered point;
    - an unused registration;
    - a duplicate;
    - a malformed name;
    - a wrong crate prefix;
    - a non-literal argument;
    - an unrecognised registry entry;
    - a sprint out of range;
  - three added by the code review:
    - `unit-sprints`, one unit under two sprints (CR-04);
    - `path-call`, `Fs::point(…)` (CR-03);
    - `test-code`, a `.point(` after `#[cfg(test)]` (CR-03).

  The rule names registries by their path from the root, `crates/store/src/points.rs`. The registry's own parse error still names `store/src/points.rs`. The commented-out, rustdoc, next-line-literal and clean fixtures pass.
- The four `xtask/src/points.rs` parser tests.
- **The Windows probe:** the leg ran the rule against real code. A `.point("store.probe.unregistered")` planted in `crates/store/src/lib.rs` made `cargo policy` exit 1, naming the point, the file and the registry. The file was then restored, with 0 changed paths (section `probe: a point not registered fails cargo policy`).

### T-005: `sim::Processes`
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1, AC-2
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-5

`sim/tests/processes.rs` holds 18 tests, all passing on both systems.

**Scheduling:**
- `processes_run_one_at_a_time`;
- `processes_return_results_in_input_order`;
- `processes_runs_are_deterministic`: 20 seeds, two runs each;
- `processes_seeds_vary_interleavings`;
- `a_point_alone_lets_another_process_run` (new, C-011): over 20 seeds, at least one runs another process between two halves of a process separated only by a point, with no syscall. This shows a point boundary is a draw site. Draws at a return and at a kill are what let any multi-process run finish.

**Orderings:**
- `orderings_hold_back_a_point`;
- `ordering_deadlock_is_reported`;
- `orderings_wait_on_a_process_end`;
- `orderings_are_released_by_a_crash`;
- `orderings_are_released_when_the_source_process_ends` (C-002):
  - the source returns, is killed, or panics, over 20 seeds each;
  - a panicking source ends the run with its payload and no deadlock;
  - in every case, at least one seed has the waiter provably waiting first.

**Kills and crashes:**
- `killed_process_state_stays_visible`;
- `kill_at_a_point`;
- `kill_then_crash_settles_by_seed`;
- `crash_in_interleaved_run_stops_every_process`;
- `a_kill_comes_before_an_ordering_wait` (new, C-003): a process killed at a point where an ordering would also hold it reports `Killed`, and the other process finishes. Were the wait first, every seed would deadlock. A mutant that moves the kill after the wait fails it.

**Order at a point:** `point_events_are_ordered` (C-003): the count, then the kill, then the ordering wait, then the actions.
- It runs 20 seeds.
- In a seed where process 0 arrives first, its action must see both arrivals counted, which only its wait allows.
- At least one seed must be of that kind.

**Panics:** `panic_in_a_process_ends_the_run` (C-001):
- the first panic reaches the caller in 10 seeds;
- no other process completes a syscall after it;
- a mutant that stops survivors from unwinding fails it.

**Setup checks:** `invalid_setups_are_refused` (CR-02): eight invalid setups and a second kill are each refused before any process starts.

**Deadlocks:** the tests that could hang run under a 60-second watchdog, `within`, so a deadlock fails the test instead of hanging `cargo ci`.

### T-006: points planted in store and workspace
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-6
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- `store_points_mark_each_stage`: each stage of `init`, `open`, `put_object` and `commit_op` reaches its points, in order. That includes the `present_synced` path and `references_resynced`.
- `workspace_points_mark_each_stage`: a snapshot, a `restore_view` and a resume after a crash each reach their points.
- `two_writer_race_fires_at_the_named_point`: the Sprint 1 race now fires from an action on `store.commit_op.renamed` or `store.put_object.renamed`, with no trace inspection. Every seed races, and the store digest constant holds.
- **Each point's expected scenario.** The plan lists one for every registered point. `cargo reach` now checks the list, as `reach::EXPECTED` (C-011), on every `cargo ci`. See T-008.

### T-007: the four-writer matrix
- **Intents:**
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-5
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-2, AC-3
- `store/tests/writers.rs` holds 12 tests; their results are in [integration-tests.md](integration-tests.md).
- `overlap_is_read_from_the_trace` (CR-01) is the unit check of the overlap reader.
- `real_writers.rs` holds the E2E test and its child entry point; its results are in [e2e-tests.md](e2e-tests.md).

### T-008: the reach table
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-6
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-9, AC-10
- **`xtask/src/reach.rs`:**
  - `reach_table_is_generated`;
  - `reach_is_idempotent` (C-010): runs `cargo reach` twice on a fixture workspace and compares the written bytes, then passes the check;
  - the failure tests:
    - `reach_fails_on_an_unreached_point` and `reach_fails_on_an_unregistered_point` (C-010): through `run()` too, each exits 1, names the item, and leaves `docs/reach.md` absent;
    - `reach_fails_on_a_missing_scenario`;
  - `reach_fails_on_an_expected_point_missed` (new, C-011): a scenario that stops reaching a point expected of it fails, even though another scenario still reaches the point;
  - `expectations_are_real` (new, C-011): every expectation names a real scenario and a registered point, and the expectations cover all 21 points once each;
  - `reach_ignores_test_points`;
  - `reach_files_parse`;
  - `reach_table_shows_the_triangle`;
  - `reach_check_fails_on_a_stale_table`.
- **`xtask/src/main.rs`:**
  - `reach_rejects_arguments`: `--check`, `-q` and `x` exit 2;
  - `reach_refuses_out_of_tree`.
- **`xtask/src/ci.rs`:**
  - `ci_steps_are_ordered_and_locked`: the six steps, in order;
  - `test_step_clears_the_recorded_reach`: `prepare()` clears the reach directory before `test`;
  - `test_step_removes_golden_create` and `proptest_variables_are_never_inherited`: `PERI_WRITER_CHILD` is scrubbed.

### T-009: skipped names reported by every snapshotting command
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-7
- `snapshotting_commands_return_reports` (C-005):
  - two raw names are planted;
  - `new_change`, `log`, `undo` and `restore` each run on a freshly opened `Workspace`, which is checked to have reported nothing first;
  - each then holds both entries, in `status`'s order.
- `failed_commands_carry_reports` (C-006): the one entry is carried by each failure:
  - `undo` with nothing to undo;
  - `restore` of an unknown op;
  - `restore` of a view with a case-folding hazard;
  - `restore` over a blocking entry (ignored, recreated with the user's content).
- `praps_commands_report_skipped_names` replaces the plan's six CLI tests:

  | Planned test | Where it is checked |
  |---|---|
  | `praps_failed_undo_reports_skipped_names` | the first `undo`, which exits 1 and prints only the report. A `restore` of an unknown op follows, which also exits 1, prints only the report, and says "no op matches" on stderr (C-006). |
  | `praps_new_reports_skipped_names` | `new` |
  | `praps_log_reports_skipped_names` | `log`: the report is last, and appears once |
  | `praps_undo_reports_skipped_names` | the second `undo` |
  | `praps_restore_reports_skipped_names` | `restore` by prefix |
  | `praps_output_unchanged_without_reports` | Sprint 3's 22 `cli.rs` tests and the 4 other `e2e.rs` tests, unchanged and passing |

  It ran on NTFS, with an unpaired surrogate, and on ext4, with byte `0xff`.
- `snapshot` clears the report first, so an early failure reports nothing stale (CR-07). Its regression test, `a_failed_snapshot_reports_nothing_stale`, came after the tested head (round-2 critique C-001; see below).

## After the tested head

The round-2 [test critique](critique.md) found that CR-07's fix had no test (C-001). Commit `6665913` adds one test and changes nothing else:
- **`a_failed_snapshot_reports_nothing_stale`** (`workspace/tests/history.rs`). On one `Workspace`:
  1. `status` reports a raw name.
  2. A second store handle commits two ops on the head.
  3. The next `status` fails with `SeveralHeads`, before its scan, and `reported()` is then empty.
- **Mutation check:** with `self.reported.clear()` removed from `snapshot`, the test fails, at `history.rs:637`.
- **Runs:** `cargo test -p workspace --test history` at `6665913` passed 15 of 15 on Windows (3.45 s) and in a fresh WSL ext4 clone (2.95 s). See section `after the tested head` in [test-matrix-log.md](test-matrix-log.md). Clippy and `cargo fmt --check` are clean.
- **Counts at `6665913`:** 71 new tests, and `history.rs` has 15. The full suite was not re-run, so the totals above stay those measured at `9fa59fb`.

