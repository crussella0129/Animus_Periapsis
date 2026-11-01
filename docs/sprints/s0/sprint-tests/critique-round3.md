# Test Critique — Sprint 0, round 3

## Concerns

### C-001: Detail-key rule tested for one modality pair
- **Where:** `crates/book/tests/book.rs` `typed_line_violations_are_reported` / `crates/book/src/lib.rs` `Modality::detail_keys`, `allows_detail` / `build-plan.md` T-006 EARS (`detail`) and Notes (allowed keys per modality) / `unit-tests.md` T-006 row "a typed line breaking one rule" / `INT-0002` AC-2
- **Quote:** "`necessary` `empirical` — x\n  - Signers: owner"
- **Failure mode:** weak-assertion
- **Why it matters:** The only disallowed (modality, key) pair that any test tries is `Signers` under `empirical`. Three mutants survive: one checks keys only for `empirical`, one accepts keys that no modality names (a typo like `Treshold:`), and one drops `Rubric` from `judged`. Each passes all 35 book tests, `book_intents_pass_strict` included. The validator is the gate for INT-0002 AC-2, so the recorded `detail` pass would stand for a validator that reports 0 violations on mistyped criteria.
- **Suggested response:** tighten-assertion — add a table-driven test over every modality and key, hard-coded from the T-006 Notes plus a key no modality names; each pair must land in `details` or give exactly one `detail` violation

### C-002: `cargo policy` is never run with a failing rule
- **Where:** `build-plan.md` T-005 EARS (`golden`); the T-001 rule clauses and the T-002 `spec` clause have the same gap / `crates/xtask/src/main.rs` `cli` policy arm, test helper `run`, `policy_command_reports_every_rule` / `unit-tests.md` T-005 golden-rule row / `integration-tests.md` `int_tooling_is_idempotent`
- **Quote:** "`cargo policy` **SHALL** fail with the new rule `golden`."
- **Failure mode:** negative-path
- **Why it matters:** Every negative rule case calls `policy::check` directly, and the only `cli` policy test runs on the clean workspace. So the arm's `FAIL` lines and its exit 1 never run, and no record shows `cargo policy` exiting 1. A mutant that always exits 0 passes all 39 xtask tests, and it exits 0 on a workspace whose `.gitattributes` leaves `*.bin` unmarked. The command-level half of the clause is unproved, and the record would read the same for that regression. `cargo ci` runs policy in-process and that path does have a tested failure.
- **Suggested response:** add-test — call `cli(["policy"], ...)` on a fixture whose `.gitattributes` is `* text=auto eol=lf`, and assert exit 1, one `FAIL golden` line, ten `PASS` lines and empty stderr

### C-003: Envelope kinds and versions sampled at a few ids
- **Where:** `crates/codec/tests/properties.rs` `envelope_rejects_unknown_version_and_kind` / `build-plan.md` T-004 EARS (envelope version and kind errors) / `unit-tests.md` T-004 envelope row / `INT-0006` Intent, Object identity (a reserved kind gets its own error, distinct from an unassigned kind)
- **Quote:** "for kind in [0x00u8, 0x06, 0xff] {"
- **Failure mode:** weak-assertion
- **Why it matters:** Only 3 of the 251 kinds that the clause assigns to `UnknownObjectKind` are ever decoded, and only 2 of the 255 rejected versions. The other envelope tests only tell `Ok` from `Err`. Making `from_id` return `ReservedObjectKind` for 0x10..=0xfe leaves all 54 codec tests passing. The T-004 row's pass would stand for a decoder that blurs the split between reserved and unknown kinds that INT-0006 keeps distinct.
- **Suggested response:** tighten-assertion — sweep kind and version over `0..=u8::MAX`, with a hard-coded expected result for every id

