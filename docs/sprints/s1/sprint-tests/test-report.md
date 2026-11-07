# Sprint 1 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1: determinism | T-002 / `simfs_runs_are_deterministic`, `every_syscall_has_no_effect_at_its_crash_point`; T-005 / `matrix_runs_are_reproducible`: every one of 18,688 cells run twice, recovered-state hashes and traces equal. The digest over all cells, `blake3:4e4d9e8b…99c59`, is equal on Windows and Linux | pass (a formal AC, checked by its stated proxy) | Test evidence links this report |
| INT-0005 | AC-2: crash matrix, ≥ 40 seeds, both directory modes, 0 violations | T-002 / the fault-model tests (`crash_point_boundary_is_exact`, `every_syscall_has_no_effect_at_its_crash_point`, `unsynced_writes_are_kept_dropped_or_torn`, `unsynced_data_stays_at_risk_across_rename`, `unsynced_overwrite_recovers_old_empty_or_torn`, `strict_mode_loses_unsynced_entries`, `strict_mode_reverts_unsynced_renames`, `strict_mode_loses_unsynced_directories`); T-005 / `crash_matrix_finds_no_violations`: 4 scenarios × every syscall × 40 seeds × 2 modes, 9,344 cells per mode, 0 violations of closure, durability, heads, atomicity (prefix), torn files, layout or stray files; 70 cells per mode run the two-writer race | pass, on both systems | as above |
| INT-0005 | AC-3: negative controls each produce violations | `pointer_file_variant_is_caught` (5,985 heads), `unsynced_objects_variant_is_caught` (6,643 closure, 9,412 durability, 9,535 torn), `unsynced_init_variant_is_caught` (20,979 durability, 1,425 closure), plus a fourth control, `write_in_place_variant_is_caught` (563 torn); all strict mode. Lenient mode catches every control except unsynced-init, as its model makes `create_dir` durable at once | pass | as above |
| INT-0005 | AC-4: no randomly seeded hash collections | Clippy `disallowed-types` in every crate, including the member configs of `sim`, `store` and `praps` (`policy_accepts_only_member_configs_with_the_hash_bans`); `cargo ci`'s clippy step on both systems | pass | as above |
| INT-0005 | Consequence: I/O only through the traits | T-003 / `policy_enforces_the_fs_boundary` (config bans; ten attribute opt-outs; Cargo lint settings; eight `#[path]`/`include` escapes; package-name matching); probe: `std::fs::read` and `Path::exists` in `store/src` fail Clippy | pass | as above |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1: recovered ops closed under ancestry, their objects present | T-005 / `crash_matrix_finds_no_violations` (closure, on the raw recovered state); probes P1 and P2 show each durability layer is needed | pass | Test evidence links this report |
| INT-0007 | AC-2: acknowledged ops survive | T-005 / durability of acknowledged ops and objects, 0 violations; `unsynced_init_variant_is_caught` | pass | as above |
| INT-0007 | AC-3 (as revised): heads derived, in-flight ops wholly present or absent, no pointer file | T-004 / `heads_of_a_chain`, `heads_of_a_fork_and_merge`, `store_creates_no_other_files`; T-005 / the heads, torn and stray checks; `pointer_file_variant_is_caught`, `write_in_place_variant_is_caught`; `fsck_is_clean_on_matrix_stores` | pass | as above |
| INT-0007 | AC-4: fsck, 100% detection, 0 false reports | T-006 / `fsck_detects_every_truncation` (243/243), `fsck_detects_every_bit_flip` (40/40), `fsck_detects_missing_references`, `fsck_detects_unparsable_names`, `fsck_is_clean_on_matrix_stores` (16,256 stores, 0 reports); `fsck_reports_each_fault_kind`; E2E `praps_fsck_end_to_end` on NTFS and ext4 | pass | as above |
| INT-0007 | AC-5: four writing processes | not this sprint, deferred by the plan; two writers in one process are tested (supporting) | not yet tested | stays open |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1, AC-5 for the `operation` body | T-001 / `operation_round_trips`, `accepted_operation_bytes_reencode` (10,000 each), the four rejection tests, `operation_rejects_trailing_bytes`, `operation_decoding_never_panics` (100,000) | pass (formal ACs, checked by property-test proxies) | Test evidence links this report |
| INT-0006 | AC-3: golden files | T-001 / `golden_v1_corpus_matches` with `operation-root.bin` and `operation-merge.bin`, `spec_examples_match_golden_files`, `golden_corpus_has_the_spec_types`; GitHub matrix for Sprint 0 recorded in [ci-sprint0.md](ci-sprint0.md) | pass locally; this sprint's GitHub run at the checkpoint | as above |
| INT-0006 | AC-6: spec before code (provisional `operation`) | spec commit `e37351d` before code `b7b1600`; `spec_registries_match_code`. The owner's sign-off covers only Sprint 0's frozen sections; `operation` is provisional (INT-0006 revised) | specified; sign-off pending | as above |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: suite on Windows and Linux, OS and toolchain named | `int_ci_windows` (Windows 11 Pro 10.0.26300) and `e2e fresh_clone_ci` (Ubuntu 26.04.1, WSL2): 242 tests each, rustc 1.99.1 (58cf8f9c3 2026-10-13) | pass | Test evidence links this report |
| INT-0004 | AC-5: Windows durability strategy accepted by a reviewer | [`docs/spec/storage-durability.md`](../../../spec/storage-durability.md); `directory_flush_needs_write_access`, `write_sequence_succeeds_on_the_real_filesystem`, `real_store_reopens_clean`, `relative_roots_work_on_the_real_filesystem` on NTFS and ext4 | documented with evidence; the owner's acceptance is requested at the checkpoint | as above |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-1, AC-8: no `unsafe`; one-word unpublished packages | `cargo policy` rules `unsafe`, `names`, `publish` over `sim`, `store`, `praps`; `RealFs` uses safe `std` only | pass | Test evidence links this report |
| INT-0003 | AC-2 to AC-4 via `cargo ci`: the runner checks its own checkout | T-008 / `xtask_outside_its_target_dir_is_refused`, `xtask_for_another_checkout_is_refused`, `xtask_in_its_own_target_dir_runs`, `usage_errors_come_before_the_checkout_checks`; the outside-target probe exits 1 with 0 steps run on both systems | pass | as above |
| INT-0003 | AC-2 to AC-4 GitHub-matrix halves (Sprint 0) | T-009 / [ci-sprint0.md](ci-sprint0.md): runs 42227550987, 42227551468 and 42303659961, every job success, 171 tests per leg | recorded | as above |
| INT-0003 | AC-5: design credits | `docs/prior-art.md` adds ALICE (OSDI 2014), SplitMix64 and jj's op-heads store; owner sign-off pending | credited; sign-off pending | as above |
| INT-0003 | test-only variants never reach production | `policy_rejects_variants_outside_dev_dependencies` (tables, features, a renamed key, the workspace table) | pass | as above |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | AC-1 sign-offs | T-009 / not given; recorded as carried forward | carried forward | unchanged |

