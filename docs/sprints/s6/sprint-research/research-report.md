# Sprint 6 Research Report

## Intents Reviewed
All 34 chapters were read against the owner's direction of 2026-11-09, and four were created. The per-chapter findings are in the [Book audit](book-audit.md).

Created, as `proposed`:
- [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) — **created**; relevance: owns the scale model and the rule that cost follows the change.
- [INT-0036](../../../intents/INT-0036-measured-speed.md) — **created**; relevance: owns the speed comparison, its workloads and its thresholds.
- [INT-0037](../../../intents/INT-0037-command-set.md) — **created**; relevance: owns the command names and what `praps` says about itself. This sprint builds its first commands.
- [INT-0038](../../../intents/INT-0038-live-view.md) — **created**; relevance: owns the screen for a person directing many agents.

Selected, to be revised in Plan:
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md) — **selected, to be revised**; current state: proposed. The charter names the three parts of the direction and maps the new chapters.
- [INT-0006](../../../intents/INT-0006-format-v1.md) — **selected, to be revised**; current state: active. A view's size must follow the change.
- [INT-0008](../../../intents/INT-0008-compact-history-storage.md) — **selected, to be revised**; current state: proposed. Takes on the storage form: packed objects, one flush for a command.
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — **selected, to be revised**; current state: active. Undo scoped to the actor; local and shared operations.
- [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) — **selected, to be revised**; current state: proposed. A merge is computed, never committed; criteria at scale.
- [INT-0013](../../../intents/INT-0013-derived-data.md) — **selected, to be revised**; current state: proposed. Derived data kept up to date in proportion to what is new, and never trusted.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — **selected, to be revised**; current state: active. A thousand workspaces; no unchanged file read. This sprint changes its code.
- [INT-0021](../../../intents/INT-0021-lanes-and-review-requests.md) — **selected, to be revised**; current state: proposed. More than one verifier; simulation at the new scale.
- [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) — **selected, to be revised**; current state: proposed. This sprint builds its registry.
- [INT-0024](../../../intents/INT-0024-staged-sync.md) — **selected, to be revised**; current state: proposed. A replica follows a part; admission by actor.
- [INT-0030](../../../intents/INT-0030-optional-scale-mode.md) — **selected, to be revised**; current state: proposed. Only servers stay optional.
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — **selected, to be revised** (small); current states: active. INT-0003 and INT-0004 also take the Sprint 5 record (backlog T-024, T-025).
- [INT-0011](../../../intents/INT-0011-change-identity-and-rewriting.md), [INT-0012](../../../intents/INT-0012-conflicts-and-merge.md), [INT-0015](../../../intents/INT-0015-phases-and-confirmation.md), [INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md), [INT-0017](../../../intents/INT-0017-event-records-and-book-view.md), [INT-0023](../../../intents/INT-0023-query-language.md), [INT-0032](../../../intents/INT-0032-intent-revisions-and-review-flags.md) — **selected, to be revised** (small); current states: proposed.

