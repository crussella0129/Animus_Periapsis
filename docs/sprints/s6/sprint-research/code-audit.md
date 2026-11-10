# Sprint 6 code audit

The whole codebase at `b0675f6`, read against the owner's direction of
2026-11-09 (scale, speed, use; see the [Book audit](book-audit.md)). The core
crates were read in full. The tooling crates, the simulator and the test
harnesses were audited by two read-only reviewers whose findings are merged
here.

This file is research provenance. Line numbers are those of `b0675f6`.

**What is sound and stays.** The careful part of the code is the part that is
hard to get right, and none of it is in question:
- the canonical encoding and its total decoders (`codec`, `hash`);
- the write sequence and its closure argument (`store`), proved by the crash
  matrices and their negative controls;
- the materialise plan and resume (`workspace`);
- the simulator's fault model (`sim`).

Every finding below is about cost, reach or use, not about correctness of
what is stored.

## 1. Cost follows the history, not the change

| # | Where | What happens | Cost |
|---|---|---|---|
| C-01 | `store/src/lib.rs:441` `Store::ops` | Reads, hashes and decodes every op file | O(ops) per call |
| C-02 | `workspace/src/lib.rs:747` `load` | Calls `Store::ops`, then `refuse_invalid_ops` (`:731`) lists `ops/` again | One full scan per call |
| C-03 | `workspace/src/lib.rs` | `load` or `Store::ops` runs 3 times for `status` and `new`, 3 for `log`, 6 for `undo` (`:498`, `:813`, `:876`, `:891`, `:928`, `:975`, `:977`, `:997`) | Each command reads the whole log 3 to 6 times |
| C-04 | `workspace/src/lib.rs:708` `tree_of` | Rebuilds a tree by replaying every changeset back to the root, cloning the tree at each step | O(depth × files) per call |
| C-05 | `workspace/src/lib.rs:964` `log` | Calls `tree_of` for every op to count its files (backlog T-018) | O(ops × depth × files) |
| C-06 | `store/src/lib.rs:203` `finish_open`, `workspace/src/lib.rs:418` `prepare` | Every command that opens a repository syncs seven directories before it reads anything | 7 flushes per command, changed or not |

Measured: [research report](research-report.md), Measurements.

## 2. Cost follows the size of the whole change, and of the whole working copy

