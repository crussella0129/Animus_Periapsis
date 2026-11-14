# Sprint 7 Research Report

## Intents Reviewed
The backlog puts the canonical merge of op heads first (T-029), and the Book says why: until it exists, two commands that overlap stop every later command, so no two agents can share a repository ([INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) Consequences; [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) Consequences). This sprint takes it up.

Selected, to be revised in Plan:
- [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) — **selected**; current state: proposed. Owns the canonical merge, the rule that a merge is computed and never committed, and the criteria for convergence, readers that write nothing, and no lost work. The sprint's main chapter.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — **selected**; current state: active. AC-3 asks for at least 8 workspaces over one repository at once, with no clones, no duplicated objects and no lost operations. No second workspace can be made today.
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — **selected**; current state: active. Its rule "undo is scoped" becomes necessary the moment a second workspace exists: today `undo` and `restore` set the whole view, so in a shared repository they would move every other workspace's pointer too.
- [INT-0006](../../../intents/INT-0006-format-v1.md) — **selected, to be revised**; current state: active. The provisional `operation` and `view` bodies have no room for the workspace that made an operation or for a pointer that two commands moved at once. Each change to the format specification needs the owner's sign-off (AC-6).
- [INT-0037](../../../intents/INT-0037-command-set.md) — **selected**; current state: active. Its table proposes `lanes` and `lane NAME` for workspaces. The owner signed AC-8 for the built commands only, so a name fixed now needs the owner's approval.
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — **selected**; current state: active. A concurrency scenario joins the reach table; the scheduler's cost at 1,000 processes matters.

Reviewed, no change expected in this sprint:
- [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md) — **reviewed**; current state: proposed. Ownership needs actors, and actors need keys and signed operations, none of which exists. Without them an operation's author can be named but not proved, so ownership can be recorded but not enforced against a faulty agent (AC-7 there; INT-0035 AC-6).
- [INT-0011](../../../intents/INT-0011-change-identity-and-rewriting.md) — **reviewed**; current state: proposed. Two concurrent rewrites of one change leave it divergent: two visible changesets with one change ID. The policy for resolving that is this chapter's; this sprint keeps both and loses neither.
- [INT-0012](../../../intents/INT-0012-conflicts-and-merge.md) — **reviewed**; current state: proposed. Merging file contents is not needed to merge views.
- [INT-0024](../../../intents/INT-0024-staged-sync.md) — **reviewed**; current state: proposed. No replica exchanges anything today. INT-0010 AC-5 asks for 2 replicas; two stores that copy each other's files in the simulator can stand in for them without building sync.

## 1. Sprint Goal
Several agents share one repository. Concretely:
- **Overlapping commands no longer stop the repository.** A command that finds several op heads merges them in one canonical step, computes that merge without committing it, and goes on. A command that changes nothing writes nothing, however many heads it finds. A command that changes something commits one operation whose parents are every head it merged.
- **Several workspaces over one store.** A second working copy can be made beside the first, sharing its store, and each workspace's changes stay its own.
- **Undo and restore are scoped.** They take back or return this workspace's own work and leave every other workspace as it is.
- **The merge is shown to be canonical** by the properties INT-0010 names: order and duplication independence, agreement with the three-way merge, grouping independence, criss-cross histories, and rejection of the known wrong merges.

Out of scope: signatures and enforced ownership (INT-0016), sync between real replicas (INT-0024), a view with shared structure and the other cost rules (INT-0006 AC-8, INT-0035 AC-1, AC-2, AC-4 and AC-5; backlog T-030), and merging file contents (INT-0012).

