# Sprint 2 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) | AC-1: auto-snapshot, 0 lost edits over a randomized edit-and-command corpus on Windows and Linux | T-005, T-006, T-007 / `edit_corpus_loses_nothing` (500 sequences, 6,000 snapshots, against an independent model, half with no executable bit); `real_edit_corpus_loses_nothing` (20 sequences on NTFS and on ext4); `snapshot_matrix_finds_no_violations` (recency and liveness over 25,360 crash cells); the T-005 unit tests (init, open, snapshot, `new`, `status`, several heads, vanished files, damaged ops); E2E `praps_end_to_end` | pass on both systems | Test evidence links this report |
| INT-0014 | AC-2: ignored paths never snapshotted; rules documented and tested | `ignore_rule_corpus` (47 paths in 18 cases agree with Git 2.56.0); nested rules, a whitelist, directory rules and `.gitignore` deletion in the edit corpus; `tracked_file_stays_tracked_when_ignored`, `tracked_file_under_ignored_dir_is_snapshotted`, `ignore_is_case_sensitive`, `always_ignored_names_ignore_case`, `ignore_strips_cr_before_an_escaped_space`; [`docs/spec/working-copy.md`](../../../spec/working-copy.md) | pass | as above |
| INT-0014 | AC-3 to AC-6 | not this sprint (several workspaces, API-only, the stat cache, conflicts) | not yet tested | stays open |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: suite on both systems, OS and toolchain named | `cargo ci` on Windows 11 Pro 10.0.26300 and in a fresh clone on Ubuntu 26.04.1 (WSL2): 332 tests each, rustc 1.99.1 | pass locally; GitHub half pending T-013 | Test evidence links this report |
| INT-0004 | AC-2: host-independent path encoding | `real_snapshot_id_is_host_independent` (`blake3:9ad9ad90…28c3` on NTFS and ext4); `RepoPath` validation tests; `real_fs_keeps_names_win32_would_rewrite` | pass | as above |
| INT-0004 | AC-7: names with no UTF-8 form reported and left alone | `real_scan_keeps_non_utf8_names`, `raw_display_never_equals_a_utf8_display`, `snapshot_reports_and_leaves_raw_names`, `raw_named_directories_are_reported_and_left_alone`, `reported_paths_are_escaped_in_full`; E2E `praps_status_reports_a_non_utf8_name` and `praps_status_reports_non_utf8_names_and_directories` (four names per platform, one a directory holding a file) | pass on both systems | as above |
| INT-0004 | AC-5 (Windows durability strategy) | unchanged; RealFs now passes verbatim paths, and the durability tests pass again | sign-off pending | unchanged |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1, AC-5 for `changeset`, `view` and the revised `operation` | T-002, T-003 / round-trip and re-encoding properties (10,000 each), the rejection tests, never-panic properties (100,000 each) | pass (formal ACs, checked by property-test proxies) | Test evidence links this report |
| INT-0006 | AC-3: golden files | `golden_v1_corpus_matches` (four new files, two operation files revised), `spec_examples_match_golden_files` (hex derived by script from the spec rules) | pass locally; GitHub half pending T-013 | as above |
| INT-0006 | AC-6: spec before or with code (provisional) | `format-v1.md` changed in the same commits as the encoders (T-002 `fc281d8`, T-003 `5e041da`); owner sign-off covers only Sprint 0's frozen sections | specified; sign-off pending | as above |
| [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) | AC-1: closure through references | `crash_matrix_finds_no_violations` (32,840 cells, both race kinds); `snapshot_matrix_finds_no_violations`; the reference, kind and re-sync unit tests; `unsynced_references_variant_is_caught` | pass | Test evidence links this report |
| INT-0007 | AC-2: acknowledged ops survive | durability in both matrices: 0 violations | pass | as above |
| INT-0007 | AC-3: derived heads, in-flight ops whole or absent | heads, torn and stray checks: 0 violations; all five controls caught | pass | as above |
| INT-0007 | AC-4: `fsck` 100% detection, 0 false reports | 207/207 truncations, 40/40 bit flips, 0 reports on 30,408 recovered stores; missing references of every kind; leftovers for unreachable objects | pass | as above |
| INT-0007 | AC-5: four writing processes | backlog T-011 | not yet tested | stays open |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-1: determinism | `matrix_runs_are_reproducible` (digest `blake3:d36b8c9c…ae812`), `snapshot_matrix_is_reproducible` (digest `blake3:c1e39cb1…13ea3`, per-cell state, trace and violations equal), equal on Windows and Linux | pass | Test evidence links this report |
| INT-0005 | AC-2: crash matrices, ≥ 40 seeds, both modes, 0 violations | both matrices; `every_syscall_has_no_effect_at_its_crash_point` with `scan` | pass | as above |
| INT-0005 | AC-3: negative controls | pointer-file, unsynced-init, unsynced-objects, write-in-place, the new unsynced-refs (store matrix), and unsynced-objects in the Snapshot matrix | pass | as above |
| INT-0005 | AC-4 and the I/O boundary | Clippy bans in every boundary crate (`workspace` included); probes: `std::fs` in `store/src`, `getrandom` in `sim/src`, `ignore`'s readers in `workspace/src`, each refused | pass | as above |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-1, AC-8 | `cargo policy` (`unsafe`, `names`, `fs-boundary`, `deps`, `credits`); `cargo deny check` | pass | Test evidence links this report |
| INT-0003 | AC-2 to AC-4 | the local halves: `cargo ci` on both systems, and the outside-target probe on both | local halves pass; GitHub halves pending T-013 | as above |
| INT-0003 | AC-5 credits | `docs/prior-art.md`: `getrandom`, `ignore`, the jj working copy, Bonsai changesets, Git's ignore semantics | credited; sign-off pending | as above |
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | AC-1 sign-offs | T-001 / [ci-sprint1.md](ci-sprint1.md); not given | carried forward (T-013) | unchanged |

