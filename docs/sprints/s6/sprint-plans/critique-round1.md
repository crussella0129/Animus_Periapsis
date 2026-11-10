# Plan Critique — Sprint 6

Round 1. A read-only subagent reviewer critiqued the plans at `8f047b2` against the Book and the code. Its concerns are given as it wrote them, with file paths made relative to the repository; the responses are at the end. The verdict was `block`, so the plans were revised and reviewed again ([critique.md](critique.md)).

## Concerns

### C-001: T-003's state kept from `open` contradicts the sentence INT-0014 gained for this sprint
- **Where:** `build-plan.md` T-003, clause 1 and Notes; `docs/intents/INT-0014-working-copy-and-workspaces.md`, Consequences
- **Quote:** Plan: "Between `open` and the first command, an operation written by another handle is not seen by that command." Chapter: "Each public command of the `workspace` crate still starts from the log as it is on disk, so it sees what another process wrote before it began."
- **Failure mode:** intent-drift
- **Why it matters:** Clause 1 requires `ops/` to be listed exactly once from `open` through the first command, so that command cannot start from the log on disk; a handle held open while another writer finishes now forks the log, where today the command reads again. The same task makes `open` refuse a damaged log before it resumes, while the chapter still says every command "resumes an interrupted write before anything else", and the reason for the new order (today `resume` drops the record when its target op file is damaged) exists only in the plan.
- **Suggested response:** fix-in-plan. Correct both sentences of INT-0014 and the spec's Resume section, or drop the state at the end of `open` and change clause 1 to one listing in `open` and one in the command.

### C-002: `undo_and_restore_scan_once` cannot pass on a correct build, and the one-scan change is not linked to INT-0009
- **Where:** `test-plan.md` T-003 unit tests; `build-plan.md` T-003 clause 6 and Intent line; `crates/workspace/src/materialise.rs` (`Disk::listing`; the hazard loop)
- **Quote:** "in the trace of `undo`, and of `restore`, the working copy's top directory is scanned once."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `materialise::plan` reads directories with the same `scan` syscall as the snapshot and always reads the top when the diff holds a path, so an `undo` that moves a file shows `scan repo` twice after T-003 (three times today). T-003 changes `undo` and `restore` and claims INT-0009 AC-2 and AC-3, yet its Intent line omits INT-0009, the chapter's Work evidence names only T-006 (as INT-0005's omits T-002), and the argument that one scan keeps AC-2 is only in the plan's Notes.
- **Suggested response:** fix-in-plan. Count something only the snapshot does, such as scans of a directory outside the diff; add the intent links; record the one-scan rule in INT-0009 or the spec.

### C-003: One new E2E test is traced to a task that cannot write it, and the other to no clause
- **Where:** `test-plan.md` Intent Traceability (first T-005 row) and End-to-End Tests; `build-plan.md` T-005 Touches
- **Quote:** "the same session with every command after `init` run from a directory two levels down."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** That session is `praps_first_session_end_to_end`, which runs `describe`, `log` for changes, `ops` and `verify`, all built in T-006, so `praps_from_a_subdirectory_end_to_end` cannot exist at T-005's commit. `praps_first_session_end_to_end` is in no traceability row and names no task or clause, and the "Cost before and after" record traces to no clause either.
- **Suggested response:** fix-in-plan. Move both E2E tests to T-006, or give T-005 a session of commands that exist then; add a row for the first-session test; say that the cost record is evidence without a clause.

