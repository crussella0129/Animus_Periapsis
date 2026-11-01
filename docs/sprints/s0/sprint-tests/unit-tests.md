# Sprint 0 Unit Test Results

- **Tested head:** `df52b9db0381fc230f7e07391bc16adb10f6ee3a` on `dev`, committed, with a clean working tree. Later commits change only the Book. This head includes the fixes from the Test-phase [code review](code-review.md) and from the test critiques ([round 1](critique-round1.md), [round 2](critique-round2.md), [round 3](critique-round3.md), [round 4](critique-round4.md), [round 5](critique-round5.md), [round 6](critique-round6.md)).
- **Runner:** the `test` step of `cargo ci`, which runs `cargo test --workspace --locked`. The listing below comes from running that command directly in the Windows working checkout, as recorded in [integration-tests.md](integration-tests.md).
- **Result:** 154 passed, 0 failed, 0 ignored, on the Windows 11 working checkout and in the WSL Ubuntu 26.04.1 ext4 clone. The `test` step also passed in the fresh `core.autocrlf=true` Windows clone ([e2e-tests.md](e2e-tests.md)). No test is platform-gated or `#[ignore]`d: `cfg(windows)` and `cfg(unix)` mark only helper pairs (`non_unicode_name` and its xtask counterpart), each with both variants. Both systems ran the same 154 tests.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, on `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`.
- **Randomness:** property tests use proptest 1.12 with fixed case counts: 10,000 cases per property in `hash`, `codec` and `book`, and 100,000 for decoder totality.
  - The seed is random on each run. proptest stores a failing seed under `proptest-regressions/` and replays it first next time; `PROPTEST_RNG_SEED` replays any seed.
  - The `cargo ci` test step does not pass `PROPTEST_*` variables to its child process, so a shell variable cannot cut those counts.
  - Each property crate also has a guard test (`case_count_is_not_overridden`, `case_counts_are_not_overridden`) that fails when `PROPTEST_CASES` would replace a configured count.
- **Stubs:** one. `ci_failures_do_not_stop_later_steps` gives the runner's loop scripted outcomes, to test the loop on its own. The real executor is tested by `steps_execute_for_real`: a real cargo step, an unknown cargo subcommand, a program that does not exist, and the in-process policy and Book steps. Every other test builds real inputs: temporary fixture trees (named by process id and test) or in-memory text. Tests call the xtask front end as a function and the `book` binary as a process. One xtask test, `caller_git_settings_are_shut_out`, runs its own test binary again in a child process, with Git variables set that the golden rule must ignore.

| Target | Tests |
|--------|-------|
| `xtask` unit tests (`policy`, `attributes`, `ci`, `deny`, `workflow`, `workspace`, `docs`, front end) | 50 |
| `hash` `tests/vectors.rs` | 13 |
| `codec` `tests/properties.rs` | 40 |
| `codec` `tests/alloc.rs` | 3 |
| `codec` `tests/mutants.rs` | 1 |
| `codec` `tests/golden.rs` | 8 |
| `codec` `tests/spec.rs` | 2 |
| `book` `tests/book.rs` | 33 |
| `book` `tests/cli.rs` | 4 |
| **Total** | **154** |

Each table below maps one task's EARS clauses (from the locked [build plan](../sprint-plans/build-plan.md)) to the tests that assert them. Where a clause is verified by an integration or E2E run, the table names that run instead. Tests added for code-review findings sit beside the clause they strengthen. [code-review.md](code-review.md) maps each finding to its regression test.

