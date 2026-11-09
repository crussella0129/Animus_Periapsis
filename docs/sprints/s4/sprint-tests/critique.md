# Test Critique — Sprint 4

Round 2. This critique was written read-only against the result files at `24e9322`, with tested head `9fa59fb`.

**Delta on round 1.** Each fix listed in the Responses table is in `9fa59fb`, and each one is sound:
- **C-001 to C-003.** `within` watchdog; the `late` counter; the panicked source; `a_kill_comes_before_an_ordering_wait`; seeds where process 0 arrives first. I re-ran two mutants in a temp clone, running only `cargo test -p sim --test processes`:
  - with the abort unwinds in `boundary` disabled, `panic_in_a_process_ends_the_run` fails, and so does `ordering_deadlock_is_reported`;
  - with the kill moved after the ordering wait, `a_kill_comes_before_an_ordering_wait` fails.
- **C-005, C-006, C-008, C-009.** Fixed as described.
- **C-007.** The odd-seed asserts are in `writers_kills_happen`.
- **C-010.** The tests now go through `run()` and check that no file is written.
- **C-011.** `EXPECTED` has 21 points and matches the plan's T-006 list. `expectations_are_real` keeps it complete. `a_point_alone_lets_another_process_run` discriminates.

**C-004 deferral.** It is reasonable. `closure()` counts every present op whose parent is absent, and each process's chain is at most two ops, the second naming the first, so closure subsumes the prefix property. But see C-002 below.

**C-012 rejection.** It is reasonable: `book` is its own package, and README line 68 carries the four-command sentence. The test report must record it as a departure.

**Record checks.**
- `test-matrix-log.md` matches `s4r2_windows.log` and `s4r2_wsl.log` line for line, after the documented filtering.
- Test counts: 381 → 451 `#[test]` attributes, +70, which agrees with 407/408 → 477/478. Every per-file count is correct.
- Every cell count, killed count, digest and time in the result docs agrees with the logs.
- Every test name the plan gives exists.

## Concerns

### C-001: The CR-07 fix has no test, though the records say every fix has one
- **Where:**
  - `code-review.md` intro and CR-07 response;
  - `unit-tests.md` T-009, last bullet;
  - `crates/workspace/src/lib.rs` `snapshot` (`self.reported.clear()`);
  - build-plan T-009 EARS.
- **Quote:** code-review.md: "each is answered at the end, and every fix has a test". unit-tests.md: "`snapshot` clears the report first, so an early failure reports nothing stale (CR-07)."
- **Failure mode:** negative-path
- **Why it matters:**
  - Commit `5dfb418` changed no workspace test for CR-07.
  - In a temp clone I commented out `self.reported.clear();` and ran `cargo test -p workspace --test history`: 14 of 14 passed. Nothing else in the workspace calls `reported()` except `praps/src/main.rs`.
  - After C-005, every report test runs on a freshly opened `Workspace`. So a reused `Workspace` whose next snapshot fails in `load` or the scan would show the previous command's entries as its own, and no test notices. That contradicts T-009's "they come from the command's first snapshot".
  - The impact is limited, because `praps` opens a fresh workspace per command. Still, the record claims coverage that does not exist.
- **Suggested response:** add-test. On one `Workspace`:
  1. Run `status` with a raw name present, so `reported()` is non-empty.
  2. Make the next snapshot fail in `load`, for example with several heads, as `praps_status_with_several_heads_fails` does.
  3. Assert that the command errs and `reported()` is empty.

  Alternatively, change the code-review record to say CR-07 is untested.

### C-002: Two integration-results claims disagree with the code and with the C-004 response
- **Where:** `integration-tests.md` "The `sim` tests" (line 133) and the four-writer section (line 86); `critique-round1.md` C-004 response.
- **Quote:**
  - "Since C-001 to C-003, every `processes.rs` run is under a deadlock watchdog, and the scheduler tests check the order of events over 10 or 20 seeds each, not one."
  - "it does not compute atomicity: that check lives in the store matrix, which still runs".
