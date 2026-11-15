# Test Critique — Sprint 7, round 1

Round 1. A read-only reviewer, run as a subagent, critiqued the Test evidence at `c3ad8b8` (tested head `5b65da8`) against the locked plans, the completion entries and the linked intents, using the Sprint Loops test-critic prompt. Its concerns are given as it wrote them; the responses are at the end. The verdict was `block`, so the evidence was revised and reviewed again ([critique-round2.md](critique-round2.md), then [critique.md](critique.md)).

## Concerns

### C-001: An unreadable `.praps/workspace` breaks three EARS promises, and no test covers it
- **Where:** `build-plan.md`. T-006 says: "WHEN the file cannot be read or does not parse, THEN with another [error], which gives the reason". T-007's "top" clause says the file "SHALL stop the search with its error", and T-007's refusal list for `every_refusal_prints_one_hint` names "an unreadable `.praps/workspace`". The `unit-tests.md` rows for T-006 and T-007 give only `a_damaged_workspace_file_is_refused` and `a_damaged_workspace_file_stops_the_search`, and both write a file that reads but does not parse. `every_refusal_prints_one_hint` in `cli.rs` covers only "a .praps/workspace that does not parse". Related code: `Workspace::open` (`lane::read(&fs, &praps)?`), `place.rs` `is_top`, and `code-review.md` finding 6.
- **Quote:** "the search reports it as a `.praps/store` that cannot be listed; `open` reports a bare I/O error"
- **Failure mode:** negative-path
- **Why it matters:** When the file cannot be read, `open` returns `Error::Io` instead of `BadWorkspaceFile`. The search, which runs first in `praps`, reports the wrong file, and no executed test reaches this case. The results list the two "damaged" tests as the evidence for these clauses without a note, so the report could pass three SHALLs that are known to fail. Also, the `init` refusal added in `1d5c94b` is not in the `every_refusal_prints_one_hint` corpus (its count stays at 39/36). Its own test does not check that the hint comes last or that it names only registry commands.
- **Suggested response:** defer-with-rationale. In the report, mark T-006's sixth clause, T-007's top clause and T-007's refusal list as unmet for an unreadable file (backlog T-036), and do not claim INT-0037 AC-4 at 100%. Add the new `init` refusal to `every_refusal_prints_one_hint`.

### C-002: The AC-1 property test cannot fail for the library's merge
- **Where:** `crates/workspace/tests/merge.rs`: the `library` adapter (lines 131–138), `check_ac1` and `merge_ignores_order_and_repeats`. The `unit-tests.md` T-003 row cites it as "AC-1, 2,000 histories". INT-0010 AC-1 is `necessary` `formal`.
- **Quote:** "The library's merge. It takes a set, so an order or a repeat reaches it as the one set a caller collects."
- **Failure mode:** weak-assertion
- **Why it matters:** Every shuffled and repeated order is passed to `merge::merge` through `set(heads)`. Each comparison is therefore the same call on the same `BTreeSet`, and it can fail only through nondeterminism. Only the test's own slice-consuming mutants (pairwise fold, fast path) could ever fail it, yet it is cited as the evidence for a formal criterion.
- **Suggested response:** tighten-assertion. State in the report that AC-1 holds by construction: the merge takes a set of heads, `writers` walks only from the heads, and every collection is ordered. Cite `merge_ignores_order_and_repeats` as a determinism check, and `wrong_merges_are_rejected` as the evidence that the property check can fail.

