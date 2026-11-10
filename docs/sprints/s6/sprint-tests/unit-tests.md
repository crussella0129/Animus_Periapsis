# Sprint 6 unit test results

Tested head: `bbffce6`, the last commit that changes code or tests. Two commits made during Test lie between T-013 and it: `4152526`, the fixes from the code review, and `bbffce6`, the tests the first test critique asked for, which changes no source outside tests. The same Windows 11 machine ran both systems: NTFS in the working checkout, and WSL Ubuntu (Linux 6.6.114.1-microsoft-standard-WSL2, ext4) in a clone on ext4, checked out at the head. rustc 1.99.1 on both. The runs did not overlap.

## Totals

| System | `test:` line | `ci:` line |
|---|---|---|
| Windows | 37 units, 588 passed, 0 failed, 0 ignored | all 6 steps passed |
| WSL | 37 units, 590 passed, 0 failed, 0 ignored | all 6 steps passed |

46 of 46 test executables were built and run on both. The suite as a whole is in [integration-tests.md](integration-tests.md).

**Against Sprint 5.** At `b0675f6` the totals were 501 and 502. The source holds 472 test functions there and 560 at the head: 88 more. Linux gained all 88 (590). Windows gained 87 (588), because one of them, `a_removed_current_directory_is_reported`, is for Unix only. Linux now has two tests Windows has not: that one, and the test that needs a real symbolic link on ext4, as before.

**No test was removed.** Eight names are gone from the source, and each is a rename with its assertions kept, except where the build plan says an assertion changes:

| Old name | New name |
|---|---|
| `fsck_exits_zero_on_a_healthy_store` | `verify_exits_zero_on_a_healthy_store` |
| `fsck_lists_corrupt_files_and_exits_one` | `verify_lists_corrupt_files_and_exits_one` |
| `fsck_on_a_non_store_exits_one` | `verify_on_a_non_store_exits_one` |
| `fsck_without_dir_checks_the_current_directory` | `verify_without_dir_checks_the_current_directory` |
| `praps_fsck_checks_a_working_copy` | `praps_verify_checks_a_working_copy` |
| `praps_fsck_reports_a_broken_repository_store` | `praps_verify_reports_a_broken_repository_store` |
| `praps_log_lists_ops_newest_first` | `praps_ops_lists_operations_newest_first` (the count is now paths changed, not files in the tree) |
| `log_lists_ops_newest_first` | `ops_list_newest_first` (the same change to the count) |

The list was made by reading every `#[test]` function at both commits with `git show`, and comparing the two sets by file and name.

## New tests, by task