### C-004: T-006 changes the `praps log` layout that INT-0022 AC-4 to AC-7 judge, and sets them aside in a note
- **Where:** `build-plan.md` T-006 Notes; `docs/intents/INT-0022-agent-and-human-interfaces.md` AC-4 to AC-7
- **Quote:** "None is claimed: lines are not fitted to the terminal, and no panel has judged the layout."
- **Failure mode:** intent-drift
- **Why it matters:** AC-4 is `necessary` and needs no panel, and the new listing prints a description's first line at any length, so the sprint builds a `log` it knows fails a necessary criterion of a chapter it moves to `planned`. AC-5 and AC-6 ask for the sampling seed and the rubric to be "registered before the layout change", which cannot be done once T-006 lands, and the chapter records neither waiver.
- **Suggested response:** fix-in-plan. Record in INT-0022 that Sprint 6's listing is provisional and when the seed and rubric are registered, or build a renderer that takes a width and test AC-4 on it now.

### C-005: INT-0003 AC-11 is claimed from one sample
- **Where:** `test-plan.md` Intent Traceability (last row) and E2E `no_job_exceeds_twenty_minutes`; `build-plan.md` Intents
- **Quote:** "from `gh api …/jobs` for that run, each job's own start to end is at most 20 minutes"
- **Failure mode:** intent-drift
- **Why it matters:** AC-11's threshold is "3 of 3 sampled runs of one commit", with each sample's cache state recorded; Sprint 5's plan had `sampled_runs_meet_the_budget` for it, and this plan samples once in the sprint that adds a crash matrix and records two again. The GitHub actions the owner approved (one dispatch, one pull request) do not include the two re-runs.
- **Suggested response:** fix-in-plan. Restore the three samples and ask for the re-runs, or stop citing AC-11 for this sprint's pull request.

### C-006: The registry's "idempotent" class is not the one INT-0022 defines
- **Where:** `build-plan.md` T-004 Notes, clauses 6 and 7; `docs/intents/INT-0022-agent-and-human-interfaces.md` Intent, AC-8, AC-9
- **Quote:** "`init` and `describe` are safe to repeat but print something else the second time, so by INT-0022 AC-9 they are not in that class."
- **Failure mode:** intent-drift
- **Why it matters:** The chapter calls a command idempotent "when running it again changes nothing", which `init` and `describe TEXT` both satisfy, and INT-0037 words the flag rule the same way. The plan turns AC-9, a criterion on the class, into its definition, so AC-8's test never looks at two commands the chapters cover, and the reason is only in the plan.
- **Suggested response:** fix-in-plan. Define the registry's class in INT-0022 and name `init` and `describe` as outside it, or keep the chapter's definition and make both repeat exactly.

### C-007: `idempotent_commands_repeat_exactly` has a fixed list that T-006 never updates
- **Where:** `test-plan.md` T-004 unit tests (the group run through the binary) and T-006 unit tests
- **Quote:** "for each command the list marks so (`status` after an edit, `log`, `fsck`, `help`, `version`)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `praps` is a binary crate, so a test that runs the binary cannot read the registry's mark, and neither the list nor `help` is required to print it. After T-006 `fsck` is an unknown word that prints the same refusal twice, and `ops` and `verify` are never run twice, so INT-0022 AC-9's "100% of idempotent commands" is not measured.
- **Suggested response:** fix-in-plan. Drive the test from the registry, and have T-006 say which names join and leave, as it does for `commands_act_the_same_from_below`.

### C-008: T-005's clauses leave three cases undecided or in conflict
- **Where:** `build-plan.md` T-005 clauses 1, 2, 5, 6 and Notes; `crates/workspace/tests/workspace.rs` (`praps_and_git_dirs_are_always_ignored`); `crates/praps/src/main.rs` (`store_dir`)
- **Quote:** "A directory holds `.praps` when listing `DIR/.praps` succeeds."
- **Failure mode:** EARS-vague
- **Why it matters:** A `.praps` directory below the top is an ordinary tracked name today, so by this rule commands run beneath it bind to a directory that is not a repository, and the plan does not say whether `init` there is refused or completes a nested repository, which INT-0037 AC-3 forbids. Clause 6 and clause 1 disagree for a working copy whose top has directories named `objects` and `ops` (today `.praps` wins, and no planned test pins it), and clause 1 demands the same stderr as a run "at the top with no directory given" while clause 2 names the top differently when the top is the directory given.
- **Suggested response:** fix-in-plan. Define the probe and the precedence, except clause 2's naming from clause 1, and add a test for each case.

