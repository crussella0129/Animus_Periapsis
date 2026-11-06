# Sprint 0 Mutation Sweep

Test critique rounds 3 to 7 each blocked on three to five material concerns, most of them assertions that a wrong program would still pass. So before round 8, the whole suite was swept for such assertions at `d7b0484`, instead of waiting for each round to find a few more.

- **Method:**
  1. Eight testers, one per task (T-004 in two halves, T-007 together with T-008), each took its task's EARS clauses from the locked [build plan](../sprint-plans/build-plan.md). Each changed the code under test one token at a time in a scratch clone, and kept each change (a *mutant*) that broke a clause while every test still passed.
  2. An independent verifier applied each kept mutant to a fresh clone and ran the whole suite. All 108 reproduced.
  3. Six implementers, each on its own files, added or tightened tests until each mutant failed one, and fixed the one code defect found.
  4. One verifier per implementer re-applied every mutant, with variants (225 in all), and tried new ones of its own. 214 failed a test. The 11 that did not were all in the `spec` rule's new hand-written Markdown reader, or equivalent to the code; see [Second pass](#second-pass).
  5. The session then replaced that reader with pulldown-cmark, in `cargo policy` and in codec's spec test, closed the verifiers' other issues, and mutated both new readers: 26 mutants, each failing a test.
- **Outcome at `0a9fd40`:** each of the 108 mutants fails at least one test. One finding was a code defect: the `spec` rule's line scan counted headings and table rows that GitHub shows as code, a comment or plain text (T-002). The other 107 were weak or missing assertions. The suite grew from 157 tests at `d7b0484` to 171.

| Task | Mutants kept | Reproduced |
|------|--------------|------------|
| T-001 | 21 | 21 |
| T-002 | 9 | 9 |
| T-003 | 12 | 12 |
| T-004a | 11 | 11 |
| T-004b | 11 | 11 |
| T-005 | 12 | 12 |
| T-006 | 16 | 16 |
| T-007/T-008 | 16 | 16 |
| **Total** | **108** | **108** |

## Findings

Each row is one finding as its implementer recorded it: the clause, with the mutant's label from the sweep (`M..`), and the test that now fails on it. The verifier's re-application of every mutant follows each table.

### `cargo policy` (T-001, T-002)

