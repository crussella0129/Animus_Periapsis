# Sprint 0 Test Plan

Every recorded run states the host OS release, `rustc -Vv`, the head SHA and
where the checkout lives. The canonical suite is `cargo ci`; `cargo deny check`
runs beside it. GitHub Actions starts only at the checkpoint, after this
sprint closes, so its evidence is carried forward (see End-to-End Tests).

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | AC-2: 0 grammar violations, ≥1 criterion per chapter | T-006 / WHEN `cargo book` runs with no arguments THEN strict rule from the marker, a counted line per `INT-*.md` chapter, ≥1 criterion each, 0 violations | `book_intents_pass_strict`; `cargo ci` step `book` |
| INT-0002 | AC-2 (rule coverage) | T-006 / typed-line rules; `missing`; fences of both styles; wrapped typed claims | `typed_line_violations_are_reported`, `missing_criteria_are_reported`, `fenced_code_is_ignored`, `typed_criteria_parse_with_details` |
| INT-0002 | AC-2 (line endings) | T-006 / CRLF built in memory from a chapter with a wrapped criterion and a detail | `crlf_parses_like_lf` |
| INT-0002 | AC-2 (declaration and marker) | T-006 / unknown `typed-acceptance` value or missing marker → error, exit 2, stderr | `unsupported_declaration_is_an_error`, `missing_marker_is_an_error`, `cli_exit_codes` |
| INT-0002 | AC-3: Certified → ¬Refuted (proxy) | T-006 / status consistency; empty list Undetermined | `status_is_consistent` (10,000 cases), `status_truth_table` |
| INT-0002 | AC-4: port rule over the Sprint Loops corpus | T-006 / declared Book rejects untyped items; undeclared Book lifts them; vendored chapters give exactly 5 and 9 criteria and the joined claim | `declared_book_rejects_untyped_items`, `undeclared_book_lifts_untyped_items`, `sprint_loops_fixtures_pass_port_rule`; E2E `e2e_port_rule_sprint_loops_book` |
| INT-0002 | AC-2 (binary contract) | T-006 / exit 0 clean, 1 violations, 2 with stderr for missing directory, flag or extra argument | `cli_exit_codes` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-1: no `unsafe` | T-001 / `lints` and `unsafe` rules; every rule passes on the workspace and on the clean fixture | `policy_flags_missing_workspace_lints`, `policy_flags_missing_forbid_unsafe`, `policy_passes_on_workspace`, `fixture_passes_every_rule` |
| INT-0003 | AC-2: pinned and locked (local half) | T-001 / `toolchain`, `lockfile` rules; T-007 / clippy and test run `--locked` | `policy_requires_exact_toolchain`, `policy_requires_lockfile`, `ci_steps_are_ordered_and_locked`; GitHub half carried forward (T-009) |
| INT-0003 | AC-3: cargo-deny clean (local half) | T-001, T-003 / `cargo deny check` passes at each boundary; T-007 / deny job in the workflow | `int_deny_at_t001`, `int_deny_at_t003`, `int_deny_windows`, `workflow_matches_ci_contract`; GitHub `deny` job carried forward (T-009) |
| INT-0003 | AC-4: fmt and clippy clean on Windows and Linux (local half) | T-007 / every step PASS and exit 0 on the three named checkouts; two failing steps → every step runs, non-zero exit, both named | `int_ci_windows`, `int_ci_wsl`, `e2e_fresh_clone_windows`, `ci_failures_do_not_stop_later_steps`; GitHub matrix carried forward (T-009) |
| INT-0003 | AC-5: design credits (owner sign-off) | T-008 / design-credit list | `prior_art_credits_every_listed_system` (supporting); owner review at the checkpoint decides AC-5 |
| INT-0003 | AC-6: C-capable dependencies pure or justified | T-001 / `credits` rule (a `yes` row names a feature its workspace entry enables, or starts `C allowed:`) and `deps` rule; T-003 / `cc`'s direct dependents equal the `yes` rows and `blake3 feature "pure"` is resolved | `policy_flags_uncredited_dependency`, `policy_requires_prior_art_file`, `policy_requires_complete_dependency_rows`, `policy_flags_direct_dependency`, `policy_passes_on_workspace`; `int_cc_build_dependency_audit` |
| INT-0003 | AC-8: single-word names, every member unpublished | T-001 / `names` and `publish` rules | `policy_requires_single_word_names`, `policy_requires_unpublished_members`, `policy_passes_on_workspace` |
| INT-0003 | AC-9: flagless, idempotent tooling | T-001 / `cargo policy`, T-006 / `cargo book`, T-007 / `cargo ci`: flags or extra arguments exit 2 with stderr; T-007 / each of the three run twice with no arguments exits 0 and leaves the tree clean | `policy_rejects_extra_arguments`, `cli_exit_codes`, `ci_rejects_extra_arguments`, `int_tooling_is_idempotent` |
| INT-0003 | AC-10: README | T-008 / README clause | `readme_keeps_statement_and_links_book` |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: full suite passes on Windows and Linux, OS and toolchain recorded | T-007 / every step PASS, exit 0 on the Windows checkout, the WSL ext4 clone and the `core.autocrlf=true` clone; T-005 / golden bytes, `golden` rule | `int_ci_windows`, `int_ci_wsl`, `e2e_fresh_clone_windows`, `ci_failures_do_not_stop_later_steps`; `golden_v1_corpus_matches` and `policy_requires_binary_golden_attr` within them |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-4: no randomly seeded hash collections | T-001 / `clippy` rule; Clippy rejects `HashMap`/`HashSet` | `policy_requires_disallowed_hash_collections`; `int_disallowed_types_probe` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1: canonical round trip | T-004 / known encodings; canonical rejections at the right offsets; per-type round trip, random and exhaustive mutation (envelopes included) | `varint_known_encodings`, `varint_rejects_non_canonical`, `u32_range_is_enforced`, `decode_rejects_invalid_primitives`, `maps_and_sets_require_strictly_increasing_keys`, `trailing_bytes_rejected`, `envelope_rejects_unknown_version_and_kind`, `object_id_hashes_whole_envelope`, `kind_registry_is_complete`, the `round_trip_*` and `mutants_*` properties, `exhaustive_mutants_reencode` |
| INT-0006 | AC-2: hash agility | T-003 / registry ids and the public registry, digest length, cross-algorithm inequality, binary and text forms | `only_registered_alg_ids_decode`, `hash_registry_is_complete`, `wrong_digest_length_rejected`, `equal_digest_different_alg_unequal`, `binary_round_trip`, `binary_rejects_malformed`, `text_round_trip`, `text_rejects_malformed` |
| INT-0006 | AC-3: golden files fail the suite on any change, on both platforms (local half) | T-005 / byte-exact corpus; changed and missing files fail with create mode off; create mode never overwrites; `create_mode` parsing; `golden` rule keeps `.bin` files binary; T-007 / `test` step strips `PERI_GOLDEN_CREATE` | `golden_v1_corpus_matches`, `golden_changed_file_fails`, `golden_missing_file_fails_without_create_mode`, `golden_create_mode_never_overwrites`, `create_mode_is_exactly_one`, `policy_requires_binary_golden_attr`, `test_step_removes_golden_create`; run in `int_ci_windows`, `int_ci_wsl`, `e2e_fresh_clone_windows`; GitHub-matrix half carried forward (T-009) |
| INT-0006 | AC-4: BLAKE3 official vectors | T-003 / 35 vectors; SHA-256 FIPS; incremental equals one-shot | `blake3_matches_official_vectors`, `sha256_matches_fips_vectors`, `incremental_matches_one_shot` |
| INT-0006 | AC-5: total decoders | T-004 / 100,000 random inputs as every type and envelope; oversize prefixes | `decode_is_total_on_random_bytes`, `oversize_prefixes_rejected_without_allocation` |
| INT-0006 | AC-6: spec before code; code and files agree with the spec | T-002 / `spec` rule at the spec's own commit; T-005 / golden bytes equal the spec's hand-written hex; registry rows equal code | `policy_requires_format_spec`, `spec_examples_match_golden_files`, `spec_registries_match_code`; T-002's commit precedes T-003 and T-004; the owner signs off the spec at the checkpoint (asserted) |

