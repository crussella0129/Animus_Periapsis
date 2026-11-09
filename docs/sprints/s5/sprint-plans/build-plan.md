# Sprint 5 Build Plan

## Intents
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: active (revised in this plan); acceptance criteria covered:
  - AC-2: every CI run is `--locked`, on the new triggers;
  - AC-3: `cargo deny check` in CI, through a checksum-verified install;
  - AC-4: fmt and Clippy in a check of their own on each platform;
  - AC-9: `cargo ci` stays flagless and idempotent with the parallel test step;
  - AC-11 (new): every check of a checkpoint run finishes within 20 minutes, with nothing dropped.
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered: AC-1 to AC-3 and AC-6. Each is kept unchanged through optimised builds and parallel test files: the four asserted matrix digests, the controls and the reach table.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered: AC-1, with every check on both platforms.
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — through T-001 (backlog T-020) only.

## Schema Tree
- Sprint Goal: CI fast enough to wait for, with every test still run and each kind of check visible on its own
  - Carried forward
    - T-001: Record the Sprint 4 checkpoint's GitHub Actions runs and owner sign-offs (backlog T-020)
  - Speed
    - T-002: Optimised test builds, with every Sprint 4 result unchanged
    - T-003: Run test targets in parallel inside `cargo ci`
  - The workflow
    - T-004: The CI workflow, its pin and its clause
  - Documentation
    - T-005: README, credits and backlog

## Execution Sequence

### T-001: Record the Sprint 4 checkpoint's GitHub Actions runs and owner sign-offs
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- **Touches:** docs/sprints/s5/sprint-tests/ci-sprint4.md, docs/intents/INT-0001-periapsis-charter.md, docs/intents/INT-0002-typed-acceptance-criteria.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0004-windows-and-linux.md, docs/intents/INT-0006-format-v1.md, docs/intents/INT-0007-crash-safe-storage.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:**
  - the GitHub-matrix halves of Sprint 4's checkpoint: INT-0003 AC-2 to AC-4, INT-0004 AC-1, and INT-0007 AC-5's CI run;
  - owner sign-offs: INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6, INT-0004 AC-5.
- **Success criterion (EARS):**
  - **WHEN** the record is written, **THEN** `ci-sprint4.md` **SHALL** name, for every workflow run of PR #6's commits `bf40f24` and `ed14283`, each of the following, all taken from `gh run view` output:
    - head SHA, event and conclusion;
    - every job, with its conclusion, runner image and attempt number;
    - the `rustc` version, the test count of each `cargo ci` leg, and the duration of each job.
  - **WHEN** a `cargo deny check` job failed and was re-run, **THEN** the record **SHALL** give both attempts, the failure's cause as the log states it, and the passing attempt's job ID.
  - **WHEN** an owner sign-off has not been given in the conversation, **THEN** the record **SHALL** say it is pending, and backlog T-020 **SHALL** be replaced by a carried-forward task. No sign-off **SHALL** be recorded without the owner's words.
  - **WHEN** every INT-0007 criterion has local and CI evidence, **THEN** INT-0007's Transition history **SHALL** record the state change the evidence supports. Otherwise it **SHALL** record what is still missing.
- **Notes:** if PR #6 has been merged by the time this runs, the `main` push run is recorded too.

### T-002: Optimised test builds, with every Sprint 4 result unchanged
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** Cargo.toml
- **Depends on:** (none)
- **Acceptance criterion:** INT-0005 AC-1 to AC-3 and AC-6 unchanged; INT-0003 AC-11 (speed).
- **Success criterion (EARS):**
  - **WHEN** the workspace builds its tests, **THEN** workspace crates **SHALL** compile at `opt-level = 1` and dependencies at `opt-level = 3`, through `[profile.dev]` and `[profile.dev.package."*"]` in the root `Cargo.toml`, with `debug-assertions` and `overflow-checks` left on.
  - **WHEN** the suite runs with that profile, **THEN** the four asserted matrix digests **SHALL** still match. They are store `d36b8c9c…`, four writers `253b1eab…`, Snapshot `c1e39cb1…` and Materialise `6274344b…`, asserted in `store/tests/crash.rs`, `store/tests/writers.rs`, `workspace/tests/crash.rs` and `workspace/tests/materialise_crash.rs`. In addition, the negative controls **SHALL** still be caught, and `cargo ci`'s `reach` step **SHALL** pass against the committed `docs/reach.md`.
- **Notes:**
  - The profile goes in `Cargo.toml`, not in the environment: `cargo ci` scrubs `CARGO_PROFILE_*` (`crates/xtask/src/ci.rs`, `SCRUBBED_PREFIXES`).
  - `profile.test` inherits from `dev`, so `cargo test`, Clippy and `cargo build` share one profile and one build cache.

