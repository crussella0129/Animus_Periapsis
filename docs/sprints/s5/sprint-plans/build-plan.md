# Sprint 5 Build Plan

## Intents
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: active (revised in this plan); acceptance criteria covered:
  - AC-2: every CI run is `--locked`, on the new triggers, and a push to `main` is never cancelled;
  - AC-3: `cargo deny check` in CI, through a checksum-verified install that retries;
  - AC-4: fmt and Clippy in a check of their own on each platform;
  - AC-5: the nextest credit;
  - AC-9: `cargo ci` stays flagless and idempotent with the parallel test step;
  - AC-10: the README;
  - AC-11 (new): each sampled run of the sprint's pull request takes at most 20 minutes, with every test executable run.
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered: AC-1 to AC-3 and AC-6, each kept unchanged through optimised builds and parallel test files. The evidence is the four asserted matrix digests, the controls and the reach table.
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
  - **WHEN** the record is written, **THEN** `ci-sprint4.md` **SHALL** name, for every workflow run of PR #6's commits `bf40f24` and `ed14283` and of the `main` merge commit `f0910e6`, each of the following, all taken from `gh run view` and `gh api …/attempts/N/jobs` output:
    - head SHA, event and conclusion;
    - every job, with its conclusion, runner image, attempt number and start and end times;
    - the `rustc` version, the test count of each `cargo ci` leg, and the `ci:` summary line.
  - **WHEN** a `cargo deny check` job failed, **THEN** the record **SHALL** give the failure's cause as the log states it, and, where it was re-run, the passing attempt's job ID. A failure that was never re-run **SHALL** be recorded as such.
  - **WHEN** an owner sign-off has not been given in the conversation, **THEN** the record **SHALL** say it is pending, and backlog T-020 **SHALL** be replaced by a carried-forward task. No sign-off **SHALL** be recorded without the owner's words. Merging PR #6 counts as accepting the checkpoint only.
  - **WHEN** every INT-0007 criterion has local and CI evidence, **THEN** INT-0007's Transition history **SHALL** record the state change the evidence supports. Otherwise it **SHALL** record what is still missing.

### T-002: Optimised test builds, with every Sprint 4 result unchanged
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** Cargo.toml, crates/xtask/src/pins.rs, crates/codec/tests/alloc.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0005 AC-1 to AC-3 and AC-6 unchanged; INT-0003 AC-11 (speed), and the Consequence on the optimised `dev` profile.
- **Success criterion (EARS):**
  - **WHEN** the root `Cargo.toml` is read, **THEN** its `[profile]` table **SHALL** hold exactly two settings:
    - `dev.opt-level = 1`;
    - `dev.package."*".opt-level = 3`.

    It **SHALL** have no `test`, `release`, `bench` or `build-override` entry, no other package entry, and no other key, so debug assertions and overflow checks keep their `dev` defaults (on).
  - **WHEN** the suite runs with that profile, **THEN** each of these **SHALL** still hold:
    - the four asserted matrix digests match: store `d36b8c9c…`, four writers `253b1eab…`, Snapshot `c1e39cb1…` and Materialise `6274344b…`, asserted by `matrix_runs_are_reproducible`, `writers_matrix_is_reproducible`, `snapshot_matrix_is_reproducible` and `materialise_matrix_is_reproducible`;
    - the negative controls are caught;
    - `cargo ci`'s `reach` step passes against the committed `docs/reach.md`.
  - **WHEN** code that reserves `COUNT` elements and then fails is measured with `allocation_counter` under this profile, **THEN** the measured peak **SHALL** be at least `COUNT × 8` bytes. This shows the allocation tests still see a reservation that optimisation could otherwise remove.
- **Notes:** the profile goes in `Cargo.toml`, not the environment, because `cargo ci` scrubs `CARGO_PROFILE_*` (`crates/xtask/src/ci.rs`, `SCRUBBED_PREFIXES`). `profile.test` inherits from `dev`, so the test build, Clippy and `cargo build` share one profile.

