# Sprint 6 Book audit

Every intent chapter, read against the owner's direction of 2026-11-09:

> this Version control paradigm should quite literally support thousands of
> agents, working in tandem, potentially in a decentralized completely
> asynchronous manner and it should be far faster and more optimized than any
> version control that currently exists, with a far more intuitive command set
> and UI

The direction has three parts, named here so each finding can say which it
serves:
- **Scale:** thousands of agents at once, decentralised and fully asynchronous.
- **Speed:** faster than any version control system in use.
- **Use:** a far more intuitive command set and UI.

This file is research provenance. The chapters are the authority; where a
finding leads to a change, the change is made in the chapter and recorded in
its Transition history ([research report](research-report.md), section 5).

**Verdicts.** *Fits*: the chapter already holds under the direction.
*Revise*: the chapter needs new or changed text or criteria. *Blocks*: the
chapter's present state stops several agents from working at all.

## Summary

| Verdict | Chapters |
|---|---|
| Blocks | INT-0010 |
| Revise, large | INT-0001, INT-0006, INT-0008, INT-0009, INT-0013, INT-0014, INT-0021, INT-0022, INT-0024, INT-0030 |
| Revise, small | INT-0003, INT-0004, INT-0005, INT-0011, INT-0012, INT-0015, INT-0016, INT-0017, INT-0023, INT-0032 |
| Fits | INT-0002, INT-0007, INT-0018, INT-0019, INT-0020, INT-0025, INT-0026, INT-0027, INT-0029, INT-0031, INT-0033, INT-0034 |
| Not applicable | INT-0028 (abandoned) |

The Book has no chapter for any of the three parts of the direction:
- **Scale** appears only as "at least three concurrent agents" (INT-0001 AC-2), "at least 8" writers and workspaces (INT-0010 AC-5, INT-0014 AC-3), and an optional last-priority mode (INT-0030).
- **Speed** is not a goal anywhere. One criterion has a time budget (INT-0014 AC-5), and that budget was never fixed.
- **Use** appears as two naming rules in INT-0022. No chapter lists the commands, says how a person finds them, or describes any screen beyond one `log` layout.

Four new chapters fill those gaps:
[INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) (scale),
[INT-0036](../../../intents/INT-0036-measured-speed.md) (speed),
[INT-0037](../../../intents/INT-0037-command-set.md) (the command set) and
[INT-0038](../../../intents/INT-0038-live-view.md) (the live view).

## What the direction requires that no chapter says

These seven findings cut across chapters. Each names where it lands.

1. **Cost must follow the change.** No chapter bounds what a command costs. The code built so far costs in proportion to the whole history (see [code audit](code-audit.md)). INT-0035 states the rule; INT-0006, INT-0008, INT-0013 and INT-0014 each take a criterion from it.
2. **A view grows with the number of agents.** The provisional `view` body lists every head and every workspace, so each operation writes all of them again. At 10,000 workspaces that is about 760 KB for every operation. INT-0006 must give views shared structure before the format freezes.
3. **A merge of op heads must not be an operation.** If every reader that finds several heads stores a signed merge op, each merge op is a new head, and thousands of readers never settle. INT-0010 must say that the merge is computed, and that only a command's own operation is written.
4. **Undo is global.** `undo` and `restore` put back a whole earlier view. With one writer that is the writer's own state. With many, it rolls back everyone else's work too. INT-0009 must scope both to the entries the actor owns.
5. **Nothing limits what one agent can do to another.** Every op is signed (INT-0016), but no chapter says an agent's op cannot move another agent's workspace or hide its changes. With thousands of agents, some faulty, each must be able to change only what it owns. INT-0016 gains that rule.
6. **Every replica would carry every agent's working operations.** Sync is defined over the whole op log (INT-0024). Each snapshot is an op, so each replica would store and transfer every keystroke-level step of every agent. INT-0009, INT-0010 and INT-0024 must let working operations stay local and let a replica follow only what it needs.
7. **One integrator is assumed.** INT-0021 speaks of "the queue" that computes dev and promotes batches. A decentralised project has several verifiers at once, so the chapter must say what happens when two promote at the same time.

