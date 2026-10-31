# Sprint 0 Test Plan

Every run records the host OS release, `rustc -Vv`, the head SHA and where the
checkout lives. The canonical suite is `cargo ci`; `cargo deny check` runs
beside it.

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) | AC-2: 0 grammar violations, ≥1 criterion per chapter | T-006 / WHEN `cargo book` runs with no arguments THEN strict rule from the marker, one chapter per `INT-*.md` file, ≥1 criterion each, 0 violations | `book_intents_pass_strict`; `cargo ci` step `book` |
| INT-0002 | AC-2 (rule coverage) | T-006 / typed-line rules; `missing`; fenced code; wrapped typed claims | `typed_line_violations_are_reported`, `missing_criteria_are_reported`, `fenced_code_is_ignored`, `typed_criteria_parse_with_details` |
| INT-0002 | AC-2 (line endings) | T-006 / CRLF generated in memory, `\r\n` asserted, equal criteria | `crlf_parses_like_lf` |
| INT-0002 | AC-2 (declaration) | T-006 / unknown `typed-acceptance` value → error, exit 2 | `unsupported_declaration_is_an_error`, `cli_exit_codes` |
| INT-0002 | AC-3: Certified → ¬Refuted (proxy) | T-006 / status consistency; empty list Undetermined | `status_is_consistent` (10,000 cases), `status_truth_table` |
| INT-0002 | AC-4: port rule over the Sprint Loops corpus | T-006 / declared rejects untyped; undeclared lifts; vendored chapters pass | `declared_book_rejects_untyped_items`, `undeclared_book_lifts_untyped_items`, `sprint_loops_fixtures_pass_port_rule`; E2E `e2e_port_rule_sprint_loops_book` |
| INT-0002 | AC-2 (binary contract) | T-006 / exit 0 clean, 1 violations, 2 missing directory, flag or extra argument | `cli_exit_codes` |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) | AC-1: no `unsafe` | T-001 / `lints` and `unsafe` rules; workspace passes | `policy_flags_missing_workspace_lints`, `policy_flags_missing_forbid_unsafe`, `policy_passes_on_workspace` |
| INT-0003 | AC-2: pinned and locked, in CI | T-001 / `toolchain`, `lockfile`, `deps` rules; T-007 / clippy and test run `--locked`; workflow contract | `policy_requires_exact_toolchain`, `policy_requires_lockfile`, `policy_flags_direct_dependency`, `ci_steps_are_ordered_and_locked`, `workflow_matches_ci_contract`; the checkpoint's GitHub run (recorded in Loop) |
| INT-0003 | AC-3: cargo-deny passes, in CI | T-001, T-003, T-004 / `cargo deny check` passes after each; T-007 / deny job in the workflow | `int_deny_windows`; `workflow_matches_ci_contract`; the checkpoint's GitHub `deny` job (recorded in Loop) |
| INT-0003 | AC-4: fmt and clippy clean on Windows and Linux, in CI | T-007 / fmt and clippy steps | `int_ci_windows`, `int_ci_wsl`, `e2e_fresh_clone_windows`; the checkpoint's GitHub run (recorded in Loop) |
| INT-0003 | AC-5: design credits (owner sign-off) | T-008 / design-credit list | `prior_art_credits_every_listed_system` (supporting evidence); owner review at the checkpoint decides AC-5 |
| INT-0003 | AC-6: C-capable dependencies built pure or justified | T-001 / `credits` rule; T-003, T-004 / dependency rows | `policy_flags_uncredited_dependency`, `policy_requires_prior_art_file`, `policy_requires_complete_dependency_rows`, `policy_passes_on_workspace`; `int_cc_build_dependency_audit` |
| INT-0003 | AC-8: single-word crate names | T-001 / `names` rule | `policy_requires_single_word_names`, `policy_passes_on_workspace` |
| INT-0003 | AC-9: flagless, idempotent tooling | T-001, T-006, T-007 / extra or `-` arguments exit 2; two `cargo ci` runs leave the tree clean; README documents it (T-008) | `policy_rejects_extra_arguments`, `cli_exit_codes`, `ci_rejects_extra_arguments`, `int_tooling_is_idempotent`, `readme_keeps_statement_and_links_book` |
| [INT-0004](../../../intents/INT-0004-windows-and-linux.md) | AC-1: full suite passes on Windows and Linux, with OS and toolchain recorded | T-007 / runner on both platforms; T-005 / golden bytes identical | `int_ci_windows`, `int_ci_wsl` (ext4 clone), `e2e_fresh_clone_windows` (`core.autocrlf=true`); `golden_v1_corpus_matches` on each |
| [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) | AC-4: no randomly seeded hash collections | T-001 / `clippy` rule; Clippy `disallowed-types` | `policy_requires_disallowed_hash_collections`; `int_disallowed_types_probe` |
| [INT-0006](../../../intents/INT-0006-format-v1.md) | AC-1: canonical round trip | T-004 / known encodings, canonical rejections, round trip and mutation over every type including envelopes | `varint_known_encodings`, `varint_rejects_non_canonical`, `u32_range_is_enforced`, `decode_rejects_invalid_primitives`, `maps_and_sets_require_strictly_increasing_keys`, `trailing_bytes_rejected`, `round_trip_all_types`, `mutated_encodings_reencode_identically` |
| INT-0006 | AC-2: hash agility | T-003 / registry ids, digest length, cross-algorithm inequality, binary and text forms | `only_registered_alg_ids_decode`, `wrong_digest_length_rejected`, `equal_digest_different_alg_unequal`, `binary_round_trip`, `binary_rejects_malformed`, `text_round_trip`, `text_rejects_malformed` |
| INT-0006 | AC-3: golden files on both platforms | T-005 / byte-exact corpus; create-only mode; missing file fails | `golden_v1_corpus_matches`, `golden_create_mode_never_overwrites`, `golden_missing_file_fails_without_create_mode`; run in `int_ci_windows`, `int_ci_wsl`, `e2e_fresh_clone_windows` |
| INT-0006 | AC-4: BLAKE3 official vectors | T-003 / 35 vectors; SHA-256 FIPS; incremental equals one-shot | `blake3_matches_official_vectors`, `sha256_matches_fips_vectors`, `incremental_matches_one_shot` |
| INT-0006 | AC-5: total decoders | T-004 / 100,000 random inputs per type and envelope; oversize prefixes | `decode_is_total_on_random_bytes`, `oversize_prefixes_rejected_without_allocation` |
| INT-0006 | AC-6: spec before code | T-002 / `spec` rule at the spec's own commit; T-004 / envelope rules; T-005 / registry rows equal code | `policy_requires_format_spec`, `policy_passes_on_workspace`, `envelope_rejects_unknown_version_and_kind`, `object_id_hashes_whole_envelope`, `spec_registries_match_code`; T-002 commit precedes T-003 and T-004 in history; owner review at the checkpoint (asserted) |

