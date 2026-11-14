Finalized - DO NOT EDIT

# Sprint 7 Build Plan

The owner approved this plan's outline on 2026-11-14. Four decisions were made with it, and the [research report](../sprint-research/research-report.md) gives the reasons for the sprint:
- each workspace's entry belongs to that workspace (INT-0016 AC-8);
- the commands are named `lane` and `lanes`;
- after two commands at once in one workspace, the next command continues from one change and keeps the other;
- during Test, `dev` is pushed, the PR is opened, and every job of the tested head is re-run twice.

The plans were then revised after the three plan critiques ([critique-round1.md](critique-round1.md), [critique-round2.md](critique-round2.md), [critique.md](critique.md)). The tasks run in the order they appear below: T-001, T-002, T-003, T-004, T-006, T-005, T-007, T-008, T-009, T-010. T-006 comes before T-005 because scoped `undo` and `restore` are tested with lanes. Every task's commit passes `cargo ci` on Windows.

## Intents
- [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) — state: planned; acceptance criteria covered:
  - AC-1 (convergence), over every generated history; AC-4 (criss-cross histories); AC-6 (wrong merges rejected); AC-7 (readers add nothing). In T-003, T-004 and T-009.
  - AC-2 (two heads equal the three-way merge), for two heads neither of which carries a merged copy;
  - AC-3 (grouping), for writers that diverge from one base whose pointer holds one changeset. Both limits are recorded in the chapter with their values (T-003).
  - AC-5 (8 writers on 2 replicas, no lost work), with replicas simulated as two stores that exchange every operation, and a reachability oracle, in T-009.

  Not AC-8 (1,000 writers on 100 replicas) and not AC-9 (merge cost); backlog.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — state: active; acceptance criteria covered: AC-3 (at least 8 workspaces at once over one store), from T-009's one-store run, with T-006 showing them in turn and T-010 through the binary; AC-1 kept.
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — state: active; acceptance criteria covered: AC-6 (undo is scoped), over 200 seeded interleavings of two lanes (T-005); AC-1 to AC-3 kept.
- [INT-0006](../../../intents/INT-0006-format-v1.md) — state: active; acceptance criteria covered: AC-1 for the two changed provisional bodies; AC-6, the owner asked to sign off the change (T-002).
- [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md) — state: planned; acceptance criteria covered: AC-8 for one class of owner, the workspace (T-001). Its ownership rule is applied when heads are merged (T-003). AC-7 needs signed operations and is not covered.
- [INT-0037](../../../intents/INT-0037-command-set.md) — state: active; acceptance criteria covered: AC-1 to AC-5 for `lane` and `lanes`, AC-3 among them from below, and AC-4 for every refusal this sprint adds (T-005 to T-007); AC-8 for those two names (T-001).
- [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) — state: active; acceptance criteria covered: AC-8, AC-9 and AC-11 kept for the two new commands (T-007).
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered: AC-1, AC-3 and AC-6 for the new Concurrency scenario (T-009); AC-1, AC-2 and AC-3 for making a lane, with a crash at every syscall in both modes over 40 seeds and the control `LaneFileInPlace` (T-006); the five existing matrices kept (T-002, T-004 to T-008).
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered: INT-0003 AC-2 to AC-4 and INT-0004 AC-1 for Sprint 6's last runs (T-001); INT-0003 AC-5 and AC-10 (T-010); INT-0003 AC-11's three samples, in Test.

## Schema Tree
- Sprint Goal: several agents share one repository
  - Records
    - T-001: Record Sprint 6's last runs, the dependency update, and the owner's two sign-offs
  - Format
    - T-002: An operation names its workspace, and a view holds one entry per workspace
  - Merge
    - T-003: The canonical merge of op heads
    - T-004: Commands work from merged states
  - Workspaces
    - T-006: Workspaces over one store, in the library
  - History
    - T-005: Undo and restore are scoped to their workspace
  - Commands
    - T-007: `praps lane DIR` and `praps lanes`
  - Simulation
    - T-008: The scheduler wakes only the process it chose
    - T-009: The Concurrency scenario
  - End to end and documentation
    - T-010: Lanes at once through the binary; spec, README, credits and backlog