| # | Where | What happens | Cost |
|---|---|---|---|
| C-07 | `store/src/lib.rs:258`, `:332` `put_object`, `valid_object` | Putting a changeset reads and hashes every blob it names, to check each is present | Every snapshot re-reads every file of the working-copy change, however small the edit |
| C-08 | `store/src/lib.rs:370` `write_file` | One temporary file, two flushes and a rename for each object | 2 flushes per file recorded |
| C-09 | `workspace/src/lib.rs:266` `Scan::dir` | Reads and hashes every tracked file on every command; no stat cache ([INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-5) | O(bytes in the working copy) per command |
| C-10 | `workspace/src/lib.rs:820` `snapshot` | Holds the content of every new file in memory until the scan ends | Memory O(new bytes) |
| C-11 | `sim/src/fs.rs:24`, `:34` `Fs::write`, `Fs::read` | Whole files only; no ranged or streamed I/O | A file larger than memory cannot be stored ([INT-0008](../../../intents/INT-0008-compact-history-storage.md) AC-6, [INT-0034](../../../intents/INT-0034-any-content.md)) |
| C-12 | `workspace/src/lib.rs:266`, `hash` | Scanning and hashing run on one thread | One core used of 24 |

## 3. Cost follows the number of agents

| # | Where | What happens | Cost |
|---|---|---|---|
| C-13 | `codec/src/view.rs:16` `View` | Holds every head and every workspace; each op writes a whole new view | O(workspaces) bytes per op |
| C-14 | `store/src/lib.rs:39` `OBJECTS`, `OPS` | Two flat directories, one file per object | Directory size O(objects); slow listing on NTFS |
| C-15 | `workspace/src/lib.rs:50` `DEFAULT_WORKSPACE` | One workspace, named in code | No second agent can have its own working-copy change |

## 4. Several writers

| # | Where | What happens |
|---|---|---|
| C-16 | `workspace/src/lib.rs:751` `load` | Several op heads are an error. No command joins them, so two commands that overlap leave the repository unusable ([INT-0010](../../../intents/INT-0010-lock-free-concurrency.md)). Measured. |
| C-17 | `workspace/src/lib.rs:628` `restore_view` | `undo` and `restore` commit an op whose view is a whole earlier view, so they would undo every other writer's work too ([INT-0009](../../../intents/INT-0009-reversible-by-default.md)) |
| C-18 | `workspace/src/lib.rs:974` `undo` | "The last op" is the single head, whoever made it |
| C-19 | `codec/src/operation.rs:16` `Operation` | No actor and no signature yet (backlog T-012), so nothing says who may change which view entry ([INT-0016](../../../intents/INT-0016-actors-and-agent-lifecycle.md)) |

## 5. Use

| # | Where | What happens |
|---|---|---|
| C-20 | `praps/src/main.rs:46` `cli` | Commands are matched by hand. There is no registry, though [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) builds both faces from one |
| C-21 | `praps/src/main.rs:51` | `praps`, `praps --help` and `praps -h` print one usage line and exit 2. No command explains itself |
| C-22 | `workspace/src/lib.rs:485` `open` | A command run in a subdirectory of a working copy says the directory is not a repository |
| C-23 | `praps/src/main.rs:90` `log` | Lists operations. A change cannot be described, and no command lists changes |
| C-24 | `praps/src/main.rs:261` `fsck` | Named after a Unix tool |
| C-25 | `workspace/src/lib.rs:83` `Error` | Messages say what is wrong; most do not say what to do next |

## 6. Duplication

| # | Copies |
|---|---|
| C-26 | `parent_dir`: `store/src/lib.rs:477`, `workspace/src/lib.rs:401` |
| C-27 | `dir_exists`: `store/src/lib.rs:236`, `workspace/src/lib.rs:408` |
| C-28 | Hex formatting of a digest: `praps/src/main.rs:85` and `:219`, `workspace/src/lib.rs:1018`, `store/src/names.rs:18`, `hash/src/lib.rs:209` |
| C-29 | Repository path to disk path: `workspace/src/lib.rs:271`, `workspace/src/materialise.rs:66` |
| C-30 | Directory listing keyed by name: `workspace/src/materialise.rs:116` and `:396` |
| C-31 | The 16-method `Fs` forwarder, three times: `sim/src/fs.rs:76`, `:129`, `:182` |

## 7. Tooling (`xtask`, `book`)

From the tooling review. "Verified" findings were read in the code.

| # | Where | Finding |
|---|---|---|
| X-01 | `xtask/src/policy.rs:51`, `:48`, `:59`; `xtask/src/points.rs:14` | The crates behind the filesystem boundary are named in lists, with no check that each name is a workspace member. A renamed or new I/O crate would silently escape the `fs-boundary`, `deps` and `points` rules |
| X-02 | `book/src/lib.rs:250`, `xtask/src/ci.rs:258` | A declared Book with no chapter passes, and `cargo ci`'s `book` step does not check which rule was applied. A mistyped marker key would select the lenient port rule unseen |
| X-03 | `xtask/src/attributes.rs:36`, `xtask/src/policy.rs:997` | The `golden` rule rejects bad rows from Git, but passes if Git prints fewer rows than asked, or none |
| X-04 | `xtask/src/policy.rs:99` | The filesystem ban lists no way to create a symbolic link, and nothing bans `std::process::Command` in boundary crates |
| X-05 | `xtask/src/policy.rs:858` | `escapes` can miss some `#![path = …]` attributes in a rule meant to fail closed |
| X-06 | `xtask/src/ci.rs:248` | The `policy` and `book` steps print only a count, so a failure seen only in CI needs a second run to diagnose |
| X-07 | `xtask/src/policy.rs:1516` | The policy tests run all 13 rules for every assertion: about 330 runs, each starting two `git` processes, so about 660 process starts per `cargo ci` |
| X-08 | `xtask/src/policy.rs` | One file of 3,561 lines (1,296 of code, 2,265 of tests) holds seven kinds of rule. Every rule parses the manifests again: 81 parses of 9 files per `cargo ci` |
| X-09 | `xtask/src/reach.rs:28`, `:42`; `policy.rs:27`; `ci.rs:38` | Counts that must be raised by hand: 5 scenarios, 21 point names, 13 rules, 6 steps, each with literal copies in tests |
| X-10 | `book/src/lib.rs:777` | `status`, `Status` and `Outcome` have no caller outside tests; `xtask/src/reach.rs:93` parses a scenario name it never checks |
| X-11 | several | Test scaffolding copied between files: `repo_file` (3), write-with-parents (6), remove-with-backoff (3), temporary directory naming (14) |

The review found no code that runs only in tests compiled into either
binary, and no panic reachable from a command.

## 8. The simulator and the crash matrices

From the simulator review. No source of nondeterminism was found: ordered
maps only, one runner at a time, and no clock in any result.

**What a change to the code moves.** Each matrix asserts a cell count and a
digest as literals:

| Matrix | Asserted at | Cells | Digest starts |
|---|---|---:|---|
| Store with two writers | `store/tests/crash.rs:94` | 32,840 | `d36b8c9c` |
| Four writers | `store/tests/writers.rs:102` | 44,836 | `253b1eab` |
| Snapshot | `workspace/tests/crash.rs:404` | 25,360 | `c1e39cb1` |
| Materialise | `workspace/tests/materialise_crash.rs:573` | 40,960 | `6274344b` |

- A digest covers each cell's recovered files and its trace text, and the
  trace includes reads (`sim/src/simfs.rs:762`).
- A matrix has one cell for every syscall of its scenario, read or write
  (`store/tests/common/matrix.rs:636`).
- So adding or removing even a read in `store` or `workspace` changes the
  cell count and the digest of every scenario that makes the call. A change
  to in-memory work alone changes neither.
- A named point changes only the four-writer digest, where it is a
  scheduling step (`sim/src/processes.rs:218`).
- 28% of the store matrix's cells crash at a read. A crash at a read leaves
  the same files as a crash at the write before it, so removing repeated
  reads from the code also shortens the matrices.

**What stands in the way of a thousand simulated agents.**

| # | Where | Finding |
|---|---|---|
| S-01 | `sim/src/processes.rs:360` | Each simulated process is an operating-system thread, started again for every run |
| S-02 | `sim/src/processes.rs:157`, `:125` | Every syscall rescans all processes and wakes every sleeping thread: O(processes) per syscall |
| S-03 | `sim/src/simfs.rs:642`, `:874`, `:934` | `recover` deep-copies the filesystem two or three times per cell; `read`, `sync_file` and `sync_dir` clone what they touch |
| S-04 | `store/tests/common/matrix.rs:636` | Each crash cell replays its scenario from the start: O(syscalls²) per seed. About 940,000 cells run in one suite |
| S-05 | `sim/src/lib.rs:19` | No clock and no network. Nothing bans `std::time` or `std::net` in the core crates, as `std::fs` is banned |

Only the mechanism of `Processes` needs threads. Its scheduler is plain data
(`sim/src/processes.rs:108`), so the same seeded draws could drive processes
written as resumable steps.

**Oracles that can pass when they should not.**

| # | Where | Finding |
|---|---|---|
| S-06 | `store/tests/common/writers.rs:283` | The four-writer harness drops each process's result. A writer whose commit fails without a crash acknowledges less, and the cell passes |
| S-07 | `store/tests/common/matrix.rs:552` | The heads check compares `Store::heads` with `store::heads_of`, the store's own function, so the oracle reads the code it checks |
| S-08 | `workspace/tests/materialise_crash.rs:547` | The matrix's second pass, 118,306 cells, is asserted only to be more than 0 |

**Duplication.** The cell loop and the digest fold appear four times, the
digest function three times with small differences, and the store's `check`
twice (`store/tests/common/matrix.rs:405`, `:517`, `:636`;
`store/tests/common/writers.rs:475`, `:575`; `workspace/tests/crash.rs:271`,
`:301`; `workspace/tests/materialise_crash.rs:370`, `:444`).

The three `Fs` forwarders (C-31) can be one macro. The guarantee that no
forwarder drops a point comes from `Fs::point` having no default body
(`sim/src/fs.rs:62`), so a macro that left it out would not compile.