### C-009: Statements about existing tests that the code contradicts
- **Where:** `test-plan.md` header, T-003, T-005 and T-006 unit tests, Intent Traceability; `build-plan.md` T-003 clause 3; `crates/praps/tests/cli.rs`; `crates/workspace/tests/workspace.rs`; `crates/xtask/src/docs.rs`
- **Quote:** "A test marked **renamed** keeps its assertions under a name that fits the new vocabulary."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `praps_ops_lists_operations_newest_first` cannot keep `lines[0].ends_with(" 2 files")`: by T-006's count the head operation after `new` changes 0 paths. Four more: `open_without_ops_fails` is cited for the `open_with_log` clause but fails at the `.praps/store` check and never calls the store; `praps_status_outside_a_repository_fails` is called unchanged although its `status nowhere` case falls under the new missing-directory clause, which T-007 lists as a separate kind of error; clause 3 says a later commit by another handle stops the next command, which is true only of a fork; and the traceability row adds "three names" to the credits list, where Mercurial already is.
- **Suggested response:** fix-in-plan. List the assertions each renamed or "unchanged" test loses, and correct the other statements.

### C-010: Touches lists miss files the test plan edits, and T-006 leaves a retired command in an error message
- **Where:** `build-plan.md` T-002 and T-006 Touches; `test-plan.md` T-002 and T-006 unit tests; `crates/workspace/src/lib.rs` (`refuse_invalid_ops`); `crates/workspace/tests/workspace.rs`; `crates/sim/tests/points.rs`
- **Quote:** "In `crates/workspace/tests/materialise.rs`: `workspace_points_mark_each_stage` gains `workspace.describe.committed`."
- **Failure mode:** hidden-dep
- **Why it matters:** That file is not in T-006's Touches, and the `sim` crate's `points.rs` test file, where `point_has_no_default_body` goes, is not in T-002's. From T-006 to T-007 the `workspace` crate still prints "run `praps fsck`", a word `praps` then refuses, because the assertion that pins the text is touched only in T-007.
- **Suggested response:** fix-in-plan. Add the two files; let T-006 change the message and its assertion, or note the gap.

### C-011: T-006 is four outcomes in one commit, and T-003 moves behaviour under the same new literals as its refactor
- **Where:** `build-plan.md` T-006 (ten clauses, four intents, 13 paths); T-003 clauses 5 and 6
- **Quote:** "`describe`, `log` for changes, `ops` for operations, and `verify`"
- **Failure mode:** granularity
- **Why it matters:** The rename to `verify`, the `log` and `ops` split with the linear listing, and `describe` with its matrix, point and reach row have separate outcomes and separate ways to fail; only the last needs the single commit the Notes justify. In T-003 the new resume order and the single scan change behaviour under the same re-recorded literals as the cost change, which gives up the isolation T-002 claims for itself.
- **Suggested response:** fix-in-plan. Split T-006 into three tasks and update the chapters' Work evidence anchors; split the scan change from T-003 or say why one re-recording is enough.

### C-012: Unit tests on `cli` run commands against the real filesystem in the package directory
- **Where:** `test-plan.md` T-004 unit tests ("In `crates/praps/src/main.rs`, on `cli` with in-memory writers"); `crates/praps/src/main.rs` (`init`)
- **Quote:** "a word is answered as a command exactly when the registry holds it."
- **Failure mode:** hidden-dep
- **Why it matters:** In-memory writers capture output only: `cli` still uses `RealFs` and the test process's current directory, the `praps` package directory, so answering `init` as a command writes `.praps` into the checkout, and after T-005 every other word searches the checkout's ancestors for a repository. `no_repository_above_the_test_directories` guards the temporary directories, not this one.
- **Suggested response:** fix-in-plan. Say that these tests stop before any command body (one argument too many, as `usage_lines_come_from_the_registry` does), or test the dispatch table itself.