## Deferred, with reasons
- **Ownership against forged operations, and reporting refused effects** (INT-0016 AC-7; INT-0035 AC-6). Without signatures a process can name any workspace, so the merge's ownership rule stops honest mistakes, not a hostile agent. A refused effect is not reported yet either: finding one means recomputing every operation's parents, a cost this sprint does not take on. Signed operations are the actors' sprint.
- **A view whose size follows the change** (INT-0006 AC-8, AC-9; INT-0035 AC-4). Every operation still writes every entry. Backlog T-030.
- **Merge cost and scale** (INT-0010 AC-8, AC-9; INT-0035 AC-1 to AC-3, AC-5). Every command still reads the whole op log, and a merge walks it again. Backlog T-030 and T-032.
- **Real replicas** (INT-0024). Two stores in one simulated filesystem exchange their files. That is enough to show that replicas holding the same operations compute one view. It is not a sync protocol, and it exchanges every operation, because operations carry no scope yet (INT-0009, INT-0010).
- **Merging file contents** (INT-0012). A pointer two commands moved at once is resolved by the race rule. Two versions of one change both stay visible; their policy is INT-0011's.
- **Two commands that write one working copy's files at once** (critique C-007). `undo` and `restore` run at the same moment in one directory can interfere through the record and the files. One working copy is one agent's; agents that work at once take a lane each. Per-process records or a lock would close it. INT-0014 records it.
- **Two lanes made at once with one name** (critique C-008). Both pass the check, and both directories then share one entry. Lanes are made one at a time by whoever sets them up; detecting the collision needs each lane to know the operation that made it. INT-0014 records it.
- **The `init` that hides a lost history** (backlog T-036, first item): unrelated to the merge.

## Execution Sequence

