# Test Critique — Sprint 3

Round 1. A read-only subagent wrote it from the bundle's `prompts/test-critic.md`, against the result files as first drafted at tested head `70a2a4d`. It is saved as written, apart from formatting. Each concern's response is in the final [critique.md](critique.md).

## Concerns

### C-001: No test runs the CLI's hazard or blocking-entry exit 1
- **Where:** `build-plan.md` T-008 EARS; `unit-tests.md` T-008; `docs/work/completed-tasks.md` T-008 Evidence; `crates/praps/tests/cli.rs`
- **Quote:** "With an unknown or ambiguous OP, a hazard or a blocking entry, it **SHALL** exit 1 with the error on stderr." The T-008 evidence says "a hazard or a blocking entry exit 1 with the error on stderr".
- **Failure mode:** negative-path
- **Why it matters:** `praps_restore_unknown_or_ambiguous_fails` covers only the unknown and ambiguous cases. A grep for `hazard` or `block` in `cli.rs` and `e2e.rs` finds nothing. Neither the hazard nor the blocking half of this error path has run through the binary: no test checks the exit code, the stderr text, or that nothing was written.
- **Suggested response:** add-test. Restore over an ignored file standing at a path to write (`Blocked`), and on Windows a reserved name or a case variant (`Hazards`). Assert exit 1, the error on stderr, empty stdout, and the files and op count unchanged.

### C-002: Windows evidence for the Snapshot matrix is missing, and the overwrite also removed part of the Materialise section
- **Where:** `integration-tests.md`, "Store and Snapshot crash matrices, re-run" and "Materialise crash matrix"; scratchpad `s3_windows.log`, `s3_windows_snapshot.log`; `test-matrix-log.md`
- **Quote:** "That one test was run again on its own at the same head, and its output is logged as `int_snapshot_matrix_windows (rerun)`." and "Every count and the digest are identical on both systems."
- **Failure mode:** evidence-drift
- **Why it matters:**
  - The rerun had no output yet when the critic read it, so the Windows Snapshot claims were unproven.
  - The NUL block also erased the `==== int_materialise_matrix_windows` header and the start of the `NoRecord` Lenient line, so the Windows "2,800 in Lenient mode" had no evidence.
  - NUL padding is what a second writer leaves, so the lines after it might come from the stopped leg. Their counts do match the WSL run at the same head.
  - `test-matrix-log.md`, which all three result files cite, did not exist yet.
- **Suggested response:** add-test. Record the rerun's output. Rerun `-p workspace --test materialise_crash` on Windows into a fresh log. Correct the note on what was overwritten. Only then build `test-matrix-log.md`.

### C-003: `cargo deny check` was not run in WSL, though the test plan requires it
- **Where:** `test-plan.md`, Integration Tests, Canonical runner; `integration-tests.md`, Canonical runner table; `s3_wsl.sh`
- **Quote:** "`cargo ci` and `cargo deny check` on the Windows host and in WSL ext4."
- **Failure mode:** evidence-drift
- **Why it matters:** `s3_wsl.sh` never calls `cargo deny`, and the results list only `int_deny_windows`. The Linux half was skipped without a word.
- **Suggested response:** add-test, or defer-with-rationale and say so.

### C-004: The Materialise crash matrix only uses Unix filesystem semantics
- **Where:** `crates/workspace/tests/materialise_crash.rs` (`SimFs::new(seed, mode)`, with no `set_has_executable_bit` or `set_name_rules`); `integration-tests.md`, Materialise crash matrix
- **Quote:** "It crashes at every syscall, for 40 seeds, in both directory modes."
- **Failure mode:** EARS-coverage
- **Why it matters:** The bytes-only "matches F" rule with no executable bit, and the case-folded check of temporary names against blocking entries, are never crash-tested. INT-0014 AC-1 is promised on Windows and Linux.
- **Suggested response:** add-test (a matrix variant with no executable bit and the Windows name rules), or defer-with-rationale.

### C-005: The edit corpora never reach a hazard refusal, and the real corpora never reach a blocking one
- **Where:** `build-plan.md` T-009 EARS (edit corpus); `integration-tests.md`, Edit corpora table; `crates/workspace/tests/common/mod.rs` `Model::blocks`
- **Quote:** "Sequences whose step would be refused (a hazard or a blocking entry) **SHALL** assert the refusal and the unchanged state instead."
- **Failure mode:** EARS-coverage
- **Why it matters:** The model has no hazard prediction and `NAMES` has no case variants or reserved names. The logs show `16 refusals (0 blocked)` for the 500-sequence corpus and `3 refusals (0 blocked)` on both real filesystems; only the long `SimFs` corpus reaches 10 blocked. The table leaves out "(0 blocked)".
- **Suggested response:** tighten-assertion. Report the blocked count per corpus, and say that hazard refusals are covered by the hazard corpora.

