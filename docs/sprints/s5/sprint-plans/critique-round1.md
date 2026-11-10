# Plan Critique — Sprint 5

Round 1. A read-only subagent reviewer critiqued the plans at `3c3124a`. Its concerns are condensed here; the responses are at the end.

## Concerns

### C-001: AC-11's "within 20 minutes of the run starting" cannot be measured on re-runs
- **Where:** INT-0003 AC-11; test-plan E2E `checkpoint_runs_meet_the_budget`
- **Quote:** "every check of a run finishes within 20 minutes of the run starting"
- **Failure mode:** EARS-vague
- **Why it matters:**
  - A re-run attempt resets the run's start time, while jobs that are not re-run keep their old times. Run `42580692694` shows this.
  - Queue time is outside the project's control.
  - Re-runs restore a warm cache, so samples 2 and 3 are not independent.
- **Suggested response:** fix-in-plan. Define the metric per attempt, report queue time apart, name the cold-cache sample, and consider `timeout-minutes`.

### C-002: The E2E assumes an open dev→main PR that does not exist, and contradicts INT-0003's checkpoint Consequence
- **Where:** test-plan E2E; INT-0003 AC-11 and Consequences
- **Quote:** "they run on the open `dev → main` pull request. If PR #6 is still open, they join it."
- **Failure mode:** e2e-drift
- **Why it matters:**
  - PR #6 is merged, and under the new triggers a push to `dev` starts no run.
  - AC-11 names the checkpoint PR, while the Consequence says CI evidence is recorded by the next sprint.
  - Opening a PR mid-sprint is an owner-visible action.
- **Suggested response:** fix-in-plan. Either open the sprint's PR during Test, with owner approval, or defer the GitHub half to Sprint 6.

### C-003: AC-2's new "on demand" clause has no verification and cannot be exercised this sprint
- **Where:** INT-0003 AC-2; test-plan traceability
- **Quote:** "CI runs on every pull request, on every push to `main`, and on demand."
- **Failure mode:** plan-test-mismatch
- **Why it matters:**
  - `workflow_dispatch` works only from the default branch's workflow.
  - `push_to_dev_starts_no_run` and `superseded_run_is_cancelled` are not mapped to AC-2.
- **Suggested response:** fix-in-plan. Defer a dispatch E2E to Sprint 6, and add the AC-2 rows.

### C-004: T-004's workflow.rs clause contradicts itself and misses the "push to main" half
- **Where:** build-plan T-004
- **Quote:** "`push` triggering on a branch other than `main`; … every problem it reports today."
- **Failure mode:** EARS-vague
- **Why it matters:** Two parts of today's checker must change:
  - its "push does not trigger on dev" problem;
  - `workflow_forms_read_alike`, which asserts that problem.

  The clause also misses three cases:
  - a missing `push` on `main`;
  - a branch-less `push`;
  - wildcard branches.
- **Suggested response:** fix-in-plan.

### C-005: The deny-install mutant has no clause, and the host-run deny and lint jobs need the pinned toolchain
- **Where:** test-plan T-004; build-plan T-004
- **Quote:** "the `deny` install moved to another job"
- **Failure mode:** plan-test-mismatch
- **Why it matters:**
  - `runs_deny_on_ubuntu` accepts a bare `cargo deny check`.
  - The Embark action's acceptance is undecided.
  - The step order (toolchain install before rust-cache and cargo) is unspecified.
- **Suggested response:** fix-in-plan.

### C-006: The `lint` clause is weaker than the `ci` clause, and the E2E check names are not specified
- **Where:** build-plan T-004
- **Quote:** "a `lint` job without both platforms, or without each of the two commands"
- **Failure mode:** EARS-vague
- **Why it matters:**
  - `runs-on: ubuntu-latest` under a two-OS matrix would pass.
  - A constant concurrency group would pass too.
  - The job names the E2E relies on are not in the EARS.
- **Suggested response:** fix-in-plan.

### C-007: The "nothing dropped" test checks discovery against itself
- **Where:** test-plan T-003
- **Quote:** "`repository_units_match_its_test_files`: … 36 test files, 34 names."
- **Failure mode:** missing-risk
- **Why it matters:** Comparing a directory scan against a directory scan cannot find a target that Cargo builds and the scan misses.
- **Suggested response:** fix-in-plan. Use Cargo's own list as the oracle.