### C-004: Printed violation's file, line and rule unchecked
- **Where:** `crates/book/tests/cli.rs` `cli_exit_codes` / `crates/book/src/lib.rs` `Kind::name`, `impl fmt::Display for Violation` / `build-plan.md` T-006 EARS 2 / `unit-tests.md` T-006 row "a typed line breaking one rule" / `critique-round1.md` C-011
- **Quote:** "assert!(String::from_utf8_lossy(&output.stdout).contains(": untyped: "));"
- **Failure mode:** weak-assertion
- **Why it matters:** That substring is the only check on printed violations. A mutant that swaps `polarity` and `modality`, prints `duplicate` for `number` and drops `{path}:{line}` passes all 35 book tests. Yet `cargo book` is the only place an author sees which rule broke and where, because `cargo ci` prints only a count. The T-006 row's "file, line and rule" pass would stand. Round 1's C-011 asked for this CLI assertion, but the response tightened only the library side.
- **Suggested response:** tighten-assertion — assert `INT-0001-x.md:4: untyped: ` in `cli_exit_codes`, and render one violation per `Kind`, checking each line starts with `{path}:{line}: {rule}: `

### C-005: Shared target dir makes `cargo ci` check another checkout
- **Where:** `code-review.md` known-limit row "tests that use `env!("CARGO_MANIFEST_DIR")`" / `crates/xtask/src/main.rs` `workspace_root` / `crates/xtask/src/ci.rs` `run`, `cargo_command` / `test-matrix-log.md` `wsl_ci.sh` and `test_matrix.sh` (`PROBE_TARGET`) / `integration-tests.md` Run steps of `int_ci_reports_failures`, `int_ci_ignores_hostile_environment`, `int_disallowed_types_probe`
- **Quote:** "Two checkouts sharing one `CARGO_TARGET_DIR` can run a test binary built for the other checkout."
- **Failure mode:** flake-risk
- **Why it matters:** xtask's `workspace_root()` is also fixed at compile time. When Cargo reuses the xtask in a shared target dir, `cargo ci` and `cargo policy` check the checkout that built it. At ac0e010, a broken checkout printed `ci: all 5 steps passed` and exited 0. The recorded runs still stand, because each probe rebuilt xtask for its own path. However, the row understates the scope, and integration-tests.md never says the probes shared target dirs, although the row's own disposition says not to share one.
- **Suggested response:** defer-with-rationale — restate the row to cover `cargo ci` and `cargo policy`, record each probe's target dir, and put a run-time workspace root in the backlog

### C-006: Git-variable scrub tested only by name
- **Where:** `crates/xtask/src/attributes.rs` `isolated`, `every_git_variable_is_scrubbed`, `check_attr_runs_at_the_workspace_and_isolated` / `critique-round2.md` Responses, C-002 / commit `7f9a63c` message / `unit-tests.md` T-005 golden-rule row / `build-plan.md` T-005 EARS (`golden`)
- **Quote:** "with the caller's Git settings and the machine's global and system attributes shut out"
- **Failure mode:** weak-assertion
- **Why it matters:** `every_git_variable_is_scrubbed` checks only the `is_git_variable` name test, and no test asserts that `isolated()` removes a variable. Replacing `env_remove(name)` with `drop(name)` still passes the 39 xtask tests, clippy and fmt. With that mutant, a caller's `GIT_CONFIG_PARAMETERS` setting `core.attributesfile` turns an unmarked `*.bin` into `PASS golden`. The C-002 claim holds at the tested head, so no recorded PASS is false. But the test name, the `7f9a63c` message and the T-005 row all cite a check that no test makes.
- **Suggested response:** add-test — have `isolated` take the inherited variable names and assert that `get_envs()` holds `(name, None)` for each `GIT_*` one, or reword the T-005 row and the C-002 response to say the scrub was checked by inspection

### C-007: Verifier runs still cited with no head
- **Where:** `critique-round2.md` Responses, C-005 / `e2e-tests.md` `e2e_port_rule_local_books` Oracle line / `code-review.md` Method step 4, and the "over two rounds" line above the fix-verification issues / `test-plan.md` recording rule
- **Quote:** "A code-review verifier also compared every criterion's line, claim and details against a markdown-it-py model, with no differences."
- **Failure mode:** evidence-drift
- **Why it matters:** The C-005 response says the verifier snapshot is no longer cited. Yet this line and Method step 4 still report verifier runs with no head, host or checkout, and neither run is in test-matrix-log.md. Step 4's whole-workspace run with `PROPTEST_CASES=1 PERI_GOLDEN_CREATE=1` was the second round's snapshot of 4a2c07a plus uncommitted fixes, but the text now credits it to the third round. The markdown-it-py comparison ran on Book code older than 40a37c6.
- **Suggested response:** tighten-assertion — give each run its head and round, or drop or relabel it; also fix "over two rounds" and reword the C-005 response

