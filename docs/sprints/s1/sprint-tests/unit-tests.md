# Sprint 1 Unit Test Results

- **Tested head:** `133127a4bdb56cf3683f5d302f192cf8d048a01f` on `dev`, committed, with a clean working tree. It includes the fixes from the Test-phase [code review](code-review.md).
- **Runner:** the `test` step of `cargo ci` (`cargo test --workspace --locked`), run in the Windows working checkout and in a fresh WSL ext4 clone ([test-matrix-log.md](test-matrix-log.md), sections `int_ci_windows` and `e2e fresh_clone_ci`).
- **Result:** 236 passed, 0 failed, 0 ignored on both systems. Sprint 0 ended with 171; Sprint 1 adds 65.
  - Two tests are platform-gated. `directory_flush_needs_write_access` has a Windows variant (write access needed for a directory flush) and a Unix variant (a read-only flush succeeds). `real_fs_meets_the_contract` runs on both.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13) on `x86_64-pc-windows-msvc` (Windows 11 Pro 10.0.26300) and `x86_64-unknown-linux-gnu` (Ubuntu 26.04.1 LTS, WSL2 kernel 6.6.114.1).
- **Randomness:**
  - Property tests keep proptest's fixed counts: 10,000 cases per property, 100,000 for decoder totality.
  - The simulator and the store use `sim::Rng` (SplitMix64) seeded explicitly. Every crash-matrix cell is a pure function of scenario, mode, seed and crash point.
- **Stubs:** none new. `SimFs` is a full implementation of `Fs`, not a stub; the real filesystem is exercised through `RealFs` in the contract, durability, relative-root, real-store and CLI tests.

| Target | Tests |
|--------|-------|
| `xtask` unit tests | 66 (59 before; +7) |
| `hash` `tests/vectors.rs` | 15 |
| `codec` `tests/properties.rs` | 49 (41 before; +8) |
| `codec` `tests/alloc.rs`, `tests/mutants.rs`, `tests/golden.rs`, `tests/spec.rs` | 4, 1, 11, 3 |
| `book` `tests/book.rs`, `tests/cli.rs` | 33, 4 |
| `sim` `tests/contract.rs` | 16 |
| `sim` `tests/real_durability.rs` | 2 |
| `store` `tests/store.rs` | 15 |
| `store` `tests/crash.rs` | 5 |
| `store` `tests/fsck.rs` | 5 |
| `store` `tests/real_store.rs`, `tests/relative_root.rs` | 1, 1 |
| `praps` `tests/cli.rs` | 5 |
| **Total** | **236** |

Each table maps one task's EARS clauses (from the locked [build plan](../sprint-plans/build-plan.md)) to the tests that assert them. Tests added for code-review findings sit beside the clause they strengthen.

## T-001: Specify and implement the provisional operation object kind
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md) AC-1, AC-3, AC-5, AC-6 (provisional)

| EARS clause | Test | Result |
|---|---|---|
| The spec defines `0x03 operation`, its body, a provisional note and Change-control bullet, and two worked examples, before the code | `spec_registries_match_code` (status `defined`), `spec_examples_match_golden_files`; commit `e37351d` (spec) precedes `b7b1600` (code) | pass |
| Any operation round-trips; accepted bodies re-encode | `operation_round_trips`, `accepted_operation_bytes_reencode` (10,000 cases each) | pass |
| Unsorted or duplicate parents or objects, or trailing bytes, are rejected | `operation_rejects_unsorted_parents` (at 34), `operation_rejects_duplicate_objects` (at 35), `operation_rejects_trailing_bytes` | pass |
| `0x03` is the operation kind; `0x02`, `0x04`, `0x05` stay reserved | `operation_kind_is_defined` (a blob envelope gives `UnexpectedObjectKind`), `other_kinds_stay_reserved`, `kind_registry_is_complete`, `envelope_rejects_unknown_version_and_kind` (2 defined, 3 reserved) | pass |
| The golden files equal the encodings and the spec's examples | `golden_v1_corpus_matches`, `golden_corpus_has_the_spec_types`, `every_golden_file_is_a_corpus_case`, `spec_examples_match_golden_files`; GitHub matrix at the checkpoint | pass |
| Random bodies never panic | `operation_decoding_never_panics` (100,000 cases) | pass |