## Unit Tests
### T-001 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- `fixture_passes_every_rule`: the clean fixture workspace → no rule fails (guards the fixture itself).
- `policy_passes_on_workspace`: the real repository root → every rule PASS.
- `policy_flags_missing_workspace_lints`: member without `[lints] workspace = true` → FAIL `lints` naming the crate.
- `policy_flags_missing_forbid_unsafe`: crate root without `#![forbid(unsafe_code)]` → FAIL `unsafe` naming the file.
- `policy_requires_exact_toolchain`: channels `stable`, `1.99`, `nightly-2026-10-01`, `1.99.1-beta` → FAIL `toolchain`; `1.99.1` → PASS.
- `policy_requires_lockfile`: no `Cargo.lock` → FAIL `lockfile`.
- `policy_requires_binary_golden_attr`: `.gitattributes` without `*.bin binary` → FAIL `golden`.
- `policy_flags_direct_dependency`: a member with `foo = "1"` under `[dependencies]`, `[dev-dependencies]` or `[build-dependencies]` → FAIL `deps` naming crate and dependency.
- `policy_requires_prior_art_file`: no `docs/prior-art.md` → FAIL `credits` naming the file.
- `policy_flags_uncredited_dependency`: a workspace dependency with no prior-art row → FAIL `credits` naming it.
- `policy_requires_complete_dependency_rows`: a row with an empty licence cell, or a C-build cell not starting with `yes`/`no` → FAIL `credits`.
- `policy_requires_disallowed_hash_collections`: `clippy.toml` missing either `std::collections::HashMap` or `HashSet` → FAIL `clippy`.
- `policy_requires_single_word_names`: packages `peri-hash`, `Hash`, `hash_ref`, `9hash` → FAIL `names`; `hash`, `sha256` → PASS.
- `policy_rejects_extra_arguments`: `["policy", "--fix"]`, `["policy", "-v"]`, `["policy", "x"]`, `[]` → usage error (exit 2).
- Stubs: temporary fixture trees under the OS temp directory; no mocks.

