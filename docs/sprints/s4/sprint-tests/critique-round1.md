# Test Critique — Sprint 4

Round 1. A read-only subagent wrote it from the bundle's `prompts/test-critic.md`, against the result files as first drafted (`bd8dc8e`) at tested head `5dfb418`. It is saved as written, apart from formatting. The responses are at the end, and the fixes are in `9fa59fb`. The final round is [critique.md](critique.md).

## Concerns

### C-001: The panic test's unwind assertion can never fail
- **Where:** `unit-tests.md` T-005 "Panics"; `crates/sim/tests/processes.rs` `panic_in_a_process_ends_the_run`; build-plan T-005 EARS on panics.
- **Quote:** build plan: "every other process **SHALL** be unwound at its next boundary with the kill marker". Test: `assert!(fs.syscalls() < 150);`
- **Failure mode:** weak-assertion
- **Why it matters:** Process 1 panics after its 4th `create_dir`. Processes 0 and 2 make at most 50 syscalls each, so the most the run can reach is 4 + 50 + 50 = 104. That is always below 150, so the assertion passes even when no other process is unwound at all. The `start.elapsed() < 30 s` guard runs only after `run` returns. A deadlock would therefore hang the test binary (and `cargo ci`) instead of failing it. The test proves only that the first panic is returned. The "unwound at its next boundary" promise is unproved.
- **Suggested response:** tighten-assertion. Assert a bound that only unwinding can meet: for example, syscalls ≤ 4 + (one more boundary per surviving process), or record each process's last `i` and assert that each survivor stopped within one boundary of the panic. Run the call under a watchdog thread that fails the test after a fixed time, so a deadlock fails rather than hangs.

### C-002: A wait on a process that panicked is never tested
- **Where:** `test-plan.md` T-005 `orderings_are_released_when_the_source_process_ends`; `processes.rs` lines 297–337; build-plan T-005 EARS.
- **Quote:** plan: "a process waiting on a point that a returned, killed or panicked process never reached is released". Test: `for (seed, how) in (0..2).enumerate()`, which covers only returned (`how 0`) and killed (`how 1`).
- **Failure mode:** negative-path
- **Why it matters:** The panicked case is the one most likely to deadlock: a waiter blocked in `wait_turn` while the source process unwinds. No test combines an ordering wait with a panic. `panic_in_a_process_ends_the_run` sets no orderings. The T-005 completion entry says "(returned and killed)" but does not record that the panicked case was dropped.
- **Suggested response:** add-test. A process waits on an ordering whose source process panics before reaching A. Assert `Stopped::Panicked` with that payload, under the watchdog from C-001. Also pin one seed per case in which the waiter is provably blocked before the source ends, for example by having the waiter's point come first and asserting the order from a log.

### C-003: `point_events_are_ordered` does not test kill-before-wait, and count-before-action depends on the seed
- **Where:** `unit-tests.md` T-005 "Order at a point"; `processes.rs` `point_events_are_ordered`; build-plan T-005 EARS steps 1 to 4.
- **Quote:** "the count, then the kill, then the ordering wait, then the actions"
- **Failure mode:** weak-assertion
- **Why it matters:** The killed process (process 1) has no ordering on it, so swapping steps 2 and 3 cannot change this test's result. The `"action, count Some(2)"` check shows "wait before action" only if process 0 arrives at `p` before process 1. With the single seed `4`, nothing asserts which process arrives first. Kill-before-wait matters in practice: the odd-seed kill cells kill process 0 on arrival at a point where it is also an ordering successor.
- **Suggested response:** add-test or tighten-assertion. Kill a process at a point where it is also the successor of an unsatisfied ordering, and assert that it reports `Killed` with no deadlock and no wait. Log the arrival order and assert that process 0 arrived first, or loop seeds and require at least one such seed.

