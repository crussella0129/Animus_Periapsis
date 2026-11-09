# Sprint 5 Test Report

**Tested head:** `87df514`, which carries T-001 to T-005 and the [code review](code-review.md) fixes (`1ebb35e`). After it came one test-only commit, `664b156` (round-1 test critique C-002), covered by its own binary on both systems. The next pull-request run will therefore print 501 tests on Windows and 502 on Linux, against 500 and 501 at the tested head.

**Where the results are:**
- [unit-tests.md](unit-tests.md);
- [integration-tests.md](integration-tests.md);
- [e2e-tests.md](e2e-tests.md);
- [test-matrix-log.md](test-matrix-log.md);
- [ci-sprint4.md](ci-sprint4.md).

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11: no job of a sampled run of the sprint's pull request over 20 minutes (aim 15), with every test executable run and every matrix at full size | T-002 + T-003 + T-004 / `sampled_runs_meet_the_budget`: three samples of run 42707577882 at `87df514`, whose slowest jobs took 9:41, 15:50 and 16:02; `every_executable_runs_in_ci` (`test: 45 test executables built, 45 run` in every leg); the 69-line matrix comparison | **pass, 3 of 3.** The 15-minute aim was met in 1 of 3 samples, missed by the Windows suite alone (backlog T-027). All three samples were warm; the only cold sample, 12:46, is from the first round at `3289008`, which differs from the tested head in xtask code only | Test evidence links this report |
| INT-0003 | AC-2: every CI run `--locked`, on pull requests, pushes to `main` and on demand; superseded pull-request runs cancelled, pushes to `main` never | T-004 / `workflow_matches_ci_contract` (47 mutants), `ci_workflow_is_the_reviewed_one`; E2E `push_to_dev_starts_no_run`, `superseded_run_is_cancelled` (run 42707506720 `cancelled`) | **Pull-request half: pass on GitHub.** **`main` half: evidenced structurally only** (the exact-group rule in `workflow.rs`) until this sprint merges; T-024 records the merge's `push` run. **On demand:** possible only after the merge (T-025) | as above |
| INT-0003 | AC-3: `cargo deny check` in CI | `cargo deny check` passed in all 7 attempts of this sprint's runs, installed with `verifying sha256 checksum`; locally on Windows and in WSL | pass | as above |
| INT-0003 | AC-4: fmt and Clippy in their own check on each platform | `lint (ubuntu-latest)` and `lint (windows-latest)` passed in every run; the checker requires a lint job of its own (CR-01) | pass | as above |
| INT-0003 | AC-9: `cargo ci` flagless and idempotent | `cargo_ci_is_idempotent`: a fresh WSL clone ran `cargo ci` twice with an empty `git status`, at both heads; the working checkout stayed clean on Windows | pass | as above |
| INT-0003 | AC-5 (credits), AC-10 (README) | nextest credited and enforced through `CREDITED`; README review (lines 63 to 70); the two README tests | pass; the AC-5 sign-off is still pending | as above |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 to AC-3, AC-6 unchanged under optimised, parallel runs | T-002 / the four `*_is_reproducible` digests and the 12 control tests at `87df514` on both systems and in GitHub; 69 of 69 matrix lines identical to Sprint 4 on both systems (at `3289008`); the `reach` step and `cargo reach` with no change | pass | Test evidence links this report |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: suite on both systems, OS and toolchain named | Windows 11 (500 tests) and WSL Ubuntu, Linux 6.6.114.1 (501), rustc 1.99.1; GitHub `windows-2025-vs2026` and `ubuntu-26.04` runners, every check on both | pass | Test evidence links this report |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1 to AC-5 | T-001 / [ci-sprint4.md](ci-sprint4.md): AC-5's CI run passed in every leg of PR #6 and its merge; this sprint's runs pass the same tests again | moved `active` → `realized` in T-001. **The owner is asked to confirm.** AC-1 holds by its proxy, and "Certified" was read under INT-0002 as now written, whose AC-1 sign-off is pending (round-1 C-005) | Test evidence links this report |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md), INT-0003, INT-0004 | Sprint 4's GitHub runs; sign-offs | T-001 / [ci-sprint4.md](ci-sprint4.md) | runs recorded (three deny download failures, two re-run to success, one superseded); sign-offs pending, carried forward in T-024 | Sprint 4 CI record linked in T-001 |

## Summary
- **Unit tests:** 500 passed on Windows and 501 on Linux at `87df514`, with 0 failed and 0 ignored. 22 are new, and none was removed.
- **Integration tests:**
  - `cargo ci` passed in 269 s on Windows and in 253 s and 207 s in WSL. Sprint 4 took 2,060 s and 1,342 s.
  - The matrices are unchanged line for line, and the reach table is unchanged.
  - `cargo deny check` passed on both systems.
  - Peak memory was 374 MB.
- **E2E tests:**
  - six GitHub samples over two heads, all green;
  - five separate checks;
  - every executable run;
  - pushes to `dev` start no run, and a superseded run is cancelled;
  - `cargo ci` is idempotent.

  Not yet possible: the `workflow_dispatch` run (T-025) and Nighthawk.

## CI Confirmation
- **Sprint 4's checkpoint** ([ci-sprint4.md](ci-sprint4.md)). All `cargo ci` legs passed: 478 tests on Windows and 479 on Linux. Three deny jobs failed on a corrupt download of cargo-deny; two were re-run to success and one was superseded. The merge into `main` (run 42681766482) passed every job.
- **This sprint's pull request,** [crussella0129/Animus_Periapsis#7](https://github.com/crussella0129/Animus_Periapsis/pull/7):
  - run 42698665267 at `3289008`: 3 attempts, all success;
  - run 42707506720 at `1ebb35e`: cancelled by the next push, as designed;
  - run 42707577882 at `87df514`, the tested head: 3 attempts, all success.

  The authoritative conclusion is **success**, from run 42707577882's attempt 3, finished 2026-11-09T22:42Z.

## Failures
None at the tested head. Problems that were found and fixed:
- **Code review:** 9 findings, all fixed except CR-05's per-unit timeout (backlog T-026).
- **Round-1 test critique:** 11 concerns, proceed-with-caveats:
  - C-002 fixed in `664b156`;
  - C-001, C-007 and C-008 fixed in the records;
  - C-003, C-004, C-005, C-006, C-009, C-010 and C-011 deferred with reasons (T-024, T-028, and the INT-0007 note).
- **Round-2 test critique:** 3 concerns, proceed-with-caveats, all answered ([critique.md](critique.md)).

## Technical Debt Identified
- **T-024:** the runs of the checkpoint and the merge. It confirms AC-2's `main` half, the Windows time against AC-11, and the 501 and 502 totals. It also carries the sign-offs.
- **T-025:** record a `workflow_dispatch` run on `dev` after the merge.
- **T-026:** a per-unit timeout in the test step (CR-05).
- **T-027:** shard `materialise_crash`, for the 15-minute aim on Windows.
- **T-028:** tighten the runner and workflow tests (round-1 C-003, C-009, C-010, C-011).
- **Lock waits.** Cargo printed "Blocking waiting for file lock" 78 times on Windows and about 146 times in WSL. They cost the short units a few seconds each, nothing on the critical path.
