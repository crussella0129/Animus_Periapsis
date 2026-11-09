# Sprint 5 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-2, AC-3, AC-4 (Sprint 4's checkpoint) | T-001 / WHEN the record is written THEN `ci-sprint4.md` SHALL name every run of `bf40f24`, `ed14283` and `f0910e6`, with jobs, images, attempts, times, `rustc`, test counts and `ci:` lines; WHEN a deny job failed THEN its cause and any passing re-run SHALL be given | review of `ci-sprint4.md` against `gh run view` and `gh api` |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | sign-offs INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6, INT-0004 AC-5 | T-001 / WHEN a sign-off has not been given THEN it SHALL be recorded as pending and carried forward | review of `ci-sprint4.md` and the backlog |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-5 (CI half) | T-001 / WHEN every criterion has local and CI evidence THEN the Transition history SHALL record the supported state | review of INT-0007 against `ci-sprint4.md` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 to AC-3, AC-6 unchanged | T-002 / WHEN the suite runs optimised THEN the four asserted digests SHALL match, controls SHALL be caught, `reach` SHALL pass | `matrix_runs_are_reproducible`, `writers_matrix_is_reproducible`, `snapshot_matrix_is_reproducible`, `materialise_matrix_is_reproducible` (digests); `crash_matrix_finds_no_violations`, `writers_matrix_finds_no_violations`, `snapshot_matrix_finds_no_violations`, `materialise_matrix_finds_no_violations`, `materialise_matrix_with_windows_rules_finds_no_violations` (controls and invariants); the `reach` step. Run on Windows, in WSL and in GitHub |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 and its Consequence | T-002 / WHEN `Cargo.toml` is read THEN `[profile]` SHALL hold exactly the two reviewed settings | `optimised_test_profile_is_the_reviewed_one` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 (nothing hidden by optimisation) | T-002 / WHEN a reserving-then-failing decode is measured THEN the peak SHALL be at least `COUNT × 8` | `a_reservation_on_the_error_path_is_measured` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11, AC-9 | T-003 / WHEN the test step runs THEN it SHALL build once, run every unit through `cargo_command`, and fail without units if the build fails | `test_step_builds_once_then_runs_each_unit`, `a_failed_build_runs_no_unit`, `hostile_variables_never_reach_a_step`, `test_step_clears_the_recorded_reach`, `test_step_removes_golden_create` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 (nothing dropped) | T-003 / WHEN units are discovered THEN `tests/*.rs` and `tests/<dir>/main.rs` SHALL yield one unit per distinct name | `units_cover_every_test_target`, `units_merge_shared_names` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 (nothing dropped) | T-003 / WHEN an executable built is not run, a unit runs nothing, or a unit compiles THEN the step SHALL fail naming it | `an_unrun_executable_fails_the_step`, `a_unit_that_runs_nothing_fails_the_step`, `a_unit_that_rebuilds_fails_the_step`, and E2E `every_executable_runs_in_ci` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 | T-003 / WHEN units run THEN at most `available_parallelism()` SHALL run at once, each unit's output SHALL be one block, and a total line SHALL end the step | `units_never_exceed_the_slots`, `unit_output_is_never_interleaved`, `the_step_totals_every_test_result` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 | T-003 / WHEN a unit fails THEN the others SHALL still run and the step SHALL fail naming it | `a_failed_unit_fails_the_step_after_all_run` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-9 | T-003 / WHEN the steps run THEN only the test step SHALL capture output; WHEN `cargo ci` runs twice THEN the tree SHALL be unchanged | `run_passes_a_clean_workspace`, `steps_execute_for_real`, E2E `cargo_ci_is_idempotent` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-2, AC-3, AC-4; INT-0004 AC-1 | T-004 / WHEN the workflow is read THEN it SHALL have the triggers, concurrency, jobs, names, timeouts and step order given | `ci_workflow_is_the_reviewed_one`, E2E `checks_appear_separately` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-2, AC-3, AC-4 | T-004 / WHEN `workflow::problems` checks a workflow THEN it SHALL report each kept, changed and new problem | `workflow_matches_ci_contract` (its mutants), `workflow_forms_read_alike` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-2 (triggers, cancelling) | T-004 / triggers and concurrency | E2E `push_to_dev_starts_no_run`, `superseded_run_is_cancelled`; `dispatch_runs_on_dev` deferred to Sprint 6 |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-5, AC-10 | T-005 / WHEN README and credits are read THEN they SHALL describe parallel test files, separate checks and the triggers, and credit nextest | `readme_keeps_statement_and_links_book`, `readme_must_document_each_command`, `prior_art_credits_every_listed_system` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 | T-002 + T-003 + T-004 composed | E2E `sampled_runs_meet_the_budget` |

