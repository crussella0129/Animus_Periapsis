# Plan Critique — Sprint 4

Round 1. A read-only subagent reviewer wrote it from the bundle's `prompts/plan-critic.md`. It read the draft plans at `9802ace`, the research report, the cited intents, and the code they change. The concerns are condensed here; the responses are at the end.

## Concerns

### C-001: The four-writer matrix will probably exceed the CI budget, and the fallback breaks INT-0005 AC-2
- **Where:** `build-plan.md` T-006, EARS and Notes
- **Quote:** "If the run exceeds 10 minutes on either system, the seed count is reduced"
- **Failure mode:** missing-risk
- **Why it matters:**
  - **Run length.** A run is closer to 250–300 syscalls than 120. That counts each process's `open`, three puts per state and the second op's listing.
  - **Repeat runs.** The matrix runs about 8 times: the correct store, two reproducibility runs, and five variants.
  - **The fallback.** Cutting below 40 seeds breaks AC-2.
  - **The budget.** It doesn't say what it covers.
- **Suggested response:** fix-in-plan. Name cost controls that keep 40 seeds, say what the budget covers, and run the variants strict only.

### C-002: `UnsyncedReferences` may never be caught under random scheduling, and the plan names a variant that doesn't exist
- **Where:** `build-plan.md` T-006, the variants clause
- **Quote:** "`PointerFile`, `UnsyncedObjects`, `UnsyncedInit`, `WriteInPlace`, `NoResync`"
- **Failure mode:** missing-risk
- **Why it matters:**
  - **The name.** The variant is `UnsyncedReferences`; there is no `NoResync`.
  - **The odds.** The parent race needs a narrow interleaving in strict mode, and each process's `open` syncs `ops/`. Under a fresh random draw at every boundary, A must be passed over about 15 times in a row, which happens about 1% of the time.
  - **The precedent.** The two-writer scenario exists because random runs missed this race.
  - **The orderings.** They drive no integration scenario.
- **Suggested response:** fix-in-plan. Force the parent race with orderings in part of the seeds, assert that it ran, and fix the name.

### C-003: "At least one violation" lets a control pass without evidence
- **Where:** T-006; `writers_matrix_catches_every_variant`
- **Quote:** "each **SHALL** produce at least one violation"
- **Failure mode:** EARS-vague
- **Why it matters:** `PointerFile` leaves `HEAD`, which counts as `stray` in every cell. Earlier sprints required semantic violations in strict mode, with specific kinds per variant.
- **Suggested response:** fix-in-plan.

### C-004: The four-writer oracle drops the acknowledged-object check and leaves "acknowledged" and "in-flight" undefined
- **Where:** T-006, the oracle list
- **Failure mode:** EARS-vague
- **Why it matters:**
  - A kill can land after `commit_op`'s last sync but before it returns.
  - The heads clause as worded isn't operational.
- **Suggested response:** fix-in-plan. Check acknowledged ⊆ present ⊆ acknowledged ∪ in-flight, check heads against `heads_of(present)`, check acknowledged objects, and keep the log outside the process threads.

### C-005: Many "one kill" cells may never kill anything
- **Where:** T-006
- **Quote:** "The killed process and its step are drawn from the seed."
- **Failure mode:** EARS-vague
- **Why it matters:** A step drawn past the process's last boundary kills nothing.
- **Suggested response:** fix-in-plan. Bound *k*, assert `Killed`, and print the kill count.

### C-006: How point actions re-enter, cross threads and relate to the hook is unspecified
- **Where:** T-002, T-004, T-005 (draft numbering)
- **Quote:** "writer B **SHALL** be fired by `on_point` at A's `rename` point"
- **Failure mode:** hidden-dep
- **Why it matters:**
  - **Re-entry.** B's commit reaches the same point inside the action.
  - **Unanswered questions:**
    - whether the hook runs at points;
    - whether an action's syscalls are switch points;
    - what `recover()` keeps.
- **Suggested response:** fix-in-plan. Specify that an action is not re-entered, that the hook runs only at syscall boundaries, and what `recover()` drops. Add `on_point_action_is_not_reentered`.

### C-007: A default no-op `point` can be silently swallowed by forwarders
- **Where:** T-002; `crates/sim/src/fs.rs:70-166`
- **Failure mode:** missing-risk
- **Why it matters:** A forwarder that doesn't override `point` drops every point that passes through it.
- **Suggested response:** fix-in-plan. Give `point` no default, or test every forwarder.

### C-008: Nothing checks the Sprint 3 digests automatically
- **Where:** T-002, T-005; the digest tests
- **Failure mode:** plan-test-mismatch
- **Why it matters:**
  - The tests only print their digests.
  - The race migration needs the point immediately after `rename` and the same visibility re-check.
- **Suggested response:** fix-in-plan. Assert the digests as constants, and state both migration conditions.

### C-009: Literal-only points clash with the shared `write_file`, and `init` has no such stages
- **Where:** T-003, T-005
- **Failure mode:** EARS-vague
- **Why it matters:**
  - `put_object` and `commit_op` share `write_file`.
  - The already-present and `WriteInPlace` paths skip stages.
