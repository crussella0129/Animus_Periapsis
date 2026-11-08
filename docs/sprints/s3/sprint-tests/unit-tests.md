# Sprint 3 Unit Test Results

- **Tested head:** `70a2a4d70065babca41483f78f5c68a258f38c06` on `dev`, committed, with a clean working tree, for the full matrix runs. It includes the fixes for the Test-phase [code review](code-review.md). The tests added for the [round-1 test critique](critique-round1.md) are in `0678139`; how they were run is under "Round 1 additions" below.
- **Runner:** the `test` step of `cargo ci` (`cargo test --workspace --locked`). It ran in the Windows working checkout and in a fresh WSL ext4 clone. See [test-matrix-log.md](test-matrix-log.md), sections `int_ci_windows` and `e2e fresh_clone_ci`.
- **Result at `70a2a4d`:** 0 failed and 0 ignored on both systems. 403 passed on Windows and 404 on Linux; Sprint 2 ended with 332.
  - The extra Linux test is `real_symlinked_directories_are_not_followed` (CR-05), in `real_hazards.rs`. It needs a real symlink on ext4, so it is compiled for Unix only.
  - Some tests run differently on each system:
    - `real_hazards_are_refused_on_ntfs` runs on Windows only;
    - `real_hazard_trees_materialise_on_ext4` runs on Linux only;
    - `windows_names_through_a_write` leaves out names the host cannot create (`writable_here` drops names with `:` on Windows) without printing anything; `windows_name_corpus`, a pure check, still covers them.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13) on `x86_64-pc-windows-msvc` (Windows 11 Pro 10.0.26300) and `x86_64-unknown-linux-gnu` (Ubuntu 26.04.1 LTS, WSL2 kernel 6.6.114.1).
- **Randomness:**
  - Property tests keep proptest's fixed counts.
  - The simulator, the store and the workspace use `sim::Rng`, seeded explicitly. Only `praps` uses `sim::os_seed`.
  - `std::hash::RandomState` is now banned in every boundary crate (T-002).

The counts below are from `cargo test --workspace --locked -- --list` at the tested head, on Windows.

