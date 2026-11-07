Finalized - DO NOT EDIT

# Sprint 2 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-6 specified before or with code (provisional) | T-002 / the spec SHALL define `changeset`, `view`, `path`, with worked examples; T-003 / the spec SHALL define the revised `operation` in the same commit as its encoder | `spec_registries_match_code`, `spec_examples_match_golden_files`; commit order checked in the code review |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1 canonical round trip | T-002, T-003 / WHEN any value is encoded and decoded THEN the codec SHALL return it; accepted bodies re-encode | `changeset_round_trips`, `view_round_trips`, `operation_round_trips`, `accepted_changeset_bytes_reencode`, `accepted_view_bytes_reencode`, `accepted_operation_bytes_reencode` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1 rejection of non-canonical input | T-002 / unsorted or duplicate keys SHALL fail with `UnsortedOrDuplicateKey`; unknown tag, unknown mode, symlink mode, invalid path, trailing bytes SHALL fail with their named errors | `changeset_rejects_unsorted_paths`, `changeset_rejects_duplicate_paths`, `view_rejects_unsorted_heads`, `view_rejects_duplicate_workspaces`, `changeset_rejects_unknown_entry_tag`, `changeset_rejects_unknown_mode`, `changeset_rejects_symlink_mode`, `changeset_rejects_invalid_paths`, `bodies_reject_trailing_bytes` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1 valid paths | T-002 / RepoPath SHALL reject each invalid form and accept the valid ones | `repo_path_rejects_invalid_forms`, `repo_path_accepts_valid_forms` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-5 total decoding; registry | T-002 / 0x02 and 0x04 accepted, 0x05 reserved; random bodies never panic; T-003 / the same for `operation` | `defined_kinds_are_accepted`, `event_kind_stays_reserved`, `kind_registry_is_complete`, `changeset_decoding_never_panics`, `view_decoding_never_panics`, `operation_decoding_never_panics` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-3 golden files | T-002, T-003 / the golden files SHALL match byte for byte | `golden_v1_corpus_matches`, `golden_corpus_has_the_spec_types`, `every_golden_file_is_a_corpus_case`; GitHub matrix at the checkpoint |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1 closure through references | T-003 / references of each kind; missing or damaged reference refused; missing view refused; present view and parents synced before the op | `references_of_each_kind`, `put_with_missing_reference_writes_nothing`, `put_with_damaged_reference_writes_nothing`, `commit_with_missing_view_writes_nothing`, `commit_with_damaged_view_writes_nothing`, `commit_resyncs_view_and_parents`, `fsck_lists_unreachable_objects_as_leftovers` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1, AC-2 under crashes | T-003 / matrix with blobs, changesets and views, race run; T-007 / Snapshot scenario | `crash_matrix_finds_no_violations` (asserts both race kinds run in every mode), `snapshot_matrix_finds_no_violations` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-4 fsck | T-003 / fsck reports a missing blob, changeset or view | `fsck_detects_missing_references` (one fixture per reference kind), `fsck_detects_every_truncation`, `fsck_detects_every_bit_flip`, `fsck_is_clean_on_matrix_stores` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 determinism | T-004 / scan deterministic; T-007 / each Snapshot cell reproduces; T-004 / store digest equal on both systems | `simfs_runs_are_deterministic` (with scan), `matrix_runs_are_reproducible`, `snapshot_matrix_is_reproducible`; digests compared in the integration log |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-2 crash model for the new syscall | T-004 / scan takes no effect at its crash point and counts as one syscall | `every_syscall_has_no_effect_at_its_crash_point` (extended to `scan`), `scan_is_one_syscall`, `scan_lists_kinds_sorted` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-3 negative controls | T-003 / `UnsyncedReferences` caught in the redesigned TwoWriters; `UnsyncedObjects` gated on every object data sync; Sprint 1 controls still caught; T-007 / `UnsyncedObjects` caught in Snapshot | `unsynced_references_variant_is_caught`, `unsynced_objects_variant_is_caught`, `pointer_file_variant_is_caught`, `unsynced_init_variant_is_caught`, `write_in_place_variant_is_caught`, `snapshot_catches_unsynced_objects` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | supporting: seeded and OS randomness | T-004 / `os_seed` from getrandom, real half only | `os_seed_varies`; `policy_confines_getrandom_to_sim` (the `deps` rule and the `sim/clippy.toml` bans); probe: `getrandom::u64` in `sim/src/simfs.rs` fails Clippy |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-2 host-independent paths | T-005 / the same nested tree snapshotted on NTFS and ext4 SHALL give one recorded changeset id | `real_snapshot_id_is_host_independent` (run on Windows and Linux) |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-7 names with no UTF-8 form | T-004 / RealFs returns Raw names, never lossy; display escaping distinct from every UTF-8 name; T-005 / reported and left alone | `scan_reports_raw_names` (SimFs), `real_scan_keeps_non_utf8_names` (byte `0xff` on Linux, unpaired surrogate on Windows), `raw_display_never_equals_a_utf8_display` (incl. a literal `a\xffb` UTF-8 name on Linux), `snapshot_reports_and_leaves_raw_names`, E2E `praps_status_reports_a_non_utf8_name` |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1 suite on both systems | every task / all tests on both systems | the integration run on the Windows host and in WSL ext4; E2E `fresh_clone_ci`; `real_scan_meets_the_contract`, `real_scan_reports_the_executable_bit`, `real_edit_corpus_loses_nothing` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1 auto-snapshot | T-005 / init and its completion; open errors; snapshot and its no-op; new; status A/M/D; mode rules; several heads stop; T-007 / 0 lost edits, Snapshot liveness | `init_creates_a_root_change`, `init_makes_praps_dir_durable`, `init_is_idempotent`, `init_completes_an_interrupted_init`, `open_without_ops_fails`, `several_heads_stop_every_command`, `init_with_store_uses_the_given_store`, `change_ids_follow_the_seed`, `snapshot_replaces_its_head`, `snapshot_records_each_edit_kind`, `unchanged_working_copy_writes_nothing`, `new_starts_a_child_change`, `status_lists_added_modified_deleted_after_new`, `status_lists_changes_sorted`, `executable_bit_is_recorded`, `no_executable_bit_keeps_the_recorded_mode`, `merge_changesets_are_refused`, `edit_corpus_loses_nothing`, `real_edit_corpus_loses_nothing`, `snapshot_matrix_finds_no_violations` (liveness) |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-2 ignore rules | T-005 / matcher composition, precedence, case sensitivity, always-ignored dirs, tracked files under ignored paths; T-008 / rules documented | `ignore_rule_corpus` (Git-recorded), `praps_and_git_dirs_are_always_ignored`, `tracked_file_stays_tracked_when_ignored`, `tracked_file_under_ignored_dir_is_snapshotted`, `ignore_is_case_sensitive`, review of `docs/spec/working-copy.md` |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1 through the CLI | T-006 / init, new, status, fsck on a working copy; usage | `praps_init_creates_a_repository`, `praps_init_is_idempotent`, `praps_new_prints_a_change_id`, `praps_status_lists_changes`, `praps_status_outside_a_repository_fails`, `praps_status_with_several_heads_fails`, `praps_fsck_checks_a_working_copy`, `praps_rejects_flags_and_extra_arguments`, E2E `praps_end_to_end` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-1, AC-8; the filesystem boundary | T-005 / new crate under policy; `ignore`'s filesystem APIs banned | `cargo policy` rules `unsafe`, `names`, `fs-boundary`, `publish`; `policy_enforces_the_fs_boundary` (workspace fixture, missing `ignore` bans rejected); probe: `GitignoreBuilder::add` in `workspace/src` fails Clippy |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-5 credits | T-004, T-008 / prior-art credits | `cargo policy` rule `credits`; `cargo deny check` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-2 to AC-4, AC-3 GitHub halves (Sprint 1) | T-001 / runs recorded from `gh` output | review of `ci-sprint1.md` against `gh run view` |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | sign-offs | T-001 / recorded only when given | review of the intents' Transition history against the conversation |

