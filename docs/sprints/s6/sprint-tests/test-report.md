# Sprint 6 Test Report

**Tested head:** `bbffce6`. It carries T-001 to T-013 and two commits made during Test: `4152526`, the fixes from the [code review](code-review.md), and `bbffce6` itself, the tests the first test critique asked for. Every commit after it changes records only.

**Where the results are:**
- [unit-tests.md](unit-tests.md), with the negative controls, the reviews the test plan names and the hint table;
- [integration-tests.md](integration-tests.md), with the cost before and after;
- [e2e-tests.md](e2e-tests.md), with GitHub's runs;
- [test-matrix-log.md](test-matrix-log.md);
- [code-review.md](code-review.md);
- [ci-sprint5.md](ci-sprint5.md), Sprint 5's GitHub record;
- [critique-round1.md](critique-round1.md) and [critique.md](critique.md).

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0037](../../../intents/INT-0037-command-set.md) | AC-1: every name one plain word, none a refused word | T-006, T-007 / `registry_names_are_plain_words`, `registry_refuses_other_systems_words`, `fsck_is_answered_with_verify`, `only_registry_words_dispatch` | pass | Test evidence links this report |
| INT-0037 | AC-2: `praps` lists every command, `help WORD` explains each, a word within two edits gets its command | T-006, T-013 / `praps_alone_lists_every_command`, `praps_help_explains_every_command`, `unknown_words_get_the_nearest_commands` (a corpus built from every listed name), `help_and_version_spellings`, `readme_lists_what_praps_lists`; E2E `praps_first_session_end_to_end` | pass, on Windows and Linux | as above |
| INT-0037 | AC-3: the same output and state from the top and from below; `init` inside a working copy refused | T-008, T-012 / `commands_act_the_same_from_below` (eleven command lines, three places each), `refusals_name_the_working_copy`, `a_missing_directory_is_not_searched_above` (eight commands), `init_inside_a_working_copy_is_refused`, `verify_chooses_its_store_in_order`; E2E `praps_end_to_end_from_below`, `praps_first_session_from_below`, `undo_from_a_directory_another_process_stands_in` | pass, on Windows and Linux | as above |
| INT-0037 | AC-4: every refusal says what was wrong and what to do next, for 100% of the kinds `praps` defines | T-011 / `every_error_kind_has_a_hint` (the ten `workspace` errors, by a match with no catch-all), `hints_take_one_of_three_forms`, `hints_name_registry_commands`, `messages_say_only_what_is_wrong`; through the binary `every_refusal_prints_one_hint`, 30 refusals on Linux and 27 on Windows; the table of 32 kinds, read by the code review and a second reader | **The 100% threshold is not claimed.** Through the binary: 31 of the 32 kinds, on at least one system (23 on both, five more on Linux, two more on Windows, one in another test). The last, a top that cannot be made the current directory, is shown only by its hint's form and a reader's reading of the code. Three places that print a hint already shown elsewhere are run by no test. Every kind named in T-011's own clause is printed in a test. Whether a hint can be acted on is a reader's judgment: all 32 rows are accepted as they stand, eleven with a caveat. Backlog T-036 carries the rest | as above; the chapter stays `active` |
| INT-0037 | AC-5: the everyday group holds at most ten | T-006 / `everyday_group_is_at_most_ten` (five today) | pass | as above |
| INT-0037 | AC-6, AC-7, AC-8 | not planned for this sprint: agents, a panel, and the owner's sign-off on the vocabulary | not tested; AC-8 is unsigned | — |
| [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) | AC-8: no flag on an idempotent command; AC-11: one description of each command | T-006 / `idempotent_commands_declare_no_flag`, `usage_lines_come_from_the_registry`, `the_list_is_rendered_from_the_registry`, `only_registry_words_dispatch` | pass | Test evidence links this report |
| INT-0022 | AC-9: an idempotent command repeats exactly | T-006 / `idempotent_commands_repeat_exactly`, over the commands whose help says so (`status`, `log`, `ops`, `verify`, `help`, `version`) | pass for the commands built; the criterion stays open for the rest | as above; the chapter stays `active` |
| [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) | none in full; cost rule 2, a first step | T-003, T-009 / `open_and_one_command_read_the_op_log_once` (eleven commands), `a_later_command_reads_the_op_log_once_more`, `op_log_lists_once_and_counts_invalid_files`, `listing_cost_grows_in_a_straight_line` with its failing run at T-008's commit | the clauses pass. **No criterion of this chapter is met**, and two agents still cannot share a repository | Test evidence links this report; the chapter stays `active` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1 kept: no command loses an edit | T-003, T-004, T-005, T-008, T-010, T-012 / the edit corpora unchanged; `an_operation_written_after_open_leaves_two_heads`, `accessors_read_what_is_stored`, `a_damaged_op_stops_open_before_resume` with its failing run at T-003's commit, `describe_records_the_working_copy_first`, the Describe crash scenario, `a_directory_that_cannot_be_removed_is_left` and its three stopping cases | pass | Test evidence links this report |
| [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | AC-1 to AC-3 kept | T-005, T-009, T-010, T-012 / `undo_and_restore_read_an_untouched_file_once` with its failing run at T-004's commit, `two_hundred_pairs_stay_distinguishable`, `restore_reaches_every_earlier_state` and `undo_after_every_op_type` with `describe` among the operations, `ops_are_never_removed` with `describe` drawn at random, `describe_is_undone_and_restored` | pass | Test evidence links this report |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 to AC-3, AC-6 | T-002, T-003, T-005, T-010 / five matrices with asserted cell counts and digests, equal on Windows and Linux; 0 violations; every control caught, the new `DropDescription` among them; `docs/reach.md` generated with the row Describe and the unit `workspace::describe` | pass. The store matrices kept Sprint 5's literals at every commit | Test evidence links this report |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-2 (`main` and on demand), AC-3, AC-4 | T-001 / [ci-sprint5.md](ci-sprint5.md), checked again against GitHub during Test; this sprint's pull request run | pass. The run on `main` was neither cancelled nor replaced, and the run on demand succeeded | Sprint 5 CI record and this report linked |
| INT-0003 | AC-5 (credits), AC-10 (README) | T-013 / `prior_art_credits_every_listed_system` with two names added, `readme_keeps_statement_and_links_book`, `readme_must_document_each_command`, `readme_lists_what_praps_lists` | pass; the AC-5 sign-off is still pending | as above |
| INT-0003 | AC-11: three samples under 20 minutes | not measured: it needs two re-runs of every job, which the owner did not approve | **not tested this sprint.** One sample: the Windows suite took 16 min 51 s at the tested head, under 20 minutes and over the aim of 15 | backlog T-035 |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: the suite on both systems | every task on Windows; the head on Windows 11 (588) and WSL Ubuntu (590), rustc 1.99.1; GitHub's `windows-2025-vs2026` and `ubuntu-26.04` | pass | Test evidence links this report |
| INT-0004 | AC-7 kept for `log` and `describe` | T-009, T-010 / `snapshotting_commands_return_reports`, E2E `praps_commands_report_skipped_names`, `praps_log_escapes_the_first_line` | pass | as above |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md), INT-0003, INT-0004 | the owner's sign-offs | T-001 / the list of 17 criteria in [ci-sprint5.md](ci-sprint5.md), found again by its search | **none is signed.** No words of the owner in this sprint sign one. Eight are carried in backlog T-035 | — |