### C-003: At the tested head, no executed test shows the Concurrency `lost` check can fail
- **Where:** `concurrency.rs`: `control()`, `concurrency_catches_*` and `check_store`. `test-matrix-log.md`: all three control lines show `lost: 0`. `integration-tests.md`: "302, 180 and 320 … measured while T-009 was built on Windows".
- **Quote:** "A control needs one such cell; running every cell costs as much as the scenario itself (the counts over all 320 are in Sprint 7's T-009 record)."
- **Failure mode:** negative-path
- **Why it matters:**
  - `lost` checks that no acknowledged op is missing, which is the core of INT-0010 AC-5 and INT-0014 AC-3. Because each control stops at the first cell that breaks its own check, `lost` fires in no executed test at `5b65da8`. Its 3,632 hits come from a Windows-only sweep during T-009's build.
  - With several heads, `held` is read from the very op that `merge::writers` named, so the comparison `held != kept` cannot fire once the `writer` check passes.
  - The controls were cut to save CI time, which sits against INT-0003's "never from shrinking a matrix".
- **Suggested response:** tighten-assertion. Make one control stop on `lost > 0`. Compare the last acknowledged entry with what each lane loads, not with `merge::merge`'s output. Record one full control sweep at the tested head on both systems.

### C-004: INT-0009 AC-6's 200 interleavings never have two op heads
- **Where:** `scoped.rs` `undo_is_scoped_over_200_interleavings`. `unit-tests.md` reports "INT-0009 AC-6: 2,363 commands".
- **Quote:** "// Each command opens its lane, as a `praps` process does."
- **Failure mode:** intent-coverage
- **Why it matters:** Each step opens a fresh handle and finishes before the next starts, so the op log stays linear. "The merge of op heads" that the criterion names is therefore always one head's view, and no `undo` or `restore` in the sample runs alongside a concurrent op of the other lane. Also, the 2,363 count includes `NothingToUndo` and `NotInWorkspace` refusals.
- **Suggested response:** add-test. In a seeded share of steps, act from a handle opened before the other lane's last command (or run both lanes under `sim::Processes`). Count successful undo and restore separately from refusals.

### C-005: Resume's "stands beside it" branch has no test
- **Where:** `build-plan.md` T-004, the resume clause. `materialise.rs` tests only an op of the same workspace that follows the target (`resume_leaves_files_when_its_workspace_moved_on`, `resume_drops_a_record_whose_target_is_not_the_head`) and another workspace's op beside it (`resume_finishes_after_another_workspace_commits`). The code is `lib.rs` `resume`, which checks `writers… == Some(&BTreeSet::from([record.target]))`.
- **Quote:** "WHEN another operation of that workspace has followed it or stands beside it, THEN resume SHALL remove the write's temporary names, drop the record, leave every other file and add a note"
- **Failure mode:** negative-path
- **Why it matters:** The race rule creates exactly this case: an op of the same workspace made beside the interrupted one. If the exact-set check were weakened to "the target is still a writer", resume would finish a write over a concurrent change and every test would still pass.
- **Suggested response:** add-test. Interrupt a write whose op has a sibling in the same workspace over the same head, and check that resume abandons it with the note, drops the record, removes the temporary names and leaves the files.

### C-006: The real-process tests never show that commands actually overlapped
- **Where:** `cli.rs` `eight_new_at_once_all_succeed` (children spawned in a loop with no barrier) and `praps_notes_a_raced_working_copy` (the race is written by hand through the store). `e2e.rs` `lanes_at_once_end_to_end`. `e2e-tests.md`: "It is research's measurement, made a test."
- **Quote:** "The interleaving is the system's; what is asserted holds under any."
- **Failure mode:** weak-assertion
- **Why it matters:** Every assertion would also pass in a fully serial run. So neither the "3 of 8, then 8 of 8" before-and-after nor INT-0014 AC-3 through the binary is shown to have produced several op heads. The only test of real concurrent `new` does not look for the race note, and the binary race-note test fakes its race with hand-built ops.
- **Suggested response:** tighten-assertion. Record in the output and the report how many ops have several parents after each run. Have `eight_new` check that `status` prints the note naming every change in a raced pointer whenever there is one.