## T-002: Filesystem and randomness traits in the sim crate
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1, AC-2; [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1

| EARS clause | Test | Result |
|---|---|---|
| Crash point *k*: earlier syscalls take effect, *k* and later fail without effect | `crash_point_boundary_is_exact` | pass |
| Same seed and crash point → identical durable state and trace | `simfs_runs_are_deterministic` (50 seeds × 11 crash points × 2 modes) | pass |
| Synced data with a synced directory survives | `synced_files_survive_any_crash` (200 seeds) | pass |
| Unsynced data is kept, dropped or torn, never invented | `unsynced_writes_are_kept_dropped_or_torn` | pass |
| Strict mode loses unsynced entries; lenient keeps a synced file's entry | `strict_mode_loses_unsynced_entries`, `strict_mode_reverts_unsynced_renames`, `lenient_mode_keeps_synced_entries` | pass |
| Strict mode loses an unsynced directory with its contents; lenient keeps it | `strict_mode_loses_unsynced_directories` | pass |
| Pending data follows a file across a rename | `unsynced_data_stays_at_risk_across_rename` | pass |
| An unsynced overwrite recovers old, empty or torn | `unsynced_overwrite_recovers_old_empty_or_torn` | pass |
| SplitMix64 matches the reference outputs | `splitmix64_matches_reference` (seeds 1234567 and 0) | pass |
| The contract holds for `RealFs` on Windows and Linux | `real_fs_meets_the_contract`, `rename_replaces_and_create_new_refuses`, `sim_fs_meets_the_contract` | pass, both systems |
| Code review 5, 11: `..` is lexical; a stale removal spares a reused name | `parent_components_are_lexical`, `stale_removals_spare_a_reused_name` | pass |

## T-003: Filesystem boundary
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) Consequences, AC-4; [INT-0003](../../../intents/INT-0003-engineering-baseline.md)

| EARS clause | Test | Result |
|---|---|---|
| Crate configs of `sim`, `store`, `praps` ban the filesystem API and keep the hash bans | `policy_enforces_the_fs_boundary` (missing config; missing `File`, `read`, `Path::exists` bans); probe in `test-matrix-log.md` ("real filesystem calls in store/src fail Clippy": `std::fs::read` and `Path::exists` rejected) | pass |
| Opt-outs only in `sim/src/real.rs` and `tests/` | `policy_enforces_the_fs_boundary`: ten attribute forms (including `allow(warnings)` after code review 3), four Cargo `[lints]` settings, a moved crate found by package name (code review 7), a `src/target` module scanned (code review 8) | pass |
| The `clippy` rule accepts only member configs that keep the hash bans | `policy_accepts_only_member_configs_with_the_hash_bans`, `policy_requires_disallowed_hash_collections` | pass |

## T-004: Object and op-file store
- **Intent:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-3, AC-4's checker, Consequences

| EARS clause | Test | Result |
|---|---|---|
| `init` creates what is missing, syncs, is idempotent and completes an interrupted init | `init_creates_and_syncs_layout`, `init_is_idempotent` | pass |
| `open` needs the layout (and, after code review 4, syncs it) | `open_needs_the_layout`, `open_syncs_the_layout` (50 strict seeds) | pass |
| Objects round-trip under `<alg>-<hex>` names that parse back and are Windows-legal | `objects_round_trip_by_id`, `file_names_round_trip_and_are_portable` | pass |
| A present file is synced again with its directory | `present_files_are_synced_again` (exact trace) | pass |
| Temporary names use a nonce and survive a leftover | `leftover_tmp_does_not_block_a_put` | pass |
| A commit with a missing parent or object writes nothing | `commit_with_missing_parent_writes_nothing`, `commit_with_missing_object_writes_nothing` | pass |
| Heads of chains, forks and merges are derived from op files | `heads_of_a_chain`, `heads_of_a_fork_and_merge` | pass |
| No other file is created; no delete or overwrite API | `store_creates_no_other_files`; the public API (reviewed) | pass |
| `fsck` reports each fault kind and lists leftovers | `fsck_reports_each_fault_kind`, `fsck_lists_tmp_files_as_leftovers` | pass |
| Code review 5: relative roots work on `RealFs` | `relative_roots_work_on_the_real_filesystem` (`repo` and `.`) | pass, both systems |

