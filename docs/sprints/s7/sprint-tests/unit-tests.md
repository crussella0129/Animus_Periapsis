# Sprint 7 unit test results

At the tested head `5b65da8`, on Windows 11 (NTFS) and in WSL Ubuntu 26.04 (ext4). The suite runs through `cargo ci`; its totals and wall times are in [integration-tests.md](integration-tests.md), the matrices in [test-matrix-log.md](test-matrix-log.md), and the runs through the binary and on GitHub in [e2e-tests.md](e2e-tests.md).

## Totals

| | At `855eb67` (Sprint 6's merge) | At the tested head |
|---|---|---|
| Test functions in the source | 560 | 631 |
| Windows `test:` line | 37 units, 588 passed | 43 units, 659 passed, 0 failed |
| WSL `test:` line | 37 units, 590 passed | 43 units, 661 passed, 0 failed |

- **71 tests are new, 68 from the tasks and 3 from the fixes made during Test, and 3 renamed; none was removed.** Each rename turns a test that pinned the old stop into a test of the merge:
  - `praps_status_with_several_heads_fails` → `praps_status_with_several_heads_works` (`crates/praps/tests/cli.rs`);
  - `several_heads_stop_every_command` → `several_heads_are_merged_for_every_command` and `an_operation_written_after_open_leaves_two_heads` → `an_operation_written_after_open_is_merged` (`crates/workspace/tests/workspace.rs`).
- Linux runs two tests more than Windows, as in Sprint 6: they exist on Unix only.
- The list was made by comparing every `#[test]` function at `855eb67` and at the tested head.

## New tests, by task

| Task | Tests |
|---|---|
| T-002 | `crates/codec/tests/objects.rs`: `view_holds_one_entry_per_workspace`, `view_references_are_each_entry`, `empty_workspace_names_are_rejected`, `empty_working_copies_are_rejected`; `properties.rs`: `operation_names_its_workspace`. Golden files for a view, a view whose pointer is in conflict, a root op and a merge op. |
| T-003 | `crates/workspace/tests/merge.rs`, 12: `merge_ignores_order_and_repeats` (AC-1, 2,000 histories); `two_heads_equal_the_three_way_merge` (AC-2, 24,090 pairs of heads with no merged copy); `committed_partial_merges_change_nothing` (AC-3, 1,313 partial merges); `criss_cross_histories_match_the_reference` (AC-4, 4,000 histories) and `a_resolution_is_not_reopened`; `stale_copies_do_not_override_the_owner`; `merged_copies_and_nested_set_backs_are_pinned`; `wrong_merges_are_rejected` and `an_arbitrary_ancestor_fails_criss_cross` (AC-6); `one_head_is_its_view`, `entries_merge_by_their_rules`, `writers_and_bases_are_found`. |
| T-004 | `cli.rs`: `eight_new_at_once_all_succeed`, `praps_notes_a_raced_working_copy`, and the renamed `praps_status_with_several_heads_works`. `workspace.rs`: `a_raced_pointer_continues_from_the_smallest`, `a_writer_commits_over_every_head`, `readers_write_nothing` (2 and 8 processes), and the two renamed. `materialise.rs`: `records_name_their_starting_change`, `resume_finishes_while_its_target_is_the_writer`, `resume_finishes_after_another_workspace_commits`, `resume_leaves_files_when_its_workspace_moved_on`. |
| T-005 | `history.rs`: `undo_is_scoped_to_its_workspace`, `restore_is_scoped_to_its_workspace`, `undo_of_a_workspaces_first_operation_has_nothing_to_undo`, `restore_to_before_the_workspace_refuses`, `undo_after_a_resolved_race_keeps_one_change`, `restore_of_a_merged_copy_keeps_one_change`, `history_in_one_lane_leaves_the_other`. `scoped.rs`: `undo_is_scoped_over_200_interleavings` (INT-0009 AC-6: 2,363 commands). `cli.rs`: `praps_undo_reports_every_op_it_went_back_to`. |
| T-006 | `lanes.rs`, 8: `a_lane_is_made_over_the_same_store`, `a_lane_starts_from_the_parent_of_the_change`, `open_finds_the_store_a_lane_names`, `lane_refusals_write_nothing`, `an_interrupted_lane_is_refused_and_made_again`, `a_lane_whose_store_is_gone_is_refused`, `a_damaged_workspace_file_is_refused`, `eight_lanes_share_one_store`. `lane_crash.rs`: `a_crash_while_making_a_lane_is_finished_or_refused`, `lane_matrix_is_reproducible`, `lane_matrix_catches_a_file_written_in_place`. `lane.rs` (unit): `lane_files_round_trip`. |
| T-007 | `cli.rs`: `praps_lane_makes_a_workspace`, `praps_lane_refusals`, `praps_lane_finishes_an_interrupted_lane`, `praps_lanes_lists_every_workspace`, `commands_act_on_the_lane_they_run_in`, `init_in_a_lane_is_refused`, `a_damaged_workspace_file_stops_the_search`, `praps_ops_names_each_workspace`. |
| T-008 | `a_thousand_processes_run`, in `crates/sim/tests/processes.rs` until Test moved it to `thousand.rs`, a binary of its own (see [integration-tests.md](integration-tests.md)). |
| T-009 | `concurrency.rs`, 10: `concurrency_on_one_store_finds_no_violations`, `concurrency_finds_no_violations`, `concurrency_is_reproducible`, `concurrency_catches_first_head_only`, `concurrency_catches_merge_on_read`, `concurrency_catches_unsynced_objects`, `readers_add_nothing_at_scale`, `readers_catch_merge_on_read`, `merging_marks_its_point`, `making_a_lane_marks_its_point`. |
| T-010 | `crates/praps/tests/e2e.rs`: `lanes_at_once_end_to_end`, `a_lane_session_end_to_end`. |
| Test fixes (`1d5c94b`, `5b65da8`) | `lanes.rs`: `a_lane_whose_write_is_blocked_is_refused`; `workspace.rs`: `a_missing_parent_op_is_refused`; `cli.rs`: `init_refuses_an_unreadable_workspace_file`, a sixth case in `praps_lane_refusals` (a last name holding a line break), and `praps_lane_finishes_an_interrupted_lane` now runs its hint's own command. Each fails without its fix ([code-review.md](code-review.md)). |

5 + 12 + 9 + 9 + 12 + 8 + 1 + 10 + 2 = 68 from the tasks, and 3 from the fixes: 71. T-001 changed records only.

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
| Readers (T-009) | `MergeOnRead` | at 2 and 8 readers, the op files change |
| The five Sprint 1 to 6 matrices | their controls | as before; see [test-matrix-log.md](test-matrix-log.md) |

## Reviews the test plan asks for

- **`ci_sprint6_record_matches_github`: matches.** Every run, job, conclusion, start and end time, runner image, cache line and `test:` line in [ci-sprint6.md](ci-sprint6.md) was fetched again during Test with `gh api …/runs/<run>`, `…/jobs` and `gh run view --job <job> --log`, and equals the record: runs 42872605202, 42927097585 and 42952942473, the merge `aab90fd` at 2026-11-11T03:27:58Z, pull request #9 (head `421c6a2`, merged at 2026-11-14T13:05:54Z as `855eb67`) with its five checks, and run 43606847894 on `main`.
- **`sign_offs_are_the_owners_words`: matches, with one correction.** The two questions, the answers chosen and what each answer said, in [sign-offs.md](../sign-offs.md), equal the conversation of 2026-11-14 word for word. The two decisions that are not sign-offs were quoted without their "(Recommended)" suffix; the record now quotes them whole.
- **`backlog_carries_what_is_pending`: yes.** After T-001, T-035 names the pending sign-offs as its clause says: `lane` and `lanes` signed, INT-0016 AC-8 signed for one class, AC-11's three samples approved for this Test. After T-010, T-029 is gone and T-037 to T-043 hold the seven things it leaves. `cargo book` passes at both.
- **`t002_moves_no_cell_count`: yes.** `git show 609a8df` over `crates/*/tests`: the five matrices' cell-count literals are unchanged and only their five digest literals change. Every other test change builds an operation or a view in the new shape.
- **`t002_changes_no_behaviour_assertion`: yes.** In the same diff over `crates/praps/tests/` and `crates/workspace/tests/`, the changed assertions read a view's entries in the new shape (`workspaces["default"].heads` for `heads`); none is about what a command prints, its exit code or the files it leaves.
- **`t004_moves_only_the_materialise_digest`: yes.** `git show ec33381` changes one literal among the five matrices', the Materialise digest. T-004's evidence gives the reason: the record names the changeset it starts from.
- **`scheduler_at_a_thousand_processes`: a fiftieth.** The same throwaway release run, 1,000 processes making 3,001 syscalls, on this Windows machine: 6.54 s at `ce4686c`, 0.13 s at T-008's commit.
- **`specs_match_the_code`: yes.** The working-copy spec describes each of T-010's eight things, under these headings: the view's entries and who writes them ("The working-copy change", "Merging op heads"); the merge ("Merging op heads"); commands that change nothing write nothing ("Merging op heads"); the race rule ("Merging op heads"); scoped `undo` and `restore` ("History commands"); lanes and `.praps/workspace` ("Lanes", "Finding the working copy"); the record's two forms ("The record"); resume's rule ("Resume"). No section says several heads stop a command. `docs/prior-art.md`'s two new rows say what the research's sources say: a linked working tree shares its repository's objects and finds it through a file at its top (`git worktree`), and `ort` merges several common ancestors into a virtual merge base (Git's merge strategies).
- **`format_sign_off_is_the_owners_answer`: not yet.** The owner has not been asked yet. As in Sprint 6, the question is put at the end of the sprint, before the merge, and only the owner's own answer will be recorded in [sign-offs.md](../sign-offs.md). Until then INT-0006 AC-6 stays open for this change.