### C-006: `ops_are_never_removed` ignores `undo` errors and checks the store, not `log`
- **Where:** `crates/workspace/tests/history.rs:472-490`; `integration-tests.md`, Reversibility corpora
- **Quote:** "`ops_are_never_removed`: random command sequences; every op seen stays in `log` (AC-1)." The code reads `2 => { let _ = ws.undo(); }` and checks `ws.store().ops()`.
- **Failure mode:** weak-assertion
- **Why it matters:** Any `undo` error is dropped, not just `NothingToUndo`. The superset check runs on the store's op files, not on `log`. The sequences use three names, with no executable files and no file↔directory swaps.
- **Suggested response:** tighten-assertion. Match `Ok(_) | Err(Error::NothingToUndo)`, and assert on `ws.log()` ids, or reword.

### C-007: CR-02's handling of names with no UTF-8 form is untested in materialise
- **Where:** `code-review.md` CR-02; `crates/workspace/src/materialise.rs:97-102`; `materialise.rs` `control_characters_in_names_are_seen`
- **Quote:** "A name with no UTF-8 form gets a NUL-prefixed key, which no repository path can equal."
- **Failure mode:** negative-path
- **Why it matters:** The only CR-02 test uses a tab, which is valid UTF-8. No test puts a name with no UTF-8 form on disk while a command writes the working copy, for example in a directory to be emptied. INT-0004 AC-7 promises such entries are never overwritten, deleted or skipped without a report.
- **Suggested response:** add-test, with byte 0xff on ext4 and an unpaired surrogate on NTFS, in a directory that `undo` would empty.

### C-008: CR-05's symlink guard is tested only on ext4, and not through resume
- **Where:** `code-review.md` CR-05; `real_hazards.rs` `real_symlinked_directories_are_not_followed` (`#[cfg(unix)]`)
- **Quote:** "The simulator does not follow links, so only a real filesystem can show this."
- **Failure mode:** EARS-coverage
- **Why it matters:** The test calls `materialise::plan` and `apply` directly, while the bug was in resume through `open`. No test covers a junction or symlink on NTFS.
- **Suggested response:** defer-with-rationale, or add a resume-path variant that writes a record and then calls `Workspace::open`.

### C-009: Test names, counts and claims in the result files do not match the code and logs
- **Where:** `unit-tests.md` table and "By build task"; `integration-tests.md`, Hazard corpora; `e2e-tests.md` step 4; `completed-tasks.md` T-005
- **Quote:** "`workspace` `tests/materialise.rs` (new) | 26 on Windows, 27 on Linux"; "the case-fold and Windows-name corpora skip names the host cannot write (a colon on Windows), and say so"
- **Failure mode:** evidence-drift
- **Why it matters:**
  - `materialise.rs` has 26 tests on both systems; the CR-05 test is in `real_hazards.rs`, so the table counted it twice.
  - `hazards_are_refused_before_any_write` is in `history.rs`, not `hazards.rs`.
  - "Nothing written" is credited to `case_fold_corpus` and `windows_name_corpus`, which only call `hazard::check`. The tests that write, `case_corpus_through_a_write` and `windows_names_through_a_write`, are not named.
  - `writable_here` drops names with `:` without printing anything.
  - E2E step 4: `log[1]` is the snapshot that `status` took, not one `undo` took.
  - The T-005 record says 53 hazardous names; `WINDOWS_HAZARDS` has 52.
- **Suggested response:** tighten-assertion (correct the result files).

### C-010: The CLI tests check little of what is printed
- **Where:** `crates/praps/tests/cli.rs` `praps_undo_restores_files`, `praps_prints_notes`; `build-plan.md` T-008 EARS
- **Quote:** "`praps undo [DIR]` … **SHALL** undo, print the op restored to, and exit 0."
- **Failure mode:** weak-assertion
- **Why it matters:** The printed op id is never compared with H's parent, and `praps_prints_notes` checks only `starts_with("note: ")`.
- **Suggested response:** tighten-assertion.

## Confidence
block
