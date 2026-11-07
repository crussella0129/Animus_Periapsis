# Test Critique — Sprint 2

Round 2, by a read-only subagent reviewer. Round 1 blocked with 9 concerns ([critique-round1.md](critique-round1.md)). The reviewer worked at tested head `c5c6965`, with the docs at `0dd1240`. It ran mutants in a throwaway worktree, which it then removed.

**Round-1 responses landed.** Each round-1 mutant that had survived is now caught:

| Mutant | Caught by |
|---|---|
| `new_change` drops tracked-but-ignored paths | `edit_corpus_loses_nothing` (the `new` assertion) |
| Other view heads dropped | `snapshot_keeps_other_heads` |
| The CR-09 fallback restored | `praps_fsck_reports_a_broken_repository_store` |
| The CR strip removed | `ignore_strips_cr_before_an_escaped_space` |
| The CR-01 revert | `edit_corpus_loses_nothing`, so the corpus now exercises the no-bit fallback |

Three new screening mutants were also caught:
- matching the shallowest `.gitignore` first;
- reading only the top `.gitignore`;
- entering an ignored directory fully.

**The oracle.** The `Model` is a faithful and independent statement of `docs/spec/working-copy.md`. Its nested-rule precedence matches Git's.

**The evidence** at `c5c6965` is consistent: 332 tests, and the same matrix, corpus and digest figures on both legs.

The concerns are condensed here; the full quotes are in the review transcript.

## Concerns

### C-001: The negative assertion in the CR-09 regression test can never fail
- **Where:** `crates/praps/tests/cli.rs`, `praps_fsck_reports_a_broken_repository_store`.
- **Quote:** `let named = format!("{} is not", root.display()); assert!(!err.contains(&named), …)`
- **Failure mode:** weak-assertion
- **Why it matters:** `praps` names the argument as given (`repo`), not the absolute path, so the check is dead. The mutant is caught only by the positive check.
- **Suggested response:** tighten-assertion.

### C-002: The C-005 docs claim more than the tests assert
- **Where:** `e2e-tests.md`; `crates/praps/tests/e2e.rs`; `raw_named_directories_are_reported_and_left_alone`.
- **Quote:** "every file keeps its content"; "its contents are neither tracked nor touched"
- **Failure mode:** evidence-drift
- **Why it matters:** The second raw file is never read back. The simulator's raw directories are empty, so "contents not touched" cannot fail there.
- **Suggested response:** tighten-assertion, and reword.

### C-003: The integration doc lists the GitHub-dependent criteria without the pending caveat that C-009 promised
- **Where:** `integration-tests.md`, "Canonical runner".
- **Quote:** INT-0006 AC-3: "the canonical suite (`cargo ci`, run locally and in the GitHub matrix)".
- **Failure mode:** evidence-drift
- **Why it matters:** Only `e2e-tests.md` says the GitHub run is pending.
- **Suggested response:** tighten the docs, and keep the report from marking these criteria met.

### C-004: The reason for keeping ignore rules out of the crash scenario is weaker than stated
- **Where:** the `crates/workspace/tests/crash.rs` header.
- **Quote:** "a crash may lose or tear one, since its writes are not synced"
- **Failure mode:** integration-drift
- **Why it matters:** A `.gitignore` synced in setup cannot be torn, so the stated reason is not the real one. The risk is low, because matching is a pure read and the corpora catch every ignore mutant tried.
- **Suggested response:** defer-with-rationale, restating the rationale.

## Confidence
proceed-with-caveats

## Responses

Every concern is addressed in `ea92720`. That commit changes only tests and docs. On it, `cargo ci` ran on Windows, and the `praps` and `workspace` CLI, E2E, workspace and edit-corpus tests ran in a fresh WSL clone.

| Concern | Response |
|---------|----------|
| C-001 | tighten-assertion. The test now asserts that stderr does not contain `repo: repo is not`, the bare-store fallback's message for the argument as given. It also asserts that stderr names the store as `.praps/store is not` (or `.praps\store` on Windows). |
| C-002 | tighten-assertion. The E2E test reads back the second raw file too. The simulator test's doc now says "reported and not entered", and names the E2E test as the evidence about a raw directory holding a file. |
| C-003 | `integration-tests.md` now says the canonical-runner rows are the local halves only. INT-0003 AC-2 to AC-4, INT-0004 AC-1 and INT-0006 AC-3 wait on the GitHub matrix (T-013). The test report does not mark those halves met. |
| C-004 | defer-with-rationale. The `crash.rs` header now states the real reason: ignore matching is a pure read of files the snapshot never writes, so a crash cannot change its answer, and the Git-recorded corpus and the edit corpora cover it. |