## Summary
- **Unit tests:** 588 passed on Windows and 590 on Linux at `bbffce6`, with 0 failed and 0 ignored. 88 test functions are new, eight were renamed, and none was removed.
- **Negative controls:** the three tests the plan wanted to see fail first did fail where it said: at T-003's, T-004's and T-008's commits.
- **Integration tests:**
  - `cargo ci` passed in 288 s on Windows and 186 s in WSL, and at every task's commit on Windows.
  - The five matrices have 0 violations and every control is caught. Snapshot and Materialise shrank because commands make fewer syscalls; no count rose.
  - `cargo book` reports 38 chapters, 267 criteria and 0 violations under the strict rule, and `cargo deny check` passes, on both systems.
- **Cost, as a record and not a criterion:** at 800 operations the operation list went from 21.6 s to 0.37 s on Windows and from 1.8 s to 0.03 s on Linux, and `status` on an unchanged working copy from 401 ms to 159 ms and from 64 ms to 16 ms. Nothing was compared with another system.
- **E2E tests:** 10 passed on NTFS and ext4. A session of only the words `praps` printed works at the top and from two levels down. `undo` from a directory another process stands in exits 0 on both systems: Windows leaves the directory with a note, Linux removes it.

Not yet possible: INT-0003 AC-11's three samples; INT-0037 AC-6 and AC-7; two agents on one repository; every criterion of INT-0035 and INT-0036; Nighthawk. [e2e-tests.md](e2e-tests.md) names what unlocks each.

