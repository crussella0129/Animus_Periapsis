# Plan Critique — Sprint 6

Round 2. The same read-only subagent reviewer re-reviewed the plans at `8a14423` against its round-1 concerns ([critique-round1.md](critique-round1.md)) and screened them afresh. The concerns below are new or partial, given as it wrote them with file paths made relative to the repository; the responses are at the end. Round 2 added one task, T-012, so the documentation task that round 1's responses call T-012 is now T-013.

## Concerns

### C-001: T-008's rules conflict in three places
- **Where:** `build-plan.md` T-008, clauses 1, 4, 5, 7, 8 and 9; `test-plan.md` T-008 unit tests; `crates/praps/tests/cli.rs` (`praps_init_creates_a_repository`); `docs/spec/working-copy.md` (Layout, step 4)
- **Quote:** "if the directory is a top, it **SHALL** answer as today and write nothing"
- **Failure mode:** EARS-vague
- **Why it matters:** (a) A top whose `init` stopped after `.praps/store` was made has no root operation: "as today" completes it, "write nothing" forbids that, and the planned tests cover only a whole repository and an empty `.praps`; inside another working copy that completion writes, which INT-0037 AC-3 forbids. (b) For `verify` in a directory that holds a `.praps` with no store, rule 1 ("holds an entry named `.praps`") checks a store that is not there, while clause 9 says a command beside it acts on the working copy above; no test decides. (c) Clauses 1, 4 and 5 are not limited to commands that open a working copy: by the letter `help` and `version` exit 1 outside a repository, `praps init repo` on a missing directory fails, and `verify nowhere` inside a working copy has two answers (clause 4 and rule 3).
- **Suggested response:** fix-in-plan. Say that `init` on a top completes a store that has no operation; give rule 1 the same test as "a top", or except `verify` from clause 9; scope clauses 1, 4, 5 and 6; add a test case for each.

### C-002: The new `log` and `describe` have no test that they report skipped names, or that `log` records the working copy first
- **Where:** `build-plan.md` T-009 clause 1 and T-010 clause 1; `test-plan.md` T-009 and T-010 unit tests, End-to-End Tests; `crates/praps/tests/e2e.rs` (`praps_commands_report_skipped_names`); `crates/workspace/tests/history.rs` (`snapshotting_commands_return_reports`)
- **Quote:** "updated to `verify` in T-007 and to `ops` in T-009 where they used `fsck` and `log`."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Those two tests are what holds INT-0004 AC-7 ("never ... skipped without a report") for each command that snapshots. After T-009 they follow `ops`, so the rewritten `log` and, after T-010, `describe` can drop the report unseen, the fault Sprint 4's T-009 fixed for `undo`, `restore` and `new`. No clause or test says the new `log` records the working copy first either (`changes_follow_first_parents_newest_first` has no pending edit), which INT-0014 and INT-0022 require of every command that opens it.
- **Suggested response:** fix-in-plan. Add `log` and `describe` to both tests, and a pending edit to the `changes()` test, with a clause in T-009 and T-010.

### C-003: T-003's read-once test is broken by T-009's rename, and never covers the two commands added later
- **Where:** `build-plan.md` T-003 clause 1, T-009 and T-010 Touches; `test-plan.md` T-003 unit tests (`open_and_one_command_read_the_op_log_once`)
- **Quote:** "(`snapshot`, `status`, `new_change`, `log`, `undo`, `restore` or `restore_view`)"
- **Failure mode:** hidden-dep
- **Why it matters:** That test lives in `crates/workspace/tests/workspace.rs` and calls `log()`, which T-009 renames, but the file is in neither T-009's nor T-010's Touches. The commands those tasks add, `changes` and `describe`, never join the list, so "a command reads the op log once" (INT-0014) is unchecked for them: a `describe` built as `new_change` is today, snapshot then load, would read twice and pass.
- **Suggested response:** fix-in-plan. Add the file to both tasks' Touches and say that T-009 and T-010 each add their command to the test.

### C-004: The one behaviour T-003 changes has no clause and no test
- **Where:** `build-plan.md` T-003 clause 6 and Notes; `test-plan.md` `accessors_read_what_is_stored`, review `t003_changes_no_assertion`; `docs/intents/INT-0014-working-copy-and-workspaces.md` Consequences
- **Quote:** "this task moves cost and no behaviour."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** INT-0014 now records the exception: the first command after `open` does not see an operation another handle wrote in between, and "both stand, the log has two heads, and the next command says so". No clause or test runs that case (`accessors_read_what_is_stored` stops before any command), so the chapter's promise that nothing is lost there rests on reading the code.
- **Suggested response:** fix-in-plan. Add a clause and a test: after `open` another handle commits; the first command succeeds; both operations are in the log; the next command stops with `SeveralHeads`. Word clause 6 as "no existing assertion changes".