| Task | File | Tests |
|---|---|---|
| T-002 | `sim/tests/contract.rs`, `sim/tests/points.rs`, `hash/tests/vectors.rs` | `every_method_reaches_through_every_forwarder`, `point_has_no_default_body`, `hex_is_two_lowercase_digits_per_byte` |
| T-003 | `store/tests/store.rs`, `workspace/tests/workspace.rs` | `op_log_lists_once_and_counts_invalid_files`; `open_and_one_command_read_the_op_log_once`, `a_later_command_reads_the_op_log_once_more`, `an_operation_written_after_open_leaves_two_heads`, `the_next_command_builds_on_another_handles_op`, `accessors_read_what_is_stored` |
| T-004 | `workspace/tests/materialise.rs` | `a_damaged_op_stops_open_before_resume` |
| T-005 | `workspace/tests/history.rs` | `undo_and_restore_read_an_untouched_file_once` |
| T-006 | `praps/src/registry.rs`, `help.rs`, `main.rs`; `praps/tests/cli.rs` | 14 in the binary crate: five registry tests (the four the plan names and `usage_lines_name_each_argument`), six help tests (the four the plan names, `help_holds_every_part` and `unknown_words_end_with_the_way_to_the_list`), three parser tests (the two the plan names and `help_and_version_have_two_spellings_each`); `praps_alone_lists_every_command`, `praps_help_explains_every_command`, `praps_help_for_an_unknown_word_suggests`, `help_and_version_spellings`, `unknown_words_get_the_nearest_commands`, `other_systems_words_are_answered`, `idempotent_commands_repeat_exactly` |
| T-007 | `praps/tests/cli.rs`, `praps/src/registry.rs` | `fsck_is_answered_with_verify`, `docs_name_only_commands_praps_has`, `registry_refuses_other_systems_words` |
| T-008 | `praps/src/place.rs`; `praps/tests/cli.rs` | five search tests on the simulator; `no_repository_above_the_test_directories`, `help_and_version_need_no_repository`, `commands_act_the_same_from_below`, `refusals_name_the_working_copy`, `a_missing_directory_is_not_searched_above`, `a_removed_current_directory_is_reported` (Linux), `init_inside_a_working_copy_is_refused`, `praps_init_completes_an_interrupted_init`, `verify_chooses_its_store_in_order`, `a_praps_entry_without_a_store_is_ordinary` |
| T-009 | `workspace/tests/history.rs`; `praps/src/commands.rs`; `praps/tests/cli.rs` | `changes_follow_first_parents_newest_first`, `changes_carry_each_description`, `listing_cost_grows_in_a_straight_line`; `first_line_is_escaped`; `praps_log_lists_changes_newest_first` |
| T-010 | `workspace/tests/history.rs`; `praps/tests/cli.rs` | `describe_sets_the_description`, `describe_records_the_working_copy_first`, `describe_with_the_same_text_writes_nothing`, `a_snapshot_keeps_the_description`, `new_starts_without_a_description`, `describe_is_undone_and_restored`; `praps_describe_sets_and_reports`, `praps_describe_says_when_nothing_changed`, `praps_log_escapes_the_first_line` |
| T-011 | `praps/src/hint.rs`; `workspace/tests/workspace.rs`; `praps/tests/cli.rs` | `every_error_kind_has_a_hint`, `hints_take_one_of_three_forms`, `hints_name_registry_commands`, `quoted_text_is_shown_only_when_it_is_plain`; `messages_say_only_what_is_wrong`; `every_refusal_prints_one_hint`, `describe_with_unquoted_words_hints_quotes`, `describe_with_a_directory_as_its_last_word_takes_one_word` |
| T-012 | `workspace/tests/materialise.rs` | `a_directory_that_cannot_be_removed_is_left`, `a_directory_in_the_way_of_a_file_stops_the_write`, `a_directory_below_a_file_to_put_stops_the_write`, `a_directory_already_gone_is_passed_over`, `a_crash_during_a_removal_still_stops_the_write`, `a_crash_is_not_taken_for_a_refusal` |
| T-013 | `praps/tests/cli.rs` | `readme_lists_what_praps_lists` |

The rows above name 79 tests. With the four tests of the Describe crash scenario and the five new end-to-end tests they are the 88. Those nine are in [integration-tests.md](integration-tests.md) and [e2e-tests.md](e2e-tests.md).

**Where the tests differ from the test plan:**
- The hint tests are in `praps/src/hint.rs`, a module the plan did not name, and not in `commands.rs`.
- `every_refusal_prints_one_hint` makes 30 refusals through the binary on Linux and 27 on Windows, more than the plan lists, and asserts the count. Of the 32 kinds in the hint table:
  - 25 are made on both systems, a change with several parents among them;
  - five more on Linux only: a removed current directory, and four that need a directory closed by its mode (a store that cannot be listed, a directory above that cannot be searched, `init` stopped by the filesystem, and `verify` on a directory it cannot read);
  - two more on Windows only: a hazard, and text with no UTF-8 form;
  - **one is made on neither: a top that cannot be made the current directory.** Nothing in the test can close a directory to entry without closing its store too. Its hint is read only by the unit test, which pairs it with an example message by hand. Backlog T-036 carries it.
  - The table has two rows the corpus counts once each way: "usage error" is made twice (too many arguments, and a flag), and text given as several words that cannot be shown is made in `describe_with_unquoted_words_hints_quotes`.
