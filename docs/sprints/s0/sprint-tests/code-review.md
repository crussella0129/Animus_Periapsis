# Sprint 0 Code Review

Test-phase review of the Sprint 0 build at `40f6afb`, run before the test reports were written.

- **Method:**
  1. Five read-only reviewers each read the code through one lens: `hash`, `codec`, `book`, `xtask`, and the tests themselves.
  2. A skeptic then re-checked every finding and tried to refute it, mostly by mutating scratch copies of the code and running the real tests against them.
  3. Each fix was verified by an independent agent the same way: replay the finding's scenario, then revert the fix and show that a test fails.
  4. A second round verified the fixes made in response to the first round's verifiers. It also ran the whole workspace on Windows and WSL with `PROPTEST_CASES=1` and `PERI_GOLDEN_CREATE=1` set, on `4a2c07a` plus fixes not yet committed: every step passed, and no golden file was created or changed. That run is not evidence for the tested head; `int_ci_ignores_hostile_environment` repeats it there.
  5. A third round verified the commits made for the round-2 test critique, and a fourth those made for the round-3 critique, each by mutating the fix and showing that a new test fails. The round-4 and round-5 fixes were checked the same way.
- **Outcome:** 45 findings raised, 43 confirmed and fixed, 2 refuted. Every confirmed finding has a regression test that fails on the reviewed code.
- **Fix commits:** dependencies `5f9efaa` (`pulldown-cmark`, `toml_edit`, `allocation-counter`, credited in [prior-art](../../../prior-art.md)); `hash` `406ed63`; `codec` `4a2c07a`; `xtask` `9672c26`; `book` `40a37c6`; test-cleanup hygiene `4e25bfc`. After the test critiques and their verification: golden rule `0b8c498` and `7f9a63c`, golden tests `76ff8a4` and `ac0e010`; then Book tests `901dbc5`, envelope tests `e882da9`, and the golden rule's per-run repository with the xtask tests `6a85e66`; then the licence-allowlist and `deps` tests `24d01f5`; then the hash text form, UTF-8 offset, deny-job and licence-match tests `94f7363`.
- **Plan deviation:** the T-006 build-plan note said the Book parser would be "line-based and std-only". The review showed that a line parser reads Markdown differently from GitHub (findings book-1 to book-10), so the validator now uses the pulldown-cmark CommonMark parser. Every Book on this host now reads exactly as CommonMark counts it (`e2e_port_rule_local_books`).

## Findings

