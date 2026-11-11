# Test Critique — Sprint 6

Round 1, on the records as committed at `5fdce92`, with the tested head `4152526`. A separate, read-only critic wrote the concerns. The responses at the end are the primary agent's. The final critique is [critique.md](critique.md).

## Concerns

### C-001: Six refusal kinds are never printed by `praps` in any test; their table rows are paired by hand
- **Where:** `unit-tests.md`, "Where the tests differ from the test plan" and "The hint table" / build plan T-011 clause 1 / `INT-0037` AC-4 / `crates/praps/src/hint.rs` `table()` / `crates/praps/tests/cli.rs` `every_refusal_prints_one_hint`.
- **Quote:** "It does not reach those that need a directory the user may not read or enter, or a change with several parents. … The unit test `hints_take_one_of_three_forms` reads the hint of every kind."
- **Failure mode:** EARS-coverage
- **Why it matters:** T-011 clause 1 names "a `.praps/store` that cannot be listed" among the kinds, and AC-4's threshold is "100% of the error kinds `praps` defines". For these kinds nothing executed shows what `praps` prints:
  - a `.praps/store` that cannot be listed;
  - a directory above that cannot be searched;
  - a top that cannot be entered;
  - `init` stopped by the filesystem;
  - `verify` stopped by the filesystem;
  - `Merge`.

  `table()` pairs a hand-typed message with a hint function the test author chose; no test ties the call sites in `commands.rs` to those functions. The code review found exactly this failure: two call sites used `unreadable_directory()` wrongly. Commit `4152526` then changed the call sites and the table together, still with no test binding them.
- **Suggested response:** add-test. A `#[cfg(unix)]` case with `chmod 000` reaches the three permission kinds and `verify` stopped by the filesystem through the binary. `Merge` can be made as the test already makes two heads, by writing a two-parent changeset through a `Store` handle. Otherwise defer-with-rationale, and have the report say that AC-4's 100% is not met for the six kinds, in place of "reads the hint of every kind".

### C-002: The fix for code-review finding 1 landed at the tested head with no test
- **Where:** `code-review.md` finding 1 / commit `4152526`, `crates/praps/src/commands.rs` `store_for` / build plan T-008, the `verify` clause.
- **Quote:** "**Fixed.** Only absence lets the search go on. Any other failure stops `verify` with that failure and a hint. The spec says so."
- **Failure mode:** negative-path
- **Why it matters:** The bug was a false `clean` for a store that was not asked about, marked "should fix". The diff of `4152526` adds the new arm, but under `tests/` it only rewords two expected strings and two comments. The Windows total is 587 at T-013's commit and 587 at the head, so no test was added. `verify_chooses_its_store_in_order` has no unreadable-directory case, so a regression to `.is_ok()` would pass the suite. `code-review.md` also says "The fixes in `4152526` were not reviewed by a second reader".
- **Suggested response:** add-test (unix: `verify DIR` inside a working copy where `DIR/objects` is unreadable must exit 1, must not print `clean`, and must end with one hint), or defer-with-rationale naming backlog T-036.

### C-003: The hint table in the record is not the table the code review read
- **Where:** `unit-tests.md`, last paragraph of "The hint table" / `code-review.md`, "The hint table, row by row" / build plan T-011 clause 5 / test plan review `hint_table_is_reviewed`.
- **Quote:** "The code review read the table as it stood at `80a3e7e`, with 31 rows. It rejected two rows and found one refusal missing from it; commit `4152526` changed those hints and added the row."
- **Failure mode:** evidence-drift
- **Why it matters:** T-011 requires "a table of every error kind with its message and its hint, read by the code review for whether a person could act on each". The recorded table has 32 rows. No reviewer has given a verdict on the new row, nor on the final wording of four changed hints: `Corrupt`, a directory above that cannot be searched, a top that cannot be entered, and `verify` stopped by the filesystem. The two rejected rows are closed only by the author's own rewording.
- **Suggested response:** tighten-assertion: have a second reader give a verdict on the five rows as they stand at `4152526` and record it. Otherwise defer-with-rationale, stating in the report that 27 of 32 final rows carry a reviewer's verdict.