### T-002 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- `policy_requires_format_spec`: no spec → FAIL `spec`; a spec missing the `## Object identity` section → FAIL `spec` naming it; Object kinds without registry rows → FAIL `spec`; the clean fixture's spec → PASS.

### T-003 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- `blake3_matches_official_vectors`: 35 (length, digest) pairs from the official file → equal digests.
- `sha256_matches_fips_vectors`: `""`, `"abc"` → FIPS 180-2 digests.
- `only_registered_alg_ids_decode`: ids 0..=255 → Ok exactly for 0x01 and 0x02; `HashAlg::DEFAULT` is BLAKE3.
- `wrong_digest_length_rejected`: 0, 31 and 33 bytes → `DigestLength` for both algorithms.
- `equal_digest_different_alg_unequal`: the same 32 bytes under both algorithms → unequal; BLAKE3 orders first even against a larger digest.
- `binary_round_trip` (property): random algorithm and digest → identical after round trip.
- `binary_rejects_malformed`: unknown id, truncated digest, empty input, trailing byte → errors.
- `text_round_trip` (property) and `text_rejects_malformed`: uppercase, unknown name, odd length, wrong length, missing `:`, non-hex → errors.
- `incremental_matches_one_shot` (property): random data and cut points → equal for both algorithms.

### T-004 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- `varint_known_encodings`: 0, 127, 128, 300, `u64::MAX` → the bytes in the EARS clause, both directions.
- `varint_rejects_non_canonical`: `80 00` → `NonMinimalVarint`; 11 × `ff` and a tenth byte of `02` → `VarintOverflow`.
- `u32_range_is_enforced`: `80 80 80 80 10` → `IntegerOutOfRange`; `ff ff ff ff 0f` → `u32::MAX`.
- `decode_rejects_invalid_primitives`: bool `02`, option tag `02`, string bytes `ff` → typed errors with offsets.
- `oversize_prefixes_rejected_without_allocation`: the `u64::MAX` varint as a `Bytes` length and as a `Vec`, `BTreeSet` and `BTreeMap` count, followed by three bytes → `UnexpectedEnd` at offset 0, no panic.
- `maps_and_sets_require_strictly_increasing_keys`: descending and duplicated map keys, duplicated set elements → `UnsortedOrDuplicateKey` at the offending key.
- `trailing_bytes_rejected`: a valid value plus one byte → `TrailingBytes`.
- `envelope_rejects_unknown_version_and_kind`: versions 0 and 2 → `UnsupportedFormatVersion`; kinds 0x02–0x05 → `ReservedObjectKind`; 0x00, 0x06, 0xff → `UnknownObjectKind`; a one-byte input → `UnexpectedEnd`.
- `object_id_hashes_whole_envelope`: `object_id(alg, env) == alg.hash(env)` and differs from the hash of the bare body, for both algorithms.
- `round_trip_all_types` (property, 10,000 cases): every supported type, including blob envelopes.
- `mutated_encodings_reencode_identically` (property, 10,000 cases): every supported type, including envelopes.
- `decode_is_total_on_random_bytes` (property, 100,000 cases): every supported type and envelopes.

### T-005 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `golden_v1_corpus_matches`: each corpus value ↔ its `.bin` file, both directions; create mode read once from `PERI_GOLDEN_CREATE`.
- `golden_create_mode_never_overwrites`: create mode on, in a temp directory → missing files written, a differing existing file left unchanged and reported.
- `golden_missing_file_fails_without_create_mode`: create mode off, empty temp directory → one failure per case, each naming its file; nothing written.
- `spec_registries_match_code`: parsed spec rows = code registries, including `default` and every `reserved` row.

