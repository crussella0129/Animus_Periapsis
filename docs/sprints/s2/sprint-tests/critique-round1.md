# Test Critique — Sprint 2

Round 1, by a read-only subagent reviewer, at tested head `d248ccf`. The reviewer ran mutants in a throwaway worktree, which it removed. The concerns are condensed here; the full quotes are in the review transcript.

## Concerns

### C-001: The edit-corpus oracle takes "tracked before" from the implementation, so `new` can lose tracked files without any test failing
- **Where:** `integration-tests.md` "Edit corpora"; `crates/workspace/tests/common/mod.rs` `edit_sequence`; INT-0014 AC-1 and the T-005 clause on tracked files that become ignored.
- **Quote:** "`let tracked_before = ws.working_copy().unwrap().2;`" is read after `new_change`.
- **Failure mode:** weak-assertion
- **Why it matters:** A mutant in which `new_change` deletes tracked `*.tmp` paths survived every test: the oracle folds whatever `new` loses back into what it expects.
- **Suggested response:** tighten-assertion. Give the oracle an independent model of the tracked set, and assert that `new` keeps the tree.

### C-002: Without an executable bit, mode preservation is never exercised by the simulator corpus, despite the claim that it is
- **Where:** `integration-tests.md` "Edit corpora"; `code-review.md` CR-01 and CR-15; `tests/edits.rs`.
- **Quote:** "On NTFS no sequence can set an executable bit, so mode preservation there is covered by the simulator."
- **Failure mode:** evidence-drift
- **Why it matters:** The no-bit sequences never hold an executable file. A CR-01 revert is killed only by one unit test.
- **Suggested response:** add-test, or reword the claim.

### C-003: "No other head SHALL change" has no test
- **Where:** T-005, the clause on replacing a head; `snapshot_replaces_its_head`.
- **Quote:** "no other head **SHALL** change."
- **Failure mode:** EARS-coverage
- **Why it matters:** The mutant `view.heads = [id].into()` survived every test.
- **Suggested response:** add-test.

### C-004: The CR-09 regression test does not exercise the failure CR-09 describes
- **Where:** `code-review.md` CR-09; `praps_fsck_checks_a_working_copy`.
- **Quote:** "Covered by `praps_fsck_checks_a_working_copy`"
- **Failure mode:** negative-path
- **Why it matters:** Reintroducing the fallback on an open failure survived.
- **Suggested response:** add-test. Make `.praps/store` fail to open.

### C-005: INT-0004 AC-7 is tested for files only, with one name per platform
- **Where:** INT-0004 AC-7; `praps_status_reports_a_non_utf8_name`; `snapshot_reports_and_leaves_raw_names`.
- **Quote:** "A working-copy file **or directory** whose name has no UTF-8 form … 100% over a fixture corpus on each platform."
- **Failure mode:** intent-coverage
- **Why it matters:** No test covers a raw-named directory, and each platform has a single real name.
- **Suggested response:** add-test, or record AC-7 as partially met.

### C-006: Two Snapshot-matrix assertions are looser than their EARS clauses
- **Where:** `crates/workspace/tests/crash.rs`.
- **Quote:** the clause says "recovered state, trace **and outcome SHALL** be identical", and the control must "report durability **or closure** violations".
- **Failure mode:** weak-assertion
- **Why it matters:** Per-cell violations are not compared between runs, and the control's sum also counted recency.
- **Suggested response:** tighten-assertion.

### C-007: Ignore rules are covered statically but barely across edit sequences or under crashes
- **Where:** `common/mod.rs` `RULES`; the `crash.rs` header.
- **Quote:** "Ignore rules are covered by the corpus and edit tests."
- **Failure mode:** integration-drift
- **Why it matters:** Only two top-level rules are ever toggled: no nested file, no negation, and no deletion of `.gitignore` over time.
- **Suggested response:** add-test, or defer with a rationale.