## Summary
- Unit tests: 242 passed / 0 failed / 242 total, on Windows and on Linux
- Integration tests: the canonical runner on both systems, the crash matrix and fsck corpus on both, deny, and the outside-target and boundary probes: all pass
- E2E tests: `praps_fsck_end_to_end` on NTFS and ext4, `fresh_clone_ci` on WSL ext4: pass. Not yet possible: power-loss crashes, Nighthawk, writers in separate processes
- CI status: Sprint 0's GitHub Actions runs are recorded (all success). This sprint's run happens when `dev` is pushed at the checkpoint

## CI Confirmation
- **Head SHA:** `9f138f4df0f2dff4451ab22b1c81511de5d32811`. Later commits change only the Book.
- **CI run:** none yet for this head. The checkpoint push triggers it, and the next sprint records it, as this sprint's T-009 recorded Sprint 0's.
- **Conclusion:** the local canonical runner succeeded on both systems.
- **Confirmations:**
  - `cargo ci` printed `PASS` for fmt, clippy, test, policy and book, then `ci: all 5 steps passed`, and exited 0 in the Windows working checkout and in the fresh WSL ext4 clone;
  - `cargo deny check` reported advisories, bans, licenses and sources ok;
  - the raw output is in [test-matrix-log.md](test-matrix-log.md).

## Failures
None at the tested head. Before it, three reviews raised 22 findings: the Test-phase [code review](code-review.md) (11) and two test critiques ([round 1](critique-round1.md), 8; [round 2](critique.md), 3). All were fixed, or answered by design with the gap recorded, each with a regression test or probe that fails on the earlier code. The most material:
- the crash matrix could not tell which durability layer did the work;
- `init` failed on a relative root on the real filesystem;
- the crash-boundary test was a tautology, letting a syscall take effect at its crash point;
- the matrix accepted a store that writes in place;
- the filesystem boundary could be bypassed by `allow(warnings)`, `#[path]`, `cfg_attr(path)` or a renamed `include!`.

One plan deviation: lenient mode now commits a directory's entry changes in order when a file's entry is synced ([code-review.md](code-review.md), Plan deviation).

## Technical Debt Identified
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-5: writers in separate processes, and with it a simulated process crash (unsynced state visible to the next process), which the store answers by design for now.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-5: the owner's acceptance of the Windows durability strategy; AC-6, Nighthawk.
- [INT-0006](../../../intents/INT-0006-format-v1.md): the provisional `operation` body must gain the view hash, actor, time and signature before the Phase 1 exit gate.
- Sign-offs carried forward: INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6 (Sprint 0's sections).
- Record this sprint's GitHub Actions run at the checkpoint.
- Known limits in [code-review.md](code-review.md): the undocumented Windows directory flush; `praps` calling `inspect` rather than `open` is reviewed, not tested, because `RealFs` keeps no trace.

## Coverage Observations
- INT-0005 AC-1 and INT-0007 AC-1 are `formal` criteria, checked here by their stated simulation proxies; machine-checked proofs belong to INT-0018 to INT-0020.
- The crash evidence is the simulator's, under a fault model at least as harsh as the filesystems it models. The negative controls show that the harness can fail, and the mutation probes show that each durability measure is needed.
- The simulator is deterministic across systems: the same digest over 18,688 cells on Windows and Linux.