## Charter and foundations

### INT-0001 Periapsis charter — Revise, large
- The Intent calls Periapsis a system "for many concurrent agents" but sets no size, no speed and no standard for use.
- The intent map lists scale last, as "Phase 5, scale and assurance", through an optional mode.
- The map omits INT-0032 and INT-0033, which were created after it. AC-1 says adding a child is a recorded revision of this chapter, so the omission is a defect.
- **Change:** the Intent names the three parts of the direction; the map gains INT-0032, INT-0033 and INT-0035 to INT-0038; three criteria tie the charter to the new chapters' suites, as AC-5 ties it to INT-0034's.

### INT-0002 Typed acceptance criteria — Fits
- Scale and speed criteria are `empirical`, with thresholds fixed before measurement, which the grammar already carries.
- Counts of syscalls, flushes and bytes from the simulator are exact, so cost criteria need no statistics.

### INT-0003 Engineering baseline — Revise, small
- `forbid(unsafe)` and the pure-Rust preference rule out memory-mapped files and hand-written assembly hashing, both of which other systems use for speed.
- **Change:** a Consequence says speed is met inside those rules, that INT-0036 measures what they cost, and that an exception goes through AC-6.

### INT-0004 Windows and Linux — Revise, small
- A Consequence still says `undo`, `restore` and `new` drop their snapshot's report "a gap that backlog T-019 closes". Sprint 4 closed it.
- Small files and flushes cost most on NTFS, so the one-flush rule of INT-0035 matters most there.
- **Change:** the stale sentence is corrected; a Consequence points to INT-0036's budgets, which hold on both platforms.

### INT-0005 Deterministic simulation testing — Revise, small
- The Intent says clocks go through traits. No clock trait exists yet, and no network trait (AC-5, with INT-0024).
- `sim::Processes` runs each simulated process as an operating-system thread ([code audit](code-audit.md)), which will not reach thousands.
- A Consequence says `RandomState` and transitive randomness "are not yet banned (backlog T-014)". Sprint 3 banned them.
- Documentation evidence says `none`, though `docs/spec/simulation.md` specifies this intent's points, processes and reach.
- **Change:** the stale sentence and the evidence field are corrected; a criterion asks for a scenario of at least 1,000 simulated processes; a Consequence says the simulator's counts are the measure for INT-0035's cost criteria.

### INT-0033 Intent topics and dependencies — Fits
- Topics matter more as the Book grows: an agent finds its intents by lookup, not by reading every chapter.

### INT-0034 Any content — Fits
- Scale needs nothing from content, so the two directions do not pull against each other.
- One gap is in code, not in the chapter: "any bytes of any size" needs streamed reads and writes, and `sim::Fs` reads and writes whole files only ([code audit](code-audit.md)).

## Reversible single-user core

### INT-0006 Format v1 — Revise, large
- **Views.** `heads: set<hashref> ‖ workspaces: map<string, hashref>` is rewritten whole by every operation (finding 2).
- **No size rule.** Nothing says an object's size follows the size of the change it records.
- **Trees.** A changeset names no tree, so a peer can check a tree only by replaying history from the root. That is sound, but a partial clone ([INT-0009](../../../intents/INT-0009-reversible-by-default.md) AC-5) cannot do it. Whether a changeset should commit to a tree root is an open format question; the chapter records it as one to settle before the freeze.
- The kinds involved (`operation`, `changeset`, `view`) are still provisional, so all of this can change without a new format version.
- **Change:** the Intent gains the size rule; a criterion checks it for views; Consequences record the tree question; backlog T-012 grows to cover it.

### INT-0007 Crash-safe storage — Fits (realized; not changed)
- Its five criteria hold and stay the regression suite for any new storage form.
- Its layout is the slowest thing measured ([research report](research-report.md), Measurements): one file per object in two flat directories, two flushes per file, and op heads found by reading every op file.
- The chapter already says "compaction and packing become later work". That work is now INT-0008's, and must pass this chapter's crash matrix and negative controls again.