### C-008: The ignore corpus cannot detect removal of the CR strip
- **Where:** `crates/workspace/src/ignore.rs`; corpus case `crlf`.
- **Quote:** `let line = line.strip_suffix('\r').unwrap_or(line);`
- **Failure mode:** weak-assertion
- **Why it matters:** Removing the strip survived. It differs only for an escaped trailing space before CR.
- **Suggested response:** add-test, or reject as an equivalent mutant.

### C-009: The GitHub halves of the evidence are missing for this head, including changed golden files
- **Where:** `e2e-tests.md` "Not yet possible"; INT-0006 AC-3; INT-0004 AC-1.
- **Quote:** "the canonical suite (`cargo ci`, run locally and in the GitHub matrix) fails on Windows and Linux whenever an encoding changes…"
- **Failure mode:** evidence-drift
- **Why it matters:** The operation golden files changed and four are new, but only the local runs exist.
- **Suggested response:** defer-with-rationale. Mark the GitHub halves pending T-013.

**Mutation record** (the reviewer's): the following mutants were killed:
- the CR-01, CR-05, CR-06, CR-10, CR-11 and CR-13 reverts;
- removing the BOM strip;
- breaking head replacement;
- snapshot dropping tracked files under ignored paths;
- `new` writing files;
- dropping reported-path escaping;
- removing the re-syncs;
- `status` ignoring mode.

The crash matrix is not vacuous: one mutant is killed only by it.

## Confidence
block

## Responses

Every concern is addressed in `c5c6965`, and the tests were re-run on both systems.

| Concern | Response |
|---------|----------|
| C-001 | tighten-assertion. `edit_sequence` keeps its own `Model` of the working-copy change's and the parent's paths and modes. The model starts empty, or from a setup snapshot it computes itself, and is never fed the workspace's state. After a `new`, the recorded tree must equal the model's tree before it. Both corpora pass. |
| C-002 | add-test. Every no-bit sequence (250 of 500) starts from a snapshot taken with the bit on, holding two executable files. Then the bit is switched off, so "keep the recorded mode" is exercised through edits, deletes and `new`s, against the model. |
| C-003 | add-test. `snapshot_keeps_other_heads` builds a view with an extra head, snapshots, and requires the extra head to be kept beside the new changeset. |
| C-004 | add-test. `praps_fsck_reports_a_broken_repository_store` removes `.praps/store/ops`. It requires exit 1, an error naming the `.praps` store, and no bare-store error about `repo`. |
| C-005 | add-test.<br>• `raw_named_directories_are_reported_and_left_alone` (SimFs, via the new `SimFs::make_raw_dir`) covers three raw forms: a leading invalid byte, a trailing invalid byte, and two surrogates.<br>• `praps_status_reports_non_utf8_names_and_directories` covers three raw names per platform: leading, trailing and truncated bytes on ext4, and a leading surrogate, two surrogates and a trailing surrogate on NTFS. One of them is a directory holding a file, which is never tracked or touched, while its sibling is snapshotted.<br>• The earlier single-name E2E test stays. |
| C-006 | tighten-assertion. `snapshot_matrix_is_reproducible` compares each cell's violations between the two runs. `snapshot_catches_unsynced_objects` requires closure or durability in strict mode; recency is no longer counted. |
| C-007 | add-test. The edit corpus's rules now include a nested `d2/.gitignore` with a whitelist (`!h.tmp`, re-including `d2/h.tmp` against the top `*.tmp`) and a directory rule (`e/`). A `.gitignore` that holds no active rule is deleted. The model applies Git's precedence independently. 500 and 20 sequences, 0 lost edits. Under crashes the rules stay out of the scenario: a torn `.gitignore` changes what a correct snapshot ignores, as the `crash.rs` header says. |
| C-008 | add-test. `ignore_strips_cr_before_an_escaped_space` (`x\ ` then CRLF ignores `x `, not `x`). Removing the strip fails it, checked by mutation. |
| C-009 | defer-with-rationale. The test report marks the GitHub-matrix halves (INT-0006 AC-3, INT-0003 AC-2 to AC-4, INT-0004 AC-1) as pending this head's checkpoint run, which backlog T-013 records. It does not mark them met. |
