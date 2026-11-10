# Sprint 6 Build Plan

The owner approved this plan's outline on 2026-11-10, with three choices: `verify` as the name of the integrity check; `log` for changes and `ops` for operations; and, on GitHub, one run on demand in T-001 and the sprint's pull request opened during Test.

The outline had eight tasks. Two plan critiques changed its shape:
- "a command reads the op log once" became T-003, T-004 and T-005, so that each commit has one outcome;
- "`describe`, `log`, `ops` and `verify`" became T-007, T-009 and T-010, for the same reason;
- T-012 was added. Running commands from inside the working copy, which the outline asked for, makes one failure common: a write that cannot remove the directory a shell stands in. T-012 keeps that from stopping the write.

The backlog entries the outline put in its last task were written during Plan instead, because the revised chapters already cite them.

## Intents
- [INT-0037](../../../intents/INT-0037-command-set.md) — state: planned; acceptance criteria covered:
  - AC-1: every command name is a plain word, and none is a refused word (T-006, T-007);
  - AC-2: `praps` lists its commands, explains each, and answers a mistyped word (T-006);
  - AC-3: a command works from any directory inside a working copy, and `init` does not nest (T-008, T-012);
  - AC-4: every refusal says what to do next (T-011);
  - AC-5: the everyday group holds at most ten commands (T-006).
- [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) — state: planned; acceptance criteria covered: AC-8 (no flag on an idempotent command) and AC-11 (one description of each command), in T-006. AC-9 (an idempotent command repeats exactly) is checked for the commands built so far and stays open for the rest. AC-4 to AC-7 are not met by the listing T-009 builds; the chapter records that.
- [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) — state: planned; acceptance criteria covered: none in full. T-003 is a first step toward cost rule 2, and T-009 removes one cost that grew with the square of the history.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — state: active; acceptance criteria covered: AC-1 kept (no lost edits) through T-003, T-004, T-005, T-008, T-010 and T-012.
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — state: active; acceptance criteria covered: AC-1, AC-2 and AC-3 kept, with one scan before a write in place of two (T-005), the operation list under its new name (T-009), `describe` added to the operations that `undo` and `restore` reverse (T-010), and a write that leaves a directory it cannot remove (T-012).
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered: AC-1 to AC-3 and AC-6. Two matrices must not move at all (T-002 onward), two are recorded again (T-003, T-005) and must then stay as they are (T-004, T-012), and one scenario and one named point are added (T-010).
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: active; acceptance criteria covered: AC-2 (the `main` and on-demand halves, by record), AC-3, AC-4, AC-5 and AC-10. AC-11 is not measured again: its threshold is three samples of one commit, which need two re-runs of every job, and those are not among the GitHub actions the owner approved. The times of single runs are recorded.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered: AC-1, on both platforms for every task; AC-7 kept for the two commands this sprint adds (T-009, T-010).
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md) — through T-001 only: the state of the owner's sign-offs.

## Schema Tree
- Sprint Goal: write the owner's direction of 2026-11-09 into the Book, and take its first steps in code
  - Carried forward
    - T-001: Record Sprint 5's GitHub Actions runs, one run on demand, and the owner's sign-offs
  - Cost
    - T-002: Remove duplicated helpers, with every matrix digest unchanged
    - T-003: A command reads the op log once
    - T-004: `open` checks the op log before it resumes an interrupted write
    - T-005: `undo` and `restore` scan the working copy once
  - Use
    - T-006: The command registry: dispatch, help and suggestions from one table
    - T-007: `verify` replaces `fsck`
    - T-008: Commands find their working copy from any directory inside it
    - T-009: `log` lists changes and `ops` lists operations
    - T-010: `describe` sets the description of the current change
    - T-011: Every refusal names the next command
    - T-012: A write leaves a directory it cannot remove
  - Documentation
    - T-013: README, specs and credits

## Deferred, with reasons
- **The canonical merge, packed storage, the stat cache and the simulator at scale.** Each needs a sprint of its own. They are backlog T-029 to T-032, in that order, and [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) records why the command set comes first.
- **The tooling findings X-01 to X-11**, among them the two the research report proposed for this sprint: a declared Book with no chapter passes, and a mistyped marker selects the lenient rule unseen (X-02); the `policy` and `book` steps of `cargo ci` print only a count (X-06). The owner's approved outline keeps every tooling finding in one backlog task (T-033). This sprint's use of those two checks is covered without the fixes:
  - the test evidence records the first and last lines of `cargo book` run by itself, on both platforms. The first names the rule applied (`typed-acceptance 1, strict`), and the last counts the chapters and criteria, so neither the lenient rule nor a Book with no chapter could pass unseen;
  - `cargo policy` and `cargo book` are run by themselves, where they print every problem, before each commit.
- **The harness findings S-03 to S-08**: backlog T-034.
- **INT-0003 AC-11's three samples**: see the Intents section. Backlog T-035, which T-001 writes, carries them.
- **Whether a write should spare the directory a command was started in**: backlog T-031. T-012 makes the write survive where the directory cannot be removed, and records what each system does.