## T-001: Workspace, guardrails, policy checker and credits
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-1, AC-2, AC-3, AC-6, AC-8, AC-9; [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-4

| EARS clause (WHEN → SHALL) | Test | Result |
|----------------------------|------|--------|
| `cargo policy` at the root → exit 0 and `PASS <rule>` for every rule | `policy_passes_on_workspace`, `policy_command_reports_every_rule` (all 11 rules print `PASS`) | pass |
| the clean fixture → every rule passes | `fixture_passes_every_rule` | pass |
| a rule fails → `cargo policy` prints `FAIL <rule>: <problem>` for each problem in its rule's place, nothing on stderr, exit 1 | `policy_command_fails_naming_each_failing_rule` compares the whole output: one failing rule, two failing rules, and one rule with two problems. `int_policy_reports_failures` (integration, Windows and WSL) | pass |
| a member that does not resolve to `publish = false` → FAIL `publish`, naming the crate | `policy_requires_unpublished_members`: both accepted forms pass; a workspace `publish = true` and a missing workspace value each fail | pass |
| no `[lints] workspace = true` → FAIL `lints`, naming the crate; the workspace lint must forbid `unsafe_code` | `policy_flags_missing_workspace_lints`: a member without `[lints]` fails. A root `unsafe_code` that is missing, `allow`, `deny`, commented out, or in the wrong table also fails | pass |
| a crate root without `#![forbid(unsafe_code)]` → FAIL `unsafe`, naming the file | `policy_flags_missing_forbid_unsafe` (`src/main.rs` and `src/lib.rs`). The attribute counts only among the crate root's leading lines: inside a block comment or a nested module it fails `unsafe`; after a doc comment and another attribute it passes | pass |
| a toolchain channel that is not an exact `X.Y.Z` → FAIL `toolchain` | `policy_requires_exact_toolchain` (`stable`, `1.99`, `nightly-2026-10-01` and `1.99.1-beta` fail; `1.99.1` passes) | pass |
| no `Cargo.lock` → FAIL `lockfile` | `policy_requires_lockfile` | pass |
| a direct dependency (normal, dev or build, or a `[dependencies.<name>]` table) → FAIL `deps`, naming crate and dependency | `policy_flags_direct_dependency`: every form, including `[target.'cfg(...)'.*]` tables, fails naming crate, dependency and table; an inherited target dependency passes | pass |
| no prior-art file; a missing or incomplete dependency row; a `yes` row without an enabled named feature or a `C allowed:` justification → FAIL `credits` | `policy_requires_prior_art_file` and `policy_flags_uncredited_dependency` (including a `[workspace.dependencies.<name>]` table). `policy_requires_complete_dependency_rows`: the C-build cell's first word must be `yes` or `no`, so `not verified` fails, and table-form `blake3` without `pure` fails | pass |
| `clippy.toml` without both hash types under `disallowed-types` → FAIL `clippy` | `policy_requires_disallowed_hash_collections`: a commented-out entry fails, as do entries under another key and a `.clippy.toml` or member `clippy.toml` that would shadow the root config | pass |
| a member uses `HashMap` or `HashSet` → Clippy fails on `disallowed_types` | `int_disallowed_types_probe` (integration: `HashMap` and `HashSet`, each in its own clone) | pass |
| a package name outside `[a-z][a-z0-9]*` → FAIL `names`, for any workspace package | `policy_requires_single_word_names` (`peri-hash`, `Hash`, `hash_ref` and `9hash` fail; `hash` and `sha256` pass). `policy_finds_members_like_cargo`: a member outside `crates/*` is checked. `policy_refuses_member_patterns_it_cannot_expand` covers `[...]` and `**` | pass |
| any argument besides the command → exit 2, usage on stderr | `policy_rejects_extra_arguments`, `non_unicode_arguments_are_usage_errors` (both also assert empty stdout) | pass |
| `cargo deny check` at the task boundary → four checks ok, recorded | `int_deny_at_t001` (integration). The licences `deny.toml` allows are themselves all GPL-3.0-compatible (INT-0003 AC-3): `deny_allows_only_gpl3_compatible_licences`, and `incompatible_licences_are_named` for the main list, both forms of per-crate exception, an identifier that only starts with an allowed one, and an unreadable policy. `deny.toml` may not hide crates from cargo-deny through `[graph] exclude` or `[licenses] clarify`, and must check dev-dependencies' licences (`include-dev = true`): `settings_that_hide_licences_are_named` | pass |
| (all rules) manifests are read as Cargo reads them | `policy_reads_manifests_as_toml`: multi-line arrays, `path="…"`, dotted keys, `publish = { workspace = true }`. Also `policy_flags_invalid_toml`, `policy_ignores_byte_order_marks`, `wildcards_match_within_a_component` and `paths_resolve_inside_the_root` | pass |

## T-002: Format v1 specification
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md) AC-6

