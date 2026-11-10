# Sprint 6 Build Plan

The owner approved this plan's outline on 2026-11-10, with three choices: `verify` as the name of the integrity check; `log` for changes and `ops` for operations; and, on GitHub, one run on demand in T-001 and the sprint's pull request opened during Test.

## Intents
- [INT-0037](../../../intents/INT-0037-command-set.md) — state: planned; acceptance criteria covered:
  - AC-1: every command name is a plain word, and none is a refused word;
  - AC-2: `praps` lists its commands, explains each, and answers a mistyped word;
  - AC-3: a command works from any directory inside a working copy, and `init` does not nest;
  - AC-4: every refusal says what to do next;
  - AC-5: the everyday group holds at most ten commands.
- [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) — state: planned; acceptance criteria covered: AC-8 (no flag on an idempotent command) and AC-11 (one description of each command). AC-9 (an idempotent command repeats exactly) is checked for the commands built so far and stays open for the rest.
- [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) — state: planned; acceptance criteria covered: none in full. T-003 is a first step toward cost rule 2: a command reads the op log once.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — state: active; acceptance criteria covered: AC-1 kept (no lost edits) through T-003, T-005 and T-006; its Consequences on cost, on finding the working copy and on `describe`.
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — state: active; acceptance criteria covered: AC-1, AC-2 and AC-3 kept, with one scan before a write in place of two, `describe` added to the operations that `undo` and `restore` must reverse, and the operation list under its new name.
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered: AC-1 to AC-3 and AC-6. Two matrices are recorded again, one scenario and one named point are added, and two matrices must not move.
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: active; acceptance criteria covered: AC-2 (the `main` and on-demand halves, by record), AC-3, AC-4, AC-5, AC-10, AC-11 (by record).
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered: AC-1, on both platforms for every task.
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md) — through T-001 only: the state of the owner's sign-offs.

## Schema Tree
- Sprint Goal: write the owner's direction of 2026-11-09 into the Book, and take its first steps in code
  - Carried forward
    - T-001: Record Sprint 5's GitHub Actions runs, one run on demand, and the owner's sign-offs
  - Cost
    - T-002: Remove duplicated helpers, with every matrix digest unchanged
    - T-003: A command reads the op log once
  - Use
    - T-004: The command registry: dispatch, help and suggestions from one table
    - T-005: Commands find their working copy from any directory inside it
    - T-006: `describe`, `log` for changes, `ops` for operations, and `verify`
    - T-007: Every refusal names the next command
  - Documentation
    - T-008: README, specs, credits and backlog

## Execution Sequence

### T-001: Record Sprint 5's GitHub Actions runs, one run on demand, and the owner's sign-offs
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** docs/sprints/s6/sprint-tests/ci-sprint5.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0004-windows-and-linux.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:**
  - INT-0003 AC-2 (a push to `main` is never cancelled or replaced; CI runs on demand), AC-3, AC-4 and AC-11, for the commits Sprint 5 added after its tested head and for its merge;
  - INT-0004 AC-1 on GitHub's runners;
  - the owner's sign-offs still pending: INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6, INT-0004 AC-5.
- **Success criterion (EARS):**
  - **WHEN** the record is written, **THEN** `ci-sprint5.md` **SHALL** name, for run 42721345619 (the pull request at `b0675f6`) and run 42726187298 (the push to `main` at `55bdc07`), each of the following, taken from `gh run view`, `gh api …/jobs` and each job's own log:
    - event, head and conclusion;
    - every job, with its conclusion, attempt and start and end times;
    - the runner image, the `rustc` version, the test count of each `cargo ci` leg and its `ci:` line.
  - **WHEN** the run on `main` is recorded, **THEN** the record **SHALL** say whether any later push cancelled or replaced it, and **SHALL** set the Windows `cargo ci` time of both runs against AC-11's 20 minutes and its aim of 15.
  - **WHEN** the test counts are recorded, **THEN** the record **SHALL** compare them with Sprint 5's tested head (500 on Windows and 501 on Linux) plus the one test added after it.
  - **WHEN** `gh workflow run ci --ref dev` has been started and has finished, **THEN** the record **SHALL** name that run, its event `workflow_dispatch`, its head and every job's conclusion. **IF** the run cannot be started, or has not finished when Test ends, **THEN** the record **SHALL** say so and a backlog task **SHALL** carry it.
  - **WHEN** an owner sign-off has not been given in the conversation, **THEN** the record **SHALL** say it is pending, and a backlog task **SHALL** carry it forward. No sign-off **SHALL** be recorded without the owner's words. Approving this plan and merging Sprint 5's pull request count as neither.
