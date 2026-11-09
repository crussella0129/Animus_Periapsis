# Plan Critique — Sprint 4

Round 2. The same read-only subagent reviewer re-reviewed the plans at `b51c473` against its round-1 concerns ([critique-round1.md](critique-round1.md)).
- **Landed in full:** most round-1 responses, as claimed: C-002 to C-005, C-008 to C-010, C-013, C-016 to C-019 and C-021.
- **Landed in part:** C-006 and C-007.
- **New problems:** the forced race, the reach recording and the `cargo reach` change introduced the ones below.

The concerns are condensed here; the responses are at the end.

## Concerns

### C-001: Crashed odd-seed cells will end in ordering deadlocks
- **Where:** T-005, the orderings and crash clauses; T-007, the odd-seed orderings
- **Failure mode:** hidden-dep
- **Why it matters:** A crash while process 0 is held stops process 2 before its releasing point, so process 0 waits forever. A crash before process 0's rename strands process 2 at its listing. These are routine cells, and T-005 contradicts itself ("finish" versus "deadlock error").
- **Suggested response:** fix-in-plan. Release every wait on a crash and on a predecessor whose process has ended. Keep the deadlock error for live processes only, and test both releases.

### C-002: Process 3 can take process 0's op out of process 2's parents
- **Where:** T-007, the scenario; `writers_forced_race_runs`
- **Failure mode:** EARS-vague
- **Why it matters:** An unordered process 3 can commit its second op naming process 0's op before process 2 lists. Process 0's op is then no longer a head, and the forced race fails.
- **Suggested response:** fix-in-plan.

### C-003: Other writers can make process 0's op durable during the hold
- **Where:** T-007, the forced race and `UnsyncedReferences`
- **Failure mode:** missing-risk
- **Why it matters:** During the hold, another process's `open`, `ops/` sync or parent re-sync makes the rename durable. The control is then left to chance, and the test doesn't check durability as the old `install_race` did.
- **Suggested response:** fix-in-plan.
  - Order processes 1 and 3 to finish before process 0's `temp_written`.
  - Assert the op is not durable at the listing.
  - Count the cells where it is still not durable at process 2's rename.

### C-004: The test-local listing point will fail `cargo reach`
- **Where:** T-007, T-008
- **Failure mode:** hidden-dep
- **Why it matters:** `reach()` reports every counted name, so the unregistered `test.writers.listing` fails `cargo reach` on every run.
- **Suggested response:** fix-in-plan. Reserve a `test.` prefix that recording drops.

### C-005: `store.commit_op.present_synced` appears unreachable by any designated scenario
- **Where:** T-006, T-008
- **Failure mode:** plan-test-mismatch
- **Why it matters:** No scenario re-commits an op that is already present, so `cargo reach` would fail.
- **Suggested response:** fix-in-plan. Name the scenario that reaches each point, or leave the point unplanted.

### C-006: The "which runs count" rule doesn't match the code
- **Where:** T-008
- **Failure mode:** EARS-vague
- **Why it matters:** In Materialise, resume runs on the same copy the oracle then uses. Snapshot has no resume: its next command is `init` and `snapshot`.
- **Suggested response:** fix-in-plan. Read reach right after the modelled command returns, and name that command per scenario.

### C-007: `cargo reach` still clashes with INT-0003's Intent prose, `readme_problems`, and the `xtask` description
- **Where:** T-010; INT-0003 Intent; `crates/xtask/src/docs.rs:62, 122-131`
- **Failure mode:** hidden-dep
- **Suggested response:** fix-in-plan.

### C-008: The cost note is short of the real run count, the budget has no end state, and parallel seeds need canonical ordering
- **Where:** T-007 Notes
- **Failure mode:** missing-risk
- **Suggested response:** fix-in-plan.
  - Share one correct run.
  - State the terminal step.
  - Order cells canonically before digesting.

### C-009: The registered units don't all fit INT-0005's definition of a unit
- **Where:** INT-0005 AC-6 Note; T-006
- **Failure mode:** intent-drift
- **Why it matters:** `workspace::record` is a module, and `workspace::apply` is `materialise::apply`.
- **Suggested response:** fix-in-plan.

## Confidence
block

## Responses

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan. T-005 now:<br>• releases a wait when the system crashes (the waiter's next syscall fails) or when the predecessor's process has ended without reaching the point;<br>• keeps the deadlock error only for live processes waiting on live processes, as a test bug and never a matrix outcome;<br>• adds `(p, End)` predecessors.<br>New tests: `orderings_are_released_by_a_crash`, `orderings_are_released_when_the_source_process_ends` and `orderings_wait_on_a_process_end`. |
| C-002 | fix-in-plan. In odd seeds, processes 1 and 3 must end before process 0's first `store.commit_op.temp_written`, so neither can take process 0's op out of the heads. |
| C-003 | fix-in-plan.<br>• **Orderings:** processes 1 and 3 end, and process 2's first commit finishes its `ops/` sync, before process 0's first `temp_written`, so no other `open` or `ops/` sync happens during the hold.<br>• **Tests:** `writers_forced_race_runs` asserts that process 0's op name is not durable at process 2's listing. The count of odd cells where it is still not durable at process 2's rename is printed per variant, and must be non-zero for `UnsyncedReferences`.<br>• **Kills:** the odd-seed kill therefore leaves an unsynced op that a later process names. |
| C-004 | fix-in-plan. The `test.` prefix is reserved: such points are never registered and are dropped when reach is recorded (`reach_drops_test_points`). |
| C-005 | fix-in-plan.<br>• **Retry:** process 1 commits its first op twice, as a retry after a lost acknowledgement, which reaches `store.commit_op.present_synced` (`writers_retry_reaches_present`).<br>• **Expected scenarios:** T-006 lists the expected scenario for every registered point.<br>• **Unreached points:** a build run that finds a point unreached extends a new scenario or removes the point, recorded in the build notes. No Sprint 3 scenario changes. |
| C-006 | fix-in-plan.<br>• **Where reach is read:** from the scenario's own `SimFs` after its run, and from the recovered copy right after the modelled command returns, before any oracle check.<br>• **The modelled commands:** `init` and `snapshot` for Snapshot; the resuming `open` for Materialise; none for store and both writer scenarios. |
| C-007 | fix-in-plan.<br>• **INT-0003:** its Intent paragraph now names `cargo reach`, recorded in Transition history.<br>• **T-010:** now touches `crates/xtask/src/docs.rs` (the four-command `NO_FLAGS` sentence, and a check that `cargo reach` is described) and `crates/xtask/Cargo.toml` (the description). The README's crate table names all four commands. |
| C-008 | fix-in-plan.<br>• **Two runs:** the correct matrix runs twice, one run shared through a once-initialised cache.<br>• **Ordering:** cells are ordered by (mode, seed, kill setting, crash step) before digesting.<br>• **Budget:** recorded, not asserted. If all three controls still miss it, it is raised to the measured time, and the raise is recorded in the test report and the test critique's responses. |
| C-009 | fix-in-plan. INT-0005 AC-6's note now reads "a function or module". The units are `workspace::record` and `workspace::materialise` (modules). |