## Execution Sequence

### T-001: Record Sprint 5's GitHub Actions runs, one run on demand, and the owner's sign-offs
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** docs/sprints/s6/sprint-tests/ci-sprint5.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0004-windows-and-linux.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:**
  - INT-0003 AC-2 (a push to `main` is never cancelled or replaced; CI runs on demand), AC-3 and AC-4, for the commits Sprint 5 added after its tested head and for its merge;
  - INT-0004 AC-1 on GitHub's runners;
  - the owner's sign-offs still pending.
- **Success criterion (EARS):**
  - **WHEN** the record is written, **THEN** `ci-sprint5.md` **SHALL** name, for run 42721345619 (the pull request at `b0675f6`) and run 42726187298 (the push to `main` at `55bdc07`), each of the following, taken from `gh run view`, `gh api …/jobs` and each job's own log:
    - event, head and conclusion;
    - every job, with its conclusion, attempt and start and end times;
    - the runner image and its version as the log's `Runner Image` group gives them, the `rustc` version, the test count of each `cargo ci` leg and its `ci:` line.
  - **WHEN** the run on `main` is recorded, **THEN** the record **SHALL** say whether any later push cancelled or replaced it. It **SHALL** give the Windows `cargo ci` time of both runs beside AC-11's 20 minutes and its aim of 15, and **SHALL** say that each is one run and not one of AC-11's three samples.
  - **WHEN** the test counts are recorded, **THEN** the record **SHALL** compare them with Sprint 5's tested head (500 on Windows and 501 on Linux) plus the one test added after it.
  - **WHEN** `gh workflow run ci --ref dev` has been started and has finished, **THEN** the record **SHALL** name that run, its event `workflow_dispatch`, its head and every job's conclusion. **IF** the run cannot be started, or has not finished when Test ends, **THEN** the record **SHALL** say so and backlog T-035 **SHALL** carry it.
  - **WHEN** the sign-offs are recorded, **THEN** the record **SHALL** list every acceptance criterion in the Book whose `Signers` detail names the project owner, and INT-0004 AC-5, which earlier sprints carried with them. For each it **SHALL** give the chapter's state and whether the criterion is signed. The record **SHALL** give the search that produced the list. No sign-off **SHALL** be recorded without the owner's words. Approving this plan and merging Sprint 5's pull request count as neither.
  - **WHEN** the task is done, **THEN** `docs/work/tasks.md` **SHALL** no longer hold T-024 or T-025, and **SHALL** hold T-035, which carries: the runs of Sprint 6's commits after its tested head and of its merge; every sign-off the record lists as unsigned that earlier sprints carried, or that belongs to a chapter that is `planned`, `active` or `realized`; and the three samples of INT-0003 AC-11.
- **Notes:**
  - This is backlog T-024 and T-025. `gh run view --job … --log` is not a source for a re-run job (Sprint 5 test critique, C-008); neither run here was re-run.
  - At Plan the search finds 17 such criteria:
    - four that earlier sprints carried, with INT-0004 AC-5: INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5 and INT-0006 AC-6;
    - three more in chapters that are planned or active: INT-0003 AC-7, INT-0035 AC-12 and INT-0037 AC-8;
    - ten in chapters still `proposed`, where a sign-off is due when the chapter is planned: INT-0011 AC-5, INT-0016 AC-8, INT-0020 AC-6, INT-0025 AC-3, INT-0031 AC-1, INT-0032 AC-7, INT-0033 AC-5, INT-0034 AC-8, INT-0036 AC-7 and INT-0038 AC-7.

    So T-035 carries eight.

### T-002: Remove duplicated helpers, with every matrix digest unchanged
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/sim/src/fs.rs, crates/sim/src/lib.rs, crates/hash/src/lib.rs, crates/store/src/lib.rs, crates/store/src/names.rs, crates/workspace/src/lib.rs, crates/workspace/src/materialise.rs, crates/praps/src/main.rs, crates/sim/tests/contract.rs, crates/sim/tests/points.rs, crates/hash/tests/vectors.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0005 AC-1 to AC-3 and AC-6 unchanged; INT-0003 AC-4 (Clippy clean).
- **Success criterion (EARS):**
  - **WHEN** the `Fs` forwarders for `&T`, `Rc<T>` and `Arc<T>` are generated by one macro, **THEN** each of the trait's 16 methods called through each of the three **SHALL** reach the underlying filesystem, and a named point marked through each **SHALL** be counted. `Fs::point` **SHALL** keep no default body.
  - **WHEN** the task is done, **THEN** each of these **SHALL** have exactly one definition outside `tests/`: `parent_dir`; `dir_exists`; the hex form of a string of bytes; the join of a repository path onto a root; the listing of a directory keyed by name.
  - **WHEN** the suite runs at this task's commit, **THEN** every test **SHALL** pass with the four asserted matrices exactly as Sprint 5 left them:
    - store: 32,840 cells, `d36b8c9c…`;
    - four writers: 44,836 cells, `253b1eab…`;
    - Snapshot: 25,360 cells, `c1e39cb1…`;
    - Materialise: 40,960 cells, `6274344b…`.

    `docs/reach.md` **SHALL** be unchanged too.
  - **WHEN** `hash::hex` is given bytes, **THEN** it **SHALL** return two lowercase hex digits for each byte, in order.
