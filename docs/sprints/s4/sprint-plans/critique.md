# Plan Critique — Sprint 4

Round 3. The same read-only subagent reviewer re-reviewed the plans at `9c08de0` against its round-2 concerns ([critique-round2.md](critique-round2.md)). Every round-2 response landed as claimed.

**The odd-seed orderings.** The reviewer checked them in both kill settings and found no deadlock between live processes:
- Processes 1 and 3 wait on nothing.
- Process 2 waits only at its listing, for process 0's arrival at `renamed`.
- Process 0 waits at `temp_written` for the ends of processes 1 and 3 and for process 2's first `dir_synced`. It also waits at `renamed` for process 2's second `renamed`.
- Crashes and early ends release waiters.

**The `UnsyncedReferences` window.** It is one crash step per odd cell. Strict mode catches it with probability about 1/4 per cell, so over about 40 trials a miss has probability about 10⁻⁵.

**Points.** Every store point has a path that reaches it.

The concerns are condensed here; the responses are at the end.

## Concerns

### C-001: The order of events at a point is unspecified, so the forced-race durability check can pass vacuously
- **Where:** T-003, T-005, T-007
- **Quote:** "process 0's op name **SHALL NOT** be durable when process 2 reaches `test.writers.listing`"
- **Failure mode:** EARS-vague
- **Why it matters:** Process 2 usually arrives at its listing before process 0's op exists. A check at arrival passes trivially for a missing name. The kill on arrival also depends on whether the kill comes before the wait.
- **Suggested response:** fix-in-plan.
  - Order the events at a point: count, kill, wait, then actions.
  - Check visibility and non-durability on release.
  - Name a test.

### C-002: Where the reach recorder lives, and which side drops `test.` names, is unspecified
- **Where:** T-008
- **Failure mode:** hidden-dep
- **Why it matters:** Two copies of the recorder, one in each test tree, would each need the drop, and `xtask`'s test would check neither.
- **Suggested response:** fix-in-plan. Have `cargo reach` ignore `test.` names, or keep one shared recorder tested once.

### C-003: The fallback for an unreached point can conflict with INT-0005's new Consequence
- **Where:** T-006; INT-0005 Consequences
- **Failure mode:** intent-drift
- **Suggested response:** fix-in-plan. Allow removal only while the unit keeps a reached point.

### C-004: The oracle's sets are not stated to be by op id, now that process 1 commits one op twice
- **Where:** T-007, the log
- **Failure mode:** EARS-vague
- **Suggested response:** fix-in-plan. Define the sets by op id, and check that a retry interrupted by a crash keeps the op acknowledged.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan.<br>• **Order at a point** (T-005): count, then a kill set for that arrival, then ordering waits, then the actions once released (`point_events_are_ordered`).<br>• **The check** (T-007): it runs in an action at process 2's listing, so on release, and asserts that process 0's op name is visible and not durable.<br>• **The kill:** the odd-seed kill on arrival at `renamed` therefore takes effect before the wait. |
| C-002 | fix-in-plan.<br>• **One recorder:** `sim::reach::record`, writing through `RealFs`, shared by both test trees and tested once (`reach_record_round_trips`).<br>• **`test.` names:** `cargo reach` ignores them and fails on any other unregistered name (`reach_ignores_test_points`, in `xtask`). |
| C-003 | fix-in-plan. An unreached point may be removed only while its unit keeps at least one reached point. Otherwise a new scenario is extended. |
| C-004 | fix-in-plan.<br>• **Sets by op id:** attempted, acknowledged and in-flight are sets of op ids, with in-flight = attempted − acknowledged. An op acknowledged once stays acknowledged.<br>• **Test:** `writers_retry_reaches_present` checks a cell that crashes during the retry. |
