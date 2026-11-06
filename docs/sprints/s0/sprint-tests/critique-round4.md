# Test Critique — Sprint 0, round 4

## Concerns

### C-001: Encoding probe's `FAIL test` comes from the Book violation
- **Where:** `integration-tests.md` `int_ci_reports_failures` (Intents line, Run step 3, Observed) / `test-matrix-log.md` both legs, and the probe steps of `test_matrix.sh` and `wsl_ci.sh` / `crates/xtask/src/ci.rs` `test` step / `crates/book/tests/book.rs` `book_intents_pass_strict` / `critique-round2.md` Responses, C-001 / `critique-round3.md` closing paragraph / `INT-0006` AC-3
- **Quote:** "AC-3: the suite fails when an encoding changes"
- **Failure mode:** weak-assertion
- **Why it matters:** Round 3 added the untyped criterion to this probe. Now `book_intents_pass_strict` fails first, and because the test step has no `--no-fail-fast`, cargo stops before codec's `tests/golden.rs` runs. Replays of `6a85e66` on Windows and in WSL ran only the three book test binaries and never mentioned golden. A mutant whose `golden_v1_corpus_matches` can never fail prints the same six logged lines and exits 1. So no run at the tested head shows the suite failing on a changed encoding, and no Linux run shows a golden failure at all. This undoes round 2's C-001 fix, and the record could support a false pass of AC-3's local half.
- **Suggested response:** tighten-assertion — move the Book violation to its own clone or `cargo ci` run, keep the byte flip with the fmt violation, and require each leg's log to show `golden_v1_corpus_matches ... FAILED` and `string-ascii.bin: encoding changed`, or drop the INT-0006 AC-3 link

### C-002: AC-3's GPL-3.0-compatible allowlist is checked by nothing
- **Where:** `INT-0003` Acceptance criteria AC-3 / `build-plan.md` T-001 Acceptance criterion ("AC-3 (deny configuration and a clean local run)") and its `cargo deny check` EARS clause / `test-plan.md` Intent Traceability, INT-0003 AC-3 row / `unit-tests.md` T-001 deny row / `integration-tests.md` `int_deny_at_t001`, `int_deny_at_t003`, `int_deny_windows` / `deny.toml` `[licenses] allow`
- **Quote:** "every dependency licence is on the GPL-3.0-compatible allowlist"
- **Failure mode:** intent-coverage
- **Why it matters:** T-001 claims AC-3 for its deny configuration, but its EARS clause and every record that cites AC-3 check only that `cargo deny check` reports `licenses ok`. That proves only that dependencies are on whatever list `allow` holds. If `SSPL-1.0` were added to `allow` along with a dependency under it, every Sprint 0 test and the `credits` rule would still pass. The nine identifiers are compatible by inspection, so nothing recorded at `6a85e66` is false. But the GPL-3.0-compatible part of a `necessary` AC rests on no test, review or sign-off, and the T-009 deny job would not catch it either.
- **Suggested response:** add-test — have xtask parse `deny.toml` and assert `[licenses].allow` is a subset of a hard-coded GPL-3.0-compatible SPDX list, with an `SSPL-1.0` negative case, and cite it under INT-0003 AC-3 beside the `int_deny_*` runs

### C-003: Records say INT-0002 makes detail keys case-exact; it does not
- **Where:** `critique-round3.md` Responses, C-001 / `code-review.md` fourth-round `crates/book/tests/book.rs` row ("INT-0002 now says keys match exactly") / `docs/intents/INT-0002-typed-acceptance-criteria.md` Grammar / `build-plan.md` T-006 Notes / `crates/book/tests/book.rs` `detail_keys_follow_each_modality`
- **Quote:** "INT-0002 now says keys match exactly, letter case included."
- **Failure mode:** evidence-drift
- **Why it matters:** No commit after `df87da1` (T-006) touches INT-0002, and `901dbc5` changes only `book.rs` and `cli.rs`. Neither INT-0002's Grammar nor the T-006 Notes mention letter case, so the rule exists only as a comment at `book.rs:123`. Two records therefore cite normative intent text that does not exist, and the owner's AC-1 review of the grammar (T-009) would not see a rule the validator enforces. The validator and the test are sound: a case-insensitive mutant fails `detail_keys_follow_each_modality`, and exact matching fails closed. No Sprint 0 AC is affected.
- **Suggested response:** tighten-assertion — reword both lines to say the case rule is the test's literal reading of the keys the T-006 Notes name, as `unit-tests.md` does, or amend INT-0002's Grammar with a Transition-history entry, as plan critique C-010 did

### C-004: Backlog task T-010 is cited but never defined
- **Where:** `code-review.md` known-limit row on xtask's `workspace_root()` and `CARGO_MANIFEST_DIR` / `critique-round3.md` Responses, C-005 / `e2e-tests.md` Carried forward / `docs/work/tasks.md`
- **Quote:** "Backlog task T-010 makes xtask refuse to run for another checkout"
- **Failure mode:** evidence-drift
- **Why it matters:** T-010 appears only on these two lines and has no intent tag. The Carried forward list defines only T-009, and `docs/work/tasks.md` has never listed T-010. The deferral of a known false-pass path therefore rests on a task nothing tracks: with a shared target directory, `cargo ci` can print `ci: all 5 steps passed` for another checkout. Plan critique C-011 found the same defect with `T-101`. No recorded result is affected, because every checkout in the matrix log builds into its own target directory.
- **Suggested response:** tighten-assertion — define `T-010 (backlog) [intent: INT-0003]` beside T-009 in `e2e-tests.md`'s Carried forward section, say Loop appends it, and align both citing lines with it