| EARS clause (WHEN → SHALL) | Test | Result |
|----------------------------|------|--------|
| `cargo policy` after the task → the `spec` rule passes | `policy_passes_on_workspace` | pass |
| no spec, a missing section, or a registry or worked-examples table without rows → FAIL `spec`, naming what is missing | `policy_requires_format_spec`: the missing file; each of the 11 sections removed on its own; each table emptied; registry rows without their status cell | pass |

The spec landed in `951be4f`, before the hash code (`0fdb12a`) and the codec code (`1f747f3`).

## T-003: Algorithm-tagged hashes in the hash crate
- **Intents:** [INT-0006](../../../intents/INT-0006-format-v1.md) AC-2, AC-4; [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-3, AC-6

| EARS clause (WHEN → SHALL) | Test | Result |
|----------------------------|------|--------|
| the 35 official BLAKE3 inputs → the first 32 bytes of each vector's `hash` | `blake3_matches_official_vectors` (includes the empty input) | pass |
| SHA-256 of `""` and `"abc"` → the FIPS 180-2 digests | `sha256_matches_fips_vectors` | pass |
| `from_id` for 0..=255 → Ok exactly for 0x01 and 0x02, `UnknownAlgorithm(id)` otherwise | `only_registered_alg_ids_decode` | pass |
| the public registry → exactly Blake3 then Sha256, Blake3 the default | `hash_registry_is_complete` | pass |
| a digest of the wrong length → `DigestLength { expected, actual }` | `wrong_digest_length_rejected` (0, 31 and 33 bytes, both algorithms) | pass |
| equal digests under different algorithms → unequal, ordered by id first | `equal_digest_different_alg_unequal` | pass |
| binary round trip; unknown id, truncation and trailing bytes rejected | `binary_round_trip` (property, 10,000 cases), `binary_rejects_malformed` | pass |
| text form `<name>:<lowercase hex>` round trip; uppercase, unknown name, odd or wrong length and a missing `:` rejected | `text_form_matches_the_spec`: the spec's BLAKE3 example and SHA-256 of "abc" as fixed strings, rendered and parsed. `text_round_trip` (property, 10,000 cases). `text_rejects_malformed` asserts the exact error for each case, including 65 digits and an uppercase or mixed-case name with valid hex | pass |
| chunked `Hasher::update` → equals one-shot, both algorithms | `incremental_matches_one_shot` (property, 10,000 cases); `case_count_is_not_overridden` | pass |
| `cc`'s direct dependents equal the C-build `yes` rows; `blake3 feature "pure"` resolved | `int_cc_build_dependency_audit` (integration) | pass |
| `cargo policy` and `cargo deny check` at the boundary → pass, recorded | `int_deny_at_t003` (integration); T-003 completion entry | pass |

## T-004: Canonical binary encoding in the codec crate
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md) AC-1, AC-5

