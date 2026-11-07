# Sprint 2 Unit Test Results

- **Tested head:** `d248ccf6d0d8e3dc65c39370c2f6af11297becf2` on `dev`, committed, with a clean working tree. It includes the fixes for the Test-phase [code review](code-review.md) (`d248ccf`).
- **Runner:** the `test` step of `cargo ci` (`cargo test --workspace --locked`). It ran in the Windows working checkout and in a fresh WSL ext4 clone; see [test-matrix-log.md](test-matrix-log.md), sections `int_ci_windows` and `e2e fresh_clone_ci`.
- **Result:** 327 passed, 0 failed, 0 ignored on both systems. Sprint 1 ended with 242; Sprint 2 adds 85.
  - Some tests run differently on each system:
    - `real_scan_keeps_non_utf8_names` uses byte `0xff` on ext4 and an unpaired surrogate on NTFS;
    - `real_scan_reports_the_executable_bit` sets the bit on Unix and checks there is none on Windows;
    - `real_scan_meets_the_contract` checks a socket as `Other` on Unix only.
  - A symlink was created on both hosts, so nothing printed `SKIPPED`.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13) on `x86_64-pc-windows-msvc` (Windows 11 Pro 10.0.26300) and `x86_64-unknown-linux-gnu` (Ubuntu 26.04.1 LTS, WSL2 kernel 6.6.114.1).
- **Randomness:**
  - Property tests keep proptest's fixed counts: 10,000 cases per property and 100,000 for decoder totality.
  - The simulator, the store and the workspace use `sim::Rng` seeded explicitly; only `praps` uses `sim::os_seed`.
  - The edit corpora and crash matrices are pure functions of their seeds.

| Target | Tests |
|--------|-------|
| `xtask` unit tests | 68 (66 before; +2) |
| `hash` `tests/vectors.rs` | 15 |
| `codec` `tests/properties.rs` | 58 (51 before; +7) |
| `codec` `tests/objects.rs` (new) | 14 |
| `codec` `tests/alloc.rs`, `tests/mutants.rs`, `tests/golden.rs`, `tests/spec.rs` | 4, 1, 11, 3 |
| `book` `tests/book.rs`, `tests/cli.rs` | 33, 4 |
| `sim` `tests/contract.rs`, `tests/real_durability.rs` | 17, 2 |
| `sim` `tests/scan.rs`, `tests/real_scan.rs` (new) | 5, 5 |
| `store` `tests/store.rs` | 23 (17 before; +6) |
| `store` `tests/crash.rs` | 7 (6 before; +1) |
| `store` `tests/fsck.rs`, `tests/real_store.rs`, `tests/relative_root.rs` | 5, 1, 1 |
| `workspace` `tests/workspace.rs` (new) | 28 |
| `workspace` `tests/crash.rs`, `tests/edits.rs`, `tests/real_edits.rs`, `tests/ignore.rs`, `tests/real_workspace.rs` (new) | 3, 1, 1, 1, 1 |
| `praps` `tests/cli.rs` | 13 (5 before; +8) |
| `praps` `tests/e2e.rs` (new) | 2 |
| **Total** | **327** |

Each table below maps one task's EARS clauses, from the locked [build plan](../sprint-plans/build-plan.md), to the tests that assert them.

### T-001: Sprint 1's CI record and sign-offs
| EARS clause | Verification | Result |
|---|---|---|
| The record names each run's head, event, conclusion, jobs, runner OS, `rustc` and counts, from `gh` output | [ci-sprint1.md](ci-sprint1.md), checked against `gh run view` | recorded: 9 jobs success, 242 tests per `cargo ci` leg |
| Missing sign-offs stay pending; backlog T-010 is replaced | backlog T-013; the intents' Transition history is unchanged | none given; carried forward |