### C-008: The hostile-variable check never runs a `--test` unit, and one existing test would silently skip the test step
- **Where:** test-plan T-003, existing tests
- **Quote:** "`hostile_variables_never_reach_a_step` (the `PROBE` workspace now runs its units through the new step)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:**
  - PROBE has no `tests/` file.
  - `test_step_removes_golden_create` skips any step that is not `Action::Cargo`.
- **Suggested response:** fix-in-plan.

### C-009: Where unit output goes is unspecified, and exact-output tests depend on it
- **Where:** build-plan T-003; `run_passes_a_clean_workspace`, `run_with`
- **Quote:** "each unit's standard output and error SHALL be printed whole"
- **Failure mode:** hidden-dep
- **Why it matters:** Four things are unspecified:
  - where unit output goes;
  - the change to `run_with`'s signature;
  - the outcome when the pre-build fails;
  - a summed test count.

  `run_passes_a_clean_workspace` asserts exact output.
- **Suggested response:** fix-in-plan.

### C-010: "No unit rebuilds" is assumed, not checked, and concurrent cargo runs are not examined
- **Where:** build-plan T-003 Notes
- **Quote:** "`--workspace` on every unit keeps feature resolution identical, so no unit rebuilds."
- **Failure mode:** missing-risk
- **Why it matters:**
  - A rebuild would race running executables on Windows.
  - Lock queueing, oversubscription, timing guards and memory on a 4-vCPU runner are not measured.
- **Suggested response:** fix-in-plan. Fail when a unit compiles, and record lock waits, flakes and memory.

### C-011: Optimisation may hide the bug the allocation tests target
- **Where:** research §4; build-plan T-002
- **Quote:** "Every matrix digest, every cell count and the reach table must still match Sprint 4's."
- **Failure mode:** missing-risk
- **Why it matters:** LLVM may delete an unused allocation. A reserving decoder could then pass `rejected_cheaply`.
- **Suggested response:** fix-in-plan. Add a negative control.

### C-012: The profile pin test leaves `[profile.test]` and per-package overrides open, and T-002's Touches is incomplete
- **Where:** test-plan T-002; build-plan T-002 Touches
- **Quote:** "Neither sets `debug-assertions`, `overflow-checks` or anything else"
- **Failure mode:** hidden-dep
- **Why it matters:** Three other profile settings are not covered:
  - `[profile.test]` overrides;
  - per-package overrides;
  - `build-override`.

  `pins.rs` is also not in T-002's Touches.
- **Suggested response:** fix-in-plan.

### C-013: New constraints and consequences live only in sprint prose
- **Where:** build-plan T-002 and T-003; INT-0003 Consequences
- **Quote:** "WHEN a member's manifest declares `[[test]]` or sets `autotests`, THEN the step SHALL fail"
- **Failure mode:** intent-drift
- **Why it matters:**
  - The `[[test]]` ban is a lasting rule.
  - The optimised `dev` profile changes local builds.
  - The "work branch" Consequence is out of date.
  - Cancelling `main` runs conflicts with AC-2's "every push to `main`".
- **Suggested response:** fix-in-plan.

### C-014: "The cargo-deny install stops failing on a bad download" is not what the plan delivers
- **Where:** research §1 and §3; build-plan T-004
- **Quote:** "Whether the official action retries downloads is unconfirmed"
- **Failure mode:** missing-risk
- **Why it matters:** A checksum check makes a failure clearer; it does not stop it.
- **Suggested response:** defer-with-rationale, or confirm the retry behaviour from source.

### C-015: The nextest credit has no automated guard
- **Where:** build-plan T-005; `crates/xtask/src/docs.rs` `CREDITED`
- **Quote:** "`crates/xtask/src/docs.rs` (only if a README check needs the new wording)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Every other credit is enforced by `prior_art_credits_every_listed_system`.
- **Suggested response:** fix-in-plan.

## Confidence
block

## Responses