### C-005: T-008 does not say what happens when the write removes the directory the command was started in
- **Where:** `build-plan.md` T-008 clause 2 and Notes; `test-plan.md` `commands_act_the_same_from_below`; `crates/workspace/src/materialise.rs` (`apply`, step 2)
- **Quote:** "`praps` makes the top its current directory and runs as if started there."
- **Failure mode:** missing-risk
- **Why it matters:** `undo` and `restore` remove a directory they empty. Windows will not remove a directory that is another process's current directory, as it is for `cmd.exe`, so started there the command fails after its operation is committed, the record stays, and every later command from that directory fails in resume until the user leaves it; on Linux the removal succeeds and the next command has no current directory, which no clause names. The equality test cannot see either, because its child process moves itself and no shell stands below.
- **Suggested response:** fix-in-plan or defer-with-rationale. For example, leave an emptied directory that cannot be removed, with a note, and add a clause for a current directory that cannot be read; or record both in INT-0014 and backlog T-034.

### C-006: Three checks cannot observe what the plan says they observe
- **Where:** `test-plan.md` T-008 unit tests (`find_stops_on_a_failure_that_is_not_absence`, `a_praps_directory_without_a_store_is_not_a_top`) and T-002 review `helpers_have_one_definition`; `build-plan.md` T-008 Notes; `crates/sim/src/simfs.rs` (`syscall`; `State::dir`); `crates/codec/src/error.rs`
- **Quote:** "The search from `a/b` returns that failure and makes no further syscall, though `a` is a top."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The simulator can fail a syscall only by crashing, and after a crash every later syscall fails too and is neither counted nor traced, so "no further syscall" cannot be seen and `a` could not be listed anyway. The note's second kind of absence, "a name on the way is not a directory", is `NotFound` on the simulator but `NotADirectory` on Linux, so no planned test meets that error. The review's "each has one result" is false for the hex format string, which stays in `codec`'s error messages, `sim`'s name escapes and the store's algorithm prefix.
- **Suggested response:** fix-in-plan. Wrap the simulator in a test `Fs` that fails one path; add a case through the binary with a file named `.praps`; say which hex sites the search may keep.

### C-007: T-001's sign-off clause says "every" and then lists ten
- **Where:** `build-plan.md` T-001, fifth clause; `test-plan.md` review `sign_offs_are_the_owners_words`
- **Quote:** "the record **SHALL** list every acceptance criterion the project owner must sign that is still unsigned, in two groups"
- **Failure mode:** EARS-vague
- **Why it matters:** The Book holds at least eight more criteria that only the owner signs and has not: INT-0003 AC-7, INT-0011 AC-5 and INT-0032 AC-7, all in chapters Sprint 6 revised, and INT-0020 AC-6, INT-0025 AC-3, INT-0031 AC-1, INT-0033 AC-5 and INT-0034 AC-8. The review cannot tell whether a record of ten meets the clause.
- **Suggested response:** fix-in-plan. State the rule that picks the ten, or list them all.

### C-008: INT-0022's new definition of the idempotent class has two tests that part
- **Where:** `docs/intents/INT-0022-agent-and-human-interfaces.md`, Intent (the paragraph revised in round 1); `docs/intents/INT-0003-engineering-baseline.md` AC-9
- **Quote:** "A command is **idempotent** when it only reports: run again, it changes nothing and prints the same"
- **Failure mode:** intent-drift
- **Why it matters:** A command that writes and still repeats exactly meets the second half and not the first. `cargo reach` is one: it rewrites `docs/reach.md`, INT-0003 AC-9 calls it idempotent and flagless, and this paragraph still says the tooling "follows the same rule" while putting every command "asked to change something" in the class that may take flags. A later `praps` command of that kind has no settled class.
- **Suggested response:** fix-in-plan. Define the class by the repeat (changes nothing and prints the same), which already leaves out `init` and `describe`, and keep "only reports" as description; or name `cargo reach` as the case.

### C-009: Links and one dependency the split left out
- **Where:** `build-plan.md` T-004, T-005, T-007, T-009 and T-012 (Intent, Depends on, Touches, Notes); the Work evidence of INT-0005, INT-0035 and INT-0014
- **Quote:** "T-003, T-004, T-005, T-009 and T-010 each correct the sentences of `docs/spec/working-copy.md` that they make false."
- **Failure mode:** hidden-dep
- **Why it matters:** T-004 names INT-0005, T-009 names INT-0035 and T-012 names INT-0014, and none of those chapters' Work evidence names the task. T-005 declares only T-003, though its negative control runs "at T-004's commit" and T-004's literals clause holds only if T-005 comes after it; and T-007 makes the spec's two `praps fsck` sentences false without touching the spec.
- **Suggested response:** fix-in-plan.

