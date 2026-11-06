# Sprint 1 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-6 specified before code (provisional) | T-001 / WHEN the spec commit is made THEN format-v1.md SHALL define `0x03 operation`, its body, a provisional note and Change-control bullet, and worked examples | `spec_registries_match_code`, `spec_examples_match_golden_files` (extended); commit order checked in the code review |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1 canonical round trip | T-001 / WHEN any Operation is encoded and decoded THEN the codec SHALL return it; accepted bodies re-encode | `operation_round_trips`, `accepted_operation_bytes_reencode` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1 rejection of non-canonical input | T-001 / WHEN a body is unsorted, duplicated or has trailing bytes THEN decoding SHALL fail | `operation_rejects_unsorted_parents`, `operation_rejects_duplicate_objects`, `operation_rejects_trailing_bytes` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-5 total decoding; registry | T-001 / 0x03 accepted, 0x02/0x04/0x05 reserved; random bodies never panic | `operation_kind_is_defined`, `other_kinds_stay_reserved`, `operation_decoding_never_panics`, updated `kind_registry_is_complete` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-3 golden files | T-001 / the operation golden files SHALL match byte for byte | `golden_v1_corpus_matches` (extended), `every_golden_file_is_a_corpus_case`; GitHub matrix at the checkpoint |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1 determinism | T-002 / same seed and crash point THEN equal state and trace; T-005 / whole matrix twice, cell by cell | `simfs_runs_are_deterministic`, `matrix_runs_are_reproducible` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-2 crash model | T-002 / crash boundary; synced files survive; unsynced data kept, dropped or torn; strict vs lenient files and directories | `crash_point_boundary_is_exact`, `synced_files_survive_any_crash`, `unsynced_writes_are_kept_dropped_or_torn`, `strict_mode_loses_unsynced_entries`, `strict_mode_reverts_unsynced_renames`, `strict_mode_loses_unsynced_directories`, `lenient_mode_keeps_synced_entries` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-2 crash matrix incl. atomicity | T-005 / correct store: 0 violations of layout, closure, durability, heads and prefix across all cells | `crash_matrix_finds_no_violations` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-3 negative controls | T-005 / each variant SHALL violate closure, durability or heads, counted apart from structural checks; zero-cell runs fail | `pointer_file_variant_is_caught`, `unsynced_objects_variant_is_caught`, `unsynced_init_variant_is_caught` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-4 no random hash collections | T-003 / member configs SHALL keep the hash bans | `cargo clippy` in `cargo ci`; `policy_accepts_only_member_configs_with_the_hash_bans` |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | Consequence: I/O only through the traits | T-003 / crate configs ban real-filesystem API; allows only in `real.rs` and `tests/` | `cargo clippy` in `cargo ci`; `policy_enforces_the_fs_boundary`; probe: `std::fs::read` added to `store/src` fails Clippy |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | supporting: seeded randomness | T-002 / SplitMix64 SHALL match reference outputs | `splitmix64_matches_reference` |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1 suite on both systems | T-002 / RealFs contract on Windows and Linux; all new tests | `real_fs_meets_the_contract`; the integration run on both systems |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-5 Windows durability strategy | T-007 / the note states strategy and evidence; write+backup handle flushes, read-only refused; a real store reopens clean | review of `docs/spec/storage-durability.md`; `directory_flush_needs_write_access` (Windows), `real_store_reopens_clean`; owner acceptance at the checkpoint |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1 ancestry closure | T-005 / closure invariant, presence meaning valid content | `crash_matrix_finds_no_violations`, `unsynced_objects_variant_is_caught` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-2 acknowledged ops survive | T-005 / durability invariant | `crash_matrix_finds_no_violations`, `unsynced_init_variant_is_caught` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-3 derived heads (as revised), no pointer file | T-004 / heads() for chains, forks, merges; only objects, ops, tmp files; T-005 / heads invariant | `heads_of_a_chain`, `heads_of_a_fork_and_merge`, `store_creates_no_other_files`, `crash_matrix_finds_no_violations`, `pointer_file_variant_is_caught` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | Intent: durable init | T-004 / init idempotent, completes interrupted init; open needs the layout; T-005 / layout invariant | `init_creates_and_syncs_layout`, `init_is_idempotent`, `open_needs_the_layout`, `crash_matrix_finds_no_violations` (init scenario) |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | Consequences: names, re-sync of present files, tmp nonces | T-004 / name form round-trips and is Windows-legal; present files re-synced; tmp retries | `file_names_round_trip_and_are_portable`, `present_files_are_synced_again`, `leftover_tmp_does_not_block_a_put`, `crash_matrix_finds_no_violations` (two-writer scenario) |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | Intent: objects before the op | T-004 / commit_op with a missing reference SHALL fail and write nothing | `commit_with_missing_parent_writes_nothing`, `commit_with_missing_object_writes_nothing` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-4 fsck | T-004 / fsck fault classes and leftovers; T-006 / 100% detection, 0 false reports | `fsck_reports_each_fault_kind`, `fsck_lists_tmp_files_as_leftovers`, `fsck_detects_every_truncation`, `fsck_detects_every_bit_flip`, `fsck_detects_missing_references`, `fsck_detects_unparsable_names`, `fsck_is_clean_on_matrix_stores` |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-5 (deferred) multi-writer | T-005 / two interleaved writers in one process (supporting only) | `crash_matrix_finds_no_violations` (two-writer scenario); four processes unlocked by a later sprint |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-1 no unsafe | T-002 / RealFs uses only safe std | `cargo policy` rule `unsafe`; `#![forbid(unsafe_code)]` in each new crate |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-8, AC-9 names and flagless commands | T-006 / praps fsck exits 0, 1 or 2 | `fsck_exits_zero_on_a_healthy_store`, `fsck_lists_corrupt_files_and_exits_one`, `usage_errors_exit_two`; `cargo policy` rule `names` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | test-only variants never reach production | T-005 / no non-dev dependency enables `variants` | `policy_rejects_variants_outside_dev_dependencies` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-5 credits | T-002, T-004 / ALICE, jj, SplitMix64 credited | review of `docs/prior-art.md` in the code review |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-2 to AC-4 via `cargo ci` | T-008 / binary outside `<root>/target` refused (exit 1); other checkout refused (exit 1); in-tree runs | `xtask_outside_its_target_dir_is_refused`, `xtask_for_another_checkout_is_refused`, `xtask_in_its_own_target_dir_runs`; probe: `CARGO_TARGET_DIR` outside the checkout → `cargo ci` exit 1 before any step |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-2 to AC-4, AC-3: GitHub-matrix halves | T-009 / record Sprint 0's runs and PR #2's run | `ci-sprint0.md` cross-checked against `gh run view` output in the Test phase |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | AC-1 sign-offs | T-009 / only sign-offs actually given are recorded | review of the chapters' Transition history against the conversation and PR record |

