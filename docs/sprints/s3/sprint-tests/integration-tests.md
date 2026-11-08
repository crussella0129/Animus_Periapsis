# Sprint 3 Integration Test Results

These are the results at tested head `70a2a4d`, after the [code review](code-review.md) fixes. Every run is in [test-matrix-log.md](test-matrix-log.md), under the section named here. The two legs ran at the same time, which made both slower.

The Windows section `int_snapshot_matrix_windows` was overwritten in the log by a stray writer: a leg stopped earlier still held the log file open. That one test was run again on its own at the same head, and its output is logged as `int_snapshot_matrix_windows (rerun)`.

### Canonical runner on the Windows host and in WSL

These are the local halves only. The GitHub matrix runs at this sprint's checkpoint, and the next sprint records it, as T-001 did for Sprint 2.
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-1 to AC-4, AC-8
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-4

| Run | System | Result |
|---|---|---|
| `int_ci_windows` | Windows 11 Pro 10.0.26300, NTFS working checkout, rustc 1.99.1 | `cargo ci`: PASS for fmt, clippy, test, policy and book; exit 0 in 1,790 s; 403 tests passed |
| `e2e fresh_clone_ci` | Ubuntu 26.04.1 (WSL2), fresh ext4 clone, rustc 1.99.1 | cold `cargo ci`: every step PASS, exit 0 in 993 s; 404 tests passed; 0 tracked files changed |
| `int_deny_windows` | Windows, cargo-deny 0.20.2 | advisories, bans, licenses and sources all ok. No dependency was added. |
| `cargo policy` (inside `cargo ci`) | both | every rule PASS, including the new `RandomState` requirement in `fs-boundary`, the transitive-randomness check in `deps`, and `workspace/variants` as a test-only feature |

### Materialise crash matrix
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1 to AC-3
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1, AC-2
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1
  - [INT-0009](../../../intents/INT-0009-reversible-by-default.md) AC-3

Sections `int_materialise_matrix_windows` and `int_materialise_matrix_linux`. Every count and the digest are identical on both systems.

The scenario is: init, edits, snapshot, new, edits, `undo`, `restore`. It crashes at every syscall, for 40 seeds, in both directory modes. An ignored file sits beside the working copy throughout. After every crash inside `undo` or `restore`, a file the write changes is edited before the recovering `open`. The oracle checks:
- closure and durability of the store;
- the working copy: it equals the target view, the from view, or the last acknowledged state, with the user's edit kept;
- liveness: after resume and a snapshot, the change equals the files, the ignored file and the edit are byte for byte intact, and no temporary name is left.

| Variant | Cells per mode | Result |
|---|---|---|
| Correct | 20,480 | 0 violations of closure, durability, working copy or liveness, in both modes. 7,998 cells resumed a record. |
| Correct, second crash (`resume_survives_a_second_crash`) | 118,306 in all | a crash at every syscall of each recovering `open`, for 10 seeds per mode: 0 violations |
| `NoRecord` (negative control) | 19,840 | caught: 2,800 working-copy violations in Lenient mode and 2,820 in Strict mode |

**Determinism:** every cell reproduces, with the same state, trace and outcome. The digest over all 40,960 correct cells is `blake3:6274344b358b15cb24d182a681c592ccad66a82564fd40c5cf45e4fcb666a1ac` on both systems.

### Store and Snapshot crash matrices, re-run
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1, AC-3
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1 to AC-3

Sections `int_crash_matrix_*`, `int_fsck_corpus_*` and `int_snapshot_matrix_*`. T-003 added two syscalls and new `SimFs` state, so these must still give Sprint 2's figures.
- **Store matrix:** the same counts as Sprint 2 for the correct store and all five controls, on both systems. The digest over 32,840 cells is still `blake3:d36b8c9c77744d485a362ef3d94cebe40dcb67ca7260eb409f5430c2a13ae812`.
- **fsck corpus:** 40 of 40 bit flips and 207 of 207 truncations detected; 0 reports on 30,408 recovered stores.
- **Snapshot matrix:** 0 violations for the correct workspace in both modes, the `UnsyncedObjects` control caught with Sprint 2's counts, and the digest over 25,360 cells still `blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3`.

