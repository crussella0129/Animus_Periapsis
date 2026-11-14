# Sprint 7 Test Plan

How to read the names:
- A test marked **new** is written by the task named.
- A test marked **renamed** keeps its assertions under a new name, except those this plan lists as changed.
- A name starting **review** is a check made against a recorded artefact, where no test can make it. Each says what is compared with what.
- Every other test exists at `855eb67` and must still pass. Where one loses or changes an assertion, this plan says so.

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | INT-0003 AC-2 to AC-4; INT-0004 AC-1 | T-001 / WHEN the record is written THEN `ci-sprint6.md` SHALL name every run after `bbffce6` and the merge's run, with event, head, attempt, conclusion, job times, `test:` and `ci:` lines; WHEN the `main` run is recorded THEN the record SHALL say whether a later push cancelled or replaced it; WHEN the dependency update is recorded THEN it SHALL name PR #9, its checks, the owner's approval and the merge commit | review `ci_sprint6_record_matches_github` |
| [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md), [INT-0037](../../../intents/INT-0037-command-set.md) | INT-0016 AC-8; INT-0037 AC-8 | T-001 / WHEN the sign-offs are recorded THEN `sign-offs.md` SHALL quote both questions and answers word for word, each criterion SHALL link it, and no other criterion SHALL be marked signed; WHEN the task is done THEN T-035 SHALL carry what is still pending | review `sign_offs_are_the_owners_words`; review `backlog_carries_what_is_pending` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1 for the two bodies | T-002 / WHEN an operation is encoded THEN its body SHALL be `parents ‖ workspace ‖ view` and an empty workspace SHALL be rejected; WHEN a view is encoded THEN its body SHALL be `workspaces: map<string, entry>` and an empty name or `working_copy` SHALL each be rejected | **new** `operation_names_its_workspace`, **new** `view_holds_one_entry_per_workspace`, **new** `empty_workspace_names_are_rejected`, **new** `empty_working_copies_are_rejected`; `golden_files_match`, `golden_files_match_the_spec` with the four vectors; the roundtrip properties over the new bodies |
| [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | INT-0006 AC-1; closure kept | T-002 / WHEN a view's references are listed THEN they SHALL be each entry's `working_copy` then `heads`, in name order | **new** `view_references_are_each_entry`; `fsck` tests unchanged in what they assert |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 to AC-3 | T-002 / WHEN the suite runs THEN the five matrices SHALL keep their cell counts with new digests as literals, every correct variant SHALL find 0 violations, every control at least one, and `docs/reach.md` SHALL be unchanged | `matrix_runs_are_reproducible`, `writers_matrix_is_reproducible`, `snapshot_matrix_is_reproducible`, `materialise_matrix_is_reproducible`, `describe_matrix_is_reproducible` (cell-count literals unchanged, digest literals new); every `*_catches_*` test; the `reach` step; review `t002_moves_no_cell_count` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0037](../../../intents/INT-0037-command-set.md) | INT-0014 AC-1; INT-0037 AC-2 kept | T-002 / WHEN a repository has one workspace THEN every command SHALL print, exit and leave files as before | every test of `crates/praps/tests/` and `crates/workspace/tests/`; review `t002_changes_no_behaviour_assertion` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-6 | T-002 / WHEN the specification change is written THEN the owner SHALL be asked to sign it off, and a sign-off recorded only from the owner's answer | review `format_sign_off_is_the_owners_answer` |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | AC-1 | T-003 / WHEN the heads are permuted or repeated THEN the merged view SHALL be the same | **new** `merge_ignores_order_and_repeats` |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | AC-2 | T-003 / WHEN two heads with one latest common ancestor are merged THEN each entry SHALL equal the classic three-way merge | **new** `two_heads_equal_the_three_way_merge` |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | AC-3 | T-003 / WHEN some heads are first merged by an operation that changes nothing else THEN merging the others with it SHALL give the flat merge, for writers diverging from one base | **new** `committed_partial_merges_change_nothing` |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | AC-4 | T-003 / WHEN at least 4,000 criss-cross histories are merged THEN each SHALL equal the recursive reference, and a resolved entry SHALL stay resolved | **new** `criss_cross_histories_match_the_reference`, **new** `a_resolution_is_not_reopened` |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md) | INT-0010's ownership rule; INT-0016's Intent (applied at merge) | T-003 / WHEN another workspace's operation carries an older copy of an entry THEN it SHALL not change the merged entry; WHEN one head is given THEN its view; WHEN several THEN each entry by its writers against their base; WHEN entries are merged THEN the pointer and heads rules | **new** `stale_copies_do_not_override_the_owner`, **new** `one_head_is_its_view`, **new** `entries_merge_by_their_rules`, **new** `writers_and_bases_are_found` |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | AC-6 | T-003 / WHEN each registered wrong merge runs through the AC-1 and AC-2 property tests THEN each SHALL fail one; the arbitrary-ancestor merge SHALL fail the AC-4 test | **new** `wrong_merges_are_rejected`, **new** `an_arbitrary_ancestor_fails_criss_cross` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-6 | T-003 / WHEN a merge of several heads is computed THEN the point `workspace.merge.computed` SHALL be marked | **new** `merging_marks_its_point`; `cargo policy` rule `points`; T-009's reach row |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | the Intent: no command fails because of another writer | T-004 / WHEN a command loads several heads THEN it SHALL work from their merge and SHALL NOT fail because of them | **renamed** `several_heads_are_merged_for_every_command` (was `several_heads_stop_every_command`; its assertion of the error becomes success and a merged view); **renamed** `an_operation_written_after_open_is_merged` (was `an_operation_written_after_open_leaves_two_heads`: the next command now succeeds); **renamed** `praps_status_with_several_heads_works` (was `praps_status_with_several_heads_fails`) |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | AC-7 at 2 and 8 readers | T-004 / WHEN such a command changes nothing THEN the stored operations SHALL be what they were | **new** `readers_write_nothing` (2 and 8 processes) |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | the Intent: one operation names every head | T-004 / WHEN such a command changes something THEN it SHALL commit one operation in its own workspace over every head, with only its own entry changed | **new** `a_writer_commits_over_every_head` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0037](../../../intents/INT-0037-command-set.md) | INT-0014 AC-1; INT-0037's rule that an answer says what happened | T-004 / WHEN its own pointer holds several changesets THEN the command SHALL continue from the smallest, keep the others among its heads, print one note naming all, and leave one after it commits | **new** `a_raced_pointer_continues_from_the_smallest`, **new** `praps_notes_a_raced_working_copy` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | INT-0014 AC-1; INT-0009 AC-3 | T-004 / WHEN a write begins THEN its record SHALL name its starting changeset and target; WHEN the target is its workspace's only writer THEN resume SHALL finish; WHEN another operation of the workspace followed or stands beside it THEN resume SHALL drop the record, leave the files and add a note; WHEN the target is absent THEN resume SHALL drop the record and touch no file | **new** `records_name_their_starting_change`, **new** `resume_finishes_while_its_target_is_the_writer`, **new** `resume_leaves_files_when_its_workspace_moved_on`, **new** `resume_finishes_after_another_workspace_commits`; `resume_finishes_an_interrupted_materialise`, `resume_survives_a_second_crash` and the Materialise matrix, unchanged in what they assert |
| [INT-0037](../../../intents/INT-0037-command-set.md) | AC-4 | T-004 / WHEN the task is done THEN no error SHALL concern several heads, and the hint table SHALL not hold one | `every_error_kind_has_a_hint`, `hints_take_one_of_three_forms`, `messages_say_only_what_is_wrong`, each with the variant gone; `every_refusal_prints_one_hint` without its several-heads case |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | the Intent, through the binary | T-004 / WHEN eight `praps new` run at once in one working copy THEN each SHALL exit 0 and the next `status` SHALL exit 0 | **new** `eight_new_at_once_all_succeed` |
| [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | AC-1 to AC-3; the rule "undo is scoped" | T-005 / WHEN `undo` runs in W THEN it SHALL return W's entry to its value before W's writers, as they were when it started, and change no other; WHEN twice THEN back; WHEN W's writer made W or is the root THEN nothing to undo | **new** `undo_is_scoped_to_its_workspace`, **new** `undo_of_a_workspaces_first_operation_has_nothing_to_undo`; `undo_twice_toggles`, `undo_on_the_root_op_fails`, `undo_and_restore_snapshot_first`, unchanged |
| [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | AC-3 | T-005 / WHEN `restore OP` runs in W THEN W's entry SHALL become its value at OP and no other change; WHEN OP has no entry for W THEN it SHALL refuse | **new** `restore_is_scoped_to_its_workspace`, **new** `restore_to_before_the_workspace_refuses` |
| [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | AC-1 to AC-3 kept | T-005 / WHEN a repository has one workspace THEN every `undo` and `restore` SHALL leave the same view object as before; WHEN two workspaces have history THEN one's `undo` and `restore` SHALL leave the other's entry and files | `restore_reaches_every_earlier_state`, `undo_after_every_op_type`, `undo_returns_to_the_parent_view`, `two_hundred_pairs_stay_distinguishable`, unchanged; **new** `history_in_one_lane_leaves_the_other` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-3 | T-006 / WHEN a lane is made THEN the seven steps in order; WHEN `open` is given its directory THEN the named store as NAME | **new** `a_lane_is_made_over_the_same_store`, **new** `open_finds_the_store_a_lane_names` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-3; AC-1 | T-006 / WHEN NAME exists or DIR is not empty THEN refused with nothing but the snapshot; WHEN a crash interrupts making a lane THEN finished or refused | **new** `lane_refusals_write_nothing`, **new** `a_crash_while_making_a_lane_is_finished_or_refused` (a crash at every syscall of making one lane, 10 seeds, both modes) |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-3 | T-006 / WHEN 8 lanes over one store run commands in turn THEN one store, one file per object, each entry its last state, each lane's files its change | **new** `eight_lanes_share_one_store` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-6 | T-006 / WHEN a lane is made THEN the point `workspace.lane.made` SHALL be marked | **new** `making_a_lane_marks_its_point`; `cargo policy` rule `points`; T-009's reach row |
| [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | INT-0037 AC-2, AC-3; INT-0014 AC-3 | T-007 / WHEN `praps lane DIR` runs THEN it SHALL make NAME at DIR over this store and print one line | **new** `praps_lane_makes_a_workspace` |
| [INT-0037](../../../intents/INT-0037-command-set.md) | AC-4 | T-007 / WHEN DIR is inside, NAME is not plain, NAME exists or DIR is not empty THEN exit 1, nothing written but the snapshot, one hint; WHEN a refusal is printed THEN one hint naming registry commands, each kind in the table | **new** `praps_lane_refusals`; `every_refusal_prints_one_hint` and `hints_take_one_of_three_forms` with the new kinds |
| [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) | INT-0037 AC-2; INT-0022 AC-8, AC-9 | T-007 / WHEN `praps lanes` runs THEN every workspace in name order with `*`, name, change ID, path count and other changes of a conflict; no write; exact repeat | **new** `praps_lanes_lists_every_workspace`; `idempotent_commands_repeat_exactly`, `idempotent_commands_declare_no_flag` |
| [INT-0037](../../../intents/INT-0037-command-set.md) | AC-3 | T-007 / WHEN any command runs in a lane, at its top or below THEN on that lane; a top is either kind; `init` refused in a lane | **new** `commands_act_on_the_lane_they_run_in`, **new** `init_inside_a_lane_is_refused`; `commands_act_the_same_from_below` |
| [INT-0037](../../../intents/INT-0037-command-set.md) | AC-2 | T-007 / WHEN `praps ops` lists operations THEN each line SHALL name its workspace | **new** `praps_ops_names_each_workspace`; `praps_ops_lists_operations_newest_first` with its line format changed |
| [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) | INT-0037 AC-1, AC-5; INT-0022 AC-11 | T-007 / WHEN the registry is read THEN the group "Working together", at most ten everyday, `lanes` idempotent with no flag, the README table equal to the list | `registry_names_are_plain_words`, `everyday_group_is_at_most_ten`, `every_command_is_documented`, `praps_alone_lists_every_command`, `praps_help_explains_every_command`, `readme_lists_what_praps_lists` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1, AC-2 | T-008 / WHEN a process reaches a boundary THEN only the drawn process SHALL wake, and the draws SHALL be unchanged | every test of `crates/sim/tests/processes.rs`; `writers_matrix_is_reproducible` with its literals as at the commit before |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | the research risk | T-008 / WHEN 1,000 processes make three syscalls each THEN all SHALL end done, in at most a tenth of the time before | **new** `a_thousand_processes_run` (all done); review `scheduler_at_a_thousand_processes` (the two times recorded) |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | INT-0010 AC-5; INT-0014 AC-3 | T-009 / WHEN 8 lane processes run on two stores for 40 seeds with a kill and a crash THEN no failed command, every acknowledged operation on both stores, both clean, each lane's writer its last acknowledged or in-flight operation, one view on every lane, and one more command each | **new** `concurrency_finds_no_violations` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1, AC-2 | T-009 / WHEN the scenario runs twice THEN equal cells, with count and digest asserted | **new** `concurrency_is_reproducible` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | INT-0005 AC-3 | T-009 / WHEN `FirstHeadOnly`, `MergeOnRead` and `UnsyncedObjects` run it THEN each SHALL produce a violation | **new** `concurrency_catches_first_head_only`, **new** `concurrency_catches_merge_on_read`, **new** `concurrency_catches_unsynced_objects` |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) | AC-7 | T-009 / WHEN 2, 8 and 1,000 readers load several heads and change nothing THEN the operations SHALL be as before and every reader SHALL compute one view; `MergeOnRead` SHALL fail | **new** `readers_add_nothing_at_scale`, **new** `readers_catch_merge_on_read` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-6 | T-009 / WHEN the reach table is generated THEN Concurrency (Sprint 7) SHALL reach both new points | `expectations_are_real`, `reach_table_shows_the_triangle` (with the new row); the `reach` step |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-3 through the binary | T-010 / WHEN 8 real processes in 8 lanes run at once THEN every command exits 0, `lanes` prints the same everywhere, each change keeps its own description, `verify` prints `clean` | E2E **new** `lanes_at_once_end_to_end` (NTFS and ext4) |
| [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | INT-0037 AC-2, AC-3; INT-0009's scoped undo | T-010 / WHEN a session uses only words `praps` printed THEN every command exits 0 and prints its step, and `undo` in one lane leaves the other | E2E **new** `a_lane_session_end_to_end` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | INT-0003 AC-10; the spec as documentation evidence | T-010 / WHEN the working-copy spec is read THEN it SHALL describe the eight things named, and no section SHALL say several heads stop a command | review `specs_match_the_code`; `docs_name_only_commands_praps_has` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-5 | T-010 / WHEN `prior-art.md` is read THEN it SHALL credit Git's linked worktrees and Git's merge strategies | `prior_art_credits_every_listed_system`; review `specs_match_the_code` |
| [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) | the criteria this sprint does not meet | T-010 / WHEN the task is done THEN `tasks.md` SHALL not hold T-029 and SHALL hold the five things it leaves | review `backlog_carries_what_is_pending` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 | Test, with the owner's approval of 2026-11-14 | E2E `three_samples_of_every_job` |

## Unit Tests
### T-001 reviews
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md), [INT-0037](../../../intents/INT-0037-command-set.md)
- review `ci_sprint6_record_matches_github`: every figure in `ci-sprint6.md` is compared with `gh api …/runs/<id>`, `…/jobs` and `gh run view --job <id> --log`, fetched again during Test.
- review `sign_offs_are_the_owners_words`: each quoted question and answer is compared with the conversation of 2026-11-14. The two criteria are the only ones whose detail lines change in this task.
- review `backlog_carries_what_is_pending`: `docs/work/tasks.md` is read after T-001 and again after T-010: T-035 and T-029 as their clauses say, and `cargo book` passes.

### T-002 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- In `crates/codec/tests/`:
  - **new** `operation_names_its_workspace`: an operation with parents, the workspace "agent-17" and a view round-trips, and its body is the three fields in order.
  - **new** `view_holds_one_entry_per_workspace`: a view with two entries, one with a pointer in conflict, round-trips; the map is in name order.
  - **new** `empty_workspace_names_are_rejected`: an operation whose workspace is "", and a view with an entry named "", are each rejected with the error for an empty name.
  - **new** `empty_working_copies_are_rejected`: a view whose entry has an empty `working_copy` is rejected with its own error.
  - **new** `view_references_are_each_entry`: for a view of two entries, `typed_references` lists each entry's `working_copy` then `heads`, entries in name order, each as a changeset.
  - `golden_files_match`, `golden_files_match_the_spec`: with `view`, `view-conflict`, `operation-root` and `operation-merge` rewritten in the spec and in `golden/v1/` in this commit.
  - The roundtrip properties and the decode mutants in `properties.rs` and `mutants.rs`, over the new bodies.
- review `t002_moves_no_cell_count`: `git show` of the task's commit, compared over `crates/*/tests`. The five matrices' cell-count literals are unchanged and only their digest literals change. Every other change to a test only builds an operation or a view in the new shape.
- review `t002_changes_no_behaviour_assertion`: the same diff over `crates/praps/tests/` and `crates/workspace/tests/`. No assertion about what a command prints, its exit code or the files it leaves changes.
- review `format_sign_off_is_the_owners_answer`: the owner's answer to the question asked after the spec text is written, quoted in `sign-offs.md`, or its absence recorded.

### T-003 unit tests
- **Intent:** [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md)
- In `crates/workspace/tests/merge.rs`, over histories generated from a seed: operations by 1 to 4 workspaces, each with one or more parents, each changing only its own entry, with pointer values from a small set so that equal values and values set back are frequent:
  - **new** `one_head_is_its_view`.
  - **new** `entries_merge_by_their_rules`: hand-built cases for the pointer (no writer changed it, one did, two set the same value, two set different values, one set back to the base) and for heads (kept by all, removed by one, added by one).
  - **new** `writers_and_bases_are_found`: hand-built graphs, among them a criss-cross and an entry with no common writer, with the writers and base each one must give.
  - **new** `merge_ignores_order_and_repeats` (AC-1): 2,000 histories; for each, the heads in three random orders and with repeats give one view.
  - **new** `two_heads_equal_the_three_way_merge` (AC-2): 2,000 histories cut at two heads with one latest common ancestor; each entry equals the classic three-way merge of the views, written in the test.
  - **new** `committed_partial_merges_change_nothing` (AC-3): 2,000 histories in which each workspace's concurrent writers diverge from one base. A random subset of heads is merged by an operation that changes nothing else, then merged with the rest; the result equals the flat merge.
  - **new** `criss_cross_histories_match_the_reference` (AC-4): 4,000 histories, each with at least one entry whose writers have two or more latest common writers. The result equals a recursive reference written in the test from the definition alone, with ancestor sets computed by brute force.
  - **new** `a_resolution_is_not_reopened` (AC-4): two writers move a pointer apart, a third merges and resolves it, and heads of other workspaces still carry the conflict; the merge gives the resolution.
  - **new** `stale_copies_do_not_override_the_owner`: W moves its pointer twice. One head of another workspace loaded between the two moves, another before both. The merge gives W's last value with no conflict.
  - **new** `wrong_merges_are_rejected` (AC-6): five wrong merges, each run through the AC-1 and AC-2 properties with their 2,000 histories; each fails at least one, and the test names which. The five are pairwise fold, the first-equals-last fast path, always-base, first-side-wins and keep-duplicates.
  - **new** `an_arbitrary_ancestor_fails_criss_cross` (AC-6): the merge that takes the first common writer by id instead of merging them fails `criss_cross_histories_match_the_reference`'s check.
  - **new** `merging_marks_its_point`: a merge of two heads marks `workspace.merge.computed` once; a load with one head marks nothing.

### T-004 unit tests
- **Intent:** [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0037](../../../intents/INT-0037-command-set.md)
- In `crates/workspace/tests/workspace.rs`:
  - **renamed** `several_heads_are_merged_for_every_command`: the store with two heads that stopped every command now lets each run. Changed assertion: success and a merged view, where it was `SeveralHeads`.
  - **renamed** `an_operation_written_after_open_is_merged`: as before, both operations stand over the old head. Changed assertion: the next command succeeds and merges them, where it stopped.
  - **new** `a_writer_commits_over_every_head`: with three heads, a snapshot of an edit commits one operation whose parents are the three, made in its workspace, whose view differs from the merge only in that entry.
  - **new** `readers_write_nothing`: with three heads, 2 and then 8 processes under `sim::Processes` run `status` with no edit, `log`, `ops` and `describe` with the text the change has. The op files are the same before and after.
  - **new** `a_raced_pointer_continues_from_the_smallest`: two `new` from one state leave the pointer at two changes. `status` with no edit continues from the smaller and writes nothing, with one note naming both. A `describe` then commits; the pointer holds one change and the other stays among the heads.
- In `crates/workspace/tests/materialise.rs` and `src/record.rs`:
  - **new** `records_name_their_starting_change`: the record's `from` is the working-copy changeset and its `target` the operation; a record of the old form is refused as corrupt.
  - **new** `resume_finishes_while_its_target_is_the_writer`: a write interrupted after its operation, with another workspace's operation committed since, is finished by the next open.
  - **new** `resume_leaves_files_when_its_workspace_moved_on`: a write interrupted after its operation, followed by another operation of the same workspace. Resume leaves the files, drops the record and adds a note.
  - **new** `resume_finishes_after_another_workspace_commits`: as the first, with the other workspace's operation committed before the interrupted write's.
  - `resume_finishes_an_interrupted_materialise`, `resume_survives_a_second_crash`, `a_damaged_op_stops_open_before_resume`: unchanged in what they assert.
- In `crates/praps/`:
  - `every_error_kind_has_a_hint`, `messages_say_only_what_is_wrong`, `hints_take_one_of_three_forms`: the several-heads variant leaves the samples and the table.
  - **renamed** `praps_status_with_several_heads_works`: two heads written through the store, then `praps status` exits 0. Changed assertion: exit 0 and no error, where it was exit 1 naming both heads.
  - `every_refusal_prints_one_hint`: its several-heads case becomes a check that the command succeeds; its expected count moves accordingly.
  - **new** `praps_notes_a_raced_working_copy`: after two concurrent `new`, written through the store, the next command prints one `note:` naming both changes.
  - **new** `eight_new_at_once_all_succeed`: eight real `praps new` processes started together in one working copy each exit 0; the next `praps status` exits 0; `praps verify` prints `clean`.

### T-005 unit tests
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- In `crates/workspace/tests/history.rs`:
  - **new** `undo_is_scoped_to_its_workspace`: lanes A and B each commit. `undo` in A returns A's entry to its value before A's last operation and leaves B's entry; a second `undo` in A returns it.
  - **new** `restore_is_scoped_to_its_workspace`: `restore` in A of an operation made in B sets A's entry to its value at that operation and leaves B's.
  - **new** `undo_of_a_workspaces_first_operation_has_nothing_to_undo`: right after lane B is made, `undo` in B fails with nothing to undo.
  - **new** `restore_to_before_the_workspace_refuses`: `restore` in B of an operation from before B existed refuses, with nothing written but its snapshot.
  - **new** `history_in_one_lane_leaves_the_other`: after `undo` and `restore` in A, B's files and entry are byte for byte what they were.
  - `restore_reaches_every_earlier_state`, `undo_after_every_op_type`, `undo_returns_to_the_parent_view`, `undo_twice_toggles`, `two_hundred_pairs_stay_distinguishable`, `ops_are_never_removed`: unchanged. Their view-id equalities are the check that one workspace behaves as before.

### T-006 unit tests
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- In `crates/workspace/tests/lanes.rs`, on the simulator:
  - **new** `a_lane_is_made_over_the_same_store`: the trace of making a lane follows the seven steps in order. Its entry, its `.praps/workspace` and its files are as the clause says; its `.praps` holds no store.
  - **new** `open_finds_the_store_a_lane_names`.
  - **new** `lane_refusals_write_nothing`: a name in use and a directory that is not empty each refuse; the trace after the snapshot holds no write.
  - **new** `a_crash_while_making_a_lane_is_finished_or_refused`: a crash at every syscall of making one lane (and after the last), 10 seeds, both directory modes. In every cell the next command in the lane's directory either finishes its files, or refuses naming the interrupted lane with nothing more written; the store checks clean.
  - **new** `eight_lanes_share_one_store`: 8 lanes made, each running an edit, `status`, `describe` and `new` in turn. One store, one file per object id, each entry its last command's state, each lane's files equal to its change.
  - **new** `making_a_lane_marks_its_point`.

### T-007 unit tests
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md)
- In `crates/praps/tests/cli.rs`, through the binary:
  - **new** `praps_lane_makes_a_workspace`: its one line; the lane's files; `.praps/workspace` naming the store by its full path.
  - **new** `praps_lane_refusals`: four refusals, each exit 1, nothing written but the snapshot, one hint: inside the working copy, a name that is not plain, a name in use, a directory that is not empty.
  - **new** `praps_lanes_lists_every_workspace`: three lanes, one with a pointer in conflict, listed in name order with `*`, change ID, path count and the conflict.
  - **new** `commands_act_on_the_lane_they_run_in`: `status`, `describe`, `new`, `log`, `ops`, `undo`, `restore` and `verify`, at a lane's top and two levels below, act on that lane.
  - **new** `init_inside_a_lane_is_refused`.
  - **new** `praps_ops_names_each_workspace`.
  - **changed** `praps_ops_lists_operations_newest_first`: each line gains its workspace; the rest of its assertions stand.
  - `idempotent_commands_repeat_exactly` takes `lanes` up as idempotent; `commands_act_the_same_from_below`, `praps_alone_lists_every_command`, `praps_help_explains_every_command`, `readme_lists_what_praps_lists`, `every_refusal_prints_one_hint`.
- In `crates/praps/src/`: `registry_names_are_plain_words`, `everyday_group_is_at_most_ten`, `idempotent_commands_declare_no_flag`, `every_command_is_documented`, `the_list_is_rendered_from_the_registry` (the new heading), `hints_take_one_of_three_forms`, `hints_name_registry_commands`.

### T-008 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- Every test in `crates/sim/tests/processes.rs`, unchanged; `writers_matrix_is_reproducible` with its literals as at the commit before.
- **new** `a_thousand_processes_run` (`crates/sim/tests/processes.rs`): 1,000 processes each make three syscalls; every one ends done.
- review `scheduler_at_a_thousand_processes`: the same run in release at the commit before and at this task's commit, on the same machine, both times recorded; the second at most a tenth of the first.

### T-009 unit tests
- Its tests are the integration tests below.

### T-010 reviews
- review `specs_match_the_code`: the working-copy spec is read against T-010's clause, one line of the record for each of the eight things with the heading that holds it. `docs/prior-art.md`'s two new rows are read against the research sources.

## Integration Tests
### The Concurrency scenario
- **Intents:** [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **new** `concurrency_finds_no_violations` (`crates/workspace/tests/concurrency.rs`): the scenario of T-009's first clause, 40 seeds, each with one kill and one crash step drawn from the seed, under both directory modes. Every oracle is checked in every cell. It is the scenario's designated reach record.
- **new** `concurrency_is_reproducible`: the cells of two runs equal, count and digest asserted.
- **new** `concurrency_catches_first_head_only`, **new** `concurrency_catches_merge_on_read`, **new** `concurrency_catches_unsynced_objects`: each at least one violation, of the oracle each is meant to break.
- **new** `readers_add_nothing_at_scale`: 2, 8 and 1,000 reader processes over a store with three heads. **new** `readers_catch_merge_on_read`.
- `expectations_are_real`, `reach_table_shows_the_triangle`: with the Concurrency row; Concurrency passes back through `workspace::snapshot`, `store::commit_op` and `store::put_object`.

### The matrices
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- The five matrices and the new scenario, run with `--nocapture` on both systems; their result lines recorded in `test-matrix-log.md` and compared between the systems.
- Each matrix's cell count at `855eb67` and at the tested head, recorded; they must be equal.

### The suite on both platforms
- **Intents:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `cargo ci` on Windows 11 (NTFS) and in WSL Ubuntu (ext4) at the tested head, from a fresh clone in WSL; the OS, `rustc`, wall time, the `test:` and `ci:` lines.
- `cargo book`, its first and last lines on both systems; `cargo deny check`; `cargo reach` followed by an empty `git status`.
- Totals: those at `855eb67` plus the tests this sprint adds; no test removed; every rename listed with its old name.

### Cost, before and after
- **Intent:** [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md)
- A record, not a test: it traces to no clause and cannot fail the sprint.
- Sprint 6's probe (`status` with nothing changed, and the operation list, at 10 to 800 operations), with the binary at `855eb67` and at the tested head, on both systems.
- One more column: `status` with nothing changed, on a store holding 2, 8 and 32 op heads, with each binary. At `855eb67` that `status` fails, which is recorded.

## End-to-End Tests
- **Status:** possible, except where listed under "Not yet possible".
- **Setup (owner's decision, 2026-11-14).** During Test, `dev` is pushed and the sprint's `dev → main` pull request is opened. Every job of the tested head's run is then re-run twice. Nothing is merged.
- **new** `lanes_at_once_end_to_end` (`crates/praps/tests/e2e.rs`, T-010, NTFS and ext4): 8 lanes made with `praps lane`. 8 real processes, started together, each run in its lane `status` after an edit, `describe` with its own text, and `new`. Pass:
  - every command exits 0;
  - `praps lanes` prints the same lines in every lane;
  - each lane's previous change carries its own description;
  - `praps verify` prints `clean`.
- **new** `a_lane_session_end_to_end` (T-010): `praps` lists `lane` and `lanes`; `praps help lane` gives an example, which is run. Then in lane A: `status`, `describe`, `new`, `log`, `ops`. In lane B: an edit and `status`; `undo` in A, after which B's files and entry are unchanged; `restore` in A of the operation `ops` printed; `verify`. Pass: every command exits 0 and prints what its step names.
- **new** `eight_new_at_once_all_succeed` (T-004): listed under T-004; it is the before-and-after of research's measurement.
- `praps_end_to_end`, `praps_history_end_to_end`, `praps_first_session_end_to_end` and the rest: unchanged, but for `ops` lines naming their workspace.
- `sprint_pull_request_checks_pass`: the pull request's five checks succeed at the tested head; each `cargo ci` leg's `test:` total equals the local one for its platform.
- `three_samples_of_every_job` (INT-0003 AC-11): the tested head's run and two re-runs of all its jobs. Each `cargo ci` job's own start-to-end time is recorded beside 20 minutes and the aim of 15.
  - Pass: every job of all three samples under 20 minutes.
  - A job over 20 minutes is a finding.
- **Not yet possible:**
  - **INT-0010 AC-8** (1,000 writers on 100 replicas) and **AC-9** (merge cost), and **INT-0035 AC-1 to AC-5**: they need the packed storage, derived merged views and the simulator at scale. Unlocked by backlog T-030 and T-032.
  - **Ownership against forged operations** (INT-0016 AC-7; INT-0035 AC-6). Unlocked by signed operations, in INT-0016's sprint.
  - **Real replicas over a network** (INT-0010 AC-5 in full; INT-0035 AC-9). Unlocked by INT-0024.
  - **Nighthawk** (INT-0004 AC-6). Unlocked when the owner approves Tailscale.