| ID | Severity | Where | Finding | Disposition | Regression test |
|----|----------|-------|---------|-------------|-----------------|
| hash-1 | low | `crates/hash/tests/vectors.rs` | The only odd-length case in `text_rejects_malformed` was rejected by the length check, so a parser that dropped an odd digit, and gave one value two text forms, would pass. | fixed `406ed63` | `text_rejects_malformed` (65 digits, exact errors) |
| hash-2 | low | `crates/hash/tests/vectors.rs` | No case had an uppercase name with valid hex, so a case-insensitive name lookup would pass. | fixed `406ed63` | `text_rejects_malformed` (`BLAKE3:`, `Blake3:`) |
| codec-1 | medium | `crates/codec/src/wire.rs` | `Vec` decoding reserved `count × size_of::<T>()` before reading an element: 33× the input for `Vec<HashRef>`. A large hostile input aborted the process (format v1, Total). | fixed `4a2c07a` | `hostile_hash_count_reserves_nothing`, `hostile_pair_count_reserves_nothing` |
| codec-2 | low | `crates/codec/src/wire.rs` | A cut-off varint was reported at the missing byte, not where the value starts (Located). | fixed `4a2c07a` | `truncated_varint_located_at_its_start` |
| codec-3 | low | `crates/codec/src/wire.rs` | `UnexpectedEnd` had two meanings, so its message stated false offsets and counts. | fixed `4a2c07a` | `overrun_reports_bytes_still_needed` |
| codec-4 | refuted | `crates/codec/src/envelope.rs` | Envelope errors carry no offset; a one-byte bad version reports truncation. Refuted: the locked test plan requires truncation for a one-byte input, and header fields sit at fixed offsets. The overclaiming module doc and the spec's Located bullet were corrected anyway (`4a2c07a`). | doc fixed | — |
| book-1 | high | `crates/book/src/lib.rs` | Fences ignored length and info strings: an inner ```` ``` ```` closed a ````` ```` ````` fence, so fenced lines were checked and real criteria hidden. | fixed `40a37c6` | `long_and_tagged_fences_follow_commonmark` |
| book-2 | medium | `crates/book/src/lib.rs` | A fence left open inside a list item hid every later criterion. | fixed `40a37c6` | `unclosed_fence_ends_with_its_item` |
| book-3 | medium | `crates/book/src/lib.rs` | The port rule rejected labelled but untyped items (`**AC-1:** …`), contrary to INT-0002; two real Books failed. | fixed `40a37c6` | `undeclared_book_lifts_untagged_labels`, `undeclared_labels_are_lifted` |
| book-4 | medium | `crates/book/src/lib.rs` | A wrapped detail value was spliced into the claim. | fixed `40a37c6` | `wrapped_details_keep_their_lines` |
| book-5 | high | `crates/book/src/lib.rs` | Ordered, `*` and `+` lists, lazy lines and indented items were merged, split or dropped. On this host's other Books, 696 criteria read as 245. | fixed `40a37c6` | `every_list_form_holds_criteria` |
| book-6 | medium | `crates/book/src/lib.rs` | HTML comments, thematic breaks and lead-in prose were lifted as criteria. | fixed `40a37c6` | `comments_breaks_and_prose_are_not_criteria` |
| book-7 | low | `crates/book/src/lib.rs` | `AC-+1` and `AC-02` passed the `number` rule. | fixed `40a37c6` | `numbers_are_plain_digits` |
| book-8 | low | `crates/book/src/lib.rs` | With a repeated `typed-acceptance:` line, the first one won, so a declared `2` could be ignored. | fixed `40a37c6` | `repeated_declaration_is_an_error`, `repeated_declaration_exits_2` |
| book-9 | low | `crates/book/src/lib.rs` | Valid ATX variants and setext headings were not recognised. | fixed `40a37c6` | `headings_follow_commonmark` |
| book-10 | low | `crates/book/src/lib.rs` | A fence line inside an indented code block opened a fence. | fixed `40a37c6` | `indented_code_opens_no_fence` |
| book-11 | low | `crates/book/src/main.rs` | A non-Unicode argument panicked (exit 101). | fixed `40a37c6` | `non_unicode_argument_is_a_book_path` |
| xtask-1 | medium | `crates/xtask/src/policy.rs` | `deps` skipped target-specific dependency tables. | fixed `9672c26` | `policy_flags_direct_dependency` |
| xtask-2 | medium | `crates/xtask/src/policy.rs` | `credits` missed `[workspace.dependencies.<name>]` tables, so `blake3` could drop `pure` unchecked. | fixed `9672c26` | `policy_flags_uncredited_dependency`, `policy_requires_complete_dependency_rows` |
| xtask-3 | medium | `crates/xtask/src/policy.rs` | `clippy` was a substring search: commented-out entries, other keys and shadowing configs passed. | fixed `9672c26` | `policy_requires_disallowed_hash_collections` |
| xtask-4 | low | `crates/xtask/src/policy.rs` | `golden` ignored overriding `.gitattributes` lines and nested files. | fixed `9672c26` (Git decides) | `policy_requires_binary_golden_attr` |
| xtask-5 | low | `crates/xtask/src/policy.rs` | A C-build cell starting with "no", such as "not verified", counted as `no`. | fixed `9672c26` | `policy_requires_complete_dependency_rows` |
| xtask-6 | low | `crates/xtask/src/policy.rs` | Nothing checked that the root sets `unsafe_code = "forbid"`. | fixed `9672c26` | `policy_flags_missing_workspace_lints` |
| xtask-7 | low | `crates/xtask/src/policy.rs` | Members were hard-coded as `crates/*`. | fixed `9672c26` | `policy_finds_members_like_cargo` |
| xtask-8 | low | `crates/xtask/src/main.rs` | A non-Unicode argument panicked (exit 101). | fixed `9672c26` | `non_unicode_arguments_are_usage_errors` |
| xtask-9 | low | `crates/xtask/src/ci.rs` | The workflow test matched substrings anywhere, not per job. | fixed `9672c26` | `workflow_matches_ci_contract` |
| xtask-10 | low | `crates/xtask/src/docs.rs` | "Git" was satisfied by "JGit". | fixed `9672c26` | `credited_names_are_whole_words` |
| xtask-11 | low | `crates/xtask/src/ci.rs` | The real step executor was never run by a test. | fixed `9672c26` | `steps_execute_for_real` |
| xtask-12 | low | `crates/xtask/src/policy.rs` | Valid inheritance spellings failed falsely. | fixed `9672c26` | `policy_reads_manifests_as_toml` |
| xtask-13 | low | `crates/xtask/src/policy.rs` | Multi-line arrays and `path="…"` were misread. | fixed `9672c26` | `policy_reads_manifests_as_toml` |
| xtask-14 | low | `crates/xtask/src/policy.rs` | A UTF-8 byte-order mark defeated four rules. | fixed `9672c26` | `policy_ignores_byte_order_marks` |
| tests-1 | low | `crates/xtask/src/ci.rs` | The step test did not pin the exact commands. | fixed `9672c26` | `ci_steps_are_ordered_and_locked` |
| tests-2 | medium | `crates/xtask/src/ci.rs` | The runner test replaced the executor with scripted outcomes. | fixed `9672c26` | `steps_execute_for_real` |
| tests-3 | low | `crates/xtask/src/ci.rs` | Same root cause as xtask-9. | fixed `9672c26` | `workflow_matches_ci_contract` |
| tests-4 | medium | `crates/xtask/src/policy.rs` | Same root cause as xtask-3. | fixed `9672c26` | `policy_requires_disallowed_hash_collections` |
| tests-5 | medium | `crates/xtask/src/policy.rs` | Same root cause as xtask-1. | fixed `9672c26` | `policy_flags_direct_dependency` |
| tests-6 | medium | `crates/xtask/src/policy.rs` | Same root cause as xtask-2. | fixed `9672c26` | `policy_flags_uncredited_dependency` |
| tests-7 | low | `crates/xtask/src/policy.rs` | Same root cause as xtask-4. | fixed `9672c26` | `policy_requires_binary_golden_attr` |
| tests-8 | low | `crates/xtask/src/policy.rs` | No tests for a workspace `publish = true`, `lib.rs` without the attribute, or an empty hash table. | fixed `9672c26` | `policy_requires_unpublished_members`, `policy_flags_missing_forbid_unsafe`, `policy_requires_format_spec` |
| tests-9 | medium | `crates/book/src/lib.rs` | Same root cause as book-1. | fixed `40a37c6` | `long_and_tagged_fences_follow_commonmark` |
| tests-10 | low | `crates/book/src/lib.rs` | Same as book-7. | fixed `40a37c6` | `numbers_are_plain_digits` |
| tests-11 | refuted | `crates/book/tests/book.rs` | The "never both" assertion was a tautology. Refuted as a gap, because the return type rules out "both"; the line was removed and the test's comment now explains the guarantee. | clarified `40a37c6` | — |
| tests-12 | low | `crates/book/tests/book.rs` | No test checked which file a violation names. | fixed `40a37c6` | every violation test (via `kinds()`) |
| tests-13 | low | `crates/xtask/src/docs.rs` | Same as xtask-10. | fixed `9672c26` | `credited_names_are_whole_words` |
| tests-14 | medium | `crates/codec/src/wire.rs` | Same as codec-2. | fixed `4a2c07a` | `truncated_varint_located_at_its_start` |

## Issues raised while verifying the fixes

The fix verifiers raised these issues. Each is fixed, accepted, or recorded as a known limit.

| Severity | Where | Issue | Disposition |
|----------|-------|-------|-------------|
| medium | `crates/book/src/lib.rs` | The new recursive reader overflowed the stack at about 850 nested list levels on Windows. | fixed `40a37c6`: depth is capped at details and 8 nested quotes. `deep_nesting_cannot_exhaust_the_stack` runs 20,000 levels on a 512 KiB thread; `quotes_are_read_eight_deep` pins the quote cap |
| medium | `crates/book/src/lib.rs` | pulldown-cmark does not end a line at a lone CR when it scans fences and HTML blocks, so a CR-only chapter could hide a violation. | fixed `40a37c6`: each lone CR becomes an LF before parsing, a one-byte swap that keeps line numbers. `line_endings_parse_alike`, plus lone-CR parity in every violation test |
| medium | `crates/xtask/src/workflow.rs` | The deny gate could be bypassed: `\|\| exit 0`, a YAML line fold, `if: false`, a matrix `exclude`, `with: command: fetch`. | fixed `9672c26`. Any shell `\|\|` or `set +e` fails, as does `if:` on the gating jobs or steps or a matrix `exclude`/`include`. The deny step must run `check`. 19 mutants in `workflow_matches_ci_contract` |
| medium | `crates/xtask/src/policy.rs` | The golden rule needed a Git checkout, so an unpacked source archive failed `cargo policy`. | fixed in `0b8c498`: Git evaluates the workspace's own `.gitattributes` files through a private repository, so the rule needs no checkout and still fails on wrong attributes. The interim skip in `9672c26` ("no `.git`, pass") was withdrawn after the round-2 test critique (C-002) showed that it contradicted the locked T-005 clause |
| medium | `crates/xtask/src/policy.rs` | Run by a user who does not own the checkout, Git refuses the repository ("dubious ownership"), so the golden rule fails. | fixed in `0b8c498`: the private repository belongs to the caller, so the checkout's owner no longer matters. Checked in WSL as root against a clone owned by another user: `text: unset` |
| medium | `crates/xtask/src/attributes.rs` | Third round: `0b8c498` did not run Git at the workspace, so Git took the current directory as a path prefix. From a subdirectory, as `cargo test` runs in `crates/xtask`, the rule checked the wrong path. | fixed in `7f9a63c`: Git runs at the root (`check_attr_runs_at_the_workspace_and_isolated`). Checked end to end from the root, `crates/` and `crates/xtask` |
| medium | `crates/xtask/src/policy.rs` | Third round: no test reached `text: unspecified`, so a rule accepting it passed the suite. | fixed in `7f9a63c`: an empty `.gitattributes` and `* eol=lf` cases |
| low | `crates/xtask/src/policy.rs` | Third round: the rule checked one invented probe path for `text` alone. A probe-only pattern, or `filter`, `ident` or `working-tree-encoding`, could still leave the real files unprotected. | fixed in `7f9a63c`: every committed golden file plus the probe, and all four byte-rewriting attributes |
| low | `crates/xtask/src/attributes.rs` | Third round: pattern case-sensitivity followed the target directory's filesystem, and paths beyond Windows' MAX_PATH were skipped with only a warning. | fixed in `7f9a63c`: `core.ignorecase=false` and `core.longpaths=true` are pinned, and any Git warning fails the rule (`golden-warning` case) |
| low | `crates/xtask/src/attributes.rs` | Third round: two processes creating the private repository at once could collide, and an existing one was trusted without checks. | fixed in `6a85e66`. `7f9a63c` built the shared repository aside and replaced one that looked unsafe, but the round-3 test critique (C-010) showed that concurrent first runs could still delete each other's repository, failing closed, and that a planted `commondir` passed its check. Each evaluation now makes its own repository in a directory it has just created and removes it afterwards: `taken_names_are_never_used`, `every_private_repository_is_new_and_removed_after_use` |
| low | `crates/xtask/src/attributes.rs` | Third round: the isolation relies on `GIT_CONFIG_GLOBAL`, which needs Git 2.32 or later. | known limit: older Git would read `~/.gitconfig`; the hosts here run Git 2.56 and 2.53 |
| low | `crates/codec/tests/golden.rs` | Third round: no test caught the byte comparison being skipped in create mode, decode errors being accepted, or the two lengths in the message being swapped. | fixed in `ac0e010`: `golden_encoder_change_fails_in_create_mode`, `golden_undecodable_file_fails`, and a different-length case. The message now reads "does not decode to its value" |
| low | `crates/book/src/lib.rs` | A tab-indented detail's violation was one line early. | fixed `40a37c6`: `tab_indented_detail_is_located_at_its_line` |
| low | `crates/book/tests/book.rs` | No test for a strict section holding only prose, or for a level-3 heading in the section. | fixed `40a37c6`: `prose_alone_is_untyped_not_missing`, `level_three_heading_stays_in_the_section` |
| low | `crates/book/tests/cli.rs` | `cli_exit_codes` counted non-Markdown `INT-*` files, such as a merge backup. | fixed `40a37c6`: it counts what the checker reads |
| low | `crates/book/src/lib.rs` | A continuation line holding only Unicode whitespace no longer adds a space to a claim. | accepted: claims are "joined by single spaces" |
| low | `crates/xtask/src/workflow.rs` | `continue-on-error: false`, quoted values and a lowercase action name failed the workflow check. | fixed `9672c26`: `workflow_forms_read_alike` |
| low | `crates/xtask/src/workspace.rs` | `[...]` and `**` member patterns were taken literally. | fixed `9672c26`: such entries fail `lints`; `policy_refuses_member_patterns_it_cannot_expand` |
| low | `crates/xtask/src/policy.rs` | Spec tests: three section-drop mutants survived, and an empty status cell counted as a row. | fixed `9672c26`: the tests assert the exact section message, and rows need filled cells |
| low | `crates/xtask/src/ci.rs` | An inherited `CLIPPY_CONF_DIR` silently lifted the HashMap ban. | fixed `9672c26`: `cargo ci` strips it, with `PERI_GOLDEN_CREATE` and `PROPTEST_*` |
| low | `crates/xtask/src/ci.rs` | The real-executor test printed cargo's errors into the log, and did not pin a multi-argument failure message. | fixed `9672c26`: its children are silenced, and it runs a two-argument unknown command |
| low | `crates/xtask/src/policy.rs` | Fixture cleanup left empty temp directories on Windows. A fixture could inherit the machine's git template, and the asserted `fatal:` text was locale-dependent. | fixed in `9672c26` (cleanup backs off for up to 5 s) and `0b8c498` (the fixtures no longer run Git at all, and the private repository uses an empty template and `LC_ALL=C`). No leftovers in repeated runs |
| low | `crates/codec/tests/golden.rs`, `crates/book/tests/cli.rs` | `remove_dir_all(...).unwrap()` in cleanup could fail a passing test on Windows. | fixed `4e25bfc`, `40a37c6` |
| low | `crates/codec/src/wire.rs` | Growing from empty can hold up to twice a large valid vector's size. | accepted: the cost of reserving nothing from an untrusted count |
| low | `crates/xtask/src/main.rs` | `main`'s `args_os` call has no test. xtask has no subprocess tests, so `cargo test` never rebuilds the binary `cargo ci` is running. | known limit; `cli` is tested with non-Unicode arguments |
| low | `crates/xtask/src/attributes.rs` | The golden rule needs `git` on `PATH`. | known limit: the rule fails, naming the problem; working on this repository requires Git. The policy fixtures no longer need Git (`0b8c498`) |
| low | `crates/xtask/src/ci.rs` | `PROPTEST_*` variables set in a cargo `[env]` table reach the tests. | known limit: `PROPTEST_CASES` still fails closed through the guard tests; the repository's `.cargo/config.toml` sets none |
| low | xtask's `workspace_root()`, and tests that use `env!("CARGO_MANIFEST_DIR")` | Cargo decides freshness by file times, so a checkout that shares a target directory with another can run binaries built for the other one. The paths compiled into them then point at the other checkout: tests read its files, and `cargo ci` and `cargo policy` check it (round-3 test critique, C-005). | known limit: give each checkout its own target directory, as every recorded run does (the matrix log prints each one). Backlog task T-010 makes xtask, and the tests that read workspace files, refuse to run for another checkout |
| low | — | Nothing automated compares the Windows and Linux test lists. | each recorded run states its count; at `94f7363` both systems run the same 153 tests ([test-matrix-log.md](test-matrix-log.md)) |
| low | `crates/xtask/src/main.rs` | Fourth round: a `cargo policy` that printed only each rule's first problem passed every test. | fixed in `6a85e66`: a case with two `golden` problems |
| low | `crates/xtask/src/attributes.rs` | Fourth round: nothing tested that a taken repository name is skipped, or that the `check_attr` call site passes the inherited variable names to `isolated`. | fixed in `6a85e66`: `taken_names_are_never_used`, and `caller_git_settings_are_shut_out`, which runs the rule in a child process with hostile Git variables. The `git init` call site was checked by inspection (round-4 test critique, C-007): without the scrub it still fails closed, and skipping `claim` fails Clippy as dead code |
| low | `crates/xtask/src/attributes.rs` | Fourth round: on Windows the variable checks compared names with case, so an inherited lowercase `git_config_global` failed them. | fixed in `6a85e66`: names compare without case on Windows, as Windows does |
| low | `crates/codec/tests/properties.rs` | Fourth round: a decoder that misread kinds only when a body followed passed the sweep. | fixed in `e882da9`: every id is also decoded with a one-byte body |
| low | `crates/book/tests/book.rs` | Fourth round: detail keys matched without regard to case passed every test. | fixed in `901dbc5`: a lowercase near miss, the test's literal reading of the keys the T-006 Notes name. The Loop phase adds the rule to INT-0002's Grammar |
| low | `crates/book/tests/cli.rs` | Fourth round: a failing assertion left the CLI test's temporary Books behind. | fixed in `901dbc5`: they are removed when dropped |
| low | `crates/xtask/src/policy.rs` | Fourth round: the fixture's cleanup comment said it waits up to 5 s; it waits up to about 9.5 s. | fixed in `6a85e66` |