- **Notes:**
  - These are findings C-26 to C-31 of the [code audit](../sprint-research/code-audit.md). The unchanged literals are the proof that no syscall moved, which is why this task comes before T-003.
  - Formatting one byte as two hex digits stays where it is not the hex form of a string: a byte quoted in an error message of `codec` or `hash`, an escape in a name's display form in `sim`, and the algorithm id that starts a store file's name. The record of the search lists each such place.

### T-003: A command reads the op log once
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/workspace/src/lib.rs, crates/store/src/lib.rs, crates/workspace/tests/workspace.rs, crates/workspace/tests/crash.rs, crates/workspace/tests/materialise_crash.rs, crates/store/tests/store.rs, docs/reach.md, docs/spec/working-copy.md, docs/work/tasks.md
- **Depends on:** T-002
- **Acceptance criterion:** INT-0014 AC-1 kept; INT-0035 cost rule 2 (a first step; no criterion met); INT-0005 AC-1 to AC-3 and AC-6 for the two matrices recorded again.
- **Success criterion (EARS):**
  - **WHEN** `Workspace::open` runs and one command then runs on that workspace (`snapshot`, `status`, `new_change`, `log`, `undo`, `restore` or `restore_view`), **THEN** the simulator's trace from the start of `open` to the end of the command **SHALL** show:
    - `ops/` listed exactly twice: once by `Store::open`, which lists it to see that it exists, and once to read the log;
    - each op file that was there at the start read exactly once;
    - beyond those, two reads under `ops/` for each operation the command commits: its parent, and its own name.
  - **WHEN** a further command runs on the same `Workspace`, or a command runs on a `Workspace` that `init` returned, **THEN** it **SHALL** list `ops/` exactly once and read each op file once, at its start.
  - **WHEN** another handle commits an operation between `open` and the first command, and that command commits one too, **THEN** the command **SHALL** succeed, both operations **SHALL** be in the log, and the next command **SHALL** stop with `SeveralHeads`.
  - **WHEN** one command on a `Workspace` has finished and another handle then writes to the store, **THEN** the next command **SHALL** start from what is on disk:
    - it **SHALL** stop with `SeveralHeads` if the log now has two heads;
    - it **SHALL** stop with `Corrupt` if an op file was damaged;
    - it **SHALL** build on the new head if the other handle left one.
  - **WHEN** `Workspace::working_copy` or `Workspace::parent_tree` is called, **THEN** it **SHALL** read the store, never the state a command kept, so the oracles that compare them with a model check what is stored.
  - **WHEN** `Store::op_log` runs, **THEN** it **SHALL** list `ops/` once and return the valid ops and the number of other files there whose names do not begin `tmp-`: a name that is not an id, bytes that do not hash to the name, a body that does not decode.
  - **WHEN** the task's diff is read, **THEN** no assertion of an existing test **SHALL** have changed except the Snapshot and Materialise literals.
  - **WHEN** the Snapshot and Materialise matrices run after the change, **THEN**:
    - each **SHALL** find 0 violations, and every negative control (`UnsyncedObjects`, `NoRecord` and the others each file runs) **SHALL** still be caught;
    - each new cell count and digest **SHALL** be asserted as a literal and recorded in the sprint's test evidence beside the old one;
    - `docs/reach.md` **SHALL** be generated again by `cargo reach`;
    - the store and four-writer matrices **SHALL** keep their literals, and `crates/store/tests/` **SHALL** change only by tests added for `Store::op_log`.
- **Notes:**
  - What a `praps` command does today, counted from the code: `Store::open` lists `objects/` and `ops/` to see that they exist, and each load lists `ops/` twice and reads every op file. `status` loads three times. So one `status` lists `ops/` seven times and reads every op file three times.
  - `Store::ops`, `Store::open` and `Store::inspect` are not changed: the store's scenarios call them. `Store::op_log` is added.
  - `Store::open` still lists `objects/` and `ops/` once each to see that they exist. Removing those two listings needs a cheaper question than `Fs` can ask today, and belongs with the storage form (backlog T-030; INT-0008 AC-8).
  - The state is passed through a command's steps. It is kept from `open` only until the first command takes it. No state outlives a command, so a failed command leaves nothing stale.
  - This task changes one behaviour, which [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) records: the first command after `open` does not see an operation written in between. The third clause tests what the chapter promises of that case. Nothing else moves, which the seventh clause checks.
  - After its own commit a command updates the state from what it wrote, building the tree with `tree::of` as a fresh read would.
  - In this task `open` still resumes before it refuses a damaged log, and `undo` and `restore` still scan twice. T-004 and T-005 change those, each with its own evidence.
  - Every command still reads the whole chain of changes to build a tree. That cost follows the history, and it waits for the storage form (backlog T-030).
  - Backlog T-027 quotes Materialise's cell count; its text is updated with the new one.