### T-002: provisional `changeset` and `view`, repository paths
| EARS clause | Tests | Result |
|---|---|---|
| The spec defines the kinds, bodies, `path` and worked examples | `spec_registries_match_code`, `spec_examples_match_golden_files` (examples `path-nested`, `changeset-root`, `changeset-edit`, `view`; hex derived from the rules by script) | pass |
| Round trip and re-encoding | `changeset_round_trips`, `view_round_trips`, `repo_path_round_trips` and their `accepted_*_bytes_reencode` properties (10,000 each) | pass |
| Unsorted or duplicate keys → `UnsortedOrDuplicateKey` | `changeset_rejects_unsorted_paths`, `changeset_rejects_duplicate_paths`, `view_rejects_unsorted_heads`, `view_rejects_duplicate_workspaces` | pass |
| Unknown tag, unknown mode, symlink mode, invalid path, trailing bytes → named errors | `changeset_rejects_unknown_entry_tag`, `changeset_rejects_unknown_mode`, `changeset_rejects_symlink_mode`, `changeset_rejects_invalid_paths`, `bodies_reject_trailing_bytes` | pass |
| `0x02`, `0x04` defined; `0x05` reserved | `defined_kinds_are_accepted`, `event_kind_stays_reserved`, `kind_registry_is_complete`, `envelope_rejects_unknown_version_and_kind` (4 defined, 1 reserved), `other_kinds_stay_reserved` | pass |
| `RepoPath` rejects invalid forms, accepts valid ones | `repo_path_rejects_invalid_forms`, `repo_path_accepts_valid_forms` | pass |
| Decoding never panics | `changeset_decoding_never_panics`, `view_decoding_never_panics` (100,000 each) | pass |
| Golden files match | `golden_v1_corpus_matches`, `golden_corpus_has_the_spec_types`, `every_golden_file_is_a_corpus_case` | pass |

### T-003: the operation names its view; closure through references
| EARS clause | Tests | Result |
|---|---|---|
| The spec defines `operation` = parents ‖ view, with golden files changed in the same commit | `spec_examples_match_golden_files` (revised `operation-root`, `operation-merge`), `golden_v1_corpus_matches` | pass |
| Operation round trip, re-encoding, never panics | `operation_round_trips`, `accepted_operation_bytes_reencode`, `operation_decoding_never_panics`, `operation_needs_a_view` | pass |
| `references` returns exactly the named ids | `references_of_each_kind` | pass |
| A missing or damaged reference is refused; nothing written | `put_with_missing_reference_writes_nothing`, `put_with_damaged_reference_writes_nothing`, `references_of_the_wrong_kind_are_refused` (CR-11) | pass |
| A missing or damaged view is refused; present view and parents are synced first | `commit_with_missing_view_writes_nothing`, `commit_with_damaged_view_writes_nothing`, `commit_resyncs_view_and_parents` | pass |
| `fsck` reports missing references of every kind; unreachable ones are leftovers | `fsck_detects_missing_references`, `fsck_reports_each_fault_kind`, `fsck_lists_unreachable_objects_as_leftovers` | pass |
| Matrix: 0 violations, both race kinds run | `crash_matrix_finds_no_violations` | pass (integration) |
| `UnsyncedObjects` → durability and torn; other controls still caught | `unsynced_objects_variant_is_caught`, `pointer_file_variant_is_caught`, `unsynced_init_variant_is_caught`, `write_in_place_variant_is_caught` | pass |
| `UnsyncedReferences` → closure in both race kinds | `unsynced_references_variant_is_caught` | pass |

### T-004: scan, executable-bit capability, OS randomness
| EARS clause | Tests | Result |
|---|---|---|
| `scan` returns entries with names and kinds, sorted by display, symlinks not followed | `scan_lists_kinds_sorted`, the scan contract in `sim_fs_meets_the_contract` and `real_fs_meets_the_contract`, `real_scan_meets_the_contract` | pass |
| A name with no UTF-8 form is `Raw`, never lossy | `real_scan_keeps_non_utf8_names` (ext4 bytes, NTFS surrogate), `scan_reports_raw_names` | pass |
| Display escaping is distinct from every UTF-8 name | `raw_display_never_equals_a_utf8_display` (40,000 raw names) | pass |
| The executable-bit capability and bit | `real_scan_reports_the_executable_bit`, `scan_lists_kinds_sorted` (both capabilities) | pass |
| SimFs models bits, symlinks, others and raw names; `scan` is one syscall with no effect at its crash point | `scan_is_one_syscall`, `every_syscall_has_no_effect_at_its_crash_point` (11 kinds), `simfs_refuses_nul_names_and_false_raw_names` (CR-12) | pass |
| `os_seed` from getrandom, confined to `sim` | `os_seed_varies`, `policy_confines_getrandom_to_sim` (with the alias case, CR-07), `policy_enforces_the_fs_boundary` (`getrandom::u64` missing) | pass |
| The matrix digest covers the new node kinds and is host-independent | `matrix_runs_are_reproducible` | pass (integration) |
| Win32 normalisation never rewrites a name (CR-02) | `real_fs_keeps_names_win32_would_rewrite` | pass |