### Reversibility corpora
- **Intents:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md) AC-1 to AC-3

Sections `int_reversibility_and_hazards_*`, `history.rs`, on both systems:
- `two_hundred_pairs_stay_distinguishable`: 200 pre-state pairs that differ only in uncommitted content, through `undo` and `restore`: 800 runs, 0 collapsed (AC-2).
- `undo_after_every_op_type`: `undo` after snapshot, new, undo and restore gives the exact earlier view and working copy, including file↔directory swaps and changed executable files. On the root op it fails (AC-3).
- `restore_reaches_every_earlier_state`: every earlier op of a mixed history is restored exactly, view and files (AC-3).
- `ops_are_never_removed`: random command sequences; every op seen stays in `log` (AC-1).

### Hazard corpora
- **Intents:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-3, AC-4

Sections `int_reversibility_and_hazards_*`, `hazards.rs` and `real_hazards.rs`:
- `case_fold_corpus` and `windows_name_corpus`: every hazard refused, 100%, and nothing written, on the simulator with the Windows rules set.
- `case_sensitive_filesystems_materialise_the_same_trees` and `other_filesystems_materialise_the_same_names`: with the Unix rules, the same trees are written exactly.
- `case_only_rename_is_refused`: the known limit, backlog T-016.
- On a real filesystem: `real_hazards_are_refused_on_ntfs` and `real_hazard_trees_materialise_on_ext4`, described in [e2e-tests.md](e2e-tests.md).

### Edit corpora
- **Intents:**
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1
  - [INT-0009](../../../intents/INT-0009-reversible-by-default.md) AC-3

Sections `int_edit_corpora_*`. The oracle is an independent model, with its own op history. It predicts each `undo` and `restore`, and its op count is checked against `log`.

| Corpus | Windows | Linux |
|---|---|---|
| `edit_corpus_loses_nothing` | 500 sequences, 6,000 snapshots, 470 undos, 496 restores, 16 refusals, 6,416 ops checked, 0 lost edits | identical |
| `long_edit_sequences_reach_refusals` | 200 sequences × 40 steps, 8,000 snapshots, 662 undos, 656 restores, 31 refusals (10 blocked), 7,389 ops checked, 0 lost edits | identical |
| `real_edit_corpus_loses_nothing` | NTFS: 20 sequences, 240 snapshots, 25 undos, 15 restores, 3 refusals, 246 ops checked, 0 lost edits | ext4: 20 sequences, 240 snapshots, 26 undos, 12 restores, 3 refusals, 240 ops checked, 0 lost edits |

The real corpus differs between systems because it uses the host's executable bit and name rules, so the model takes different branches. Each run is checked against its own model.

`ignore_rule_corpus` (47 paths in 18 cases agree with Git) and `real_snapshot_id_is_host_independent` (changeset `blake3:9ad9ad90…28c3`) give Sprint 2's results.

### Probes
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-4

| Probe | Result |
|---|---|
| `CARGO_TARGET_DIR` outside the checkout, on both systems | `cargo ci` exits 1 with 0 steps run |
| `std::fs::read` and `Path::exists` in `store/src` | Clippy: disallowed method, exit 101 |
| `getrandom::u64()` in `sim/src/rng.rs` | Clippy: disallowed method, exit 101 |
| `std::hash::RandomState` in `store/src` (new) | Clippy: disallowed type, exit 101 |
| `GitignoreBuilder::add` and `WalkBuilder` in `workspace/src` | Clippy: disallowed method and type, exit 101 |

Each probe file was restored, leaving 0 changed paths.
