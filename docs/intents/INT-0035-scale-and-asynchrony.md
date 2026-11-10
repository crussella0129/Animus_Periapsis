# INT-0035 — Scale: thousands of agents at once, with cost that follows the change

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0035
- **State:** planned
- **Work evidence:** [Sprint 6 T-003 build plan](../sprints/s6/sprint-plans/build-plan.md#t-003-a-command-reads-the-op-log-once), [Sprint 6 T-009 build plan](../sprints/s6/sprint-plans/build-plan.md#t-009-log-lists-changes-and-ops-lists-operations)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Periapsis serves thousands of agents working on one project at the same time,
on one machine or on thousands, with no agent waiting for another. One rule
measures it: **what a command costs depends on the size of its own change,
not on the size of the history, the number of agents, or what the other
agents are doing.**

Scale model. The design is held to the first column and tested first at the
second:

| | Design point | First test point |
|---|---|---|
| Agents active at once | 10,000 | 1,000 |
| Replicas | 10,000 | 100 |
| Operations in the log | 10⁹ | 10⁶ |
| Changes | 10⁸ | 10⁵ |
| Workspaces in one view | 100,000 | 1,000 |
| Files in one working copy | 10⁶ | 10⁵ |

Cost rules. Here *n* is the size of what a command touches (operations,
changes, view entries or files) and *k* the size of its own change:
1. **Writing.** A command writes O(*k* log *n*) bytes, and the number of
   durable flushes it makes grows with neither *k* nor *n*.
2. **Loading.** A command reads what is new since its replica last ran one,
   plus O(log *n*). It never reads the whole log.
3. **Merging.** Joining concurrent op heads costs O(*d* log *n*), where *d*
   is how much those heads differ from their common base.
4. **Workspaces.** Making a workspace writes O(log *n*) bytes and no
   working-copy file unless one is asked for.
5. **Status.** A command reads no file whose metadata is unchanged since the
   last snapshot.
6. **Queries.** The reference queries of
   [INT-0023](INT-0023-query-language.md) are answered from indexes.
7. **Sync.** Bytes follow what is missing and what the replica follows
   ([INT-0024](INT-0024-staged-sync.md)).

No coordination point:
- **Nothing to wait for.** No lock, leader, lease server or shared file that
  is changed in place is needed for any command to finish
  ([INT-0010](INT-0010-lock-free-concurrency.md)).
- **Asynchronous.** No result depends on a clock, on a limit to how long a
  message takes, or on a peer being reachable. Replicas that hold the same
  operations compute the same state.
- **Authority without agreement.** A decision that needs authority, such as
  promotion to `public`, a Book signature or an erasure, is a signed event by
  a key that repository policy names. The agents never have to agree on
  anything. Two such decisions made at once that cannot both stand show as
  divergence, and a rule that policy names settles it; arrival order never
  does.

Isolation:
- **An agent changes only what it owns.** Each entry of the view has an
  owner. An operation's effect on entries its actor does not own is not
  applied when operations are merged, and is reported
  ([INT-0016](INT-0016-actors-and-agent-lifecycle.md)).
- **A faulty agent costs the others nothing more than skipping its work.**

Following a part:
- **A replica chooses what it follows**: the public line, the Book, named
  lanes. Its storage and sync costs follow that choice.
- **Working operations stay home.** The snapshots and undos of a workspace
  stay on its replica until the workspace publishes
  ([INT-0009](INT-0009-reversible-by-default.md)).

Accelerators are optional. A local service that gathers many agents' writes
into one flush, caches, and the servers of
[INT-0030](INT-0030-optional-scale-mode.md) make things faster. With all of
them absent, every command still works and gives the same answer.

Boundaries: this intent owns the scale model, the cost rules and the suite
that checks them. How each rule is met belongs to the chapter that owns the
structure:
- object sizes: [INT-0006](INT-0006-format-v1.md);
- flushes and packing: [INT-0008](INT-0008-compact-history-storage.md);
- merging: [INT-0010](INT-0010-lock-free-concurrency.md);
- indexes: [INT-0013](INT-0013-derived-data.md);
- workspaces and status: [INT-0014](INT-0014-working-copy-and-workspaces.md);
- ownership: [INT-0016](INT-0016-actors-and-agent-lifecycle.md);
- queues: [INT-0021](INT-0021-lanes-and-review-requests.md);
- sync: [INT-0024](INT-0024-staged-sync.md);
- the simulator: [INT-0005](INT-0005-deterministic-simulation-testing.md).

How fast each command is, against other systems, is
[INT-0036](INT-0036-measured-speed.md).

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Writes follow the change: in the simulator, each state-changing command writes the same number of bytes, within a factor of 2, on stores of 10³, 10⁴, 10⁵ and 10⁶ operations, and makes the same number of flushes for a change of 1 file as for a change of 1,000.
  - Sample: every state-changing command in the registry, with changes of 1, 10, 100 and 1,000 files.
  - Threshold: bytes within ×2 across store sizes; flush counts equal across store sizes and change sizes.
- **AC-2** `necessary` `empirical` — Loads follow what is new: with derived data up to date, each command reads the same number of bytes, within a factor of 2, on stores of 10³ to 10⁶ operations; and after *k* operations arrive from elsewhere, the next command's extra reads grow no faster than *k*.
  - Sample: every command in the registry; *k* of 10 and 1,000.
  - Threshold: bytes within ×2 across store sizes; extra reads at *k* = 1,000 at most 200 times those at *k* = 10.
- **AC-3** `necessary` `empirical` — Nobody waits and nothing is lost: 1,000 simulated processes each run ten commands against one store at the same time, with processes killed and the system crashed at seeded steps. Every command that is not killed completes, none fails because of another writer, no acknowledged operation is lost, and every process then computes the same view.
  - Sample: at least 40 seeded schedules.
  - Threshold: 0 failed commands, 0 lost operations, 1 view.
- **AC-4** `necessary` `empirical` — A view does not grow with the fleet: a snapshot in one workspace writes the same number of bytes, within a factor of 2, whether the view holds 10³, 10⁴ or 10⁵ workspaces.
- **AC-5** `necessary` `empirical` — Merging follows the difference: joining *h* op heads that each differ from their common base in at most *c* entries reads a number of objects in proportion to *h* · *c*, on views of 10³ and 10⁵ entries.
  - Sample: *h* of 2, 100 and 1,000; *c* of 1 and 10.
  - Threshold: objects read per *h* · *c* within ×3 across the grid.
- **AC-6** `necessary` `empirical` — A faulty agent harms only itself: in simulation, agents that commit well-formed operations which try to move, hide or rewrite what they do not own change no entry outside their own, on any replica, and no other agent's command fails.
  - Sample: 1,000 agents, 10 of them faulty, over at least 40 seeds.
  - Threshold: 0 entries changed outside their owner's operations; 0 failed commands among the other agents.
- **AC-7** `necessary` `formal` — No clock decides: no result in a core crate depends on the time of day or on a timeout.
  - Obligation: `unstated`.
  - Proxy: a `cargo policy` rule that confines time sources to the real half of `sim`, as randomness is confined, and simulation runs with every process's clock skewed.
- **AC-8** `necessary` `empirical` — Accelerators are optional: every scenario of AC-1 to AC-6 gives the same views, operations and command output with the local service off, and with all derived data deleted before each command.
  - Threshold: 0 differences; only the costs differ.
- **AC-9** `supporting` `empirical` — Replicas converge without coordination: 100 simulated replicas and 1,000 agents exchange over a network that delays, reorders, duplicates and partitions. Once every message is delivered, replicas that follow the same things hold the same view.
  - Sample: at least 40 seeds, with the network simulation of [INT-0024](INT-0024-staged-sync.md).
  - Threshold: 0 replicas that differ.
- **AC-10** `supporting` `empirical` — Following a part costs a part: in a project with 1,000 active lanes, a replica that follows the public line and one lane stores and receives at most twice the bytes it would if those were the whole project.
- **AC-11** `supporting` `empirical` — A real fleet: at least 1,000 real agent processes work on one project for an hour. No command fails because of another agent, and the median command stays within the interactive budget of [INT-0036](INT-0036-measured-speed.md) AC-5.
- **AC-12** `necessary` `asserted` — The project owner approves the scale model and the cost rules.
  - Signers: project owner.

## Rationale
- Owner direction, 2026-11-09: Periapsis "should quite literally support
  thousands of agents, working in tandem, potentially in a decentralized
  completely asynchronous manner".
- Measured on the code as it stood
  ([Sprint 6 research](../sprints/s6/sprint-research/research-report.md)):
  - cost follows history, not the change: on Linux, `status` on an unchanged
    working copy of 100 files took 7.5 ms with 10 operations in the log and
    42 ms with 800, and `log` went from 3.6 ms to 1.8 s;
  - two commands that overlapped left two op heads, after which every
    command failed.
- By arithmetic: the provisional view lists every workspace, about 76 bytes
  each. At 10,000 workspaces every operation would write about 760 KB to move
  one pointer.
- Why this rule. A cost that follows the change makes a thousand agents cost
  a thousand times one agent. A cost that follows the history, or the fleet,
  makes them cost a million times.
- Why it can be checked. The simulator counts syscalls, flushes and bytes
  exactly, so these criteria need no timing and give the same answer on every
  machine.
- Why no coordination point. Agents on different machines, some offline,
  cannot share a lock. Systems that reach this scale today do it through a
  central server (Mononoke; Oak, 2026). Git gives each agent a checkout of
  its own and makes them queue at any branch they share.
- Why isolation. Among a thousand agents some are always faulty. Signatures
  say who did a thing; only ownership limits what that thing can touch.
- Sources: [Sprint 6 research](../sprints/s6/sprint-research/research-report.md);
  [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md) II §2.1,
  II §2.3, R1-E12, R3-B.

## Alternatives
- **A central server that orders every write** (Mononoke, Oak). Simple, and
  fast when it is up. It is a coordination point: an agent that cannot reach
  it cannot work, and the charter is distributed first. Servers stay as
  optional accelerators.
- **Scale as an optional mode added last**
  ([INT-0030](INT-0030-optional-scale-mode.md) as first written). Formats
  frozen without scale in mind would need a second format to fix. The view is
  the example.
- **One clone or worktree per agent, joined at a shared branch** (Git today).
  Each agent pays for a whole checkout, and all of them queue at the branch.
- **Locks with timeouts.** A timeout is a clock deciding a result, and a
  crashed holder stalls everyone until it expires.

## Consequences
- The provisional kinds of [INT-0006](INT-0006-format-v1.md) cannot freeze
  until AC-1 and AC-4 hold: the view needs shared structure first.
- The derived-data engine ([INT-0013](INT-0013-derived-data.md)) and the
  storage form ([INT-0008](INT-0008-compact-history-storage.md)) become
  Phase 1 work, not later work.
- The canonical merge of [INT-0010](INT-0010-lock-free-concurrency.md) comes
  before further single-user features: until it exists, nothing works with
  two agents. There is one exception, which the owner approved with Sprint
  6's plan on 2026-11-10: that sprint builds the command set of
  [INT-0037](INT-0037-command-set.md) first. The command set is one of the
  three aims of the direction this chapter answers, and the merge needs
  decisions that still wait on the owner: who owns which entries of the
  view. The merge is the next work after it (backlog T-029).
- The simulator needs counts of cost, a way to run a thousand processes, and
  a clock and a network
  ([INT-0005](INT-0005-deterministic-simulation-testing.md)).
- More structure to keep correct: shared-structure maps, indexes, operation
  scopes and ownership. Each needs its own simulation scenarios and negative
  controls.
- Two decisions wait on the owner: who owns which entries of the view
  ([INT-0016](INT-0016-actors-and-agent-lifecycle.md)), and how colliding
  claims are settled
  ([INT-0017](INT-0017-event-records-and-book-view.md)).

## Transition history
- 2026-11-09: created as `proposed` (Sprint 6 research; owner direction of 2026-11-09 for thousands of agents, decentralised and asynchronous).
- 2026-11-10: `proposed` → `planned` (Sprint 6 build plan T-003: a command reads the op log once, a first step toward cost rule 2. It meets no criterion of this chapter. The criteria wait for the merge, the packed storage form, the stat cache and the simulator at scale: backlog T-029 to T-032).
- 2026-11-10: revised while `planned` (Sprint 6 plan critique, round 1, C-017; `editorial`: no criterion changes): the Consequence that puts the merge before further single-user features names Sprint 6's command-set work as the exception the owner approved, and why.