- **Notes:** this is backlog T-024 and T-025. `gh run view --job … --log` is not a source for a re-run job (Sprint 5 test critique, C-008); neither run here was re-run.

### T-002: Remove duplicated helpers, with every matrix digest unchanged
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/sim/src/fs.rs, crates/sim/src/lib.rs, crates/hash/src/lib.rs, crates/store/src/lib.rs, crates/store/src/names.rs, crates/workspace/src/lib.rs, crates/workspace/src/materialise.rs, crates/praps/src/main.rs, crates/sim/tests/contract.rs, crates/hash/tests/vectors.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0005 AC-1 to AC-3 and AC-6 unchanged; INT-0003 AC-4 (Clippy clean).
- **Success criterion (EARS):**
  - **WHEN** the `Fs` forwarders for `&T`, `Rc<T>` and `Arc<T>` are generated by one macro, **THEN** each of the trait's 16 methods called through each of the three **SHALL** reach the underlying filesystem, and a named point marked through each **SHALL** be counted. `Fs::point` **SHALL** keep no default body.
  - **WHEN** `parent_dir`, `dir_exists`, the hex form of bytes, the join of a repository path onto a root, and the listing of a directory keyed by name each have one definition, **THEN** every test **SHALL** pass with the four asserted matrices exactly as Sprint 5 left them:
    - store: 32,840 cells, `d36b8c9c…`;
    - four writers: 44,836 cells, `253b1eab…`;
    - Snapshot: 25,360 cells, `c1e39cb1…`;
    - Materialise: 40,960 cells, `6274344b…`.

    `docs/reach.md` **SHALL** be unchanged too.
  - **WHEN** `hash::hex` is given bytes, **THEN** it **SHALL** return two lowercase hex digits for each byte, in order.
- **Notes:** these are findings C-26 to C-31 of the [code audit](../sprint-research/code-audit.md). The unchanged literals are the proof that no syscall moved, which is why this task comes before T-003.

### T-003: A command reads the op log once
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/workspace/src/lib.rs, crates/store/src/lib.rs, crates/workspace/tests/workspace.rs, crates/workspace/tests/history.rs, crates/workspace/tests/materialise.rs, crates/workspace/tests/crash.rs, crates/workspace/tests/materialise_crash.rs, crates/store/tests/store.rs, docs/reach.md, docs/spec/working-copy.md, docs/work/tasks.md
- **Depends on:** T-002
- **Acceptance criterion:** INT-0014 AC-1 kept; INT-0009 AC-2 and AC-3 kept; INT-0035 cost rule 2 (a first step; no criterion met); INT-0005 AC-1 to AC-3 and AC-6 for the two matrices recorded again.
- **Success criterion (EARS):**
  - **WHEN** `Workspace::open` runs and one command then runs on that workspace (`snapshot`, `status`, `new_change`, `log`, `undo`, `restore` or `restore_view`), **THEN** the simulator's trace from the start of `open` to the end of the command **SHALL** show:
    - `ops/` listed exactly once;
    - `objects/` never listed;
    - each op file that was there at the start read exactly once, and beyond those only the files an operation the command commits names: its parents and its own.
  - **WHEN** a further command runs on the same `Workspace`, **THEN** it **SHALL** list `ops/` exactly once more, at its start.
  - **WHEN** one command on a `Workspace` has finished and another handle then commits an operation or damages an op file, **THEN** the next command on that `Workspace` **SHALL** stop with `SeveralHeads` or `Corrupt`, as it does today.
  - **WHEN** `Workspace::working_copy` or `Workspace::parent_tree` is called, **THEN** it **SHALL** read the store, never the state a command kept, so the oracles that compare them with a model check what is stored.
  - **WHEN** `ops/` holds a file that does not hash to its name or does not decode, **THEN** `open` **SHALL** stop with `Corrupt` before it resumes an interrupted write, and the working-copy record **SHALL** stay in place.
  - **WHEN** `undo` or `restore` runs, **THEN** it **SHALL** scan the working copy once, before it plans the write, and an edit made before the command **SHALL** be recorded before any file is overwritten.
  - **WHEN** `Store::open_with_log` is given a directory without `objects/` or without `ops/`, **THEN** it **SHALL** fail as `Store::open` does, having written nothing.
  - **WHEN** the Snapshot and Materialise matrices run after the change, **THEN**:
    - each **SHALL** find 0 violations, and every negative control (`UnsyncedObjects`, `NoRecord` and the others each file runs) **SHALL** still be caught;
    - each new cell count and digest **SHALL** be asserted as a literal and recorded in the sprint's test evidence beside the old one;
    - `docs/reach.md` **SHALL** be generated again by `cargo reach`;
    - the store and four-writer matrices **SHALL** keep their literals, and `crates/store/tests/` **SHALL** change only by tests added for the two new store functions.