## 2. Existing Code Survey
| File | What it does today | What the goal needs from it |
|------|--------------------|-----------------------------|
| `crates/codec/src/view.rs` | `View { heads: set<hashref>, workspaces: map<string, hashref> }`, one working-copy changeset per workspace name | A pointer that two commands moved at once has no representation; per-workspace heads would make each workspace's changes its own |
| `crates/codec/src/operation.rs` | `Operation { parents: set<hashref>, view: hashref }`; several parents already allowed | Nothing names the workspace that made an operation, so "the last operation this workspace made" (INT-0009) cannot be found |
| `docs/spec/format-v1.md` | Both bodies *provisional*; golden vectors include `operation-merge` with two parents | A format change needs the spec first, new golden vectors and the owner's sign-off (INT-0006 AC-6) |
| `crates/codec/tests/golden.rs` | Pins every encoding against the spec's vectors | Re-recorded with the new bodies |
| `crates/store/src/lib.rs` | `op_log` lists `ops/` once (Sprint 6); `heads_of` derives heads from the op files; `commit_op` refuses missing parents | Unchanged in principle: an operation with several parents already commits |
| `crates/store/src/fsck.rs` | Closure: every op's parents and view, every object a view reaches | Must follow the new view body's references |
| `crates/workspace/src/lib.rs` | `state_of` refuses several heads with `SeveralHeads`; `Loaded` holds one `head`; `replace_working_copy` commits over that head; `restore_view_in` commits the target view object unchanged; `undo` takes the head's first parent | The core of the change: a merged state with a set of heads; commits over all of them; scoped undo and restore |
| `crates/workspace/src/record.rs` | The record names `from <op>` and `target <op>` | A merged state has no single op for `from`; it can name the working-copy changeset instead |
| `crates/workspace/src/materialise.rs` | Writes a tree F → T, crash-safely | Writing a new workspace's files is F = empty → T |
| `crates/praps/src/place.rs` | A top is a directory whose `.praps/store` can be listed | A second working copy has no store of its own; it needs a pointer to one |
| `crates/praps/src/commands.rs` | Each command opens the workspace at the top it found | Commands open a named workspace over a store that may live elsewhere |
| `crates/praps/src/registry.rs` | Eleven commands | `lane` and `lanes`, if the owner approves the names |
| `crates/praps/src/hint.rs` | `SeveralHeads` → "this build cannot join operations made at the same time yet; nothing was lost" | That refusal goes away |
| `crates/sim/src/processes.rs` | One OS thread per process; a baton; every boundary wakes all threads (`Condvar::notify_all`) | Runs 1,000 processes, slowly (Measurements) |
| `crates/store/tests/writers.rs` | Four writers with kills and crashes at the store level | The model for a workspace-level scenario |
| `crates/workspace/tests/workspace.rs` | `several_heads_stop_every_command`, `an_operation_written_after_open_leaves_two_heads` pin today's stop | Become tests of the merge |
| `crates/workspace/tests/history.rs` | Undo and restore over the whole view | Become scoped |
| `crates/workspace/tests/crash.rs` (and `materialise_crash.rs`, `describe_crash.rs`) | Matrices whose digests cover every byte written | Any encoding change moves their digests; their cell counts should not move |
| `docs/spec/working-copy.md` | "Reading the op log" says several heads stop a command | Rewritten for the merge, workspaces and scoped history |

## Measurements
**Eight `praps new` at once, today.** The release build at `855eb67`, on Windows, in one working copy of 10 files:
- 3 of the 8 commands succeeded. The other 5 failed: "the op log has 2 heads, which cannot be merged yet".
- The store then held 5 operations and 2 op heads. `praps status` refused with the same message and exit 1, as would every later command. `praps verify` printed `clean`: nothing was damaged, the repository just could not go on.

Sprint 6 measured the same in WSL: all 8 commands exited 0 and left 8 heads ([Sprint 6 research](../../s6/sprint-research/research-report.md)). Which commands fail depends on timing; that the repository stops does not.

**The simulator at 1,000 processes.** A throwaway test ran `sim::Processes` with 8, 100 and 1,000 processes, each making three syscalls, on Windows:

| Processes | Boundaries | Time |
|---|---|---|
| 8 | 25 | 0.9 ms |
| 100 | 301 | 99 ms |
| 1,000 | 3,001 | 9.0 s |

Every process finished in every run. The cost of a boundary grows with the number of processes, about 3 ms each at 1,000, because each boundary wakes every thread to see whose turn it is. A run of 1,000 readers that each open a small repository (some tens of syscalls each) would take minutes for one seed. Waking only the process the scheduler chose would cut that; the draws, and so every digest, would not change.

