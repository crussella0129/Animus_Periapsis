# Completed Tasks Log (Append-Only)

## T-001 (sprint 0)
- **Description:** Workspace, guardrails, policy checker and credits
- **Intent:** [INT-0003](../intents/INT-0003-engineering-baseline.md), [INT-0005](../intents/INT-0005-deterministic-simulation-testing.md)
- **Completed:** 2026-10-31T21:03:40Z
- **Files modified:** Cargo.toml, Cargo.lock, rust-toolchain.toml, deny.toml, clippy.toml, .gitattributes, .cargo/config.toml, docs/prior-art.md, crates/xtask/Cargo.toml, crates/xtask/src/main.rs, crates/xtask/src/policy.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo policy` → PASS for lints, unsafe, toolchain, lockfile, publish, deps, credits, clippy, names (exit 0); `cargo policy --fix` → usage on stderr, exit 2; `cargo test --workspace --locked` → 15 passed; `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo fmt --all --check` clean; `cargo deny check` (cargo-deny 0.20.2) → advisories ok, bans ok, licenses ok, sources ok; probe in a scratch copy with a planted `std::collections::HashMap` → Clippy fails on `disallowed_types`.
- **Commit:** `f89c74156ea2b0fef17ee52ce29cefec98bd7100`

## T-002 (sprint 0)
- **Description:** Format v1 specification
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md)
- **Completed:** 2026-10-31T21:05:20Z
- **Files modified:** docs/spec/format-v1.md, crates/xtask/src/policy.rs, docs/intents/INT-0006-format-v1.md
- **Evidence:** Windows 11 host, rustc 1.99.1. The spec lands before any hash or codec code. `cargo policy` → PASS for all 10 rules including the new `spec` rule (exit 0); `cargo test --workspace --locked` → 16 passed, including `policy_requires_format_spec` (missing spec, missing Object identity section, empty Object kinds, empty Worked examples → FAIL `spec`); fmt and Clippy clean.
- **Commit:** `951be4fe76da7c7e754f3914b49ac3a4ef09d39d`

## T-003 (sprint 0)
- **Description:** Algorithm-tagged hashes in the hash crate
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md), [INT-0003](../intents/INT-0003-engineering-baseline.md)
- **Completed:** 2026-10-31T21:06:56Z
- **Files modified:** Cargo.toml, Cargo.lock, docs/prior-art.md, crates/hash/Cargo.toml, crates/hash/src/lib.rs, crates/hash/tests/vectors.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p hash` → 11 passed (35 official BLAKE3 vectors, SHA-256 FIPS vectors, registry ids 0..=255, `hash_registry_is_complete`, digest lengths, cross-algorithm order, binary and text round trips and rejections, chunked = one-shot). `cargo tree --workspace --target all -e normal,build -i cc --depth 1` → `cc` ← `blake3` only, the single C-build `yes` row. `cargo tree --workspace -e features -i blake3` → `blake3 feature "pure"`. `cargo policy` → 10 rules PASS. `cargo deny check` → advisories ok, bans ok, licenses ok, sources ok. deny.toml needed no change.
- **Commit:** `0fdb12a3c769dbaeecee195aba3ecbbfee1c75f8`