## Unit Tests
### T-001 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `fixture_passes_every_rule`: the clean fixture workspace → no rule fails. This guards the fixture the negative tests rely on.
- `policy_passes_on_workspace`: the real repository root → every rule PASS.
- `policy_flags_missing_workspace_lints`: member without `[lints] workspace = true` → FAIL `lints` naming the crate.
- `policy_flags_missing_forbid_unsafe`: crate root without `#![forbid(unsafe_code)]` → FAIL `unsafe` naming the file.
- `policy_requires_exact_toolchain`: channels `stable`, `1.99`, `nightly-2026-10-01`, `1.99.1-beta` → FAIL `toolchain`; `1.99.1` → PASS.
- `policy_requires_lockfile`: no `Cargo.lock` → FAIL `lockfile`.
- `policy_requires_unpublished_members`: a member without `publish.workspace = true` or `publish = false` → FAIL `publish` naming the crate; a member with either → PASS.
- `policy_flags_direct_dependency`: `foo = "1"` under `[dependencies]`, `[dev-dependencies]` or `[build-dependencies]`, and a `[dependencies.foo]` table → FAIL `deps` naming crate and dependency.
- `policy_requires_prior_art_file`: no `docs/prior-art.md` → FAIL `credits` naming the file.
- `policy_flags_uncredited_dependency`: a workspace dependency with no row → FAIL `credits` naming it.
- `policy_requires_complete_dependency_rows`: an empty licence cell; a C-build cell that does not start with `yes`/`no`; too few cells; a `yes` row with an empty build cell; a `yes` row naming `features = ["pure"]` when the workspace entry does not enable `pure` → FAIL `credits`. A `yes` row whose named feature is enabled, or whose cell starts `C allowed:` with a justification → PASS.
- `policy_requires_disallowed_hash_collections`: `clippy.toml` missing `HashMap` or `HashSet` → FAIL `clippy`.
- `policy_requires_single_word_names`: packages `peri-hash`, `Hash`, `hash_ref`, `9hash` → FAIL `names`; `hash`, `sha256` → PASS.
- `policy_rejects_extra_arguments`: `["policy", "--fix"]`, `["policy", "-v"]`, `["policy", "x"]` → exit 2, non-empty stderr, empty stdout.
- Stubs: temporary fixture trees under the OS temp directory; the command-line front end is called as a function; no mocks.

