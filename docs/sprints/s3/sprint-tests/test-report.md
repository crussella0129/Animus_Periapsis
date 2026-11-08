# Sprint 3 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0009](../../../intents/INT-0009-reversible-by-default.md) | AC-1: injectivity; every prior state recoverable from the op log | T-007, T-009 / `ops_are_never_removed` (100 random sequences, `log` equal to the store's ops after every step, no op lost); `restore_reaches_every_earlier_state`; no command removes an op by construction | pass (formal AC, checked by a randomized property proxy) | Test evidence links this report |
| INT-0009 | AC-2: snapshot-before-overwrite, ≥ 200 pairs | `two_hundred_pairs_stay_distinguishable` (200 pairs, 800 runs through `undo` and `restore`, 0 collapsed); `undo_and_restore_snapshot_first` | pass on both systems | as above |
| INT-0009 | AC-3: `undo` and `restore` exact for every op type | `undo_after_every_op_type` (snapshot, new, undo, restore, with file↔directory swaps and executable files; the root op refused); `restore_reaches_every_earlier_state`; `undo_twice_toggles`; the edit corpora (1,132 undos and 1,152 restores against an independent model, plus 40 on NTFS and 38 on ext4, 0 lost edits); the Materialise crash matrix; the CLI tests and E2E `praps_history_end_to_end`; `praps_restore_refusals_exit_one` | pass on both systems | as above |
| INT-0009 | AC-4, AC-5 (erasure, partial clones) | not this sprint | not yet tested | stays open |
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1: 0 lost edits, now that commands also write files | T-006, T-009 / user edits left in place, blocking entries refused, and edits saved during a write kept (`edits_during_the_write_are_kept`). The Materialise crash matrix: 40,960 cells, 0 violations, with a user edit after every crash inside a write and an ignored file kept throughout; 118,306 second-crash cells; 40,960 more under the no-executable-bit rule. The edit corpora, 0 lost edits. | pass on both systems | Test evidence links this report |
| INT-0014 | AC-2 to AC-6 | unchanged this sprint (AC-2's ignore corpus re-run, 47 paths agree with Git) | AC-2 still passes; the rest not yet tested | stays open |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: suite on both systems, OS and toolchain named | `cargo ci` on Windows 11 Pro 10.0.26300 (407 tests) and in a fresh clone on Ubuntu 26.04.1, WSL2 (408), rustc 1.99.1 | pass locally; GitHub half pending T-017 | Test evidence links this report |
| INT-0004 | AC-3: case-fold collisions detected when materialising | `case_fold_corpus` (15 cases), `case_corpus_through_a_write` (refused, nothing written), `real_hazards_are_refused_on_ntfs`; on case-sensitive rules the same trees are written (`case_sensitive_filesystems_materialise_the_same_trees`, `real_hazard_trees_materialise_on_ext4`). A case-only rename is refused (known limit, T-016). | pass | as above |
| INT-0004 | AC-4: Windows-unrepresentable paths named in an error | `windows_name_corpus` (52 names, 17 safe ones), `windows_names_through_a_write`, `unchanged_hazardous_names_are_not_refused`, `real_hazards_are_refused_on_ntfs` (`CON`, `dot.`) | pass | as above |
| INT-0004 | AC-7: names with no UTF-8 form | Sprint 2's tests re-run; `real_names_without_utf8_survive_undo` (kept through `undo` on NTFS and ext4); `control_characters_in_names_are_seen` (CR-02) | kept and never deleted: pass. `undo`, `restore` and `new` do not report the name; only `status` does (backlog T-019). | as above |
| INT-0004 | AC-5, AC-6 | sign-off pending; Nighthawk not reachable | unchanged | unchanged |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1, AC-2 under crashes inside `undo` and `restore` | Materialise matrix: closure and durability, 0 violations; store and Snapshot matrices re-run with unchanged digests | pass | Test evidence links this report |
| INT-0007 | AC-3, AC-4 | store matrix (heads, torn and stray checks) and the `fsck` corpus re-run: Sprint 2's figures | pass | as above |
| INT-0007 | AC-5 | backlog T-011 | not yet tested | stays open |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1: determinism | `materialise_matrix_is_reproducible` (40,960 cells, digest `blake3:6274344b…a1ac`); store `blake3:d36b8c9c…ae812` and Snapshot `blake3:c1e39cb1…13ea3` unchanged after T-003's new syscalls; equal on Windows and Linux; `simfs_runs_are_deterministic` extended | pass | Test evidence links this report |
| INT-0005 | AC-2: crash model and matrices | `every_syscall_has_no_effect_at_its_crash_point` over 13 kinds and 14 syscalls, with `remove_dir` and `set_executable`; the Materialise matrix (40 seeds, both modes, 0 violations) | pass | as above |
| INT-0005 | AC-3: negative controls | `materialise_catches_no_record`: 2,800 and 2,820 working-copy violations; every store and Snapshot control caught as before | pass | as above |
| INT-0005 | AC-4: randomness only through `sim` | `std::hash::RandomState` banned in every boundary crate and required by `fs-boundary`; transitive `getrandom`/`rand`/`uuid`/`fastrand` refused by `deps` (`policy_rejects_transitive_randomness`); the `RandomState` Clippy probe | pass | as above |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-1, AC-8, the policy rules | `cargo policy`, including `workspace/variants` as a test-only feature; `cargo deny check` on Windows | pass | Test evidence links this report |
| INT-0003 | AC-2 to AC-4 | local halves on both systems, plus the outside-target probe on both; `cargo deny` in WSL not run (cargo-deny not installed there; the GitHub Linux job runs it) | local halves pass; GitHub halves pending T-017 | as above |
| INT-0003 | AC-5 credits | `docs/prior-art.md`: jj's operation log and stale working copy, Microsoft's file-naming rules | credited; sign-off pending | as above |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md) | Sprint 2's GitHub runs; AC sign-offs | T-001 / [ci-sprint2.md](ci-sprint2.md): runs 42403413644, 42403409376 and 42404724911, every job success; sign-offs not given | recorded; sign-offs carried forward (T-017) | Test evidence links `ci-sprint2.md` (T-001) |