### C-007: Parts of T-007's clauses have no test that could fail
- **Where:** `build-plan.md` T-007. `cli.rs`: `praps_lane_refusals`, `idempotent_commands_repeat_exactly`, `praps_lanes_lists_every_workspace`, `praps_ops_names_each_workspace`, `commands_act_on_the_lane_they_run_in`, `commands_act_the_same_from_below`.
- **Quote:** "`lanes` SHALL be marked idempotent and declare no flag"
- **Failure mode:** EARS-coverage
- **Why it matters:** No test would fail if any of these broke:
  - `lanes` lost its idempotent mark: `idempotent_commands_repeat_exactly` finds the set from help and asserts only that there are at least 5 and that `status` is among them.
  - `lanes` stopped recording the working copy: no test makes an edit before running it.
  - A NAME of `.`, `..` or non-UTF-8 were accepted: only `/` and a line break are tried.
  - `ops` showed a raced pointer's larger change: no test has a raced pointer there.
  - Commands inside a lane behaved differently from below its top: INT-0037 AC-3 is never compared byte for byte inside a lane, and `new`, `undo`, `restore` and `verify` there are checked by exit code only.
- **Suggested response:** add-test. Pin `lanes` in the idempotent set. Make an edit before `lanes` and check its path count and the op it records. Add `.`, `..` (and non-UTF-8 on Unix) to the refusal cases. Add a raced pointer to the `ops` test. Run the `commands_act_the_same_from_below` corpus inside a lane.

### C-008: The lane crash matrix accepts "no repository" in every cell where the op did not land
- **Where:** `lane_crash.rs` `check()`. `build-plan.md` T-006, the crash clause and the clause on opening an interrupted lane.
- **Quote:** "`(false, Err(Error::UnfinishedLane(..) | Error::NotInitialised(_))) => {}`"
- **Failure mode:** weak-assertion
- **Why it matters:** The clause allows "no repository" only "before `.praps/workspace` is in place", but the check accepts it whatever the recovered file says. In that branch it also does not check that the record was dropped, that nothing else was written, or which lane the error names. A regression that answered a finished lane file with the `praps init` hint would therefore pass all 5,760 cells. The "make it again" check also re-makes only at the same directory, so code-review finding 2's path (re-making at another directory) is outside every check.
- **Suggested response:** tighten-assertion. Accept `NotInitialised` only where `lane/.praps/workspace` is absent after recovery. In the not-landed branch, assert that the record is gone, the rest of the filesystem is unchanged, and `UnfinishedLane` names this lane and directory.

### C-009: The Concurrency checks are narrower than INT-0014 AC-3 and INT-0010 AC-5
- **Where:** `concurrency.rs` `check()`: a lane's `.praps` is checked only for an entry named `store`, and `exchange` runs only after recovery. `lanes.rs` `eight_lanes_share_one_store`. `test-plan.md`: "among them one file per object".
- **Quote:** "// Each object is one file: its name is its id."
- **Failure mode:** weak-assertion
- **Why it matters:**
  - That check compares a directory listing with its own de-duplicated copy, which no filesystem can fail. The concurrent run rejects only a lane `.praps` entry named `store`, so "without duplicated objects" has no check under concurrency that could fail.
  - Each lane runs on one store, and the stores exchange once after recovery. No op is ever made on top of ops received from the other store, so the two stores' equal views follow simply from holding equal sets of ops.
- **Suggested response:** tighten-assertion. After recovery and after the next command, assert that each lane's `.praps` holds only `workspace` (plus at most a record) and that no object lies outside the stores. State the single-exchange design in the report as a limit of AC-5, next to INT-0024 (defer-with-rationale for that half).

