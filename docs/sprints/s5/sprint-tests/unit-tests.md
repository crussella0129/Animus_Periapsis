# Sprint 5 Unit Test Results

**Tested head:** `3289008` (`32890088d62beed6d6fec059ff4a65b27bb307f2`), which carries T-001 to T-005.

**Run by:** `cargo ci` on Windows 11 (NTFS) in the working checkout, and twice in a fresh WSL Ubuntu clone (ext4), rustc 1.99.1. Every test named here printed `... ok` in the Windows run's log. The WSL runs passed the same tests and one more, `real_symlinked_directories_are_not_followed`, which is Unix only.

| System | `test:` total line | `ci:` line |
|---|---|---|
| Windows | `test: 36 units, 496 passed, 0 failed, 0 ignored` | `ci: all 6 steps passed` |
| WSL, run 1 | `test: 36 units, 497 passed, 0 failed, 0 ignored` | `ci: all 6 steps passed` |
| WSL, run 2 | `test: 36 units, 497 passed, 0 failed, 0 ignored` | `ci: all 6 steps passed` |

Sprint 4's GitHub legs ran 478 tests on Windows and 479 on Linux. This sprint adds 18:
- `optimised_test_profile_is_the_reviewed_one` (T-002);
- `a_reservation_on_the_error_path_is_measured` (T-002);
- the 16 tests in `crates/xtask/src/units.rs` (T-003).

478 + 18 = 496, and 479 + 18 = 497. No test was removed.

## By build task

### T-002: Optimised test builds
- `optimised_test_profile_is_the_reviewed_one` (`crates/xtask/src/pins.rs`).
  - It parses the root `Cargo.toml` with `toml_edit`. The profile holds exactly `dev.opt-level = 1` and `dev.package."*".opt-level = 3`.
  - Eight mutants are each caught:
    - either table removed;
    - `[profile.test]` added;
    - `overflow-checks = false`;
    - `debug-assertions = false`;
    - a `codec` package override;
    - a `build-override`;
    - a `[profile.release]` entry.
- `a_reservation_on_the_error_path_is_measured` (`crates/codec/tests/alloc.rs`).
  - Under `opt-level = 1`, a function that reserves `COUNT` elements and then fails, with no `black_box` or other barrier, measures at least `COUNT × 8` bytes. So optimisation did not remove the allocation, and `rejected_cheaply` still sees a reserving decoder.
  - The fallback the plan decided on, a `codec` override, was **not needed**. The pin keeps exactly two settings.
- **The behavioural evidence** is the four reproducibility tests, unchanged, each asserting its digest as a constant. All passed on both systems:
  - `matrix_runs_are_reproducible` (store `d36b8c9c…`);
  - `writers_matrix_is_reproducible` (four writers `253b1eab…`);
  - `snapshot_matrix_is_reproducible` (Snapshot `c1e39cb1…`);
  - `materialise_matrix_is_reproducible` (Materialise `6274344b…`).

### T-003: The parallel test step
All in `crates/xtask/src/units.rs` unless noted.

- **Discovery:**
  - `units_cover_every_test_target`: `tests/a.rs`, `tests/b.rs` and `tests/c/main.rs` become units, and `tests/common/mod.rs` and fixtures do not. The order is `--lib --bins`, then a, b, c, then `--doc`.
  - `units_merge_shared_names`: two members' `tests/crash.rs` files become one unit.
  - `repository_units_match_its_test_files`: this repository's 34 distinct stems become 34 `--test` units, between the two fixed units.
- **Commands:**
  - `test_step_builds_once_then_runs_each_unit`: the first command is `test --workspace --locked --no-run`, and every unit's command starts `test --workspace --locked --no-fail-fast`.
  - `a_failed_build_runs_no_unit`: only the pre-build runs, and its output is printed.
  - `the_step_sets_cargo_terminal_settings`: the pre-build and both unit commands carry `CARGO_TERM_QUIET=false`, `CARGO_TERM_VERBOSE=false` and `CARGO_TERM_COLOR=never`.
- **Running units:**
  - `units_never_exceed_the_slots`: 12 units under 3 slots. At most 3 ran at once, and all 12 ran.
  - `unit_output_is_never_interleaved`: 8 units with 50 marker lines each, under 4 slots. Each unit's markers are contiguous, directly under its header.
  - `the_doc_unit_runs_alone_and_last`: `--doc` started when no other unit was running, and finished last.