- **Failure mode:** evidence-drift
- **Why it matters:**
  - Only 3 of the 18 tests call `within`: `orderings_are_released_when_the_source_process_ends`, `a_kill_comes_before_an_ordering_wait` and `panic_in_a_process_ends_the_run`.
  - Several tests use a single seed: `processes_return_results_in_input_order`, `ordering_deadlock_is_reported`, `killed_process_state_stays_visible` and `kill_at_a_point`.
  - (`unit-tests.md`'s "the tests that could hang" is the accurate wording.)
  - The atomicity sentence gives a different rationale from the accepted deferral. The store matrix's atomicity check covers only the single-writer `Chain` scenario, so it says nothing about four writers. The sound argument, closure subsuming the per-process prefix, appears only in the round-1 Responses. A reader of the result doc would take INT-0005 AC-2's atomicity as evidenced under four writers by a matrix that does not run four writers.
- **Suggested response:** tighten-assertion, in the docs only:
  - name the three watchdog tests and drop "every … 10 or 20 seeds each";
  - replace the atomicity sentence with the closure argument the C-004 response gives, and say that the summary line's "atomicity 0" is uncomputed for this matrix.

### C-003: The doc-only EARS clauses of T-001, T-008 (spec) and T-010 have no recorded verification
- **Where:** build-plan T-001, T-008 "the spec", T-010 EARS; test-plan Intent Traceability rows for INT-0003 AC-5/9/10 and T-001; `unit-tests.md` "By build task", which covers T-002 to T-009 only; `crates/xtask/src/docs.rs`.
- **Quote:** test-plan: "`readme_problems` with the four-command sentence, review of README and `docs/prior-art.md`"; "review of `ci-sprint3.md` against `gh run view`".
- **Failure mode:** EARS-coverage
- **Why it matters:**
  - This sprint extended `readme_keeps_statement_and_links_book` and `readme_must_document_each_command`, giving them the new `NO_FLAGS` sentence and the "README must say what `cargo reach` does" check. They are the executed tests behind INT-0003 AC-9 and AC-10, but no result doc names them.
  - No result doc records the reviews the plan names:
    - `ci-sprint3.md` against `gh run view`;
    - the prior-art credits;
    - the INT-0005 and INT-0007 Consequence rewrites;
    - `docs/spec/simulation.md` against its six required topics;
    - the backlog moves.
  - So these EARS clauses trace to nothing in the Test-phase evidence.
- **Suggested response:** tighten-assertion. Add a T-001/T-010 section, or say so in `test-report.md`:
  - name the two README tests;
  - record each review and what was checked, including the C-012 departure;
  - for T-008, record that the spec was checked against its list.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---|---|
| C-001 | **add-test.** `a_failed_snapshot_reports_nothing_stale` (`workspace/tests/history.rs`, commit `6665913`) follows the suggested shape. On one `Workspace`, `status` reports a raw name. A second store handle then commits two ops on the head, the next `status` fails with `SeveralHeads`, and `reported()` is empty. With `self.reported.clear()` removed, the test fails. It passed 15 of 15 in the `history` binary on Windows and in a fresh WSL clone at `6665913`, which changes nothing else ([unit-tests.md](unit-tests.md) and [test-matrix-log.md](test-matrix-log.md), "After the tested head"). `code-review.md`'s intro and its CR-07 row now say the test came after this critique. |
| C-002 | **tighten-assertion.** [integration-tests.md](integration-tests.md) now names the three watchdog tests and the five that loop over seeds, and says other tests, such as `ordering_deadlock_is_reported` and `kill_at_a_point`, still use one seed. The atomicity sentence now gives the closure argument, says the store matrix's check runs one writer only, and says the four-writer summary's `atomicity 0` is not a result. |
| C-003 | **tighten-assertion.** [test-report.md](test-report.md) gains "The documentation clauses (T-001, T-008's spec, T-010)". It names the two README tests and records each review and what it checked: `ci-sprint3.md` against `gh run view`, re-read for all four runs; the credits; both Consequences; the spec's six topics; the backlog moves; and the C-012 departure. |