### T-004: `open` checks the op log before it resumes an interrupted write
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/tests/materialise.rs, docs/spec/working-copy.md
- **Depends on:** T-003
- **Acceptance criterion:** INT-0014 AC-1 kept: a half-written working copy is never recorded as the user's edits. INT-0005 AC-1 unchanged.
- **Success criterion (EARS):**
  - **WHEN** a working-copy record exists and `ops/` holds a file that is not a valid op, **THEN** `open` **SHALL** fail with `Corrupt`, the record **SHALL** be byte for byte as it was, and no file outside `.praps` **SHALL** have changed.
  - **WHEN** that op file's bytes are put back, **THEN** `open` **SHALL** resume, finish the write, and leave the working copy equal to the target tree.
  - **WHEN** the same test is run at T-003's commit, **THEN** it **SHALL** fail there, with the record gone. That run is recorded as the negative control.
  - **WHEN** the four matrices run at this task's commit, **THEN** each **SHALL** keep the literals it had after T-003: the check makes no syscall of its own.
- **Notes:** today resume looks for the record's target among the valid ops only. A target whose file is damaged looks like a write that never began, so the record is dropped, and `open` then fails on the damaged file. With the file repaired, the half-written working copy would be recorded as edits. The chapter records the new order.

### T-005: `undo` and `restore` scan the working copy once
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/tests/history.rs, crates/workspace/tests/materialise_crash.rs, docs/reach.md, docs/spec/working-copy.md, docs/work/tasks.md
- **Depends on:** T-004
- **Acceptance criterion:** INT-0009 AC-2 and AC-3 kept; INT-0014 AC-1 kept; INT-0005 AC-1 to AC-3 and AC-6 for the Materialise matrix.
- **Success criterion (EARS):**
  - **WHEN** `undo` or `restore` runs, **THEN** a tracked file that the move leaves as it is **SHALL** be read exactly once during the command. At T-004's commit it is read twice, and that run is recorded as the negative control.
  - **WHEN** a file was edited before the command, **THEN** the edit **SHALL** be recorded in an operation of its own before the operation that moves the working copy, and `restore` to that operation **SHALL** bring it back.
  - **WHEN** `undo` has run, **THEN** `Workspace::reported` **SHALL** hold each entry the snapshot left alone once.
  - **WHEN** the matrices run at this task's commit, **THEN**:
    - Materialise **SHALL** find 0 violations with every control caught, and assert its new literals, recorded beside the old ones;
    - `docs/reach.md` **SHALL** be generated again;
    - Snapshot and the two store matrices **SHALL** keep their literals: none of their scenarios runs `undo` or `restore`.
- **Notes:**
  - As built in Sprint 3, `undo` and `restore` snapshot, and then call `restore_view`, which snapshots again. The second scan reads every file once more.
  - One scan just before the write is planned keeps INT-0009 AC-2: the write still reads each path again before it removes or replaces it and leaves one that changed. [INT-0009](../../../intents/INT-0009-reversible-by-default.md) records the rule.
  - `restore_view` called directly still snapshots first, once.
  - Counting scans of the top directory would not show the change: the write's own planning reads directories with the same syscall.
  - It comes after T-004 so that T-004's claim, that no literal moves, is made on the literals T-003 left.