- **Suggested response:** fix-in-plan. Name every point, and say where each sits.

### C-010: Text parsing of `.point(` will fail on `sim`'s forwarders and is easy to fool
- **Where:** T-003
- **Failure mode:** EARS-vague
- **Why it matters:** Comments, rustdoc and line breaks would confuse it, and the registry's syntax is unfixed.
- **Suggested response:** fix-in-plan. Exempt `sim`, ignore comments, fix one registry form, and add fixtures.

### C-011: Reach files will flap between parallel tests and variants, and which filesystem's counts go in is undefined
- **Where:** T-007, the first EARS clause
- **Failure mode:** hidden-dep
- **Why it matters:**
  - Resume runs only in the recovered copy.
  - The oracle's own `open` and `init` would count.
- **Suggested response:** fix-in-plan. Use one designated test per scenario, the correct variant only, writes through a temporary file and a rename, and a stated rule for which runs count.

### C-012: `cargo reach --check` breaks INT-0003's flagless-tooling contract
- **Where:** T-007 Notes; INT-0003 AC-9, AC-10
- **Failure mode:** intent-drift
- **Suggested response:** fix-in-plan. Make `cargo reach` a flagless writer, have `cargo ci` compare in-process, guard `cargo reach` like `ci`, and revise INT-0003.

### C-013: The triangle cells and the cleared directory have no named test
- **Where:** T-007; `test-plan.md`
- **Failure mode:** plan-test-mismatch
- **Suggested response:** fix-in-plan. Add a test over the committed table, and extend `ci_steps_are_ordered_and_locked`.

### C-014: AC-6 can be met with zero points, and "unit" is undefined in the intent
- **Where:** INT-0005 AC-6
- **Failure mode:** intent-drift
- **Suggested response:** fix-in-plan. Define "unit", and add a Consequence that each new durable unit registers a point.

### C-015: Points are scheduling boundaries, so adding one later changes the four-writer runs
- **Where:** T-002 versus T-004 (draft numbering)
- **Failure mode:** intent-drift
- **Suggested response:** fix-in-plan. Record this in the spec and in INT-0005's Consequences, with the expected response.

### C-016: T-008 (draft numbering) leaves `log` and the error paths silent
- **Where:** T-008; INT-0004 AC-7
- **Failure mode:** missing-risk
- **Why it matters:** `log` snapshots and drops the report. A failed `undo` says nothing, and `undo` snapshots twice.
- **Suggested response:** fix-in-plan.

### C-017: The real test's heads check is a tautology, and Windows sharing is unaddressed
- **Where:** T-006, the real-filesystem clause
- **Failure mode:** EARS-vague
- **Why it matters:**
  - `heads()` is `heads_of(ops())`, so the check can't fail.
  - Children's parents are unspecified.
  - A leftover child-mode variable could turn a no-op test into a writer.
- **Suggested response:** fix-in-plan. Have children report their ops, compute heads from the reports, give each child distinct objects, and scrub the variable.

### C-018: T-002 (draft numbering) mixes the thread-safety refactor with point semantics
- **Failure mode:** granularity
- **Suggested response:** fix-in-plan. Split the task, gating each part on the digests.

### C-019: Scheduler edge cases are untested or unspecified
- **Where:** T-004 (draft numbering)
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Unspecified are:
  - result order;
  - draws at exit, kill and wait;
  - when threads start;
  - lock poisoning;
  - unwinding blocked processes.
- **Suggested response:** fix-in-plan.

### C-020: Trace-text hooks remain in the workspace tests
- **Where:** `workspace.rs:651`, `materialise.rs:717`
- **Failure mode:** missing-risk
- **Suggested response:** defer-with-rationale, with a backlog item.

### C-021: "Power loss" has no unlocking intent
- **Where:** `test-plan.md` End-to-End
- **Failure mode:** e2e-drift
- **Suggested response:** fix-in-plan.

## Confidence
block