- Tests not in the plan: `a_directory_already_gone_is_passed_over` and `quoted_text_is_shown_only_when_it_is_plain` (a case found while writing T-012, and which words a hint may show); `a_crash_is_not_taken_for_a_refusal` (below); and the four tests named in the T-006 row.
- `a_crash_during_a_removal_still_stops_the_write` shows the outcome only: after a crash the simulator fails every later syscall, so the write stops whether or not the code tells a crash from a refusal (code review, finding 4). `a_crash_is_not_taken_for_a_refusal` closes that. Its filesystem fails only the removal, with the simulator's crash error, and lets every later call through. With the arm that tells a crash from a refusal taken out of `materialise.rs`, that test fails and the older one passes; this was run once, by hand, and the source put back.
- `op_log_lists_once_and_counts_invalid_files`: the plan says three ops and one read of each of five files. The test damages one of its three ops, so it asserts the two that are whole, three invalid files, and one read of each of the four files whose names are ids.
- `a_missing_directory_is_not_searched_above` runs the clause for every command that takes a directory, each with the directory in the place it takes it: `status`, `new`, `log`, `ops`, `undo`, `restore OP`, `describe TEXT` and `verify`, for a directory that is not there and for a file.
- `a_later_command_reads_the_op_log_once_more` also runs a later command that commits: one listing and one read of each op file, all before its first write, then one more read of its parent and one of its own name.

## Negative controls

Each test below was written first and run where the plan says it must fail.

| Test | Run at | Result there |
|---|---|---|
| `a_damaged_op_stops_open_before_resume` (T-004) | T-003's commit `0ca3cea` | FAILED: `no such file: repo/.praps/working-copy`. The record had been dropped, as the plan predicted. |
| `undo_and_restore_read_an_untouched_file_once` (T-005) | T-004's commit `83d0f6d` | FAILED: `undo scans once`, left 2, right 1. The file was read twice. |
| `listing_cost_grows_in_a_straight_line`, its half for operations (T-009) | T-008's commit `510c3e0` | FAILED: 859, 2,959 and 6,309 syscalls at 51, 101 and 151 operations; the growth was 2,100 and then 3,350. |

At the head the listing test prints, on both systems: `ops: 183, 358, 533 syscalls; growth 175 then 175` and `changes: 202, 402, 602 syscalls; growth 200 then 200`.

T-012 has no control in its plan. One was taken anyway, with `praps` built at T-011's commit `4adba79` on Windows, while a second process had the directory as its current directory:
- `praps undo` exited 1 with `The process cannot access the file because it is being used by another process. (os error 32)`;
- the next command failed the same way, with `.praps/working-copy` still there;
- after the other process left, the next command finished the write.

At the head the same `undo` exits 0 ([e2e-tests.md](e2e-tests.md)).

## Reviews the test plan asks for

- **`helpers_have_one_definition` (T-002).** The search, run at T-002's commit:

  ```
  grep -rnE "fn (parent_dir|dir_exists|on_disk|listing|hex|write_hex)\b" crates/*/src
  ```

  It found one definition each of `parent_dir` and `dir_exists` (`sim/src/fs.rs`), `hex` with its writer `write_hex` (`hash/src/lib.rs`), the join of a repository path onto a root, `on_disk`, and the listing of a directory by name, `listing` (`workspace/src/materialise.rs`). The second `listing` there is a method of the lazy reader that calls the first. A search for the two-digit hex format (`02x`, `:x}`) outside `xtask` found 14 places: `hash::write_hex` itself; ten that print one byte in an error message (`codec/src/error.rs`, `hash/src/lib.rs`); two escapes in a name (`sim/src/name.rs`); and the algorithm id that starts a store file's name (`store/src/names.rs`). None formats a string of bytes.