### T-006: The command registry: dispatch, help and suggestions from one table
- **Intent:** [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md), [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** crates/praps/src/main.rs, crates/praps/src/registry.rs (new), crates/praps/src/help.rs (new), crates/praps/src/commands.rs (new), crates/praps/tests/cli.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0022 AC-8 and AC-11, and AC-9 for the commands built so far; INT-0037 AC-1 (plain words), AC-2 and AC-5.
- **Success criterion (EARS):**
  - **WHEN** `praps` runs with no argument, **THEN** it **SHALL** print every registry command on stdout, grouped by purpose with the everyday group first and each with its one-line summary, and exit 0.
  - **WHEN** `praps help WORD` names a registry command, **THEN** it **SHALL** print that command's usage line, summary, description, arguments, at least one example, and whether the command is idempotent, and exit 0. **WHEN** WORD is not a command, **THEN** it **SHALL** answer as for an unknown word.
  - **WHEN** `--help` or `-h` is among the arguments, **THEN** `praps` **SHALL** answer as `help` for the command named, or with the list if none is. **WHEN** `--version` or `-V` is, **THEN** it **SHALL** answer as `version`, which prints `praps` and the package's version.
  - **WHEN** the first word is not a command, **THEN** `praps` **SHALL** exit 2 with nothing on stdout, and on stderr:
    - name every command within two edits of it, nearest first and then by name;
    - for a word in the table of other systems' words (`commit`, `add`, `checkout` and the rest), say what to use in its place, naming only registry commands;
    - end by pointing to `praps help`.
  - **WHEN** any other argument begins with `-`, or a command is given the wrong number of arguments, **THEN** `praps` **SHALL** exit 2, print that command's usage line as the registry gives it, and change nothing.
  - **WHEN** the registry is read, **THEN** every name **SHALL** match `[a-z]+`; the everyday group **SHALL** hold at most ten commands; no command marked idempotent **SHALL** declare a flag; and every command **SHALL** have a summary, a usage line and an example.
  - **WHEN** a command whose help says it is idempotent is run twice in a row, **THEN** the second run **SHALL** print what the first printed and **SHALL** leave every file as the first left it.
  - **WHEN** arguments are parsed, **THEN** a word **SHALL** be taken as a command exactly when the registry holds it: dispatch has no other table.
- **Notes:**
  - The seven commands keep their names and output in this task. `fsck` is renamed in T-007, where the refused-word check of INT-0037 AC-1 lands with it.
  - Parsing is a function from the arguments to what will be done, and touches no file. The registry's unit tests are run on it, so none of them runs a command.
  - The registry marks five commands idempotent: `status`, `log`, `fsck`, `help` and `version`. `ops` joins them in T-009. `init` and `describe` do not: [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) defines the class by the repeat, and theirs does not print the same.
  - `help` prints the mark, so a test that runs the binary can find the class without a list of its own.
  - The table of other systems' words names only commands that exist at each commit. Its answer for `fsck` arrives in T-007, and its answer for `commit` gains `describe` in T-010.
  - Edits are counted as single-character insertions, deletions and substitutions.
  - `cargo policy`'s `fs-boundary` rule refuses one word that help text could easily hold, in string literals too (`crates/xtask/src/policy.rs`, `escapes`). The texts avoid it, and `cargo policy` in `cargo ci` checks they do.
  - `crates/praps/tests/cli.rs` asserts today that `praps`, `--help` and `-h` are usage errors. Those assertions change with the behaviour, by this task's clauses.

### T-007: `verify` replaces `fsck`
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** crates/praps/src/registry.rs, crates/praps/src/commands.rs, crates/praps/tests/cli.rs, crates/praps/tests/e2e.rs, crates/workspace/src/lib.rs, crates/workspace/tests/workspace.rs, README.md, docs/spec/working-copy.md, docs/spec/storage-durability.md
- **Depends on:** T-006
- **Acceptance criterion:** INT-0037 AC-1 (no refused word) and AC-2.
- **Success criterion (EARS):**
  - **WHEN** `verify` runs, **THEN** it **SHALL** print and return what `fsck` did for the same store.
  - **WHEN** `fsck` is typed, **THEN** `praps` **SHALL** exit 2, write nothing to stdout and name `verify`.
  - **WHEN** the registry is read, **THEN** no name **SHALL** be a word INT-0037 refuses: `fsck`, `rebase`, `checkout`, `stash`, `reflog`, `cherry-pick`.
  - **WHEN** the `workspace` crate refuses a store whose op files are not all valid, **THEN** its message **SHALL** name `praps verify`, not `praps fsck`.
  - **WHEN** `README.md`, `docs/spec/working-copy.md` and `docs/spec/storage-durability.md` are read after this task, **THEN** every `praps WORD` in them **SHALL** name a command `praps` lists, and the word `fsck` **SHALL** appear only in `Store::fsck` or in a file's name.
- **Notes:** only the command is renamed. `Store::fsck` and `crates/store/src/fsck.rs` keep their names, and `crates/store/tests/` does not change. The documents that name the command change in the same commit, so that none of them names a word `praps` refuses.

### T-008: Commands find their working copy from any directory inside it
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/praps/src/place.rs (new), crates/praps/src/main.rs, crates/praps/src/commands.rs, crates/praps/tests/cli.rs, crates/praps/tests/e2e.rs
- **Depends on:** T-007
- **Acceptance criterion:** INT-0037 AC-3.
- **Success criterion (EARS):** in these clauses, "a command that opens a working copy" is `status`, `new`, `log`, `undo` or `restore`, and from T-009 and T-010 also `ops` and `describe`. `help` and `version` look at no directory and are not changed by this task. A directory is a **top** when listing its `.praps/store` succeeds.
  - **WHEN** a command that opens a working copy is given a directory that exists (or none, meaning the current one), **THEN** it **SHALL** act on the nearest top among that directory and those above it.
  - **WHEN** such a command runs at the top with no directory given, and the same command runs on an equal working copy from a directory below the top, or with a directory below the top given, **THEN** stdout, stderr, the exit code and the state left **SHALL** be equal.
  - **WHEN** a refusal names the working copy, **THEN** it **SHALL** name the directory as given if one was given and it is the top, and the top's full path otherwise.
  - **WHEN** such a command is given a directory that does not exist or is not a directory, **THEN** it **SHALL** fail naming it as given, exit 1, write nothing, and **SHALL NOT** act on a working copy above it.
  - **WHEN** no directory from the given one upward is a top, **THEN** such a command **SHALL** exit 1, say that no repository was found there or above, and write nothing.
  - **WHEN** listing a directory's `.praps/store` fails for a reason other than its absence, **THEN** such a command **SHALL** stop and report that failure, and **SHALL NOT** go on to a directory further up.
  - **WHEN** the current directory is needed and cannot be read, **THEN** such a command **SHALL** exit 1 and say so.
  - **WHEN** `init` is given a directory, which need not exist, **THEN**:
    - if a directory above it is a top, it **SHALL** write nothing, exit 1 and name that working copy, whatever the directory itself holds;
    - otherwise it **SHALL** do as today: make the repository; or complete one whose `init` was interrupted, whether that left an empty `.praps` or a store with no operation; or, for a whole repository, say so and write nothing.
  - **WHEN** `verify` is given a directory that does not exist or is not a directory, **THEN** it **SHALL** fail naming it and exit 1. Otherwise the store it checks **SHALL** be the first of:
    1. `DIR/.praps/store`, if DIR is a top, whatever state that store is in;
    2. DIR itself, if DIR holds `objects` and `ops`, as today, whatever lies above it;
    3. the store of the nearest top above DIR;
    4. DIR itself, which then fails as not a store, as today.
  - **WHEN** an entry named `.praps` below the top is a file, or a directory that holds no `store`, **THEN** it **SHALL** be tracked as before, and every command run beside or beneath it, `verify` among them, **SHALL** act on the working copy above.
- **Notes:**
  - On Windows `RealFs` takes `..` lexically and cannot climb above the current directory, and the simulator takes no absolute path. So the search starts from the current directory made absolute, and when the top is above the starting directory `praps` makes the top its current directory and runs as if started there. What is written and synced is then exactly what a run from the top writes and syncs.
  - The search is a function over `sim::Fs`, tested on the simulator with relative paths. The step to an absolute path and the change of directory are two calls in `main.rs`, covered through the binary.
  - Listing `DIR/.praps/store` does not list DIR itself, which may be large. `Fs` has no cheaper question to ask. Absence means "not found", or that a name on the way is not a directory. The second is a different error on Linux than on the simulator, so a test through the binary covers it with a file named `.praps`.
  - The paths above the starting directory are taken from its full path as written, one name at a time. A symbolic link in that path is not resolved first.
  - "Equal" in the second clause is tested on two byte-for-byte copies of one repository, with each copy's own top path replaced by one marker before stderr is compared.
  - `verify`'s first rule changes one case: a directory that holds `.praps` with no `store` in it. Today `verify` reports that `.praps/store` is not a store. From this task it goes on to rules 2 to 4, since such a directory is no top. A `.praps/store` that exists and is damaged is still reported as today.
  - The tests make their directories under the system's temporary directory. A test checks that no directory above them is a top, so a stray repository there fails one test by name instead of many by surprise.
  - A `.praps` directory below the top that holds a store is the top of a repository of its own, as [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) now records. `init` there is refused like any other inside a working copy. What the working copy around it should do is backlog T-031.
  - Not changed here, and not tested: opening a working copy syncs the directory that holds it, and on Windows a directory is opened for that with write access (`crates/sim/src/real.rs`, `open_dir_for_sync`). By the code, a working copy directly under a directory its user cannot write would open only from inside, where `..` stops at the current directory. Checking that, and deciding what should happen, is in backlog T-034.

### T-009: `log` lists changes and `ops` lists operations
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/tests/history.rs, crates/workspace/tests/workspace.rs, crates/workspace/tests/common/mod.rs, crates/praps/src/registry.rs, crates/praps/src/commands.rs, crates/praps/tests/cli.rs, crates/praps/tests/e2e.rs, docs/spec/working-copy.md, docs/work/tasks.md
- **Depends on:** T-003, T-008
- **Acceptance criterion:** INT-0037's vocabulary for `log` and `ops`, and AC-2; INT-0009 AC-1 and AC-3 kept; INT-0014 AC-1 and INT-0004 AC-7 kept for the new `log`; backlog T-018.
- **Success criterion (EARS):**
  - **WHEN** `log` runs, **THEN** it **SHALL** list the working-copy change and the changes before it along first parents, newest first, one line each: the first 12 hex digits of the change ID, the number of paths the change changes, and the first line of its description, escaped as names are, or `(no description)`.
  - **WHEN** `log` runs, **THEN** it **SHALL** first record the working copy, as every command that opens one does: an edit made before it **SHALL** be counted in the first line, and each entry the snapshot left alone **SHALL** be reported as `status` reports it.
  - **WHEN** `ops` runs, **THEN** it **SHALL** list every operation, children before parents and ties by id, one line each: the first 12 hex digits of its id, the change ID of its working-copy change, and the number of paths that change changes.
  - **WHEN** `log` or `ops` runs after `open`, **THEN** it **SHALL** read the op log as T-003's first clause says of every command.
  - **WHEN** `ops` or `log` lists its entries in a history where every change sits on top of the last, **THEN** the syscalls each makes **SHALL** grow by the same amount, within a tenth, from 50 to 100 entries as from 100 to 150. Run against the listing as it stood at T-008's commit, the same test **SHALL** fail, and that run is recorded as the negative control.
  - **WHEN** the workspace's history tests run, **THEN** each **SHALL** pass on `ops()` with its assertions unchanged, except those on the count: an entry's count is now the number of paths its working-copy change changes, where it was the number of files in the tree.
  - **WHEN** the task is done, **THEN** `docs/work/tasks.md` **SHALL** no longer hold T-018.
- **Notes:**
  - The count `ops` prints no longer needs a tree, which cost a walk through the whole chain of changes for every line.
  - This changes what `praps log` prints. [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) records that the new listing is provisional, that it does not meet AC-4, and when the seed and rubric of AC-5 and AC-6 are registered.
  - `log` and `ops` print every entry. An idempotent command takes no flag (INT-0022 AC-8), and how a long listing is limited is not decided in this sprint.

### T-010: `describe` sets the description of the current change
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/src/points.rs, crates/workspace/src/variant.rs, crates/workspace/tests/history.rs, crates/workspace/tests/workspace.rs, crates/workspace/tests/materialise.rs, crates/workspace/tests/describe_crash.rs (new), crates/praps/src/registry.rs, crates/praps/src/commands.rs, crates/praps/tests/cli.rs, crates/praps/tests/e2e.rs, crates/xtask/src/reach.rs, docs/reach.md, docs/spec/working-copy.md
- **Depends on:** T-009
- **Acceptance criterion:** INT-0037's vocabulary for `describe`, and AC-2; INT-0014 AC-1 and INT-0004 AC-7 kept; INT-0009 AC-1 and AC-3 kept; INT-0005 AC-1 to AC-3 and AC-6 for the new scenario.
- **Success criterion (EARS):**
  - **WHEN** `describe TEXT` runs, **THEN** the working copy **SHALL** be recorded first, with each entry the snapshot left alone reported as `status` reports it, and one operation **SHALL** then leave the working-copy change with the same change ID, parents and changes and with TEXT's bytes as its description. `praps` **SHALL** say so in one line that names the change and shows the first line of the text, escaped as names are.
  - **WHEN** TEXT equals the change's description, **THEN** `describe` **SHALL** write no operation of its own and **SHALL** say that nothing changed.
  - **WHEN** `describe` runs after `open`, **THEN** it **SHALL** read the op log as T-003's first clause says of every command.
  - **WHEN** a later snapshot records an edit, **THEN** the change **SHALL** keep its description. **WHEN** `new` runs, **THEN** the new change **SHALL** have none, and the change before it **SHALL** keep its own.
  - **WHEN** `undo` follows `describe`, **THEN** the working-copy change **SHALL** have the description it had before. **WHEN** `restore` is then given the describing operation, **THEN** the change **SHALL** have the new description again. No working-copy file **SHALL** change in either.
  - **WHEN** the system crashes at any syscall of a scenario of edits, snapshots, `describe` and `new`, under both directory modes and 40 seeds, **THEN** on the state a restart sees:
    - the store **SHALL** check clean, and every acknowledged operation **SHALL** be present;
    - the working-copy change's description **SHALL** be the last one acknowledged or the one in flight;
    - a snapshot taken on that state **SHALL** keep it.

    A workspace variant whose snapshot drops the description, and the store's `UnsyncedObjects` variant, **SHALL** each produce at least one violation. The matrix's cell count and digest **SHALL** be asserted as literals.
  - **WHEN** the reach table is generated, **THEN** a scenario named Describe, added in Sprint 6, **SHALL** reach the new point `workspace.describe.committed` of unit `workspace::describe`, and `docs/reach.md` **SHALL** show both.
  - **WHEN** a session uses only words `praps` itself printed (the list, then `help`, `init`, `status`, `describe`, `new`, `log`, `ops`, `undo`, `restore` and `verify`), **THEN** every command **SHALL** exit 0 and print what its step names, at the top and from a directory two levels down.
- **Notes:**
  - `describe` reuses `Workspace::replace_working_copy`.
  - The point, its registration, its expectation in `crates/xtask/src/reach.rs` and the scenario that reaches it land in this one commit: `cargo policy` and `cargo reach` each fail on any one without the others.
  - TEXT is one argument. More than one word needs quotes, and T-011 gives the hint.
  - This is the task after which every command of the sprint exists, so the two sessions of the last clause are written here.
  - The table of other systems' words now answers `commit` with `describe` and `new`.

### T-011: Every refusal names the next command
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** crates/praps/src/commands.rs, crates/praps/src/main.rs, crates/workspace/src/lib.rs, crates/praps/tests/cli.rs, crates/workspace/tests/workspace.rs
- **Depends on:** T-010
- **Acceptance criterion:** INT-0037 AC-4.
- **Success criterion (EARS):**
  - **WHEN** `praps` prints an error of any kind it defines, **THEN** it **SHALL** print what was wrong and then exactly one line starting `hint:` that gives one of: a command to run next; what to put right outside `praps`; or that this build has no remedy yet and that nothing was lost. The kinds are:
    - a usage error and an unknown word;
    - each variant of `workspace::Error`;
    - each way `verify` fails;
    - each way finding the working copy fails: a missing directory, no repository there or above, a `.praps/store` that cannot be listed, a current directory that cannot be read, and `init` inside a working copy.
  - **WHEN** a hint names a command, **THEN** that command **SHALL** be a registry name.
  - **WHEN** a message of the `workspace` crate is printed, **THEN** it **SHALL** say only what is wrong: each variant's text is fixed by a test, and what to do about it appears once, in the hint.
  - **WHEN** `describe` is given three or more words, **THEN** `praps` **SHALL** exit 2 and the hint **SHALL** show the text in quotes. **WHEN** it is given two words and the second is not a directory, **THEN** the refusal **SHALL** name the missing directory and the hint **SHALL** show both words as one quoted text.
  - **WHEN** the task is done, **THEN** the test evidence **SHALL** hold a table of every error kind with its message and its hint, read by the code review for whether a person could act on each.
- **Notes:**
  - Several op heads, a change with several parents and a failure of the filesystem have no next command in this build. Their hints say so (INT-0037 AC-4).
  - A check of form cannot tell a useful hint from a useless one of the right shape. The table and its review are the evidence for that part.
  - `praps describe update docs`, where `docs` is a directory inside the working copy, describes the change as `update`. The rule that a command takes what it acts on and then a directory makes that correct, and the line `describe` prints shows the text it took. The help for `describe` gives a quoted example.

### T-012: A write leaves a directory it cannot remove
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/workspace/src/materialise.rs, crates/workspace/tests/materialise.rs, crates/praps/tests/e2e.rs, docs/spec/working-copy.md
- **Depends on:** T-005, T-008
- **Acceptance criterion:** INT-0037 AC-3 (a command works from any directory inside); INT-0014 AC-1 and INT-0009 AC-3 kept; INT-0005 AC-1 unchanged.
- **Success criterion (EARS):**
  - **WHEN** a write has emptied a directory and removing it fails, for any reason but a crash of the simulator, and the target tree has no file at that path, **THEN** the write **SHALL** go on: every file of the target **SHALL** be written, the record **SHALL** be dropped, the directory **SHALL** be left, and a note **SHALL** name it.
  - **WHEN** the target tree has a file at that path, **THEN** the write **SHALL** stop with the failure, as today, with the record kept, and a later command **SHALL** finish it once the directory can be removed.
  - **WHEN** `undo` is run from a directory that its write empties, while another process has that directory as its current directory, **THEN** `praps` **SHALL** exit 0 and the working copy's files **SHALL** equal the target's. The directory **SHALL** either be gone, or be left and named in a note. The test evidence **SHALL** record which happened on Windows and which on Linux.
  - **WHEN** the five matrices run at this task's commit, **THEN** each **SHALL** keep its literals: in their scenarios no removal fails.
- **Notes:**
  - Today a failed removal stops the write after its operation is committed. Every later command then resumes the write and fails the same way, until nothing holds the directory.
  - Directories are not tracked, so a directory left empty changes nothing a snapshot records. [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) records the rule and its one exception.
  - A removal that fails because the simulator crashed still stops the write: a crash is not a refusal.
  - It comes after T-008 because its end-to-end test runs `undo` from inside the working copy.

### T-013: README, specs and credits
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** README.md, docs/spec/working-copy.md, docs/spec/storage-durability.md, docs/prior-art.md, crates/praps/tests/cli.rs, crates/xtask/src/docs.rs
- **Depends on:** T-011, T-012
- **Acceptance criterion:** INT-0003 AC-5 and AC-10; INT-0037 AC-2 (the README says what `praps` says).
- **Success criterion (EARS):**
  - **WHEN** the README's command table is read, **THEN** it **SHALL** hold one row for each command `praps` lists and no other, each with the summary `praps` prints, and the README's existing checks **SHALL** still pass.
  - **WHEN** `docs/spec/working-copy.md` is read, **THEN** it **SHALL** describe each of these: `describe`; `log`; `ops`; how a command finds its working copy; the refusal of `init` inside a working copy; one read of the op log for a command; the check of the op log before resume; the single scan of `undo` and `restore`; and a directory a write cannot remove. Its Snapshot section **SHALL** no longer name `new` and `status` as the only commands that snapshot.
  - **WHEN** `docs/spec/storage-durability.md` is read, **THEN** its list of what remains asserted **SHALL** no longer say that writers in separate processes and process crashes are untested, which Sprint 4 did.
  - **WHEN** `docs/prior-art.md` is read, **THEN** it **SHALL** credit the Command Line Interface Guidelines for help, suggestions and next-step hints, Mercurial for the name `verify`, and, among designs evaluated, Dolt's history-independent trees for the view of INT-0006.
- **Notes:** T-003, T-004, T-005, T-007, T-009, T-010 and T-012 each correct the sentences of the specs that they make false. This task adds what is new and checks the whole.