- **Failures:**
  - `a_failed_unit_fails_the_step_after_all_run`: all 7 commands ran, and the step names only `u2`.
  - `a_failing_unit_is_not_reported_as_unrun`: the failed unit is named, and the executable it never reached is not reported as unrun.
  - `an_unrun_executable_fails_the_step`: `y-2` is named.
  - `a_unit_that_runs_nothing_fails_the_step`: `a` is named; `--doc` with no `Running` line passes.
  - `a_unit_that_rebuilds_fails_the_step`: a plain `Compiling` line and an ANSI-coloured one both fail unit `a`. The same lines in `--doc` do not.
- **Output:**
  - `the_step_totals_every_test_result`: `test: 2 units, 4 passed, 2 failed, 1 ignored`.
  - `cargo_status_lines_parse_on_both_systems`: a Windows `Executable unittests src\lib.rs (target\debug\deps\codec-….exe)`, a Unix `Running tests/crash.rs (…)`, a `Running unittests` line and a coloured `Running` line yield the right file names.
- **Updated in `crates/xtask/src/ci.rs`:**
  - `ci_steps_are_ordered_and_locked`: `test` is `Action::Test`.
  - `test_step_clears_the_recorded_reach`.
  - `test_step_removes_golden_create`: now covers the pre-build and all 36 unit commands of this repository, and none inherits a scrubbed variable.
  - `hostile_variables_never_reach_a_step`: the child `cargo ci` printed both `test inherits_no_stripped_variable ... ok` and `test probe_inherits_no_stripped_variable ... ok`. So a `--lib --bins` unit and a `--test probe` unit both ran under the scrubbed environment.
  - `run_passes_a_clean_workspace`;
  - `steps_execute_for_real`;
  - `ci_failures_do_not_stop_later_steps`.

### T-004: The workflow
- `workflow_matches_ci_contract` (`crates/xtask/src/ci.rs`). The committed workflow has no problems, and each of the following mutants is caught.

  **39 mutants:**
  - triggers (7): `push` on `[dev, main]`, `['**']` and `[develop]`; `push` without branches; `push` removed; `pull_request_target`; `workflow_dispatch` removed;
  - concurrency (4): removed; constant group; group without `github.sha`; `cancel-in-progress: true`;
  - the suite (5): `ci` on `ubuntu-latest` only; `cargo ci` moved out of its job; `cargo ci` with `if: false`; a matrix `exclude`; `continue-on-error`;
  - lint (8): one platform; `runs-on: ubuntu-latest`; job `if: false`; Clippy removed; fmt removed; no `-D warnings`; a matrix `include`; Clippy with `if: false`;
  - deny (9): install removed; install moved to `lint`; install after the check; the Embark action; another tool installed; install with `if: false`; check with `if: false`; job with `if: false`; deny on Windows with an unrelated job added;
  - shell (6): five `||` forms and `set +e`.

  **The loops:**
  - 8 narrowed deny commands, including `cargo deny fetch` and `fetch && echo check`;
  - 4 settings that move the check (`working-directory`, `shell`, job `defaults`, workflow `defaults`);
  - 8 trigger filters;
  - 2 `!` patterns;
  - an inline `pull_request`.

  `pull_request` with `branches: [dev, main]`, and `cargo deny --all-features --locked check`, are both accepted.
- `workflow_forms_read_alike` (`crates/xtask/src/workflow.rs`).
  - Block sequences, quoted scalars, comments, `||` inside expressions and mixed-case action names read like `ci.yml`'s forms.
  - `push` on `develop` gives exactly two problems: "push does not trigger on main" and "push triggers on another branch or a pattern: develop".
- `ci_workflow_is_the_reviewed_one`: `ci.yml` equals the pinned `CI_WORKFLOW`.

### T-005: README and credits
- `readme_keeps_statement_and_links_book` and `readme_must_document_each_command` pass on the new README.
- `prior_art_credits_every_listed_system` passes with `nextest`, now the 37th name in `CREDITED`. The nextest row in `docs/prior-art.md` is what satisfies it.