### T-002 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- `policy_requires_format_spec`: no spec → FAIL `spec`; a spec without `Object identity` → FAIL naming it; Object kinds without registry rows → FAIL; Worked examples without rows → FAIL; the clean fixture's spec → PASS.

### T-003 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- `blake3_matches_official_vectors`: 35 (length, digest) pairs from the official file → equal digests.
- `sha256_matches_fips_vectors`: `""`, `"abc"` → FIPS 180-2 digests.
- `only_registered_alg_ids_decode`: ids 0..=255 → Ok exactly for 0x01 and 0x02.
- `hash_registry_is_complete`: `HashAlg::ALL` is exactly (0x01, `blake3`, 32), (0x02, `sha256`, 32) in that order; `HashAlg::DEFAULT` is Blake3.
- `wrong_digest_length_rejected`: 0, 31 and 33 bytes → `DigestLength` for both algorithms.
- `equal_digest_different_alg_unequal`: the same 32 bytes under both algorithms → unequal; BLAKE3 orders first even against a larger digest.
- `binary_round_trip` (property): random algorithm and digest → identical after round trip.
- `binary_rejects_malformed`: unknown id, truncated digest, empty input, trailing byte → errors.
- `text_round_trip` (property) and `text_rejects_malformed`: uppercase, unknown name, odd length, wrong length, missing `:`, non-hex → errors.
- `incremental_matches_one_shot` (property): random data and cut points → equal for both algorithms.