### C-004: `a_crash_during_a_removal_still_stops_the_write` cannot fail, yet is the named evidence for the crash exception
- **Where:** `unit-tests.md`, "Where the tests differ from the test plan", fourth bullet / test plan T-012 traceability row / `crates/workspace/tests/materialise.rs` / `crates/workspace/src/materialise.rs`.
- **Quote:** "After a crash the simulator fails every later syscall, so the write stops whether or not the code tells a crash from a refusal."
- **Failure mode:** weak-assertion
- **Why it matters:** T-012 clause 1 carves out "for any reason but a crash of the simulator", and the arm `Err(error) if sim::is_crash(&error) => return Err(error)` is the code that keeps it. The record admits the test passes with that arm deleted, so the exception is unproved.
- **Suggested response:** tighten-assertion. The `Held` wrapper already fails only `remove_dir`. If it returned a crash-kind error there and passed later calls through, a write that treated a crash as a refusal would go on, drop the record and leave a note, and the test would fail. If `sim` cannot build such an error outside the simulator, defer-with-rationale and say in the report that the arm is unverified.

### C-005: The new-test table adds up to 88, the record says 87
- **Where:** `unit-tests.md`, "New tests, by task", row T-006, against "Totals" and `integration-tests.md`, "At each task's commit".
- **Quote:** "six registry tests, six help tests, three parser tests"
- **Failure mode:** evidence-drift
- **Why it matters:** The table's rows, plus the four Describe tests and five end-to-end tests it points to, sum to 88; the record says "87 more". At T-006's commit `registry.rs` holds five `#[test]` functions; the sixth, `registry_refuses_other_systems_words`, arrived in T-007 and is also listed in the T-007 row.
- **Suggested response:** tighten-assertion: correct the row to five registry tests, 14 in all. Also name `usage_lines_name_each_argument`, `help_holds_every_part`, `unknown_words_end_with_the_way_to_the_list` and `help_and_version_have_two_spellings_each`, which are in the source, not in the test plan, and not in the list of differences.

### C-006: Deviations from the test plan that the records do not list
- **Where:** `unit-tests.md`, "Where the tests differ from the test plan" / `e2e-tests.md`, "Where the first session differs from the test plan" / `integration-tests.md`, first table.
- **Quote:** "The session as written asks for help on nine commands and checks each usage line, runs `init` as that help shows it"
- **Failure mode:** evidence-drift
- **Why it matters:** Each record presents its list of differences as complete. These are missing or overstated:
  - **`init` "as that help shows it".** `first_session` asserts only that the help starts with the usage line and then runs the hard-coded `init project`. Nothing reads an example from the help and runs it.
  - **`undo_from_a_directory_another_process_stands_in`.** The plan has `d1/d2/f`, with the helper and `undo` in `d1/d2`. The test uses `work/file` and `work/deep/other`, with both in `work`, one level down.
  - **`op_log_lists_once_and_counts_invalid_files`.** The plan says "`op_log` returns the three ops" and "one `read` of each of the five files". The test flips one of the three ops, so it asserts two ops and four reads.
  - **`ci_sprint5_record_matches_github`.** The plan says "the ten job logs". The record says "the six `cargo ci` logs" were fetched again, yet claims every "runner image and version" equals the record.
  - **`cargo reach` on Windows.** The plan wants `docs/reach.md` unchanged after `cargo reach` "On both". The table says "not run by itself" for Windows.
  - **The WSL clone.** `integration-tests.md` says WSL "ran in a fresh clone at the head" and also that "the build was warm from a run at `80a3e7e`". Both cannot describe the same clone.
- **Suggested response:** tighten-assertion: list each in the records. For the first, either parse the example line from `praps help init` and run it, or change the sentence to what the test asserts.

### C-007: The missing-directory clause is run for four of the seven commands
- **Where:** build plan T-008 clause 4 / `crates/praps/tests/cli.rs` `a_missing_directory_is_not_searched_above`.
- **Quote:** `for command in ["status", "new", "log", "undo"] {`
- **Failure mode:** negative-path
- **Why it matters:** The clause covers `restore`, `ops` and `describe` as well, and `restore OP DIR` and `describe TEXT DIR` parse their directory at a different position. The `verify` clause's "or is not a directory" half is also untested: `verify_chooses_its_store_in_order` tries `nowhere` only.
- **Suggested response:** add-test: extend the loop with `ops`, `restore OP` and `describe TEXT`, and add `verify a-file`.

### C-008: "A later command reads the op log once more" is shown for `status` only, and its position is not asserted
- **Where:** build plan T-003 clause 2 / `crates/workspace/tests/workspace.rs` `a_later_command_reads_the_op_log_once_more`.
- **Quote:** "it **SHALL** list `ops/` exactly once and read each op file once, at its start"
- **Failure mode:** weak-assertion
- **Why it matters:** The first, second and third commands are all `status`, which commits nothing. A second command that commits would pass unseen if it re-read the log after its own commit. "At its start" is not checked either.
- **Suggested response:** tighten-assertion: make one of the later rounds a committing command and expect one listing and N+2 reads, or defer-with-rationale.

