# Simulation: named points, several processes and reach

This chapter specifies three additions to the deterministic simulator
([INT-0005](../intents/INT-0005-deterministic-simulation-testing.md)) made in
Sprint 4:
- named points;
- several simulated processes, with a seeded scheduler;
- the reach table.

[INT-0007](../intents/INT-0007-crash-safe-storage.md) AC-5 (four writing
processes, under crash injection) is the first scenario built on them.

## Named points

A named point is a dormant, coded breakpoint. Code marks one with
`Fs::point("crate.function.stage")`.
- **On the real filesystem** (`RealFs`), it does nothing.
- **On the simulator** (`SimFs`), it:
  - counts the name ([`SimFs::reach`]);
  - runs the actions a test attached to it with `SimFs::on_point`, in the order attached.

**A point is not a syscall.** It is not counted, not traced, draws nothing
from the seed, and is not a crash point. Adding a point therefore changes no
single-process run. The three Sprint 3 matrix digests are asserted as
constants, and planting the Sprint 4 points left all three unchanged.

**Actions:**
- **When they run.** An action runs with no lock held, on the thread that reached the point. Its syscalls are that thread's own.
- **No re-entry.** An action is not entered again for the same name on the same thread while it runs. A point reached inside its own action is counted, and its actions do not run again.
- **After a crash.** Points still count, but run no actions.
- **On recovery.** `SimFs::recover` starts with no actions and no reach.

**`Fs::point` has no default body.** Every implementation states what it
does, so no forwarder (`&T`, `Rc<T>`, `Arc<T>`) can drop points.

**Where points live.** Points mark the stages of the durable units:
- the store's layout, object and op writes, and its re-syncs;
- the working copy's snapshot, record, writes and resume.

They are not placed by hand at arbitrary lines. Each sits at a boundary the
crash and interleaving tests need to name, and a name survives refactoring
where a line number would not.

## The registry and its policy rule

Each boundary crate that marks points lists every one in `src/points.rs`:

```rust
pub const POINTS: &[Point] = &[
    Point { name: "store.commit_op.renamed", unit: "store::commit_op", sprint: 1 },
    // …
];
```

- **`name`** is `crate.function.stage`, in lowercase ASCII with `_`, and starts with the crate's name.
- **`unit`** is the function or module of the boundary crate the point marks, as `crate::name`: a column of the reach table.
- **`sprint`** is the sprint whose build plan delivered that unit, not the sprint that planted the point.

`cargo policy`'s `points` rule reads each boundary crate's `src/` except
`sim`, whose forwarders pass names on. It removes comments first, so a point
in a comment or in rustdoc does not count. It fails on any of these:
- a `.point(` call whose argument is not one string literal;
- a marked name that is not registered;
- a registered name that is not marked;
- a duplicate;
- a malformed name or unit;
- a registry entry not of the form above;
- a `sprint` outside 1 to the latest `docs/sprints/sN`.

**The reserved `test.` prefix.** Tests may mark points with names under this
prefix. They are never registered, and the reach table ignores them.

Each new durable unit registers at least one point. Otherwise no scenario
could be shown to reach it (INT-0005 Consequences).

## Several processes

`sim::Processes` runs several closures as simulated processes over one
`SimFs`.

**The baton.** Each process is an OS thread, but a baton lets exactly one run
at a time. A thread starts only once it holds the baton.

**Draws.** The next runner is drawn from the run's seed, among the processes
able to run, at each of these:
- every syscall boundary and every point boundary of the running process;
- the process's end;
- its kill;
- its wait on an ordering.

A run is therefore a function of its seeds: the same seeds give the same
trace, filesystem and results.

**Orderings** follow RocksDB's sync-point dependencies. Process *q*, on its
*m*-th arrival at point B, waits until one of these holds:
- process *p* has arrived at point A for the *n*-th time;
- process *p* has ended (`After::End(p)`).

How a wait ends:
- **Arrival counts at once.** A process counts as having arrived as soon as it arrives, even if it then waits.
- **Released.** A wait is released when the system has crashed, or when the predecessor's process has ended without reaching A.
- **Deadlock.** If every live process waits on a live one, the run stops with `Stopped::Deadlock`, naming the waiting points. That is a test bug, never a matrix outcome.

