# Sprint 3 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | AC-1 injectivity (op log append-only; every prior state recoverable) | T-005 / views are existing view objects; T-007 / every earlier op still present | `ops_are_never_removed` (property over random command sequences), `restore_reaches_every_earlier_state` |
| INT-0009 | AC-2 snapshot-before-overwrite: 200 pairs, 0 collapsed | T-005 / undo and restore snapshot first; T-007 / the pairs corpus | `undo_and_restore_snapshot_first`, `two_hundred_pairs_stay_distinguishable` |
| INT-0009 | AC-3 undo and restore exact for every op type | T-005, T-007 / the corpus over init, snapshot, new, undo, restore | `restore_reaches_every_earlier_state`, `undo_returns_to_the_parent_view`, `undo_twice_toggles`, `undo_on_the_root_op_fails`, E2E `praps_history_end_to_end` |
| INT-0009 | AC-3 through the CLI | T-006 / log, undo, restore and their exit codes | `praps_log_lists_ops_newest_first`, `praps_undo_restores_files`, `praps_undo_with_nothing_to_undo_fails`, `praps_restore_by_prefix`, `praps_restore_unknown_or_ambiguous_fails`, `praps_restore_without_op_is_a_usage_error` |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-3 case-fold collisions, 100% | T-005, T-007 / the case corpus on a case-insensitive `SimFs` | `case_fold_corpus` (≥ 10 cases, including an untracked variant on disk), `case_sensitive_filesystems_materialise_the_same_trees` |
| INT-0004 | AC-4 Windows-unrepresentable names, 100% | T-005, T-007 / the name corpus | `windows_name_corpus` (≥ 30 names), `other_filesystems_materialise_the_same_names`, `real_hazards_are_refused_on_ntfs` (Windows) and `real_hazard_trees_materialise_on_ext4` (Linux) |
| INT-0004 | AC-1 suite on both systems | every task | the integration run on both systems; the `RealFs` contract tests for `remove_dir` and `set_executable` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1 0 lost edits, with writes | T-005 / user edits left in place; T-007 / the extended edit corpus and the crash liveness | `materialise_leaves_user_edits`, `edit_corpus_loses_nothing` (with undo and restore), `real_edit_corpus_loses_nothing`, `materialise_matrix_finds_no_violations` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1, AC-2 under crashes in materialise | T-007 / Materialise scenario: closure, durability | `materialise_matrix_finds_no_violations` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 determinism | T-003 / the new syscalls deterministic; T-007 / cells reproduce | `simfs_runs_are_deterministic` (extended), `materialise_matrix_is_reproducible`, the store digest unchanged |
| INT-0005 | AC-2 crash model for the new syscalls | T-003 / no effect at the crash point; recovery semantics | `every_syscall_has_no_effect_at_its_crash_point` (13 kinds), `unsynced_remove_dir_may_revert`, `unsynced_mode_change_may_revert` |
| INT-0005 | AC-3 negative control | T-007 / `NoRecord` caught | `materialise_catches_no_record` |
| INT-0005 | AC-4, randomness only through `sim` | T-002 / `RandomState` banned; no transitive randomness | `policy_requires_the_random_state_ban`, `policy_rejects_transitive_randomness`; Clippy probe |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-5 credits; the policy rules | T-008, T-002, T-005 | `cargo policy` (`credits`, `deps` with the generalised test-only features); `policy_rejects_variants_outside_dev_dependencies` (extended to `workspace`) |
| INT-0003, [INT-0006](../../../intents/INT-0006-format-v1.md), INT-0004, [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | Sprint 2's GitHub halves; sign-offs | T-001 | review of `ci-sprint2.md` against `gh run view` |

## Unit Tests
### T-002 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `policy_requires_the_random_state_ban`: a boundary config without `std::hash::RandomState` fails `fs-boundary`.
- `policy_rejects_transitive_randomness`: fixture `Cargo.lock`s where `store` depends on `x`, which depends on `rand`, and where `praps` depends on `uuid`, fail `deps` naming the path; `sim` → `getrandom` passes.

### T-003 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `remove_dir` and `set_executable` contract cases inside `sim_fs_meets_the_contract` and `real_fs_meets_the_contract`: removing an empty directory, refusing a non-empty or missing one; setting and clearing the bit, or a no-op without one.
- `unsynced_remove_dir_may_revert`, `unsynced_mode_change_may_revert`: recovery keeps or reverts each across seeds; after a sync, kept.
- `every_syscall_has_no_effect_at_its_crash_point` (13 kinds), `simfs_runs_are_deterministic` (extended).
- `name_rules_report_the_platform`: `RealFs` per platform; `SimFs` configurable.

### T-005 unit tests
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- `log_lists_ops_newest_first`, `undo_returns_to_the_parent_view`, `undo_twice_toggles`, `undo_on_the_root_op_fails`, `undo_and_restore_snapshot_first`.
- `restore_by_unique_prefix`, `restore_unknown_or_ambiguous_prefix_fails` (writes nothing).
- `materialise_writes_removes_and_sets_modes` (files, nested directories, an emptied directory removed, an executable bit).
- `materialise_leaves_user_edits` (a file changed since F keeps its bytes and is snapshotted later), `materialise_keeps_directories_with_other_files`.
- `materialise_syncs_before_dropping_the_record` (trace order).
- `resume_finishes_an_interrupted_materialise`, `resume_drops_a_record_whose_op_never_landed`, `resume_removes_leftover_temporaries`, `temporaries_are_never_snapshotted`.
- `hazards_are_refused_before_any_write` (trace: no write syscall, no record).

### T-006 unit tests
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- `praps_log_lists_ops_newest_first`, `praps_undo_restores_files`, `praps_undo_with_nothing_to_undo_fails`, `praps_restore_by_prefix`, `praps_restore_unknown_or_ambiguous_fails`, `praps_restore_without_op_is_a_usage_error`, and the flag cases in `praps_rejects_flags_and_extra_arguments`.

## Integration Tests
### Reversibility corpora
- **Intents:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- `two_hundred_pairs_stay_distinguishable`: 200 pairs; 0 collapsed.
- `restore_reaches_every_earlier_state`: every op of a corpus covering init, snapshot, new, undo and restore, restored exactly (view object and working copy).
- `ops_are_never_removed`: random sequences; every op ever listed is still listed.

### Hazard corpora
- **Intents:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `case_fold_corpus`, `windows_name_corpus` (100%, 0 files changed); `case_sensitive_filesystems_materialise_the_same_trees`, `other_filesystems_materialise_the_same_names`.
- `real_hazards_are_refused_on_ntfs` (Windows), `real_hazard_trees_materialise_on_ext4` (Linux).

### Crash matrices
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- `materialise_matrix_finds_no_violations`, `materialise_matrix_is_reproducible`, `materialise_catches_no_record`.
- The store and Snapshot matrices, re-run: digests recorded and compared across systems.

### Edit corpora
- **Intents:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- `edit_corpus_loses_nothing` (≥ 500 sequences with undo and restore), `real_edit_corpus_loses_nothing` (≥ 20 on NTFS and ext4).

### Canonical runner
- `cargo ci` and `cargo deny check` on the Windows host and in WSL ext4.

## End-to-End Tests
- **Status:** possible
- `praps_history_end_to_end`, on a real directory on NTFS and on ext4:
  1. `init`, write files, `status`;
  2. `new`, edit and delete, `status`;
  3. `undo`, after which the files are back to their state before `new`;
  4. `restore` of the newer op, after which the edits are back;
  5. `log` lists every op;
  6. `fsck` is clean.
- `real_hazards_are_refused_on_ntfs`: a restore into a tree holding `CON`, `a.` and `A`/`a` is refused with nothing written.
- `fresh_clone_ci` in WSL.
- Not yet possible:
  - power-loss testing (the simulator stands in);
  - Nighthawk (the Tailscale approval);
  - several processes and workspaces (T-011, with INT-0010).