### C-013: Backlog tasks the revised chapters cite do not exist until T-008, and T-012 never gains what two chapters say it lists
- **Where:** `build-plan.md` T-001 (clauses 4 and 5) and T-008 (last clause); `docs/work/tasks.md`
- **Quote:** "the record **SHALL** say it is pending, and a backlog task **SHALL** carry it forward."
- **Failure mode:** hidden-dep
- **Why it matters:** INT-0005, INT-0010, INT-0013 and INT-0035 and the test plan's "Not yet possible" list already cite T-029 to T-032, which the last task creates, while T-001, the first, must add a backlog task and the next free number is T-029. INT-0006 ("the fields backlog T-012 lists") and INT-0016 point to content T-012 does not hold, and no clause changes T-012.
- **Suggested response:** fix-in-plan. Create T-029 to T-034 before the plans lock, name the id of T-001's task, and give T-012's new text to a task.

### C-014: Research items with no task and no stated deferral
- **Where:** `research-report.md` sections 4 and 5; `book-audit.md`, "Stale statements found in passing"; `build-plan.md` T-001 and T-008
- **Quote:** "close two holes in the checks this sprint leans on (X-02, X-06)"
- **Failure mode:** missing-risk
- **Why it matters:** X-02 and X-06 reappear only inside backlog T-033, with no reason given for moving them out of the sprint that leans on them. T-001 carries five old sign-offs and none of the new ones (INT-0035 AC-12, INT-0036 AC-7, INT-0037 AC-8, INT-0038 AC-7), though "the owner's decisions" is a named unknown and this sprint ships INT-0037's vocabulary; and the audit's stale sentence in the working-copy spec (only `new` and `status` snapshot) has no clause in T-008.
- **Suggested response:** fix-in-plan. Add the two fixes or state the deferral, list the new sign-offs in T-001's record and backlog task, and add the Snapshot sentence to T-008.

### C-015: AC-4's hints are checked for form only, and the `describe` hint misses its likeliest case
- **Where:** `build-plan.md` T-007 clauses 1, 3 and 4; `test-plan.md` T-007 unit tests; `crates/workspace/src/lib.rs` (`Error::Blocked`)
- **Quote:** "is one of: a command to run; something to put right outside `praps`; or no remedy yet"
- **Failure mode:** EARS-vague
- **Why it matters:** No test can fail on the second form, so INT-0037 AC-4's "100% of the error kinds" rests on nobody reading the hints, and `messages_carry_no_advice` passes on the `Blocked` message's "move them and try again". Clause 4 is tested with three words, a usage error; with two, `praps describe fix typo` is a missing-directory error whose hint is not tested, and `praps describe update docs` succeeds with the description "update".
- **Suggested response:** fix-in-plan. Add a review that records each error kind with its hint, test the two-word case, and say what happens when the last word names a directory.

### C-016: Clauses whose named tests pass on a wrong implementation
- **Where:** `build-plan.md` T-002 clause 2, T-003 Notes, T-006 clause 8, T-008 clause 1, with their rows in `test-plan.md`; `crates/workspace/src/lib.rs` (`refuse_invalid_ops`); `crates/sim/src/real.rs` (`sync_dir`)
- **Quote:** "the workspace **SHALL** read a fixed number of objects for each and derive no tree for it"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `listing_cost_grows_in_a_straight_line` counts syscalls on a history of one file, so a listing that still derives a tree for each entry from changesets it has read passes; "each have one definition" has no test or review; `op_log_lists_once_and_counts_invalid_files` has no file whose name does not parse, which `refuse_invalid_ops` counts today; and `docs_name_only_commands_praps_has` matches `praps WORD`, so the README's crate rows can keep `fsck`. The note that the layout syncs "show that `objects/` does" exist holds only in the simulator: on a real filesystem `sync_dir` succeeds on a regular file.
- **Suggested response:** fix-in-plan. Add a case or a review for each of the four, and correct the note.