## Unit Tests
### T-002 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- `changeset_round_trips`, `view_round_trips`: proptest, 10,000 random values each → equal after decoding.
- `accepted_changeset_bytes_reencode`, `accepted_view_bytes_reencode`: proptest over byte strings; every accepted body re-encodes to its input.
- `changeset_rejects_unsorted_paths`, `changeset_rejects_duplicate_paths`, `view_rejects_unsorted_heads`, `view_rejects_duplicate_workspaces` → `UnsortedOrDuplicateKey`.
- `changeset_rejects_unknown_entry_tag` → `UnknownEntryTag`; `changeset_rejects_unknown_mode` → `UnknownMode`; `changeset_rejects_symlink_mode` → `ReservedMode`; `changeset_rejects_invalid_paths` → `InvalidPath`; `bodies_reject_trailing_bytes` → the trailing-bytes error.
- `defined_kinds_are_accepted`, `event_kind_stays_reserved`, `kind_registry_is_complete`: `from_id` over `0x00..=0xff`.
- `changeset_decoding_never_panics`, `view_decoding_never_panics`: 100,000 random byte strings each.
- `repo_path_rejects_invalid_forms`: empty, `/a`, `a/`, `a//b`, `.`, `a/./b`, `a/..`, NUL, `\` → rejected. `repo_path_accepts_valid_forms`: `a`, `a/b.c`, `ü/x`, `.gitignore` → accepted.
- `golden_v1_corpus_matches`, `golden_corpus_has_the_spec_types`, `every_golden_file_is_a_corpus_case`, `spec_registries_match_code`, `spec_examples_match_golden_files`: byte comparison against `tests/golden/v1/` and the spec.

### T-003 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `operation_round_trips`, `accepted_operation_bytes_reencode`, `operation_decoding_never_panics`, the revised `operation-root.bin` and `operation-merge.bin` in `golden_v1_corpus_matches`.
- `references_of_each_kind`: a blob, a changeset with two parents, two puts and a delete, a view, an operation → exactly the named ids.
- `put_with_missing_reference_writes_nothing`, `put_with_damaged_reference_writes_nothing`, `commit_with_missing_view_writes_nothing`, `commit_with_damaged_view_writes_nothing`: SimFs; error `NotFound`; the tree is unchanged.
- `fsck_lists_unreachable_objects_as_leftovers`: a changeset no op reaches, with a missing blob → a leftover, not a fault; the same changeset reached from an op → a missing reference.
- `commit_resyncs_view_and_parents`: the SimFs trace shows `sync_file` of the view and `sync_dir` of `objects/`, and the same for each parent op and `ops/`, before the op's `create_new`.
- `fsck_detects_missing_references` (extended): one fixture per kind of reference.
- `unsynced_references_variant_is_caught`: TwoWriters, strict mode → closure violations > 0 in view-race cells and in parent-race cells.
- `unsynced_objects_variant_is_caught` (revised expectations): durability and torn violations > 0 in strict mode.

### T-004 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `scan_lists_kinds_sorted`: SimFs with files, an executable file, a directory, a symlink, an `Other` entry → sorted entries with kinds.
- `scan_reports_raw_names`: SimFs raw byte and code-unit names → `Name::Raw` with the specified display.
- `raw_display_never_equals_a_utf8_display`: property test over raw names and UTF-8 names, including the literal name `a\xffb` against the raw bytes `a 0xff b`.
- `scan_is_one_syscall`, `every_syscall_has_no_effect_at_its_crash_point` (extended), `simfs_runs_are_deterministic` (extended).
- `real_scan_meets_the_contract`: the contract on RealFs in a temporary directory, Windows and Linux. The symlink case reports skipped when Windows denies creating one, and the `Other` case runs on Unix only.
- `real_scan_keeps_non_utf8_names`: on Linux a name with byte `0xff`; on Windows an unpaired surrogate (`OsString::from_wide`) → `Name::Raw`.
- `real_scan_reports_the_executable_bit`: on Unix the bit is reported; on Windows `has_executable_bit()` is `false` and `executable` is always `false`.
- `os_seed_varies`: two calls differ (false failure probability 2⁻⁶⁴).
- `policy_confines_getrandom_to_sim`: fixtures where `getrandom` is a dependency of `store`, and where `sim/clippy.toml` lacks the `getrandom` bans → rejected; the real layout → accepted.

### T-005 unit tests
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `init_creates_a_root_change`, `init_is_idempotent`, `init_completes_an_interrupted_init` (a store with a layout and no op), `open_without_ops_fails`, `several_heads_stop_every_command`, `init_with_store_uses_the_given_store`.
- `init_makes_praps_dir_durable`: SimFs strict mode, crash after `init` is acknowledged → `.praps/store` and the root op survive every recovery seed.
- `change_ids_follow_the_seed`: two workspaces with the same seed mint the same change IDs; different seeds mint different ones.
- `snapshot_replaces_its_head`: after a snapshot, the view's heads are the new changeset in place of the old, with the other heads unchanged.
- `snapshot_records_each_edit_kind`: add, modify, mode change, delete, nested directories, empty file → the changeset's map against the parent tree.
- `unchanged_working_copy_writes_nothing`: the second snapshot's trace has no `create_new`, `write`, `rename` or `sync_*`.
- `new_starts_a_child_change`: new change ID, single parent, empty map, view heads replaced, no working-copy syscall that writes.
- `status_lists_added_modified_deleted_after_new`: after `new`, add, modify content, change a mode and delete → `A`, `M`, `M`, `D`, and the changeset holds `put` and `delete` entries.
- `status_lists_changes_sorted`: a root change with nested paths and reported entries → sorted output.
- `executable_bit_is_recorded` (SimFs, bit on), `no_executable_bit_keeps_the_recorded_mode` (SimFs, bit off: a tracked executable stays executable, a new file is regular).
- `merge_changesets_are_refused`: a hand-built two-parent changeset in the view → error.
- `snapshot_reports_and_leaves_raw_names`: a raw name, a symlink and an `Other` entry → reported with reason and display, untracked, unchanged; siblings snapshotted.
- `ignore_rule_corpus`: fixture of `.gitignore` files, paths, tracked-before flags, and two expected columns (Git's matcher verdict and the snapshot outcome), covering anchoring, `**`, negation, directory-only patterns, a nested file overriding its parent, `!` under an excluded parent, and a tracked file under an ignored directory.
- `praps_and_git_dirs_are_always_ignored`, `tracked_file_stays_tracked_when_ignored`, `tracked_file_under_ignored_dir_is_snapshotted`, `ignore_is_case_sensitive`.
- `real_snapshot_id_is_host_independent`: RealFs, fixed seed, the same nested tree → the constant changeset id, on Windows and Linux.

### T-006 unit tests
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- `praps_init_creates_a_repository`, `praps_init_is_idempotent`, `praps_new_prints_a_change_id`, `praps_status_lists_changes`, `praps_status_outside_a_repository_fails`, `praps_status_with_several_heads_fails`, `praps_fsck_checks_a_working_copy`, `praps_rejects_flags_and_extra_arguments`. Each runs the built binary in a temporary directory and checks exit codes and output lines.

## Integration Tests
### Crash matrices
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `crash_matrix_finds_no_violations`, `matrix_runs_are_reproducible`: Sprint 1's scenarios rebuilt with blobs, changesets and views, and the redesigned TwoWriters. Cell counts, race counts, violation totals and the digest are recorded on both systems and compared.
- `snapshot_matrix_finds_no_violations`, `snapshot_matrix_is_reproducible`, `snapshot_catches_unsynced_objects`: the workspace Snapshot scenario, every syscall × 40 seeds × 2 modes, with closure, durability, recency and liveness.
- The negative controls listed in the traceability table.

### Edit corpora
- **Intents:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `edit_corpus_loses_nothing`: ≥ 500 seeded sequences on SimFs; 0 lost edits against the tracked-or-not-ignored oracle.
- `real_edit_corpus_loses_nothing`: ≥ 20 sequences on RealFs on NTFS and on ext4, with the oracle applying each system's executable-bit rule; 0 lost edits.

### Canonical runner
- `cargo ci` (fmt, clippy, test, policy, book) and `cargo deny check` on the Windows host and in WSL ext4, with OS and `rustc` recorded.

## End-to-End Tests
- **Status:** possible
- `praps_end_to_end`, on a real directory on NTFS and on ext4:
  - `praps init`; create files and a `.gitignore`;
  - `praps status` lists the added files, and not the ignored ones;
  - `praps new`; edit, delete and add files; `praps status` shows `M`, `D` and `A`;
  - `praps fsck .` exits 0.
- `praps_status_reports_a_non_utf8_name`: a non-UTF-8 name on ext4 and an unpaired-surrogate name on NTFS → a `skipped` line with the escaped display; the file is untouched and untracked.
- `fresh_clone_ci` (INT-0004 AC-1, INT-0003 AC-2 to AC-4): a fresh clone in the WSL ext4 home directory runs `cargo ci`.
- Not yet possible:
  - power-loss testing; the simulator stands in (INT-0005);
  - Nighthawk, pending the Tailscale approval (INT-0004 AC-6);
  - several processes and workspaces, unlocked by backlog T-011 with INT-0010 and INT-0014 AC-3;
  - `status` time on 100,000 files, unlocked by the stat cache (INT-0014 AC-5).