| Task | Finding | Result | Test that now fails |
|------|---------|--------|---------------------|
| T-001 | T-001 lints: a member without [lints] workspace = true fails `lints` and names the crate (M01: only a missing [lints] table counted) | killed | policy_flags_missing_workspace_lints: new fixture lints-own-table, where the member has its own [lints.rust] missing_docs = "warn" instead of [lints] workspace = true. It fails `lints`, naming crates/hash. |
| T-001 | T-001 lints, root half: the workspace lint must forbid unsafe_code (M03 any table form accepted; M29 any lint starting with `unsafe` accepted) | killed | policy_flags_missing_workspace_lints: the failing root list gains `unsafe_code = { level = "allow", priority = -1 }` and `unsafe_op_in_unsafe_fn = "forbid"`. Each fails `lints`, naming unsafe_code. |
| T-001 | T-001 unsafe: each crate root (src/lib.rs, src/main.rs) without #![forbid(unsafe_code)] fails `unsafe` and names the file (M04: only main.rs checked when it exists) | killed | policy_flags_missing_forbid_unsafe: new fixture unsafe-lib-beside-main, where main.rs has the attribute and lib.rs does not. It fails `unsafe`, naming crates/hash/src/lib.rs. |
| T-001 | T-001 toolchain: a channel that is not an exact X.Y.Z release fails `toolchain` (M05 len >= 3, M06 empty parts, M07 alphanumeric parts) | killed | policy_requires_exact_toolchain: the channel list gains 1.99.1.0, 1.99. and 1.99.x. Each fails `toolchain`, naming the channel. |
| T-001 | T-001 toolchain with INT-0003 AC-2 (rust-toolchain.toml names an exact release): a missing file or a file with no channel pins nothing (M25, M26) | killed | policy_requires_exact_toolchain: fixture toolchain-missing (file removed) fails `toolchain` with "rust-toolchain.toml is missing". Fixture toolchain-no-channel (`[toolchain] components = ["clippy"]`) fails `toolchain` with "has no [toolchain] channel". I used the corrected snippet, which binds its own fixtures. |
| T-001 | T-001 deps: a direct dependency fails `deps`, naming the crate and the dependency, even after an inherited entry (M08 break at an inherited entry; M28 take(1)) | killed | policy_flags_direct_dependency: the per-form tuple list gains `[dependencies]\nblake3.workspace = true\nfoo = "1"`. It must fail `deps`, naming crates/hash, `foo` and [dependencies]. |
| T-001 | T-001 deps under [target.<cfg>]: the underscore spellings dev_dependencies and build_dependencies (M09, manifest.rs) | killed | policy_flags_direct_dependency: two tuples, `[target.'cfg(unix)'.dev_dependencies] libc` and `[target.'cfg(unix)'.build_dependencies] cc`. Each fails `deps`, naming the dependency and the exact table header. |
| T-001 | T-001 deps, and manifests read as Cargo reads them: a top-level inline `dependencies = { .. }` table (M30, manifest.rs as_table) | killed | policy_flags_direct_dependency: new fixture deps-inline-table, a manifest opening with `dependencies = { foo = "1" }`. It fails `deps`, naming crates/hash, `foo` and [dependencies]. |
| T-001 | T-001 publish: a member that does not resolve to publish = false fails `publish` (M31: any array accepted) | killed | policy_requires_unpublished_members: the publish-true fixture became a loop over `publish = true` and `publish = ["crates-io"]` (publish-own-0/1). Each fails `publish`, naming crates/hash. |
| T-001 | T-001 credits: every non-path entry of [workspace.dependencies] needs a row (M10 skips plain strings; M11 skips git entries) | killed | policy_flags_uncredited_dependency: fixtures credits-source-0 (`zstd = "0.13"`) and credits-source-1 (`zstd = { git = ... }`). Each fails `credits`, naming `zstd`. |
| T-001 | T-001 credits: a dependency without its own row fails (M12: row matched by name prefix) | killed | policy_flags_uncredited_dependency: fixture credits-prefix-name adds `blake = {...}` while only the `blake3` row exists. It fails `credits`, naming `blake`. |
| T-001 | T-001 credits: a `yes` row's how cell must start with `C allowed:` and a reason (M13: split_once anywhere in the cell) | killed | policy_requires_complete_dependency_rows: failing row `\| `blake3` \| ... \| yes \| Rust only (C allowed: never) \|`. |
| T-001 | T-001 credits: a `yes` row must name, as features = ["<name>"], a feature the workspace entry enables (M14 empty list; M15 substring) | killed | policy_requires_complete_dependency_rows: failing rows `features = []` and `features = ["pur"]`. |
| T-001 | T-001 credits: the licence cell must be filled; "—" marks an empty cell (M16) | killed | policy_requires_complete_dependency_rows: failing row with licence `—`. |
| T-001 | T-001 clippy: clippy.toml must list std::collections::HashMap and HashSet under disallowed-types (M17 last-segment match; M18 suffix match) | killed | policy_requires_disallowed_hash_collections: configs clippy-not-std-0 (hashbrown::HashMap/HashSet) and clippy-not-std-1 (bare HashMap/HashSet). Each must fail `clippy` with one problem naming std::collections::HashMap and another naming std::collections::HashSet. |
| T-001 | T-001 names: a package name outside [a-z][a-z0-9]* fails `names` (M19 ASCII alphanumeric; M20 Unicode lowercase) | killed | policy_requires_single_word_names: the list gains "hashRef" and "café". |
| T-001 | T-001 names for any workspace package: `exclude` matches whole path components (M21: string prefix, workspace.rs) | killed | policy_finds_members_like_cargo: fixture members-exclude-prefix, with `exclude = ["crates/old"]` and a member crates/older named "Older". It fails `names`, naming crates/older. |
| T-001 | T-001 names for any workspace package: path dependencies of members are members, recursively (M22, workspace.rs) | killed | policy_finds_members_like_cargo: fixture members-nested-path (hash -> tool -> util, with util named "Util"). It fails `names`, naming tools/util. |
| T-001 | T-001 names: a member's own (non-inherited) path dependency is a member (M23, workspace.rs) | killed | policy_finds_members_like_cargo: fixture members-own-path (`tool = { path = "../../tools/tool" }`, with Tool's name invalid). Exactly [deps, names] must fail. |
| T-001 | unit-tests.md T-001 deps row (line 41) claims every form, including [target.'cfg(...)'.*] tables, is tried | killed | policy_flags_direct_dependency now tries all three missing forms: a direct entry after an inherited one, the underscore spellings under [target.*], and the top-level inline table. |
| T-002 | T-002 EARS 2: a required section that is absent, as GitHub shows the spec, fails `spec` (D1 ##Conventions, D2 ###Object kinds, D3 a fenced Change control, D4 #######, D6 an HTML comment) | killed | New test policy_reads_the_spec_as_markdown: each D-form fails `spec`, naming the section. Further cases cover an unclosed ~~~ fence, closing-fence length, marker and indentation, a 4-space or tab-indented heading, HTML other than a comment, indented fences and comments in a list, a closing fence followed by a tab, and headings in lists, quotes or setext form. Passing cases cover indent 1, a tab after #, a closed fence and comments, inline triple-backtick code, and CRLF and CR line endings. |
| T-002 | T-002 EARS 1/2: the Hash algorithms and Object kinds tables need a registry row as GitHub shows it (D7 the table in an HTML comment; D5 the table in a fence) | killed | policy_reads_the_spec_as_markdown: the row in a comment and in a fence, and the Hash algorithms table in an unclosed fence, each fail with "<table>" has no table rows. Also: rows without a header and delimiter, a column-count mismatch, a delimiter with an empty column or indented 4, a blank line before the row, a row with no cell (`\|`), lazy list-item lines, escaped pipes in the header or row, and a status cell past the header's columns. Passing cases cover alignment colons, no trailing pipes, an indented row inside a table, and tables right after a blank line, a fence or a comment. |
| T-002 | T-002 EARS 2: a renamed heading is a missing section (section filter: text.contains / heading.contains / heading.starts_with) | killed | policy_requires_format_spec: in the per-section loop, each heading is renamed to `## <section> notes` (spec-renamed-N) and must fail with no "<section>" section. |
| T-002 | T-002 EARS 2: a table's rows count only under its own heading (table_rows: inside sticks, starts_with, contains, starts true) | killed | policy_requires_format_spec: fixture spec-row-outside. The Object kinds row is removed, and full tables (header, delimiter, row) are placed above the first heading and under `### Object kinds (retired)`. It must fail with "Object kinds" has no table rows. |
| T-002 | T-002 EARS 1/2: a registry row starts with a `0x..` id (SPEC_TABLES prefix loosened to "\| `" for both, hash only or kinds only) | killed | policy_requires_format_spec short_rows gains `\| `01` \| `blake3` \| 32 \| default \|` and `\| `01` \| `blob` \| defined \|`, plus `\| `blob` \| `0x01` \| defined \|`. Each fails with "<table>" has no table rows. |
| T-002 | T-002 EARS 2: a spec that is not UTF-8 fails rather than passes (fail open via read_text else return) | killed | policy_requires_format_spec: fixture spec-not-utf8, with the bytes ff fe 00. It fails `spec`, naming docs/spec/format-v1.md. |
| T-002 | T-002 EARS 2: every missing section and empty table is named, and nothing else (take(1), early return, a filter that reports everything) | killed | policy_requires_format_spec: fixture spec-every-problem removes ## Conventions, ## Change control and the kind row. It asserts that the failures are exactly [("spec", [no Conventions, no Change control, Object kinds has no table rows])]. |
| T-002 | T-002 EARS 2: a heading or row indented four spaces after a blank line is code (heading trim_start; row trim_start or contains) | killed | policy_reads_the_spec_as_markdown: `\n ## Change control`, `\n\t## Change control` and `\n \| `0x01` \| `blob` \| defined \|` each fail. policy_requires_format_spec: the reordered row `\| `blob` \| `0x01` \| defined \|`. |
| T-002 | T-002 EARS 1: `spec` requires nothing more (stricter mutants: BOM kept, filled == cells, no trailing trim, ### and deeper dropped) | killed | policy_requires_format_spec passing cases (spec-ok-N): a BOM before `## Conventions`, `## Change control ##`, `## Change control ` (trailing space), `## Change control ## `, `#### Change control`, a five-column Hash table, and a fifth cell past a four-column header. |
| T-005 | T-005 EARS 8 (golden): Git evaluates every committed golden file, not only the first or last (drain, truncate(1), take(BYTE_ATTRIBUTES.len())) | killed | policy_requires_binary_golden_attr: fixture golden-every-file, with blob-envelope, u32-max and uvarint-0 .bin and `u32-max.bin text`. It fails `golden`, naming u32-max.bin and `text: set`. |
| T-005 | T-005 EARS 8 (golden): `text=false` is not unset text, and Git still converts line endings (keeps_bytes accepts "false") | killed | policy_requires_binary_golden_attr: fixture golden-text-false (`*.bin text=false`) fails `golden` with `text: false`. |

Verifier: 68 mutants re-applied, 57 caught, 11 survived.

The survivors: one mutant of the new reader equivalent to the code (finding 27's analogue), and ten new mutants of the hand-written reader that let `spec` pass documents in which GitHub shows no such heading or row. At `0a9fd40` that reader is gone (see [Second pass](#second-pass)). Each of the ten documents is now a case in `policy_reads_the_spec_as_markdown`, and each fails `spec` as pulldown-cmark reads it. The T-002 rows above name cases that still exist; the line scan their mutants changed does not.

### `cargo ci`, the workflow rule and the README check (T-007, T-008)

| Task | Finding | Result | Test that now fails |
|------|---------|--------|---------------------|
| T-007 | T-007: WHEN the runner starts the `test` step it SHALL remove PERI_GOLDEN_CREATE from that child's environment (mutant: run() re-adds env::vars_os() to each command) | killed | ci::tests::hostile_variables_never_reach_a_step (new) with its child half ci::tests::run_passes_a_clean_workspace (new) |
| T-007 | T-007: WHEN cargo ci runs it SHALL execute exactly fmt, clippy, test, policy, book in order, printing one PASS/FAIL line per step (mutant: run_with(&STEPS[..4], …)) | killed | ci::tests::hostile_variables_never_reach_a_step / run_passes_a_clean_workspace |
| T-007 | T-007 step commands: fmt `cargo fmt --all --check`, clippy `cargo clippy --workspace --all-targets --locked -- -D warnings`, test `cargo test --workspace --locked` (mutants: split args at "--"; drop the last arg; append -A clippy::disallowed_types in cargo_command) | killed | ci::tests::ci_steps_are_ordered_and_locked (extended) |
| T-007 | T-007: print one `PASS <step>` or `FAIL <step>: <reason>` line per step (mutants: reason dropped; PASS printed for every step; FAIL lines written after the loop) | killed | ci::tests::ci_failures_do_not_stop_later_steps (tightened) |
| T-007 | T-007: WHEN two steps fail the runner SHALL still run every step, exit non-zero and name both (mutant: break after the second failure) | killed | ci::tests::ci_failures_do_not_stop_later_steps (fifth step added) |
| T-007 | T-007: a failed step (e.g. cargo fmt --check exiting 1) is reported FAIL (mutant: only exit code 101 fails) | killed | ci::tests::steps_execute_for_real (extended) |
| T-007 | T-007: the `policy` step runs in-process and fails on any policy problem (mutants: .filter(rule != "golden"); .take(1)) | killed | ci::tests::policy_step_fails_on_a_problem (new) |
| T-007 | T-007 environment row: no cargo step inherits CLIPPY_CONF_DIR, so the HashMap ban holds (INT-0005 AC-4) (mutant: scrub only when args[0] == "test") | killed | ci::tests::test_step_removes_golden_create (now loops over every cargo step); ci::tests::hostile_variables_never_reach_a_step |
| T-007 | unit-tests.md: the cargo ci test step does not pass PROPTEST_* variables to its child process (mutant: scrubbed(std::iter::empty())) | killed | ci::tests::hostile_variables_never_reach_a_step / run_passes_a_clean_workspace |
| T-007 | T-007: WHEN cargo ci receives any argument besides its command name it SHALL exit 2 and write a usage message to stderr (mutants: cli drops "--"; cli drops empty arguments) | killed | tests::ci_rejects_extra_arguments, tests::policy_rejects_extra_arguments (both extended); assert_usage_error now runs cli on a missing root |
| T-007 | T-007: … exit 2 and write a usage message to stderr (mutant: USAGE = "ci: failed") | killed | tests::assert_usage_error (used by ci_rejects_extra_arguments, policy_rejects_extra_arguments, non_unicode_arguments_are_usage_errors); tests::main_passes_every_argument_to_cli |
| T-008 | T-008: the README test SHALL find a link to docs/SUMMARY.md (mutants: link check deleted; loosened to contains("docs/SUMMARY.md")) | killed | docs::readme_keeps_statement_and_links_book (extended) |
| T-008 | T-008: the owner's two original paragraphs verbatim, compared against copies embedded from README.md at 620f99c (mutants: contains(paragraph); trim_end comparison; only the first paragraph checked) | killed | docs::readme_keeps_statement_and_links_book (extended) |
| T-008 | T-008: the exact sentence "`cargo ci`, `cargo policy` and `cargo book` take no flags." (mutants: check deleted; loosened to contains("take no flags")) | killed | docs::readme_keeps_statement_and_links_book (extended) |
| T-008 | T-008: `cargo ci` and `cargo deny check` as fenced command lines (mutant: fenced_lines takes every line) | killed | docs::readme_must_document_each_command (extended) |
| T-008 | T-008: docs/prior-art.md SHALL name Git, JGit, … each as a whole word (mutant: names() reads the first char before the match instead of the last) | killed | docs::credited_names_are_whole_words (extended) |
| T-001 | T-001 CLI: WHEN the policy command receives any argument besides its command name … THEN it SHALL exit 2 and write a usage message to stderr (mutant: main uses env::args_os().skip(1).take(1)) | killed | tests::main_passes_every_argument_to_cli (new) with its child half tests::main_rejects_extra_arguments (new) |

Verifier: 27 mutants re-applied, 27 caught.

### `hash` (T-003)

| Task | Finding | Result | Test that now fails |
|------|---------|--------|---------------------|
| T-003 | T-003 text form: WHEN text has an unknown name, THEN parsing SHALL fail (a name that is longer, shorter or empty) | killed | text_rejects_malformed: new rows blake3x:<hex> -> UnknownAlgorithmName("blake3x"), blake:<hex> -> UnknownAlgorithmName("blake"), :<hex> -> UnknownAlgorithmName("") |
| T-003 | T-003 text form: WHEN text has an unknown name or odd or wrong-length hex, THEN parsing SHALL fail (whitespace around the parts, a 0x prefix) | killed | text_rejects_malformed: rows " {valid}" -> UnknownAlgorithmName(" blake3"), "blake3 :<hex>" -> UnknownAlgorithmName("blake3 "), "blake3: <hex>" -> InvalidHex, "{valid}\n" -> InvalidHex, "blake3:0x<hex>" -> InvalidHex |
| T-003 | T-003 text form: WHEN text has uppercase hex, THEN parsing SHALL fail (uppercase only in the low nibble) | killed | text_rejects_malformed: two rows -> InvalidHex. One is the spec example with only its low nibbles uppercased: blake3:aF1349b9f5f9a1a6a0404DeA36dCc9499BcB25c9aDc112b7cC9A93cAe41F3262. The other is its mirror with only the high nibbles uppercased: blake3:Af1349B9F5F9A1A6A0404dEa36DcC9499bCb25C9AdC112B7Cc9a93CaE41f3262 |
| T-003 | T-003: WHEN HashRef::new(alg, digest) receives a digest whose length differs from alg.digest_len(), THEN it SHALL return DigestLength { expected, actual } | killed | wrong_digest_length_rejected: now every length 0..=300 except 32, plus 65,568 (2^16 + 32), for both algorithms -> DigestLength { alg, expected: 32, actual: len } |
| T-003 | T-003 binary form: WHEN the binary input has trailing bytes, THEN parsing SHALL fail (SHA-256) | killed | binary_rejects_malformed: per algorithm, the hash of "x" plus one 0x00 -> TrailingBytes { extra: 1 }, and two hashes back to back -> TrailingBytes { extra: 33 } |
| T-003 | T-003 text form: WHEN text has uppercase hex or an unknown name, THEN parsing SHALL fail (SHA-256) | killed | text_rejects_malformed: rows "sha256:<uppercase SHA-256 of x>" -> InvalidHex, "SHA256:<hex>" -> UnknownAlgorithmName("SHA256"), "sha-256:<hex>" -> UnknownAlgorithmName("sha-256"), "SHA-256:<hex>" -> UnknownAlgorithmName("SHA-256") |
| T-003 | T-003 text form: WHEN text has no ':', THEN parsing SHALL fail | killed | text_rejects_malformed: new loop, "blake3{c}<hex>" for every ASCII char c other than ':' -> MissingSeparator |
| T-003 | T-003: WHEN input is fed to Hasher::update in arbitrary chunks, THEN finalize SHALL equal the one-shot hash for both algorithms | killed | New test incremental_matches_one_shot_on_large_input: 1 MiB vector_input fed in 1,000-, 4,097- and 65,537-byte pieces must equal the one-shot hash, for both algorithms. sha256_matches_fips_vectors gains FIPS 180-2 B.3: one million 'a' -> cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0 |
| T-003 | T-003 binary form: WHEN the binary input has an unknown id, THEN parsing SHALL fail | killed | binary_rejects_malformed: every id except 0x01 and 0x02, followed by 32 digest bytes -> UnknownAlgorithm(id). This replaces the single 0x09 case |
| T-003 | T-003 binary form: WHEN the binary input has a truncated digest, THEN parsing SHALL fail | killed | binary_rejects_malformed: every proper prefix (0 to 32 bytes) of each algorithm's encoding. Length 0 -> Truncated { needed: 1, available: 0 }. Length n -> Truncated { needed: 32, available: n - 1 } |
| T-003 | T-003 binary form: WHEN a HashRef goes to binary [id][digest] and back, THEN it SHALL equal the original | killed | binary_round_trip (property) now asserts bytes[1..] == hash.digest(). New test binary_form_matches_the_spec checks to_bytes against the spec's worked examples hashref-blake3 (01 af1349b9...3262) and hashref-sha256 (02 ba7816bf...15ad) and parses each back |
| T-003 | T-003: WHEN two HashRefs hold identical digest bytes under different algorithms, THEN they SHALL compare unequal and order by algorithm id first | killed | equal_digest_different_alg_unequal: blake3.cmp(&sha256) and higher_digest.cmp(&sha256) must both be Ordering::Less |

Verifier: 33 mutants re-applied, 33 caught.

### `codec` encoding (T-004)

| Task | Finding | Result | Test that now fails |
|------|---------|--------|---------------------|
| T-004 | T-004 EARS: a bad bool / option tag as the second element fails with InvalidBool / InvalidOptionTag at the offset where the bad element starts | killed | decode_rejects_invalid_primitives (crates/codec/tests/properties.rs) |
| T-004 | T-004 round trip for String and Bytes; spec: bytes/string = uvarint length \|\| content (mutant: Encoder::bytes writes the length as one byte) | killed | lengths_and_counts_are_varints (new); round_trip_bytes, mutants_bytes, round_trip_string, round_trip_nested via widened generators (properties.rs) |
| T-004 | T-004 round trip for BTreeSet and BTreeMap; spec: set/map = uvarint count \|\| elements (mutant: count written as one byte, separately for set and map) | killed | lengths_and_counts_are_varints (properties.rs) |
| T-004 | T-004 EARS: keys not strictly increasing in canonical order -> UnsortedOrDuplicateKey at the offending key; bytes order lexicographically by raw content | killed | maps_and_sets_require_strictly_increasing_keys (properties.rs) |
| T-004 | T-004 EARS: any u64 encodes as minimal LEB128 (mutant: wrong encoding for values in [2^35, 2^42)) | killed | varint_known_encodings; round_trip_u64, mutants_u64, round_trip_option, mutants_option, round_trip_pair, mutants_pair via the new any_u64() strategy (properties.rs) |
| T-004 | T-004 EARS: map keys not strictly increasing -> UnsortedOrDuplicateKey at the offending key's offset (mutant: the value is decoded before the key-order check) | killed | maps_and_sets_require_strictly_increasing_keys (properties.rs) |
| T-004 | T-004 round trip for String through the exported impl Encode for str; spec: string = uvarint length \|\| UTF-8 (mutant: enc.raw(self.as_bytes())) | killed | lengths_and_counts_are_varints (properties.rs) |
| T-004 | T-004 EARS: trailing bytes -> TrailingBytes; error.rs defines at = first trailing byte and remaining = their count | killed | trailing_bytes_rejected (properties.rs) |
| T-004 | T-004 oversize-prefix EARS / error.rs needed (mutants: Decoder::length checks len <= input.len() or len <= remaining + 1) | killed | overrun_reports_bytes_still_needed (properties.rs) |
| T-004 | T-004 EARS: a non-minimal second element of a Vec<u64> -> NonMinimalVarint at offset 2 (mutant: at self.pos - 2) | killed | varint_rejects_non_canonical (properties.rs) |
| T-004 | Spec Decoder requirements, Bounded: no decoder reserves memory from a declared count (mutant: BTreeSet / BTreeMap decode buffers into Vec::with_capacity(count)) | killed | hostile_hash_count_reserves_nothing (crates/codec/tests/alloc.rs) |
| T-004 | T-004 envelope: a format version other than 1 -> UnsupportedFormatVersion (mutant: kind checked before version) | killed | envelope_rejects_unknown_version_and_kind (properties.rs) |
| T-004 | T-004 allocation bounds / spec Bounded (mutants: A = Vec decode reserves count - 1 after the first element; B = with_capacity(count) when size_of::<T>() <= 8) | killed | hostile_hash_count_reserves_nothing, hostile_pair_count_reserves_nothing, hostile_small_element_count_reserves_nothing (new), valid_vec_grows_as_it_decodes (alloc.rs) |
| T-004 | T-004 allocation bounds: UnexpectedEnd at the prefix with needed = shortfall (mutants a: len <= input.len(); b: len <= remaining + 1; c: needed capped at u32::MAX) | killed | overrun_reports_bytes_still_needed; oversize_prefixes_rejected_without_allocation (properties.rs) |
| T-004 | INT-0006 AC-5 totality and the T-004 round trip for String/Bytes (W1: Decoder::str panics for len > 255; W2: Encoder::bytes writes the length as a byte) | killed | lengths_and_counts_are_varints (new); round_trip_string, mutants_string (.{0,300}); round_trip_bytes, mutants_bytes (0..400) (properties.rs) |
| T-004 | T-004 totality: 100,000 random byte strings of lengths 0 to 256 never panic (mutant: decode_envelope panics on inputs of 256+ bytes) | killed | decode_is_total_on_random_bytes (0..=256); envelope_rejects_unknown_version_and_kind (300-byte blob); round_trip_envelope, mutants_envelope (bodies 0..400) (properties.rs) |
| T-004 | T-004 identity: object_id(alg, envelope) equals alg.hash(envelope) over the whole envelope (mutant: hashes only the first 64 bytes) | killed | object_id_hashes_whole_envelope (properties.rs) |
| T-004 | T-004 envelope: kinds 0x02 to 0x05 -> ReservedObjectKind, distinct from unknown (mutant: reported as UnknownObjectKind when the body has 2+ bytes) | killed | envelope_rejects_unknown_version_and_kind (properties.rs) |
| T-004 | T-004 envelope per the locked test plan ('a one-byte input -> UnexpectedEnd', code-review codec-4) and the UnexpectedEnd contract (mutants a: needed 1; b: at 0; c: version checked before length) | killed | envelope_rejects_unknown_version_and_kind (properties.rs) |
| T-004 | Decoder::take larger than the remaining input -> UnexpectedEnd with needed = excess (mutant: if self.pos + len > self.input.len()) | killed | overrun_reports_bytes_still_needed (properties.rs) |
| T-004 | Spec Object kinds / INT-0006: reserved kinds are rejected with a distinct reserved-kind error (mutant: the ReservedObjectKind message reads 'unknown object kind 0x..') | killed | envelope_rejects_unknown_version_and_kind (properties.rs) |
| T-004 | unit-tests.md T-004 allocation row cites valid_vec_grows_as_it_decodes (mutant: Vec decode uses Vec::with_capacity(count)) | already killed | hostile_hash_count_reserves_nothing, hostile_pair_count_reserves_nothing (already at HEAD); valid_vec_grows_as_it_decodes now also kills it (alloc.rs) |

Verifier: 33 mutants re-applied, 33 caught.

### Golden corpus and spec consistency (T-005)

| Task | Finding | Result | Test that now fails |
|------|---------|--------|---------------------|
| T-005 | T-005 EARS 1 and EARS 4: byte-for-byte encoding; a differing existing file fails (encoder-only changes, equal and different lengths) | killed | golden_encoder_change_fails, golden_encoder_change_fails_in_create_mode (changed_encoder now changes four cases) |
| T-005 | T-005 EARS 7: every row of the Hash algorithms and Object kinds tables equals the code registries | killed | spec_registries_match_code (via the strict rows() in spec.rs) |
| T-005 | T-005 EARS 6: every golden file equals the Worked examples hex for its name, and every corpus name has a row | killed | spec_examples_match_golden_files |
| T-005 | T-005 EARS 1: each named corpus value encodes byte for byte to tests/golden/v1/<name>.bin (u32-max deleted from the corpus, or made a u64) | killed | every_golden_file_is_a_corpus_case (new), golden_corpus_has_the_spec_types (new) |
| T-005 | T-005 EARS 1, second half: decoding the blob-envelope file yields the corpus value | killed | golden_undecodable_file_fails (extended to four sub-cases) |
| T-005 | T-005 EARS 1, second half: decoding the file yields the corpus value (typed cases decode the whole file) | killed | golden_undecodable_file_fails (sub-case uvarint-0 = 00 00) |
| T-005 | T-005 EARS 5: create_mode returns true exactly for Some("1") | killed | create_mode_is_exactly_one |
| T-005 | T-005 EARS 4: with create mode on, the harness writes the missing files (missing golden directory) | killed | golden_create_mode_makes_a_missing_directory (new) |
| T-005 | T-005 EARS 2 with the T-005 Notes: PERI_GOLDEN_CREATE is read once through create_mode, so a plain run has create mode off | killed | golden_v1_corpus_matches |
| T-005 | T-005 EARS 6, second half: every corpus name has a row (record credits spec_examples_match_golden_files alone) | already killed | golden_v1_corpus_matches (already); now also every_golden_file_is_a_corpus_case and golden_corpus_has_the_spec_types |

Verifier: 23 mutants re-applied, 23 caught.

### `book` (T-006)

| Task | Finding | Result | Test that now fails |
|------|---------|--------|---------------------|
| T-006 | T-006 EARS 1 (unit-tests.md row 1): `cargo book` at the repository root SHALL print one line per docs/intents/INT-*.md chapter with its criterion count. | killed | crates/book/tests/cli.rs cli_exit_codes |
| T-006 | T-006 EARS 11 / INT-0003 AC-9 (row 11): an argument that starts with `-` SHALL exit 2 with a non-empty stderr message and empty stdout. | killed | crates/book/tests/cli.rs cli_exit_codes |
| T-006 | T-006 EARS 2 as `cargo book` prints it (row 11): every violating line is reported with file, line and rule. | killed | crates/book/tests/cli.rs cli_exit_codes |
| T-006 | T-006 EARS 2, rule `duplicate`: a repeated AC-n SHALL be one violation at its 1-based line. | killed | crates/book/tests/book.rs typed_line_violations_are_reported |
| T-006 | T-006 EARS 2, rule `claim`: no ` — ` separator. | killed | crates/book/tests/book.rs typed_line_violations_are_reported |
| T-006 | T-006 EARS 2, rules `polarity` and `modality` (an unknown word). | killed | crates/book/tests/book.rs typed_line_violations_are_reported |
| T-006 | T-006 EARS 2, rule `number`: an AC-n that is non-numeric, unterminated or zero. | killed | crates/book/tests/book.rs numbers_are_plain_digits, typed_line_violations_are_reported |
| T-006 | T-006 EARS 2, rule `detail`: it applies to a criterion's own indented detail items. | killed | crates/book/tests/book.rs quotes_join_a_claim_and_code_does_not |
| T-006 | T-006 EARS 2 in an undeclared Book (port rule): a tagged but malformed item is still checked. | killed | crates/book/tests/book.rs typed_line_violations_are_reported |
| T-006 | T-006 EARS 6: an untyped top-level item in an undeclared Book SHALL be read as `necessary` `asserted`. | killed | crates/book/tests/book.rs undeclared_book_lifts_untagged_labels |
| T-006 | T-006 EARS 3: a declared Book's chapter with no `## Acceptance criteria` section SHALL report `missing`. | killed | crates/book/tests/book.rs missing_criteria_are_reported |
| T-006 | T-006 EARS 5: a wrapped typed claim SHALL be the trimmed lines joined by single spaces. | killed | crates/book/tests/book.rs quotes_join_a_claim_and_code_does_not |
| T-006 | T-006 EARS 1: one counted line per docs/intents/INT-*.md chapter; the checker reads every `.md` file except README.md. | killed | crates/book/tests/book.rs declared_book_rejects_untyped_items |
| T-006 | Outside the literal EARS text: INT-0002's grammar (`necessary` marks a falsifier) as the typed-line parser reads it. | killed | crates/book/tests/book.rs typed_criteria_parse_with_details |
| T-006 | unit-tests.md, T-006 row 'a typed line breaking one rule': `numbers_are_plain_digits` (`+1`, `02`, `0`, overflow) | killed | crates/book/tests/book.rs numbers_are_plain_digits |
| T-006 | unit-tests.md, T-006 row 'a typed line breaking one rule': 'Every violation test checks the path, and that CRLF and a lone CR give the same violations' | killed | crates/book/tests/book.rs printed_violations_name_file_line_and_rule |

Verifier: 41 mutants re-applied, 41 caught.

## Second pass

The verifiers' issues other than surviving mutants, and what became of each.

| Severity | Where | Issue | Disposition |
|----------|-------|-------|-------------|
| blocking | `crates/xtask/src/policy.rs` | The `spec` rule's new hand-written reader, the fix for the T-002 defect, left holes of the same kind: ten single-token mutants passed every test, each letting `spec` pass a document in which GitHub shows no such heading or row. | fixed in `0a9fd40`: the rule reads the spec with pulldown-cmark (with tables), the parser the Book checker already uses, so a heading or row in code, a comment or other HTML is none. A heading in a list item or a quote ends the section above it and names none; a table counts only at the top level. The ten documents are cases in `policy_reads_the_spec_as_markdown`, with more for nested headings and tables, a first cell that is not code, a heading's code spans and lines, and lone-CR line endings. 15 mutants of the new reader each fail a test |
| should-fix | `crates/xtask/src/policy.rs` | Where the hand-written reader stopped (a fence in a list item, a `<details>` block), `spec` reported every later section missing, blaming the wrong thing. | fixed in `0a9fd40`: pulldown-cmark reads the whole file |
| should-fix | `crates/codec/tests/spec.rs` | The spec test's line scan checked for HTML and fences only inside the target section, so HTML or a fence opened earlier could hide a whole table that the test still read. | fixed in `0a9fd40`: the test reads the spec with pulldown-cmark and reports what keeps it from reading a section whole. `spec_tables_are_read_as_shown` runs the reader on small documents: tables hidden in comments, fences and indented code, sections that are missing or repeated, two tables, other columns, and rows that do not start with an id. 10 mutants of the reader each fail a test |
| blocking | `crates/codec/tests/golden.rs` | `every_golden_file_is_a_corpus_case` compared the golden directory with the corpus both ways, so in create mode it listed the files before `golden_v1_corpus_matches` wrote a missing one and failed: the documented way to add a file broke. | fixed in `0a9fd40`: it checks only that each file has a case; `golden_v1_corpus_matches` already shows that each case has a file |
| should-fix | `crates/xtask/src/ci.rs` | `hostile_variables_never_reach_a_step` looked for the probe's own result line, which quiet cargo output (`CARGO_TERM_QUIET`, or `term.quiet` in a Cargo config) prints as a dot, so the test failed on such a machine though every step passed. | fixed in `0a9fd40`: the child runs with `CARGO_TERM_QUIET=false`; checked with `CARGO_TERM_QUIET=true` set |
| should-fix | `crates/xtask/src/main.rs` | `main_passes_every_argument_to_cli` said `main` hands `cli` every argument, but showed only that `main` drops none after the command name: mutants that keep the program name or skip two passed. | fixed in `0a9fd40`: the doc says what the test shows. That `main` skips exactly the program name and exits 0 for a valid command is shown by every `cargo policy` and `cargo ci` run in the integration and E2E records; the known-limit row in [code-review.md](code-review.md) says so |
| should-fix | the test records | The records described the suite before the sweep. | fixed: [unit-tests.md](unit-tests.md) describes the tests at the tested head |