### C-008: code-review.md gives 134 tests; the tested head ran 140
- **Where:** `code-review.md` Issues raised while verifying the fixes, last row / `unit-tests.md` Result line / `integration-tests.md` `int_ci_windows`, `int_ci_wsl` / `test-matrix-log.md` ("tests passed: 140", "tests passed on Linux: 140")
- **Quote:** "the Sprint 0 runs list the same 134 tests on both systems"
- **Failure mode:** evidence-drift
- **Why it matters:** The other records and the raw log give 140 tests on both systems at ac0e010. 134 was the count at 4e25bfc and 0b8c498, before six tests were added. So a row committed after the tested head contradicts the results, on the very point it offers in place of an automated Windows/Linux comparison. It understates the suite rather than hiding a failure, and no AC rests on it.
- **Suggested response:** tighten-assertion — change 134 to 140 at ac0e010 and cite test-matrix-log.md lines 65 and 224

### C-009: Per-chapter awk counts are not in the raw log
- **Where:** `e2e-tests.md` `e2e_port_rule_sprint_loops_book` Independent count table / `test-matrix-log.md` e2e_port_rule_sprint_loops_book section / the `test_matrix.sh` awk loop reproduced there
- **Quote:** "gives the same number for every chapter"
- **Failure mode:** evidence-drift
- **Why it matters:** The record names test-matrix-log.md as its raw output, but the log holds only the awk total (82), because the loop adds up the per-file counts without printing them. So the table's 13 per-chapter awk values come from no logged run. They match a recomputation at e8c3a58, and the plan's pass condition is the logged total, so only the provenance is missing.
- **Suggested response:** tighten-assertion — print each chapter's awk count inside the loop, or label the column as recomputed outside the logged run