### INT-0008 Compact history storage — Revise, large
- The chapter covers only how a blob is compressed. No chapter owns how objects are packed, found or flushed.
- AC-6 forbids holding a whole large file in memory, which today's `Fs` trait cannot meet.
- **Change:** the Intent takes on the storage form as a whole: objects written together in one file per command with one flush, an index that is derived data, and streamed reads and writes. Three criteria: flushes per command, cost of opening a large store, and INT-0007's criteria holding on the new form.

### INT-0009 Reversible by default — Revise, large
- `undo` and `restore` are global (finding 4).
- With every snapshot an operation, thousands of agents fill the log quickly. Packing old operations is not erasure, and working operations need not leave their replica (finding 6).
- Code and completion evidence say `none`, though Sprint 3 built AC-1 to AC-3's code.
- **Change:** a criterion scopes `undo` and `restore` to what the actor owns; the Intent separates local from shared operations and says packing removes nothing; evidence fields are filled.

### INT-0013 Derived data — Revise, large
- This chapter is the engine of scale, and nothing of it is built. Every structure INT-0035 needs is on its list: op heads, trees, ancestry, per-file history.
- The chapter says derived data can be recomputed. It does not say it must be kept up to date in proportion to what is new, or that a wrong entry must never be used.
- **Change:** three criteria: incremental upkeep, detection of wrong entries, and every result being the same with all derived data deleted. A Consequence says in-memory tables inside one command are the first instance.

### INT-0014 Working copy and workspaces — Revise, large
- AC-3 tests 8 workspaces. AC-4, the workspace with no checkout, is the main path for agents and is only `supporting`.
- AC-5's time budget was to be "fixed before the first measurement" and never was. Every command still reads and hashes every file.
- **Change:** two criteria: 1,000 workspaces, each cheap to create, and no unchanged file read. AC-5's budget is fixed by reference to INT-0036.

## Rewriting and conflicts

### INT-0010 Lock-free concurrency — Blocks
- Nothing of the canonical merge is built. A command that finds several op heads stops, and no command joins them.
- Measured this sprint: two `praps new` commands that overlap leave two heads, and every later command fails. Eight started together left eight heads on Linux, and two or three on Windows, where four or five of the eight failed ([research report](research-report.md), Measurements).
- The text does not say whether a merge is stored as an operation (finding 3).
- AC-5 tests "at least 8 writers on at least 2 replicas".
- **Change:** the Intent says a merge is computed and never an operation; three criteria: readers add no heads, 1,000 writers on 100 replicas lose nothing, and merge cost follows the difference. The merge itself is the next sprint's work.

### INT-0011 Change identity and rewriting — Revise, small
- Divergence is resolved "with prompts for humans". At this scale a person cannot be asked each time.
- **Change:** every kind of divergence has a default rule that needs no person; a person is asked only where policy says so. One criterion checks it.

### INT-0012 Conflicts and merge — Revise, small
- The resolution store is shared, so one agent's resolution is replayed for every agent, and only the last two merge stages need a confirmation.
- **Change:** a replayed resolution carries its resolver, and policy names whose resolutions replay without a confirmation. One criterion checks it.

### INT-0023 Query language — Revise, small
- AC-4 bounds ancestry queries only. Over 10⁸ changes, any predicate that scans is unusable, and Gate 2 selects batches by query.
- **Change:** a criterion says the reference queries are answered from indexes.

## Agent-native layer

### INT-0015 Phases and confirmation — Revise, small
- The charter already says verification, not generation, limits throughput. The chapter does not say verifiers can be many.
- **Change:** a Consequence says policy may name any number of verifier keys, that confirmations are events and need no coordination, and that INT-0035 measures verifier throughput.

### INT-0016 Actors and agent lifecycle — Revise, small
- Signatures say who made an operation, not what that actor may change (finding 5).
- Thousands of short-lived agents mean thousands of births; the operator's key signs each one.
- **Change:** the Intent gains authority over view entries, with one criterion; a Consequence covers delegated spawning.