### C-010: Per-platform digests remove the Windows/Linux comparison, and "nothing else differs" is not shown
- **Where:** the `#[cfg(windows)]` digest literals in `lane_crash.rs` and `concurrency.rs`. `integration-tests.md`, `test-matrix-log.md`.
- **Quote:** "Nothing else differs: the cell counts and every violation count are equal (see the matrix log)."
- **Failure mode:** evidence-drift
- **Why it matters:** The stated cause (the store path's separator) is credible, but the evidence given is counts, not file contents. With one digest per platform, any other Windows/Linux difference in the ops, views or files of the only scenarios that merge heads and make lanes is now hidden, which goes against INT-0004's "same results" Intent.
- **Suggested response:** tighten-assertion. Add a second per-cell digest with the `store` line in `.praps/workspace` normalised (or the file left out), and assert it as one literal on both systems.

### C-011: The scheduler speed-up and the five-minute guards rest on unmeasured claims
- **Where:** the header of `sim/tests/thousand.rs`; `GUARD` in `processes.rs`; `integration-tests.md`, "GitHub's Windows runner and the scheduler tests"; `unit-tests.md` `scheduler_at_a_thousand_processes`; `e2e-tests.md`, which gives no per-unit times for the three GitHub samples.
- **Quote:** "its 3,001 handoffs between threads take tens of seconds on GitHub's Windows runners under the suite's load"
- **Failure mode:** flake-risk
- **Why it matters:**
  - No test asserts that a boundary wakes only the chosen process, so a return to waking every process would fail nothing in CI.
  - The speed-up's only evidence is one local run (6.54 s, then 0.13 s), while the GitHub record implies tens of seconds for the same run.
  - The five-minute guard is sized from one 77 s run of the binary, on a diagnosis ("slow, not stuck") that no GitHub timing at `5b65da8` confirms.
- **Suggested response:** add-test. Count wakes in `sim::Processes` and assert at most one per boundary, except when the run is aborting. Record the `processes`, `thousand` and `concurrency` unit times from the three GitHub samples, next to the guard and the job's time limit.

### C-012: Records the report must not present as met
- **Where:** the T-002 entry (sprint 7) in `completed-tasks.md`. The `build-plan.md` Intents section lists "AC-6, the owner asked to sign off the change (T-002)". `unit-tests.md` gives `format_sign_off_is_the_owners_answer: not yet`. INT-0003 AC-11. In `e2e-tests.md`, the GitHub table and the INT-0037 AC-6 row under "Not yet possible".
- **Quote:** "The owner is asked to sign off the specification change at the end of the sprint, before the merge (sign-offs.md)."
- **Failure mode:** evidence-drift
- **Why it matters:**
  - The "not yet" is honest, but T-002 is recorded complete although its last SHALL (asking the owner) has not happened at the tested head.
  - INT-0003 AC-11 asks for the wait for a runner to be "reported separately", and the table leaves it out.
  - The INT-0037 AC-6 row names no backlog item as what unlocks it.
- **Suggested response:** tighten-assertion (of the records). List the T-002 sign-off clause and INT-0006 AC-6 as open, ask the question against `docs/spec/format-v1.md` at `609a8df`, and make the merge wait for the owner's own words. Add each sample's queue time, and name a backlog item for INT-0037 AC-6.

## Confidence
block

## Responses

The tests below are commit `b951d6d`, which is now the tested head. `cargo ci` passed there on both systems: 663 tests on Windows and 666 in WSL. Where a response says that a test fails under some change to the code, that change was made once, the test run, and the source put back.

- **C-001: add-test, and the code fixed.** An unreadable `.praps/workspace` is now refused as that file, with the system's reason, in all three places.
  - `open` reads the file through one function, which turns a failed read into `BadWorkspaceFile`, as for a file that does not parse. Only a crash of the simulated system stays an I/O error.
  - The search for a top says which file could not be read, the store or the lane file. `praps` prints that file's path and reason, and for the lane file the hint "make that file readable, then run the command again".
  - Tests: `an_unreadable_workspace_file_is_refused` in `lanes.rs`, and two new cases in `every_refusal_prints_one_hint`: an unreadable `.praps/workspace` (a directory in its place, so on both systems) and `init` refused there. The test now counts 41 refusals on Linux and 38 on Windows, and checks for each that the hint comes last and names only registry commands.
  - Without the fix in `open`, the first test fails. With the search reporting the store instead of the lane file, `every_refusal_prints_one_hint` fails.
  - So T-006's sixth clause, T-007's top clause and its list of refusals now hold for an unreadable file. Code-review finding 6 is fixed in full, and backlog T-036 no longer carries it.
  - **Corrected in round 2:** this fix covered a lane's top only. From below the top, and through `verify`, the lane file still met the wrong hint ([critique-round2.md](critique-round2.md), C-001).
- **C-002: tighten-assertion, by rewording.** `unit-tests.md` now says AC-1 holds by construction: the library's merge takes a set of heads, `writers` walks only from them, and every collection it builds is ordered. `merge_ignores_order_and_repeats` is cited as a check that the library is deterministic. The evidence that the property check can fail is its two mutants that take the heads as a list, the pairwise fold and the first-equals-last fast path, in `wrong_merges_are_rejected`.
- **C-003: tighten-assertion.**
  - A fourth control, `concurrency_catches_lost_ops`, runs the store's `UnsyncedObjects` and stops only at a cell where `lost` is above 0. Its one-store run breaks every cell but loses no acknowledged op; the first cell of its two-store run loses 29.
  - Where a lane's last acknowledged op is still its writer, the `kept` check now compares the entry the lane loads in a fresh open with that op's entry. It no longer compares with the merge's own output, so the comparison that could not fire is gone. Where an op in flight landed after it, the acknowledged entry must still be there to restore, as before.
  - `PRAPS_FULL_CONTROLS=1` runs every control over all its cells. That sweep ran at the tested head on both systems, and its counts are in the matrix log. In `cargo ci` each control still stops at its first caught cell. Every cell of the scenario itself still runs; only the controls, whose job is to show that an oracle can fire, stop early.
- **C-004: add-test.** In about half of the `undo` and `restore` steps, drawn by the seed, the other lane records an edit after this lane's handle has opened. The command therefore runs from a state without the other lane's op. When it succeeds, the store holds two op heads, which the test asserts, and the other lane's entry in their merge must be unchanged. The op to restore is drawn from those the handle loaded. The test counts the commands that succeeded apart from the refusals, and asserts at least 1,000 checked, 500 succeeded and 200 run beside the other lane's op. At the tested head it printed: 2,373 commands checked over 200 interleavings, 2,002 of which succeeded, 1,023 of those beside the other lane's op with two heads; the same on both systems.
- **C-005: add-test.** `resume_leaves_files_when_its_workspace_raced` interrupts a write, then commits a sibling op of the same workspace over the same head. Resume must then remove `.praps-tmp-0`, drop the record, leave the files, and add the note that the write was abandoned. With resume's check weakened to "the target is one of the writers", this test fails and `resume_leaves_files_when_its_workspace_moved_on` still passes.
- **C-006: tighten-assertion.** `eight_new_at_once_all_succeed` now prints how many op heads, merge ops and raced changes the eight `new`s left. Whenever the pointer holds more than one change, it checks that `status`'s note names each. `lanes_at_once_end_to_end` prints the op count, the heads and the merges. In three more runs on each system, the eight `new`s left 2 to 4 op heads in WSL and 6 to 8 on Windows, each time with as many changes in the pointer, and `status`'s note named them all. The eight lanes left 35 ops, of which 15 or 16 were made over several heads, and 3 to 8 op heads at the end. A run with no overlap would leave one head and no op over several. The counts are in `e2e-tests.md`.
- **C-007: add-test.** Each gap now has a test that fails if its clause breaks:
  - `lanes` is pinned in the idempotent set;
  - an edit made before `lanes` must be recorded, one op more, and counted on the lane's line;
  - `.` and `..` are among `praps_lane_refusals`' 8 cases, and `praps_lane_refuses_a_last_name_that_is_not_utf8` runs on Unix;
  - in `praps_ops_names_each_workspace`, an op of the lane whose pointer holds two changes must show the smaller;
  - `commands_act_the_same_from_below_in_a_lane` runs `status`, `log`, `ops`, `describe`, `undo`, `restore`, `verify`, `lanes` and a refusal at the lane's top, two levels down, and with a directory below the top given. Each run is on its own copy, and exit code, stdout, stderr and every file are compared byte for byte.
  - **Not compared byte for byte: `new`.** Each run draws a new change id, so its output and files differ by design. Inside a lane it is still checked by `commands_act_on_the_lane_they_run_in`.
- **C-008: tighten-assertion, with one path deferred.** `check()` now accepts "no repository" only where no `lane/.praps/workspace` survived recovery and opening wrote nothing. It accepts an unfinished lane only when the error names this lane and directory, and the only change opening made is to drop the record. With `UnfinishedLane` replaced by `NotInitialised` in `open`, the matrix fails. **Deferred: making the lane again at another directory.** That is code-review finding 2, which backlog T-042 holds. Its fix, a `.praps/workspace` that names the op that made the lane, changes that file's form.
- **C-009: tighten-assertion, with half deferred.**
  - After the next command, which finishes or drops any record a kill left, each lane's `.praps` must hold its `workspace` file and nothing else, apart from a record's temporary name (`working-copy.new`) that an interrupted write left and the next record replaces. No object file may lie outside the two stores.
  - In `lanes.rs`, the check that could not fail is replaced: no object outside the store, more than 8 objects in it, and `fsck` clean.
  - After the next command, the two stores exchange again and every view is compared. Ops made on top of received ops are now checked.
  - **Deferred: many exchanges.** A store that syncs with another again and again belongs to INT-0024. The report says that the scenario exchanges twice.
- **C-010: tighten-assertion.** The lane matrix and both Concurrency runs now also compute a portable digest, which reads every `\` in a lane file as `/`, and assert it as one literal on every system. It equals the Linux digest, and it passed on Windows, in WSL and on GitHub's two systems. So the separator in the store's path is the only difference between the systems' states.
- **C-011: add-test, and the times recorded.**
  - `sim::Processes` now counts needless wakes: those where a process wakes while another was chosen to run, and the run is not aborting. `a_thousand_processes_run` asserts at most one per 100 syscalls. It saw 1 on Windows and 0 in WSL over 3,002 syscalls.
  - With every process woken at each boundary, the count was 1,819,674 and the test failed.
  - The bound is not zero because a condition variable may wake a thread spuriously.
  - The `processes`, `thousand` and `concurrency` unit times of every GitHub sample, and each job's wait for a runner, are now in `e2e-tests.md`, next to the guard of 300 s and the job's limit of 30 minutes. At `b951d6d`, on Windows, `processes` took 27.5, 50.2 and 20.8 s and `thousand` 16.3, 15.8 and 18.4 s; on Linux, 0.4 to 0.9 s and 2.3 to 2.8 s. The slowest `processes` run is under a fifth of the 300 s each guarded test may take.
- **C-012: tighten-assertion of the records.**
  - The test report lists T-002's last clause and INT-0006 AC-6 as open. The question is put at the end of the sprint against `docs/spec/format-v1.md` as T-002 changed it at `609a8df`. Only the owner's own answer is recorded, and the merge waits for it. Backlog T-035 already carries each later change to the format specification.
  - Each sample's wait for a runner is in `e2e-tests.md`.
  - INT-0037 AC-6 names backlog T-044, new in this round.
- **Also found while answering:** `gh run view --job <id> --log`, used to read each attempt's job log, returns the log of the run's latest attempt whatever the job. So `e2e-tests.md` gave attempt 3's Linux runner version, 20261111.82.6, for all three attempts at `5b65da8`; attempts 1 and 2 ran 20261111.196.1. Every job log is now read by its own id (`gh api …/actions/jobs/<job>/logs`), and the record is corrected. The record of Sprint 6's runs is not affected, since each of those runs had one attempt.
