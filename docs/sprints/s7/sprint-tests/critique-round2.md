# Test Critique — Sprint 7, round 2

Round 2. A read-only reviewer, run as a subagent, checked the responses to round 1 ([critique-round1.md](critique-round1.md)) against the repository at `4872ee3`, with the tested head `b951d6d`, using the Sprint Loops test-critic prompt. Its concerns and its account of round 1 are given as it wrote them; the responses are at the end. The verdict was `block`, so the code and the evidence were revised and reviewed again ([critique.md](critique.md)).

## Concerns

### C-001: The lane-file refusals are right at a lane's top, but not from below it or through `verify`
- **Where:**
  - `crates/praps/src/commands.rs`:
    - `top_above` (lines 183–197) gives `hint::unreadable_above()` for every `place::Unreadable`.
    - `store_of` (654–663) treats a `.praps/workspace` that does not parse as if there were no lane file. It falls back to the lane's own `.praps/store`.
    - `verify` (789–794) then answers that missing store with `hint::unfinished_store()`, which offers `praps init`.
  - `build-plan.md`:
    - T-007: "A `.praps/workspace` that exists but cannot be read or does not parse **SHALL** stop the search with its error".
    - T-006: "Neither hint **SHALL** offer `init`".
  - The records: `critique-round1.md`, the response to C-001; `code-review.md` finding 6, now "Fixed"; `tasks.md` T-036, which no longer carries it.
- **Quote:** "The search for a top says which file could not be read, the store or the lane file. `praps` prints that file's path and reason, and for the lane file the hint "make that file readable, then run the command again"."
- **Failure mode:** negative-path
- **Why it matters:**
  - **From below the top.** Run any command below the top of a lane whose `.praps/workspace` cannot be read. The message now names the file, but the hint is still "make the store of each repository above that directory readable". `every_refusal_prints_one_hint` makes this case only at the top.
  - **`verify` with a lane file that does not parse.** Run `praps verify` in such a lane, at its top or below it:
    - `is_top` stops at the lane, and `store_of` picks `<lane>/.praps/store`.
    - The refusal says that path "is not a Periapsis store", and the hint says to run `praps init` there.
    - `init` then refuses ("it is a lane's working copy"), and the lane file's own error is never printed.
  - **`verify` in a lane whose store is gone.** It gives the same `init` hint, where `status` says to put the store back.
  - **No test reaches these paths.** No test runs `verify` in such a lane. `a_damaged_workspace_file_stops_the_search` and the refusal corpus use `status` only.
  - **A false pass follows.** The test plan counts `verify` among the commands that act on a lane (`commands_act_on_the_lane_they_run_in`). So the report would trace T-007's top clause, and INT-0037 AC-4's "every way `verify` … can fail", to evidence that does not cover them.
- **Suggested response:** add-test, with the fix:
  - Choose `top_above`'s hint by the `Unreadable` kind.
  - Have `verify` refuse a lane file that does not parse, and a lane whose store is gone, with those errors' own text and hints.
  - Add three cases to `every_refusal_prints_one_hint`: an unreadable file seen from below, `verify` with a damaged lane file, and `verify` with its store gone.
  - Add the lane-file rows to the review `table()` in `hint.rs`, which has no row for `unreadable_file`.

  Otherwise defer-with-rationale: put the from-below hint back in T-036 with the `verify` paths, and mark T-007's top clause unmet for `verify` in the report.

