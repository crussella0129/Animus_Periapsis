# Sprint 4 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-5: four writing processes, no corrupt record and no lost op, under crash injection | T-005, T-006 / the four-writer matrix: 44,836 cells (40 seeds × 2 modes × every crash step × {no kill, one kill}), 0 violations of any kind, 0 `fsck` problems, 6,552 cells with a killed process; odd seeds force the parent race and kill process 0 at its held rename, while process 2 names its unsynced op (`writers_kills_happen`); `writers_retry_reaches_present` (196 cells crashed during a retry, the op present in each); five negative controls caught; `real_four_processes_lose_nothing` on NTFS and ext4 (100 acknowledged ops from four OS processes, `fsck` clean) | pass on both systems; simulated and reproducible, plus one unreproducible real run per system | Test evidence links this report |
| INT-0007 | AC-1 to AC-3 under four writers | the same matrix: closure, durability of every acknowledged op, and `store.heads()` equal to the heads of the present logged ops. Atomicity is not computed by the four-writer oracle (C-004, below); closure over each process's chain covers the prefix property | pass | as above |
| INT-0007 | AC-1 to AC-4, single writer | store matrix (32,840 cells, digest `blake3:d36b8c9c…ae812`), Snapshot and Materialise matrices, `fsck` corpus: Sprint 3's figures after 21 points were planted | pass | as above |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1: determinism | `writers_matrix_is_reproducible` (`blake3:253b1eab…0855c` over 44,836 cells, equal on Windows and Linux); the three earlier digests unchanged by the `Mutex` move and the planted points; `sim::Processes` runs are a function of the seed (`processes_runs_are_deterministic`, `processes_seeds_vary_interleavings`) | pass | Test evidence links this report |
| INT-0005 | AC-2: crash model and matrices | the four-writer matrix with process kills; points are neither syscalls nor traced (`points_are_not_syscalls`), so crash numbering is unchanged | pass | as above |
| INT-0005 | AC-3: negative controls | the store's five controls caught again under four writers; `UnsyncedReferences` caught only through the forced race (closure 12) | pass | as above |
| INT-0005 | AC-4: randomness only through `sim` | `cargo policy` (unchanged rules) and the `Processes` scheduler drawing only from `sim::Rng` | pass | as above |
| INT-0005 | AC-6: reach | T-007 / `docs/reach.md` generated on both systems, byte-identical; all 21 registered points reached; each scenario reaches every point `reach::EXPECTED` lists for it (C-011); the `points` policy rule (registered once, used, one sprint per unit); `reach_check_fails_on_a_stale_table` and the other `reach_fails_*` tests | pass | as above |
| INT-0005 | AC-5 (network faults) | with INT-0024 | not yet tested | stays open |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: suite on both systems, OS and toolchain named | `cargo ci` on Windows 11 Pro 10.0.26300 (477 tests) and in a fresh clone on Ubuntu 26.04.1, WSL2 (478), rustc 1.99.1 | pass locally; GitHub half at this checkpoint (T-020) | Test evidence links this report |
| INT-0004 | AC-7: names with no UTF-8 form, reported | T-008, T-009 / `snapshotting_commands_return_reports` (each command on a freshly opened workspace, two names in `status`'s order), `failed_commands_carry_reports` (four failures), `a_failed_snapshot_reports_nothing_stale` (after the tested head), `praps_commands_report_skipped_names` on NTFS and ext4 | pass. Backlog T-019 closes: `undo`, `restore` and `new` now report what `status` reports | as above |
| INT-0004 | AC-5, AC-6 | sign-off pending; Nighthawk not reachable | unchanged | unchanged |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1: 0 lost edits | Snapshot and Materialise matrices re-run with points planted: Sprint 3's counts and digests; the Materialise points reached in every expected scenario | pass on both systems | Test evidence links this report |
| INT-0014 | AC-3 (several workspaces) | after INT-0010's merge | not yet tested | stays open |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-1, AC-8 to AC-10: the canonical runner and its policy | `cargo ci` gains a `reach` step (six steps); `cargo policy` gains the `points` rule with fixtures; `cargo deny check` on both systems, now including WSL (closes Sprint 3's C-003) | pass | Test evidence links this report |
| INT-0003 | AC-2 to AC-4 | local halves on both systems; the outside-target probe on WSL | local halves pass; GitHub halves at this checkpoint (T-020) | as above |
| INT-0003 | AC-5 credits | `docs/prior-art.md`: fail-rs, RocksDB sync points, FoundationDB BUGGIFY, shuttle and PCT, and the owner's proposal | credited; sign-off pending | as above |
| INT-0003, [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md) | Sprint 3's GitHub runs; AC sign-offs | T-001 / [ci-sprint3.md](ci-sprint3.md): runs 42443548089, 42443543930, 42536778981 and 42537492546, every job success; sign-offs not given | recorded; sign-offs carried forward (T-020) | the Test evidence of INT-0003, INT-0004 and INT-0006 links `ci-sprint3.md` (T-001); INT-0001 and INT-0002 hold sign-offs only |