## T-004 (sprint 0)
- **Description:** Canonical binary encoding in the codec crate
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md)
- **Completed:** 2026-10-31T21:09:14Z
- **Files modified:** Cargo.toml, Cargo.lock, crates/codec/Cargo.toml, crates/codec/src/lib.rs, crates/codec/src/error.rs, crates/codec/src/wire.rs, crates/codec/src/envelope.rs, crates/codec/tests/properties.rs, crates/codec/tests/mutants.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p codec` → 38 passed in 6.4 s: 13 round-trip and 13 mutation properties at 10,000 cases each (u64, u32, bool, String, Bytes, HashRef, Option, Vec, BTreeSet, BTreeMap, pair, nested map, blob envelope); `decode_is_total_on_random_bytes` (100,000 cases, every type and the envelope); `exhaustive_mutants_reencode` (every single-byte substitution, insertion and deletion of one value per type, at least one decoding mutant each); known varint encodings; non-canonical varints, bad bool, option tag and UTF-8, and `u64::MAX` length and count prefixes all rejected at offset 2 (UTF-8 at the first content byte, 3); `u32` range; strictly increasing keys; trailing bytes; reserved versus unknown kinds; whole-envelope object id; kind registry. No new third-party dependency; `cargo policy` 10 PASS; `cargo deny check` ok.
- **Commit:** `1f747f36a54f6d32bd8ffdb9ff0837cd24415e25`

## T-005 (sprint 0)
- **Description:** Golden corpus and spec consistency
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md), [INT-0004](../intents/INT-0004-windows-and-linux.md)
- **Completed:** 2026-10-31T21:11:01Z
- **Files modified:** .gitattributes, crates/xtask/src/policy.rs, crates/codec/tests/golden.rs, crates/codec/tests/golden/v1/*.bin (23 files), crates/codec/tests/spec.rs, docs/intents/INT-0004-windows-and-linux.md
- **Evidence:** Windows 11 host, rustc 1.99.1. Golden files created once with `PERI_GOLDEN_CREATE=1 cargo test -p codec --test golden golden_v1_corpus_matches`; later runs compare only. `spec_examples_match_golden_files` passed on its first run: all 23 hand-derived worked-example hex strings in the spec equal the encoder output, the encoder's independent check. `spec_registries_match_code` passed (hash and kind tables, reserved rows included). Golden tests 5 passed: corpus matches; one changed byte fails, naming only that file; a missing file fails without create mode; create mode never overwrites; `create_mode` true only for "1". `git check-attr` reports the .bin files as binary. `cargo policy` → 11 rules PASS, including the new `golden` rule (`policy_requires_binary_golden_attr`).
- **Commit:** `8f75733232dc44de4d6776a903ced35ddc6e3afd`

## T-006 (sprint 0)
- **Description:** Typed-acceptance validator in the book crate
- **Intent:** [INT-0002](../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../intents/INT-0003-engineering-baseline.md)
- **Completed:** 2026-10-31T21:15:04Z
- **Files modified:** Cargo.toml, Cargo.lock, docs/prior-art.md, docs/.sprint-loop-book, crates/book/Cargo.toml, crates/book/src/lib.rs, crates/book/src/main.rs, crates/book/tests/book.rs, crates/book/tests/cli.rs, crates/book/tests/fixtures/sprint-loops/ (marker, NOTICE.md, two chapters), docs/intents/INT-0002-typed-acceptance-criteria.md
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p book` → 14 passed: every typed-line rule including AC-0, each with exactly one violation; `missing` for an absent or empty section only; backtick and tilde fences, including a fenced criteria heading; wrapped typed claims; declared versus undeclared; CRLF built in memory; unknown declaration and marker-less directory refused; status truth table and 10,000-case consistency property; this Book strict-clean; fixtures give 5 and 9 criteria with the joined claim; CLI exit codes 0, 1 and 2, with stderr on every 2. `cargo book` → 31 chapters, 164 criteria, 0 violations, exit 0. `cargo book --strict` → exit 2. `cargo book ~/Animus_Sprint_Loops/docs` at e8c3a58 → 13 chapters, 82 criteria (equal to the independent count), 0 violations. Sprint Loops helpers accept `typed-acceptance: 1` (check-book valid, substrate-complete). `cargo policy` 11 PASS.
- **Commit:** `df87da1e8125010eb16c2dcfdf8b9e1022cc4b06`