### C-002: The lost-op control fires because an exchange fails, not because a crash loses an op, and its comment says otherwise
- **Where:** `concurrency.rs` `concurrency_catches_lost_ops`; `crates/store/src/lib.rs` `write_file` (`let sync_data = dir != self.objects || self.syncs_object_data();`); `test-matrix-log.md`; the negative-controls table in `unit-tests.md`; "The controls" in `integration-tests.md`.
- **Quote:** "Op files whose data is never synced do not all survive the crash: an acknowledged op is missing from the store, which the `lost` oracle finds"
- **Failure mode:** evidence-drift
- **Why it matters:**
  - **Op files survive.** `UnsyncedObjects` syncs every op file's data and loses only object data. The matrix log says so itself: "a crash loses objects but no op".
  - **The "lost" ops are ops never exchanged.** The 29 ops of the first two-store cell, and the 3,632 of the full sweep, are ops that `exchange` never copied. `copy_object` stops at the first object whose data the crash lost, and that cell's `store` count rises for the same failure.
  - **What the control shows.** It shows that `lost` counts an op missing from a store. It does not show that a crash that loses an acknowledged op is caught, yet "loses 29 acknowledged ops" invites the report to claim that.
  - **Its one-store half is dead weight.** That half can never fire, yet it runs all 160 cells in every `cargo ci`.
- **Suggested response:** tighten-assertion of the comment and the records:
  - Say the 29 are ops that a failed exchange left on one store.
  - Run the two-store run first.
  - Claim that a crash-lost op is caught only after adding a store variant that does not sync op files.

### C-003: After the second exchange, only convergence is checked
- **Where:** in `concurrency.rs` `check()`, the block after "The ops just made stand on ops received from the other store"; "The Concurrency scenario" in `integration-tests.md`; the response to C-009 in `critique-round1.md` ("Ops made on top of received ops are now checked").
- **Quote:** "in the two-store run, after the next command, that a second exchange brings both stores and every lane to one view, so ops made on top of received ops are checked too (C-009)"
- **Failure mode:** weak-assertion
- **Why it matters:**
  - **The new ops are not checked for loss.** The after-recovery `describe` ops are added to no lane's acknowledged list. `lost`, `writer` and `kept` are not applied after the second exchange.
  - **What is checked cannot fail on a merge bug.** The checks are that the exchange succeeds, and that both stores and every lane compute the same library merge. That is equal views from equal op sets, the comparison round 1 said cannot fail.
  - **A wrong merge would pass.** A merge that took a stale writer for an op made on received ops would pass.
  - **Where the risk is.** T-009's clause is met by the first exchange's checks; the risk is the overstated record.
- **Suggested response:** tighten-assertion. Add each lane's after-recovery op to its acknowledged ops, then run `check_store` on both stores after the second exchange, with the fresh-load `kept` comparison. Or reword the record to say only that the stores converge. Deferring many exchanges to INT-0024 stands.

### C-004: `new` in a lane is checked only by its exit code, and the reason given for leaving it out does not hold
- **Where:**
  - `cli.rs`:
    - `commands_act_the_same_from_below_in_a_lane` has no `new`.
    - `commands_act_on_the_lane_they_run_in` checks `new` by exit code only.
    - `commands_act_the_same_from_below` (lines 1253–1276) already compares `new` from the top and from below, with the drawn change id masked.
  - `e2e-tests.md`.
  - T-007: "WHEN any command runs in a lane, at its top or below it, THEN it SHALL act on that lane".
- **Quote:** "`new` is not in the corpus: each run draws a new change id, so its output differs by design."
- **Failure mode:** weak-assertion
- **Why it matters:**
  - **Masking already works.** The repository-level test compares the number of ops, the files outside `.praps` and the next `status`, so the same comparison works in a lane.
  - **A misdirected `new` would pass.** As it stands, nothing fails if `new` run below a lane's top records its change in another workspace. The `undo` and `restore` that follow it exit 0 either way.
  - **Round 1 named this.** It is round 1's C-007, last bullet.
- **Suggested response:** add-test. Either give the in-lane test the masked `new` comparison, or assert after each `new` that the first line of `ops` names the lane. Correct the record's reason.

## Round-1 concerns at `b951d6d`, and the open items
- **Round-1 C-001: partly resolved.**
  - Resolved for `open` (`an_unreadable_workspace_file_is_refused`), for the search at a lane's top, and for `init`. Each has a test that fails without its fix.
  - Still open from below the top and through `verify`: concern C-001 above.