### T-006 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- `typed_line_violations_are_reported`: one fixture per rule class — `polarity` (unknown, missing), `modality` (unknown, missing), `claim` (wrong separator, empty), `number` (`x`, unterminated, `0`), `duplicate`, `detail` (key not allowed, no colon) → exactly that violation, with its line.
- `missing_criteria_are_reported`: declared Book, chapter without the section and chapter with an empty section → `missing`.
- `typed_criteria_parse_with_details`: a wrapped typed claim with two details → joined claim, details kept, strength computed.
- `fenced_code_is_ignored`: a typed-looking line inside a fence in the criteria section → ignored.
- `declared_book_rejects_untyped_items`: marker with `typed-acceptance: 1` and one untyped item → `untyped`.
- `undeclared_book_lifts_untyped_items`: marker without the declaration → `necessary` `asserted`, wrapped text joined.
- `crlf_parses_like_lf`: CRLF built in memory, `\r\n` asserted present → equal criteria.
- `unsupported_declaration_is_an_error`: `typed-acceptance: 2` → error.
- `status_truth_table`: certified, refuted and undetermined cases, and the empty list → Undetermined.
- `status_is_consistent` (property, 10,000 cases).
- `book_intents_pass_strict`: this Book → strict rule; chapters = number of `docs/intents/INT-*.md` files; ≥1 criterion each; 0 violations.
- `sprint_loops_fixtures_pass_port_rule`: two vendored chapters → 0 violations, all `necessary` `asserted`.
- `cli_exit_codes`: clean Book → 0; violating Book → 1; missing directory, `--strict`, `-v`, two directories → 2 with nothing on stdout.

### T-007 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- `ci_steps_are_ordered_and_locked`: the step list is exactly fmt, clippy, test, policy, book; clippy and test carry `--locked`; clippy denies warnings.
- `ci_failures_do_not_stop_later_steps`: a failing step (including a missing tool) between passing steps → every step runs; the failed step is reported.
- `ci_rejects_extra_arguments`: `["ci", "--fast"]`, `["ci", "-q"]`, `["ci", "x"]` → usage error (exit 2).
- `workflow_matches_ci_contract`: the workflow file meets every point of the T-007 workflow clause.
- `dependabot_targets_work_branch`: `cargo` and `github-actions` ecosystems, each targeting `dev`.

### T-008 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- `prior_art_credits_every_listed_system`: each name in the T-008 list appears in `docs/prior-art.md`.
- `readme_keeps_statement_and_links_book`: the owner's two original paragraphs verbatim, a `docs/SUMMARY.md` link and `cargo ci`.

## Integration Tests
### Canonical runner, deny and probes on the Windows host and WSL
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- `int_ci_windows`: on the Windows 11 host, in the working checkout, `cargo ci` → every step PASS.
- `int_deny_windows`: on the Windows 11 host, `cargo deny check` (cargo-deny 0.20.2) → advisories, bans, licenses and sources ok.
- `int_ci_wsl`: in WSL Ubuntu 26.04.1, `git clone` the committed `dev` head into the ext4 home directory (not `/mnt/c`) and run `cargo ci` → every step PASS.
- `int_tooling_is_idempotent`: in that WSL clone, `cargo ci` twice → `git status --porcelain` empty after each run.
- `int_cc_build_dependency_audit`: `cargo tree -e build -i cc --workspace` → the crates listed match the C-build column of `docs/prior-art.md` (`blake3` yes, built with `pure`).
- `int_disallowed_types_probe`: in a scratch copy, add a `std::collections::HashMap` to `codec` and run `cargo clippy --workspace --all-targets -- -D warnings` → it fails on `disallowed_types`; the copy is discarded.

## End-to-End Tests
- **Status:** possible
- `e2e_fresh_clone_windows`: `git clone -c core.autocrlf=true` the committed `dev` head into a new Windows temp directory, confirm every `.bin` golden file is byte-identical to the original, run `cargo ci` → every step PASS.
- `e2e_port_rule_sprint_loops_book`: `cargo book ~/Animus_Sprint_Loops/docs` (an undeclared Book) → chapter count equal to that Book's `INT-*.md` files, 0 violations, exit 0. Record the Animus_Sprint_Loops commit SHA and the host.
- Not yet possible: an E2E run on the Debian host Nighthawk. Unlocked when the owner approves the Tailscale SSH check (INT-0004 AC-6). The GitHub Actions conclusion becomes available when the checkpoint pushes `dev`; Loop records it as the CI evidence for INT-0003 AC-2 to AC-4.