## T-007 (sprint 0)
- **Description:** Canonical suite runner and CI workflow
- **Intent:** [INT-0003](../intents/INT-0003-engineering-baseline.md), [INT-0004](../intents/INT-0004-windows-and-linux.md), [INT-0002](../intents/INT-0002-typed-acceptance-criteria.md)
- **Completed:** 2026-10-31T21:16:49Z
- **Files modified:** crates/xtask/Cargo.toml, crates/xtask/src/main.rs, crates/xtask/src/ci.rs, .github/workflows/ci.yml, .github/dependabot.yml, Cargo.lock
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p xtask` → 22 passed, including `ci_steps_are_ordered_and_locked`; `ci_failures_do_not_stop_later_steps` (two failures, every step runs, exit 1, summary names both); `ci_rejects_extra_arguments`; `test_step_removes_golden_create`; `workflow_matches_ci_contract`. `cargo ci` on this checkout → PASS fmt, clippy, test, policy, book; "ci: all 5 steps passed"; exit 0 in 9.4 s. `cargo ci --fast` → usage on stderr, exit 2. The book and policy steps run in-process; the nested `cargo test` did not rebuild the running xtask binary. The WSL ext4 clone, fresh `core.autocrlf=true` clone and idempotence runs need committed state and run in the Test phase.
- **Commit:** `1a0658ed3fa7e7d9b08cf89f2e509119eaafc2d8`

## T-008 (sprint 0)
- **Description:** README and documentation checks
- **Intent:** [INT-0003](../intents/INT-0003-engineering-baseline.md)
- **Completed:** 2026-10-31T21:18:01Z
- **Files modified:** README.md, crates/xtask/src/main.rs, crates/xtask/src/docs.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `prior_art_credits_every_listed_system` passed: all 36 listed systems, projects and test-data sources are named in docs/prior-art.md (prior-art needed no change). `readme_keeps_statement_and_links_book` passed: the two owner paragraphs from 620f99c are verbatim, including the trailing space; there is a `docs/SUMMARY.md` link; all four commands are documented; and the exact sentence "`cargo ci`, `cargo policy` and `cargo book` take no flags." is present. `cargo ci` → all 5 steps PASS, exit 0.
- **Commit:** `7c0ae6e3a1ebd09af5dbceaf283490ea00f2bd00`

## T-001 (sprint 1)
- **Description:** Specify and implement the provisional operation object kind
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md)
- **Completed:** 2026-11-06T23:12:28Z
- **Files modified:** docs/spec/format-v1.md (spec commit e37351d, before the code), crates/codec/src/operation.rs, crates/codec/src/envelope.rs, crates/codec/src/error.rs, crates/codec/src/lib.rs, crates/codec/tests/golden.rs, crates/codec/tests/properties.rs, crates/codec/tests/golden/v1/operation-root.bin, crates/codec/tests/golden/v1/operation-merge.bin
- **Evidence:** Windows 11 host, rustc 1.99.1. The spec landed first (e37351d): `0x03 operation` defined, body `parents: set<hashref> ‖ objects: set<hashref>`, provisional note and Change-control bullet, worked examples `operation-root` and `operation-merge` derived by hand from the known BLAKE3 and SHA-256 vectors. The golden files written by `PERI_GOLDEN_CREATE=1` equal those examples (`spec_examples_match_golden_files` passes), and `golden_v1_corpus_matches`, `golden_corpus_has_the_spec_types` and `spec_registries_match_code` pass. New tests pass: `operation_round_trips` and `accepted_operation_bytes_reencode` (10,000 cases each), `operation_decoding_never_panics` (100,000 cases), `operation_rejects_unsorted_parents` (at 34), `operation_rejects_duplicate_objects` (at 35), `operation_rejects_trailing_bytes`, `operation_kind_is_defined` (a blob envelope gives `UnexpectedObjectKind`), `other_kinds_stay_reserved` (0x02, 0x04, 0x05). `envelope_rejects_unknown_version_and_kind` now counts 2 defined and 3 reserved kinds. `cargo ci`: PASS fmt, clippy, test (179 tests), policy, book.
- **Commit:** `b7b1600d7f03ebe1fc0bfb6af7015be0f994eb96`

## T-002 (sprint 1)
- **Description:** Filesystem and randomness traits in the sim crate
- **Intent:** [INT-0005](../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../intents/INT-0004-windows-and-linux.md)
- **Completed:** 2026-11-06T23:19:05Z
- **Files modified:** crates/sim/Cargo.toml, crates/sim/src/lib.rs, crates/sim/src/fs.rs, crates/sim/src/real.rs, crates/sim/src/simfs.rs, crates/sim/src/rng.rs, crates/sim/tests/contract.rs, Cargo.lock, docs/prior-art.md
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p sim`: 14 passed, among them `crash_point_boundary_is_exact` (crash at k: syscalls before k took effect, k and later fail without effect), `simfs_runs_are_deterministic` (50 seeds × 11 crash points × 2 modes: equal traces and recovered snapshots), `synced_files_survive_any_crash`, `unsynced_writes_are_kept_dropped_or_torn`, `strict_mode_loses_unsynced_entries`, `strict_mode_reverts_unsynced_renames` (one name or the other, never both), `strict_mode_loses_unsynced_directories` (with their contents; lenient keeps them), `lenient_mode_keeps_synced_entries`, `unsynced_data_stays_at_risk_across_rename`, `unsynced_overwrite_recovers_old_empty_or_torn`, `splitmix64_matches_reference` (seeds 1234567 and 0), `sim_fs_meets_the_contract`, `real_fs_meets_the_contract` and `rename_replaces_and_create_new_refuses` on SimFs and RealFs (Windows; Linux at the Test phase). The first run found a lenient-mode bug (promoting a rename left its source name's pending create behind) and a contract name pair that collides on NTFS; both fixed before this commit. No third-party dependency added. `cargo ci`: PASS fmt, clippy, test, policy, book.
- **Commit:** `9fcc2464d4a0f5d08959567e28667267ec9fd467`

## T-003 (sprint 1)
- **Description:** Filesystem boundary: only sim's real half touches the real filesystem
- **Intent:** [INT-0005](../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../intents/INT-0003-engineering-baseline.md)
- **Completed:** 2026-11-06T23:27:06Z
- **Files modified:** crates/sim/clippy.toml, crates/sim/src/real.rs, crates/xtask/src/policy.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `crates/sim/clippy.toml` repeats the root's HashMap/HashSet bans and disallows the std::fs types File, OpenOptions and DirBuilder, 18 std::fs functions and 10 std::path::Path methods that reach the filesystem; `crates/sim/src/real.rs` alone switches them off. Probe: `use std::{fs, io}; fs::read(p)` and `p.exists()` added to `crates/sim/src/lib.rs` → `cargo clippy` fails on `disallowed_methods` for both; `#![allow(clippy::style)]` there → `cargo policy` FAIL fs-boundary (and FAIL unsafe, since it displaced the first line); file restored. `cargo policy` gains rule `fs-boundary` (12 rules), and rule `clippy` now accepts a member's own clippy.toml only when it lists both hash bans. New tests `policy_enforces_the_fs_boundary` (missing config; a missing File, read or Path::exists ban; nine opt-out forms including `#![allow(clippy::style)]`, `#[expect(...)]`, `cfg_attr`, the old lint names, multi-line and `# ! [` spacing, which the test caught on its first run; allowed in sim/src/real.rs and tests/, refused in sim/src/simfs.rs) and `policy_accepts_only_member_configs_with_the_hash_bans`; the three PASS-report expectations gain `PASS fs-boundary`. `cargo test -p xtask`: 61 passed. `cargo ci`: PASS fmt, clippy, test, policy, book.
- **Commit:** `5319a5e8ed262558f1e38e13fbf5c7e157860234`

## T-004 (sprint 1)
- **Description:** Object and op-file store with derived heads and fsck in the store crate
- **Intent:** [INT-0007](../intents/INT-0007-crash-safe-storage.md)
- **Completed:** 2026-11-06T23:34:41Z
- **Files modified:** crates/store/Cargo.toml, crates/store/clippy.toml, crates/store/src/lib.rs, crates/store/src/names.rs, crates/store/src/fsck.rs, crates/store/tests/store.rs, Cargo.toml, Cargo.lock, docs/prior-art.md, docs/intents/INT-0007-crash-safe-storage.md
- **Evidence:** Windows 11 host, rustc 1.99.1. INT-0007 planned → active. `store` writes every file as tmp-<nonce> → sync_file → rename to `<alg id>-<digest hex>` → sync_dir; `init` creates what is missing and syncs objects/, ops/, the root and its parent; `commit_op` checks and re-syncs the parents and objects it names, then writes the op; heads are recomputed from the op files; no delete or overwrite API. `cargo test -p store`: 14 passed: `init_creates_and_syncs_layout` (layout survives a crash after init for 50 strict seeds), `init_is_idempotent` (no writing syscall the second time; a half-made layout completed), `open_needs_the_layout`, `objects_round_trip_by_id`, `file_names_round_trip_and_are_portable` (200 ids; bad forms including the `blake3:` text form refused), `present_files_are_synced_again` (exact trace), `leftover_tmp_does_not_block_a_put`, `commit_with_missing_parent_writes_nothing`, `commit_with_missing_object_writes_nothing`, `heads_of_a_chain`, `heads_of_a_fork_and_merge`, `store_creates_no_other_files`, `fsck_reports_each_fault_kind` (Unexpected HEAD, BadName, HashMismatch, Undecodable, MissingParent, MissingObject), `fsck_lists_tmp_files_as_leftovers`. `crates/store/clippy.toml` keeps the hash and filesystem bans, so `cargo policy` rule fs-boundary covers `store`. Credits: jj's op-heads store in docs/prior-art.md (the ALICE fault model was credited in T-002). `cargo ci`: PASS fmt, clippy, test, policy, book.
- **Commit:** `b7aa06f5f5e3b11586895caf6d0d14609dd679fa`

## T-005 (sprint 1)
- **Description:** Crash matrix with negative controls
- **Intent:** [INT-0005](../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../intents/INT-0007-crash-safe-storage.md)
- **Completed:** 2026-11-06T23:54:41Z
- **Files modified:** crates/store/Cargo.toml, crates/store/src/lib.rs, crates/store/src/variant.rs, crates/store/tests/common/mod.rs, crates/store/tests/common/matrix.rs, crates/store/tests/crash.rs, crates/sim/src/fs.rs, crates/xtask/src/policy.rs, Cargo.lock
- **Evidence:** Windows 11 host, rustc 1.99.1. Four scenarios (init; a five-op chain; fork and merge; two writers interleaved at the syscall level through a SimFs hook) × 40 seeds × lenient and strict × a crash at every syscall of the uncrashed run and once after it; invariants checked on the raw recovered SimFs state. `crash_matrix_finds_no_violations`: correct store 9,577 cells per mode, 0 violations of closure, durability, heads, atomicity, layout or stray files. `matrix_runs_are_reproducible`: the whole matrix twice, every cell's recovered-state hash and trace equal; digest over 19,154 cells blake3:c0b62cc835ad9d27d7e319ffc76ac91789ff71645c4cdc1d0324ce94b9f8f87c. Negative controls, strict mode: pointer-file 6,187 heads violations (10,857 cells; lenient 6,168), unsynced-objects 7,094 closure (8,642 cells), unsynced-init 9,766 durability and 1,526 closure (8,937 cells). Probe: with the store's re-sync of already-present files removed, the matrix finds 9 closure violations in the two-writer cells (5 lenient, 4 strict); this probe first found that B's own write of Y also synced objects/ and hid the race, so the scenario was restructured (A commits Y's op first; B only finds X) before this commit. `Variant` exists only under store's `variants` feature, enabled by store's self dev-dependency; `cargo policy` rule `deps` now fails a non-dev or workspace-wide enabling of it (`policy_rejects_variants_outside_dev_dependencies`; `policy_flags_invalid_toml` now expects `deps` too, since it reads the root). `sim` gains `Fs for Rc<T>`. `cargo test -p store --test crash`: 5 passed in 59 s. `cargo ci`: PASS fmt, clippy, test, policy, book (2 min 52 s).
- **Commit:** `68872225695794b11b12a819bd4dbb1c5dc3d9e6`

