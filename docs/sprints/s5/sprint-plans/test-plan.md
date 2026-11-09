# Sprint 5 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-2, AC-3, AC-4 (Sprint 4's checkpoint) | T-001 / WHEN the record is written THEN `ci-sprint4.md` SHALL name every run, job, image, attempt, `rustc`, test count and duration; WHEN a deny job failed and was re-run THEN both attempts SHALL be given | review of `ci-sprint4.md` against `gh run view` |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | sign-offs INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6, INT-0004 AC-5 | T-001 / WHEN a sign-off has not been given THEN it SHALL be recorded as pending and carried forward | review of `ci-sprint4.md` and the backlog |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-5 (CI half) | T-001 / WHEN every criterion has local and CI evidence THEN the Transition history SHALL record the supported state | review of INT-0007 against `ci-sprint4.md` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 to AC-3, AC-6 unchanged | T-002 / WHEN the suite runs optimised THEN the four asserted digests SHALL match, controls SHALL be caught, `reach` SHALL pass | `matrix_runs_are_reproducible`, `writers_matrix_is_reproducible`, `snapshot_matrix_is_reproducible`, `materialise_matrix_is_reproducible` (the digests), `crash_matrix_finds_no_violations`, `writers_matrix_finds_no_violations`, `snapshot_matrix_finds_no_violations`, `materialise_matrix_finds_no_violations`, `materialise_matrix_with_windows_rules_finds_no_violations`, the negative-control tests, the `reach` step; on Windows, in WSL and in GitHub |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 | T-002 / WHEN tests build THEN workspace crates SHALL be `opt-level = 1`, dependencies `3`, assertions on | `optimised_test_profile_is_the_reviewed_one` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11, AC-9 | T-003 / WHEN the test step runs THEN it SHALL build once and run every unit through `cargo_command` | `test_step_builds_once_then_runs_each_unit`, `hostile_variables_never_reach_a_step`, `test_step_clears_the_recorded_reach` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 (nothing dropped) | T-003 / WHEN units are discovered THEN every `tests/*.rs` and `tests/<dir>/main.rs` SHALL yield one unit per distinct name | `units_cover_every_test_target`, `units_merge_shared_names`, `repository_units_match_its_test_files` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 (nothing dropped) | T-003 / WHEN a manifest declares `[[test]]` or `autotests` THEN the step SHALL fail naming it | `declared_test_targets_are_refused`, `autotests_setting_is_refused` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 | T-003 / WHEN units run THEN at most `available_parallelism()` SHALL run at once, and each unit's output SHALL be one block | `units_never_exceed_the_slots`, `unit_output_is_never_interleaved` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 | T-003 / WHEN a unit fails THEN the others SHALL still run and the step SHALL fail naming it | `a_failed_unit_fails_the_step_after_all_run` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-9 | T-003 / WHEN the step runs twice THEN the tree SHALL be unchanged | `steps_execute_for_real`, the AC-9 check in E2E |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-2, AC-3, AC-4; INT-0004 AC-1 | T-004 / WHEN the workflow is read THEN it SHALL have the triggers, concurrency, `lint`, `ci` and `deny` jobs and caches named | `workflow_matches_ci_contract` with its new mutants, `ci_workflow_is_the_reviewed_one`, and the GitHub runs in E2E |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-5, AC-10 | T-005 / WHEN README and credits are read THEN they SHALL describe parallel test files, separate checks and the triggers, and credit nextest's model | `readme_keeps_statement_and_links_book`, `readme_must_document_each_command`, review of README and `docs/prior-art.md` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-11 | T-002 + T-003 + T-004 composed | E2E `checkpoint_runs_meet_the_budget` |

## Unit Tests
### T-002 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `optimised_test_profile_is_the_reviewed_one` (`crates/xtask/src/pins.rs`): the root `Cargo.toml`'s `[profile.dev]` has `opt-level = 1` and `[profile.dev.package."*"]` has `opt-level = 3`. Neither sets `debug-assertions`, `overflow-checks` or anything else, so both stay on. Removing either table, or turning assertions off, fails the test.
- The four `*_is_reproducible` tests, unchanged, are this task's behavioural evidence: they assert the digests as constants.