## Summary
- **Unit tests:** 477 passed on Windows and 478 on Linux at the tested head, with 0 failed and 0 ignored. 70 are new. The extra Linux test needs a real ext4 symlink. Sprint 3 ended with 407 and 408. One more test, `a_failed_snapshot_reports_nothing_stale`, came after the tested head (round-2 C-001); its binary passed on both systems.
- **Integration tests:** all pass on both systems:
  - the canonical runner, now six steps with `reach`;
  - the four-writer matrix and its five controls;
  - the store, Snapshot and Materialise matrices, with Sprint 3's digests;
  - the reach table, identical on both systems;
  - `cargo deny` on both systems;
  - two probes (an unregistered point; a target directory outside the checkout).
- **E2E tests:**
  - `real_four_processes_lose_nothing` on NTFS and ext4;
  - `praps_commands_report_skipped_names` on NTFS and ext4, with Sprint 3's CLI tests unchanged;
  - `fresh_clone_ci` on WSL ext4.

  Not yet possible: power loss, Nighthawk, several workspaces, and a process killed on a real filesystem.
- **CI status:** Sprint 3's GitHub runs are recorded, all success. This sprint's runs start when `dev` is pushed at the checkpoint; backlog T-020 records them.

## CI Confirmation
- **Head SHA:**
  - The full leg scripts ran at `9fa59fb8d0f94170860cff5bbee685625c612233` on both systems ([test-matrix-log.md](test-matrix-log.md)).
  - They ran first at `5dfb418` and were re-run in full after the round-1 test critique fixes. Every matrix count and all four digests are the same in both rounds.
  - One commit after `9fa59fb` changes a crate: `6665913` adds the test `a_failed_snapshot_reports_nothing_stale` to `workspace/tests/history.rs` and changes nothing else. That binary passed 15 of 15 at `6665913` on Windows and in a fresh WSL clone; the full suite was not re-run for it.
  - Every other commit after `9fa59fb` changes only the Book: results, critiques, this report, intent evidence and the backlog.
- **CI run:** none yet for this head. The checkpoint push triggers it.
- **Conclusion:** the local canonical runner succeeded on both systems.
- **Confirmations:**
  - `cargo ci` printed `PASS` for fmt, clippy, test, reach, policy and book, then `ci: all 6 steps passed`, with exit 0. It did so in the Windows checkout (2,060 s) and in a fresh WSL ext4 clone (1,342 s).
  - `cargo deny check` reported advisories, bans, licenses and sources ok on both systems.
- **How the legs ran:** at the same time, on one 24-thread host. Every duration is longer than either leg alone; no budget was exceeded (the `writers` binary took 133 s on Windows against a 10-minute allowance).

## Failures
None at the tested head. Three reviews raised 22 findings, all fixed or answered:
- the Test-phase [code review](code-review.md): 7 findings, all fixed, each with a regression test where one applies;
- the test critique [round 1](critique-round1.md): 12 concerns, verdict `block`. Ten were fixed in `9fa59fb`, one deferred (C-004) and one rejected (C-012);
- the test critique [round 2](critique.md): 3 concerns, verdict `proceed-with-caveats`. It confirmed every round-1 response by reading and by two mutants, and found CR-07's fix untested (now tested, in `6665913`), two overstated sentences in the integration results (corrected), and no recorded check of the documentation clauses (below).

The most material:
- **CR-01:** the evidence that writers overlap was read from actions that run after the scheduler's draw, so it could be true without an overlap. It is now read from the trace.
- **C-001 to C-003:** the scheduler's panic, release and kill paths were tested on one seed each, without a deadlock guard. They now run over 10 or 20 seeds under a watchdog, and require the interleaving they need in at least one seed. Two mutations confirmed they catch real faults: moving the kill after the ordering wait, and removing the unwind on abort.
- **C-007:** no test showed a killed process's unsynced op being named by a later writer. `writers_kills_happen` now asserts it in every odd seed.
- **C-009:** the retry test read its outcome and its log from two different runs. They now come from one, with the recovered ops.
- **C-011:** a scenario could stop reaching a point without the table failing, as long as some other scenario still reached it. `cargo reach` now checks each scenario against the build plan's expected points.