### T-003: Run test targets in parallel inside `cargo ci`
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/xtask/src/ci.rs, crates/xtask/src/units.rs (new), crates/xtask/src/main.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0003 AC-11 (speed with nothing dropped) and AC-9 (`cargo ci` flagless and idempotent).
- **Success criterion (EARS):**
  - **WHEN** `cargo ci` reaches its `test` step, now `Action::Test`, **THEN** it **SHALL**:
    - clear the recorded reach, as before;
    - build once with `cargo test --workspace --locked --no-run`, capturing the build's output;
    - if that build fails, print its output and fail the step without running any unit;
    - otherwise run these units:
      - `--lib --bins`;
      - one unit per distinct test-target name;
      - `--doc`.

    Each unit **SHALL** run as `cargo test --workspace --locked <selection>`, built by `cargo_command`, so it inherits none of the scrubbed variables.
  - **WHEN** units are discovered, **THEN** each workspace member's `tests/*.rs` and `tests/<dir>/main.rs` **SHALL** yield a unit named after the file stem or directory, in sorted order between `--lib --bins` and `--doc`. A name shared by several members, as `crash` and `cli` are, **SHALL** yield one unit.
  - **WHEN** all units have finished, **THEN** the step **SHALL** fail, naming each executable or unit, in any of these cases:
    - an executable the pre-build reported on an `Executable … (<path>)` line was not reported on a `Running … (<path>)` line by any unit;
    - a unit other than `--doc` reported no `Running` line;
    - a unit's output contains a `Compiling ` line, meaning it rebuilt something.

    Executables are compared by file name.
  - **WHEN** units run, **THEN**:
    - at most `std::thread::available_parallelism()` of them **SHALL** run at once;
    - each unit's captured standard output and error **SHALL** be written whole to the step's writer, as one block headed by the unit's name, result and duration, in the order units finish;
    - no two units' output **SHALL** interleave;
    - the step **SHALL** end with one line, `test: <u> units, <p> passed, <f> failed, <i> ignored`, summed from every `test result:` line.
  - **WHEN** any unit fails, **THEN** every other unit **SHALL** still run, and the step **SHALL** fail, naming each failed unit.
  - **WHEN** the steps run, **THEN** `run_with`'s execute closure **SHALL** receive the step writer (`FnMut(&Step, &mut dyn Write) -> Outcome`). Steps other than `test` **SHALL** keep their output inherited, as now.
  - **WHEN** `cargo ci` runs twice, **THEN** the working tree **SHALL** be unchanged, as AC-9 requires.
- **Notes:**
  - `crates/xtask/src/workspace.rs` `members` gives the members.
  - The runner takes the unit command as a closure, as `execute_with` takes `cargo`, so its tests need no real cargo.
  - The pre-build keeps the units cheap. Every unit keeps `--workspace`, so feature resolution is identical and nothing should rebuild. The `Compiling` check turns any rebuild into a failure rather than a slow, racy pass.