### T-004 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- `varint_known_encodings`: 0, 127, 128, 300, `u64::MAX` → the bytes in the EARS clause, both directions.
- `varint_rejects_non_canonical`: in a `Vec<u64>`'s second element, `80 00` → `NonMinimalVarint { at: 2 }`; 11 × `ff` and a tenth byte of `02` → `VarintOverflow { at: 2 }`.
- `u32_range_is_enforced`: `80 80 80 80 10` → `IntegerOutOfRange`; `ff ff ff ff 0f` → `u32::MAX`.
- `decode_rejects_invalid_primitives`: second element `02` of a `Vec<bool>` → `InvalidBool { at: 2 }`; tag `02` in a `Vec<Option<u64>>` → `InvalidOptionTag { at: 2 }`; a `Vec<String>` whose second string holds `ff` → `InvalidUtf8` at that string's first content byte.
- `oversize_prefixes_rejected_without_allocation`: as the second element of a two-element `Vec` whose first element is one byte, the `u64::MAX` varint as a `Bytes` and `String` length and as a `Vec`, `BTreeSet` and `BTreeMap` count, followed by three bytes → `UnexpectedEnd { at: 2 }`, no panic.
- `maps_and_sets_require_strictly_increasing_keys`: descending and duplicated map keys, duplicated set elements → `UnsortedOrDuplicateKey` at the offending key.
- `trailing_bytes_rejected`: a valid value plus one byte → `TrailingBytes`.
- `envelope_rejects_unknown_version_and_kind`: versions 0 and 2 → `UnsupportedFormatVersion`; kinds 0x02–0x05 → `ReservedObjectKind`; 0x00, 0x06, 0xff → `UnknownObjectKind`; a one-byte input → `UnexpectedEnd`.
- `object_id_hashes_whole_envelope`: `object_id(alg, env) == alg.hash(env)` and differs from the hash of the bare body, for both algorithms.
- `kind_registry_is_complete`: `ObjectKind::ALL` and `RESERVED_KINDS` have exactly the ids and names in the EARS clause.
- `round_trip_<type>` (13 properties, 10,000 cases each): one per type in the EARS list, envelopes included.
- `mutants_<type>` (13 properties, 10,000 cases each): random mutants that decode re-encode identically.
- `exhaustive_mutants_reencode`: every single-byte substitution, insertion and deletion of a fixed value of each type → decoding mutants re-encode to themselves; at least one mutant per type decodes.
- `decode_is_total_on_random_bytes` (property, 100,000 cases): every type and envelopes.

### T-005 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `golden_v1_corpus_matches`: each corpus value ↔ its `.bin` file, both directions.
- `golden_changed_file_fails`: create mode off, a temp copy of the corpus with one byte flipped in one file → exactly one failure, naming that file.
- `golden_missing_file_fails_without_create_mode`: create mode off, empty temp directory → one failure per case naming its file; nothing written.
- `golden_create_mode_never_overwrites`: create mode on → missing files written; a differing existing file left unchanged and reported as a failure.
- `create_mode_is_exactly_one`: `Some("1")` → true; `Some("0")`, `Some("true")`, `Some("")`, `None` → false.
- `spec_examples_match_golden_files`: every `.bin` equals the spec's worked-example hex for its name; every corpus name has a row.
- `spec_registries_match_code`: parsed spec rows equal the code registries, including `default` and every `reserved` row.
- `policy_requires_binary_golden_attr`: `.gitattributes` without `*.bin binary` → FAIL `golden`.

### T-006 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `typed_line_violations_are_reported`: one fixture per rule class → exactly that violation, with its line:
  - `polarity` (unknown, missing);
  - `modality` (unknown, missing);
  - `claim` (wrong separator, empty);
  - `number` (`x`, unterminated, `0`);
  - `duplicate`;
  - `detail` (key not allowed, no colon).
- `missing_criteria_are_reported`: declared Book, chapter without the section and chapter with an empty section → `missing`; the same chapters under the port rule → no violation.
- `typed_criteria_parse_with_details`: a wrapped typed claim with two details → trimmed lines joined, details kept.
- `fenced_code_is_ignored`: a backtick fence and a tilde fence inside the criteria section, and a fenced `## Acceptance criteria` heading with a typed-looking line in the Intent section → all ignored.
- `declared_book_rejects_untyped_items`: marker with `typed-acceptance: 1` and one untyped item → `untyped`.
- `undeclared_book_lifts_untyped_items`: marker without the declaration → `necessary` `asserted`, wrapped text joined.
- `crlf_parses_like_lf`: a chapter with a wrapped criterion and a detail, converted to CRLF in memory, `\r\n` asserted present → equal criteria.
- `unsupported_declaration_is_an_error`: `typed-acceptance: 2` → error.
- `missing_marker_is_an_error`: a Book directory without `.sprint-loop-book` → error.
- `status_truth_table`: certified, refuted and undetermined cases, and the empty list → Undetermined.
- `status_is_consistent` (property, 10,000 cases).
- `book_intents_pass_strict`: this Book → strict rule; chapters = number of `docs/intents/INT-*.md` files; ≥1 criterion each; 0 violations.
- `sprint_loops_fixtures_pass_port_rule`: `INT-0001-project-book.md` → 5 criteria and `INT-0013-verification-integrity.md` → 9, all `necessary` `asserted`; INT-0001's first claim equals the joined text in the EARS clause; 0 violations.
- `cli_exit_codes`:
  - no arguments, run with `Command::current_dir` at the repository root → 0, one counted line per chapter;
  - clean Book named explicitly → 0;
  - violating Book → 1;
  - a missing directory, `--strict`, `-v`, two directories, a `typed-acceptance: 2` Book, or a Book without a marker → 2, with non-empty stderr and empty stdout.