## 3. External Sources
- [Jujutsu: concurrency](https://jj-vcs.github.io/jj/latest/technical/concurrency/) — Jujutsu takes no locks: a command commits against the operation it loaded, and divergence is found by the next load, which merges the views three-way against their common ancestor. A pointer moved differently on two sides becomes a conflicted value, "moved from A to B or C", and commands that need it fail until it is resolved. The page warns that use from several machines is not well tested and may lose pointer moves.
- [Jujutsu: `lib/src/repo.rs`](https://raw.githubusercontent.com/jj-vcs/jj/main/lib/src/repo.rs) — The load with several heads folds them pairwise, starting from the first, and writes an operation that records the merge. A workspace's working-copy commit that both sides moved keeps the side being merged into, so the result depends on the order of the fold. (Read through a fetch that returned part of the file verbatim and summarised the rest; the working-copy rule comes from the summary.) These are the two behaviours INT-0010 rules out: a merge written as an operation, and a merge whose result depends on order.
- [Jujutsu: working copy](https://jj-vcs.github.io/jj/latest/working-copy/) — A second workspace has its own `.jj` directory linked to the shared repository. The working copy records the operation it was last brought up to; if a command did not finish bringing it up to date, it is "stale" and a command brings it up to date, or records what is on disk as a recovery commit.
- [Git: `git-worktree`](https://git-scm.com/docs/git-worktree) — Linked working trees share every ref and object; each keeps its own HEAD and index. A `.git` file at the top of a linked tree points back to its entry in the main repository. Git takes no lock across worktrees.
- [Git: merge strategies](https://git-scm.com/docs/merge-strategies) — When two heads have more than one common ancestor, `ort` first merges those ancestors into a virtual merge base and merges against that. INT-0010 asks for the same for criss-cross histories of operations.

## 4. Risks, Unknowns, Dependencies
- **The format changes, and with it every digest.** An operation that names its workspace and a view whose pointers can hold several changesets change the bytes of every operation and view. The golden vectors, and the digests of all five crash matrices, move. Their cell counts should not: a field added to an object adds no syscall. The owner signs off each specification change (INT-0006 AC-6).
- **A pointer two commands moved at once.** Two commands in the same workspace at the same time each move its working-copy pointer. The merge keeps both values. The next command in that workspace must choose one to work from; a fixed rule keeps every replica's choice the same, and the other stays visible. When both were rewrites of one change, the change becomes divergent: its policy is INT-0011's and is not settled here.
- **Ownership is named, not enforced.** Without signatures a process can claim any workspace, so a faulty agent can still change another's entry. This sprint records who made each operation; enforcing it waits for INT-0016.
- **Resume after a merge.** The record of an interrupted write names the operation it started from. A merged state has no such operation, so the record has to name what the working copy held (the working-copy changeset) instead.
- **Cost.** Every command still reads the whole op log (Sprint 6, T-003), and finding the common ancestors walks it again. Merging reads the base's and each head's view. None of INT-0035's cost rules is met by this sprint; that is backlog T-030.
- **The simulator at 1,000 processes** is slow (Measurements). One seed of 1,000 readers is affordable; many seeds are not, unless the scheduler wakes only the process it chose.
- **Names.** `lane` and `lanes` are proposals in INT-0037's table; fixing them needs the owner.
- **Not in scope but open:** the `init` that hides a lost history (backlog T-036, first item) is unrelated to the merge and stays in the backlog.

## 5. Recommended Approach
1. **Record what Sprint 6 left** (T-035): the runs after its tested head and of its merge, and this boundary's dependency update (PR #9, merged with the owner's approval).
2. **Change the format once**, with the specification first:
   - an operation names the workspace that made it;
   - a view holds, for each workspace, its working-copy pointer as a set of changesets (one when resolved, several when two commands moved it at once: the support-set normal form of INT-0010) and its heads.
3. **The canonical merge** as a pure function over views: the base is the heads' lowest common ancestor in the op graph, or the canonical merge of several when the history criss-crosses; each pointer becomes the base's value if no head changed it, else the union of the values of the heads that did; each set of heads loses what any head removed and gains what any head added. Property tests for order, duplication, grouping, criss-cross histories, two-head agreement with the three-way merge, and the registered wrong merges.
4. **Load merged states**: a command with several heads computes the merge and keeps it in memory; it writes nothing unless it changes something, and then commits one operation over every head. A working-copy pointer left in conflict is resolved by a fixed rule, with a note naming the change kept and the ones still visible.
5. **Scoped undo and restore**: `undo` takes back the last operation this workspace made, and `restore OP` returns this workspace to its entry at OP; neither touches another entry.
6. **Several workspaces**: open a named workspace over a store that may live elsewhere; a command to make one beside the first, and one to list them, if the owner approves their names.
7. **Simulation**: 8 workspaces' processes at once with kills and crashes, over two stores that copy each other's files; 2, 8 and 1,000 readers that write nothing. The existing matrices re-recorded, their cell counts unchanged.
8. **The scheduler wakes only the process it chose**, if the 1,000-reader run needs it.

## Artifacts
- This report. The measurements above were taken with throwaway scripts outside the repository; their commands and results are given in full in Measurements.
