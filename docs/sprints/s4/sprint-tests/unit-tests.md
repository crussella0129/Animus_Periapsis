# Sprint 4 Unit Test Results

- **Tested head:** `5dfb418e94403e4a38fff3ceda6353b309ac031b` on `dev`, committed, with a clean working tree. It includes the fixes for the Test-phase [code review](code-review.md) (CR-01 to CR-07).
- **Runner:** the `test` step of `cargo ci` (`cargo test --workspace --locked`). It ran in the Windows working checkout and in a fresh WSL ext4 clone. See [test-matrix-log.md](test-matrix-log.md), sections `int_ci_windows` and `e2e fresh_clone_ci`.
- **Result:**
  - 0 failed and 0 ignored on both systems;
  - 473 passed on Windows and 474 on Linux; Sprint 3 ended with 407 and 408;
  - the extra Linux test is still `real_symlinked_directories_are_not_followed`, which is compiled for Unix only.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13):
  - `x86_64-pc-windows-msvc`, on Windows 11 Pro 10.0.26300;
  - `x86_64-unknown-linux-gnu`, on Ubuntu 26.04.1 LTS (WSL2 kernel 6.6.114.1).
- **Randomness:**
  - Every new test that varies a run takes an explicit seed through `sim::Rng`. That includes `sim::Processes`, whose seed draws the next runner at every boundary.
  - Points take no seed and draw nothing.

## New tests by file

The 66 new tests, counted by `#[test]` against Sprint 3's tested head `ad22319`. `cargo test --workspace --locked -- --list` at `5dfb418` agrees on Windows: 473.

| File | Sprint 3 | Sprint 4 |
|------|---------:|---------:|
| `sim/tests/points.rs` (new) | 0 | 10 |
| `sim/tests/processes.rs` (new) | 0 | 16 |
| `sim/tests/threads.rs` (new) | 0 | 3 |
| `store/tests/writers.rs` (new) | 0 | 12 |
| `store/tests/real_writers.rs` (new; one test is the child entry point) | 0 | 2 |
| `store/tests/crash.rs` | 7 | 8 |
| `store/tests/store.rs` | 23 | 24 |
| `workspace/tests/materialise.rs` | 26 | 27 |
| `workspace/tests/history.rs` | 12 | 14 |
| `praps/tests/e2e.rs` | 4 | 5 |
| `xtask`: `reach.rs` (new), `points.rs` (new), `main.rs`, `ci.rs`, `policy.rs` | 70 in all | 87 in all (+9, +4, +2, +1, +1) |

## By build task

Every test the [test plan](../sprint-plans/test-plan.md) names exists and passed on both systems, with one exception. The six `praps` reporting tests the plan named for T-009 are one test, `praps_commands_report_skipped_names`. The T-009 completion entry records the change, and the mapping is under T-009 below.

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
- `points_do_not_change_traces`: the contract scenario gives equal traces and states with and without points, over 40 seeds × 2 modes.
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
- `policy_points_rule`: eleven fixtures fail, each naming the point or call and the file:
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

  The commented-out, rustdoc, next-line-literal and clean fixtures pass.
- The four `xtask/src/points.rs` parser tests.
- The Windows leg's probe ran the rule against real code. A `.point("store.probe.unregistered")` planted in `crates/store/src/lib.rs` made `cargo policy` exit 1, naming the point and the registry. The file was then restored, with 0 changed paths (section `probe: a point not registered fails cargo policy`).

### T-005: `sim::Processes`
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1, AC-2
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-5

`sim/tests/processes.rs` holds 16 tests, all passing on both systems:
- **Scheduling:**
  - `processes_run_one_at_a_time`;
  - `processes_return_results_in_input_order`;
  - `processes_runs_are_deterministic` (20 seeds, two runs each);
  - `processes_seeds_vary_interleavings`.
- **Orderings:**
  - `orderings_hold_back_a_point`;
  - `ordering_deadlock_is_reported`;
  - `orderings_wait_on_a_process_end`;
  - `orderings_are_released_by_a_crash`;
  - `orderings_are_released_when_the_source_process_ends`.
- **Kills and crashes:**
  - `killed_process_state_stays_visible`;
  - `kill_at_a_point`;
  - `kill_then_crash_settles_by_seed`;
  - `crash_in_interleaved_run_stops_every_process`.
- **Order at a point:** `point_events_are_ordered`: the count, then the kill, then the ordering wait, then the actions.
- **Panics:** `panic_in_a_process_ends_the_run`.
- **Setup checks:** `invalid_setups_are_refused` (CR-02): eight invalid setups and a second kill are each refused before any process starts.

### T-006: points planted in store and workspace
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-6
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- `store_points_mark_each_stage`: each stage of `init`, `open`, `put_object` and `commit_op` reaches its points, in order. That includes the `present_synced` path and `references_resynced`.
- `workspace_points_mark_each_stage`: a snapshot, an `undo` and a resume after a crash each reach their points.
- `two_writer_race_fires_at_the_named_point`: the Sprint 1 race now fires from an action on `store.commit_op.renamed` or `store.put_object.renamed`, with no trace inspection. Every seed races, and the store digest constant holds.

### T-007: the four-writer matrix
- **Intents:**
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-5
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-2, AC-3
- `store/tests/writers.rs` holds 12 tests. Their results are in [integration-tests.md](integration-tests.md).
- `overlap_is_read_from_the_trace` (CR-01) is the unit check of the overlap reader.
- `real_writers.rs` holds the E2E test and its child entry point. Its results are in [e2e-tests.md](e2e-tests.md).

### T-008: the reach table
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-6
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-9, AC-10
- `xtask/src/reach.rs`:
  - `reach_table_is_generated`;
  - `reach_is_idempotent`;
  - the failure tests, each of which names the item and writes nothing:
    - `reach_fails_on_an_unreached_point`;
    - `reach_fails_on_a_missing_scenario`;
    - `reach_fails_on_an_unregistered_point`;
  - `reach_ignores_test_points`;
  - `reach_files_parse`;
  - `reach_table_shows_the_triangle`;
  - `reach_check_fails_on_a_stale_table`.
- `xtask/src/main.rs`: `reach_rejects_arguments` (`--check`, `-q` and `x` exit 2) and `reach_refuses_out_of_tree`.
- `ci_steps_are_ordered_and_locked`, extended:
  - the six steps in order;
  - the reach directory cleared by `prepare()` before `test`;
  - `PERI_WRITER_CHILD` scrubbed.

### T-009: skipped names reported by every snapshotting command
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-7
- `snapshotting_commands_return_reports`: `status`, `new_change`, `log`, `undo` and `restore` each hold the one raw entry.
- `failed_undo_carries_reports`.
- `praps_commands_report_skipped_names` replaces the plan's six CLI tests:

  | Planned test | Where it is checked |
  |---|---|
  | `praps_failed_undo_reports_skipped_names` | the first `undo`, which exits 1 and prints only the report |
  | `praps_new_reports_skipped_names` | `new` |
  | `praps_log_reports_skipped_names` | `log`: the report is last, and appears once |
  | `praps_undo_reports_skipped_names` | the second `undo` |
  | `praps_restore_reports_skipped_names` | `restore` by prefix |
  | `praps_output_unchanged_without_reports` | Sprint 3's 22 `cli.rs` tests and 4 earlier `e2e.rs` tests, unchanged and passing |

  It ran on NTFS, with an unpaired surrogate, and on ext4, with byte `0xff`.
- `snapshot` now clears the report first, so an early failure reports nothing stale (CR-07).
