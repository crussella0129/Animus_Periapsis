# Sprint 4 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 determinism, with points and interleavings | T-002 / Sprint 3 digests asserted; T-003 / a point is not a syscall, traced or seeded; T-005 / same seed gives the same run; T-007 and T-008 / the four-writer digest reproducible and asserted | `matrix_runs_are_reproducible`, `snapshot_matrix_is_reproducible`, `materialise_matrix_is_reproducible` (each asserting its constant), `points_are_not_syscalls`, `points_do_not_change_traces`, `processes_runs_are_deterministic`, `processes_seeds_vary_interleavings`, `writers_matrix_is_reproducible` |
| INT-0005 | AC-2 crash matrix with several writers and a process crash | T-005 / kills; T-007 / every cell, forced race, bounded kills | `killed_process_state_stays_visible`, `kill_at_a_point`, `kill_then_crash_settles_by_seed`, `crash_in_interleaved_run_stops_every_process`, `writers_matrix_finds_no_violations`, `writers_forced_race_runs`, `writers_overlap_in_some_even_seed`, `writers_kills_happen` |
| INT-0005 | AC-3 negative controls under four writers, semantic violations in strict mode | T-007 | `writers_catch_pointer_file`, `writers_catch_unsynced_objects` (durability and torn), `writers_catch_unsynced_init`, `writers_catch_write_in_place`, `writers_catch_unsynced_references` (closure in forced-race cells) |
| INT-0005 | AC-6 reach: every point reached; the table regenerated and checked | T-003 / reach counted; T-004 / the registry complete; T-006 / the points planted; T-008 / the table, failures, the `reach` step | `reach_counts_points`, `policy_points_rule`, `store_points_mark_each_stage`, `workspace_points_mark_each_stage`, `reach_table_is_generated`, `reach_is_idempotent`, `reach_fails_on_an_unreached_point`, `reach_fails_on_a_missing_scenario`, `reach_fails_on_an_unregistered_point`, `reach_ignores_test_points`, `reach_record_round_trips`, `reach_rejects_arguments`, `reach_table_shows_the_triangle`, `ci_steps_are_ordered_and_locked` (extended), the `reach` step of `cargo ci` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-5: four processes, no corrupt record, no lost op, under crash injection | T-007 | `writers_matrix_finds_no_violations`, E2E `real_four_processes_lose_nothing` (NTFS, ext4) |
| INT-0007 | AC-1 to AC-3 with several writers; the two-writer race preserved | T-006, T-007 | `two_writer_race_fires_at_the_named_point`, the store digest constant, `writers_matrix_finds_no_violations` (closure, acknowledged ⊆ present ⊆ acknowledged ∪ in-flight, objects, heads from the log, `fsck`) |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-7 reported by every command that snapshots, on success and failure | T-009 | `snapshotting_commands_return_reports`, `failed_undo_carries_reports`, `praps_log_reports_skipped_names`, `praps_undo_reports_skipped_names`, `praps_restore_reports_skipped_names`, `praps_new_reports_skipped_names`, `praps_failed_undo_reports_skipped_names`, `praps_output_unchanged_without_reports` |
| INT-0004 | AC-1 suite on both systems | every task | the integration run on Windows and in WSL; GitHub halves recorded by the next sprint |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1, 0 lost edits | T-006, T-009 | the Snapshot and Materialise matrices (digest constants), the edit corpora |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-9, AC-10 four flagless commands; AC-5 credits; policy rules | T-004, T-008, T-010 | `reach_rejects_arguments`, `reach_refuses_out_of_tree`, `cargo policy`, `readme_problems` with the four-command sentence, review of README and `docs/prior-art.md` |
| INT-0003, [INT-0006](../../../intents/INT-0006-format-v1.md), INT-0004, [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | Sprint 3's GitHub halves; sign-offs | T-001 | review of `ci-sprint3.md` against `gh run view` |

## Unit Tests
### T-002 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `simfs_is_send_and_sync`: a compile-time assertion, and `Arc<SimFs>` used from two threads.
- `poisoned_state_is_recovered`: a thread panicking inside a hook while holding no lock, and one panicking with the state locked (by a test-only path), leave later calls working.
- The three reproducibility tests, now asserting their Sprint 3 digest constants.

### T-003 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `points_are_not_syscalls`: points before, between and after syscalls leave `syscalls()`, the trace and every crash outcome as without them, for every syscall kind.
- `points_do_not_change_traces`: the contract scenario with and without points over 40 seeds × 2 modes gives equal traces and states.
- `reach_counts_points`: `reach()` returns each name with its call count, sorted.
- `on_point_actions_run_in_order`: two actions on one name run in the order attached, and none run on other names.
- `on_point_action_is_not_reentered`: an action that reaches its own point on the same thread is not entered again, and the point is still counted.
- `points_count_but_do_not_act_after_a_crash`.
- `recover_drops_actions_and_reach`.
- `points_reach_through_every_forwarder`: `&SimFs`, `Rc<SimFs>` and `Arc<SimFs>` each count a point.
- `real_fs_point_is_a_no_op`.

### T-004 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `policy_points_rule`, with fixtures that each fail and name the point and file:
  - an unregistered point;
  - an unused registration;
  - a duplicate;
  - a malformed name;
  - a wrong crate prefix;
  - a non-literal argument;
  - an unrecognised registry entry;
  - a sprint out of range.

  Fixtures that must pass: a commented-out and a rustdoc `.point("…")` (neither counts), a literal on the line after `.point(`, and the clean fixture.

### T-005 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `processes_run_one_at_a_time`: a shared counter shows no two processes inside a step together, including before their first boundary.
- `processes_return_results_in_input_order`: values, `Killed` and a panic, in input order.
- `processes_runs_are_deterministic`: three processes writing files, two runs per seed over 20 seeds: equal traces, states and results.
- `processes_seeds_vary_interleavings`: at least two distinct traces over 20 seeds.
- `orderings_hold_back_a_point`: with an ordering, B is never passed before A in 20 seeds.
- `ordering_deadlock_is_reported`: crossed orderings between live processes end with the deadlock error naming both points.
- `orderings_wait_on_a_process_end`: a `(p, End)` predecessor holds *q* until *p* returns.
- `orderings_are_released_by_a_crash`: a process held by an ordering when the system crashes is released, and its next syscall fails with the crash.
- `orderings_are_released_when_the_source_process_ends`: a process waiting on a point that a returned, killed or panicked process never reached is released.
- `killed_process_state_stays_visible`: process 0 writes without syncing and is killed at a step. Process 1 reads its bytes. Process 0 made no further syscall and reports `Killed`.
- `kill_at_a_point`: a kill on the *n*-th arrival at a named point stops the process there.
- `kill_then_crash_settles_by_seed`: after a kill and a crash, `recover` keeps, drops or tears the pending data by the seed, the same in two runs.
- `crash_in_interleaved_run_stops_every_process`.
- `point_events_are_ordered`: at one point, the count, then a kill, then the ordering wait, then the actions, in that order.
- `reach_record_round_trips`: `sim::reach::record` writes through a temporary file and a rename, and the file reads back to the same counts.
- `panic_in_a_process_ends_the_run`: the first panic reaches the caller after every thread has joined, under a timeout guard.

### T-006 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `store_points_mark_each_stage`:
  - a fresh `put_object` and `commit_op` reach `temp_written`, `temp_synced`, `renamed` and `dir_synced` once each, in order;
  - an object already present reaches only `present_synced`;
  - `init` reaches `layout_created` then `sync_layout.synced`;
  - `open` reaches `sync_layout.synced`;
  - `commit_op` with parents reaches `references_resynced`.
- `workspace_points_mark_each_stage`: a snapshot, an `undo` (record written, paths written and removed, record dropped) and a resume after a crash each reach their points.
- `two_writer_race_fires_at_the_named_point`: every seed races, as before, with no trace inspection, and the store digest constant holds.

### T-008 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `reach_table_is_generated`: fixture registries and scenario files give the expected Markdown, with rows by sprint and columns grouped by sprint.
- `reach_is_idempotent`: two runs write identical bytes.
- `reach_fails_on_an_unreached_point`, `reach_fails_on_a_missing_scenario`, `reach_fails_on_an_unregistered_point`: each fails, names the item, and writes nothing.
- `reach_rejects_arguments`: `--check`, `-q` and `x` each exit 2.
- `reach_ignores_test_points`: a `test.`-prefixed name in a scenario file is ignored, while any other unregistered name fails.
- `reach_refuses_out_of_tree`.
- `ci_steps_are_ordered_and_locked` (extended): six steps in order, the reach directory cleared before `test`, and `PERI_WRITER_CHILD` scrubbed.
- `reach_table_shows_the_triangle`: reads the committed `docs/reach.md`. The four-writer row has non-zero Sprint 1 `store::commit_op` and `store::put_object` cells. The Materialise row has non-zero Sprint 1 store cells and a non-zero Sprint 2 `workspace::snapshot` cell.

### T-009 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `snapshotting_commands_return_reports`: on `SimFs` with a raw name, `log`, `undo`, `restore` and `new_change` each return the entry once, sorted as `status` returns it.
- `failed_undo_carries_reports`: `undo` on the root op returns `NothingToUndo` carrying the entry.

## Integration Tests
### The four-writer matrix
- **Intents:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `writers_matrix_finds_no_violations`: 40 seeds × 2 modes × every global crash step × {no kill, one kill}, with 0 violations. It prints the cell count, the killed-cell count and the wall time, and writes the reach file.
- `writers_forced_race_runs`: in every odd seed's uncrashed run, process 2's second op names process 0's op while process 0 is held, and, when process 2 is released from its listing, that op's name is visible and not durable.
- `writers_retry_reaches_present`: process 1's second commit of the same op is acknowledged and reaches `store.commit_op.present_synced`. In a cell that crashes during the retry, the op stays in the acknowledged set and is present.
- `writers_overlap_in_some_even_seed`: at least one even seed overlaps two commits without orderings.
- `writers_kills_happen`: every uncrashed kill run reports `Killed` for the chosen process.
- `writers_catch_pointer_file`, `writers_catch_unsynced_objects`, `writers_catch_unsynced_init`, `writers_catch_write_in_place`, `writers_catch_unsynced_references`: strict mode, with the semantic kinds above and the counts printed. `writers_catch_unsynced_references` also requires a non-zero count of odd cells in which process 0's op is not durable at process 2's rename.
- `writers_matrix_is_reproducible`: two runs (one shared with `writers_matrix_finds_no_violations`) give equal cells in canonical order, and the digest equals its constant on both systems.
- Budget: the `writers` binary within 10 minutes wall-clock on the Windows host, with the measured time recorded.

### The existing matrices and corpora
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- The store, two-writer, Snapshot and Materialise matrices, with their digest constants asserted. The edit, ignore, hazard and reversibility corpora, with unchanged results.

### Reach and CI
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `cargo ci` with six steps (fmt, clippy, test, reach, policy, book) on Windows, and in a fresh WSL ext4 clone. The `reach` step passes on both, so the committed `docs/reach.md` is the same when generated on either system.
- `cargo deny check` on both systems, with cargo-deny installed in WSL.

## End-to-End Tests
- **Status:** possible
- `real_four_processes_lose_nothing`: four child processes each commit 25 ops of distinct content to one store, on NTFS (Windows) and ext4 (WSL). It passes when:
  - every op a child reports is present and valid;
  - `fsck` is clean;
  - `store.heads()` equals the heads computed from the children's reports.
- The `praps` skipped-name tests:
  - `praps_log_reports_skipped_names`, `praps_undo_reports_skipped_names`, `praps_restore_reports_skipped_names`, `praps_new_reports_skipped_names` and `praps_failed_undo_reports_skipped_names`;
  - run on ext4 with a non-UTF-8 name and on NTFS with an unpaired surrogate;
  - each prints the name in `status`'s format and keeps its exit code.
- `praps_output_unchanged_without_reports`: Sprint 3's CLI outputs unchanged.
- **Not yet possible:**
  - power loss: out of scope by INT-0007's Consequences; the simulator's crash model stands in;
  - Nighthawk: Tailscale approval pending, INT-0004 AC-6;
  - several workspaces: INT-0014 AC-3, after INT-0010's merge.
