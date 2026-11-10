# Test Critique — Sprint 5

Round 1. A read-only subagent reviewer critiqued the result files at `81d130d` (tested head `87df514`). It checked every job time in [e2e-tests.md](e2e-tests.md) against `gh api …/attempts/N/jobs`; all six attempts matched to the second. The per-unit times and cache lines matched the job logs. The concerns are condensed here; the responses are at the end.

## Concerns

### C-001: The negative-control promise cites the wrong tests, and the results name none
- **Where:** test-plan traceability, INT-0005 row; `unit-tests.md` T-002
- **Quote:** "`crash_matrix_finds_no_violations`, … (controls and invariants)"
- **Failure mode:** EARS-coverage
- **Why it matters:** The `*_finds_no_violations` tests run the correct variants only; the 12 control tests are not named for the tested head. They do pass in the GitHub logs at `87df514`.
- **Suggested response:** add-test (name the 12 controls).

### C-002: `run_command_keeps_the_streams_in_order` does not test order
- **Where:** `code-review.md` CR-06; `units.rs`
- **Quote:** "A child that writes to stdout, then stderr, then stdout again: the captured output keeps that order."
- **Failure mode:** weak-assertion
- **Why it matters:** No child wrote to both streams, so reverting CR-06 would still pass.
- **Suggested response:** tighten-assertion.

### C-003: Workflow mutants assert "some problem", not the intended one
- **Where:** `workflow_matches_ci_contract`
- **Quote:** "**47 mutants:** 39 from the build and 8 from the code review."
- **Failure mode:** weak-assertion
- **Why it matters:** Deleting a rule leaves the test green whenever another rule catches its mutant. No mutant was found to be caught by the wrong rule today.
- **Suggested response:** tighten-assertion.

### C-004: AC-2's `main` half is not shown to work, and is not listed as deferred
- **Where:** INT-0003 AC-2; `e2e-tests.md` "Not yet possible"; T-024
- **Quote:** "a run for a push to `main` is never cancelled or replaced by a later push."
- **Failure mode:** e2e-cop-out
- **Why it matters:** It becomes observable only at the merge, and T-024 does not say to confirm it.
- **Suggested response:** defer-with-rationale.

### C-005: INT-0007 moved to `realized` during Build, on a Certified rule the owner has not approved, with AC-1 held only by proxy
- **Where:** INT-0007 history and Completion evidence; `ci-sprint4.md`
- **Quote:** "AC-1 is `formal` with obligation `unstated`, so its proxy evidence counts as `empirical`, per INT-0002."
- **Failure mode:** intent-coverage
- **Why it matters:** Four problems:
  - it is the Book's first `realized` chapter;
  - it rests on INT-0002 semantics that the owner has not signed off;
  - it moved before this sprint's critique;
  - the Sprint 5 T-001 completion link is missing.

  The substance is sound.
- **Suggested response:** defer-with-rationale (link, note, owner confirmation).

### C-006: AC-11 passes with little headroom, and the Windows time depends on the runner
- **Where:** `e2e-tests.md`; INT-0003 AC-11
- **Quote:** "In both runs over 15 minutes, `materialise_crash` alone took 715 s and 729 s"
- **Failure mode:** flake-risk
- **Why it matters:**
  - The Windows leg's six samples span 9:41 to 16:12.
  - The only cold sample is at `3289008`.
  - Nothing watches the 20-minute limit after this sprint.
- **Suggested response:** defer-with-rationale.

### C-007: Local evidence is not archived, and the plan's per-unit times are incomplete
- **Where:** `integration-tests.md`; `unit-tests.md`; `e2e-tests.md`
- **Quote:** "Every line reporting a matrix result was compared, as a multiset, …"
- **Failure mode:** evidence-drift
- **Why it matters:** Four gaps in the record:
  - no matrix log is committed;
  - per-unit times at the tested head are incomplete;
  - `run_passes_a_clean_workspace`'s own `ok` is vacuous;
  - no job IDs are given.
- **Suggested response:** add-test (evidence).

### C-008: `ci-sprint4.md` names a log source that returns the wrong attempt
- **Where:** `ci-sprint4.md` sources and table
- **Quote:** "`gh run view --job <job> --log`."
- **Failure mode:** evidence-drift
- **Why it matters:**
  - For a re-run job, that command returns the latest attempt's log.
  - The deny jobs' image version is not given.

  The recorded values are right.
- **Suggested response:** tighten-assertion.

### C-009: The new README sentences have no executed test
- **Where:** build-plan T-005; `unit-tests.md` T-005
- **Quote:** "`readme_keeps_statement_and_links_book` and `readme_must_document_each_command` pass on the new README."
- **Failure mode:** EARS-coverage
- **Why it matters:** Neither test checks the three new statements.
- **Suggested response:** tighten-assertion, or record README review.