## Unit Tests
### T-001 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- `operation_round_trips`: proptest, random parent and object sets over both hash algorithms → decode(encode(op)) = op.
- `accepted_operation_bytes_reencode`: proptest over random and mutated bodies → every accepted body re-encodes to the same bytes.
- `operation_rejects_unsorted_parents`, `operation_rejects_duplicate_objects`, `operation_rejects_trailing_bytes`: hand-built bodies → the named errors.
- `operation_decoding_never_panics`: 100,000 random byte strings → a value or an error.
- `operation_kind_is_defined`, `other_kinds_stay_reserved`: id 3 → the operation kind; ids 2, 4 and 5 → `ReservedObjectKind`.
- Updated in `properties.rs`: `expected_header` (`0x02` and `0x04..=0x05` reserved), the class counts, the reserved-kind message test (now for `0x04`), `kind_registry_is_complete`.
- Extended in `spec.rs` and `golden.rs`: `spec_registries_match_code` (status `defined`), `spec_examples_match_golden_files`, `golden_v1_corpus_matches` with `operation-root.bin` (no parents, one object) and `operation-merge.bin` (two parents, two objects across both algorithms).

### T-002 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `crash_point_boundary_is_exact`: crash at *k* → syscall *k* had no effect and failed; later syscalls fail without effect; crash at *k* + 1 → syscall *k* took effect.
- `simfs_runs_are_deterministic`: a scripted sequence, the same seed and crash point twice → equal recovered-state hash and trace.
- `synced_files_survive_any_crash`: create, write, sync, `sync_dir`, then a crash at every later point (no rename-over or removal) → content and name intact in both modes.
- `unsynced_writes_are_kept_dropped_or_torn`: over 200 seeds, all three outcomes seen; torn content is always a prefix of what was written.
- `strict_mode_loses_unsynced_entries`, `strict_mode_reverts_unsynced_renames`, `strict_mode_loses_unsynced_directories` (the directory and everything under it), `lenient_mode_keeps_synced_entries`.
- `rename_replaces_and_create_new_refuses`: the same contract for `SimFs` and `RealFs`.
- `splitmix64_matches_reference`: seeds 0 and 1234567 → the reference sequences (credited in `docs/prior-art.md`).
- `real_fs_meets_the_contract`: the contract suite on `RealFs` in a freshly claimed temporary directory.

### T-003 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `policy_enforces_the_fs_boundary`: fixtures where the crate has no config → fail; an allow in `src/` outside `sim/src/real.rs` → fail; the real layout → pass.
- `policy_accepts_only_member_configs_with_the_hash_bans`: a member config missing `HashSet` → fail; one listing both → pass (replaces the old blanket "no member config" behaviour).
- Probe (recorded in the code review): `std::fs::read` and `Path::exists` added to `store/src` → `cargo clippy` fails on `disallowed_methods`.