## Unit Tests
### T-002 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `optimised_test_profile_is_the_reviewed_one` (`crates/xtask/src/pins.rs`): parses the root `Cargo.toml` with `toml_edit`. Its `profile` table holds exactly `dev.opt-level = 1` and `dev.package."*".opt-level = 3`, and nothing else. Each of these fails the test:
  - removing either table;
  - adding `[profile.test]`;
  - adding `dev.overflow-checks = false`;
  - adding `dev.package.codec`;
  - adding `dev.build-override`.
- `a_reservation_on_the_error_path_is_measured` (`crates/codec/tests/alloc.rs`): a function shaped like a decoder reserves `Vec::<u64>::with_capacity(COUNT)` and then returns an error without using it. Measured with `allocation_counter::measure`, `bytes_max` is at least `COUNT × 8`. This is the negative control for `rejected_cheaply` under the optimised profile.
- The four `*_is_reproducible` tests, unchanged, are the behavioural evidence: they assert the digests as constants.

### T-003 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `units_cover_every_test_target`: a temporary workspace whose members have `tests/a.rs`, `tests/b.rs`, `tests/c/main.rs` and `tests/common/mod.rs`. It yields `--lib --bins`, `--test a`, `--test b`, `--test c` and `--doc`, in that order, and nothing for `common`.
- `units_merge_shared_names`: two members that both have `tests/crash.rs` yield one `--test crash` unit.
- `an_unrun_executable_fails_the_step`: the pre-build reports executables `x-1` and `y-2`, but the units report running only `x-1`. The step fails and names `y-2`.
- `a_unit_that_runs_nothing_fails_the_step`: a `--test a` unit reports no `Running` line, and the step fails naming it. A `--doc` unit with none passes.
- `a_unit_that_rebuilds_fails_the_step`: a unit whose output has `   Compiling codec v0.1.0` fails the step as a rebuild.
- `units_never_exceed_the_slots`: 12 fake units, each sleeping briefly and counting itself in and out with an atomic counter, under 3 slots. The highest count seen is 3, and all 12 ran.
- `unit_output_is_never_interleaved`: 8 fake units, each producing 50 marker lines with yields between them. In the step's writer, each unit's lines are contiguous under its header.
- `a_failed_unit_fails_the_step_after_all_run`: unit 2 of 6 fails. All 6 ran, and the step fails naming exactly that unit.
- `the_step_totals_every_test_result`: units whose outputs hold `test result: ok. 3 passed; 0 failed; 1 ignored` and `test result: FAILED. 1 passed; 2 failed; 0 ignored` end the step with `test: 2 units, 4 passed, 2 failed, 1 ignored`.
- `a_failed_build_runs_no_unit`: when the recorded cargo's `--no-run` command fails, the step fails, prints the build output, and runs no unit command.
- `test_step_builds_once_then_runs_each_unit`: with a recording closure for cargo, the first command is `test --workspace --locked --no-run`. Every following command starts `test --workspace --locked`, one per unit.
- Existing tests, kept and updated:
  - `ci_steps_are_ordered_and_locked`: `test` is `Action::Test`.
  - `test_step_clears_the_recorded_reach`.
  - `test_step_removes_golden_create`: asserts over every command the test step builds, the pre-build and each unit, so none can inherit `PERI_GOLDEN_CREATE`.
  - `hostile_variables_never_reach_a_step`: `clean_workspace` gains `tests/probe.rs` with the same assertions as `PROBE`, so both a `--lib --bins` unit and a `--test probe` unit run under the scrubbed environment.
  - `run_passes_a_clean_workspace`: checks that the six `PASS` lines appear in order and that the run ends `ci: all 6 steps passed`. Unit blocks now sit between them.
  - `steps_execute_for_real`.
  - `ci_failures_do_not_stop_later_steps`: for the new execute signature.