Reviewed, no change needed:
- [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — **reviewed**; current state: active. Its grammar already carries scale and speed criteria.
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — **reviewed**; current state: realized. Its criteria hold; it gets one dated note for the renamed command.
- [INT-0018](../../../intents/INT-0018-proof-pairing-certificates.md), [INT-0019](../../../intents/INT-0019-cheat-resistant-verifier.md), [INT-0020](../../../intents/INT-0020-intent-calculus.md), [INT-0025](../../../intents/INT-0025-git-bridge.md), [INT-0026](../../../intents/INT-0026-sprint-loops-on-periapsis.md), [INT-0027](../../../intents/INT-0027-port-sprint-loops-projects.md), [INT-0029](../../../intents/INT-0029-reproducible-releases.md), [INT-0031](../../../intents/INT-0031-visual-identity.md), [INT-0033](../../../intents/INT-0033-intent-topics-and-dependencies.md), [INT-0034](../../../intents/INT-0034-any-content.md) — **reviewed**; current states: proposed. They hold under the direction.
- [INT-0028](../../../intents/INT-0028-entropy-layer.md) — **reviewed**; current state: abandoned. Nothing reopens it.

## 1. Sprint Goal
Write the owner's direction of 2026-11-09 into the Book, and take the first steps toward it in code.

The direction, in the owner's words: Periapsis "should quite literally support thousands of agents, working in tandem, potentially in a decentralized completely asynchronous manner and it should be far faster and more optimized than any version control that currently exists, with a far more intuitive command set and UI".

This sprint does three things:
- **The Book.** Four new chapters (scale, speed, the command set, the live view). Every chapter the audit finds lacking is revised, and statements that have gone stale are corrected.
- **Use.** `praps` gets a command registry, help that it prints itself, suggestions for mistyped words, and commands that work from any directory of a working copy. The vocabulary gains `describe`, `log` for changes, `ops` for operations, and `verify` in place of `fsck`.
- **Cost.** A command reads the op log once, not three to six times, and listing history costs the same for each entry however long the history is.

This sprint does not build the canonical merge, the packed storage form or the stat cache. Each is a sprint of its own. Section 5 gives their order.

## 2. Existing Code Survey
The whole codebase was read. The findings, with line numbers, are in the [code audit](code-audit.md).

| File | Relevance | Notes |
|------|-----------|-------|
| `crates/store/src/lib.rs` | high | The write sequence (two flushes per object), `ops` and heads by reading every op file, seven directory syncs at every open |
| `crates/store/src/fsck.rs` | medium | The check behind `praps fsck`; sound; the command is renamed, the function is not |
| `crates/store/src/names.rs` | low | File names of objects; one more copy of hex formatting |
| `crates/workspace/src/lib.rs` | high | `load` three to six times per command; `tree_of` replays history; `log` quadratic; `undo` and `restore` global; several heads are an error |
| `crates/workspace/src/materialise.rs` | high | Plan and apply for writing the working copy; sound; duplicated listing code |
| `crates/workspace/src/tree.rs` | high | A tree is a flat map cloned for every changeset |
| `crates/workspace/src/record.rs` | medium | The working-copy record; sound |
| `crates/workspace/src/hazard.rs` | low | Name hazards; sound |
| `crates/workspace/src/ignore.rs` | low | Ignore rules; sound |
| `crates/workspace/src/points.rs` | medium | Point registry; a new durable unit needs an entry |
| `crates/codec/src/view.rs` | high | The view lists every head and workspace |
| `crates/codec/src/operation.rs` | high | Parents and a view; no actor, no scope |
| `crates/codec/src/changeset.rs` | high | Changes against the first parent; names no tree |
| `crates/codec/src/wire.rs` | medium | The canonical encoder and decoder; sound |
| `crates/codec/src/envelope.rs` | medium | Envelope and object id; sound |
| `crates/hash/src/lib.rs` | medium | BLAKE3 and SHA-256; one thread |
| `crates/sim/src/fs.rs` | high | The `Fs` trait: whole-file reads and writes; three copies of the forwarder |
| `crates/sim/src/real.rs` | medium | The real filesystem; verbatim paths on Windows |
| `crates/sim/src/simfs.rs` | high | The simulator; traces reads, so every syscall is a crash cell |
| `crates/sim/src/processes.rs` | high | One thread per simulated process |
| `crates/praps/src/main.rs` | high | Commands matched by hand; no help; no registry |
| `crates/praps/tests/cli.rs` | high | Pins today's behaviour, including `--help` as a usage error |
| `crates/praps/tests/e2e.rs` | medium | End-to-end through the binary |
| `crates/workspace/tests/crash.rs` | high | The Snapshot matrix; cell count and digest asserted as literals |
| `crates/workspace/tests/materialise_crash.rs` | high | The Materialise matrix; the same |
| `crates/workspace/tests/history.rs` | medium | `log`, `undo`, `restore` through the library |
| `crates/store/tests/common/matrix.rs` | medium | The store matrix; its heads oracle reads the store's own function |
| `crates/store/tests/common/writers.rs` | medium | The four-writer matrix; drops each process's result |
| `crates/xtask/src/policy.rs` | medium | 13 rules; lists of boundary crates with no membership check |
| `crates/xtask/src/ci.rs` | medium | The suite runner; the `book` step does not check which rule applied |
| `crates/xtask/src/reach.rs` | medium | Scenario and point names listed by hand |
| `crates/xtask/src/units.rs` | low | The parallel test runner; sound |
| `crates/book/src/lib.rs` | medium | The typed-acceptance validator; a declared Book with no chapter passes |
| `docs/spec/working-copy.md` | high | Specifies `log`, `undo`, `restore`; changes with the vocabulary |
| `docs/spec/storage-durability.md` | medium | Two statements stale since Sprint 4 |
| `README.md` | high | The command table; `xtask` checks parts of it |

## Measurements
A throwaway probe timed the release build of `b0675f6` against Git on one machine. Each figure is one run unless it says median. They show where the time goes; they are not the suite of [INT-0036](../../../intents/INT-0036-measured-speed.md).

- **Windows:** Windows 11, NTFS, 24 hardware threads, virus scanning on, Git 2.56.0.
- **Linux:** WSL2 Ubuntu on the same machine, ext4, Git 2.53.0.
- **"Git, flushing"** is Git with `core.fsync=all`, which keeps what it acknowledged through a crash, as Periapsis does. **"Git, defaults"** does not flush loose objects.
- Jujutsu is not installed on this machine and was not measured.

**Recording files.** Working copies of 2 KiB text files, 100 to a directory.

| | `praps` | Git, flushing | Git, defaults |
|---|---:|---:|---:|
| Linux, 1,000 new files | 10.7 s | 6.1 s | 0.29 s |
| Linux, 10,000 new files | 146.5 s | 66.9 s | 1.9 s |
| Windows, 1,000 new files | 13.0 s | 3.3 s | 5.4 s |
| Windows, 10,000 new files | 140.4 s | 24.2 s | 35.5 s |

- `praps` costs about 14 ms for each file on both systems. It flushes twice for each file; Git flushing does so once, at about 6.6 ms on Linux.
- Git at its defaults is 77 times faster than `praps` on Linux, because it does not flush at all.
- On Windows, Git's two columns came out in the opposite order to Linux's. Each ran once, so that is probably noise.

**Commands after the first snapshot.** Medians of 7 (`status`) and 5 (one edit).

| 10,000 files | `praps` | Git, flushing | Git, defaults |
|---|---:|---:|---:|
| Linux: `status`, nothing changed | 148 ms | 10 ms | 11 ms |
| Linux: record one edit | 287 ms | 38 ms | 16 ms |
| Linux: record one edit while the change holds all 10,000 files | 344 ms | — | — |
| Windows: `status`, nothing changed | 1,188 ms | 78 ms | 85 ms |
| Windows: record one edit | 1,612 ms | 186 ms | 127 ms |
| Windows: record one edit while the change holds all 10,000 files | 2,890 ms | — | — |
| Starting the program, Linux | 1.1 ms | 1.5 ms | |
| Starting the program, Windows | 23 ms | 55 ms | |

- `praps` reads and hashes every file for every command. Git reads only file metadata.
- An edit costs more while the working-copy change is large, because the store reads every blob a changeset names again before it accepts the changeset.

**Cost against history.** A working copy of 100 files; each step adds two operations (Linux).

| Operations in the log | `status`, nothing changed | `log` |
|---:|---:|---:|
| 10 | 7.5 ms | 3.6 ms |
| 50 | 9.8 ms | 13 ms |
| 100 | 11.4 ms | 37 ms |
| 200 | 15.3 ms | 131 ms |
| 400 | 23.9 ms | 481 ms |
| 800 | 41.7 ms | 1,813 ms |

- `status` grows in a straight line with the log, though nothing changed.
- `log` grows with its square: four times the time for twice the operations.
- On Windows the same steps took 65 ms and 186 ms at 50 operations, and 412 ms and 18.5 s at 800.

**Commands at the same time.**

| Started together in one repository | Result |
|---|---|
| 2 × `praps new`, three runs | In two runs both exit 0 and the log has two heads. Every later command then fails: "the op log has 2 heads, which cannot be merged yet", while `praps fsck` says `clean`. In the third run the two did not overlap |
| 8 × `praps new`, Linux | All exit 0, leaving eight heads |
| 8 × `praps new`, Windows, two runs | Three or four exit 0 and the rest exit 1, leaving two or three heads |
| Git, 32 worktrees each committing 20 times, Linux | 0 of 640 commits failed; 1,272 commits a second |
| Git, 32 worktrees, Windows | 0 of 640 failed; 6 commits a second, on a machine that was busy |

- Nothing was lost in any `praps` run: the store checks clean. But no command exists to join two heads, so once two commands overlap the repository cannot be used again.
- Git did not fail with 32 writers on 32 branches. Its cost for many agents lies elsewhere: a checkout for each agent, and a queue at any branch they share. Neither was measured.

**What the numbers say.**
1. Flushes decide recording time. One flush for a command would bring `praps` to about Git's default speed with every guarantee kept (Linux: 147 s, against 1.9 s for no flushes).
2. Reading every file decides `status`. Git, which reads only metadata, is about fifteen times faster at 10,000 files.
3. Reading the whole log decides everything else as history grows.
4. Two agents cannot use one repository at all until op heads are merged.

## 3. External Sources
- [Jujutsu: concurrency](https://github.com/jj-vcs/jj/blob/main/docs/technical/concurrency.md) — the model Periapsis follows. Operations and views are content-addressed and written without locks, and several op heads are joined by a three-way merge of their views against the common ancestor. It warns that use from several machines can lose pointers, though never commits, and names no number of writers it was built for. It does not say what happens when two operations move one workspace's pointer.
- [Git: `core.fsync` and `core.fsyncMethod`](https://git-scm.com/docs/git-config) — Git's default does not flush loose objects, and its own documentation says recent work can be lost if the system stops uncleanly. Its `batch` method stages many writes and then makes one full flush, which is the one-flush idea.
- [DoltHub: history independence](https://www.dolthub.com/blog/2024-11-26-history-independence/) — a map whose stored form depends only on its contents, not on the order of edits. Two versions then share storage, a comparison skips every subtree whose hash is equal, and diff and merge cost in proportion to the difference. This is what a view needs.
- [Command Line Interface Guidelines](https://clig.dev/) — practice collected from many tools: print short help when run with no arguments, accept `-h` and `--help`, suggest the command a mistyped word probably meant, rewrite errors for the person reading them, and say which command to run next.
- [Augment Code: Git worktrees for parallel agents](https://www.augmentcode.com/guides/git-worktrees-parallel-ai-agent-execution) — current practice gives each agent a worktree, and the guide presents that as enough for three to five agents at once. The parts all worktrees share are the object store, the refs and the configuration.
- [Developers Digest: Oak](https://www.developersdigest.tech/blog/oak-version-control-agents-git-alternative) — a 2026 version control system for agents, built on a server that holds the source of truth, with virtual mounts and a branch for each task. The article notes that its speed claim comes with no benchmark anyone can repeat. It is the centralised answer to the same problem.

## 4. Risks, Unknowns, Dependencies
- **Risk: a wrong sentence in an intent misleads every later sprint.** This sprint revises about twenty chapters.
  - The plan critic reads the revised chapters, not a summary of them.
  - Each new chapter has a criterion for the owner's sign-off, so nothing in it is settled by this sprint alone.
- **Risk: the matrices' literals change.** Reading the op log once removes syscalls from the working-copy code. The Snapshot and Materialise matrices have one crash cell for each syscall, so their cell counts, digests and the reach table all change.
  - The evidence that behaviour is unchanged is the oracles and the negative controls, which must still pass and still catch every control.
  - The store's own matrices must not change at all: their literals are the check that the store was not touched.
- **Risk: renaming commands breaks what reads their output.** No release exists, so only this repository's tests and documents depend on the names.
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) is `realized` and names `praps fsck`. It gets a dated note, not a rewrite.
- **Risk: the timings come from one noisy machine.** They are a baseline, not evidence for any criterion. INT-0036 requires medians, spread and exact counts.
- **Unknown: the owner's decisions.** The scale model's sizes, "8 of 8 workloads" as the bar for speed, the vocabulary, who owns which view entries, and how colliding claims are settled each wait on a sign-off criterion.
- **Unknown: whether a changeset should commit to a tree.** A partial clone cannot check a tree by replaying history. The question is recorded in INT-0006 for the format sprint.
- **Unknown: Git on one shared branch with many writers, and Jujutsu at all.** Neither was measured.
- **Unknown: how much `forbid(unsafe)` costs in speed.** No memory-mapped files, no assembly hashing. INT-0036's suite is to measure it.
- **Dependency: no new crate.** Suggestions need an edit distance, written in the `praps` crate.
- **Dependency: `gh` for the Sprint 5 record.** Backlog T-025 starts one CI run on `dev` through `workflow_dispatch`, which is an action on GitHub.
- **Dependency: Nighthawk** is still not reachable, so [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-6 does not move.

## 5. Recommended Approach
Primary:
1. **Record Sprint 5's runs** (backlog T-024, T-025): the checkpoint head's run, the merge's run on `main`, one run on demand, and the state of the owner's sign-offs.
2. **Revise the Book in Plan**, before the critic reads it: the changes the [Book audit](book-audit.md) lists for each chapter, and its table of stale statements.
3. **Build the registry and the vocabulary** ([INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md), [INT-0037](../../../intents/INT-0037-command-set.md)):
   - one table of commands, from which dispatch, help and usage are generated;
   - `praps` alone, `help`, `version`, and `--help` and `-h` as spellings of `help`;
   - a mistyped word answered with the nearest command, and a word from another system (`commit`, `add`, `checkout`) answered with what to use here;
   - the working copy found from any directory inside it;
   - `describe`, `log` for changes, `ops` for operations, `verify`;
   - every refusal naming the next command.
4. **Make cost stop following history where the storage form allows** ([INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0013](../../../intents/INT-0013-derived-data.md)):
   - a command reads the op log once and keeps what it derives while it runs;
   - `ops` and `log` cost the same for each entry (backlog T-018);
   - the matrices' literals and the reach table are recorded again, with the oracles and controls as the proof;
   - the probe is run before and after.
5. **Remove the duplicates** the code audit lists (C-26 to C-31), and close two holes in the checks this sprint leans on (X-02, X-06).
6. **Record what comes next**, as backlog tasks tied to their intents, in this order:
   1. The canonical merge of op heads, with ownership of view entries, scoped undo and a view with shared structure (INT-0010, INT-0016, INT-0009, INT-0006). Until it exists, two agents cannot share a repository.
   2. The packed storage form: one flush for a command, an object index as derived data, streamed reads and writes, and the speed suite to measure it (INT-0008, INT-0013, INT-0036).
   3. The stat cache, scanning on every core, and many workspaces (INT-0014).
   4. The simulator at scale: counts of cost, processes as resumable steps, a clock and a network (INT-0005).

Alternative considered: **start the canonical merge in this sprint.** It needs the view's new format, ownership rules and an algebra checked over thousands of histories ([INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) AC-1 to AC-6). Half a merge is worse than none, and it should not share a sprint with a revision of twenty chapters.

Alternative considered: **keep op heads and trees in files as derived data now.** Both would make `status` flat against history at once. Both would also be replaced, by the merge (heads) and by the packed form with tree manifests (trees). The in-memory step is kept by every later design.

Alternative considered: **revise the chapters in Build, as tasks.** Intents are the authority the plans are checked against, so they are revised before the plans lock, as Sprint 5 did.

Rationale: the direction changes what the Book asks for more than it changes any one module, so the Book comes first. The code steps chosen are the ones every later design keeps: a registry, a vocabulary, and a command that reads its state once.

## Artifacts
- [`book-audit.md`](book-audit.md) — every chapter's verdict, findings and change; the seven findings that cut across chapters; stale statements.
- [`code-audit.md`](code-audit.md) — findings by file and line: cost, several writers, use, duplication, tooling, the simulator.
- The timing probe was a throwaway script outside the repository, and changed nothing in it. Its numbers are recorded above. The release binary of `b0675f6` is kept outside the repository so the Test phase can time before and after on the same machine.

## Budget Override
The code survey lists 36 files and the sources number six, above the limits of 20 and 5. The owner asked for a sprint that analyses "the entire intent book AND codebase", so the whole of both was read: all 38 chapters, every source file of the six core crates, and, through two read-only reviewers, the tooling crates, the simulator and the test harnesses. A survey cut to 20 files would leave out findings the owner asked for. The sixth source is the one centralised system built for the same problem, which the chapters' Alternatives cite.