## Responses

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan.<br>• **Smaller scenario:** processes 0 and 1 commit one op each, and only processes 2 and 3 commit a second. That makes about 200 syscalls.<br>• **Variants:** strict only, still with both kill settings.<br>• **Budget:** 10 minutes wall-clock for the `writers` test binary on the Windows host, measured and recorded.<br>• **Cost controls, in order:** seeds in parallel threads, thread reuse, and stopping a variant once its required kinds are found.<br>• **Seeds:** never fewer than 40. |
| C-002 | fix-in-plan.<br>• **Name:** corrected to `UnsyncedReferences`.<br>• **Forced race:** odd seeds force the parent race with two orderings keyed by process and occurrence. Process 0 is held between its op's rename and its `ops/` sync while process 2 lists and names that op. This is the owner's "table of breakpoints that coordinates integration testing", driving a real scenario.<br>• **Kills:** the odd-seed kill kills process 0 there, so a later process sees a dead process's unsynced op (T-011's own wording).<br>• **Tests:** `writers_forced_race_runs` asserts the race, and `writers_catch_unsynced_references` requires closure violations in those cells. |
| C-003 | fix-in-plan. Every variant must produce semantic violations in strict mode, with the kinds Sprints 1 to 3 required: durability and torn for `UnsyncedObjects`, and closure for `UnsyncedReferences` in the forced-race cells. Each variant has its own named test. |
| C-004 | fix-in-plan.<br>• **Log:** outside the threads, with *attempted*, *acknowledged* and *in-flight* defined.<br>• **Checks:** acknowledged ⊆ present ⊆ acknowledged ∪ in-flight; acknowledged objects present and valid; `store.heads()` equal to heads computed from the logged ops restricted to those present (not from the store's own listing); `fsck` with leftovers allowed. |
| C-005 | fix-in-plan. *k* is bounded by the process's boundary count in the seed's uncrashed run. `writers_kills_happen` asserts `Killed`, and the killed-cell count is printed. |
| C-006 | fix-in-plan. T-003 specifies:<br>• an action is not re-entered for the same name on the same thread (`on_point_action_is_not_reentered`);<br>• the hook runs only at syscall boundaries;<br>• an action's syscalls belong to the process that reached the point;<br>• `recover()` drops actions, reach and the switch callback (`recover_drops_actions_and_reach`). |
| C-007 | fix-in-plan. `point` has no default body. `points_reach_through_every_forwarder` covers `&`, `Rc` and `Arc`. |
| C-008 | fix-in-plan.<br>• **Digests:** T-002 asserts the full Sprint 3 digests as constants in the three reproducibility tests.<br>• **Migration conditions** (T-006): the point sits immediately after `rename` returns, and the action re-checks visibility and durability as `install_race` does. |
| C-009 | fix-in-plan.<br>• **Named points:** T-006 names every point and where it sits. `write_file` chooses `store.put_object.*` or `store.commit_op.*` by what it writes, each a literal.<br>• **`init`:** has its own `layout_created` and the shared `sync_layout.synced`.<br>• **Test:** `store_points_mark_each_stage` covers fresh writes and the already-present path. |
| C-010 | fix-in-plan. The rule:<br>• skips `sim`;<br>• removes comments;<br>• allows line breaks inside the parentheses;<br>• accepts one registry form in `src/points.rs`;<br>• checks the crate prefix and the sprint range.<br>The fixtures cover a commented-out and a rustdoc point, a literal on the next line, and an unrecognised entry. |
| C-011 | fix-in-plan.<br>• **Designated tests:** one per scenario, correct store only.<br>• **Writes:** through a temporary file and a rename.<br>• **What counts:** the scenario's run and the next command it models (the resuming `open`); the oracle's checks do not. |
| C-012 | fix-in-plan.<br>• **The command:** `cargo reach` is a flagless writer, guarded out of tree like `ci`.<br>• **In CI:** `cargo ci` gains an in-process `reach` step that compares and writes nothing.<br>• **INT-0003:** revised (AC-9, AC-10, Transition history) to four flagless project commands; the README follows in T-010. |
| C-013 | fix-in-plan. `reach_table_shows_the_triangle` reads the committed table. `ci_steps_are_ordered_and_locked` is extended to cover the cleared directory, the six steps and the scrubbed child variable. |
| C-014 | fix-in-plan. INT-0005 AC-6 defines "unit". A new Consequence requires a point for each new durable unit, and the policy rule bounds the `sprint` field. |
| C-015 | fix-in-plan. Recorded in INT-0005's Consequences, with the expected response, and required in the spec (T-008). |
| C-016 | fix-in-plan. T-009 covers `log`, error exits and a single report per command, taken from its first snapshot. New tests: `praps_log_reports_skipped_names`, `failed_undo_carries_reports` and `praps_failed_undo_reports_skipped_names`. |
| C-017 | fix-in-plan.<br>• **Reports:** children print their acknowledged ops and parents, and heads are computed from the reports.<br>• **Content:** each child writes distinct content with a fixed parent chain, so no file is renamed over another child's.<br>• **Errors:** a sharing error fails the test.<br>• **Child mode:** needs both an exact test-name argument and `PERI_WRITER_CHILD`, which `cargo ci` scrubs. |
| C-018 | fix-in-plan. Split into T-002 (the thread-safe `SimFs`, with the asserted digests as its only evidence) and T-003 (points). The tasks are renumbered T-001 to T-010. |
| C-019 | fix-in-plan. T-005 specifies:<br>• threads start with the baton;<br>• draws at exit, kill and ordering waits;<br>• results in input order (`processes_return_results_in_input_order`);<br>• poisoned locks recovered (T-002);<br>• other processes unwound with the kill marker, and the first panic reported. |
| C-020 | defer-with-rationale. The two remaining trace-text hooks are single-process unit tests that work today. T-010 adds a backlog task to move them to named points once points exist in `workspace`. |
| C-021 | fix-in-plan. INT-0007's Consequences now record power loss as out of scope, with the simulator's crash model standing in. The test plan cites it. |