### T-004: The CI workflow, its pin and its clause
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** .github/workflows/ci.yml, crates/xtask/src/pins.rs, crates/xtask/src/workflow.rs, crates/xtask/src/ci.rs (tests)
- **Depends on:** T-003
- **Acceptance criterion:** INT-0003 AC-2, AC-3, AC-4 and AC-11; INT-0004 AC-1.
- **Success criterion (EARS):**
  - **WHEN** the workflow is read, **THEN** it **SHALL** contain:
    - **Triggers:** `pull_request` (unfiltered), `push` with `branches: [main]`, and `workflow_dispatch`.
    - **Concurrency:** a top-level `concurrency` block with `group: ci-${{ github.ref }}` and `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`.
    - **Job `lint`,** named `lint (${{ matrix.os }})`:
      - a `fail-fast: false` matrix of `ubuntu-latest` and `windows-latest`, `runs-on: ${{ matrix.os }}`, and `timeout-minutes: 30`;
      - steps `actions/checkout@v7`, `run: rustup toolchain install`, `Swatinem/rust-cache@v2`, `run: cargo fmt --all --check` and `run: cargo clippy --workspace --all-targets --locked -- -D warnings`.
    - **Job `ci`,** named `cargo ci (${{ matrix.os }})`:
      - the same matrix and `timeout-minutes: 30`;
      - steps checkout, `rustup toolchain install`, `Swatinem/rust-cache@v2` and `run: cargo ci`.
    - **Job `deny`,** named `cargo deny check`:
      - `runs-on: ubuntu-latest` and `timeout-minutes: 10`;
      - steps checkout, `rustup toolchain install`, `taiki-e/install-action@v2` with `tool: cargo-deny@0.20.2`, and `run: cargo deny check`.
  - **WHEN** `workflow::problems` checks a workflow, **THEN** it **SHALL** report each of the following:
    - **Kept:** `push` excludes branches with a `!` pattern; there is no `pull_request` trigger; `pull_request` is configured inline or filtered, or a `pull_request` branch list omits `dev` or `main`; `push` is filtered by `paths`, `paths-ignore` or `branches-ignore`; the workflow sets step defaults; no job runs `cargo ci` on both platforms; a job or step continues on error; a command handles its own failure with `||` or `set +e`.
    - **Dropped:** "push does not trigger on dev".
    - **Changed:** "no deny job runs cargo-deny on ubuntu-latest" now also covers a `deny` job without its own `taiki-e/install-action` step installing `cargo-deny`, one whose install step comes after `cargo deny check`, and one that uses `EmbarkStudios/cargo-deny-action`, which is no longer accepted.
    - **New:**
      - `push` does not trigger on `main`;
      - `push` triggers on another branch or a pattern (any entry other than `main`);
      - `push` has no branch list;
      - there is no `workflow_dispatch` trigger;
      - there is no top-level `concurrency`, its `group` does not contain `github.ref`, or its `cancel-in-progress` is not exactly `${{ github.event_name == 'pull_request' }}`;
      - no `lint` job runs both commands on both platforms. The test is the `ci` job's predicate: a matrix of both platforms with no `include` or `exclude`, `runs-on: ${{ matrix.os }}`, no `if:` on the job or on either command step, and one step each running `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`.
  - **WHEN** `workflow_matches_ci_contract` runs, **THEN** each new or changed problem **SHALL** have a mutant that is caught, and the existing mutants **SHALL** be kept, adapted to the new text. `workflow_forms_read_alike` **SHALL** assert the new problem set.
  - **WHEN** `ci.yml` differs from `pins.rs`'s `CI_WORKFLOW`, **THEN** `ci_workflow_is_the_reviewed_one` **SHALL** fail.
- **Notes:**
  - Actions are pinned by major tag, as `actions/checkout@v7` already is.
  - `rust-cache` comes after `rustup toolchain install`, because its key includes the rustc version.
  - `install-action` verifies the checksum by default and retries downloads with `curl --retry 10` (its `main.sh`, read on 2026-11-09).

### T-005: README, credits and backlog
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** README.md, docs/prior-art.md, crates/xtask/src/docs.rs, docs/work/tasks.md
- **Depends on:** T-003, T-004
- **Acceptance criterion:** INT-0003 AC-5 (credits) and AC-10 (README).
- **Success criterion (EARS):**
  - **WHEN** the README is read, **THEN** its Build and test section **SHALL** say:
    - that `cargo ci` runs test files in parallel and fails if any test executable goes unrun;
    - that CI shows fmt and Clippy, the suite and cargo-deny as separate checks;
    - when CI runs: pull requests, pushes to `main`, and on demand.

    `readme_keeps_statement_and_links_book` and `readme_must_document_each_command` **SHALL** still pass.
  - **WHEN** `docs/prior-art.md` is read, **THEN** it **SHALL** credit nextest's model of running test binaries in parallel (Apache-2.0 or MIT), with no code taken from it. `nextest` **SHALL** join `CREDITED` in `crates/xtask/src/docs.rs`, so `prior_art_credits_every_listed_system` enforces it.
  - **WHEN** the backlog is read, **THEN** it **SHALL** contain each of these:
    - T-020 replaced by its carried-forward task (the merge and close runs, and the sign-offs);
    - a task for Sprint 6 to run the workflow by `workflow_dispatch` on `dev` and record it;
    - if AC-11 is missed, a task naming the slowest unit.