## T-005: Crash matrix with negative controls
- **Intents:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1 to AC-3; [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1 to AC-3

| EARS clause | Test | Result |
|---|---|---|
| The correct store has 0 violations across all scenarios, crash points, 40 seeds and both modes, checked on the raw recovered state | `crash_matrix_finds_no_violations`: 9,344 cells per mode, 0 violations of closure, durability (ops and, after code review 2, objects), heads, atomicity, layout or stray files | pass, both systems |
| Every cell twice gives equal state and trace | `matrix_runs_are_reproducible`: 18,688 cells; digest `blake3:4e4d9e8b…99c59`, equal on Windows and Linux | pass |
| Each negative control produces closure, durability or heads violations in strict mode; zero-cell runs fail | `pointer_file_variant_is_caught` (5,985 heads), `unsynced_objects_variant_is_caught` (6,643 closure, 9,412 durability), `unsynced_init_variant_is_caught` (19,066 durability, 1,220 closure) | pass |
| The matrix prints its counts | the `int_crash_matrix_*` sections of `test-matrix-log.md` | pass |
| `deps` rejects `variants` outside dev-dependencies | `policy_rejects_variants_outside_dev_dependencies` (three tables, three features, a renamed key, the workspace table) | pass |
| Each durability layer is tested on its own (code review 1, 2) | probes in [code-review.md](code-review.md): P1 (`commit_op` re-sync removed: 9 closure), P2 (`put_object`'s directory sync removed: 1,953 durability) | caught |

## T-006: fsck corruption corpus and praps fsck
- **Intents:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-4; [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-8

| EARS clause | Test | Result |
|---|---|---|
| 100% detection over truncations, bit flips, missing references and bad names | `fsck_detects_every_truncation` (243 of 243), `fsck_detects_every_bit_flip` (40 of 40), `fsck_detects_missing_references`, `fsck_detects_unparsable_names` | pass |
| 0 false reports on matrix-recovered stores | `fsck_is_clean_on_matrix_stores` (16,256 raw recovered states that open) | pass |
| `praps fsck` exits 0, 1 or 2 | `fsck_exits_zero_on_a_healthy_store`, `fsck_without_dir_checks_the_current_directory`, `fsck_lists_corrupt_files_and_exits_one`, `fsck_on_a_non_store_exits_one`, `usage_errors_exit_two` | pass, both systems |

## T-007: Windows durability strategy
- **Intents:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-5; [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)

| EARS clause | Test | Result |
|---|---|---|
| The note states the strategy, evidence and assumptions | [`docs/spec/storage-durability.md`](../../../spec/storage-durability.md), reviewed | written |
| A write+backup-semantics handle flushes; a read-only one is refused | `directory_flush_needs_write_access` (Windows variant) | pass |
| A real store reopens with the same heads and objects, `fsck` clean | `real_store_reopens_clean`, `write_sequence_succeeds_on_the_real_filesystem` | pass, both systems |

## T-008: cargo ci refuses a foreign target directory
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4 (the runner)

| EARS clause | Test | Result |
|---|---|---|
| A binary outside `<root>/target` is refused, exit 1 | `xtask_outside_its_target_dir_is_refused`; probe `CARGO_TARGET_DIR` outside → exit 1, 0 steps run, on both systems | pass |
| A run in another workspace is refused, exit 1 | `xtask_for_another_checkout_is_refused`; manual probe in another Cargo workspace (T-008 evidence) | pass |
| The checks run after argument validation | `usage_errors_come_before_the_checkout_checks` | pass |
| The in-tree binary runs | `xtask_in_its_own_target_dir_runs` (real canonical paths); every `cargo ci` run in this record | pass |

## T-009: Sprint 0's GitHub Actions record
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)

| EARS clause | Verification | Result |
|---|---|---|
| Runs 42227550987 and 42227551468, and PR #2's run 42303659961 recorded | [ci-sprint0.md](ci-sprint0.md), every value read from `gh run view` | recorded |
| Sign-offs not given stay unsigned and are carried forward | `ci-sprint0.md` Owner sign-offs; no chapter records a sign-off | recorded |