### T-004 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `workflow_matches_ci_contract`: the committed workflow has no problems. Each of these new mutants is caught:
  - `push` on `[dev, main]`;
  - `push` on `['**']`;
  - `push` with no `branches`;
  - `push` removed;
  - `workflow_dispatch` removed;
  - `concurrency` removed;
  - `group: ci`, a constant;
  - `cancel-in-progress: true`;
  - `lint` without `windows-latest`;
  - `lint` with `runs-on: ubuntu-latest`;
  - `lint` with `if: false`;
  - `lint` without its Clippy step;
  - Clippy without `-D warnings`;
  - `lint` with `include:`;
  - `deny` without the install step;
  - `deny`'s install step moved to the `lint` job;
  - `deny`'s install step after `cargo deny check`;
  - `deny` using `EmbarkStudios/cargo-deny-action@v2`.

  The existing mutants are kept, adapted to the new text.
- `workflow_forms_read_alike`: updated to the new problem set; "push does not trigger on dev" is gone.
- `ci_workflow_is_the_reviewed_one`: `ci.yml` equals the new `CI_WORKFLOW`.

### T-005 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `readme_keeps_statement_and_links_book` and `readme_must_document_each_command` pass on the new README.
- `prior_art_credits_every_listed_system` passes with `nextest` in `CREDITED`, and fails without its credit.

## Integration Tests
### The suite, optimised and parallel
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `cargo ci` on Windows 11 (NTFS) and in WSL Ubuntu (ext4) at the tested head.
- Recorded for each:
  - wall time, against Sprint 4's 2,060 s and 1,342 s;
  - each unit's time;
  - the `test:` total line;
  - any "Blocking waiting for file lock" lines;
  - any test that failed and then passed on a re-run, a flake against a timing guard.
- On Windows, peak working-set memory of the run, from a 1-second `Get-Process` sample.
- On both systems:
  - the `test:` total equals Sprint 4's 478 and 479 (including the CR-07 test) plus this sprint's new tests;
  - the four matrix digests match.
- `docs/reach.md` is unchanged after `cargo reach`.
- `cargo deny check` passes in WSL and on Windows.

### The matrices printed in full
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- On both systems, the four matrix files run optimised with `--nocapture`. Their per-scenario cell counts and violation lines match Sprint 4's `test-matrix-log.md` line for line, after the same filtering.

## End-to-End Tests
- **Status:** possible, except `dispatch_runs_on_dev`.
- **Setup (owner's decision, 2026-11-09).** During Test, `dev` is pushed and the sprint's `dev → main` pull request is opened. It is the pull request the Loop checkpoint uses. Nothing is merged.
- `sampled_runs_meet_the_budget`. Three samples of the tested head:
  - the pull request's first run, with a cold cache;
  - two attempts of "re-run all jobs" (`gh run rerun <run>`), with a warm cache.

  For each attempt, from `gh api …/actions/runs/<run>/attempts/<n>/jobs`:
  - **Time:** the first job's `started_at` to the last job's `completed_at`.
  - **Queue time:** each job's `created_at` to `started_at`, reported separately.

  Pass:
  - every job of every sample succeeds;
  - each sample's time is at most 20 minutes.

  Recorded: each job's duration, whether the 15-minute aim was met, and which sample was cold.
- `checks_appear_separately`. The pull request shows these checks:
  - `lint (ubuntu-latest)`;
  - `lint (windows-latest)`;
  - `cargo ci (ubuntu-latest)`;
  - `cargo ci (windows-latest)`;
  - `cargo deny check`.
- `every_executable_runs_in_ci`. On both `cargo ci` legs, the step passes its unrun-executable check. Each `test:` total equals the local one for that platform.
- `push_to_dev_starts_no_run`. A push to `dev` starts only a `pull_request` run, with no `push` run. This is read from `gh run list`.
- `superseded_run_is_cancelled`. Two sprint commits are pushed in quick succession, for example the test results, then a follow-up. The earlier `pull_request` run ends `cancelled`, and the later one completes. This is read from `gh run list`.
- `cargo_ci_is_idempotent`. In a fresh WSL clone, `cargo ci` run twice leaves `git status --porcelain` empty (INT-0003 AC-9).
- **Not yet possible:**
  - `dispatch_runs_on_dev`. `workflow_dispatch` works only once the workflow declaring it is on `main`. Unlocked by this sprint's merge; recorded by Sprint 6 through the backlog task T-005 adds.
  - Nighthawk (INT-0004 AC-6). Unlocked when the owner approves Tailscale.