## Summary
- **Unit tests:** 407 passed on Windows and 408 on Linux, with 0 failed and 0 ignored. The extra Linux test needs a real ext4 symlink. Sprint 2 ended with 332.
- **Integration tests:** all pass on both systems:
  - the canonical runner;
  - the store, Snapshot and Materialise crash matrices with their controls, including the second-crash pass and the Windows-rules variant;
  - the `fsck` corpus;
  - the reversibility, hazard, edit and ignore corpora;
  - `cargo deny` on Windows;
  - five probes.
- **E2E tests:**
  - `praps_history_end_to_end` and `praps_restore_refusals_exit_one`, on NTFS and ext4;
  - `real_hazards_are_refused_on_ntfs` and `real_hazard_trees_materialise_on_ext4`;
  - Sprint 2's E2E tests;
  - `fresh_clone_ci` on WSL ext4.

  Not yet possible: power loss, Nighthawk, several processes or workspaces.
- **CI status:** Sprint 2's GitHub runs are recorded, all success. This sprint's runs start when `dev` is pushed at the checkpoint; backlog T-017 records them.

## CI Confirmation
- **Head SHA:**
  - The full matrix scripts ran at `ad2231956a00a9ac01c9014cfb5434ece706ed78` on both systems ([test-matrix-log.md](test-matrix-log.md)).
  - Every commit after it changes only the Book: results, critiques, this report, intent evidence and the backlog.
  - The C-001 recheck of `real_hazards` ran at `19ec3d6`, whose crates equal those of `ad22319`.
- **CI run:** none yet for this head. The checkpoint push triggers it.
- **Conclusion:** the local canonical runner succeeded on both systems.
- **Confirmations:**
  - `cargo ci` printed `PASS` for fmt, clippy, test, policy and book, then `ci: all 5 steps passed`, with exit 0. It did so in the Windows checkout (1,885 s) and in a fresh WSL ext4 clone (1,267 s).
  - `cargo deny check` on Windows reported advisories, bans, licenses and sources ok.

## Failures
None at the tested head. Before it, three reviews raised 20 findings, all fixed or answered:
- the Test-phase [code review](code-review.md), 6 findings, each with a regression test;
- the test critiques, [round 1](critique-round1.md) with 10 and [round 2](critique.md) with 4.

The most material:
- **CR-01:** after an interrupted write, resume could delete an ignored file at a temporary name, with no copy kept anywhere. On Windows a locked file was enough to trigger it, with no crash. Every temporary name is now checked before writing, and resume removes only files it placed.
- **CR-03:** an edit saved during a write could be overwritten. Each path is now re-read before it is removed or replaced. A small window between that read and the next syscall remains, and the spec states it.
- **CR-02, CR-04, CR-05:** control-character names went unseen, an entry at a temporary name stopped every later command, and a symlinked directory could be followed during resume.
- **Round-1 critique C-001:** the CLI's exit 1 for a hazard or a blocking entry had no test. It has one now.
- **Round-1 critique C-002:** two Windows matrix runs wrote into one log. Both legs were re-run in full, into new logs.

## Technical Debt Identified
- **INT-0004 AC-7, reporting:** `undo`, `restore` and `new` drop their snapshot's report of names with no UTF-8 form (backlog T-019). Such names are kept, but only `status` reports them.
- **INT-0004 AC-3, case-only renames:** these are refused on case-insensitive filesystems (backlog T-016). Name rules come from the platform, not from a probe, so case-sensitive NTFS directories are over-reported and WSL drvfs mounts are not protected (backlog T-015).
- **INT-0013, INT-0009:** `log` is quadratic in history length (backlog T-018).
- **INT-0014 AC-1, the CR-03 window:** a user edit saved between the re-read and the rename can still be lost; the mode is not re-read. Closing it would need an atomic compare-and-replace.
- **CR-05 on Windows:** NTFS junctions were not tested, only directory symlinks.
- **Carried forward:**
  - INT-0007 AC-5 and INT-0014 AC-3: several processes and workspaces (T-011);
  - INT-0006: the provisional object bodies (T-012);
  - INT-0014 AC-5: the stat cache;
  - the owner's sign-offs (T-017).