### C-004, deferred
The critique asked for torn records to count separately from semantic violations, and for the four-writer oracle to compute atomicity. Neither is done this sprint:
- `torn` has counted as a semantic violation since Sprint 1, and the store matrix's `WriteInPlace` control is caught by torn records alone in every sprint. Changing that would change what four sprints of results mean.
- The four-writer `check` does not compute atomicity, so its printed 0 is not a finding. Each process's ops form a chain, each naming its predecessor, so the closure check covers the prefix property that atomicity states for one writer.
- The summary line prints an unchecked kind as 0. It shares the `Violations` type with the Sprint 1 to 3 matrices, so it is left alone, to keep their printed output unchanged.

### C-012, rejected: a departure from T-010's wording
T-010 asked that the README and the `xtask` description name the four project commands. `cargo book` is an alias of the `book` package, not of `xtask`. So the `xtask` description and its README row name the three commands it serves, while the README's four-command sentence and the `NO_FLAGS` check cover all four. This is recorded as a departure from the clause's wording, not from its purpose.

## The documentation clauses (T-001, T-008's spec, T-010)
These EARS clauses ask for records and prose, not behaviour. Each was checked during the Test phase, on 2026-11-09:

| Clause | Check | Result |
|---|---|---|
| T-001: `ci-sprint3.md` names each run of `4f05dd1` and `7b2fd59` with head, event and conclusion, every job with its conclusion and runner, the toolchain, test counts and `cargo deny` | re-read all four runs with `gh run view <run> --json headSha,event,conclusion,jobs` | runs 42443548089 (pull request) and 42443543930 (push) on `4f05dd1`, 42536778981 and 42537492546 (push) on `7b2fd59`: every run and all twelve jobs `success`, as recorded |
| T-001: unsigned sign-offs pending, T-017 replaced | read `ci-sprint3.md` "Owner sign-offs" and the backlog | each pending sign-off says so; T-017 is gone, and T-020 carries them forward |
| T-008: `docs/spec/simulation.md` states six topics | read against the list | what a point is and is not ("A point is not a syscall", under "Named points"); the registry and its rule; the scheduler's draws, orderings and kills ("Several processes"); the process-crash model; points as scheduling boundaries, with the four-writer digest's expected change and the response to it; the reach table. All six present |
| T-010: credits | read `docs/prior-art.md` | fail-rs, RocksDB sync points, FoundationDB BUGGIFY, shuttle with PCT (Burckhardt et al., ASPLOS 2010), and the owner's proposal of coded breakpoints and the run × unit triangle |
| T-010: Consequences | read INT-0005 and INT-0007 | INT-0005 now describes the process-crash model built, not "not simulated yet"; INT-0007 records what four writers showed: no lock needed, the parent re-sync load-bearing, several heads derived, merging left to INT-0010 |
| T-010: README and `readme_problems` | executed tests `readme_keeps_statement_and_links_book` and `readme_must_document_each_command` (`crates/xtask/src/docs.rs`), extended this sprint for the four-command sentence and a line on what `cargo reach` does; they pass in every `cargo ci` run above. README line 68 reads "`cargo ci`, `cargo policy`, `cargo book` and `cargo reach` take no flags." | pass |
| T-010: the `xtask` description and README row name all four commands | read `crates/xtask/Cargo.toml` and the README crate table | they name three: `cargo ci`, `cargo policy` and `cargo reach`. This is the C-012 departure above; the four-command sentence and the `NO_FLAGS` check cover `cargo book` |
| T-010: backlog | read `docs/work/tasks.md` and `completed-tasks.md` | T-011 and T-019 have left the backlog; T-010's completion entry records them as absorbed into T-007 and T-009 (T-009's own entry names T-019; T-007's does not name T-011). T-021 (test selection from reach) and T-022 (the remaining trace-text hooks) are new. T-017 is replaced by T-020 |

## Technical Debt Identified
- **INT-0005 AC-6, test selection:** the reach table records which scenarios reach which units, but nothing yet uses it to pick the tests a change needs (backlog T-021).
- **Trace-text hooks:** the Sprint 1 two-writer race moved to a named point, but some tests still find their moment by matching trace text (backlog T-022).
- **C-004:** the four-writer oracle does not compute atomicity, and its summary line prints the unchecked kind as 0.
- **The process-kill model:** a kill runs only in the simulator. A process killed on a real filesystem is not tested; INT-0007 relies on `fsync` and the operating system's page cache there.
- **`real_four_processes_lose_nothing`:** the kernel picks the interleaving, so no run reproduces, and the children never name each other's ops.
- **Carried forward:**
  - INT-0014 AC-3: several workspaces, after INT-0010;
  - INT-0006: the provisional object bodies (T-012);
  - INT-0004 AC-3: case-only renames (T-016) and probed name rules (T-015);
  - INT-0013, INT-0009: `log` is quadratic in history length (T-018);
  - the owner's sign-offs and this checkpoint's GitHub runs (T-020).