### T-007 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `ci_steps_are_ordered_and_locked`: the step list is exactly fmt, clippy, test, policy, book; clippy and test carry `--locked` and `--workspace`; clippy denies warnings.
- `ci_failures_do_not_stop_later_steps`: two failing steps (one a missing tool) among passing ones → every step runs; exit status non-zero; summary names both.
- `ci_rejects_extra_arguments`: `["ci", "--fast"]`, `["ci", "-q"]`, `["ci", "x"]` → exit 2, non-empty stderr.
- `test_step_removes_golden_create`: the `test` step's command removes `PERI_GOLDEN_CREATE` from its environment.
- `workflow_matches_ci_contract`: the workflow file meets every point of the T-007 workflow clause.

### T-008 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `prior_art_credits_every_listed_system`: each name in the T-008 list appears in `docs/prior-art.md`.
- `readme_keeps_statement_and_links_book`: the two paragraphs embedded from `README.md` at `620f99c` appear verbatim; a `docs/SUMMARY.md` link; all four commands; the exact sentence "`cargo ci`, `cargo policy` and `cargo book` take no flags."

## Integration Tests
### Canonical runner, deny and probes on the Windows host and in WSL
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- `int_deny_at_t001`, `int_deny_at_t003`: `cargo deny check` on the Windows host at those task boundaries → all four checks ok; output kept with the completion entries.
- `int_ci_windows`: on the Windows 11 host, in the working checkout, `cargo ci` → every step PASS, exit 0.
- `int_deny_windows`: on the Windows 11 host at the tested head, `cargo deny check` (cargo-deny 0.20.2) → advisories, bans, licenses and sources ok.
- `int_ci_wsl`: in WSL Ubuntu 26.04.1, `git clone` the committed `dev` head into the ext4 home directory (not `/mnt/c`) and run `cargo ci` → every step PASS, exit 0.
- `int_tooling_is_idempotent`: in that WSL clone, `cargo ci`, `cargo policy` and `cargo book` twice each with no arguments → every run exits 0, `cargo book` prints one counted line per chapter, and `git status --porcelain` is empty after every run.
- `int_cc_build_dependency_audit`: `cargo tree --workspace --target all -e normal,build -i cc --depth 1` → `cc`'s direct dependents are exactly the third-party `yes` rows (`blake3`); `cargo tree --workspace -e features -i blake3` → shows `blake3 feature "pure"`.
- `int_disallowed_types_probe`: in a scratch copy of the repository, add a `std::collections::HashMap` to `codec` and run `cargo clippy --workspace --all-targets -- -D warnings` → fails on `disallowed_types`; the copy is discarded.

## End-to-End Tests
- **Status:** possible
- `e2e_fresh_clone_windows`: `git clone -c core.autocrlf=true` the committed `dev` head into a new Windows temp directory, confirm every golden `.bin` is byte-identical to the original, run `cargo ci` → every step PASS, exit 0.
- `e2e_port_rule_sprint_loops_book`: `cargo book ~/Animus_Sprint_Loops/docs` (an undeclared Book). Pass requires:
  - the chapter count equals that Book's `INT-*.md` files;
  - every chapter has ≥1 criterion;
  - the total criterion count equals an independent count of top-level `- ` items under each `## Acceptance criteria` heading (82 at commit `e8c3a58`);
  - 0 violations and exit 0.

  Record the Animus_Sprint_Loops commit and the host.
- Carried forward to Sprint 1, as `T-009 (backlog) [intent: INT-0001, INT-0002, INT-0003, INT-0006]`, which Loop appends and links from those chapters' Work evidence: the GitHub Actions run of the checkpoint (head SHA, both matrix legs with their golden and `cargo ci` results, the `deny` job, runner OS and `rustc`) as INT-0003 AC-2 to AC-4 and INT-0006 AC-3 evidence, and the owner's checkpoint sign-offs (INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6). The checkpoint opens only after `close-sprint.sh`, so no Sprint 0 phase can record them.
- Optional: a `cargo ci` run on the Debian host Nighthawk, with its toolchain recorded, once the owner approves its Tailscale SSH check. Sprint 0 loses nothing without it, because WSL covers Linux.
- Not yet possible: INT-0004 AC-6's end-to-end scenarios on Nighthawk (multi-peer sync, crash and restart). They are unlocked by the first sprint that plans INT-0007 or INT-0024 scenarios.