## Summary
- **Unit tests:** 332 passed, 0 failed, 332 in total, on Windows and on Linux.
- **Integration tests:** all pass on both systems:
  - the canonical runner;
  - the store and Snapshot crash matrices, with controls;
  - the `fsck` corpus;
  - the edit and ignore corpora;
  - deny;
  - the four probes.
- **E2E tests:**
  - `praps_end_to_end`, `praps_status_reports_a_non_utf8_name` and `praps_status_reports_non_utf8_names_and_directories` on NTFS and ext4;
  - `fresh_clone_ci` on WSL ext4.

  Not yet possible: power-loss crashes, Nighthawk, several processes and workspaces, and the `status` time budget.
- **CI status:**
  - Sprint 1's GitHub Actions runs are recorded (all success).
  - This sprint's run happens when `dev` is pushed at the checkpoint.
  - The GitHub halves of INT-0003 AC-2 to AC-4, INT-0004 AC-1 and INT-0006 AC-3 are therefore pending, not met.

## CI Confirmation
- **Head SHA:**
  - The matrix and corpus evidence ran at `c5c6965249467a27ea8aecd21a30aec21b625458`, with the full matrix scripts on both systems ([test-matrix-log.md](test-matrix-log.md)).
  - The final head `ea927200efe954247a8188d86b690defb9b7092e` changes only tests and docs after it.
  - At `ea92720`, `cargo ci` ran on Windows, and the affected `praps` and `workspace` tests ran in a fresh WSL clone; all passed.
  - Later commits change only the Book.
- **CI run:** none yet for this head. The checkpoint push triggers it, and backlog T-013 records it.
- **Conclusion:** the local canonical runner succeeded on both systems.
- **Confirmations:**
  - `cargo ci` printed `PASS` for fmt, clippy, test, policy and book, then `ci: all 5 steps passed`, with exit 0: at `c5c6965` in the Windows checkout and in a fresh WSL ext4 clone, and at `ea92720` on Windows.
  - `cargo deny check` reported advisories, bans, licenses and sources ok.

## Failures
None at the tested head. Before it, three reviews raised 28 findings, all fixed or answered with a regression test, a mutation check or a recorded rationale:
- the Test-phase [code review](code-review.md) (15);
- the test critiques ([round 1](critique-round1.md), 9; [round 2](critique.md), 4).

The most material:
- On Windows, a name such as `foo.` or `aux.txt` was read as another file, so its content was silently replaced (CR-02). `RealFs` now passes verbatim paths.
- Without an executable bit, a restored file lost its parent's mode (CR-01).
- The edit-corpus oracle took "tracked before" from the implementation, so `new` could lose tracked files unnoticed (round 1, C-001). The oracle is now an independent model.
- A first snapshot was quadratic, and a file vanishing mid-scan aborted it (CR-04, CR-05).
- `init` did not always sync the root's parent (CR-06).
- Damaged op files were skipped silently (CR-10), and reference kinds were unchecked (CR-11).

Plan deviation: each `.gitignore` matcher is rooted at `.` and given directory-relative paths, not the directory's repository path ([code-review.md](code-review.md), Plan deviation).

## Technical Debt Identified
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-5 and [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-3: writers in separate processes and several workspaces, with a simulated process crash (backlog T-011). Until [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md)'s merge exists, commands stop on several heads.
- [INT-0006](../../../intents/INT-0006-format-v1.md): the provisional bodies still lack the operation's actor, time and signature; the changeset's author, intent references, renames and conflict terms; and the view's named pointers (backlog T-012, revised).
- INT-0014 AC-5: no stat cache, so every command reads and hashes every tracked file, and the store re-reads the blobs a changed changeset names. A changed large file is held whole in memory (INT-0008 AC-6).
- Code review CR-07 (part): `std::hash::RandomState` and transitive OS randomness (`rand`, `uuid`) are not banned. No boundary crate uses them today.
- Sign-offs carried forward (T-013): INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6 (Sprint 0's sections), INT-0004 AC-5. Also this checkpoint's GitHub Actions run.
- Symbolic links are reported, not tracked, until they are specified (mode `0x02` reserved).

## Coverage Observations
- INT-0006 AC-1 and AC-5 are `formal` criteria, checked here by their stated property-test proxies.
- The crash evidence is the simulator's. The negative controls and the reviewers' mutation runs show that the matrices, the edit corpus and the ignore corpus can fail.
- Each reviewer mutant is now caught, including the round-1 survivors.
- The simulator is deterministic across systems: both matrix digests are equal on Windows and Linux.
- The edit corpus's ignore oracle covers 11 names under four rules; the general matcher is covered by the corpus recorded from Git.