### T-004 unit tests
- **Intent:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `init_creates_and_syncs_layout`: the trace shows `sync_dir` for `objects/`, `ops/` and the root, the last syscall being a sync.
- `init_is_idempotent`: second init → success, no writing syscall; init over a half-made layout completes it.
- `open_needs_the_layout`: missing `ops/` → not-a-store error.
- `objects_round_trip_by_id`, `file_names_round_trip_and_are_portable`: `<alg>-<hex>` parses back; no `<>:"/\|?*`, no trailing dot or space.
- `present_files_are_synced_again`: put twice → the second call's trace still syncs the file and the directory.
- `leftover_tmp_does_not_block_a_put`: a leftover `tmp-*` with the next nonce's name → the put succeeds.
- `commit_with_missing_parent_writes_nothing`, `commit_with_missing_object_writes_nothing`: error, and the directory listing is unchanged.
- `heads_of_a_chain`, `heads_of_a_fork_and_merge`: expected head sets.
- `store_creates_no_other_files`: after a scripted session, every path is under `objects/` or `ops/`, and no `tmp-*` remains after successful calls.
- `fsck_reports_each_fault_kind`, `fsck_lists_tmp_files_as_leftovers`: one fixture per fault class, plus a leftover temporary file.

### T-005 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `crash_matrix_finds_no_violations`: 4 scenarios × every syscall index × 40 seeds × 2 modes on the correct store → 0 violations; prints cells and violations.
- `matrix_runs_are_reproducible`: the whole matrix twice → each cell's recovered-state hash and trace equal.
- `pointer_file_variant_is_caught`, `unsynced_objects_variant_is_caught`, `unsynced_init_variant_is_caught`: > 0 closure, durability or heads violations each, in strict mode at least; a variant that ran 0 cells fails.
- `policy_rejects_variants_outside_dev_dependencies`: a fixture with `store = { features = ["variants"] }` under `[dependencies]` → fail.

### T-006 unit tests
- **Intent:** [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `fsck_detects_every_truncation`: each op file truncated to every length 0..len → reported.
- `fsck_detects_every_bit_flip`: each bit of a small object flipped → reported.
- `fsck_detects_missing_references`: a missing object; a missing parent op → reported.
- `fsck_detects_unparsable_names`: a file renamed to `blake3:…` or `zz-…` → reported.
- `fsck_is_clean_on_matrix_stores`: every recovered store from the correct store's matrix → no corruption.
- `fsck_exits_zero_on_a_healthy_store`, `fsck_lists_corrupt_files_and_exits_one`, `usage_errors_exit_two`: run the `praps` binary.

### T-007 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `directory_flush_needs_write_access` (Windows only): a write+backup-semantics handle → `sync_all` Ok; a read-only handle → permission error.
- `real_store_reopens_clean`: init, two objects, three ops through `RealFs`; reopen → same heads and objects; `fsck` clean.

### T-008 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `xtask_outside_its_target_dir_is_refused`: an executable path outside `<root>/target` → exit 1, message names both paths.
- `xtask_for_another_checkout_is_refused`: a located workspace differing from the build-time root → exit 1.
- `xtask_in_its_own_target_dir_runs`: the real paths → no refusal.

### T-009 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md)
- No code. The record is checked by re-reading `gh run view` output for each run id in the Test phase.

## Integration Tests
### Canonical runner on the Windows host and in WSL
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- `cargo ci` on the Windows host and in a WSL ext4 clone: every step PASS. Recorded per system:
  - test counts;
  - the crash matrix's cell count and violations per variant and mode;
  - OS and `rustc`.
- `cargo deny check` and `cargo policy` (with `fs-boundary`): PASS.
- Cross-system determinism: the matrix prints a digest over all cells' recovered-state hashes. It must be equal on Windows and in WSL, since the simulator never touches the real filesystem.
- Probe: `CARGO_TARGET_DIR` set outside the checkout → `cargo ci` exits 1 before any step (T-008).

## End-to-End Tests
- **Status:** possible
- `praps_fsck_end_to_end`, on both systems:
  1. Build a store through `RealFs` with a test helper → `praps fsck` exits 0.
  2. Truncate one op file and flip a bit in one object by hand → both files listed, exit 1.
  3. Restore them → exit 0.
- `fresh_clone_ci`: a fresh clone on WSL ext4 runs `cargo ci` to all PASS.
- Not yet possible:
  - **Power-loss crashes:** would need hardware fault injection, which isn't planned. Simulation and the documented Windows strategy stand in (INT-0004 AC-5 is `asserted`).
  - **Nighthawk runs:** unlocked by the owner approving the Tailscale SSH check (INT-0004 AC-6).
  - **Four writing processes:** unlocked by a later sprint on INT-0007 AC-5.
