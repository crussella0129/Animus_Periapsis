# Sprint 7 code review

A read-only reviewer, run as a subagent during Test, read the changes from `855eb67` to `38a6b96` under `crates/` against the working-copy and format specifications and INT-0009, INT-0010, INT-0014 and INT-0016. It was told what the plans had already accepted and should not be reported:
- the merge's known limit;
- two commands writing one working copy at once, and two lanes made at once with one name (backlog T-042);
- ownership not enforced against a forged workspace name;
- the view rewritten whole by each operation;
- merge cost.

It reported six findings. Each was checked against the code before anything was changed, and every fix has a test that fails without it: the fix was taken out, the test run, and the fix put back.

## Findings

| # | Where | Finding | Severity (reviewer's) | Outcome |
|---|---|---|---|---|
| 1 | `workspace::Workspace::make_lane` | It refused a hazard but not a blocked path, so a path whose temporary name another path of the change takes was left out of the lane without a word. `restore` refuses the same plan. | should fix | **Fixed.** Making the lane is refused with `Blocked`, before `.praps/workspace` is written. Test `a_lane_whose_write_is_blocked_is_refused`: the lane's change tracks `.praps-tmp-1` and `a`; without the fix the lane was made without `a`. The spec's step 4 says so. |
| 2 | `state_of`, `refuse_unless_free` | A lane's directory is tied to its entry by name only. If `praps lane ../a/NAME` is killed before its op lands and the lane is then made at another directory with the same last name, the first directory opens as that lane, and its first command records its empty working copy into the lane's entry. | should fix | **Deferred to backlog T-042, which the plan already holds for the same cause.** It needs a kill and then a retry under another directory. Nothing is lost, since every operation stays and `restore` returns to any of them, but the two directories then drive one entry. The fix is the one T-042 names, `.praps/workspace` naming the operation that made the lane, which changes that file's form; it is not made at the end of Test. T-042 now gives this case. |
| 3 | `workspace::merge` | `writers`, `Ancestry::of` and `Merger::view_of` index the op map and panic when an op names a parent that is not in `ops/`. Sprint 6's `undo` returned `Corrupt("op X names a missing parent Y")` there instead. | could fix | **Fixed.** Every load refuses an op log in which an op names a missing parent, as `Corrupt`, as it refuses a damaged op file. Test `a_missing_parent_op_is_refused`; without the fix it panicked in `merge.rs`. |
| 4 | `lane::LaneFile::encode`; `praps lane` | The lane file writes the store's path as it displays, which loses a path that is not UTF-8, and the name unescaped, so a last name holding a line break makes a file that does not parse. In both cases the op has landed, so the lane can be neither used nor made again. | could fix | **Partly fixed.** `praps lane` refuses a last name that holds a control character, as no plain text: a new case in `praps_lane_refusals`. A store path that is not UTF-8 is **deferred to T-042**, which revises the lane file. |
| 5 | `praps` error report | `UnfinishedLane` carried the working copy's root, `.` when the command runs at the lane's top. The message named the lane "at .", and the hint `praps lane .`, which, followed in the repository, aims at the repository itself. | could fix | **Fixed.** The error names the lane by the place the refusal names, its full path when the command runs inside it. `praps_lane_finishes_an_interrupted_lane` now runs the hint's own command and finishes the lane with it. |
| 6 | `praps init`; the search; `open` | A `.praps/workspace` that cannot be read (here a directory in its place): `init` took it as absent and made a store inside what may be a lane; the search reports it as a `.praps/store` that cannot be listed; `open` reports a bare I/O error. | could fix | **Partly fixed.** `init` refuses it and names the file, with the new hint "make that file readable, then run the command again": test `init_refuses_an_unreadable_workspace_file`. The two messages are **deferred to backlog T-036**, which holds the earlier review's findings about refusals and paths. |

## What the review covered

- **Read in full:** `merge.rs`, `lib.rs`, `lane.rs`, `record.rs`, `materialise.rs`, `place.rs`.
- **Read in their diffs:** the codec, and `praps`'s `commands.rs`, `hint.rs` and `registry.rs`.
- **Read in part:** the store's `lib.rs`; the specifications and intents named above; the tests `lanes.rs` and `lane_crash.rs`, and the lane parts of `e2e.rs` and `cli.rs`.
- **Checked and found as the specification says:**
  - the writers walk, and the bases with the virtual base when history criss-crosses;
  - `merge_entries`;
  - the race rule wherever a pointer is chosen;
  - the scoping of `undo` and `restore`, and the reuse of an existing view object;
  - every crash window of making a lane and of `restore` against resume;
  - the lane command's search and its free check;
  - the order of the codec's maps and sets;
  - the paths by which a command that changes nothing writes nothing.
- **Considered and not reported:**
  - The race note could say an unchosen change stays among the heads after the merge dropped it. Every way the reviewer found to reach that needs a racing `undo` or `restore`, the accepted case.
  - `bases` keeps every common op's ancestor set, quadratic in a long race, which falls under merge cost.

## What it did not cover

- **Tests not read:** `concurrency.rs`, `merge.rs`, `history.rs`, `scoped.rs` and `workspace.rs`, and the codec and store test changes. The test critic reads the evidence.
- **Not read at all:** the internals of `xtask` and `sim`.
- **No tests run.** The reviewer ran none; each finding came from reading the code paths, and the checks above confirmed each by a test.
