# Sprint 3 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | AC-1 injectivity: the op log is append-only, and every prior state is recoverable | T-007 / no command removes an op; T-009 / every earlier op stays present | `ops_are_never_removed` (random command sequences, no proptest), `restore_reaches_every_earlier_state` |
| INT-0009 | AC-2 snapshot-before-overwrite: 200 pairs, 0 collapsed | T-007 / snapshot first; T-009 / the pairs corpus | `undo_and_restore_snapshot_first`, `two_hundred_pairs_stay_distinguishable` |
| INT-0009 | AC-3 undo exact after every op type | T-007 / H is the head at start, the target is H's parent's view; T-009 | `undo_after_every_op_type`, `undo_twice_toggles`, `undo_on_the_root_op_fails` |
| INT-0009 | AC-3 restore exact for every op type | T-007, T-009 | `restore_reaches_every_earlier_state`, `restore_by_unique_prefix`, `restore_unknown_or_ambiguous_prefix_fails` |
| INT-0009 | AC-3 through the CLI | T-008 | `praps_log_lists_ops_newest_first`, `praps_undo_restores_files`, `praps_undo_with_nothing_to_undo_fails`, `praps_restore_by_prefix`, `praps_restore_unknown_or_ambiguous_fails`, `praps_restore_without_op_is_a_usage_error`, `praps_rejects_flags_and_extra_arguments` (extended), E2E `praps_history_end_to_end` |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-3 case-fold collisions, 100%, with case-only renames allowed | T-003 / case-insensitive SimFs; T-005; T-009 | `case_fold_corpus` (≥ 12 cases), `case_only_rename_undoes`, `case_sensitive_filesystems_materialise_the_same_trees`, `simfs_case_insensitive_lookups`, `real_case_rename_undoes_on_ntfs` |
| INT-0004 | AC-4 Windows names, 100%, for the paths written | T-005; T-009 | `windows_name_corpus` (≥ 34 names), `unchanged_hazardous_names_are_not_refused`, `other_filesystems_materialise_the_same_names`, `real_hazards_are_refused_on_ntfs`, `real_hazard_trees_materialise_on_ext4` |
| INT-0004 | AC-1 suite on both systems | every task | the integration run on both systems; the `RealFs` contract for `remove_dir` and `set_executable` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1, 0 lost edits, with writes | T-006 / user edits left in place, blocking entries refused, no name ignored; T-009 | `materialise_leaves_user_edits`, `blocking_entries_are_refused`, `prefix_names_are_tracked`, `edit_corpus_loses_nothing` (with undo and restore), `real_edit_corpus_loses_nothing`, `materialise_matrix_finds_no_violations` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1, AC-2 under crashes in materialise | T-009 | `materialise_matrix_finds_no_violations` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 determinism | T-003, T-009 | `simfs_runs_are_deterministic` (extended), `materialise_matrix_is_reproducible` (state, trace and outcome per cell); the store and Snapshot digests unchanged |
| INT-0005 | AC-2 crash model for the new syscalls | T-003 | `every_syscall_has_no_effect_at_its_crash_point` (13 kinds), `unsynced_remove_dir_may_revert`, `unsynced_mode_change_may_revert`, `remove_dir_then_create_dir_survives_crashes` |
| INT-0005 | AC-3 negative control | T-009 / `NoRecord` caught in both modes | `materialise_catches_no_record` |
| INT-0005 | AC-4, randomness only through `sim` | T-002 | `policy_requires_the_random_state_ban`, `policy_rejects_transitive_randomness`; Clippy probe |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-5 credits; the policy rules | T-010, T-002, T-006 | `cargo policy`; `policy_rejects_variants_outside_dev_dependencies` (extended to `workspace/variants`) |
| INT-0003, [INT-0006](../../../intents/INT-0006-format-v1.md), INT-0004, [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | Sprint 2's GitHub halves; sign-offs | T-001 | review of `ci-sprint2.md` against `gh run view` |

## Unit Tests
### T-002 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `policy_requires_the_random_state_ban`: a boundary config without `std::hash::RandomState` fails `fs-boundary`.
- `policy_rejects_transitive_randomness`:
  - fails: `store` → registry `x` → `rand`;
  - passes: a member with `proptest` as a dev-dependency only, and `store` → `sim` → `getrandom`.

### T-003 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Contract:** `remove_dir` and `set_executable` cases in `sim_fs_meets_the_contract` and `real_fs_meets_the_contract`.
- **Crash model:**
  - `unsynced_remove_dir_may_revert`;
  - `unsynced_mode_change_may_revert`;
  - `remove_dir_then_create_dir_survives_crashes`;
  - `every_syscall_has_no_effect_at_its_crash_point` (13 kinds);
  - `simfs_runs_are_deterministic` (extended).
- **Name rules and case:**
  - `name_rules_report_the_platform`;
  - `simfs_case_insensitive_lookups`: lookup, `create_new` of a variant, case rename, and σ/ς folding.

### T-005 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `case_fold_corpus`, `windows_name_corpus`, `unchanged_hazardous_names_are_not_refused`, `case_only_rename_is_not_a_hazard`.
- `hazards_are_refused_before_any_write`: a clean working copy, so the trace has no write syscall at all.

### T-006 unit tests
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- **Materialise:**
  - `materialise_writes_removes_and_sets_modes`;
  - `materialise_leaves_user_edits`;
  - `materialise_removes_only_directories_it_emptied`;
  - `blocking_entries_are_refused` (an untracked file at a T path, a file where T needs a directory, a directory where T needs a file).
- **Order and naming:**
  - `materialise_syncs_before_dropping_the_record` (trace order, ending with the `.praps` sync);
  - `temporary_names_are_deterministic`.
- **Resume:**
  - `resume_finishes_an_interrupted_materialise`;
  - `resume_drops_a_record_whose_op_never_landed`;
  - `resume_drops_a_record_whose_target_is_not_the_head`;
  - `corrupt_record_is_an_error`;
  - `init_never_resumes`.
- `prefix_names_are_tracked`: a user file named `.praps-tmp-x` is tracked like any other.

### T-007 unit tests
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- `log_lists_ops_newest_first`, `undo_returns_to_the_parent_view`, `undo_twice_toggles`, `undo_on_the_root_op_fails` (snapshot only), `undo_and_restore_snapshot_first`, `restore_by_unique_prefix`, `restore_unknown_or_ambiguous_prefix_fails`.

### T-008 unit tests
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- `praps_log_lists_ops_newest_first`, `praps_undo_restores_files`, `praps_undo_with_nothing_to_undo_fails`, `praps_restore_by_prefix`, `praps_restore_unknown_or_ambiguous_fails`, `praps_restore_without_op_is_a_usage_error`, `praps_rejects_flags_and_extra_arguments` (extended).

## Integration Tests
### Reversibility corpora
- **Intents:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- `two_hundred_pairs_stay_distinguishable`, `restore_reaches_every_earlier_state`, `undo_after_every_op_type`, `ops_are_never_removed`.

### Hazard corpora
- **Intents:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `case_fold_corpus`, `windows_name_corpus` (100%, 0 files changed), `case_sensitive_filesystems_materialise_the_same_trees`, `other_filesystems_materialise_the_same_names`, `case_only_rename_undoes`.
- `real_hazards_are_refused_on_ntfs`, `real_case_rename_undoes_on_ntfs` (Windows); `real_hazard_trees_materialise_on_ext4` (Linux).

### Crash matrices
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- `materialise_matrix_finds_no_violations`: the exact oracle (T, F, or the last acknowledged state).
- `materialise_matrix_is_reproducible`.
- `materialise_catches_no_record`: both modes.
- The store and Snapshot matrices, re-run: digests unchanged and equal across systems.

### Edit corpora
- **Intents:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- `edit_corpus_loses_nothing` (≥ 500 sequences with `undo` and `restore`; the model predicts its own op list, and its count is asserted against `log`).
- `real_edit_corpus_loses_nothing` (≥ 20 on NTFS and ext4).

### Canonical runner
- `cargo ci` and `cargo deny check` on the Windows host and in WSL ext4.

## End-to-End Tests
- **Status:** possible
- `praps_history_end_to_end`, on NTFS and ext4:
  1. `init`, write files, `status`;
  2. `new`, edit and delete, `status`;
  3. `undo`, after which the files are back to their state before `new`;
  4. `restore` of the newer op, after which the edits are back;
  5. `log` lists every op;
  6. `fsck` is clean.
- `real_hazards_are_refused_on_ntfs` and `real_case_rename_undoes_on_ntfs`.
- `fresh_clone_ci` in WSL.
- Not yet possible:
  - power-loss testing;
  - Nighthawk (the Tailscale approval);
  - several processes and workspaces (T-011 with INT-0010);
  - probe-based case detection (the new backlog task).