### C-010: The deferral of X-02 covers half of it
- **Where:** `build-plan.md` "Deferred, with reasons"; `test-plan.md` Integration Tests, "The suite on both platforms"; `crates/book/src/main.rs`; `crates/xtask/src/ci.rs`
- **Quote:** "the test evidence records `cargo book`'s own last line, with its count of chapters and criteria"
- **Failure mode:** missing-risk
- **Why it matters:** X-02's other half is that a mistyped marker selects the lenient port rule unseen. The rule is on `cargo book`'s first line ("typed-acceptance 1, strict" or "undeclared, port rule"), not its last, so the record as planned reads the same under either rule. The test plan also takes the line from the `book` step of `cargo ci`, which prints only `PASS book`.
- **Suggested response:** fix-in-plan. Record the first line too, and take both from `cargo book` run by itself.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---|---|
| C-001 | **fix-in-plan.** T-008's clauses are scoped: "a command that opens a working copy" is named, `help` and `version` look at no directory (a test pins that they need no repository), and `init` and `verify` each have a clause of their own. (a) `init` has two cases: with a top above the directory it writes nothing, whatever the directory holds, be it nothing, a half-made store or a whole repository; otherwise it does as today, which includes completing a store that has no operation. The tests cover a half-made store and a whole repository inside a working copy, and both interrupted states outside one. (b) `verify`'s first rule now uses the same test as "a top", so a directory holding `.praps` with no store falls through to the working copy above; the build plan notes the one case this changes, and a test covers it. (c) `verify` on a missing directory fails naming it, before the four rules. |
| C-002 | **fix-in-plan.** T-009 has a clause that `log` records the working copy first and reports what the snapshot left alone, and T-010's first clause says the same of `describe`. `changes_follow_first_parents_newest_first` has an edit not yet recorded; `snapshotting_commands_return_reports` and `praps_commands_report_skipped_names` gain `changes` (the new `log`) in T-009 and `describe` in T-010, and keep `ops`. |
| C-003 | **fix-in-plan.** `crates/workspace/tests/workspace.rs` is in the Touches of T-009 and T-010. Each has a clause that its command reads the op log as T-003's first clause says, and `open_and_one_command_read_the_op_log_once` gains `changes` in T-009 and `describe` in T-010. |
| C-004 | **fix-in-plan.** T-003 has a clause for the case INT-0014 records, with `an_operation_written_after_open_leaves_two_heads`: the first command succeeds, both operations are in the log, the next command stops with `SeveralHeads`. The diff clause now says only that no existing assertion changes, and the Notes say this is the one behaviour the task changes. |
| C-005 | **fix-in-plan.** A new task, T-012: a write leaves a directory it emptied and cannot remove, with a note, unless the target has a file at that path, where it stops as today. Its end-to-end test runs `undo` from a directory another process stands in, on both systems, and records what each does. T-008 has a clause for a current directory that cannot be read, with a Linux test, and T-011 lists it among the refusals that need a hint. INT-0014 records the rule; whether a write should spare the directory a command was started in is left to backlog T-031. |
| C-006 | **fix-in-plan.** `find_stops_on_a_failure_that_is_not_absence` wraps the simulator in a test filesystem that refuses one listing and records every call, so it can show that nothing further up was asked. `a_praps_entry_without_a_store_is_ordinary` runs through the binary with a file named `.praps`, which on Linux is the "not a directory" case. T-002's clause now says "the hex form of a string of bytes", and its review lists every place the two-digit format remains, with what each is. |
| C-007 | **fix-in-plan.** The clause gives the rule: every criterion whose `Signers` detail names the project owner, and INT-0004 AC-5, each with its chapter's state, found by a search the record gives. The Notes list the 17 the search finds at Plan and the eight T-035 carries. The review repeats the search. |
| C-008 | **fix-in-plan.** INT-0022 defines the class by the repeat alone: run twice in a row, the second run changes nothing and prints the same. It says the `praps` commands in it are those that report, that a first run may write (every command records the working copy first), and names `cargo reach` as a case of the same rule. INT-0037 words its flag rule to match. The plans say "idempotent" throughout. |
| C-009 | **fix-in-plan.** The Work evidence of INT-0005 names T-004 and T-012, of INT-0035 T-009, and of INT-0014 T-012 and T-013. T-005 depends on T-004. T-007 touches the README and both specs, with a clause and a test, `docs_name_only_commands_praps_has`, which lands there. |
| C-010 | **fix-in-plan.** The record takes the first and last lines of `cargo book` run by itself, on both platforms, and the first must name the strict rule. The deferral says what each line shows. |