### T-001: Record Sprint 6's last runs, the dependency update, and the owner's two sign-offs
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md), [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** docs/sprints/s7/sprint-tests/ci-sprint6.md (new), docs/sprints/s7/sign-offs.md (new), docs/intents/INT-0016-actors-and-agent-lifecycle.md, docs/intents/INT-0037-command-set.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:**
  - INT-0003 AC-2 (a push to `main` is never cancelled or replaced), AC-3 and AC-4, for Sprint 6's commits after its tested head and for its merge;
  - INT-0004 AC-1 on GitHub's runners;
  - INT-0016 AC-8 and INT-0037 AC-8, each for what the owner approved.
- **Success criterion (EARS):**
  - **WHEN** the record is written, **THEN** `ci-sprint6.md` **SHALL** name every run of PR #8 after its tested head `bbffce6`, and the run of its merge into `main` (`aab90fd`). For each run and each of its jobs it **SHALL** give, from `gh api`:
    - event, head, attempt and conclusion;
    - start and end.

    For each `cargo ci` job it **SHALL** also give the `test:` and `ci:` lines from the job's own log.
  - **WHEN** the run on `main` is recorded, **THEN** the record **SHALL** say whether any later push cancelled or replaced it.
  - **WHEN** the dependency update is recorded, **THEN** the record **SHALL** name PR #9, its five checks' conclusions at its head, the owner's approval given before the sprint began, and the merge commit.
  - **WHEN** the sign-offs are recorded, **THEN** `sign-offs.md` **SHALL** quote, word for word, the question and the answer chosen for:
    - INT-0016 AC-8, the workspace as a class of owner;
    - INT-0037 AC-8, the names `lane` and `lanes`.

    Each criterion **SHALL** gain a detail line linking that record and saying what it covers. No other criterion **SHALL** be marked signed.
  - **WHEN** the task is done, **THEN** backlog T-035 **SHALL** no longer carry Sprint 6's runs after its tested head or its merge. It **SHALL** still carry:
    - INT-0003 AC-5 and AC-7;
    - the names of INT-0037 AC-8's table that are not built;
    - each later change to the format specification (INT-0006 AC-6);
    - the ten sign-offs in chapters still `proposed`, now nine with INT-0016 AC-8 covered for one class;
    - INT-0003 AC-11's three samples, until Test takes them.
- **Notes:** The owner's answers were given on 2026-11-14 in the Plan, before the plan's approval, as answers to separate questions. Approving the plan is not a sign-off; these two answers are.

### T-002: An operation names its workspace, and a view holds one entry per workspace
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** docs/spec/format-v1.md, crates/codec/src/operation.rs, crates/codec/src/view.rs, crates/codec/src/references.rs, crates/codec/src/error.rs, crates/codec/src/lib.rs, crates/codec/tests/golden.rs, crates/codec/tests/spec.rs, crates/codec/tests/objects.rs, crates/codec/tests/properties.rs, crates/codec/tests/mutants.rs, crates/codec/tests/golden/v1/ (view.bin, view-conflict.bin, operation-root.bin, operation-merge.bin), crates/store/src/lib.rs, crates/store/src/fsck.rs, crates/store/tests/ (every file that builds an operation or a view), crates/workspace/src/lib.rs, crates/workspace/tests/ (likewise), crates/praps/tests/cli.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 AC-1 (one canonical encoding, pinned by golden files) for the two changed bodies, and AC-6 (the owner asked to sign off the change); INT-0005 AC-1 to AC-3 for the five matrices, whose cell counts must not move.
- **Success criterion (EARS):**
  - **WHEN** an operation is encoded, **THEN** its body **SHALL** be `parents: set<hashref> ‖ workspace: string ‖ view: hashref`, and a body whose workspace is empty **SHALL** be rejected with its own error.
  - **WHEN** a view is encoded, **THEN** its body **SHALL** be `workspaces: map<string, entry>` with `entry = working_copy: set<hashref> ‖ heads: set<hashref>`. An empty workspace name and an empty `working_copy` **SHALL** each be rejected with its own error.
  - **WHEN** a view's references are listed, **THEN** they **SHALL** be each entry's `working_copy` and then its `heads`, entries in name order, each to be a changeset.
  - **WHEN** the worked examples are checked, **THEN** `view`, `view-conflict` (one entry whose `working_copy` holds two changesets), `operation-root` and `operation-merge` **SHALL** each match its golden file byte for byte. The specification's text and examples **SHALL** be in the same commit as the code.
  - **WHEN** the suite runs at this task's commit, **THEN** every test **SHALL** pass, and:
    - the store, four-writer, Snapshot, Materialise and Describe matrices **SHALL** have the cell counts they have at the commit before, with their new digests asserted as literals;
    - every correct variant **SHALL** find 0 violations and every negative control **SHALL** find at least one;
    - `docs/reach.md` **SHALL** be unchanged.
  - **WHEN** a repository has one workspace, **THEN** every command **SHALL** print, exit and leave files as before this task. The praps tests change only where they build operations or views by hand.
  - **WHEN** the specification change is written, **THEN** the owner **SHALL** be asked to sign it off (INT-0006 AC-6), and a sign-off **SHALL** be recorded only from the owner's own answer.
- **Notes:**
  - Both bodies are provisional (the specification's Change control): body, worked examples, golden files and encoder change together in one commit.
  - The repository-wide `heads` set goes; each entry carries its own.
  - A field adds bytes, not syscalls, so no cell count should move. A control's violation count may move with the bytes and is recorded.
  - The workspace crate keeps refusing several heads until T-004; this task changes the shape of a view, not what a command does.

### T-003: The canonical merge of op heads
- **Intent:** [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md)
- **Touches:** crates/workspace/src/merge.rs (new), crates/workspace/src/lib.rs, crates/workspace/tests/merge.rs (new)
- **Depends on:** T-002
- **Acceptance criterion:** INT-0010 AC-1; AC-2 for two heads neither of which carries a merged copy; AC-3 for writers that diverge from one base whose pointer holds one changeset; AC-4 and AC-6. INT-0016's ownership rule, applied when heads are merged.
- **Success criterion (EARS):**
  - **WHEN** the merge is given one head, **THEN** it **SHALL** return that head's view.
  - **WHEN** it is given several heads, **THEN** it **SHALL**, for each workspace name, find that workspace's writers: the operations made in that workspace, among the heads and their ancestors, that no other operation made there follows. The merged entry **SHALL** be:
    - absent, with no writer;
    - the writer's entry, with one;
    - otherwise the merge of the writers' entries against their base. The base is the entry at their latest common operation made in that workspace; or the merge of those operations' entries when there are several; or absent when there is none.
  - **WHEN** entries are merged against a base, **THEN**:
    - the `working_copy` **SHALL** be the base's when no writer changed it, and otherwise the union of the changed writers' sets;
    - a changeset **SHALL** be among the `heads` when it is in the base and every writer kept it, or is not in the base and some writer has it.
  - **WHEN** the heads given are permuted or repeated, **THEN** the merged view **SHALL** be the same, in every generated history (AC-1).
  - **WHEN** two heads with one latest common ancestor are merged, and neither carries a merged copy (in each head's own history every workspace has one latest operation), **THEN** each entry **SHALL** equal the classic three-way merge of the two head views against that ancestor's view, a conflict shown as the union of the two sides (AC-2).
  - **WHEN** some heads are first merged by an operation that changes nothing else, **THEN** merging the others with that operation **SHALL** give the flat merge (AC-3). This holds in histories where each workspace's concurrent writers diverge from one base whose pointer holds one changeset.
  - **WHEN** at least 4,000 random criss-cross histories are merged, **THEN** each result **SHALL** equal a slow recursive reference merge written apart from it, and an entry its owner resolved after seeing both sides **SHALL** stay resolved (AC-4).
  - **WHEN** another workspace's operation carries an older copy of an entry, **THEN** that copy **SHALL** not change the merged entry (ownership).
  - **WHEN** the cases of the known limit are merged, **THEN** the merge **SHALL** give the values INT-0010 records for them, pinned by hand: a head that carries a merged copy, and nested writers with a value set back.
  - **WHEN** each registered wrong merge is run through the AC-1 and AC-2 property tests, **THEN** each **SHALL** fail at least one of them:
    - pairwise fold and the first-equals-last fast path **SHALL** fail AC-1 and pass AC-2;
    - always-base and union of heads **SHALL** fail AC-2 and pass AC-1;
    - first-side-wins **SHALL** fail both.

    The wrong merge that takes one arbitrary common ancestor **SHALL** fail the AC-4 test (AC-6).
- **Notes:**
  - A pure function over the op log and a way to read views. It writes nothing, and no command calls it until T-004.
  - Only the entry's own workspace writes it, so another workspace's stale copy is never a "change". This is why a single base for all heads is not used: research and Plan found that it gives spurious conflicts whenever an agent that loaded early still has a head.
  - The known limit: a value set back to an earlier one reads as no change against an older base. With a head that carries a merged copy, or writers whose ancestry nests, the merge can then differ from the classic three-way merge. Matching the classic merge there would mean grouping the writers by the heads, and then a commit by another workspace over the same heads would change this entry, which ownership forbids.
  - The first draft registered "keep-duplicates". Writers are a set, and both rules ask "every writer" or "some writer", so a repeated head cannot change a result: that wrong merge cannot be told apart from the merge. "Union of heads", in which a head one writer removed stays because another kept it, takes its place.
  - First-side-wins is the rule research found in Jujutsu for a working-copy commit both sides moved.

### T-004: Commands work from merged states
- **Intent:** [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/src/record.rs, crates/workspace/tests/workspace.rs, crates/workspace/tests/materialise.rs, crates/workspace/tests/materialise_crash.rs, crates/workspace/tests/real_hazards.rs, crates/workspace/tests/history.rs, crates/praps/src/hint.rs, crates/praps/src/commands.rs, crates/praps/tests/cli.rs, docs/spec/working-copy.md
- **Depends on:** T-003
- **Acceptance criterion:** INT-0010's Intent (no command fails because of another writer) and AC-7 at 2 and 8 readers; INT-0014 AC-1 kept; INT-0037 AC-4 kept.
- **Success criterion (EARS):**
  - **WHEN** a command loads a store with several op heads, **THEN** it **SHALL** work from their canonical merge, computed in memory, and **SHALL NOT** fail because there are several.
  - **WHEN** such a command changes nothing (`status` with no edit, `log`, `ops`, `describe` with the text the change has), **THEN** the set of stored operations **SHALL** be exactly what it was. This is checked with 2 and with 8 reader processes on the simulator.
  - **WHEN** such a command changes something, **THEN** it **SHALL** commit one operation made in its own workspace, whose parents are every head it merged and whose view is the merged view with only its own entry changed. That entry's pointer **SHALL** hold one changeset.
  - **WHEN** its own workspace's working-copy pointer holds several changesets, **THEN** the command:
    - **SHALL** continue from the one with the smallest id;
    - **SHALL** keep the others among its entry's heads;
    - **SHALL** print one note that names every one of them.
  - **WHEN** a write of the working copy begins, **THEN** its record **SHALL** name what the write starts from, and its target operation. The record has two forms, both specified and tested: `from <changeset>`, the working-copy changeset; and `from empty`, the empty tree, which T-006 is the first to write. At the next command:
    - **WHEN** resume finds the target present, and still its workspace's only writer, **THEN** it **SHALL** finish the write;
    - **WHEN** another operation of that workspace has followed it or stands beside it, **THEN** resume **SHALL** remove the write's temporary names, drop the record, leave every other file and add a note;
    - **WHEN** the target is absent, **THEN** resume **SHALL** drop the record and touch no file.
  - **WHEN** the task is done, **THEN** no error of the `workspace` crate **SHALL** concern several heads, and the hint table **SHALL** no longer hold one.
  - **WHEN** eight `praps new` run at once in one working copy, as real processes, **THEN** each **SHALL** exit 0, and the next `praps status` **SHALL** exit 0.
  - **WHEN** the suite runs at this task's commit, **THEN** the store, four-writer, Snapshot and Describe matrices **SHALL** keep their cell counts and digests. The Materialise matrix **SHALL** keep its cell count, 34,400, and its digest **SHALL** move, because a record a crash leaves names a changeset. The task's evidence **SHALL** record the new digest and that reason.
- **Notes:**
  - `Loaded` holds the set of heads merged in place of one head.
  - The record's `from` changes meaning, from an operation to a changeset, because a merged state has no single operation. The tree it names is the same F as before.
  - The seventh clause is the measurement that motivated the sprint. Research found 3 of 8 succeeding at `855eb67`.

### T-006: Workspaces over one store, in the library
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** crates/workspace/src/lane.rs (new), crates/workspace/src/lib.rs, crates/workspace/src/record.rs, crates/workspace/src/variant.rs, crates/workspace/tests/lanes.rs (new), crates/workspace/tests/lane_crash.rs (new), crates/workspace/tests/workspace.rs, crates/praps/src/hint.rs, crates/praps/tests/cli.rs, docs/spec/working-copy.md
- **Depends on:** T-004
- **Acceptance criterion:** INT-0014 AC-3 in part (8 workspaces over one store in turn, with no clone and one file per object; at once in T-009) and AC-1 kept; INT-0005 AC-1 to AC-3 for making a lane.
- **Success criterion (EARS):**
  - **WHEN** a lane NAME is made at DIR from workspace W, **THEN** the library **SHALL**, in order:
    1. snapshot W;
    2. refuse unless NAME is no workspace yet and DIR is free (see below);
    3. form the lane's first change: no changes, and as its only parent the parent of W's working-copy change, or no parent when that change has none;
    4. make DIR and `DIR/.praps` durable, then write `DIR/.praps/workspace`, naming NAME and the store, through a temporary name and a rename, durably;
    5. write a record whose `from` is `empty` and whose target is the lane's operation;
    6. commit that operation, made in NAME over every head, whose view adds NAME's entry: the new change as pointer and as head;
    7. write the change's files into DIR;
    8. drop the record.
  - **WHEN** DIR is absent, empty, or holds nothing but a `.praps` an interrupted making left, **THEN** DIR **SHALL** count as free. Such a `.praps` holds no `workspace` file, or one naming a workspace no operation made. Beside it there may be at most the temporary names of `workspace` and of the record (`working-copy.new`), and the record itself.
  - **WHEN** NAME is already a workspace, or DIR is not free, **THEN** making the lane **SHALL** refuse, having written nothing but W's snapshot.
  - **WHEN** `Workspace::open` is given a directory with no `.praps/store` but with `.praps/workspace`, **THEN** it **SHALL** open the store that file names, as workspace NAME. Where `.praps/store` is present, `open` **SHALL** make the syscalls it makes today and no more.
  - **WHEN** a lane is opened whose `.praps/workspace` names a workspace that no operation made, **THEN** the command **SHALL** drop the record if one is left, as resume drops one whose target is absent, and refuse with its own error naming the lane. Nothing else **SHALL** be written. Its hint **SHALL** be a `Fix` that names no command: make the lane again at that directory, from the working copy. T-007 makes it a `Run` of `praps lane DIR`.
  - **WHEN** the store `.praps/workspace` names does not open, **THEN** the command **SHALL** refuse with an error of its own; **WHEN** the file cannot be read or does not parse, **THEN** with another, which gives the reason. Neither hint **SHALL** offer `init`.
  - **WHEN** this task is done, **THEN** `workspace::Error` **SHALL** have five new kinds, each with a hint: a name in use, a DIR that is not free, an interrupted lane, a store that does not open, and a `.praps/workspace` that cannot be read or does not parse.
  - **WHEN** the system crashes at any syscall of making a lane, for 40 seeds in the lenient and the strict mode, **THEN**, at the next command in DIR:
    - if the lane's operation landed, the lane's files **SHALL** be finished, and making the lane again **SHALL** refuse because NAME is in use;
    - if it did not, the command **SHALL** refuse as an interrupted lane, or, before `.praps/workspace` is in place, find no repository at DIR, and making the lane at DIR again **SHALL** then succeed.

    In the cell after the last syscall the lane **SHALL** be complete: its operation present, its files written, no record left. Two runs **SHALL** give every cell the same trace and state, and the cell count and digest **SHALL** be asserted as literals.
  - **WHEN** the variant `LaneFileInPlace`, which writes `.praps/workspace` in place with no temporary name and no sync, runs the same matrix, **THEN** it **SHALL** produce at least one violation.
  - **WHEN** 8 lanes over one store run commands in turn, **THEN**:
    - every operation **SHALL** be in that one store, and no lane **SHALL** hold a store of its own;
    - each object **SHALL** be one file;
    - each lane's entry **SHALL** hold its last command's state;
    - each lane's files **SHALL** equal its working-copy change.
  - **WHEN** this task adds an error kind, **THEN** the kind **SHALL** have a hint in the table, and the workspace crate's `messages_say_only_what_is_wrong` **SHALL** fix its text. That test's check that a text gives no advice **SHALL** look for a command (`praps ` or `` `praps``) rather than the bare word, so that a text may name `.praps/workspace`. T-007 prints each kind through the binary.
  - **WHEN** the suite runs at this task's commit, **THEN** the five matrices **SHALL** keep their cell counts and digests.
- **Notes:**
  - Before `.praps/workspace` is in place, DIR holds at most an empty `.praps` or a temporary file, and nothing in the store names it: it is no repository. A command there says so, as in any directory, with the same hint, and INT-0014 records why that hint stays. Making the lane again at DIR finishes it, because DIR counts as free.
  - `.praps/workspace` holds two lines, `name NAME` and `store PATH`. `praps` writes the store's full path, as a linked Git worktree does; the simulator uses paths from its root.
  - A lane's `.praps` holds no store, so it is no clone.
  - The lane starts from the parent of W's working-copy change because W rewrites that change at every snapshot and `describe`. A lane made on it would sit on a version W has already replaced, and the change would diverge (INT-0011). W's edits in progress are therefore not in the lane.
  - A lane is a checkout: every file is written into DIR. The cheap, API-only workspace (INT-0014 AC-4) is still to come.

### T-005: Undo and restore are scoped to their workspace
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/tests/history.rs, crates/workspace/tests/materialise.rs, crates/workspace/tests/scoped.rs (new), crates/workspace/tests/workspace.rs, crates/praps/src/hint.rs, crates/praps/src/commands.rs, crates/praps/tests/cli.rs, docs/spec/working-copy.md
- **Depends on:** T-006
- **Acceptance criterion:** INT-0009 AC-6 (undo is scoped), over 200 seeded interleavings of two lanes; AC-1 to AC-3 kept.
- **Success criterion (EARS):**
  - **WHEN** `undo` runs in workspace W, **THEN** it **SHALL**:
    - take W's writers as they were when it started, before its own snapshot;
    - return W's entry to its value just before them: W's entry in the merge of their parents.

    No other entry **SHALL** change.
  - **WHEN** `undo` runs twice in a row, **THEN** W's entry **SHALL** return to its value before the first.
  - **WHEN** W's writer is the operation that made W, or the root, **THEN** `undo` **SHALL** fail with nothing to undo, having written only its snapshot. Its text **SHALL** become "nothing to undo: this workspace's last op is the one that made it", which holds for the root, the op that made `default`, and for a lane's first operation.
  - **WHEN** `restore OP` runs in W, **THEN** W's entry **SHALL** become its value in OP's view, and no other entry **SHALL** change.
  - **WHEN** OP's view has no entry for W, **THEN** `restore` **SHALL** refuse with an error of its own, with nothing written but its snapshot. It is the one kind this task adds.
  - **WHEN** the entry `undo` or `restore` would set has a pointer of several changesets, **THEN** the command **SHALL**:
    - write the smallest, and commit it alone as the pointer;
    - keep the others among the heads;
    - print the race note.
  - **WHEN** `undo` succeeds, **THEN** it **SHALL** report the operations it went back to, the parents of the writers it undid. With one parent it **SHALL** print today's line; with several, `undid the last op: the working copy is back to ops X, Y`, every parent in id order.
  - **WHEN** a repository has one workspace and the target is one operation's own view, **THEN** `undo` and `restore` **SHALL** name that view rather than write it again, and so leave the same view object as before this task. The history tests' assertions on view ids stand unchanged. **WHEN** the target is the merge of several operations' views, **THEN** the command **SHALL** write it as a new view.
  - **WHEN** two lanes run 200 seeded interleavings of edits, `describe`, `new`, `undo` and `restore`, **THEN**, after each `undo` or `restore` by one lane, every entry the other owns **SHALL** equal what the merge gave just before, and its files **SHALL** be as they were (AC-6).
  - **WHEN** this task adds an error kind or changes a text, **THEN** the kind **SHALL** have a hint in the table, and the workspace crate's `messages_say_only_what_is_wrong` **SHALL** fix its text. T-007 prints the new kind through the binary.
  - **WHEN** the suite runs at this task's commit, **THEN** the five matrices **SHALL** keep their cell counts and digests.
- **Notes:**
  - "The last operation this workspace made" is its writers, the same notion the merge uses.
  - With one workspace the new view is the old target view byte for byte. The command names that view instead of writing it again, so it makes no new syscall, and the old assertions are the check that nothing else moved.

### T-007: `praps lane DIR` and `praps lanes`
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/src/lane.rs, crates/praps/src/registry.rs, crates/praps/src/help.rs, crates/praps/src/hint.rs, crates/praps/src/place.rs, crates/praps/src/commands.rs, crates/praps/src/main.rs, crates/praps/tests/cli.rs, README.md, docs/spec/working-copy.md
- **Depends on:** T-005
- **Acceptance criterion:** INT-0037 AC-1 to AC-5 for the two commands, AC-3 among them from below, and AC-4 for every refusal T-005 and T-006 added; INT-0022 AC-8, AC-9 and AC-11 kept.
- **Success criterion (EARS):**
  - **WHEN** `praps lane DIR` runs in a working copy, **THEN** it **SHALL** make workspace NAME, the last component of DIR, at DIR, over this repository's store named by its full path. It **SHALL** print one line naming NAME, DIR and the new change.
  - **WHEN** any of these holds, **THEN** `praps lane` **SHALL** exit 1, write nothing but the snapshot, and end with one hint:
    - DIR lies inside a working copy, this one or another, found by searching up from DIR's parent with the top rule below (DIR itself is judged by T-006's rule for a free DIR, which accepts an interrupted lane);
    - NAME is empty, `.`, `..`, or not UTF-8;
    - NAME is already a workspace;
    - DIR is not free.
  - **WHEN** `praps lanes` runs, **THEN** it **SHALL** snapshot like every command, then list every workspace in name order, one line each:
    - `*` for this one;
    - the name;
    - the change ID of its working copy, and the number of paths that change changes;
    - the other changes of a pointer in conflict.

    With no edit it **SHALL** write nothing and repeat exactly.
  - **WHEN** any command runs in a lane, at its top or below it, **THEN** it **SHALL** act on that lane over the shared store. A top **SHALL** be a directory whose `.praps/store` lists or whose `.praps/workspace` reads. A `.praps/workspace` that exists but cannot be read or does not parse **SHALL** stop the search with its error.
  - **WHEN** `praps init` runs at a lane's top or below it, **THEN** it **SHALL** refuse.
  - **WHEN** `praps ops` lists operations, **THEN** each line **SHALL** name the workspace that made the operation and show that workspace's change in the operation's view, the smallest when its pointer holds several.
  - **WHEN** the registry is read, **THEN**:
    - `lane` and `lanes` **SHALL** be in a group headed "Working together";
    - the everyday group **SHALL** still hold at most ten;
    - `lanes` **SHALL** be marked idempotent and declare no flag;
    - the README's command table **SHALL** equal the list.
  - **WHEN** any refusal this sprint added is printed (the two commands', `restore` of an operation with no entry for this workspace, the interrupted lane, the lane whose store is gone, an unreadable `.praps/workspace`, `init` in a lane), **THEN** `every_refusal_prints_one_hint` **SHALL** reach it through the binary, and it **SHALL** end with one hint naming only registry commands. The interrupted lane's hint **SHALL** become a `Run` of `praps lane DIR`.
  - **WHEN** `praps lanes` is run at a working copy's top and from below it, **THEN** it **SHALL** print and leave the same, byte for byte (INT-0037 AC-3).
  - **WHEN** `praps lane DIR` is run at a working copy's top and from below it, with DIR given relative to each place, **THEN** the two runs **SHALL** agree as two `new` runs do: the printed line the same once the change ID and DIR are masked, and the same exit code, number of operations, files outside `.praps`, files in the lane, and output of the `praps lanes` that follows. The change ID is drawn at random, so the operation's bytes differ (INT-0037 AC-3).
  - **WHEN** the suite runs at this task's commit, **THEN** the five matrices **SHALL** keep their cell counts and digests.
- **Notes:**
  - "Working together" is the purpose INT-0037's table gives these names.
  - A lane inside a working copy would have its files snapshotted by that working copy, and its `.praps` tracked as content; hence the refusal anywhere inside one.

### T-008: The scheduler wakes only the process it chose
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/sim/src/processes.rs, crates/sim/tests/processes.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0005 AC-1 and AC-2 (the same draws, so the same digests); the research's risk at 1,000 processes.
- **Success criterion (EARS):**
  - **WHEN** a process reaches a boundary, **THEN** the scheduler **SHALL** wake only the process it draws next. The draws **SHALL** be unchanged: every `Processes` test and the four-writer matrix **SHALL** pass with the literals they have at the commit before.
  - **WHEN** 1,000 processes each make three syscalls, **THEN** all 1,000 **SHALL** end done. The run **SHALL** take at most a tenth of its time at the commit before, in one release run on the same machine, recorded.
- **Notes:** Research measured 9 s for that run, about 3 ms a boundary, because every boundary wakes every thread.

### T-009: The Concurrency scenario
- **Intent:** [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/workspace/tests/concurrency.rs (new), crates/workspace/src/variant.rs, crates/workspace/src/lib.rs, crates/workspace/src/lane.rs, crates/workspace/src/points.rs, crates/xtask/src/reach.rs, docs/reach.md, docs/spec/simulation.md
- **Depends on:** T-005, T-008
- **Acceptance criterion:** INT-0010 AC-5 (8 writers on 2 replicas, no lost work) and AC-7 (readers add nothing at 2, 8 and 1,000); INT-0014 AC-3 (8 workspaces at once over one store); INT-0005 AC-1, AC-3 and AC-6.
- **Success criterion (EARS):**
  - **WHEN** 8 lane processes run at once over one store, under the seeded scheduler for 40 seeds, **AND** one process is killed and the system crashes at seeded steps, **THEN**, after recovery:
    - no command that started before the crash, and was not killed, **SHALL** have failed;
    - every acknowledged operation **SHALL** be in the store;
    - each object **SHALL** be one file, and the store **SHALL** check clean;
    - each lane's writer **SHALL** be its last acknowledged operation, or an operation of the command in flight that follows it;
    - every lane **SHALL** compute the same view, and then run one more command.
  - **WHEN** 8 lane processes run at once, four on each of two stores, under the same conditions, **THEN**, after recovery and after the two stores exchange their operations and objects:
    - every acknowledged operation **SHALL** be on both stores, and both stores **SHALL** check clean;
    - each lane's writer **SHALL** be as above;
    - for each lane whose writer is its last acknowledged operation, every changeset in that entry, pointer and heads, **SHALL** be in the merged view on both stores, with everything it follows present;
    - for each lane whose command in flight landed an operation, that operation **SHALL** follow the last acknowledged one, and every changeset the last acknowledged entry held **SHALL** still be in the store and named by that entry's view, which the op log keeps, so `restore` can return to it;
    - every lane, on either store, **SHALL** compute the same view, and then run one more command.

    Each process runs a fixed script: edits recorded by `status`, `describe`, `new`, and `undo`.
  - **WHEN** a run is repeated, **THEN** every cell's trace and state **SHALL** be equal, and each run's cell count and digest **SHALL** be asserted as literals.
  - **WHEN** each of these runs the scenario, **THEN** each **SHALL** produce at least one violation:
    - the workspace variant `FirstHeadOnly`, which loads only its first head;
    - the variant `MergeOnRead`, which commits a merge when it loads several heads;
    - the store's `UnsyncedObjects`.
  - **WHEN** 2, 8 and 1,000 reader processes load a store with several heads and run only commands that change nothing, **THEN** the stored operations **SHALL** be exactly as before, and every reader **SHALL** compute the same view. `MergeOnRead` **SHALL** fail this.
  - **WHEN** a command merges several heads, **THEN** it **SHALL** mark `workspace.merge.computed`; **WHEN** a lane is made, **THEN** it **SHALL** mark `workspace.lane.made`. Both **SHALL** be registered in Sprint 7, for units `workspace::merge` and `workspace::lane`.
  - **WHEN** the reach table is generated, **THEN** the scenario Concurrency, added in Sprint 7, **SHALL** reach both points, and `docs/reach.md` **SHALL** show them.
- **Notes:**
  - Two stores in one simulated filesystem stand in for two replicas. Their exchange copies what each lacks through the store's own `put_object` and `commit_op`, parents and references first. It exchanges every operation, because operations carry no scope yet.
  - A system crash stops every process, so commands after it fail with the crash and are not counted.
  - The points are added here, with the scenario that reaches them, so that the reach step passes at every task's commit. Under several processes a point is a scheduling boundary; no earlier matrix merges heads or makes a lane, so none moves.
  - INT-0005 AC-2 (a crash at every syscall) is claimed for making a lane (T-006), not for this scenario's seeded crash steps.

### T-010: Lanes at once through the binary; spec, README, credits and backlog
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** crates/praps/tests/e2e.rs, docs/spec/working-copy.md, README.md, docs/prior-art.md, crates/xtask/src/docs.rs, docs/work/tasks.md
- **Depends on:** T-007, T-009
- **Acceptance criterion:** INT-0014 AC-3 through the binary on NTFS and ext4; INT-0037 AC-2 and AC-3 for a lane session; INT-0003 AC-5 and AC-10.
- **Success criterion (EARS):**
  - **WHEN** 8 lanes are made with `praps lane`, one at a time, and 8 real processes then run `status`, `describe` and `new` in them at once, **THEN**:
    - every command **SHALL** exit 0;
    - `praps lanes` **SHALL** then print the same lines in every lane, apart from the `*`, which marks each lane's own line;
    - each lane's change **SHALL** carry its own description;
    - `praps verify` **SHALL** print `clean`.
  - **WHEN** a session uses only words `praps` printed (`lane`, `lanes`, `status`, `describe`, `new`, `log`, `ops`, `undo`, `restore`, `verify`), **THEN** every command **SHALL** exit 0 and print what its step names. `undo` in one lane **SHALL** leave the other's files and entry as they were.
  - **WHEN** `docs/spec/working-copy.md` is read, **THEN** it **SHALL** describe each of these:
    - the view's entries and who writes them;
    - the merge;
    - commands that change nothing write nothing;
    - the race rule;
    - scoped `undo` and `restore`;
    - lanes and `.praps/workspace`;
    - the record's two forms;
    - resume's rule.

    No section **SHALL** still say that several heads stop a command.
  - **WHEN** `docs/prior-art.md` is read, **THEN** it **SHALL** credit Git's linked worktrees for linked working copies, and Git's merge strategies for the virtual merge base.
  - **WHEN** the task is done, **THEN** `docs/work/tasks.md` **SHALL** no longer hold T-029. It **SHALL** hold what T-029 leaves:
    - ownership against forged operations, and the reporting of refused effects (INT-0016);
    - a view whose size follows the change (INT-0006 AC-8, AC-9; INT-0035 AC-4);
    - merge cost (INT-0010 AC-9);
    - 1,000 writers on 100 replicas (INT-0010 AC-8);
    - the merge's known limit (INT-0010 AC-2 and AC-3);
    - two commands that write one working copy's files at once, and two lanes made at once with one name (INT-0014);
    - what the ownership rule implies once local operations stay on their replica (INT-0009, INT-0010).
- **Notes:** The real-process test has no fixed interleaving; what it asserts holds under any interleaving.