### INT-0017 Event records and the Book view — Revise, small
- One record per event already scales: it is why concurrent appends never conflict.
- The collision policy is "still to be made", and the examples given (leases) need a clock or a coordinator, which the direction rules out.
- **Change:** a default claim rule that needs neither, offered for the owner's decision; one criterion at 1,000 agents.

### INT-0018, INT-0019, INT-0020 Proof pairing, verifier, intent calculus — Fit
- Each check is per change and runs in parallel across changes.
- Their cost sets the system's throughput, which INT-0035 measures.

### INT-0021 Lanes and review requests — Revise, large
- "The queue" assumes one integrator (finding 7).
- The simulation behind AC-1 and AC-2 used 300 and 1,000 review requests and 8 and 16 workers. The window is capped at 32.
- **Change:** the Intent says any verifier may run a queue and what happens when two promote at once; two criteria: the simulation at 10,000 requests, and concurrent promotions never landing an untested combination.

### INT-0022 Agent and human interfaces — Revise, large
- The registry it requires does not exist: `praps` matches command names by hand.
- No criterion covers help, suggestions or error messages. `praps` alone prints a usage error.
- Thousands of agents on one machine each start a process and load the repository; nothing lets them share one.
- **Change:** criteria for help generated from the registry and for errors that say what to do; a Consequence for an optional local service. The command list moves to INT-0037, and the screen for people to INT-0038.

### INT-0026 Sprint Loops on Periapsis — Fits
- AC-1's three agents is the floor for self-hosting. INT-0035 checks scale.
- The head-count budget per intent is the throttle on exploration, and it needs no change.

### INT-0027 Porting Sprint Loops projects — Fits

### INT-0032 Intent revisions and review flags — Revise, small
- An agent must learn that its intent changed before it finishes. That needs intent records to reach every replica first.
- **Change:** a Consequence says every replica follows the Book's records, whatever else it follows.

## Distribution

### INT-0024 Staged sync — Revise, large
- Byte cost already follows what is missing (AC-1), which is the right shape.
- Missing: what a replica may choose not to follow (finding 6), how thousands of peers find and relay, and what stops one actor flooding the rest. AC-2 simulates three peers.
- **Change:** the Intent gains scoped replication and admission by actor; three criteria: cost follows what is followed, a 1,000-peer simulation, and a flood dropped at stage 1.

### INT-0025 Git bridge — Fits

## Scale and assurance

### INT-0029 Reproducible releases — Fits
- A release profile tuned for speed must stay reproducible. Nothing in the chapter prevents it.

### INT-0030 Optional scale mode — Revise, large
- The chapter makes scale optional and says it "should not shape the Phase 1–4 formats". The direction reverses both.
- **Change:** the segmented graph index, per-file history, lazy fetch and sparse checkout move to the chapters that own them (INT-0013, INT-0023, INT-0024, INT-0014). What stays optional is what truly is: servers as caches, and server-side pushrebase.

## Identity

### INT-0031 Visual identity — Fits
- INT-0038's screens take their look from this chapter once the owner approves it.

## Abandoned

### INT-0028 Entropy layer — Not applicable
- Nothing in the direction reopens it.

## Stale statements found in passing

Not about the direction, but wrong today:

| Where | Statement | What is true |
|---|---|---|
| INT-0004 Consequences | `undo`, `restore` and `new` drop their snapshot's report (T-019) | Closed in Sprint 4 |
| INT-0005 Consequences | `RandomState`, `rand` and `uuid` are not yet banned (T-014) | Banned in Sprint 3 |
| INT-0005 header | Documentation evidence: none | `docs/spec/simulation.md` |
| INT-0009 header | Code evidence: none | `crates/workspace` (Sprint 3) |
| INT-0001 intent map | Omits INT-0032 and INT-0033 | Both exist since Sprint 0 |
| `docs/spec/storage-durability.md`, "What remains asserted" | Writers in separate processes are not yet tested; a process crash is not simulated | Both done in Sprint 4 |
| `docs/spec/working-copy.md`, "Snapshot" | Names `new` and `status` as the commands that snapshot | `log`, `undo` and `restore` do too |