### T-003: Run test targets in parallel inside `cargo ci`
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/xtask/src/ci.rs, crates/xtask/src/units.rs (new), crates/xtask/src/main.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0003 AC-11 (speed with nothing dropped) and AC-9 (`cargo ci` flagless and idempotent).
- **Success criterion (EARS):**
  - **WHEN** `cargo ci` reaches its `test` step, **THEN** it **SHALL**:
    - clear the recorded reach, as before;
    - build once with `cargo test --workspace --locked --no-run`;
    - then run these units:
      - `--lib --bins`;
      - one unit per distinct test-target name across the workspace members;
      - `--doc`.

    Each unit **SHALL** run as `cargo test --workspace --locked <selection>`, built by `cargo_command`, so it inherits none of the scrubbed variables.
  - **WHEN** units are discovered, **THEN** every member's `tests/*.rs` and `tests/<dir>/main.rs` **SHALL** yield a unit named after the file stem or directory, in sorted order. A name shared by several members, as `crash` and `cli` are, **SHALL** yield one unit.
  - **WHEN** a member's manifest declares `[[test]]` or sets `autotests`, **THEN** the step **SHALL** fail, naming the manifest, without running any unit. Discovery would not see such a target.
  - **WHEN** units run, **THEN**:
    - at most `std::thread::available_parallelism()` of them **SHALL** run at once;
    - each unit's standard output and error **SHALL** be printed whole, as one block headed by the unit's name, result and duration, in the order units finish;
    - no two units' output **SHALL** interleave.
  - **WHEN** any unit fails, **THEN** every other unit **SHALL** still run, and the step **SHALL** fail, naming each failed unit.
  - **WHEN** the step runs twice, **THEN** the working tree **SHALL** be unchanged, as AC-9 requires.
- **Notes:**
  - `crates/xtask/src/workspace.rs` `members` gives the members. The runner takes the unit command as a closure, as `execute_with` does, so its tests need no cargo.
  - The pre-build is what makes the units cheap: `--workspace` on every unit keeps feature resolution identical, so no unit rebuilds.

### T-004: The CI workflow, its pin and its clause
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** .github/workflows/ci.yml, crates/xtask/src/pins.rs, crates/xtask/src/workflow.rs, crates/xtask/src/ci.rs (tests)
- **Depends on:** T-003
- **Acceptance criterion:** INT-0003 AC-2, AC-3, AC-4 and AC-11; INT-0004 AC-1.
- **Success criterion (EARS):**
  - **WHEN** the workflow is read, **THEN** it **SHALL**:
    - trigger on `pull_request`, on `push` to `main` only, and on `workflow_dispatch`;
    - set a `concurrency` group keyed by `github.ref` with `cancel-in-progress: true`;
    - define these jobs:
      - `lint`, on a matrix of `ubuntu-latest` and `windows-latest`, running `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`;
      - `ci`, on the same matrix, running `cargo ci`;
      - `deny`, on `ubuntu-latest`, installing `cargo-deny@0.20.2` with `taiki-e/install-action@v2` and then running `cargo deny check`;
    - put `Swatinem/rust-cache@v2` on `lint` and `ci`.
  - **WHEN** `workflow::problems` checks the workflow, **THEN** it **SHALL** report each of the following:
    - `push` triggering on a branch other than `main`;
    - a missing `pull_request` or `workflow_dispatch`;
    - a missing cancelling `concurrency`;
    - a `lint` job without both platforms, or without each of the two commands;
    - every problem it reports today.

    Each **SHALL** have a mutant in `workflow_matches_ci_contract`.
  - **WHEN** `ci.yml` differs from `pins.rs`'s `CI_WORKFLOW`, **THEN** `ci_workflow_is_the_reviewed_one` **SHALL** fail.
- **Notes:**
  - Actions are pinned by major tag, as `actions/checkout@v7` already is.
  - The `deny` job keeps `runs_deny_on_ubuntu`'s rules: no `if:`, no narrowing flags, and no `||`.

### T-005: README, credits and backlog
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** README.md, docs/prior-art.md, docs/work/tasks.md, crates/xtask/src/docs.rs (only if a README check needs the new wording)
- **Depends on:** T-003, T-004
- **Acceptance criterion:** INT-0003 AC-5 (credits) and AC-10 (README).
- **Success criterion (EARS):**
  - **WHEN** the README is read, **THEN** its Build and test section **SHALL** say:
    - that `cargo ci` runs test files in parallel;
    - that CI shows fmt and Clippy, the suite and cargo-deny as separate checks;
    - when CI runs: pull requests, pushes to `main`, and on demand.

    `readme_keeps_statement_and_links_book` and `readme_must_document_each_command` **SHALL** still pass.
  - **WHEN** `docs/prior-art.md` is read, **THEN** it **SHALL** credit nextest's model of running test binaries in parallel (Apache-2.0 or MIT), with no code taken from it.
  - **WHEN** the sprint's backlog is read, **THEN** T-020 **SHALL** be replaced by its carried-forward task. Any budget miss **SHALL** have a task naming the slow test file.