- **Round-1 C-002: resolved, honestly reworded.** `merge.rs` builds only ordered collections, so "by construction" holds. The two mutants that take the heads as a list show the property check can fail.
- **Round-1 C-003: resolved.**
  - The `kept` check now compares a fresh load with the acknowledged entry.
  - A control now makes `lost` fire.
  - The full sweep is recorded on both systems, and its counts match T-009's: writer 2,974, readers 1,170, lost 3,632.
  - Caveats:
    - The mechanism behind the lost-op control: concern C-002 above.
    - Under INT-0003 AC-11, the report should say that `cargo ci` stops each Concurrency control at its first caught cell.
- **Round-1 C-004: resolved.** 1,023 commands ran beside the other lane's op, with two heads asserted, and successes are counted apart from refusals.
- **Round-1 C-005: resolved.** The sibling op is made over the same head, and the note, the record, the temporary name and the files are each checked.
- **Round-1 C-006: resolved.**
  - Overlap is recorded rather than asserted, which suits real processes.
  - The race note is asserted whenever the pointer holds several changes.
  - The counts are consistent: 35 ops = 3 + 8 + 24.
- **Round-1 C-007: resolved except for `new`** (concern C-004 above).
  - `.` and `..` are refused as directories that are not free.
  - `full_path` normalises DIR first, so the name check for `.` and `..` cannot be reached. The two cases show the outcome, not that check.
- **Round-1 C-008: resolved.**
  - The lane matrix now enforces "before `.praps/workspace` is in place", and checks the interrupted lane's name and directory.
  - Deferring finding 2 (making the lane again at another directory) to T-042 is defensible:
    - T-006 promises a remake "at DIR" only.
    - Nothing is lost.
    - The fix changes the lane file's form.
  - Caveats:
    - INT-0014 still says "A lane whose operation never landed is refused at its next command" and lists two open cases. The kill route is a third way for two directories to drive one entry. It belongs in the report's known defects, and in that chapter at close.
    - T-042 says the kill "reaches the first case", but it reaches the second, and the full stop before that sentence is missing.
- **Round-1 C-009: first half resolved, second half deferred.**
  - First half: the `.praps` checks and the no-object-outside-the-stores check run under concurrency, and `eight_lanes_share_one_store` now asserts something that can fail.
  - Second half: deferring many exchanges to INT-0024 is defensible:
    - The locked plan scopes AC-5 to two simulated stores that exchange every operation.
    - It lists "INT-0010 AC-5 in full" as not yet possible.
    - The report should claim AC-5 only within that scope (the exchanges come after the run, none during).
  - The added second exchange is weaker than recorded (concern C-003 above).
- **Round-1 C-010: resolved.** The three portable literals equal the Linux digests in the code and in the log.
- **Round-1 C-011: resolved.**
  - The needless-wake count catches the every-process wake: 1,819,674 against a bound of 30.
  - GitHub's per-attempt times and waits are recorded and consistent; the slowest `processes` run took 50.2 s against a 300 s guard.
  - GitHub's own wake counts are not recorded, so the bound rests on local runs only.
- **Round-1 C-012: resolved in the records.** T-044 exists, waits of 2 to 4 s are recorded, and the attempt-log correction is made.
  - Leaving the format sign-off for later is defensible as a caveat:
    - The test plan's review allows "its absence recorded".
    - No test can supply an owner's answer.
    - `sign-offs.md` and `unit-tests.md` keep INT-0006 AC-6 and T-002's last clause open, with the merge waiting on them.
  - The report must say T-002 is complete except that clause.