### C-010: Smaller T-003 clauses have no asserting test
- **Where:** build-plan T-003
- **Quote:** "as one block headed by the unit's name, result and duration, in the order units finish"
- **Failure mode:** EARS-coverage
- **Why it matters:** Three things have no test:
  - the duration in a header, a `FAIL` header, and finish order;
  - inherited output for the other steps;
  - `slots()`'s wiring.

  All are low risk.
- **Suggested response:** tighten-assertion, or defer-with-rationale.

### C-011: `units_never_exceed_the_slots` depends on timing
- **Where:** `units.rs`
- **Quote:** "`assert_eq!(most.load(Ordering::SeqCst), 3);`"
- **Failure mode:** flake-risk
- **Why it matters:** A starved worker thread would fail it. None of 9 recorded runs failed.
- **Suggested response:** defer-with-rationale, or use a barrier.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---|---|
| C-001 | **add-test (evidence).** [unit-tests.md](unit-tests.md) T-002 names the 12 control tests: five store `*_variant_is_caught`, five `writers_catch_*`, `snapshot_catches_unsynced_objects` and `materialise_catches_no_record`. It says the plan's mapping was wrong. All 12 printed `... ok` at `87df514`, locally on Windows and in WSL (24 lines) and in GitHub jobs 129468494598 and 129468494534 (12 each), as re-read for this response. |
| C-002 | **tighten-assertion.** In `664b156` the test runs this test binary as a child (`stream_order_child`) that writes stdout, then stderr, then stdout, and asserts the markers' order. With the pre-CR-06 capture put back, the test fails with `order-second` last; that mutation was run and reverted. `xtask`'s binary passed 111 of 111 at `664b156` on Windows and in a fresh WSL clone. |
| C-003 | **defer-with-rationale.** The reviewer traced the main mutants and found none caught by the wrong rule, so this is a regression gap, not a false pass. Pairing 47 mutants with their problem texts is backlog **T-028**. |
| C-004 | **defer-with-rationale.** The `main` half becomes observable only when this sprint merges. **T-024** now also records that the merge started a `push` run on `main` that was neither cancelled nor replaced. The test report marks AC-2's `main` half as evidenced structurally only (`workflow.rs`'s exact-group rule) until then. |
| C-005 | **defer-with-rationale.** INT-0007 links `[Sprint 5 T-001 completion]`. A new history note says three things: AC-1 holds by its proxy; "Certified" was read under INT-0002 as now written, whose AC-1 sign-off is pending; and the move is put to the owner at this checkpoint, with a follow-on chapter if the owner does not accept it, since `realized` is terminal. |
| C-006 | **defer-with-rationale.** [e2e-tests.md](e2e-tests.md) records the six-sample spread (Windows 9:41 to 16:12, at least 3:48 under the budget), the runner dependence of `writers` (256 to 544 s) and `materialise_crash` (397 to 729 s), and that the only cold sample is at `3289008`, which stands for the tested head because the later changes touch only xtask. T-024 records the checkpoint and merge runs' Windows times against AC-11. T-027 aims at 15 minutes. |
| C-007 | **add-test (evidence).** [test-matrix-log.md](test-matrix-log.md) commits the filtered matrix lines of both systems and the comparison result. [integration-tests.md](integration-tests.md) gives every unit's time at `87df514`. [e2e-tests.md](e2e-tests.md) gives all 15 job IDs. [unit-tests.md](unit-tests.md) says `run_passes_a_clean_workspace` is proved through its parent, `hostile_variables_never_reach_a_step`. |
| C-008 | **tighten-assertion.** [ci-sprint4.md](ci-sprint4.md) names `…/actions/jobs/<job>/logs` as the source and says why `gh run view --job` is not. The deny row now covers all six attempts with image `ubuntu-26.04` 20261028.624, with `invalid magic` counts of 6 in each failed log and 0 in the passing ones, as re-read for this response. |
| C-009 | **tighten-assertion (by review).** [unit-tests.md](unit-tests.md) T-005 records the README review as the evidence for the clause, quoting what README lines 63 to 70 say. Adding the three statements to `readme_problems` joins T-028's list of test tightenings. |
| C-010 | **defer-with-rationale.** Each gap is low risk, and every CI log shows `---- unit …: PASS in N s` blocks. The duration, `FAIL` header and finish-order assertions, and the inherited-output and `slots()` checks, are backlog **T-028**. |
| C-011 | **defer-with-rationale.** None of the 9 recorded runs failed, and the test runs only in the `lib and bins` unit. A barrier instead of sleeps is in **T-028**. |