### T-005: the `workspace` crate
| EARS clause | Tests | Result |
|---|---|---|
| `init` creates, syncs and commits the root op; idempotent; completes an interrupted init | `init_creates_a_root_change`, `init_makes_praps_dir_durable` (40 strict seeds), `init_is_idempotent`, `init_completes_an_interrupted_init`, `init_and_open_sync_the_roots_parent` (CR-06) | pass |
| `init_with_store` uses the given store; change IDs follow the seed | `init_with_store_uses_the_given_store`, `change_ids_follow_the_seed` | pass |
| `open` refuses no op and several heads | `open_without_ops_fails`, `several_heads_stop_every_command`, `invalid_op_files_are_refused` (CR-10) | pass |
| Snapshot writes blobs, changeset, view and op; nothing when unchanged; replaces its head | `snapshot_records_each_edit_kind`, `unchanged_working_copy_writes_nothing`, `snapshot_replaces_its_head`, `a_file_vanishing_mid_scan_is_deleted` (CR-05) | pass |
| `new` starts a child change without writing files | `new_starts_a_child_change` | pass |
| Ignore rules; tracked files stay tracked; case-sensitive; always-ignored names | `ignore_rule_corpus` (47 paths, 18 cases, all agreeing with Git), `praps_and_git_dirs_are_always_ignored`, `tracked_file_stays_tracked_when_ignored`, `tracked_file_under_ignored_dir_is_snapshotted`, `ignore_is_case_sensitive`, `always_ignored_names_ignore_case` (CR-13) | pass |
| Reported entries left alone; escaped in full | `snapshot_reports_and_leaves_raw_names`, `reported_paths_are_escaped_in_full` (CR-08) | pass |
| Executable-bit rule on each kind of filesystem | `executable_bit_is_recorded`, `no_executable_bit_keeps_the_recorded_mode`, `no_executable_bit_restores_the_parents_mode` (CR-01) | pass |
| `status` lists A, M, D (including a mode change) | `status_lists_added_modified_deleted_after_new`, `status_lists_changes_sorted` | pass |
| Same tree on NTFS and ext4 gives one changeset id | `real_snapshot_id_is_host_independent` (`blake3:9ad9ad90…28c3` on both) | pass |
| Merges refused | `merge_changesets_are_refused` | pass |
| Policy covers the crate; `ignore` readers banned | `policy_requires_the_ignore_bans`, `policy_enforces_the_fs_boundary`; the Clippy probe | pass |

### T-006: `praps init`, `new`, `status`
| EARS clause | Tests | Result |
|---|---|---|
| `init` creates or reports | `praps_init_creates_a_repository`, `praps_init_is_idempotent` | pass |
| `new` prints a change ID | `praps_new_prints_a_change_id` | pass |
| `status` lists changes and skipped entries; exit 1 outside a repository or with several heads | `praps_status_lists_changes`, `praps_status_outside_a_repository_fails`, `praps_status_with_several_heads_fails` | pass |
| `fsck` on a working copy | `praps_fsck_checks_a_working_copy`, the Sprint 1 fsck tests | pass |
| Flags and extra arguments exit 2 | `praps_rejects_flags_and_extra_arguments`, `usage_errors_exit_two` | pass |

### T-007: crash scenario and edit corpora
See [integration-tests.md](integration-tests.md): `snapshot_matrix_finds_no_violations`, `snapshot_matrix_is_reproducible`, `snapshot_catches_unsynced_objects`, `edit_corpus_loses_nothing`, `real_edit_corpus_loses_nothing`.

### T-008: docs, credits, backlog
| EARS clause | Verification | Result |
|---|---|---|
| `working-copy.md` states the layout, sequences, ignore rules, reported entries, executable rule, several-heads stop and deferred costs; linked from SUMMARY | review; `cargo policy` (book) | pass |
| README lists the crate and commands | review | pass |
| Credits | `cargo policy` rule `credits`; `cargo deny check` | pass |
| T-012 revised; T-011 remains | `docs/work/tasks.md` | pass |