### C-009: The per-seed syscall figures in the matrix log are quotients, not measurements
- **Where:** `test-matrix-log.md`, "Cell counts and digests, before and after" / test plan, "The matrices, before and after".
- **Quote:** "Each total divides exactly; that every seed makes the same number of syscalls was not checked seed by seed."
- **Failure mode:** evidence-drift
- **Why it matters:** The plan asks for "the scenario's syscall count for one seed". The column is the total divided by 80. The statements "T-003 took 13 syscalls out of each Snapshot run and 66 out of each Materialise run" are therefore averages presented as per-run facts.
- **Suggested response:** tighten-assertion: print `fs.syscalls()` for one seed and mode with `--nocapture` and record it, or reword to "on average per run".

## What was checked and found sound
- **Describe crash scenario** (`describe_crash.rs`): the oracle compares files and description together against the last acknowledged state or a step in flight; liveness re-snapshots under the variant; both controls assert a count above zero; the literals 28,720 and `686dfb8f…` are asserted. The 39 log lines, the per-mode cell counts and the control counts in `test-matrix-log.md` agree with each other.
- **T-003, T-004, T-005 and T-010 unit tests** assert what their clauses say, apart from C-008.
- **`t003_changes_no_assertion`** and **`store_tests_only_gain`** hold.
- **Totals**: 501+86=587 and 502+87=589 agree with the per-commit series, the hint table's 32/17/12/3, the code review's 2+8+21=31, and the GitHub job times.

## Confidence
block

The block is narrow and rests on C-001 to C-003: INT-0037 AC-4's "100% of the error kinds" and T-011's reviewed-table clause are not proved at `4152526`, and a "should fix" correctness fix has no test. Any of C-001 to C-003 may be cleared by a test or by an explicit deferral in the report. C-004 to C-009 are caveats.

## Responses

The tests below are commit `bbffce6`, which is now the tested head. `cargo ci` passed there on both systems: 588 tests on Windows and 590 in WSL.

- **C-001: add-test, with one kind deferred.** `every_refusal_prints_one_hint` now makes five of the six through the binary: a change with several parents on both systems, and on Linux, by file modes, a store that cannot be listed, a directory above that cannot be searched, `init` stopped by the filesystem and `verify` stopped by the filesystem. It asserts how many refusals it made (30 on Linux, 27 on Windows) and fails, rather than skips, if run as root. **Deferred: a top that cannot be made the current directory.** Nothing in the test can close a directory to entry without closing its store too. So INT-0037 AC-4's 100% is shown through the binary for 31 of the 32 kinds across the two systems, and for that one kind only by the unit test's hand-made pairing. `unit-tests.md` says so, and backlog T-036 carries it.
- **C-002: add-test.** The same test runs `verify bak` inside a working copy, where `bak/objects` may not be read. It must exit 1, print nothing on stdout (so nothing is called clean) and end with the hint for a store that cannot be read. Linux only.
- **C-003: tighten-assertion.** A second reader judged the five rows as they stand at `bbffce6`: all accepted, each with a caveat, recorded in `code-review.md`. One caveat is serious and is not fixed in this sprint: on a repository that has lost `ops/`, `praps init` starts a new history and hides the loss. That is `init`'s behaviour from before this sprint; the record explains it, and backlog T-036 names it first.
- **C-004: tighten-assertion.** `a_crash_is_not_taken_for_a_refusal` fails only the removal, with the simulator's own crash error, and lets every later call through. With the arm taken out of `materialise.rs` it fails; that was run once and the source put back.
- **C-005: tighten-assertion.** The row is corrected to 14 tests, five of them the registry's, and the four tests the plan does not name are named. The table's rows now sum to the 88 the head has.
- **C-006: tighten-assertion.** Each is now in the records. The first session reads the example from `praps help init` and runs it as printed. The other nine job logs were read for their images. `cargo reach` was run on Windows, with no difference. The WSL clone is described as it was: made fresh for the first run, then checked out at each later head.
- **C-007: add-test.** The loop runs `status`, `new`, `log`, `ops`, `undo`, `restore OP`, `describe TEXT` and `verify`, each for a directory that is not there and for a file.
- **C-008: tighten-assertion.** A later command that commits is run: one listing and one read of each op file, all before its first write, then the two reads its commit makes.
- **C-009: tighten-assertion, by rewording.** The column is headed as the average it is, and the log says that one seed's count was not printed.