| Target | Tests |
|--------|-------|
| `xtask` unit tests | 70 (68 before; +2: `policy_requires_the_random_state_ban`, `policy_rejects_transitive_randomness`) |
| `hash` `tests/vectors.rs` | 15 |
| `codec` `tests/properties.rs`, `objects.rs`, `alloc.rs`, `mutants.rs`, `golden.rs`, `spec.rs` | 58, 14, 4, 1, 11, 3 |
| `book` `tests/book.rs`, `tests/cli.rs` | 33, 4 |
| `sim` `tests/contract.rs` | 25 (17 before; +8, for `remove_dir`, `set_executable` and `name_rules`) |
| `sim` `tests/real_durability.rs`, `scan.rs`, `real_scan.rs` | 2, 5, 5 |
| `store` `tests/store.rs`, `crash.rs`, `fsck.rs`, `real_store.rs`, `relative_root.rs` | 23, 7, 5, 1, 1 |
| `workspace` `tests/workspace.rs`, `crash.rs`, `edits.rs`, `ignore.rs`, `real_edits.rs`, `real_workspace.rs` | 31, 3, 2 (+1), 1, 1, 1 |
| `workspace` `tests/hazards.rs` (new) | 9 |
| `workspace` `tests/materialise.rs` (new) | 26 |
| `workspace` `tests/history.rs` (new) | 12 |
| `workspace` `tests/materialise_crash.rs` (new) | 4 |
| `workspace` `tests/real_hazards.rs` (new) | 1 on Windows, 2 on Linux (with CR-05's test) |
| `praps` `tests/cli.rs`, `tests/e2e.rs` | 21, 4 (+1: `praps_history_end_to_end`) |

## By build task

Every named test in the [test plan](../sprint-plans/test-plan.md) exists and passed on both systems.

- **T-002** (`xtask`): the two policy tests. The Clippy probe for `RandomState` is in [integration-tests.md](integration-tests.md).
- **T-003** (`sim`):
  - the `remove_dir` and `set_executable` cases in `sim_fs_meets_the_contract` and `real_fs_meets_the_contract`;
  - `unsynced_remove_dir_may_revert`, `unsynced_mode_change_may_revert`, `create_dir_is_refused_while_its_removal_is_pending`;
  - `every_syscall_has_no_effect_at_its_crash_point`, now over 13 kinds and 14 syscalls;
  - `simfs_runs_are_deterministic` (extended), `name_rules_report_the_platform`, `make_executable_is_not_a_syscall`.
- **T-005** (`hazards.rs`):
  - pure checks of `hazard::check`: `case_fold_corpus`, `windows_name_corpus` (52 hazardous names, as files and as directory components; the T-005 record's "53" is a miscount), `unchanged_hazardous_names_are_not_refused`;
  - through a real write on `SimFs`, with nothing written: `case_corpus_through_a_write`, `windows_names_through_a_write`, `case_only_rename_is_refused`;
  - `hazards_are_refused_before_any_write` is in `history.rs`: through `undo` and `restore`, only the snapshot is written.
- **T-006** (`materialise.rs`): every test the plan names under materialise, order and naming, and resume, plus `prefix_names_are_tracked`. The code review added five here:
  - `removal_temporary_names_are_checked` and `resume_keeps_a_tracked_file_at_a_temporary_name` (CR-01);
  - `control_characters_in_names_are_seen` (CR-02);
  - `edits_during_the_write_are_kept` (CR-03);
  - `resume_skips_a_directory_at_a_temporary_name` (CR-04).

  A sixth, `real_symlinked_directories_are_not_followed` (CR-05, ext4 only), is in `real_hazards.rs`.
- **T-007** (`history.rs`): `log_lists_ops_newest_first`, `undo_returns_to_the_parent_view`, `undo_twice_toggles`, `undo_on_the_root_op_fails`, `undo_and_restore_snapshot_first`, `restore_by_unique_prefix`, `restore_unknown_or_ambiguous_prefix_fails`.
- **T-008** (`praps` `cli.rs`): `praps_log_lists_ops_newest_first`, `praps_undo_restores_files`, `praps_undo_with_nothing_to_undo_fails`, `praps_restore_by_prefix`, `praps_restore_unknown_or_ambiguous_fails`, `praps_restore_without_op_is_a_usage_error`, `praps_prints_notes`, `praps_rejects_flags_and_extra_arguments` (extended).

## Round 1 additions

The [round-1 test critique](critique-round1.md) asked for these. They are committed in `0678139`. Each was run on Windows (NTFS) and on ext4 in WSL, from a copy of the working tree. `cargo clippy --workspace --all-targets -- -D warnings` was clean.

| Test | Concern | Windows | Linux |
|---|---|---|---|
| `praps` `praps_restore_refusals_exit_one` | C-001 | passed: the blocking entry, then the case-only rename as a hazard | passed: the blocking entry only |
| `workspace` `real_names_without_utf8_survive_undo` (`real_hazards.rs`) | C-007 | passed, with an unpaired surrogate | passed, with byte `0xff` |
| `workspace` `real_resume_does_not_follow_a_symlinked_directory` (`real_hazards.rs`) | C-008 | passed, with a directory symlink (none `SKIPPED`) | passed |
| `workspace` `ops_are_never_removed`, tightened | C-006 | passed | passed |
| `praps` `praps_undo_restores_files` and `praps_prints_notes`, tightened | C-010 | passed | passed |
| `workspace` `materialise_matrix_with_windows_rules_finds_no_violations` | C-004 | see [integration-tests.md](integration-tests.md) | same |

The tightened `praps_undo_restores_files` checks the op that `undo` prints. It is the parent of H, the head when `undo` started; here H is the `new` op, and `undo`'s own snapshot of the edits is the op after it. `praps_prints_notes` now checks that the note says the write was abandoned, and names the target op.

With these, the `praps` `cli.rs` count is 22, `real_hazards.rs` is 3 on Windows and 4 on Linux, and `materialise_crash.rs` is 5. That gives 409 tests on Windows and 410 on Linux.