### C-004: The `WriteInPlace` control is caught only by `torn`, which the EARS list of semantic kinds leaves out, and "atomicity 0" is never computed
- **Where:** `integration-tests.md` four-writer controls table; `crates/store/tests/writers.rs` `caught`; `common/matrix.rs` `Violations::semantic`; `common/writers.rs` `check`; build-plan T-007 EARS; INT-0005 AC-2.
- **Quote:** EARS: "**SHALL** produce semantic violations (closure, durability, heads or atomicity) … `WriteInPlace`: at least one". Log: "writers write-in-place Strict: closure 0 durability 0 heads 0 atomicity 0 torn 6758".
- **Failure mode:** EARS-coverage
- **Why it matters:** `semantic()` is `closure + durability + heads + atomicity + torn`. As the EARS clause is worded, `writers_catch_write_in_place` passes on a kind the clause does not list. Separately, the four-writer `check` never increments `atomicity` (INT-0005 AC-2: the recovered op chain is a prefix), yet every summary line prints "atomicity 0". A reader can take that as checked and clean. In this scenario closure probably implies the per-process prefix, because each later op names its predecessor, but no document says so.
- **Suggested response:** defer-with-rationale. Record in the test report that `torn` has counted as semantic since Sprint 1 (the Sprint 3 store matrix's `WriteInPlace` is also torn-only). Also record that atomicity is subsumed by closure for the four-writer scenario, or compute it and stop printing an unchecked kind as 0.

### C-005: `snapshotting_commands_return_reports` cannot tell a fresh report from one left over by the previous command
- **Where:** `unit-tests.md` T-009; `crates/workspace/tests/history.rs` `snapshotting_commands_return_reports`.
- **Quote:** `let expected = ws.status().unwrap().reported; … ws.new_change().unwrap(); assert_eq!(ws.reported(), expected);`
- **Failure mode:** weak-assertion
- **Why it matters:** `Workspace::reported` is a field that persists across calls on one `Workspace`, and it is cleared only inside `snapshot()`. If `new_change`, `log`, `undo` or `restore` stopped snapshotting, the field would still hold the entry from the earlier `status`, and every assertion would pass. With one entry, the "sorted as `status` returns it" part is also untested.
- **Suggested response:** tighten-assertion. Run each command on a fresh `Workspace` over the same filesystem, or clear and check `reported()` between commands. Plant two raw names, so the order is checked.

### C-006: Three of the four failure kinds in the T-009 EARS have no test
- **Where:** build-plan T-009 EARS; `test-plan.md` `failed_undo_carries_reports`, `praps_failed_undo_reports_skipped_names`; `e2e-tests.md` `praps_commands_report_skipped_names`.
- **Quote:** "**WHEN** such a command fails after its snapshot (`NothingToUndo`, hazards, a blocking entry, an unknown op), **THEN** its error **SHALL** carry the same entries."
- **Failure mode:** negative-path
- **Why it matters:** Only `NothingToUndo` is tested: in the unit test, and through `praps undo` exiting 1. A failed `restore` (unknown op, hazard or blocking entry) is never shown to keep its report or print it on the CLI with exit 1. The design change, from "error carries" to the `reported()` accessor, is recorded in the T-009 completion entry. The behavioural promise "on success and on failure" is evidenced for one command and one error only.
- **Suggested response:** add-test. Add a unit case and a CLI case for `restore <unknown>` with a raw name present: exit 1, stdout holds only the report line, and stderr names the error. Add one hazard or blocking-entry failure as a unit case.

### C-007: In the odd-seed kill cells, nothing asserts that a later process sees the dead process's op
- **Where:** build-plan T-007 "The kill"; `writers.rs` `writers_kills_happen`; `integration-tests.md` kills.
- **Quote:** "In odd seeds, process 0 is killed on arrival at its held `store.commit_op.renamed`. A later process then sees the dead one's unsynced op."
- **Failure mode:** EARS-coverage
- **Why it matters:** `writers_kills_happen` asserts only `killed[process]`. `writers_forced_race_runs` runs without a kill. In the matrix, nothing shows that process 2 lists and names the killed process 0's op, which is the process-crash Consequence in multi-writer form. The unit test `killed_process_state_stays_visible` covers the model, but not this scenario.
- **Suggested response:** add-test. In every odd seed's uncrashed kill run, assert `log.names_op0` and that `at_listing == Some((true, false))`. Then assert that the matrix's correct cells with that kill keep closure.

### C-008: `policy_points_rule` checks only a message fragment, not that the point and file are named
- **Where:** `unit-tests.md` T-004; `crates/xtask/src/policy.rs` `policy_points_rule`; build-plan T-004 EARS.
- **Quote:** unit-tests.md: "eleven fixtures fail, each naming the point or call and the file". Test: `problems.iter().any(|problem| problem.contains(expected))` with fragments such as `"is not \`store.function.stage\`"`, `"an entry not of the form"`, `"\`::point(\`"` and `"in test code"`.
- **Failure mode:** weak-assertion
- **Why it matters:** The EARS clause requires each failure to "name the point and file". No fixture asserts the file path, and five of the eleven do not assert the point name. The Windows probe shows file naming for the unregistered case only.
- **Suggested response:** tighten-assertion. Assert that each failing fixture's problem contains both the fixture's file path (`crates/store/src/lib.rs` or `points.rs`) and the offending name or call text.

### C-009: The retry evidence covers less than the docs say, and one of its assertions cannot fail
- **Where:** `integration-tests.md` "`writers_retry_reaches_present`: 196 cells crashed during the retry"; `writers.rs` lines 176–203.
- **Quote:** `for seed in 0..4 { … run_cell_logged(Variant::Correct, *mode, seed, None, crash_at) … assert!(log.acked.contains(&op1));`
- **Failure mode:** evidence-drift
- **Why it matters:** The 196 cells come from 4 seeds × 2 modes with no kill, not from the 40-seed matrix, and the doc does not say so. `retry_started` is set only after the first commit of op 1 is acknowledged, so `acked.contains(&op1)` is always true. The real check is `durability == 0`. The log and the outcome also come from two separate runs, `run` and then `run_cell`, so the result depends on determinism.
- **Suggested response:** tighten-assertion. State the seed and kill scope in the report. Check op 1's presence directly in the recovered ops from the same run as the log. Either drop the tautological assertion or replace it with the presence check.

### C-010: Several result claims attribute coverage to the wrong test, or to a test whose content differs from the plan
- **Where:** `unit-tests.md` "By build task" and T-008; `test-plan.md` T-003, T-005, T-006, T-008.
- **Quote:** "Every test the test plan names exists and passed on both systems, with one exception." Also: "`ci_steps_are_ordered_and_locked`, extended: … the reach directory cleared by `prepare()` before `test`; `PERI_WRITER_CHILD` scrubbed."
- **Failure mode:** evidence-drift
- **Why it matters:**
  - `ci_steps_are_ordered_and_locked` checks only the six steps. The clearing is in `test_step_clears_the_recorded_reach`, which no result doc names. The scrubbing is in the variable-stripping tests.
  - `reach_fails_on_an_unreached_point` and `reach_fails_on_an_unregistered_point` call the pure `table()`, so "writes nothing" is not asserted for them.
  - `reach_is_idempotent` calls a pure function twice. The real idempotence evidence is `int_reach_*`.
  - Further deviations are not covered by the "one exception":
    - `processes_return_results_in_input_order` has no panic. This one is recorded in T-005.
    - `points_do_not_change_traces` uses `every_kind`, not "the contract scenario".
    - `workspace_points_mark_each_stage` uses `restore_view`, not `undo`.
    - `orderings_are_released_when_the_source_process_ends` has no panicked case (C-002).
  - The claim that "The T-009 completion entry records the change" for the six CLI tests is loose. The entry describes the single test but does not say it replaces six planned names.
- **Suggested response:** tighten-assertion. Correct the attributions in `unit-tests.md`, and list each content deviation from the plan with the test that covers it instead. Make one failure test call `run()` and check that `docs/reach.md` is absent. Make `reach_is_idempotent` run `run()` twice and compare the file bytes.

### C-011: No test checks per-scenario point expectations or the scheduler's draw sites
- **Where:** build-plan T-006 "each registered point's expected scenario **SHALL** be as listed"; T-005 "A draw happens at each of these: every syscall boundary and point boundary …; its return; its kill; its wait on an ordering."
- **Quote:** "four writers: `store.commit_op.present_synced`, through process 1's retry"
- **Failure mode:** EARS-coverage
- **Why it matters:** The reach gate checks only that some scenario reaches each point, and the table is per unit. If the four-writer retry stopped reaching `commit_op.present_synced`, the gate would still pass while another scenario hits it (`writers_retry_reaches_present` checks one seed only). No test shows that a draw happens at a point boundary, a return or a kill. The only guard is the four-writer digest constant, which was taken from the implementation.
- **Suggested response:** defer-with-rationale, or add-test. Either assert the plan's expected points against each scenario's reach file in `reach_table_shows_the_triangle` or a sibling test, or record the clause as checked only in aggregate. Add a small scheduler test showing that a point alone, with no syscall, lets another process run under some seed.

### C-012: The `xtask` description names three commands, where T-010's EARS says four
- **Where:** build-plan T-010; `crates/xtask/Cargo.toml` description; README crate table; T-010 completion entry.
- **Quote:** EARS: "the `xtask` crate's description and its row in the README's crate table **SHALL** name all four commands". Description: "Project tooling behind the cargo ci, cargo policy and cargo reach aliases".
- **Failure mode:** evidence-drift
- **Why it matters:** `cargo book` is an alias of the `book` package, not of `xtask`, so three is arguably correct. Still, the locked EARS clause is not met as written, and the completion entry ("names the three aliases") does not flag this as a departure.
- **Suggested response:** reject. The critique is wrong if `book` is out of `xtask`'s scope by design, but the test report should record that departure explicitly.

## Confidence
block

## Responses

All but C-004 and C-012 changed tests or code. The fixes are in `9fa59fb`, and both legs ran again in full at that head ([test-matrix-log.md](test-matrix-log.md)).

| Concern | Response |
|---------|----------|
| C-001 | **Fixed.** `panic_in_a_process_ends_the_run` now runs 10 seeds. Each process counts any `create_dir` that completes after process 1 has set its panic flag, and the test requires that count to be 0. Only unwinding at the next boundary can meet that. The test runs under a `within` watchdog of 60 s, so a deadlock fails instead of hanging. A mutant that stops survivors from unwinding fails it: "seed 0: a syscall completed after the panic". |
| C-002 | **Fixed.** `orderings_are_released_when_the_source_process_ends` covers a source that returns, is killed, and panics. Each case runs over 20 seeds under the watchdog. For the panic, it requires `Stopped::Panicked` with the source's payload and no deadlock. In every case, at least one seed must log process 0 arriving at its held point before the source ends, so it was provably waiting. |
| C-003 | **Fixed, with a new test.** `a_kill_comes_before_an_ordering_wait`: process 0 is killed on arrival at `p`, where an ordering would make it wait for process 1's end, while process 1 waits for a later point of process 0's. Were the wait first, every seed would deadlock. It passes in 20 seeds. A mutant that moves the kill after the wait fails it. `point_events_are_ordered` now runs 20 seeds and logs arrivals. In a seed where process 0 arrives first, the log must be: arrive, arrive, `action, count Some(2)`, then past `p`. At least one seed must be of that kind. |
| C-004 | **Deferred, with the rationale recorded in the test report.** `torn` has counted as semantic since Sprint 1, and the store matrix's `WriteInPlace` control is torn-only in every sprint. The four-writer `check` does not compute `atomicity`, so its 0 is not a finding. Each process's ops form a chain, each naming its predecessor, so closure covers the prefix property. The report says so. Printing an unchecked kind as 0 is a fault in the summary line; the same `Violations` type is shared with the Sprint 1 to 3 matrices, so the line is left alone to keep their printed output unchanged. |
| C-005 | **Fixed.** Each of `new_change`, `log`, `undo` and `restore` runs on a freshly opened `Workspace`, and the test asserts it has reported nothing before the command. Two raw names make the order checked against `status`'s. |
| C-006 | **Fixed.** `failed_commands_carry_reports` replaces `failed_undo_carries_reports`. It covers `NothingToUndo`, and `restore` failing with an unknown op, a hazard (case folding) and a blocking entry. Each failure carries the one entry. `praps_commands_report_skipped_names` adds `praps restore 0000000000000000`: exit 1, stdout only the report line, and stderr "no op matches". |
| C-007 | **Fixed.** `writers_kills_happen` now asserts, for every odd seed's uncrashed kill run in both modes, that the killed process is process 0, that process 2 named its op, and that the op was visible and not durable at the listing. Closure for those cells is part of `writers_matrix_finds_no_violations`, which has 0 violations. |
| C-008 | **Fixed.** Each failing fixture must yield one problem that starts with the file, names the offending point, unit or call, and has the rule's text. The policy rule now names registries by their path from the root (`crates/store/src/points.rs`). The registry's own parse error still names `store/src/points.rs`, because `points::registries` has no root. |
| C-009 | **Fixed.** `run_cell_logged` now makes one run and returns its outcome, its log and the ops present after recovery; `run_cell` uses it. The `Outcome` is unchanged, so the digest is too. The retry test asserts that process 1's op is present after recovery, which replaces the assertion that could not fail. The result documents now give its scope: seeds 0 to 3, both modes, no kill. |
| C-010 | **Fixed.** `unit-tests.md` now names `test_step_clears_the_recorded_reach` and the variable-stripping tests, and lists each content deviation from the plan with the test that covers it. `reach_fails_on_an_unreached_point` and `reach_fails_on_an_unregistered_point` also run `cargo reach` on a fixture workspace: exit 1, the item named, `docs/reach.md` absent. `reach_is_idempotent` runs `run()` twice, compares the written bytes, and then passes `check`. |
| C-011 | **Fixed, both parts.** `cargo reach` now checks each scenario against the build plan's T-006 list (`reach::EXPECTED`). A scenario that stops reaching a point expected of it fails the table and the `reach` step of `cargo ci`, even when another scenario reaches the point (`reach_fails_on_an_expected_point_missed`). `expectations_are_real` keeps the list naming real scenarios and points, and covering all 21. Draw sites: `a_point_alone_lets_another_process_run` shows another process running between two halves of a process separated only by a point. Draws at a return and at a kill are what let any run with several processes finish, so every multi-process test exercises them. |
| C-012 | **Rejected, and recorded.** `cargo book` is an alias of the `book` package, not of `xtask`. The `xtask` description and its README row name the three commands it serves; the README's four-command sentence and the `NO_FLAGS` check cover all four. The test report records this as a departure from the T-010 clause's wording. |
