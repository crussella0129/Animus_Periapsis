# Plan Critique — Sprint 5

Round 2. The same read-only subagent reviewer re-reviewed the plans at `f59d1ea` against its round-1 concerns ([critique-round1.md](critique-round1.md)). It found all 15 responses in the build plan, test plan, INT-0003 and research report. It also verified that `f0910e6` is `main`'s merge commit, and that 478 and 479 match Sprint 4's report plus the CR-07 test. The concerns below are new or partial; the responses are at the end.

## Concerns

### C-001: Captured unit output removes the evidence `hostile_variables_never_reach_a_step` checks
- **Where:** `build-plan.md` T-003; `test-plan.md` T-003 existing tests
- **Quote:** "`run_passes_a_clean_workspace`: checks that the six `PASS` lines appear in order and that the run ends `ci: all 6 steps passed`."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The parent test proves PROBE ran through `test inherits_no_stripped_variable ... ok` on the child's stdout. Unit blocks now go to the step writer, a `Vec` in the child, so that line disappears. One shared test name also cannot show that both units ran.
- **Suggested response:** fix-in-plan.

### C-002: Cargo's terminal settings can change or hide the lines the coverage and rebuild checks parse
- **Where:** `build-plan.md` T-003; `ci.rs` `SCRUBBED_PREFIXES`
- **Quote:** "a unit's output contains a `Compiling ` line, meaning it rebuilt something."
- **Failure mode:** missing-risk
- **Why it matters:** `CARGO_TERM_QUIET`, `CARGO_TERM_VERBOSE` and `CARGO_TERM_COLOR`, or `term.*` in a user's config, can each do one of these:
  - hide the `Running` lines;
  - reshape them;
  - put ANSI codes inside `Compiling`.

  The unit tests use fake names, not literal Cargo lines.
- **Suggested response:** fix-in-plan. Set the three variables on the step's commands, add literal-line parser tests, and check that `--doc` does not trip the rebuild check.

### C-003: Without `--no-fail-fast`, one failing executable is misreported as "unrun" and hides later failures
- **Where:** `build-plan.md` T-003
- **Quote:** "an executable the pre-build reported on an `Executable … (<path>)` line was not reported on a `Running … (<path>)` line by any unit"
- **Failure mode:** EARS-vague
- **Why it matters:** `cargo test` stops at the first failing executable. A unit that spans several executables would then hide the later failures, and its skipped executables would be reported as unrun.
- **Suggested response:** fix-in-plan.

### C-004: INT-0003 AC-2's "a push to `main` is never cancelled" is not what GitHub concurrency guarantees
- **Where:** INT-0003 AC-2; `build-plan.md` T-004
- **Quote:** "a push to `main` is never cancelled."
- **Failure mode:** intent-drift
- **Why it matters:** A group keeps at most one pending run, so a newer pending push run replaces an older pending one.
- **Suggested response:** fix-in-plan. Make the group per commit for runs that are not pull-request runs, or reword AC-2.

### C-005: The AC-11 metric still counts runner queue time, and the "cold cache" sample is assumed, not recorded
- **Where:** INT-0003 AC-11; `test-plan.md` E2E
- **Quote:** "which leaves out time spent waiting for a runner" / "the pull request's first run, with a cold cache"
- **Failure mode:** EARS-vague
- **Why it matters:**
  - The window still contains any wait for a runner after the first job starts.
  - An earlier pull-request run of the same key could warm sample 1.
- **Suggested response:** fix-in-plan.

### C-006: The allocation control may fail at `opt-level = 1`, and the plan has no response ready
- **Where:** `build-plan.md` T-002; `test-plan.md` `a_reservation_on_the_error_path_is_measured`
- **Quote:** "a function shaped like a decoder reserves `Vec::<u64>::with_capacity(COUNT)` and then returns an error without using it."
- **Failure mode:** missing-risk
- **Why it matters:**
  - LLVM may remove an unused allocation.
  - The obvious fix is forbidden by T-002's own pin.
  - A control with an optimisation barrier would pass by construction.
- **Suggested response:** fix-in-plan.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---|---|
| C-001 | **fix-in-plan.** `tests/probe.rs` uses its own test name, `probe_inherits_no_stripped_variable`. `run_passes_a_clean_workspace` asserts that its captured output holds `test inherits_no_stripped_variable ... ok` and `test probe_inherits_no_stripped_variable ... ok`, then prints that output to stdout. `hostile_variables_never_reach_a_step` asserts both lines on the child's stdout. |
| C-002 | **fix-in-plan.** The pre-build and every unit command set `CARGO_TERM_QUIET=false`, `CARGO_TERM_VERBOSE=false` and `CARGO_TERM_COLOR=never`, which override the environment and `term.*` config alike. New parser tests use literal lines from both systems: `Executable unittests src\lib.rs (target\debug\deps\codec-….exe)`, `Running tests/crash.rs (target/debug/deps/crash-…)`, a `Running unittests` line, and a coloured `Compiling` line, which must still be found. The `--doc` unit runs alone, after the parallel units, and is exempt from the rebuild check: rustdoc may build a library that no test target links. That keeps it from touching artefacts while executables run. |
| C-003 | **fix-in-plan.** Every unit command has `--no-fail-fast`. The unrun-executable check reports only when every unit passed, so a failure is never misreported as a discovery miss. |
| C-004 | **fix-in-plan.** The group is `ci-${{ github.ref }}-${{ github.event_name == 'pull_request' || github.sha }}`. Pull-request runs share one group per pull request, and each push to `main` gets a group of its own, so no other push can cancel or replace it. AC-2 now says "a run for a push to `main` is never cancelled or replaced by a later push". `workflow.rs` also reports a group without `github.sha`, with a mutant. |
| C-005 | **fix-in-plan.** AC-11 is reworded to measure each job's own `started_at` to `completed_at`. A sample passes when its slowest job takes at most 20 minutes, so time spent waiting for a runner is excluded. Queue time is reported separately. The cold-cache claim is dropped from AC-11. Each sample records rust-cache's restore line ("Cache restored from key" or "No cache found"), so warm and cold are observed, not assumed. Cancelled runs are never samples. |
| C-006 | **fix-in-plan.** The control uses no optimisation barrier: no `black_box`, no `#[inline(never)]`, no volatile access. If it fails under `opt-level = 1`, the decided response is a reviewed `[profile.dev.package.codec] opt-level = 0`. The pin test then allows exactly that third entry, and INT-0003's Consequences record that `codec` and its allocation tests run unoptimised, at a cost in speed. Either outcome is recorded in the test report. |