### C-017: INT-0035 puts the merge before further single-user features, and T-004 to T-007 are such features
- **Where:** `docs/intents/INT-0035-scale-and-asynchrony.md` Consequences; `build-plan.md` Schema Tree, "Use"
- **Quote:** "The canonical merge of INT-0010 comes before further single-user features: until it exists, nothing works with two agents."
- **Failure mode:** intent-drift
- **Why it matters:** This plan links the chapter and moves it to `planned`, yet four of its tasks build single-user features first. The owner's approval of that order is in the build plan, not in the chapter a later sprint will read.
- **Suggested response:** fix-in-plan. Reword the Consequence to name Sprint 6's command-set work as the approved exception.

## Confidence
block

## Responses

Task numbers below are those of the plan as revised after this round; round 2 then added one task, so its documentation task, T-012 here, is T-013 in the locked plan. The eight tasks became twelve: the old T-003 is now T-003 to T-005, the old T-004 is T-006, the old T-005 is T-008, the old T-006 is T-007, T-009 and T-010, the old T-007 is T-011 and the old T-008 is T-012.

| Concern | Response |
|---|---|
| C-001 | **fix-in-plan.** The state from `open` is kept for the first command, and INT-0014 now says so: a command starts from the log on disk, except the first after opening, which uses opening's reading; an operation written in between leaves two heads, as any overlap does. INT-0014 also records the new resume order and its reason. The order itself is a task of its own, T-004, with a test that fails before it. `Store::open_with_log` is dropped, so `Store::open` is unchanged and still lists `ops/` to see that it exists: T-003's clause now says "exactly twice", once for that and once to read the log. |
| C-002 | **fix-in-plan.** The scan change is T-005, linked to INT-0009, whose Consequences now record the one-scan rule and why it keeps AC-2. Its test, `undo_and_restore_read_an_untouched_file_once`, counts reads of a tracked file outside the diff, which only the snapshot makes, and is run first at T-004's commit, where the file is read twice. INT-0009's Work evidence names T-005, T-009 and T-010, and INT-0005's gains T-002 and T-005. |
| C-003 | **fix-in-plan.** T-008 has `praps_end_to_end_from_below`, the session of the existing `praps_history_end_to_end` from two levels down, which uses only commands that exist then. The two first-session tests are T-010's, the task after which every command exists, under a clause and a traceability row of their own. The cost record says it is evidence without a clause and cannot fail the sprint. |
| C-004 | **fix-in-plan.** INT-0022 records that Sprint 6's `log` listing is provisional, that it does not meet AC-4, that nothing is claimed for AC-5 to AC-7, and that the seed and the rubric are registered before the first designed layout, with the provisional listing as what the panel compares it with. A width-aware renderer needs the terminal's width and the display width of text, each a new dependency under this project's rules, so it is not built in a sprint whose listing the owner asked for by name. |
| C-005 | **fix-in-plan.** AC-11 is no longer cited for this sprint's pull request. One run's job times are recorded as one sample; the three samples are listed as not yet possible, since the two re-runs were not among the actions the owner approved; backlog T-035 carries them. A job over 20 minutes is a finding. |
| C-006 | **fix-in-plan.** INT-0022 now defines the idempotent class as the commands that only report, names `init` and `describe` as outside it and says why, and notes that recording the working copy first does not move a command out of the class. INT-0037 words its flag rule the same way. AC-8 and AC-9 are unchanged. |
| C-007 | **fix-in-plan.** `praps help WORD` prints whether a command only reports (a new clause of T-006), and `reporting_commands_repeat_exactly` builds its list from that output, so `ops` and `verify` join as they land and `fsck` leaves. The test fails if it finds fewer than five such commands. |
| C-008 | **fix-in-plan.** T-008 defines the probe: a directory is a top when listing its `.praps/store` succeeds. A `.praps` directory with no store is ordinary, as today, and commands beneath it act on the working copy above (two tests pin it); INT-0014 records it, and what a nested repository should mean is backlog T-031. `init` has three cases in order, and inside a working copy it never makes or completes a repository. `verify` has four in order, with `.praps` before a bare store before the working copy above, and one test case for each. The equality clause now compares a run at the top with no directory against runs from below, and the naming clause covers the case where the top is the directory given. |
| C-009 | **fix-in-plan.** The test plan's header says a renamed test keeps its assertions except those listed, and lists them: the count assertions of both operation-list tests, with the new expected endings; the `status nowhere` case and the changed text of `praps_status_outside_a_repository_fails`; the cases that leave `usage_errors_exit_two`; the advice assertions two workspace tests lose. T-003's clause on another handle now has three outcomes, with a new test for the case that does not stop. The credits list gains two names. The `open_with_log` clause is gone. |
| C-010 | **fix-in-plan.** `crates/sim/tests/points.rs` is in T-002's Touches and `crates/workspace/tests/materialise.rs` in T-010's. T-007, the rename, changes the `workspace` message from `praps fsck` to `praps verify` with its assertion, in the same commit. |
| C-011 | **fix-in-plan.** The old T-006 is three tasks: T-007 (`verify`), T-009 (`log` and `ops`, with the listing's cost) and T-010 (`describe`, with its matrix, point and reach row). The old T-003 is three: T-003 moves cost only, and a review of its diff checks that no existing assertion changed; T-004 changes the resume order, with the four matrices' literals required not to move; T-005 changes the scan, with Materialise recorded again and Snapshot required not to move. The chapters' Work evidence anchors are updated. |
| C-012 | **fix-in-plan.** Parsing is a function from arguments to what will be done, which touches no file. `usage_lines_come_from_the_registry` and `only_registry_words_dispatch` test that function, so no command body runs in a unit test. |
| C-013 | **fix-in-plan.** T-029 to T-034 are in `docs/work/tasks.md` now, written during Plan. T-001 names its backlog task, T-035. T-012's text gains the operation's scope and the owner of each view entry, with the two chapters among its intents. |
| C-014 | **fix-in-plan and defer-with-rationale.** The build plan has a section of deferrals with reasons. X-02 and X-06 stay in T-033: the owner's approved outline keeps the tooling findings together, and this sprint's use of the two checks is covered by recording `cargo book`'s own count line on both platforms and by running `cargo policy` and `cargo book` directly, where they print every problem. T-001's record and T-035 list the new sign-offs, INT-0016 AC-8 among them. T-012's spec clause names the Snapshot sentence. |
| C-015 | **fix-in-plan.** Each `workspace` message is fixed by exact text in `messages_say_only_what_is_wrong`, so the advice in `Blocked` cannot stay unseen. The hint table is printed by a test, recorded, and read by the code review for whether each hint can be acted on. `describe` is tested with three words, with two where the second is no directory, and with two where it is one: that last case succeeds with a one-word description by INT-0037's argument rule, the line `describe` prints shows the text taken, and a test pins it. INT-0037 now says text of several words is one quoted argument. |
| C-016 | **fix-in-plan.** The listing clause no longer claims what its test cannot see: it requires the syscalls to grow in a straight line, with a recorded failure at the earlier commit. "One definition" has a recorded search. The `op_log` test has a file whose name is not an id. The docs test also refuses the word `fsck` outside `Store::fsck` and file names. The note about layout syncs went with `open_with_log`. |
| C-017 | **fix-in-plan.** INT-0035's Consequence names Sprint 6's command-set work as the exception the owner approved on 2026-11-10, with the reasons: it is one of the three aims, and the merge waits on the owner's decision about who owns which view entries. |