| Concern | Response |
|---|---|
| C-001 | **fix-in-plan.** AC-11 now defines the metric. Each sample is one attempt in which every job runs: the first is the PR's own run with a cold cache, then two "re-run all jobs" attempts with a warm one. The time is measured from the attempt's first job start to its last job end, so it excludes queue time, which is reported separately. Every job also gets `timeout-minutes`. |
| C-002 | **fix-in-plan, by owner decision (2026-11-09).** During Test, `dev` is pushed and the sprint's `dev → main` PR opens early. It is the PR the Loop checkpoint uses. AC-11 now names "the sprint's pull request". The Consequence is reworded: runs of the PR's tested head are recorded in the sprint, and later runs (close, record checkpoint, the merge into `main`) in the next. PR #6's merge run is added to T-001. |
| C-003 | **fix-in-plan.** A dispatch run on `dev` is deferred to Sprint 6, once `main` has the trigger: a new backlog task in T-005 and an E2E marked not yet possible. `push_to_dev_starts_no_run` and `superseded_run_is_cancelled` now trace to AC-2. |
| C-004 | **fix-in-plan.** T-004 lists the exact problem set. "push does not trigger on dev" is dropped. Added problems: `push` does not trigger on `main`; `push` triggers on another branch or a pattern; `push` has no branch list. `workflow_forms_read_alike` is updated, and each new problem has a mutant. |
| C-005 | **fix-in-plan.** The Embark action is no longer accepted. The `deny` job must run `taiki-e/install-action@…` with `tool: cargo-deny@0.20.2` before `cargo deny check`, in its own steps. Step order is fixed for every job: checkout, `rustup toolchain install`, rust-cache (lint and ci), then commands. Mutants cover the install being absent, in another job, and the Embark action. |
| C-006 | **fix-in-plan.** `lint` uses the `ci` job's predicate: both platforms, `runs-on: ${{ matrix.os }}`, no `include` or `exclude`, no `if:`. The concurrency group must contain `github.ref`. The job names are in the EARS and the pin. Mutants added: `lint` with `runs-on: ubuntu-latest`, `lint` with `if:`, and a constant group. |
| C-007 | **fix-in-plan.** Cargo is the oracle at run time. The pre-build's `Executable …` lines list every test executable Cargo built. The step fails if any of them is not reported `Running …` by some unit, or if a non-doc unit runs nothing. A target that discovery misses, `[[test]]` included, fails `cargo ci` loudly. The manifest ban is dropped (C-013a). |
| C-008 | **fix-in-plan.** `clean_workspace` gains `tests/probe.rs` with the same assertions, so a `--test probe` unit runs. `test_step_removes_golden_create` asserts over every command the test step builds. |
| C-009 | **fix-in-plan.** The execute closure receives the step writer (`FnMut(&Step, &mut dyn Write) -> Outcome`). The test step captures unit output and writes each block to that writer; other cargo steps keep inherited output. A failed pre-build fails the step without running units, printing its output. The step ends with one total line, `test: <u> units, <p> passed, <f> failed, <i> ignored`. `run_passes_a_clean_workspace` is updated to check the PASS lines in order. |
| C-010 | **fix-in-plan.** A unit whose output contains a `Compiling ` line fails the step as a rebuild. Integration records "Blocking waiting for file lock" lines, any timing-guard failures, and peak working-set memory on Windows via a sampled `Get-Process` maximum. GitHub has no memory probe here, so a runner OOM would show as a failed job. |
| C-011 | **fix-in-plan.** T-002 adds `a_reservation_on_the_error_path_is_measured` to `codec/tests/alloc.rs`: a decode-shaped function that reserves `COUNT` elements and then fails must show `bytes_max ≥ COUNT × 8` under the optimised profile. If LLVM removed such allocations, the control would fail. |
| C-012 | **fix-in-plan.** The pin test asserts that the root manifest's `[profile]` holds exactly `dev.opt-level = 1` and `dev.package."*".opt-level = 3`, and nothing else: no `test`, `release` or `build-override` entries and no other package. `pins.rs` and `alloc.rs` are added to T-002's Touches. |
| C-013 | **fix-in-plan.** (a) The ban is dropped (C-007). (b) The optimised `dev` profile, with slower local compiles and optimised debug builds, is now an INT-0003 Consequence. (c) The CI-evidence Consequence is reworded. (d) `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`, so pushes to `main` are never cancelled; AC-2 says superseded pull-request runs may be. |
| C-014 | **fix with evidence.** install-action's own `main.sh` downloads with `curl … --retry 10` inside its `retry` helper (lines 20 and 67, read on 2026-11-09). With the checksum check, a bad download is retried, and one that stays bad fails clearly. The research goal is reworded to match. |
| C-015 | **fix-in-plan.** `nextest` joins `CREDITED` in `crates/xtask/src/docs.rs`, which T-005 now always touches. The credit traces to `prior_art_credits_every_listed_system`. |