**At a point,** in this order:
1. the point is counted;
2. a kill set for that arrival takes effect;
3. the process waits on its orderings;
4. once released, the point's actions run.

**Kills.** A process can be killed at its *k*-th boundary, or on its *n*-th
arrival at a named point. It stops there and makes no further syscall.
Unwinding marks it `Killed`.

**Panics.** A panic in any process ends the run. The others unwind at their
next boundary, and the run returns the first panic once every thread has
joined.

## The process-crash model

A killed process is a process crash with the system still running.
- **What survives.** Everything it wrote stays visible to the other processes, synced or not, as the page cache keeps it.
- **What is lost.** Its in-memory state, including any acknowledgement it was about to give.
- **A later system crash** (`set_crash_at`, then `recover`) settles every pending change, the killed process's included, by the seed.

The four-writer matrix exercises the case backlog T-011 named. In odd seeds,
process 0 is killed between its op's rename and its `ops/` sync, while
process 2 lists that op and names it as a parent. A later process thus sees
the crashed one's unsynced state before the system crash resolves it.

**The oracle.** The matrix's log lives outside the process threads, so it
survives kills. It keeps three sets of op ids:
- *attempted*: logged before `commit_op`;
- *acknowledged*: `commit_op` returned;
- *in-flight*: attempted but not acknowledged.

Every recovered state must satisfy:
- closure;
- acknowledged ⊆ present ⊆ attempted;
- every acknowledged object present;
- the heads the store reports equal to the heads of the logged ops that are present;
- no torn or stray file;
- for the correct store, `fsck` clean.

## Points are scheduling boundaries

Under `Processes`, every point is a draw and a kill boundary. Adding a point
changes the four-writer interleavings, its kill steps and its digest, though
it changes no single-process run.

When a sprint adds a point, the expected response is:
1. confirm that the forced races still run (`writers_forced_race_runs`);
2. confirm that every negative control is still caught;
3. re-record the four-writer digest in that sprint's test evidence and in
   the asserted constant.

The forced race itself depends on named points, not on the order of draws,
so it survives new points.

## The reach table

`docs/reach.md` is generated, never edited.

**Recording.** Each simulation scenario's designated test records the points
its cells reached, summed over the cells. It writes them with
`sim::reach::record` to `$CARGO_TARGET_TMPDIR/reach/<scenario>.txt`, through
a temporary file and a rename.

**What counts:**
- the scenario's own run;
- the next command it models on the recovered filesystem (`init` and `snapshot` for Snapshot; the resuming `open` for Materialise), read before the oracle checks anything.

The oracle's own checks never count.

| Scenario | Designated test | Added in |
|---|---|---|
| store | `crash_matrix_finds_no_violations` | Sprint 1 |
| two writers | `crash_matrix_finds_no_violations` | Sprint 1 |
| Snapshot | `snapshot_matrix_finds_no_violations` | Sprint 2 |
| Materialise | `materialise_matrix_finds_no_violations` (Unix rules) | Sprint 3 |
| four writers | `writers_matrix_finds_no_violations` | Sprint 4 |

**Generating.** `cargo reach` takes no flags. It reads the files and the
registries and writes the table:
- one row per scenario, by the sprint that added it;
- one column per unit, by the sprint that built it;
- each cell counting the unit's point hits in that scenario;
- then every point with its total.

A cell in a later row under an earlier sprint's column is a regression run
passing back through that unit: the owner's run × unit triangle.

**Failing.** `cargo reach` fails, writing nothing, if any of these holds:
- a registered point has no hit;
- a scenario's file is missing;
- a file names a point that is neither registered nor under the `test.` prefix.

**In `cargo ci`.**
- It clears the recorded reach before `test`, so a scenario that stopped recording cannot leave a stale file.
- After `test`, its `reach` step generates the table in memory and fails if it differs from the committed one. That step writes nothing.

**Not yet built.** Selecting which tests to run from the table is backlog
work.

[`SimFs::reach`]: ../../crates/sim/src/simfs.rs