### C-010: Private-repository race and unchecked tamper paths
- **Where:** `code-review.md` Issues raised while verifying the fixes, third-round private-repository row ("fixed in `7f9a63c`") / `crates/xtask/src/attributes.rs` `private_repository`, `is_pristine` / `critique-round2.md` Responses, C-002
- **Quote:** "Third round: two processes creating the private repository at once could collide, and an existing one was trusted without checks."
- **Failure mode:** flake-risk
- **Why it matters:** Row 79 calls this fixed, yet `private_repository` removes whatever is at `attributes.git` without checking again. In repeated trials, 4 to 6 of 90 concurrent first runs printed a spurious `FAIL golden`. Also, `is_pristine` accepts a planted `commondir` that made the rule pass a workspace with `* text=auto eol=lf`. No test covers creation, replacement or `is_pristine`.
- **Suggested response:** reject (the critique is wrong because the race fails closed and tampering needs write access to cargo's target dir, which no AC or EARS clause covers) — add no test; restate row 79 as a known limit

## Confidence
block

## Responses

Round 3 reviewed the records at `4134ae8`, which describe the tested head `ac0e010`. Each concern is addressed in the head that the final critique (`critique.md`) reviews. The fixes are in `901dbc5` (Book tests), `e882da9` (envelope tests) and `6a85e66` (the golden rule and the xtask tests). An independent verifier mutated each fix, and every mutant named below fails the new tests.

| Concern | Response |
|---------|----------|
| C-001 | tighten-assertion in `901dbc5`. `detail_keys_follow_each_modality` tries all five modalities with the eleven keys the T-006 Notes name and four near misses (`Treshold`, `Signer`, `Notes`, `threshold`): 75 pairs. The allowed keys are written out from the Notes, not read from the code. An allowed pair must give no violation and exactly that detail; any other pair exactly one `detail` violation at its line. Mutants that check keys for one modality only, accept keys no modality names, drop `Rubric`, widen `formal` or ignore letter case all fail it. The case rule is the test's literal reading of the keys the T-006 Notes name; the Loop phase adds it to INT-0002's Grammar (corrected after the round-4 critique, C-003). |
| C-002 | add-test in `6a85e66`. `policy_command_fails_naming_each_failing_rule` runs the `cargo policy` front end on failing fixtures and compares the whole output: exit 1, one line per rule in order, nothing on stderr, a `FAIL golden` line, a second failing rule in its own place, and a `FAIL` line for each of one rule's two problems. Mutants that exit 0, drop the `FAIL` lines or print only a rule's first problem fail it. `int_policy_reports_failures` runs the real `cargo policy` on Windows and in WSL, in a clone whose `.gitattributes` marks nothing: `FAIL golden` and exit 1. |
| C-003 | tighten-assertion in `e882da9`. `envelope_rejects_unknown_version_and_kind` decodes all 256 versions and all 256 kinds, each with an empty and a one-byte body, against the spec's version rule and kind table written out by hand, and checks `ObjectKind::from_id` for every id. Class counts (1 defined, 4 reserved and 251 unknown kinds; 1 supported and 255 unsupported versions) fail a sweep that skips ids. Mutants that reserve 0x10 to 0xfe, do so only when a body follows, shorten the reserved range, accept version 0x80 or report the wrong id fail it. |
| C-004 | tighten-assertion in `901dbc5`. `printed_violations_name_file_line_and_rule` produces one violation of each of the eight rules and checks that each prints `path:line: rule: `, with the rule word and the line written out per case. `cli_exit_codes` requires exactly one stdout line that starts with the violating chapter's full path and `:4: untyped: `. Swapped or wrong rule names, a dropped path and an off-by-one line fail them, on Windows and on Linux. |
| C-005 | defer-with-rationale. The code-review row now covers xtask: its workspace root is fixed when it is built, so with a shared target directory `cargo ci` and `cargo policy` can check the checkout they were built for. Every checkout in the final matrix builds into its own target directory, and the log prints each one. Backlog task T-010 makes xtask, and the tests that read workspace files, refuse to run for another checkout (widened after the round-5 critique, C-006, and redefined after round 6, C-008: `cargo ci` refuses a target directory outside the workspace root). |
| C-006 | add-test in `6a85e66`. `isolated` now takes the inherited variable names, and `isolated_removes_every_inherited_git_variable` checks that each `GIT_*` name, in any letter case, is removed while other variables stay. `caller_git_settings_are_shut_out` runs the rule in a child process whose `GIT_CONFIG_PARAMETERS` and `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_0` and `GIT_CONFIG_VALUE_0` point `core.attributesFile` at a file that marks `*.bin` binary. The rule still finds the file unmarked, while a plain `git` given the same variables reports it binary. Removing the scrub, or the call site's use of it, fails these tests. `int_policy_reports_failures` repeats the hostile-variable run against the real `cargo policy` on both systems. |
| C-007 | tighten-assertion. The sentence about a markdown-it-py comparison of every criterion is dropped from `e2e-tests.md`: that comparison ran on Book code older than `40a37c6`, so it is no evidence for the tested head. Method step 4 of `code-review.md` now gives the whole-workspace run with `PROPTEST_CASES=1` and `PERI_GOLDEN_CREATE=1` to the second verifier round, on `4a2c07a` plus fixes not yet committed, and says `int_ci_ignores_hostile_environment` repeats it at the tested head. The table's "over two rounds" is gone. With these changes round 2's C-005 response holds. |
| C-008 | tighten-assertion. The row now gives the count at the tested head, 149 on both systems, as the matrix log records it. |
| C-009 | tighten-assertion. `test_matrix.sh` now prints each chapter's `awk` count, so every value in the per-chapter table comes from the logged run. |
| C-010 | fixed in `6a85e66` rather than rejected. Each evaluation claims a new directory under the temp directory, makes its own bare repository there and removes it afterwards, so no process shares or reuses a repository and nothing can be planted in one. `taken_names_are_never_used` checks that a taken name is skipped and left as it was; `every_private_repository_is_new_and_removed_after_use` checks a fresh repository per evaluation and its removal. |

The concern the verifiers refuted, that `cargo ci`'s Book step was never shown failing on a violation, gets a test anyway: `book_step_fails_on_a_violation`. `int_ci_reports_failures` now also plants an untyped criterion, so its runs must print `FAIL book`. (Superseded by round 4's C-001: the untyped criterion moved to its own probe, `int_ci_reports_book_violation`.)