- **Numbers checked, all consistent with each other and with the code:**
  - test totals: 663 and 666; 636 = 560 + 76, of which 3 are Unix-only;
  - refusals checked: 41 and 38 (36 on both systems, plus 5 on Unix and 2 on Windows);
  - matrix result lines: 99 = 94 + 5;
  - full control sweep: 308, 180, 320 and 3,632;
  - GitHub: every job time, and the 1 min 27 s over the 15-minute aim.
- **What the verdict rests on.**
  - The four open items alone would support proceed-with-caveats:
    - the format sign-off;
    - finding 2;
    - two exchanges, with INT-0024 for the rest;
    - `new` left out inside a lane.
  - The block rests on concern C-001: a T-007 clause that fails for `verify`, and a round-1 fix recorded as complete that is only partly done.

## Confidence
block

## Responses

The tests below are commit `4068f43`, which is now the tested head. `cargo ci` passed there on both systems: 663 tests on Windows and 666 in WSL; the round-2 cases are new checks inside existing tests, so the totals do not move. Each new check was shown able to fail: the change named was made once, the test run, and the source put back.

- **C-001: add-test, and the code fixed.**
  - `top_above` chooses its hint by what could not be read. A lane's `.praps/workspace` gets "make that file readable"; a store above keeps the store's hint.
  - `verify` still finds a lane's store through its lane file. A lane file that does not parse is now refused as opening the lane refuses it, with `BadWorkspaceFile`'s text and hint, and a file that cannot be read with its path and the file's hint.
  - A lane whose named store is not there is refused as `StoreGone`, with "put the store back", as `status` refuses it. No refusal in a lane offers `init`.
  - The hint table has four new rows, two for the lane file and two for `verify` in a lane: 43 kinds.
  - `every_refusal_prints_one_hint` makes three more kinds: an unreadable `.praps/workspace` from below the lane's top, `verify` in a lane whose file does not parse, and `verify` in a lane whose store is gone. It counts 44 on Linux and 41 on Windows.
  - The test fails with the store's hint put back for a lane file in `top_above`, with a damaged lane file taken for no lane file in `store_of`, and with a lane's missing store answered as a working copy's.
- **C-002: tighten-assertion.**
  - The control's comment and the records now say what it shows: an exchange stopped at an object the crash lost, and left on one store acknowledged ops it had not yet copied.
  - Its two-store run comes first, so `cargo ci` no longer runs the 160 one-store cells that cannot fire.
  - That the check catches an acknowledged op which a crash itself loses is not claimed. A store variant that does not sync op files is added to backlog T-034.
- **C-003: tighten-assertion.**
  - The op each lane's next command makes is now acknowledged as the lane's last.
  - After that command, and in the two-store run after the second exchange, every store is checked again as after recovery: no acknowledged op missing, one writer for each lane, the entry each lane loads equal to its last acknowledged one, and one view for every store and lane.
  - With the second exchange left out, `concurrency_finds_no_violations` fails.
  - The scenario still finds 0 violations in all 320 cells, with the same coverage, and its digests do not move: they are taken before the checks run.
- **C-004: add-test.**
  - `commands_act_the_same_from_below_in_a_lane` now runs `new` at the lane's top, from two levels down, and with a directory below the top given, each on its own copy.
  - It masks the drawn change ID and compares the number of ops, the lane's files outside `.praps`, and what `status` prints next.
  - It checks that the newest op `ops` lists names the lane.
  - The record's reason is corrected.
- **The caveats.**
  - INT-0014's chapter gains the kill route as a third way for two directories to drive one entry, at the sprint's close, and the test report lists it among the known defects. T-042's sentence is corrected.
  - The test report says three things:
    - `cargo ci` stops each Concurrency control at its first caught cell;
    - INT-0010 AC-5 holds for two simulated stores that exchange after the run, not during it;
    - T-002 is complete except its last clause.
  - `.` and `..` are refused as directories that are not free, as the critique says, and `unit-tests.md` says so.
  - GitHub's runs check the needless-wake bound but do not print the count; the counts recorded are local.