- **Notes:**
  - What a `praps` command does today, counted from the code: `Store::open` lists `objects/` and `ops/` to see that they exist, and each load lists `ops/` twice and reads every op file. `status` loads three times. So one `status` lists `ops/` seven times, reads every op file three times, and lists every object once.
  - `Store::ops`, `Store::open` and `Store::inspect` are not changed: the store's scenarios call them. Two functions are added:
    - `Store::op_log` gives, from one listing, the valid ops and the number of op files that are not valid;
    - `Store::open_with_log` opens a store and reads its log with that one listing. The listing shows that `ops/` exists, and the layout syncs, which `open` makes anyway, show that `objects/` does.
  - The state is passed through a command's steps. It is kept from `open` only until the first command takes it. No state outlives a command, so a failed command leaves nothing stale.
  - Between `open` and the first command, an operation written by another handle is not seen by that command. The two then leave two op heads, as any two writers that overlap do, and the next command stops. A `praps` process runs `open` and its command back to back.
  - After its own commit a command updates the state from what it wrote, building the tree with `tree::of` as a fresh read would.
  - `undo` and `restore` scanned the working copy twice, once themselves and once inside `restore_view`. One scan immediately before the write is planned keeps INT-0009 AC-2: the write itself still leaves any file that changed after the scan.
  - Every command still reads the whole chain of changes to build a tree. That cost follows the history, and it waits for the storage form (backlog T-030).
  - Backlog T-027 quotes Materialise's cell count; its text is updated with the new one.

