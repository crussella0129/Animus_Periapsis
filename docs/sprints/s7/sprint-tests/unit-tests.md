# Sprint 7 unit test results

At the tested head `4068f43`, on Windows 11 (NTFS) and in WSL Ubuntu 26.04 (ext4). The suite runs through `cargo ci`; its totals and wall times are in [integration-tests.md](integration-tests.md), the matrices in [test-matrix-log.md](test-matrix-log.md), and the runs through the binary and on GitHub in [e2e-tests.md](e2e-tests.md).

## Totals

| | At `855eb67` (Sprint 6's merge) | At the tested head |
|---|---|---|
| Test functions in the source | 560 | 636 |
| Windows `test:` line | 37 units, 588 passed | 43 units, 663 passed, 0 failed |
| WSL `test:` line | 37 units, 590 passed | 43 units, 666 passed, 0 failed |

- **76 tests are new: 68 from the tasks, 3 from the code review's fixes and 5 from the test critique's first round. 3 were renamed, and none was removed.** Each rename turns a test that pinned the old stop into a test of the merge:
  - `praps_status_with_several_heads_fails` → `praps_status_with_several_heads_works` (`crates/praps/tests/cli.rs`);
  - `several_heads_stop_every_command` → `several_heads_are_merged_for_every_command` and `an_operation_written_after_open_leaves_two_heads` → `an_operation_written_after_open_is_merged` (`crates/workspace/tests/workspace.rs`).
- Linux runs three tests more than Windows. Two are Sprint 6's tests that exist on Unix only. The third is `praps_lane_refuses_a_last_name_that_is_not_utf8`, new here, which needs a file name that is not UTF-8.
- The list was made by comparing every `#[test]` function at `855eb67` and at the tested head. The test critique's second round added checks to existing tests only, so the totals are those at `b951d6d`.

## New tests, by task

| Task | Tests |
|---|---|
| T-002 | `crates/codec/tests/objects.rs`: `view_holds_one_entry_per_workspace`, `view_references_are_each_entry`, `empty_workspace_names_are_rejected`, `empty_working_copies_are_rejected`; `properties.rs`: `operation_names_its_workspace`. Golden files for a view, a view whose pointer is in conflict, a root op and a merge op. |
| T-003 | `crates/workspace/tests/merge.rs`, 12: `merge_ignores_order_and_repeats` (AC-1, which holds by construction; see below); `two_heads_equal_the_three_way_merge` (AC-2, 24,090 pairs of heads with no merged copy); `committed_partial_merges_change_nothing` (AC-3, 1,313 partial merges); `criss_cross_histories_match_the_reference` (AC-4, 4,000 histories) and `a_resolution_is_not_reopened`; `stale_copies_do_not_override_the_owner`; `merged_copies_and_nested_set_backs_are_pinned`; `wrong_merges_are_rejected` and `an_arbitrary_ancestor_fails_criss_cross` (AC-6); `one_head_is_its_view`, `entries_merge_by_their_rules`, `writers_and_bases_are_found`. |
| T-004 | `cli.rs`: `eight_new_at_once_all_succeed`, `praps_notes_a_raced_working_copy`, and the renamed `praps_status_with_several_heads_works`. `workspace.rs`: `a_raced_pointer_continues_from_the_smallest`, `a_writer_commits_over_every_head`, `readers_write_nothing` (2 and 8 processes), and the two renamed. `materialise.rs`: `records_name_their_starting_change`, `resume_finishes_while_its_target_is_the_writer`, `resume_finishes_after_another_workspace_commits`, `resume_leaves_files_when_its_workspace_moved_on`. |
| T-005 | `history.rs`: `undo_is_scoped_to_its_workspace`, `restore_is_scoped_to_its_workspace`, `undo_of_a_workspaces_first_operation_has_nothing_to_undo`, `restore_to_before_the_workspace_refuses`, `undo_after_a_resolved_race_keeps_one_change`, `restore_of_a_merged_copy_keeps_one_change`, `history_in_one_lane_leaves_the_other`. `scoped.rs`: `undo_is_scoped_over_200_interleavings` (INT-0009 AC-6: 2,373 commands checked, 2,002 of which succeeded, 1,023 of those beside the other lane's op with two heads, the same on both systems). `cli.rs`: `praps_undo_reports_every_op_it_went_back_to`. |
| T-006 | `lanes.rs`, 8: `a_lane_is_made_over_the_same_store`, `a_lane_starts_from_the_parent_of_the_change`, `open_finds_the_store_a_lane_names`, `lane_refusals_write_nothing`, `an_interrupted_lane_is_refused_and_made_again`, `a_lane_whose_store_is_gone_is_refused`, `a_damaged_workspace_file_is_refused`, `eight_lanes_share_one_store`. `lane_crash.rs`: `a_crash_while_making_a_lane_is_finished_or_refused`, `lane_matrix_is_reproducible`, `lane_matrix_catches_a_file_written_in_place`. `lane.rs` (unit): `lane_files_round_trip`. |
| T-007 | `cli.rs`: `praps_lane_makes_a_workspace`, `praps_lane_refusals`, `praps_lane_finishes_an_interrupted_lane`, `praps_lanes_lists_every_workspace`, `commands_act_on_the_lane_they_run_in`, `init_in_a_lane_is_refused`, `a_damaged_workspace_file_stops_the_search`, `praps_ops_names_each_workspace`. |
| T-008 | `a_thousand_processes_run`, in `crates/sim/tests/processes.rs` until Test moved it to `thousand.rs`, a binary of its own (see [integration-tests.md](integration-tests.md)). |
| T-009 | `concurrency.rs`, 10: `concurrency_on_one_store_finds_no_violations`, `concurrency_finds_no_violations`, `concurrency_is_reproducible`, `concurrency_catches_first_head_only`, `concurrency_catches_merge_on_read`, `concurrency_catches_unsynced_objects`, `readers_add_nothing_at_scale`, `readers_catch_merge_on_read`, `merging_marks_its_point`, `making_a_lane_marks_its_point`. |
| T-010 | `crates/praps/tests/e2e.rs`: `lanes_at_once_end_to_end`, `a_lane_session_end_to_end`. |
| Test fixes (`1d5c94b`, `5b65da8`) | `lanes.rs`: `a_lane_whose_write_is_blocked_is_refused`; `workspace.rs`: `a_missing_parent_op_is_refused`; `cli.rs`: `init_refuses_an_unreadable_workspace_file`, a sixth case in `praps_lane_refusals` (a last name holding a line break), and `praps_lane_finishes_an_interrupted_lane` now runs its hint's own command. Each fails without its fix ([code-review.md](code-review.md)). |
| Test critique, round 1 (`b951d6d`) | `lanes.rs`: `an_unreadable_workspace_file_is_refused`; `materialise.rs`: `resume_leaves_files_when_its_workspace_raced`; `concurrency.rs`: `concurrency_catches_lost_ops`; `cli.rs`: `commands_act_the_same_from_below_in_a_lane`, and on Unix only `praps_lane_refuses_a_last_name_that_is_not_utf8`. Tightened: in `cli.rs`, `every_refusal_prints_one_hint`, `eight_new_at_once_all_succeed`, `idempotent_commands_repeat_exactly`, `praps_lanes_lists_every_workspace`, `praps_lane_refusals` and `praps_ops_names_each_workspace`; `lanes_at_once_end_to_end`; the Concurrency scenario's checks and both matrices' reproducibility tests, which also assert a portable digest; the lane matrix's check of each cell; `eight_lanes_share_one_store`; `undo_is_scoped_over_200_interleavings`; `a_thousand_processes_run`. See [critique-round1.md](critique-round1.md). `.` and `..` are refused as directories that are not free: DIR is made a full path first, so the check of the last name never sees them. |
| Test critique, round 2 (`4068f43`) | No new test functions. Tightened: `every_refusal_prints_one_hint` makes three more kinds (an unreadable `.praps/workspace` from below the lane's top, `verify` in a lane whose file does not parse, `verify` in a lane whose store is gone), 44 on Linux and 41 on Windows; the hint table of `hints_take_one_of_three_forms` has four more rows, 43 kinds; `commands_act_the_same_from_below_in_a_lane` compares `new` and checks that its op names the lane; the Concurrency scenario checks every store again after each lane's next command; `concurrency_catches_lost_ops` runs its two-store run first. See [critique-round2.md](critique-round2.md). |

5 + 12 + 9 + 9 + 12 + 8 + 1 + 10 + 2 = 68 from the tasks, 3 from the code review's fixes and 5 from the test critique's: 76. T-001 changed records only.

**INT-0010 AC-1 holds by construction.** The library's merge takes a set of heads, `writers` walks only from them, and every collection the merge builds is ordered. `merge_ignores_order_and_repeats` hands the library each shuffled and repeated order as the one set a caller collects, so for the library it is a check that the merge is deterministic. The evidence that the property check itself can fail is its two mutants that take the heads as a list: the pairwise fold and the first-equals-last fast path fail it in 340 and 826 of 2,000 histories.

## Negative controls

Every control is caught at the tested head, on both systems.

| Where | Control | Caught |
|---|---|---|
| The merge (T-003) | pairwise fold; the first-equals-last fast path | AC-1 fails in 340 and 826 of 2,000 histories; AC-2 holds |
| | always-base; union of heads | AC-2 fails in 1,231 and 1,312 of 24,090 pairs; AC-1 holds |
| | first-side-wins (Jujutsu's working-copy rule) | both |
| | one arbitrary common ancestor | 1,971 of 4,000 criss-cross histories differ from the reference |
| The lane matrix (T-006) | `LaneFileInPlace` | 1,394 of its 5,440 cells: the next command at the lane does not do what the cell allows (and making the lane again, in 796) |
| Concurrency (T-009) | `FirstHeadOnly`, `MergeOnRead`, the store's `UnsyncedObjects` | each in the first cell, by the oracle it breaks (writer, readers, store) |
| Concurrency (test critique, C-003; round 2, C-002) | the store's `UnsyncedObjects` again, stopped only where an acknowledged op is missing from a store; its two-store run first | in the first two-store cell: an exchange stopped at an object the crash lost and left acknowledged ops on one store, 58 counted over the cell's two checks. A crash that itself loses an op file is not shown, since no store variant leaves op files unsynced (backlog T-034) |
| The scheduler (T-008; test critique, C-011) | every process woken at each boundary | 1,819,674 needless wakes over 3,002 syscalls, where the bound is 30 |
| Readers (T-009) | `MergeOnRead` | at 2 and 8 readers, the op files change |
| The five Sprint 1 to 6 matrices | their controls | as before; see [test-matrix-log.md](test-matrix-log.md) |

- **Every control over all its cells.** `cargo ci` stops each Concurrency control at its first caught cell. With `PRAPS_FULL_CONTROLS=1` every cell runs; that sweep ran at the tested head on both systems, and its counts are in [test-matrix-log.md](test-matrix-log.md).
- **Fixes taken out once.** Five fixes from the test critique were each taken out, their test run once, and the source put back. Every test failed:
  - resume's check that the target is its workspace's only writer, weakened to "one of the writers" (C-005): `resume_leaves_files_when_its_workspace_raced`;
  - `UnfinishedLane` replaced by `NotInitialised` in `open` (C-008): the lane matrix;
  - an unreadable lane file reported as a bare I/O error by `open` (C-001): `an_unreadable_workspace_file_is_refused`;
  - the same file reported by the search as the store (C-001): `every_refusal_prints_one_hint`;
  - every process woken at each boundary (C-011): `a_thousand_processes_run`.
- **Fixes from the second round, taken out once.** Each test failed, and the source was put back:
  - the store's hint put back for a lane file in `top_above` (C-001): `every_refusal_prints_one_hint`;
  - a lane file that does not parse taken for no lane file in `verify`'s `store_of` (C-001): the same;
  - a lane's missing store answered as a working copy's in `verify` (C-001): the same;
  - the Concurrency scenario's second exchange left out (C-003): `concurrency_finds_no_violations`.

## Reviews the test plan asks for

- **`ci_sprint6_record_matches_github`: matches.** Every run, job, conclusion, start and end time, runner image, cache line and `test:` line in [ci-sprint6.md](ci-sprint6.md) was fetched again during Test with `gh api …/runs/<run>`, `…/jobs` and `gh run view --job <job> --log`, and equals the record. That command returns the log of a run's latest attempt whatever job is named, and each of these runs had one attempt, so each log read was the job's own: runs 42872605202, 42927097585 and 42952942473, the merge `aab90fd` at 2026-11-11T03:27:58Z, pull request #9 (head `421c6a2`, merged at 2026-11-14T13:05:54Z as `855eb67`) with its five checks, and run 43606847894 on `main`.
- **`sign_offs_are_the_owners_words`: matches, with one correction.** The two questions, the answers chosen and what each answer said, in [sign-offs.md](../sign-offs.md), equal the conversation of 2026-11-14 word for word. The two decisions that are not sign-offs were quoted without their "(Recommended)" suffix; the record now quotes them whole.
- **`backlog_carries_what_is_pending`: yes.** After T-001, T-035 names the pending sign-offs as its clause says: `lane` and `lanes` signed, INT-0016 AC-8 signed for one class, AC-11's three samples approved for this Test. After T-010, T-029 is gone and T-037 to T-043 hold the seven things it leaves. `cargo book` passes at both.
- **`t002_moves_no_cell_count`: yes.** `git show 609a8df` over `crates/*/tests`: the five matrices' cell-count literals are unchanged and only their five digest literals change. Every other test change builds an operation or a view in the new shape.
- **`t002_changes_no_behaviour_assertion`: yes.** In the same diff over `crates/praps/tests/` and `crates/workspace/tests/`, the changed assertions read a view's entries in the new shape (`workspaces["default"].heads` for `heads`); none is about what a command prints, its exit code or the files it leaves.
- **`t004_moves_only_the_materialise_digest`: yes.** `git show ec33381` changes one literal among the five matrices', the Materialise digest. T-004's evidence gives the reason: the record names the changeset it starts from.
- **`scheduler_at_a_thousand_processes`: a fiftieth.** The same throwaway release run, 1,000 processes making 3,001 syscalls, on this Windows machine: 6.54 s at `ce4686c`, 0.13 s at T-008's commit. Since the test critique, `a_thousand_processes_run` also checks the mechanism: at most one needless wake, a process woken while another was chosen, per 100 syscalls. It saw 0 on both systems over 3,002 syscalls at the tested head.
- **`specs_match_the_code`: yes.** The working-copy spec describes each of T-010's eight things, under these headings: the view's entries and who writes them ("The working-copy change", "Merging op heads"); the merge ("Merging op heads"); commands that change nothing write nothing ("Merging op heads"); the race rule ("Merging op heads"); scoped `undo` and `restore` ("History commands"); lanes and `.praps/workspace` ("Lanes", "Finding the working copy"); the record's two forms ("The record"); resume's rule ("Resume"). No section says several heads stop a command. `docs/prior-art.md`'s two new rows say what the research's sources say: a linked working tree shares its repository's objects and finds it through a file at its top (`git worktree`), and `ort` merges several common ancestors into a virtual merge base (Git's merge strategies).
- **`format_sign_off_is_the_owners_answer`: yes.** The question was planned for the end of the sprint, before the merge, as in Sprint 6, and only the owner's own answer is recorded. The owner merged #10 first, at `b951d6d`, and the question was then put. The owner's answer, "Sign off (Recommended)", is recorded word for word in [sign-offs.md](../sign-offs.md), so T-002's last clause and INT-0006 AC-6 hold for this change. T-002's entry in `completed-tasks.md` records its commit and is left as it is.
