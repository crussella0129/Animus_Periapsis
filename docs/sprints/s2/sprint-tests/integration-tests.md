# Sprint 2 Integration Test Results

These are the results at tested head `c5c6965`, the final run of the Test phase. The run at `d248ccf`, before the [round-1 test critique](critique-round1.md), gave the same matrix, corpus and digest figures. Every run is in [test-matrix-log.md](test-matrix-log.md), under the section named here. The two legs ran at the same time, which made both slower.

### Canonical runner on the Windows host and in WSL
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-1 to AC-4, AC-8
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1 to AC-4
  - [INT-0006](../../../intents/INT-0006-format-v1.md) AC-1, AC-3, AC-5
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1 to AC-4
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1, AC-2

| Run | System | Result |
|---|---|---|
| `int_ci_windows` | Windows 11 Pro 10.0.26300, NTFS working checkout, rustc 1.99.1 | `cargo ci`: PASS for fmt, clippy, test, policy and book; exit 0 in 1,270 s; 332 tests passed |
| `e2e fresh_clone_ci` | Ubuntu 26.04.1 (WSL2), fresh ext4 clone, rustc 1.99.1 | cold `cargo ci`: every step PASS, exit 0 in 1,181 s; 332 tests passed; 0 tracked files changed |
| `int_deny_windows` | Windows, cargo-deny 0.20.2 | advisories, bans, licenses and sources all ok, including the new `getrandom` and `ignore` trees |
| `cargo policy` (inside `cargo ci`) | both | every rule PASS, including `fs-boundary` (the `workspace` crate, the `getrandom` and `ignore` bans) and `deps` (getrandom confined to `sim`) |

### Store crash matrix
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1 to AC-3
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1 to AC-3

Sections `int_crash_matrix_windows` and `int_crash_matrix_linux`. Every count is identical on both systems.

**Correct store:** 0 violations of any kind (closure, durability, heads, atomicity, torn, layout, stray). Each mode has 16,420 cells, of which 280 ran the view race and 40 the parent race.

**Negative controls**, in strict mode:

| Control | Cells | Violations |
|---|---|---|
| pointer-file | 17,700 | 10,718 heads |
| unsynced-init | 15,620 | 88,332 durability, 3,065 closure, 12,826 layout |
| unsynced-objects | 14,620 | 55,330 durability, 54,193 torn, 24,988 closure |
| unsynced-refs (new) | 14,740 | 25 closure, in both race kinds |
| write-in-place | 14,320 | 1,134 torn |

In lenient mode every control except unsynced-init is caught as well (unsynced-refs: 19 closure).

**Determinism:** the digest over all 32,840 correct cells is `blake3:d36b8c9c77744d485a362ef3d94cebe40dcb67ca7260eb409f5430c2a13ae812` on Windows and on Linux.

### fsck corpus
- **Intents:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-4

Sections `int_fsck_corpus_*`: 207 of 207 truncations and 40 of 40 bit flips were detected, and there were 0 reports on 30,408 recovered stores, on both systems.

### Snapshot crash matrix
- **Intents:**
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1 to AC-3
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1, AC-2
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1

Sections `int_snapshot_matrix_windows` and `int_snapshot_matrix_linux`.

| Variant | Cells per mode | Violations |
|---|---|---|
| Correct | 12,680 | 0 closure, durability, recency and liveness, in both modes |
| UnsyncedObjects | 11,640 | strict: closure 100,560, recency 9,939, liveness 9,379; lenient: closure 101,135, recency 9,909, liveness 9,508 |

**Determinism:** every cell reproduces, with the same state, trace and violations, and the digest over all 25,360 correct cells is `blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3` on both systems. The cell count grew from 12,400 before the code-review fixes, because `init` now always syncs the root's parent (CR-06).

### Edit corpora and the ignore corpus
- **Intents:**
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1, AC-2
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1, AC-2

Sections `int_edit_corpora_*`:
- **`edit_corpus_loses_nothing`:** 500 seeded sequences, 6,000 snapshots, 0 lost edits on both systems.
  - The oracle is an independent model of the tracked paths and modes, of the change and its parent. It never reads the workspace's state, and after every `new` the tree must be unchanged (critique C-001).
  - The rules toggled include a nested `d2/.gitignore` with a whitelist and a directory rule, and a `.gitignore` is deleted when it holds no active rule (C-007).
  - Half the sequences run without an executable bit, starting from tracked executable files recorded while there was one (C-002).
- **`real_edit_corpus_loses_nothing`:** 20 sequences, 240 snapshots on NTFS and on ext4, against the same model, 0 lost edits. NTFS has no executable bit to set; the no-bit rule is exercised by the simulator corpus and the `no_executable_bit_*` tests.
- **`ignore_rule_corpus`:** 47 paths in 18 cases agree with Git 2.56.0.windows.3's recorded verdicts.
- **`real_snapshot_id_is_host_independent`:** the changeset is `blake3:9ad9ad907eb641bf35322a3db8ed790384b4b1c8bd6f5f4377fe54c2142228c3` on both systems.

### Probes
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) Consequences

| Probe | Result |
|---|---|
| `CARGO_TARGET_DIR` outside the checkout, on both systems | `cargo ci` exits 1 with 0 steps run |
| `std::fs::read` and `Path::exists` in `store/src` | Clippy: disallowed method, exit 101 |
| `getrandom::u64()` in `sim/src/rng.rs` | Clippy: disallowed method, exit 101 |
| `GitignoreBuilder::add` and `WalkBuilder` in `workspace/src` | Clippy: disallowed method and type, exit 101 |

Each probe file was restored, leaving 0 changed paths.