| EARS clause (WHEN → SHALL) | Test | Result |
|----------------------------|------|--------|
| `u64` → minimal LEB128: `00`, `7f`, `80 01`, `ac 02`, `ff … 01` | `varint_known_encodings` (both directions) | pass |
| `80 00`, an 11-byte varint or a tenth byte of `02` as a `Vec<u64>`'s second element → `NonMinimalVarint` or `VarintOverflow` at 2 | `varint_rejects_non_canonical`; `truncated_varint_located_at_its_start` (a cut-off varint is reported at 2, where it starts) | pass |
| `u32` from `80 80 80 80 10` → `IntegerOutOfRange`; from `ff ff ff ff 0f` → `u32::MAX` | `u32_range_is_enforced` | pass |
| bad bool, option tag or UTF-8 as a second element → the named error at its offset (UTF-8 at the first content byte) | `decode_rejects_invalid_primitives`, with UTF-8 fixtures that separate the first content byte from the first invalid byte and from one past the length prefix's start | pass |
| a `u64::MAX` length or count prefix → `UnexpectedEnd { at: 2 }` without allocating | `oversize_prefixes_rejected_without_allocation`; `overrun_reports_bytes_still_needed` (`needed` is the shortfall; the message is true). `hostile_hash_count_reserves_nothing` and `hostile_pair_count_reserves_nothing` cap a rejected input's peak heap below 1,024 bytes. `valid_vec_grows_as_it_decodes` | pass |
| map or set keys not strictly increasing → `UnsortedOrDuplicateKey` at the offending key | `maps_and_sets_require_strictly_increasing_keys` | pass |
| bytes after the value → `TrailingBytes` | `trailing_bytes_rejected` | pass |
| 10,000 values per type → decode(encode(v)) = v | `round_trip_{u64, u32, bool, string, bytes, hash_ref, option, vec, set, map, pair, nested, envelope}` (13 properties) | pass |
| 10,000 random mutants per type that still decode → re-encode to the mutant | `mutants_{…}` (13 properties, same types) | pass |
| every single-byte substitution, insertion and deletion of a fixed value per type → decoding mutants re-encode to themselves; at least one decodes per type | `exhaustive_mutants_reencode` | pass |
| 100,000 random byte strings as every type and as an envelope → Ok or Err, never a panic | `decode_is_total_on_random_bytes`; `case_counts_are_not_overridden` | pass |
| envelope version ≠ 1, reserved kinds 0x02–0x05, unknown kinds → the three named errors; `object_id` hashes the whole envelope | `envelope_rejects_unknown_version_and_kind`: all 256 versions and all 256 kinds, each with an empty and a one-byte body, against the spec's version rule and kind table written out by hand, and `ObjectKind::from_id` for every id; class counts fail a sweep that skips ids. `object_id_hashes_whole_envelope` | pass |
| the public kind registry → exactly blob defined and the four reserved kinds | `kind_registry_is_complete` | pass |