### C-005: `deps` row says the table is named; no test checks it
- **Where:** `unit-tests.md` T-001 `deps` row / `crates/xtask/src/policy.rs` `policy_flags_direct_dependency`, `policy_ignores_byte_order_marks`, `fails_exactly`
- **Quote:** "every form, including `[target.'cfg(...)'.*]` tables, fails naming crate, dependency and table"
- **Failure mode:** weak-assertion
- **Why it matters:** Every `deps` needle passed to `fails_exactly` is a crate or a dependency, never a table, and a mutant that prints `[dependencies]` for every form passes all 46 xtask tests. The EARS clause asks only for crate and dependency, and every form asserts both. So the row vouches for a check no test makes, but no clause is left unproved.
- **Suggested response:** tighten-assertion — drop "and table" from the row, or add each form's table header as a third needle (which needs a new tested head)

### C-006: CRLF count cannot see nine tracked files
- **Where:** `e2e-tests.md` `e2e_fresh_clone_windows`, Observed / `test-matrix-log.md` `test_matrix.sh`, the `text files with CRLF` line (`xargs file ... | grep -c CRLF`)
- **Quote:** "no tracked text file has CRLF line endings"
- **Failure mode:** evidence-drift
- **Why it matters:** For CRLF copies of the five crate roots, which start with `#![forbid(unsafe_code)]`, file 5.48 prints only "Rust Source file". For CRLF copies of the four member `Cargo.toml` files it prints an INI type. Neither label includes "with CRLF line terminators", so the logged 0 would stay 0 if those nine files were converted. `PASS fmt` (default `newline_style`) and the `unsafe` rule would not catch it either. The claim holds at `6a85e66`, where `git ls-files --eol` shows no `w/crlf` or `w/mixed` entry, and no AC or EARS clause depends on it.
- **Suggested response:** tighten-assertion — count `w/crlf` and `w/mixed` entries from `git ls-files --eol` instead of `file` labels

### C-007: Untested `claim` and scrub calls in `PrivateRepository::new`
- **Where:** `code-review.md` fourth-round `crates/xtask/src/attributes.rs` row (row 101) / `critique-round3.md` Responses, C-006 and C-010 / `crates/xtask/src/attributes.rs` `PrivateRepository::new`, `taken_names_are_never_used`, `every_private_repository_is_new_and_removed_after_use`, `caller_git_settings_are_shut_out`
- **Quote:** "nothing tested that a taken repository name is skipped, or that the rule's real call sites pass the inherited variable names"
- **Failure mode:** weak-assertion
- **Why it matters:** Two mutants of `new` pass all 46 xtask tests. One takes the first candidate name without calling `claim`, so `git init` reuses a planted directory whose `git/attributes` gives `probe.bin` `text: unset`. The other drops the scrub from the `git init` call. Row 101 says the call sites are tested, but `caller_git_settings_are_shut_out` covers only the `check_attr` call site.
- **Suggested response:** reject (the critique is wrong because neither mutant yields a false pass: skipping `claim` fails `cargo ci`'s clippy step as dead code, and the unscrubbed `git init` still fails closed under 13 hostile Git settings) — add no test; say in row 101 that the `git init` call site was checked by inspection

## Confidence
block

## Responses

Round 4 reviewed the records at `6db3cae`, which describe the tested head `6a85e66`. Each concern is addressed in the head that the final critique (`critique.md`) reviews. The code fixes are in `24d01f5`; mutating each fix makes a new or tightened test fail.

| Concern | Response |
|---------|----------|
| C-001 | tighten-assertion. The untyped criterion that round 3 added to `int_ci_reports_failures` made `cargo test` stop at the Book test before the golden test ran, as the critique found. It now has its own probe, `int_ci_reports_book_violation`, and `int_ci_reports_failures` is back to one changed golden byte and one formatting violation. On both systems that probe's log now shows the test step's own report: `golden_v1_corpus_matches ... FAILED` and `string-ascii.bin: encoding changed`. The INT-0006 AC-3 link rests on those lines. |
| C-002 | add-test in `24d01f5`. A test-only `deny` module reads `deny.toml` and checks every licence it allows, in the main list and in per-crate exceptions of either TOML form, against a hard-coded list of GPL-3.0-compatible licences: the Free Software Foundation's, plus `Apache-2.0` with the LLVM exception and `Unicode-3.0`, each with its reason. `incompatible_licences_are_named` shows that `SSPL-1.0`, `BUSL-1.1` and `OpenSSL` are named, and that an unreadable policy is an error. Mutants that ignore either form of exception, accept a missing list, never report, or put `SSPL-1.0` on the list fail it. The records cite it under INT-0003 AC-3 beside the `int_deny_*` runs. |
| C-003 | tighten-assertion. Both lines now say that the case rule is the test's literal reading of the keys the T-006 Notes name, and that the Loop phase adds it to INT-0002's Grammar with a Transition-history entry. |
| C-004 | tighten-assertion. The Carried forward section of `e2e-tests.md` now defines `T-010 (backlog) [intent: INT-0003]` beside T-009, and says the Loop phase appends both to `docs/work/tasks.md`. |
| C-005 | tighten-assertion in `24d01f5`. Every form in `policy_flags_direct_dependency` now also needs its table header, such as `[target.'cfg(unix)'.build-dependencies]`, in the problem. A mutant that prints `[dependencies]` for every form fails it. |
| C-006 | tighten-assertion. The fresh-clone check now counts the `w/crlf` and `w/mixed` entries of `git ls-files --eol` over every tracked file, instead of reading `file` labels. |
| C-007 | rejected, as the critique proposes. Row 101 of `code-review.md` now says that the `check_attr` call site is tested and the `git init` call site was checked by inspection: without the scrub it still fails closed, and skipping `claim` fails Clippy as dead code. |