- **`t003_changes_no_assertion` (T-003).** Yes. The code review read `git show 0ca3cea -- crates/*/tests`: two files gained tests at their ends, and the only lines removed under any `tests/` directory are the cell count and digest of Snapshot and of Materialise.
- **`store_tests_only_gain`.** Yes. `git diff b0675f6..80a3e7e -- crates/store/tests` is 67 insertions and no deletion, one test appended. The later commit `4152526` does not touch that directory.
- **`ci_sprint5_record_matches_github` (T-001).** On 2026-11-10, during Test, the three runs and their fifteen jobs were fetched again with `gh api …/actions/runs/<run>` and `…/jobs`, and the six `cargo ci` logs with `gh run view --job <job> --log`. The other nine logs were then read for their runner image and version. Every event, head, conclusion, attempt, start and end time, `test:` line and `ci:` line, and the image and version of all fifteen jobs, equals what [ci-sprint5.md](ci-sprint5.md) records. `gh run list --branch main` still shows the merge's run as the latest on `main`. The toolchain and cache cells of the record were not read again.
- **`sign_offs_are_the_owners_words` (T-001).** The record's search was run again and finds the same 17 criteria. The record marks none as signed, and the conversation of this sprint holds no words of the owner that sign one: the owner's approval of the plan chose three things (the name `verify`, `log` and `ops`, and the GitHub actions) and nothing else.
- **`backlog_carries_what_is_pending`.** `docs/work/tasks.md` holds T-035 with the runs after the tested head, eight sign-offs and the three samples of INT-0003 AC-11. T-024, T-025 and T-018 are named there only inside T-035's own text, as work done. `cargo book` passes.
- **`specs_match_the_code` (T-013).** In `docs/spec/working-copy.md`:

  | What T-013 names | Heading |
  |---|---|
  | `describe` | Snapshot |
  | `log`, `ops` | Writing the working copy, History commands |
  | How a command finds its working copy; `init` inside a working copy | Finding the working copy |
  | One read of the op log for a command | Reading the op log |
  | The check of the op log before resume | Resume |
  | The single scan of `undo` and `restore` | History commands |
  | A directory a write cannot remove | Materialise |
  | The commands that snapshot, all seven | Snapshot |

  `docs/spec/storage-durability.md`, "What remains asserted", now names Sprint 4's simulated and real four-writer runs and says what neither shows. `docs/prior-art.md` credits the Command Line Interface Guidelines, Mercurial for the name `verify`, and Dolt among designs evaluated. Both licences were read from the projects' repositories on 2026-11-10.
- **`hint_table_is_reviewed` (T-011).** See the next section and [code-review.md](code-review.md).

## The hint table

Printed by `hints_take_one_of_three_forms` with `--nocapture` at `4152526`; `bbffce6` changes no hint. Backticks inside a cell are shown as apostrophes. A ` / ` stands for a line break.