## T-005: Golden corpus and spec consistency
- **Intents:** [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3, AC-6; [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1

| EARS clause (WHEN → SHALL) | Test | Result |
|----------------------------|------|--------|
| each corpus value ↔ its `.bin` file, byte for byte, both directions | `golden_v1_corpus_matches` (23 files) | pass |
| create mode off, a missing file → fail, naming it | `golden_missing_file_fails_without_create_mode` (one failure per case; nothing written) | pass |
| create mode off, one changed byte → fail, naming that file | `golden_changed_file_fails`: exactly two failures, both naming that file, asserted word for word. The locked test plan says one; the harness compares the bytes and then decodes, and the changed byte also breaks the decoded value. `golden_encoder_change_fails`: when only the encoder's output changes, with equal and with different lengths, exactly one "encoding changed" failure per file. `golden_undecodable_file_fails`: when the encoder and the file agree on bytes that do not decode, exactly one "does not decode" failure | pass |
| create mode on → write only missing files; a differing file left unchanged and failed | `golden_create_mode_never_overwrites`; `golden_encoder_change_fails_in_create_mode` (a file that still decodes but differs from the encoder fails and is left as committed) | pass |
| `create_mode` → true exactly for `Some("1")` | `create_mode_is_exactly_one` (`"0"`, `"true"`, `""` and unset are false) | pass |
| every golden file equals the spec's hand-written hex; every corpus name has a row | `spec_examples_match_golden_files` | pass |
| the spec's hash and kind tables equal the code registries | `spec_registries_match_code` | pass |
| `.gitattributes` does not mark `*.bin` files as binary → FAIL `golden` | `policy_requires_binary_golden_attr`. Git evaluates the workspace's own `.gitattributes` files for every committed golden file and a probe, from the workspace root, inside a checkout or out of one, through a private repository made for each evaluation in a new directory and removed afterwards. Each of these fails: no `*.bin binary` (`text: unspecified`, `text: auto`); `*.bin text`; a later overriding line; a nested file at any level down to `v1/`; `filter`, `ident` or `working-tree-encoding` set; a real golden file left out by a probe-only pattern; anything Git warns about; Git unable to answer. A broken `.git` changes nothing. `policy_command_fails_naming_each_failing_rule`: `cargo policy` itself prints `FAIL golden` and exits 1. The `attributes` unit tests: `check_attr_runs_at_the_workspace_and_isolated`; `isolated_removes_every_inherited_git_variable` and `every_git_variable_is_scrubbed`; `caller_git_settings_are_shut_out`, where Git variables that mark `*.bin` binary for a plain `git` change nothing for the rule; `taken_names_are_never_used` and `every_private_repository_is_new_and_removed_after_use`; `a_missing_workspace_is_an_error`. `int_policy_reports_failures` (integration, Windows and WSL) | pass |

## T-006: Typed-acceptance validator in the book crate
- **Intents:** [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) AC-2, AC-3, AC-4; [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-9

The validator reads chapters as CommonMark (pulldown-cmark), the way GitHub renders the Book. The code review found that the first, line-based parser did not; see [code-review.md](code-review.md).

| EARS clause (WHEN → SHALL) | Test | Result |
|----------------------------|------|--------|
| `cargo book` with no arguments → strict rule from the marker, a counted line per chapter, ≥1 criterion each, 0 violations, exit 0 | `book_intents_pass_strict`, `cli_exit_codes` (no-argument case run at the repository root) | pass |
| a typed line breaking one rule → exactly one violation with file, line and rule | `typed_line_violations_are_reported` (polarity, modality, claim, number, duplicate, detail). `detail_keys_follow_each_modality`: every modality with the eleven keys the T-006 Notes name and four near misses, exact case included; an allowed key is kept as a detail, any other is one `detail` violation. `printed_violations_name_file_line_and_rule`: each of the eight rules prints `path:line: rule: `. `numbers_are_plain_digits` (`+1`, `02`, `0`, overflow). `tab_indented_detail_is_located_at_its_line`. Every violation test checks the path, and that CRLF and a lone CR give the same violations | pass |
| a declared Book's chapter without criteria, or with an empty section → `missing` | `missing_criteria_are_reported` (no violation under the port rule); `prose_alone_is_untyped_not_missing` | pass |
| fenced lines of both styles, including a fenced criteria heading → ignored | `fenced_code_is_ignored`, `long_and_tagged_fences_follow_commonmark`, `unclosed_fence_ends_with_its_item`, `indented_code_opens_no_fence`, `comments_breaks_and_prose_are_not_criteria` | pass |
| a wrapped typed claim → trimmed lines joined by single spaces | `typed_criteria_parse_with_details`, `wrapped_details_keep_their_lines`, `quotes_join_a_claim_and_code_does_not` | pass |
| an untyped item → `untyped` in a declared Book, `necessary` `asserted` with joined text in an undeclared one | `declared_book_rejects_untyped_items`, `undeclared_book_lifts_untyped_items`, `undeclared_book_lifts_untagged_labels`, `every_list_form_holds_criteria` (`1.`, `1)`, `*`, `+`, indented and lazy items), `headings_follow_commonmark`, `level_three_heading_stays_in_the_section`, `deep_nesting_cannot_exhaust_the_stack`, `quotes_are_read_eight_deep`; CLI `undeclared_labels_are_lifted` | pass |
| CRLF built in memory (with `\r\n` asserted present) → criteria equal the LF parse | `crlf_parses_like_lf` (a lone CR too); `line_endings_parse_alike` (fences, HTML blocks, indented code and quotes under LF, CRLF and a lone CR, both rules) | pass |
| `typed-acceptance:` other than `1`, a repeated declaration, or no marker → error; the binary exits 2 with stderr | `unsupported_declaration_is_an_error`, `repeated_declaration_is_an_error`, `missing_marker_is_an_error`, `cli_exit_codes`, `repeated_declaration_exits_2` | pass |
| `status` over any outcomes → never both Certified and Refuted; Certified iff all hold; Refuted iff a necessary criterion fails; empty → Undetermined | `status_is_consistent` (10,000 cases; `status` returns one variant, so "both" cannot be represented, and the two iff assertions pin which); `status_truth_table`; `case_count_is_not_overridden` | pass |
| the vendored Sprint Loops chapters, undeclared → 0 violations, 5 and 9 criteria, all `necessary` `asserted`, INT-0001's joined first claim | `sprint_loops_fixtures_pass_port_rule` | pass |
| exit 1 on violations, 0 when clean, 2 with stderr and empty stdout for a missing directory, a flag or two directories | `cli_exit_codes` (the violation prints as the chapter's full path, `:4: untyped: `), `non_unicode_argument_is_a_book_path` | pass |

## T-007: Canonical suite runner and CI workflow
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2, AC-3, AC-4, AC-9; [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1; [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) AC-2

| EARS clause (WHEN → SHALL) | Test | Result |
|----------------------------|------|--------|
| `cargo ci` → exactly fmt, clippy, test, policy, book in order, one PASS or FAIL line each | `ci_steps_are_ordered_and_locked` (the whole step list, commands included); every `cargo ci` run in the integration and E2E records | pass |
| two failing steps, one a missing tool → every step runs, non-zero exit, both named | `ci_failures_do_not_stop_later_steps` (the loop, with scripted outcomes); `steps_execute_for_real` (an unknown cargo subcommand and a program that does not exist give FAIL with their reason); `book_step_fails_on_a_violation` (a Book with violations fails the `book` step with their count); `int_ci_reports_failures` (integration, Windows and WSL: a real `cargo ci` with fmt and test failing prints both, `ci: failed: fmt, test`, and exits 1); `int_ci_reports_book_violation` (the same with test and book failing, `ci: failed: test, book`) | pass |
| any argument besides the command → exit 2, usage on stderr | `ci_rejects_extra_arguments`, `non_unicode_arguments_are_usage_errors` | pass |
| the `test` step → `PERI_GOLDEN_CREATE` removed from its environment | `test_step_removes_golden_create` (and `PROPTEST_CASES`, `CLIPPY_CONF_DIR`); `proptest_variables_are_never_inherited`; `int_ci_ignores_hostile_environment` (integration: a deleted golden file is not recreated) | pass |
| `cargo ci` on the three checkouts → all PASS, exit 0, OS and `rustc -Vv` recorded | `int_ci_windows`, `int_ci_wsl`, `e2e_fresh_clone_windows` | pass |
| `cargo ci`, `cargo policy` and `cargo book` twice each → exit 0, one counted line per chapter, clean tree | `int_tooling_is_idempotent` | pass |
| the workflow file → push on `dev` and `main`, `pull_request`, the two-OS matrix running `cargo ci`, a `deny` job, no swallowed failures | `workflow_matches_ci_contract`: each point is checked on its own job, and 19 mutants fail. No trigger may be filtered (`paths`, `paths-ignore`, `branches-ignore`, `types`, an inline `pull_request`, a `pull_request` branch list without dev and main), and each such form must fail. The deny job must run every check on this workspace, as a step whose inputs the rule reads and whose place and shell nothing changes: its narrowed forms must not count and three full forms must. `workflow_forms_read_alike` | pass |

## T-008: README and documentation checks
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-5 (supporting), AC-10

| EARS clause (WHEN → SHALL) | Test | Result |
|----------------------------|------|--------|
| prior-art names every listed system, project and test-data source | `prior_art_credits_every_listed_system` (36 names, each matched as a whole word); `credited_names_are_whole_words` | pass |
| the README keeps both owner paragraphs verbatim, links `docs/SUMMARY.md`, documents the four commands and states the no-flags sentence exactly | `readme_keeps_statement_and_links_book`: `cargo ci` and `cargo deny check` as fenced command lines, and a line besides the no-flags sentence on what `cargo policy` and `cargo book` do. `readme_must_document_each_command`: four READMEs that only mention the commands fail | pass |