### T-004: The command registry: dispatch, help and suggestions from one table
- **Intent:** [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md), [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** crates/praps/src/main.rs, crates/praps/src/registry.rs (new), crates/praps/src/help.rs (new), crates/praps/src/commands.rs (new), crates/praps/tests/cli.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0022 AC-8 and AC-11, and AC-9 for the commands built so far; INT-0037 AC-1 (plain words), AC-2 and AC-5.
- **Success criterion (EARS):**
  - **WHEN** `praps` runs with no argument, **THEN** it **SHALL** print every registry command on stdout, grouped by purpose with the everyday group first and each with its one-line summary, and exit 0.
  - **WHEN** `praps help WORD` names a registry command, **THEN** it **SHALL** print that command's usage line, summary, description, arguments and at least one example, and exit 0. **WHEN** WORD is not a command, **THEN** it **SHALL** answer as for an unknown word.
  - **WHEN** `--help` or `-h` is among the arguments, **THEN** `praps` **SHALL** answer as `help` for the command named, or with the list if none is. **WHEN** `--version` or `-V` is, **THEN** it **SHALL** answer as `version`, which prints `praps` and the package's version.
  - **WHEN** the first word is not a command, **THEN** `praps` **SHALL** exit 2 with nothing on stdout, and on stderr:
    - name every command within two edits of it, nearest first and then by name;
    - for a word in the table of other systems' words (`commit`, `add`, `checkout` and the rest), say what to use in its place, naming only registry commands;
    - end by pointing to `praps help`.
  - **WHEN** any other argument begins with `-`, or a command is given the wrong number of arguments, **THEN** `praps` **SHALL** exit 2, print that command's usage line as the registry gives it, and change nothing.
  - **WHEN** the registry is read, **THEN** every name **SHALL** match `[a-z]+`; the everyday group **SHALL** hold at most ten commands; no command marked as changing nothing when run again **SHALL** declare a flag; and every command **SHALL** have a summary, a usage line and an example.
  - **WHEN** a command marked as changing nothing when run again is run twice in a row, **THEN** the second run **SHALL** print what the first printed and **SHALL** leave every file as the first left it.
  - **WHEN** a word runs a command, **THEN** that word **SHALL** be a registry name: dispatch has no other table.
- **Notes:**
  - The seven commands keep their names and output in this task. `fsck` is renamed in T-006, where the refused-word check of INT-0037 AC-1 lands with it.
  - The registry marks six commands as changing nothing when run again: `status`, `log`, `fsck`, `help`, `version` and, from T-006, `ops`. `init` and `describe` are safe to repeat but print something else the second time, so by INT-0022 AC-9 they are not in that class.
  - The table of other systems' words names only commands that exist at each commit, so its answer for `commit` gains `describe` in T-006.
  - Edits are counted as single-character insertions, deletions and substitutions.
  - `cargo policy`'s `fs-boundary` rule refuses one word that help text could easily hold, in string literals too (`crates/xtask/src/policy.rs`, `escapes`). The texts avoid it, and `cargo policy` in `cargo ci` checks they do.
  - `crates/praps/tests/cli.rs` asserts today that `praps`, `--help` and `-h` are usage errors. Those assertions change with the behaviour, by this task's clauses.

### T-005: Commands find their working copy from any directory inside it
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/praps/src/place.rs (new), crates/praps/src/main.rs, crates/praps/src/commands.rs, crates/praps/tests/cli.rs, crates/praps/tests/e2e.rs
- **Depends on:** T-004
- **Acceptance criterion:** INT-0037 AC-3.
- **Success criterion (EARS):**
  - **WHEN** a command other than `init` is given a directory that exists (or none, meaning the current one) and that directory or one above it holds `.praps`, **THEN** the command **SHALL** act on the nearest such working copy. Its stdout, its stderr, its exit code and the state it leaves **SHALL** equal those of the same command run at the top with no directory given.
  - **WHEN** a refusal names the working copy, **THEN** it **SHALL** name it by the directory given, if one was given and it is the top, and by the top's full path otherwise.
  - **WHEN** the directory given does not exist, **THEN** the command **SHALL** fail naming it, exit 1, and **SHALL NOT** act on a working copy above it.
  - **WHEN** no directory from the given one upward holds `.praps`, **THEN** the command **SHALL** fail as it does today, exit 1.
  - **WHEN** `init` is given a directory that lies inside a working copy and is not its top, **THEN** it **SHALL** write nothing, exit 1 and name that working copy. **WHEN** the directory is the top of a repository, **THEN** it **SHALL** answer as today.
  - **WHEN** the integrity check is given a directory that itself holds `objects` and `ops`, **THEN** it **SHALL** check that directory as a bare store, as today, whatever lies above it.
- **Notes:**
  - On Windows `RealFs` takes `..` lexically and cannot climb above the current directory, and the simulator takes no absolute path. So the search starts from the current directory made absolute, and when the top is above the starting directory `praps` makes the top its current directory and runs as if started there. What is written and synced is then exactly what a run from the top writes and syncs.
  - The search is a function over `sim::Fs`, tested on the simulator with relative paths. The step to an absolute path and the change of directory are two calls in `main.rs`, covered through the binary.
  - A directory holds `.praps` when listing `DIR/.praps` succeeds. `Fs` has no cheaper question to ask, and this one does not list the directory itself, which may be large. Any failure but "not found" stops the search and is reported: a `.praps` that cannot be read is never passed over for a repository further up.
  - The paths above the starting directory are taken from its full path as written, one name at a time. A symbolic link in that path is not resolved first.
  - The tests make their directories under the system's temporary directory. A test checks that no directory above them holds `.praps`, so a stray repository there fails one test by name instead of many by surprise.
  - Not changed here, and not tested: opening a working copy syncs the directory that holds it, and on Windows a directory is opened for that with write access (`crates/sim/src/real.rs`, `open_dir_for_sync`). By the code, a working copy directly under a directory its user cannot write would open only from inside, where `..` stops at the current directory. Checking that, and deciding what should happen, goes to the backlog with the harness findings (T-034).

### T-006: `describe`, `log` for changes, `ops` for operations, and `verify`
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/src/points.rs, crates/workspace/src/variant.rs, crates/workspace/tests/history.rs, crates/workspace/tests/common/mod.rs, crates/workspace/tests/describe_crash.rs (new), crates/praps/src/registry.rs, crates/praps/src/commands.rs, crates/praps/tests/cli.rs, crates/praps/tests/e2e.rs, crates/xtask/src/reach.rs, docs/reach.md, docs/work/tasks.md
- **Depends on:** T-003, T-005
- **Acceptance criterion:** INT-0037 AC-1 and AC-2; INT-0014 AC-1 kept; INT-0009 AC-1 and AC-3 kept; INT-0005 AC-1 to AC-3 and AC-6 for the new scenario; backlog T-018.
- **Success criterion (EARS):**
  - **WHEN** `describe TEXT` runs, **THEN** the working copy **SHALL** be recorded first, and one operation **SHALL** then leave the working-copy change with the same change ID, parents and changes and with TEXT's bytes as its description. `praps` **SHALL** say so in one line that names the change.
  - **WHEN** TEXT equals the change's description, **THEN** `describe` **SHALL** write no operation of its own and **SHALL** say that nothing changed.
  - **WHEN** a later snapshot records an edit, **THEN** the change **SHALL** keep its description. **WHEN** `new` runs, **THEN** the new change **SHALL** have none, and the change before it **SHALL** keep its own.
  - **WHEN** `undo` follows `describe`, **THEN** the working-copy change **SHALL** have the description it had before. **WHEN** `restore` is then given the describing operation, **THEN** the change **SHALL** have the new description again. No working-copy file **SHALL** change in either.
  - **WHEN** the system crashes at any syscall of a scenario of edits, snapshots, `describe` and `new`, under both directory modes and 40 seeds, **THEN** on the state a restart sees:
    - the store **SHALL** check clean, and every acknowledged operation **SHALL** be present;
    - the working-copy change's description **SHALL** be the last one acknowledged or the one in flight;
    - a snapshot taken on that state **SHALL** keep it.

    A workspace variant whose snapshot drops the description, and the store's `UnsyncedObjects` variant, **SHALL** each produce at least one violation.
  - **WHEN** `log` runs, **THEN** it **SHALL** list the working-copy change and the changes before it along first parents, newest first, one line each: the first 12 hex digits of the change ID, the number of paths the change changes, and the first line of its description, escaped as names are, or `(no description)`.
  - **WHEN** `ops` runs, **THEN** it **SHALL** list every operation, children before parents and ties by id, one line each: the first 12 hex digits of its id, the change ID of its working-copy change, and the number of paths that change changes.
  - **WHEN** `ops` or `log` lists its entries, **THEN** the workspace **SHALL** read a fixed number of objects for each and derive no tree for it: the syscalls each command makes **SHALL** grow by the same amount, within a tenth, from 50 to 100 entries as from 100 to 150, in a history where every change sits on top of the last.
  - **WHEN** `verify` runs, **THEN** it **SHALL** print and return what `fsck` did. **WHEN** `fsck` is typed, **THEN** `praps` **SHALL** exit 2 and name `verify`. No registry name **SHALL** be a word INT-0037 refuses.
  - **WHEN** the reach table is generated, **THEN** a scenario named Describe, added in Sprint 6, **SHALL** reach the new point `workspace.describe.committed` of unit `workspace::describe`, and `docs/reach.md` **SHALL** show both.
- **Notes:**
  - `describe` reuses `Workspace::replace_working_copy`. `Store::fsck` and `crates/store/src/fsck.rs` keep their names: only the command is renamed.
  - The number `ops` prints is no longer the size of the tree (which cost a walk through the whole history for every line) but the number of paths the working-copy change changes.
  - This changes what `praps log` prints, so INT-0022 AC-4 to AC-7 now apply to the new listing. None is claimed: lines are not fitted to the terminal, and no panel has judged the layout.
  - `log` and `ops` print every entry. A command that changes nothing when run again takes no flag (INT-0022 AC-8), and how a long listing is limited is not decided in this sprint.
  - The point, its registration, its expectation in `crates/xtask/src/reach.rs` and the scenario that reaches it land in this one commit: `cargo policy` and `cargo reach` each fail on any one without the others.
  - TEXT is one argument. More than one word needs quotes, and T-007 gives the hint.

### T-007: Every refusal names the next command
- **Intent:** [INT-0037](../../../intents/INT-0037-command-set.md)
- **Touches:** crates/praps/src/commands.rs, crates/praps/src/main.rs, crates/workspace/src/lib.rs, crates/praps/tests/cli.rs, crates/workspace/tests/workspace.rs
- **Depends on:** T-006
- **Acceptance criterion:** INT-0037 AC-4.
- **Success criterion (EARS):**
  - **WHEN** `praps` prints an error of any kind it defines, **THEN** it **SHALL** print what was wrong and then one line starting `hint:` that gives one of: a command to run next; what to put right outside `praps`; or that this build has no remedy yet and that nothing was lost. The kinds are:
    - a usage error and an unknown word;
    - each variant of `workspace::Error`;
    - each way `verify` fails;
    - each way finding the working copy fails, the nested `init` among them.
  - **WHEN** a hint names a command, **THEN** that command **SHALL** be a registry name.
  - **WHEN** a message of the `workspace` crate carried advice of its own ("run `praps init`", "run `praps fsck`"), **THEN** that advice **SHALL** appear once, in the hint, and the message **SHALL** say only what is wrong.
  - **WHEN** `describe` is given several words and the last is not a directory, **THEN** the hint **SHALL** say to put the text in quotes.
- **Notes:** several op heads, a change with several parents and a failure of the filesystem have no next command in this build. Their hints say so (INT-0037 AC-4, as revised in this Plan).

### T-008: README, specs, credits and backlog
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0037](../../../intents/INT-0037-command-set.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** README.md, docs/spec/working-copy.md, docs/spec/storage-durability.md, docs/prior-art.md, docs/work/tasks.md, crates/praps/tests/cli.rs, crates/xtask/src/docs.rs
- **Depends on:** T-007
- **Acceptance criterion:** INT-0003 AC-5 and AC-10; INT-0037 AC-2 (the README says what `praps` says).
- **Success criterion (EARS):**
  - **WHEN** the README's command table is read, **THEN** it **SHALL** hold one row for each command `praps` lists and no other, each with the summary `praps` prints, and the README's existing checks **SHALL** still pass.
  - **WHEN** `docs/spec/working-copy.md` is read, **THEN** it **SHALL** describe `describe`, `log`, `ops`, how a command finds its working copy, the refusal of a nested `init`, and that a command reads the op log once.
  - **WHEN** `docs/spec/storage-durability.md` is read, **THEN** it **SHALL** name the command `verify`, and its list of what remains asserted **SHALL** no longer say that writers in separate processes and process crashes are untested, which Sprint 4 did.
  - **WHEN** `docs/prior-art.md` is read, **THEN** it **SHALL** credit the Command Line Interface Guidelines for help, suggestions and next-step hints, Mercurial for the name `verify`, and, among designs evaluated, Dolt's history-independent trees for the view of INT-0006.
  - **WHEN** `docs/work/tasks.md` is read, **THEN** it **SHALL** hold one backlog task for each of these, with its intents: the canonical merge with ownership and scoped undo (T-029); packed storage, the object index, streamed I/O and the speed suite (T-030); the stat cache, scanning on every core and many workspaces (T-031); the simulator at scale (T-032); the tooling findings (T-033); the harness findings (T-034). T-018, T-024 and T-025 **SHALL** be gone, done by T-006 and T-001.
