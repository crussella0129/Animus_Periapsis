# Sprint 4 Code Review

A read-only subagent reviewer read `git diff 7b2fd59..HEAD -- crates/ .cargo/`: Sprint 4's code against the Sprint 3 merge, with the locked plan and the spec. It ran no tests.

**What held up.** The reviewer found no high-severity defect. The baton scheduler checks out on every point examined:
- the lock order is fixed: the scheduler's lock, then the SimFs state, never the reverse;
- every wakeup happens under the lock, and every wait condition, once true, stays true;
- abort and deadlock handling are sound;
- no SimFs lock is held while a process unwinds;
- the switch guard is dropped before its callback runs;
- the re-entry guard works;
- runs are deterministic with seeds in parallel cells.

The findings are condensed here; each is answered at the end, and every fix has a test.

## Findings

### CR-01 (medium): the overlap evidence used actions that run after the scheduler's draw
- **Where:** `crates/store/tests/common/writers.rs` (`Watch`), with `crates/sim/src/simfs.rs` (`point`).
- **What is wrong:** A point's actions run after the count, the kill, the wait and the draw. So the "open commits" counter rose and fell when each process resumed, not when its rename and directory sync happened.
- **Effect:** `overlapped` could be wrong both ways:
  - **false positive:** a process's decrement is delayed past another's rename;
  - **false negative:** the other process renames and syncs before the first process's action runs.

  `writers_overlap_in_some_even_seed` could therefore pass on a spurious overlap. `undurable_at_rename` undercounts for the same reason. That is harmless, because its test asserts only `> 0`.

### CR-02 (low): orderings and kills were not validated
- **Where:** `crates/sim/src/processes.rs`.
- **What is wrong:**
  - an arrival or step of 0 never matches;
  - an out-of-range process index was ignored, or panicked inside the scheduler as a confusing process panic;
  - a second kill for the same process silently replaced the first.
- **Effect:** An off-by-one could leave a run with no kill or no forced race, and its test would pass without testing anything.

### CR-03 (low): the `points` rule did not fail closed
- **Where:** `crates/xtask/src/points.rs`.
- **What is wrong:** It recognised only `.point(`, so it missed:
  - a call through a path (`Fs::point(…)`), which was neither counted nor reported;
  - a `.point("…")` in a `#[cfg(test)]` module under `src/`, which counted as a production use.

### CR-04 (low): a unit registered with two sprints would be double-counted
- **Where:** `crates/xtask/src/reach.rs` and the `points` rule.
- **What is wrong:** Columns were keyed by (sprint, unit) but cells filtered by unit alone, and policy allowed one unit to name two sprints. Each of the two columns would then show all of the unit's hits.

### CR-05 (low): a misplaced doc comment on `RealFs`
- **Where:** `crates/sim/src/real.rs`.
- **What is wrong:** `fn point` was inserted between `name_rules`'s doc comment and the function. The comment, with its T-015 caveat, ended up documenting `point`.

### CR-06 (low): a hook that unwinds was lost
- **Where:** `crates/sim/src/simfs.rs` (`syscall`).
- **What is wrong:** The hook was taken from its slot and put back only on a normal return. A panic, or a kill under `Processes`, removed it for good. This is latent today: no hook used under `Processes` makes syscalls.

### CR-07 (low): `reported()` could hold an earlier command's entries
- **Where:** `crates/workspace/src/lib.rs` (`snapshot`).
- **What is wrong:** The field was overwritten only after a successful scan. A command whose `load` or scan failed would leave the previous command's report in place on a reused `Workspace`. `praps` opens a fresh workspace per command, so it was not affected.

## Responses

| Finding | Response |
|---------|----------|
| CR-01 | **Fixed.** `overlapped` is now read from the trace, in syscall order: an op renamed into `ops/` while an earlier op's rename there has not been followed by a sync of `ops/` (`writers::overlapped`, tested by `overlap_is_read_from_the_trace`). The two actions that kept the counter are removed. They ran no syscall, so no trace, state or digest changes. `undurable_at_rename` is kept, documented as a lower bound: a name, once durable, stays durable until a crash, so a late check can only undercount. |
| CR-02 | **Fixed.** `Processes::run` validates the setup before any thread starts. A process index outside the run, or an arrival or step of 0, in an ordering, a predecessor or a kill, panics with a message naming it. A second kill for one process panics in `kill`. `invalid_setups_are_refused` covers all eight cases and the double kill, and checks that no process started. |
| CR-03 | **Fixed.** The rule reports any `::point(` call, which it asks to be written as `.point("…")`. It also reports any `.point(` after a file's first `#[cfg(test)]` (test modules sit at the end of a file), so test code in `src/` neither marks nor counts as a use. The `path-call` and `test-code` fixtures cover both. |
| CR-04 | **Fixed, both ways.** The `points` rule requires one sprint per unit (the `unit-sprints` fixture), and the reach table filters each cell by unit and sprint. The committed table is unchanged. |
| CR-05 | **Fixed.** `point` and its own doc comment now sit above `name_rules`'s doc block. |
| CR-06 | **Fixed.** A drop guard puts the hook back when it returns or unwinds, unless a new hook was set meanwhile. A comment says that a hook used under `Processes` should make no syscall of its own. Tested by `a_hook_that_unwinds_is_put_back`. |
| CR-07 | **Fixed.** `snapshot` clears `reported` before it loads, so a snapshot that fails early reports nothing stale. A command keeps the latest snapshot's report, not the first's. The plan's "first snapshot only" was meant to keep `undo`'s second snapshot from adding duplicates. It still adds none: the field is replaced, and both snapshots scan the same files. |

Also added while answering: `reach_check_fails_on_a_stale_table`. It shows that the `reach` step of `cargo ci` passes on the table `cargo reach` wrote, fails on a stale or missing table, and writes nothing. This replaces a Windows-leg probe that would have rerun all of `cargo ci`.