## T-006 (sprint 1)
- **Description:** fsck corruption corpus and the praps fsck command
- **Intent:** [INT-0007](../intents/INT-0007-crash-safe-storage.md), [INT-0003](../intents/INT-0003-engineering-baseline.md)
- **Completed:** 2026-11-07T00:00:59Z
- **Files modified:** crates/store/tests/fsck.rs, crates/praps/Cargo.toml, crates/praps/clippy.toml, crates/praps/src/main.rs, crates/praps/tests/cli.rs, Cargo.lock, README.md
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p store --test fsck`: 5 passed: `fsck_detects_every_truncation` (243 of 243: each of three op files cut to every shorter length), `fsck_detects_every_bit_flip` (40 of 40 bits of a 5-byte object, each reported as HashMismatch), `fsck_detects_missing_references` (a missing object reported by both ops naming it; a missing parent reported once), `fsck_detects_unparsable_names` (`blake3:…`, `zz-00`, `01-ABCD` reported as BadName), `fsck_is_clean_on_matrix_stores` (19,154 recovered stores from the correct store's matrix, 0 reports). `praps` (one-word binary package, publish = false, its own clippy.toml under the fs-boundary rule): `cargo test -p praps`: 5 passed: `fsck_exits_zero_on_a_healthy_store` (RealFs store, `clean`, idempotent), `fsck_without_dir_checks_the_current_directory`, `fsck_lists_corrupt_files_and_exits_one` (hand-truncated op and flipped object listed, exit 1; restored → 0; a leftover tmp file reported as harmless, exit 0), `fsck_on_a_non_store_exits_one`, `usage_errors_exit_two` (no args, --help, -h, a flag after fsck, an unknown subcommand, an extra argument). README's status and crate table name sim, store and praps. `cargo ci`: PASS fmt, clippy, test, policy, book.
- **Commit:** `3ef5094b40dcfb89dc3b5315729775d5c822449c`

## T-007 (sprint 1)
- **Description:** Windows durability strategy and real-filesystem evidence
- **Intent:** [INT-0004](../intents/INT-0004-windows-and-linux.md), [INT-0007](../intents/INT-0007-crash-safe-storage.md)
- **Completed:** 2026-11-07T00:10:03Z
- **Files modified:** docs/spec/storage-durability.md, crates/sim/tests/real_durability.rs, crates/store/tests/real_store.rs, docs/SUMMARY.md
- **Evidence:** Windows 11 host (NTFS), rustc 1.99.1. `docs/spec/storage-durability.md` states the layout and on-disk names, the write sequence, what the simulator models in strict and lenient modes, how rename and sync_dir are realized on Linux (rename(2); fsync of a read-only directory handle) and Windows (std::fs::rename with POSIX semantics where supported; FlushFileBuffers on a directory handle opened with FILE_FLAG_BACKUP_SEMANTICS and write access), the evidence, and what remains asserted (undocumented directory flush, no power-cut test, filesystems not tested, multi-process writers). `cargo test -p sim --test real_durability`: 2 passed — `directory_flush_needs_write_access` (RealFs's handle flushes; a read-only backup-semantics handle fails with PermissionDenied; a plain directory open fails) and `write_sequence_succeeds_on_the_real_filesystem`. `cargo test -p store --test real_store`: `real_store_reopens_clean` (init, two objects, three ops through RealFs; reopened: same heads and objects, fsck clean, no leftovers; init again changes nothing). The Linux (ext4, WSL) runs of these tests are recorded at the Test phase. SUMMARY.md links the format spec and the durability note. `cargo ci`: PASS fmt, clippy, test, policy, book.
- **Commit:** `4755fb3a06bbf757934fbbcc01a5c94510e9a679`

## T-008 (sprint 1)
- **Description:** cargo ci refuses a target directory outside the checkout (absorbs former backlog T-010)
- **Intent:** [INT-0003](../intents/INT-0003-engineering-baseline.md)
- **Completed:** 2026-11-07T00:14:12Z
- **Files modified:** crates/xtask/src/main.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `main` now runs `guarded`: after argument validation (usage errors still exit 2 first), a `ci` or `policy` command is refused with exit 1 when `foreign` finds the canonicalized `current_exe()` outside the canonicalized `<root>/target`, or the canonicalized parent of the manifest `cargo locate-project --workspace` reports (inherited environment) differs from the canonicalized build-time root; `ci::run` and the policy rules are unchanged. New tests: `xtask_outside_its_target_dir_is_refused`, `xtask_for_another_checkout_is_refused`, `xtask_in_its_own_target_dir_runs` (real canonical paths, verbatim-prefixed on Windows), `usage_errors_come_before_the_checkout_checks`; `cargo test -p xtask --bin xtask tests::`: 55 passed. Probes: `CARGO_TARGET_DIR=<scratchpad>/outside-target cargo ci` exits 1 with `xtask: <scratchpad>/outside-target/debug/xtask.exe is not inside C:/Users/charl/Animus_Periapsis/target; a target directory outside the checkout (CARGO_TARGET_DIR or build.target-dir) is refused`, no step run; the same for `cargo policy`; the in-tree binary run from a temporary directory says `the current directory is not in a Cargo workspace` and exits 1; run inside another Cargo workspace it says `built for the checkout at ..., but run in the workspace at ...` and exits 1. Another checkout building into this checkout's in-tree target/ stays an open limit, as the backlog entry stated. The former backlog entry T-010 was removed from tasks.md when the sprint queue was written (b5d6cbf). `cargo ci`: PASS fmt, clippy, test, policy, book.
- **Commit:** `5ffb848336552402aefaef261de9eaf11cad2de9`

## T-009 (sprint 1)
- **Description:** Record Sprint 0's GitHub Actions runs and owner sign-offs (absorbs former backlog T-009)
- **Intent:** [INT-0001](../intents/INT-0001-periapsis-charter.md), [INT-0002](../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../intents/INT-0003-engineering-baseline.md), [INT-0006](../intents/INT-0006-format-v1.md)
- **Completed:** 2026-11-07T00:16:50Z
- **Files modified:** docs/sprints/s1/sprint-tests/ci-sprint0.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0006-format-v1.md
- **Evidence:** Read with `gh run view` and `gh run view --job --log` on 2026-11-06. Runs 42227550987 (push) and 42227551468 (pull request) at 5062157: all six jobs success; both `cargo ci` legs on `ubuntu-26.04` 20261101.186.1 and `windows-2025-vs2026` 20261103.298.1 with rustc 1.99.1 (58cf8f9c3 2026-10-13), 171 tests passed and 0 failed per leg, every step PASS, the three golden tests ok; both `cargo deny check` jobs: advisories, bans, licenses and sources ok. PR #2: run 42237753919 at d17b135 failed only `pins::ci_workflow_is_the_reviewed_one` (the pin catching the checkout bump); run 42303659961 at 2c7e74a success, 171 tests per leg. The record is linked as Test evidence from INT-0003 and INT-0006. None of the four named sign-offs (INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6) has been given; the merge of PR #1 is recorded as checkpoint acceptance only, so INT-0001 and INT-0002 are unchanged and the sign-offs are carried forward. The former backlog entry T-009 was removed from tasks.md when the sprint queue was written (b5d6cbf).
- **Commit:** `1179a7ec935ebc6b340c3b4b18a4fae2eef94d61`

## T-001 (sprint 2)
- **Description:** Record Sprint 1's GitHub Actions runs and owner sign-offs (absorbs former backlog T-010)
- **Intent:** [INT-0003](../intents/INT-0003-engineering-baseline.md), [INT-0004](../intents/INT-0004-windows-and-linux.md), [INT-0006](../intents/INT-0006-format-v1.md), [INT-0001](../intents/INT-0001-periapsis-charter.md), [INT-0002](../intents/INT-0002-typed-acceptance-criteria.md)
- **Completed:** 2026-11-07T14:29:31Z
- **Files modified:** docs/sprints/s2/sprint-tests/ci-sprint1.md, docs/intents/INT-0001-periapsis-charter.md, docs/intents/INT-0002-typed-acceptance-criteria.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0004-windows-and-linux.md, docs/intents/INT-0006-format-v1.md, docs/work/tasks.md
- **Evidence:** Read with `gh run view --json` and `gh run view --job --log` on 2026-11-07. Runs 42329906757 (pull request) and 42329902655 (push) at 9ffa162 and 42375204698 (push to main, merge 5c3b81b): all nine jobs success; every `cargo ci` leg on `ubuntu-26.04` or `windows-2025-vs2026` with rustc 1.99.1 (58cf8f9c3 2026-10-13) passed 242 tests in 29 binaries, every step PASS, with `crash_matrix_finds_no_violations`, `matrix_runs_are_reproducible`, `pointer_file_variant_is_caught`, `write_in_place_variant_is_caught`, `fsck_is_clean_on_matrix_stores` and `golden_v1_corpus_matches` ok; each `cargo deny check` job reported advisories, bans, licenses and sources ok. The matrix's per-cell summary lines are captured by `cargo test` and so absent from the GitHub logs; the record says so. No owner sign-off has been given; the five pending ones are carried forward as backlog T-013, which replaces T-010 (removed when the sprint queue was written). The record is linked as Test evidence from INT-0003 and INT-0006.
- **Commit:** `6d2800da685b12190f31e85b1121b198f5c9017d`

## T-002 (sprint 2)
- **Description:** Specify and implement the provisional changeset and view kinds and repository paths
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md)
- **Completed:** 2026-11-07T14:36:30Z
- **Files modified:** docs/spec/format-v1.md, crates/codec/src/lib.rs, crates/codec/src/envelope.rs, crates/codec/src/error.rs, crates/codec/src/operation.rs, crates/codec/src/changeset.rs, crates/codec/src/view.rs, crates/codec/src/path.rs, crates/codec/tests/golden.rs, crates/codec/tests/properties.rs, crates/codec/tests/objects.rs, crates/codec/tests/golden/v1/path-nested.bin, crates/codec/tests/golden/v1/changeset-root.bin, crates/codec/tests/golden/v1/changeset-edit.bin, crates/codec/tests/golden/v1/view.bin
- **Evidence:** Windows 11 host, rustc 1.99.1. format-v1.md (in the same commit as the code) defines `0x02 changeset` and `0x04 view` as provisional, the `path` type with its Repository paths section, and worked examples `path-nested`, `changeset-root`, `changeset-edit` and `view`, whose hex was derived by script from the spec rules, not from the encoder. `operation` is unchanged, so every user still compiles (`cargo clippy --workspace --all-targets -D warnings` clean). codec gains `RepoPath`/`PathError`, `Changeset`/`Entry`/`Mode`/`CHANGE_ID_LEN`, `View`, the errors `UnknownEntryTag`, `UnknownMode`, `ReservedMode`, `InvalidPath`, and a shared `decode_as`; `RESERVED_KINDS` is now only `0x05 event`. `cargo test -p codec`: all pass, including `changeset_round_trips`, `view_round_trips`, `repo_path_round_trips` and their re-encoding properties (10,000 cases each), `changeset_decoding_never_panics` and `view_decoding_never_panics` (100,000 each), the new `tests/objects.rs` (`repo_path_rejects_invalid_forms`, `repo_path_accepts_valid_forms`, `changeset_rejects_unsorted_paths`, `changeset_rejects_duplicate_paths`, `view_rejects_unsorted_heads`, `view_rejects_duplicate_workspaces`, `changeset_rejects_unknown_entry_tag`, `changeset_rejects_unknown_mode`, `changeset_rejects_symlink_mode`, `changeset_rejects_invalid_paths`, `bodies_reject_trailing_bytes`, `defined_kinds_are_accepted`, `event_kind_stays_reserved`), and the updated `kind_registry_is_complete`, `envelope_rejects_unknown_version_and_kind` (4 defined, 1 reserved), `other_kinds_stay_reserved`, `golden_v1_corpus_matches`, `golden_corpus_has_the_spec_types`, `spec_registries_match_code`, `spec_examples_match_golden_files`. Golden files created with PERI_GOLDEN_CREATE=1 and matched against the spec's hand-derived hex. `cargo fmt`, `cargo policy` pass.
- **Commit:** `fc281d877262a883d7c9782edde455d012cb1015`