### T-003 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `units_cover_every_test_target`: a temporary workspace whose members have `tests/a.rs`, `tests/b.rs`, `tests/c/main.rs` and `tests/common/mod.rs`. It yields `--lib --bins`, `--test a`, `--test b`, `--test c` and `--doc`, in that order, and nothing for `common`.
- `units_merge_shared_names`: two members that both have `tests/crash.rs` yield one `--test crash` unit.
- `repository_units_match_its_test_files`: this repository yields exactly one unit per distinct stem of `crates/*/tests/*.rs`, plus the two fixed units: 36 test files, 34 names.
- `declared_test_targets_are_refused`: a member with `[[test]]` fails discovery, and the error names its manifest.
- `autotests_setting_is_refused`: a member with `autotests = false` (or `true`) fails discovery, and the error names its manifest.
- `units_never_exceed_the_slots`: 12 fake units, each sleeping briefly and counting itself in and out with an atomic counter, under 3 slots. The highest count seen is 3, and every unit ran.
- `unit_output_is_never_interleaved`: 8 fake units, each writing its own marker line 50 times with yields between writes. In the step's output, each unit's lines are contiguous under its header.
- `a_failed_unit_fails_the_step_after_all_run`: unit 2 of 6 fails. All 6 ran, and the step fails naming exactly that unit.
- `test_step_builds_once_then_runs_each_unit`: with a recording closure for cargo, the first command is `test --workspace --locked --no-run`. Each unit's command follows, each starting `test --workspace --locked`.
- Existing tests, kept and updated to `Action::Test`:
  - `ci_steps_are_ordered_and_locked`;
  - `test_step_clears_the_recorded_reach`;
  - `test_step_removes_golden_create`;
  - `hostile_variables_never_reach_a_step` (the `PROBE` workspace now runs its units through the new step);
  - `steps_execute_for_real`.

### T-004 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `workflow_matches_ci_contract`: the committed workflow has no problems. New mutants, each caught:
  - `push` on `[dev, main]`;
  - `workflow_dispatch` removed;
  - `concurrency` removed;
  - `cancel-in-progress: false`;
  - the `lint` matrix without `windows-latest`;
  - the `lint` job without its Clippy step;
  - the Clippy step without `-D warnings`;
  - the `deny` install moved to another job.

  The existing mutants are kept, adapted to the new text.
- `ci_workflow_is_the_reviewed_one`: `ci.yml` equals the new `CI_WORKFLOW`.

### T-005 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `readme_keeps_statement_and_links_book` and `readme_must_document_each_command` pass on the new README.

## Integration Tests
### The suite, optimised and parallel
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `cargo ci` on Windows 11 (NTFS) and in WSL Ubuntu (ext4) at the tested head. Recorded for each: wall time, against Sprint 4's 2,060 s and 1,342 s, and each unit's time.
- On both systems:
  - the test count equals Sprint 4's 477 and 478 plus this sprint's new tests;
  - the four matrix digests match.
- `docs/reach.md` is unchanged after `cargo reach`.
- `cargo deny check` passes in WSL and on Windows.

### The matrices printed in full
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- On both systems, the four matrix files are run with `--nocapture`, optimised. Their per-scenario cell counts and violation lines match Sprint 4's `test-matrix-log.md` line for line, after the same filtering.

## End-to-End Tests
- **Status:** possible
- `checkpoint_runs_meet_the_budget`. During Test, this sprint's commits are pushed to `dev`, so they run on the open `dev → main` pull request. If PR #6 is still open, they join it.
  - The run is sampled 3 times: the run itself, then 2 re-runs of every job.
  - Pass:
    - every check of all 3 samples succeeds;
    - the slowest job of each sample finishes within 20 minutes of the run's start;
    - both `cargo ci` legs print a test count equal to the local one for their platform;
    - the checks appear separately as `lint (ubuntu-latest)`, `lint (windows-latest)`, `cargo ci (ubuntu-latest)`, `cargo ci (windows-latest)` and `cargo deny check`.
  - Recorded: each check's duration, and whether 15 minutes, the aim, was met.
- `superseded_run_is_cancelled`. Two pushes in quick succession: the earlier run is cancelled, and only the later one completes. This is recorded from `gh run list`.
- `push_to_dev_starts_no_run`. A push to `dev` starts only the `pull_request` run, with no `push` run. This is recorded from `gh run list`.
- `cargo_ci_is_idempotent`. In a fresh clone, `cargo ci` run twice leaves `git status` empty. This is INT-0003 AC-9.
- **Not possible:** Nighthawk (INT-0004 AC-6), since Tailscale is not approved.
