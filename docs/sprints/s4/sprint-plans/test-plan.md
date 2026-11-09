# Sprint 4 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 determinism, with points and interleavings | T-002 / a point is not a syscall, traced or seeded; T-004 / same seed gives the same run; T-005 / digests unchanged; T-006 / the four-writer digest is reproducible | `points_are_not_syscalls`, `points_do_not_change_traces`, `processes_runs_are_deterministic`, `processes_seeds_vary_interleavings`, the store, Snapshot and Materialise digest tests, `writers_matrix_is_reproducible` |
| INT-0005 | AC-2 crash matrix with several writers and a process crash | T-004 / kill; T-006 / every cell | `killed_process_state_stays_visible`, `kill_then_crash_settles_by_seed`, `crash_in_interleaved_run_stops_every_process`, `writers_matrix_finds_no_violations` |
| INT-0005 | AC-3 negative controls under four writers | T-006 / each variant caught | `writers_matrix_catches_every_variant` |
| INT-0005 | AC-6 reach: every point reached; the table regenerated and checked | T-002 / reach counted; T-003 / the registry complete; T-007 / the table, the unreached-point failure, `--check` | `reach_counts_points`, `policy_points_rule` (fixtures), `reach_table_is_generated`, `reach_fails_on_an_unreached_point`, `reach_fails_on_a_missing_scenario`, `reach_check_fails_on_a_stale_table`, `cargo reach --check` in `cargo ci` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-5: four processes, no corrupt record, no lost op, under crash injection | T-006 | `writers_matrix_finds_no_violations`, `writers_overlap_in_some_seed`, E2E `real_four_processes_lose_nothing` (NTFS, ext4) |
| INT-0007 | AC-1 to AC-3 with several writers; the two-writer race preserved | T-005, T-006 | `two_writer_race_fires_at_the_named_point`, the store matrix digest, `writers_matrix_finds_no_violations` |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-7 reported by every command that snapshots | T-008 | `undo_restore_and_new_return_reports`, `praps_undo_reports_skipped_names`, `praps_restore_reports_skipped_names`, `praps_new_reports_skipped_names`, `praps_output_unchanged_without_reports` |
| INT-0004 | AC-1 suite on both systems | every task | the integration run on Windows and in WSL; GitHub halves recorded by the next sprint |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1, 0 lost edits | T-005, T-008 | the Snapshot and Materialise matrices (digests unchanged), the edit corpora |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-5 credits; policy rules | T-003, T-009 | `cargo policy`; review of `docs/prior-art.md` |
| INT-0003, [INT-0006](../../../intents/INT-0006-format-v1.md), INT-0004, [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | Sprint 3's GitHub halves; sign-offs | T-001 | review of `ci-sprint3.md` against `gh run view` |

## Unit Tests
### T-002 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `points_are_not_syscalls`: points before, between and after syscalls leave `syscalls()`, the trace and every crash outcome as without them, for every syscall kind.
- `points_do_not_change_traces`: the contract scenario with and without points, over 40 seeds × 2 modes: equal traces and states.
- `reach_counts_points`: `reach()` returns each name with its call count, sorted by name.
- `on_point_actions_run_in_order`: two actions on one name run in the order attached, and none on other names.
- `points_count_but_do_not_act_after_a_crash`.
- `real_fs_point_is_a_no_op`: `RealFs::point` changes nothing on disk.
- `simfs_is_send_and_sync`: a compile-time assertion, and `Arc<SimFs>` used from two threads.

### T-003 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `policy_points_rule`: fixtures for an unregistered point, an unused registration, a duplicate, a malformed name, a non-literal argument (each fails and names the point and file), and the clean fixture (passes).

### T-004 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `processes_run_one_at_a_time`: a shared counter shows no two processes inside a step together.
- `processes_runs_are_deterministic`: three processes writing files, run twice per seed over 20 seeds: equal traces, states and results.
- `processes_seeds_vary_interleavings`: at least two distinct traces over 20 seeds.
- `orderings_hold_back_a_point`: with `(A, B)`, B is never passed before A in 20 seeds.
- `ordering_deadlock_is_reported`: `(A, B)` and `(B, A)` across two processes end with the deadlock error naming both points.
- `killed_process_state_stays_visible`: process 0 writes without syncing and is killed. Process 1 reads its bytes. It made no further syscall and reports `Killed`.
- `kill_then_crash_settles_by_seed`: after a kill and a crash, `recover` keeps, drops or tears the killed process's pending data by the seed, the same in two runs.
- `crash_in_interleaved_run_stops_every_process`: every process's syscalls after the crash point fail, and the run returns.
- `panic_in_a_process_ends_the_run`: the panic reaches the caller after every thread has joined, with no deadlock and a timeout guard.

### T-005 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `store_points_mark_each_stage`: `put_object` and `commit_op` on `SimFs` reach their stage points in order, each once per call.
- `workspace_points_mark_each_stage`: snapshot, record write and remove, materialise and resume reach their points.
- `two_writer_race_fires_at_the_named_point`: every seed races, as before, with no trace inspection.
- The existing digest tests: the store, Snapshot and Materialise matrices with unchanged digests.

### T-007 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `reach_table_is_generated`: fixture registries and scenario files give the expected Markdown, with rows by sprint and columns grouped by sprint.
- `reach_fails_on_an_unreached_point`, `reach_fails_on_a_missing_scenario`, `reach_check_fails_on_a_stale_table`, `reach_rejects_extra_arguments`.

### T-008 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `undo_restore_and_new_return_reports`: on `SimFs` with a raw name, each command returns the reported entry, sorted as `status` returns it.

## Integration Tests
### The four-writer matrix
- **Intents:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `writers_matrix_finds_no_violations`: 40 seeds × 2 modes × every global crash step × {no kill, one kill}, 0 violations, with the cell count printed.
- `writers_overlap_in_some_seed`: at least one uncrashed seed overlaps two commits.
- `writers_matrix_catches_every_variant`: each of the five variants produces at least one violation, with the counts printed.
- `writers_matrix_is_reproducible`: equal digests in two runs, and a printed digest compared across Windows and Linux.

### The existing matrices and corpora
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- The store, two-writer, Snapshot and Materialise matrices and the edit, ignore, hazard and reversibility corpora: unchanged results, and digests equal to Sprint 3's.

### Reach and CI
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `cargo ci` with six steps (fmt, clippy, test, reach, policy, book) on Windows, and in a fresh WSL ext4 clone.
- `cargo deny check` on both systems, with cargo-deny installed in WSL.
- `docs/reach.md` the same when generated on either system.

## End-to-End Tests
- **Status:** possible
- `real_four_processes_lose_nothing`:
  - four child processes, each committing 25 ops to one store;
  - on NTFS (Windows) and ext4 (WSL);
  - pass: every acknowledged op present and valid, `fsck` clean, and heads derived from all ops.
- `praps_undo_reports_skipped_names`, `praps_restore_reports_skipped_names`, `praps_new_reports_skipped_names`: on ext4 with a non-UTF-8 name, and on NTFS with an unpaired surrogate. The command prints the name in `status`'s format and keeps exit 0.
- `praps_output_unchanged_without_reports`: Sprint 3's CLI outputs unchanged.
- **Not yet possible:**
  - power loss (no hardware rig);
  - Nighthawk (Tailscale approval pending, INT-0004 AC-6);
  - several workspaces (INT-0014 AC-3, after INT-0010's merge).