| Kind | Message | Hint |
|---|---|---|
| workspace::Error::Io | `praps undo: project: Access is denied. (os error 5)` | `hint: run 'praps verify' to check what is stored; if it is clean, the cause is outside the repository` |
| workspace::Error::NotInitialised | `praps undo: project: not an initialised Periapsis repository` | `hint: run 'praps init' in that directory to finish making it a repository` |
| workspace::Error::SeveralHeads | `praps undo: project: the op log has 2 heads, which cannot be merged yet: blake3:d33fb48ab5adff269ae172b29a6913ff04f6f266207a7a8e976f2ecd571d4492 blake3:dc770fff53f50835f8cc957e01c0d5731d3c2ed544c375493a28c09be5e09763` | `hint: this build cannot join operations made at the same time yet; nothing was lost` |
| workspace::Error::Corrupt | `praps undo: project: the repository is corrupt: 2 of 9 op files do not hash to their names or do not decode` | `hint: run 'praps verify' to list what is damaged in the store; this build cannot repair what it finds, or what the message names outside the store` |
| workspace::Error::Merge | `praps undo: project: changeset blake3:18aa4baf95441d4aed8189d9ad1c5688dcd61fda51cb2c7958d91d1e80cb1189 has several parents; merges are not supported yet` | `hint: this build cannot read a change that has several parents yet; nothing was lost` |
| workspace::Error::Hazards | `praps undo: project: the working copy cannot be written on this filesystem; no file was changed: /   CON: 'CON' 'CON' is a reserved Windows device name` | `hint: this build cannot write those paths on this filesystem yet; nothing was lost` |
| workspace::Error::Blocked | `praps undo: project: untracked entries are in the way, and no file was changed: /   b: an untracked entry is where a file must be written` | `hint: move those entries out of the way, then run the command again` |
| workspace::Error::NothingToUndo | `praps undo: project: nothing to undo: the last op is the root op` | `hint: run 'praps ops' to list the operations there are` |
| workspace::Error::UnknownOp | `praps undo: project: no op matches '3f9a'` | `hint: run 'praps ops' to list the operations and their ids` |
| workspace::Error::AmbiguousOp | `praps undo: project: '3' matches 2 ops: /   blake3:d33fb48ab5adff269ae172b29a6913ff04f6f266207a7a8e976f2ecd571d4492 /   blake3:dc770fff53f50835f8cc957e01c0d5731d3c2ed544c375493a28c09be5e09763` | `hint: give more digits of the id, enough to name one of those` |
| usage error | `usage: praps status [DIR]` | `hint: run 'praps help status' for what it takes` |
| usage error, no command named | `usage: praps COMMAND [ARGUMENT]...` | `hint: run 'praps help' to list the commands` |
| a word that is not a command | `praps: 'statsu' is not a command` | `hint: run 'praps help' to list the commands` |
| an argument that cannot be an operation's id | `usage: praps restore OP [DIR]` | `hint: run 'praps ops' to list the operations and their ids` |
| text with no UTF-8 form | `usage: praps describe TEXT [DIR]` | `hint: give TEXT as Unicode text, which can be stored as UTF-8` |
| text given as several words | `usage: praps describe TEXT [DIR]` | `hint: run 'praps describe "Add the parser"' to give those words as one text` |
| text given as several words that cannot be shown | `usage: praps describe TEXT [DIR]` | `hint: run 'praps help describe' for how to give text of more than one word` |
| text given as two words, the second no directory | `praps describe: parser: no such directory` | `hint: run 'praps describe "Add parser"' if both words are the text` |
| a directory that is not there | `praps status: nowhere: no such directory` | `hint: give a directory that exists, or none for the current directory` |
| a file given as the directory | `praps status: notes.txt: not a directory` | `hint: give a directory that exists, or none for the current directory` |
| no repository there or above | `praps status: /home/me/work: no Periapsis repository here or in any directory above` | `hint: run 'praps init' in the directory that is to be the top of the working copy` |
| a `.praps/store` that cannot be listed | `praps status: work/.praps/store: Permission denied (os error 13)` | `hint: make that directory readable, then run the command again` |
| a directory above that cannot be searched | `praps status: /home/me/work: a directory above it could not be searched for a repository: Permission denied (os error 13)` | `hint: make the store of each repository above that directory readable, then run the command again` |
| a top that cannot be entered | `praps status: /home/me/work: Permission denied (os error 13)` | `hint: make that directory one you may enter, then run the command again` |
| a current directory that cannot be read | `praps status: the current directory cannot be read: No such file or directory (os error 2)` | `hint: change to a directory that exists, or give the working copy's directory` |
| `init` inside a working copy | `praps init: sub: it lies inside the working copy at /home/me/work, and a repository is not made inside another` | `hint: run 'praps status' to work in that working copy, or give 'praps init' a directory outside it` |
| `init` stopped by the filesystem | `praps init: project: Permission denied (os error 13)` | `hint: run 'praps init' again once the cause is put right: it finishes what it began` |
| `verify` given a directory that is not there | `praps verify: nowhere: no such directory` | `hint: give the directory of a working copy or of a store` |
| `verify` on a directory that is no store | `praps verify: plain: plain is not a Periapsis store (it needs objects/ and ops/)` | `hint: run 'praps init' to make a repository, or give 'praps verify' the directory of one` |
| `verify` on a working copy's store that is no store | `praps verify: work: work/.praps/store is not a Periapsis store (it needs objects/ and ops/)` | `hint: run 'praps init' there if its 'init' was interrupted; if the store was whole before, put it back from a copy of the repository` |
| `verify` stopped by the filesystem | `praps verify: work: Permission denied (os error 13)` | `hint: make the store's directories readable, then run the command again` |
| `verify` finds problems | `3 problems` | `hint: put back the files named above from a copy of the repository; this build cannot repair a store` |

32 kinds: 17 name a command, 12 say what to put right, 3 say there is no remedy yet

The code review read the table as it stood at `80a3e7e`, with 31 rows. It rejected two rows and found one refusal missing from it; commit `4152526` changed five hints and added the row. A second reader then judged those five rows as they stand. Both sets of verdicts are in [code-review.md](code-review.md).