## CI Confirmation
- **Sprint 5's checkpoint** ([ci-sprint5.md](ci-sprint5.md)). The pull request's last run, the merge's run on `main` and one run on demand all concluded `success`, with 501 tests on Windows and 502 on Linux.
- **This sprint's pull request,** [crussella0129/Animus_Periapsis#8](https://github.com/crussella0129/Animus_Periapsis/pull/8), opened during Test with the owner's approval. Nothing was merged and no job was re-run.
  - run 42856495316 at `80a3e7e`: replaced by the next push, as designed;
  - run 42858372557 at `4152526`: all five jobs `success`;
  - run 42867476706 at `bbffce6`, the tested head: all five jobs `success`, 588 tests on Windows and 590 on Linux.

  The authoritative conclusion is **success**, from run 42867476706, attempt 1, finished 2026-11-10T17:58Z. The commits after `bbffce6` change records only; their runs are carried in backlog T-035.

## Failures
None at the tested head. Problems that were found and fixed:
- **During Build:**
  - T-005's edit script failed without a visible error, so one spec sentence and one comment missed the task's commit; a follow-up commit added them.
  - T-008's first `cargo ci` failed its policy step: a test filesystem forwarded a named point, which the points rule forbids.
  - Before T-012, `praps undo` failed on Windows whenever another process stood in a directory the write emptied. T-012 was added to the plan for that, and a run of the T-011 binary shows the failure.
- **Code review:** 9 findings and no blocker. Both "should fix" findings are fixed: `verify` could call a store clean that it had not been asked about, and the hint for a corrupt repository led nowhere for three errors. Of the hint table's rows, two were rejected and reworded.
- **Round-1 test critique:** 9 concerns, verdict `block`. Seven are answered with tests or corrected records. Two are answered in part, with the rest deferred and stated: one kind of refusal is still printed by no test, and the second reader's most serious caveat is carried forward.
- **Final test critique:** 4 concerns, verdict `proceed-with-caveats`, all answered in the records ([critique.md](critique.md)). It holds on the condition this report keeps: INT-0037 AC-4's 100% is not claimed.

## Technical Debt Identified
- **`praps init` on a repository that has lost `ops/`** starts a new history and says nothing, and two hints suggest `init` for that state. Nothing is deleted, but the loss is hidden. This is older than the sprint; the sprint's reviews found it. Backlog T-036, first item.
- **One refusal is printed by no test:** a top that cannot be made the current directory. Three places that print a hint are run by no test. Backlog T-036.
- **Hints that readers accepted with a caveat** (an I/O failure with no path in its message; a hazard that moving an entry would cure; `verify`'s problems when they are permission failures; text that begins with `-`). Backlog T-036.
- **The helper variable `PRAPS_E2E_STAND`** is not kept out of `cargo ci`'s test step, as `PERI_WRITER_CHILD` is. Backlog T-036.
- **Two commands at the same time still leave two op heads.** The direction of this sprint's Book changes cannot be met until the canonical merge exists. Backlog T-029.
- **Recording many new files is still slower than Git**, because each file is flushed twice. Backlog T-030.
- **`describe_crash` is the second slowest unit** (166 s on Windows), beside `materialise_crash` (285 s). The Windows suite on GitHub took 16 min 51 s, over the aim of 15 minutes. Backlog T-027 already carries the split.
- **The tooling and harness findings of the research audit** stay in backlog T-033 and T-034.
- **The owner's sign-offs:** eight are carried in backlog T-035, with the runs after the tested head and INT-0003 AC-11's three samples.
