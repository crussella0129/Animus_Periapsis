# Sprint 0 Build Plan

## Intents
- [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — state: planned; acceptance criteria covered: AC-2 (0 grammar violations, including the `missing` rule), AC-3 (status consistency, by proxy), AC-4 (port rule over the Animus_Sprint_Loops corpus). AC-1 and AC-5 are sign-offs outside this sprint.
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: planned; acceptance criteria covered:
  - locally: AC-1, AC-6, AC-8, AC-9 and AC-10, plus the local half of AC-2 to AC-4 (pinned, locked, deny-clean, fmt- and clippy-clean on Windows and Linux) and supporting evidence for AC-5 (the design-credit list);
  - carried forward: the GitHub Actions half of AC-2 to AC-4. It can only run at the checkpoint, after this sprint closes, so the carried-forward task `T-009 (backlog)` below records it;
  - owner actions: AC-5's sign-off (at the checkpoint) and AC-7 (crates.io reservation).
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: planned; acceptance criteria covered: AC-1 for this sprint's code (Windows host checkout, WSL ext4 clone, fresh Windows clone with `core.autocrlf=true`). AC-2 to AC-6 need paths, a working copy, storage or the INT-0007/INT-0024 scenarios.
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: planned; acceptance criteria covered: AC-4 (no randomly seeded hash collections, enforced by Clippy). AC-1 to AC-3 and AC-5 arrive with the first storage code.
- [INT-0006](../../../intents/INT-0006-format-v1.md) — state: planned; acceptance criteria covered: AC-1, AC-2, AC-4 and AC-5; AC-3's local half (the canonical suite on Windows and Linux), with its GitHub-matrix half carried forward in the same backlog task as INT-0003's; and AC-6 up to the owner's sign-off of the spec at the checkpoint, which is also when Sprint 0's encodings freeze. AC-7 waits for signatures (INT-0016).
- Carried-forward task: Loop appends `T-009 (backlog) [intent: INT-0001, INT-0002, INT-0003, INT-0006]` and links it from those chapters' Work evidence. Its job is to record the Sprint 0 checkpoint's GitHub Actions run (head SHA, both matrix legs with their golden and `cargo ci` results, the `deny` job, runner OS and `rustc`) and the owner's checkpoint sign-offs (INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6).
- Context, not advanced:
  - [INT-0001](../../../intents/INT-0001-periapsis-charter.md) stays `proposed` until the owner signs off its decomposition (its AC-1). Its Consequences now record that figures quoted from the R1 to R5 prototypes are targets until each owning intent's sprint re-measures them; Sprint 0 quotes none as evidence.
  - [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) holds the product form of the owner's naming and flag rule.
  - [INT-0008](../../../intents/INT-0008-compact-history-storage.md) and [INT-0020](../../../intents/INT-0020-intent-calculus.md) were reworded to agree with INT-0006 and INT-0002.

## Schema Tree
- Sprint Goal: Phase-0 Rust foundation that holds on Windows and Linux
  - Baseline
    - T-001: Workspace, guardrails, policy checker and credits
  - Format v1, spec first
    - T-002: Format v1 specification
    - T-003: Algorithm-tagged hashes in the hash crate
    - T-004: Canonical binary encoding in the codec crate
    - T-005: Golden corpus and spec consistency
  - Typed Book
    - T-006: Typed-acceptance validator in the book crate
  - Verification surface
    - T-007: Canonical suite runner and CI workflow
  - Documentation
    - T-008: README and documentation checks

## Execution Sequence

### T-001: Workspace, guardrails, policy checker and credits
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`, `clippy.toml`, `.gitattributes`, `.cargo/config.toml`, `docs/prior-art.md`, `crates/xtask/Cargo.toml`, `crates/xtask/src/main.rs`, `crates/xtask/src/policy.rs`
- **Depends on:** (none)
- **Acceptance criterion:** INT-0003 AC-1 (no `unsafe`), AC-2 (pinned and locked, local half), AC-3 (deny configuration and a clean local run), AC-6 (dependencies credited, with licence and C-build status, pure or justified), AC-8 (single-word names, with every member unpublished as INT-0003's boundary requires), AC-9 (flagless `cargo policy`); INT-0005 AC-4 (no randomly seeded hash collections)
- **Success criterion (EARS):**
  - **WHEN** `cargo policy` runs at the repository root after this task, **THEN** it **SHALL** exit 0 and print `PASS <rule>` for each of `lints`, `unsafe`, `toolchain`, `lockfile`, `publish`, `deps`, `credits`, `clippy` and `names`.
  - **WHEN** the rules run against the clean fixture workspace that the negative tests start from, **THEN** every rule **SHALL** pass, so each negative test fails for its one planted defect only.
  - **WHEN** a member manifest does not resolve to `publish = false` (neither `publish = false` nor `publish.workspace = true` with the workspace value `false`), **THEN** policy **SHALL** fail with rule `publish` and name the crate, until INT-0003 records a publishable crate.
  - **WHEN** a member crate's `Cargo.toml` lacks `[lints]` with `workspace = true`, **THEN** policy **SHALL** fail with rule `lints` and name the crate.
  - **WHEN** a member crate root (`src/lib.rs` or `src/main.rs`) lacks `#![forbid(unsafe_code)]`, **THEN** policy **SHALL** fail with rule `unsafe` and name the file.
  - **WHEN** `rust-toolchain.toml` names a channel that is not an exact `X.Y.Z` release, **THEN** policy **SHALL** fail with rule `toolchain`.
  - **WHEN** `Cargo.lock` is absent, **THEN** policy **SHALL** fail with rule `lockfile`.
  - **WHEN** a member manifest declares a normal, dev or build dependency, or a `[dependencies.<name>]` table, without `workspace = true`, **THEN** policy **SHALL** fail with rule `deps` and name the crate and the dependency.
  - **WHEN** `docs/prior-art.md` is missing; or a non-path entry of `[workspace.dependencies]` lacks a dependency-table row with a non-empty licence cell and a C-build cell that starts with `yes` or `no`; or a `yes` row's "how Periapsis builds it" cell neither names a feature, as `features = ["<name>"]`, that the matching `[workspace.dependencies]` entry enables, nor starts with `C allowed:` followed by a justification, **THEN** policy **SHALL** fail with rule `credits` and name the dependency or the missing file.
  - **WHEN** `clippy.toml` does not list both `std::collections::HashMap` and `std::collections::HashSet` under `disallowed-types`, **THEN** policy **SHALL** fail with rule `clippy`.
  - **WHEN** a member crate uses `std::collections::HashMap` or `HashSet`, **THEN** `cargo clippy --workspace --all-targets -- -D warnings` **SHALL** fail on `disallowed_types`.
  - **WHEN** any workspace package's name does not match `[a-z][a-z0-9]*`, **THEN** policy **SHALL** fail with rule `names` and name the package.
  - **WHEN** the policy command receives any argument besides its command name, including any argument starting with `-`, **THEN** it **SHALL** exit 2 and write a usage message to stderr.
  - **WHEN** `cargo deny check` runs at this task's boundary, **THEN** advisories, bans, licenses and sources **SHALL** all report ok, and the output **SHALL** be recorded with the task's completion entry.
- **Notes:**
  - Workspace: resolver 3, edition 2024, `rust-version = "1.99"`, `license = "GPL-3.0-only"`, `publish = false`, `[workspace.lints.rust] unsafe_code = "forbid"`. The toolchain is pinned to 1.99.1 with rustfmt and clippy.
  - `.gitattributes` gets `* text=auto eol=lf`; the index is already all-LF, so nothing renormalises. The `*.bin binary` line and the `golden` rule arrive with the golden files in T-005.
  - `.cargo/config.toml` defines the aliases `ci`, `policy` (both `run --locked --quiet --package xtask -- <command>`) and `book` (`run --locked --quiet --package book --`); `book` and `ci` start working in T-006 and T-007.
  - `deny.toml` starts from `cargo deny init` (cargo-deny 0.20.2), with a GPL-3.0-compatible licence allowlist and unpublished workspace crates ignored.
  - `docs/prior-art.md` credits the designs the Book's intents adopt or evaluate, with licences where they could be verified; its dependency table starts empty and each task that adds a dependency adds its row.
  - The command-line front end is a function from arguments to (exit code, stdout, stderr), so its tests need no subprocess. `xtask` has no `tests/` directory, so `cargo test` never rebuilds the `xtask` binary while `cargo ci` runs it.
  - Policy rules are pure functions over a root path, tested on temporary fixture trees; `cargo policy` must pass at every task boundary.

### T-002: Format v1 specification
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** `docs/spec/format-v1.md`, `crates/xtask/src/policy.rs`
- **Depends on:** T-001
- **Acceptance criterion:** INT-0006 AC-6 (each encoding specified before its implementation lands)
- **Success criterion (EARS):**
  - **WHEN** `cargo policy` runs after this task, **THEN** a new rule `spec` **SHALL** pass. It requires that `docs/spec/format-v1.md` exists; that it has the sections Conventions, Primitives, Canonical order, Hash algorithms, Objects, Object kinds, Object identity, Worked examples, Signatures (reserved), Decoder requirements and Change control; that the Hash algorithms and Object kinds sections each hold at least one ``| `0x..` |`` registry row with a status column; and that Worked examples holds at least one ``| `name` | … | `hex` |`` row.
  - **WHEN** the spec file is missing, a required section is absent, or a registry or worked-examples table has no rows, **THEN** policy **SHALL** fail with rule `spec` and name what is missing.
- **Notes:** The spec is the source for T-003, T-004 and T-005. It defines:
  - the hash registry (0x01 `blake3`, 32 bytes, default; 0x02 `sha256`, 32 bytes, supported), the binary form `id ‖ digest` and the text form `<name>:<lowercase hex>`;
  - `uvarint` (minimal LEB128, at most 10 bytes, tenth byte ≤ `01`), `u32`, `bool`, `bytes`, `string`, `option`, `seq`, `set`, `map` and `pair`, with canonical key order per type;
  - the envelope `01 ‖ kind ‖ body` and the kind registry: 0x01 `blob` defined, with a raw-content body; 0x02 `changeset`, 0x03 `operation`, 0x04 `view` and 0x05 `event` reserved and rejected as reserved; other ids rejected as unknown;
  - object identity as the hash of the whole envelope, the reserved signature-payload prefix, the decoder requirements, and staged freezing at the owner's sign-off;
  - **worked examples:** the exact hex of every golden corpus value, derived by hand from the rules above.

### T-003: Algorithm-tagged hashes in the hash crate
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** `Cargo.toml`, `Cargo.lock`, `deny.toml`, `docs/prior-art.md`, `crates/hash/Cargo.toml`, `crates/hash/src/lib.rs`, `crates/hash/tests/vectors.rs`
- **Depends on:** T-002
- **Acceptance criterion:** INT-0006 AC-2 (hash agility) and AC-4 (BLAKE3 official vectors); INT-0003 AC-3 and AC-6 for the dependencies this task introduces (`blake3`, `sha2`, and `proptest` for the workspace's property tests)
- **Success criterion (EARS):**
  - **WHEN** `HashAlg::Blake3.hash(input)` runs for each of the 35 official BLAKE3 test-vector inputs (lengths 0 to 102,400, byte `i` = `i mod 251`), **THEN** it **SHALL** return the first 32 bytes of that vector's `hash` output.
  - **WHEN** `HashAlg::Sha256.hash` runs on `""` and `"abc"`, **THEN** it **SHALL** return the FIPS 180-2 digests `e3b0c442…b855` and `ba7816bf…15ad`.
  - **WHEN** `HashAlg::from_id(id)` runs for every `id` in 0..=255, **THEN** it **SHALL** succeed exactly for 0x01 and 0x02 and return `UnknownAlgorithm(id)` otherwise.
  - **WHEN** the public hash registry is read, **THEN** `HashAlg::ALL` **SHALL** be exactly Blake3 (0x01, `blake3`, 32 bytes) and Sha256 (0x02, `sha256`, 32 bytes), in that order, and `HashAlg::DEFAULT` **SHALL** be Blake3.
  - **WHEN** `HashRef::new(alg, digest)` receives a digest whose length differs from `alg.digest_len()`, **THEN** it **SHALL** return `DigestLength { expected, actual }`.
  - **WHEN** two `HashRef`s hold identical digest bytes under different algorithms, **THEN** they **SHALL** compare unequal and order by algorithm id first.
  - **WHEN** a `HashRef` goes to binary `[id][digest]` and back, **THEN** it **SHALL** equal the original; **WHEN** the binary input has an unknown id, a truncated digest or trailing bytes, **THEN** parsing **SHALL** fail.
  - **WHEN** a `HashRef` is rendered as text, **THEN** it **SHALL** read `<name>:<lowercase hex>` and parse back to itself; **WHEN** text has uppercase hex, an unknown name, odd or wrong-length hex, or no `:`, **THEN** parsing **SHALL** fail.
  - **WHEN** input is fed to `Hasher::update` in arbitrary chunks, **THEN** `finalize` **SHALL** equal the one-shot hash for both algorithms.
  - **WHEN** `cargo tree --workspace --target all -e normal,build -i cc --depth 1` runs after this task, **THEN** the direct dependents of `cc` it lists **SHALL** be exactly the third-party crates with C-build `yes` in prior-art (here `blake3`); **WHEN** `cargo tree --workspace -e features -i blake3` runs, **THEN** it **SHALL** show `blake3 feature "pure"`, so the pure build is checked on the resolved graph and not only in prose.
  - **WHEN** `cargo policy` and `cargo deny check` run at this task's boundary, **THEN** both **SHALL** pass with `blake3`, `sha2` and `proptest` credited, and the deny output **SHALL** be recorded with the completion entry.
- **Notes:**
  - Dependencies: `blake3 = { version = "1.8.7", default-features = false, features = ["std", "pure"] }`; `sha2 = { version = "0.11.1", default-features = false }`; dev-dependency `proptest = { version = "1.12.0", default-features = false, features = ["std"] }`, which T-004 and T-006 reuse.
  - sha2 0.11.1 was verified with `cargo info` on 2026-10-31: MIT OR Apache-2.0, MSRV 1.88, features `alloc`/`oid`/`zeroize`, no assembly feature.
  - Text names are `blake3` and `sha256`. The in-memory digest is a fixed `[u8; 32]`; the wire form is what the spec freezes.
  - The vectors come from BLAKE3-team/BLAKE3 `test_vectors.json` (CC0-1.0 OR Apache-2.0); this task adds their Test-data row to prior-art.

### T-004: Canonical binary encoding in the codec crate
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** `Cargo.toml`, `Cargo.lock`, `crates/codec/Cargo.toml`, `crates/codec/src/lib.rs`, `crates/codec/src/error.rs`, `crates/codec/src/wire.rs`, `crates/codec/src/envelope.rs`, `crates/codec/tests/properties.rs`, `crates/codec/tests/mutants.rs`
- **Depends on:** T-003
- **Acceptance criterion:** INT-0006 AC-1 (canonical round trip) and AC-5 (total decoders)
- **Success criterion (EARS):**
  - **WHEN** any `u64` is encoded, **THEN** the encoder **SHALL** emit minimal unsigned LEB128 of at most 10 bytes: 0 → `00`, 127 → `7f`, 128 → `80 01`, 300 → `ac 02`, `u64::MAX` → `ff ff ff ff ff ff ff ff ff 01`.
  - **WHEN** a `Vec<u64>` whose second element is `80 00`, an 11-byte varint, or a 10-byte varint with tenth byte `02` is decoded, **THEN** decoding **SHALL** fail with `NonMinimalVarint` or `VarintOverflow` at offset 2, where that element starts.
  - **WHEN** a `u32` is decoded from `80 80 80 80 10` (2³²), **THEN** decoding **SHALL** fail with `IntegerOutOfRange`; **WHEN** it is decoded from `ff ff ff ff 0f` (2³² − 1), **THEN** it **SHALL** succeed.
  - **WHEN** the second element of a `Vec<bool>` is `02`, of a `Vec<Option<u64>>` has tag `02`, or of a `Vec<String>` holds bytes that are not UTF-8, **THEN** decoding **SHALL** fail with `InvalidBool`, `InvalidOptionTag` or `InvalidUtf8`. The offset is where the bad element starts, except that `InvalidUtf8` carries the offset of the string's first content byte, after its length prefix.
  - **WHEN** a byte-string or string length, or the element count of a `Vec`, `BTreeSet` or `BTreeMap`, is the ten-byte varint for `u64::MAX` followed by fewer bytes, as the second element of a `Vec` whose first element takes one byte, **THEN** decoding **SHALL** fail with `UnexpectedEnd { at: 2 }`, the prefix's offset, without attempting the allocation; a pre-allocation of that size would panic and fail the test.
  - **WHEN** a map or set encoding has keys that are not strictly increasing in the key type's canonical order, **THEN** decoding **SHALL** fail with `UnsortedOrDuplicateKey` at the offending key's offset.
  - **WHEN** `from_slice` decodes input that has bytes left after the value, **THEN** it **SHALL** fail with `TrailingBytes`.
  - **WHEN** a property test generates 10,000 values of a supported type, **THEN** decoding each value's encoding **SHALL** give the value back. This holds separately for each of u64, u32, bool, String, Bytes, HashRef, Option, Vec, BTreeSet, BTreeMap, 2-tuples, nested maps and blob envelopes with arbitrary bodies, each with its own 10,000 cases.
  - **WHEN** a valid encoding of any of those types is mutated at random (byte flip, insertion or deletion; 10,000 cases per type) and the mutant still decodes, **THEN** re-encoding the decoded value **SHALL** reproduce the mutant byte for byte.
  - **WHEN** every single-byte substitution, insertion and deletion of a fixed value of each type is decoded, **THEN** each mutant that decodes **SHALL** re-encode to itself, and at least one mutant per type **SHALL** decode.
  - **WHEN** 100,000 random byte strings (lengths 0 to 256) are each decoded as every one of those types and as an envelope, **THEN** every call **SHALL** return `Ok` or `Err` without panicking.
  - **WHEN** an envelope is decoded, **THEN** a format version other than 1 **SHALL** fail with `UnsupportedFormatVersion`, kinds 0x02 to 0x05 **SHALL** fail with `ReservedObjectKind`, and kind 0x00 or 0x06 to 0xff **SHALL** fail with `UnknownObjectKind`; **WHEN** `object_id(alg, envelope)` is computed, **THEN** it **SHALL** equal `alg.hash(envelope)` over the whole envelope.
  - **WHEN** the public kind registry is read, **THEN** `ObjectKind::ALL` **SHALL** be exactly blob (0x01, `blob`), and `RESERVED_KINDS` **SHALL** be exactly (0x02, `changeset`), (0x03, `operation`), (0x04, `view`) and (0x05, `event`).
- **Notes:** No serde and no new third-party dependency. `proptest` comes from T-003, and the root manifest gains the `hash` path entry. Canonical key order is Rust `Ord` per key type: strings and bytes lexicographic by raw content, integers numeric, `HashRef` by (id, digest).

### T-005: Golden corpus and spec consistency
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** `.gitattributes`, `crates/xtask/src/policy.rs`, `crates/codec/tests/golden.rs`, `crates/codec/tests/golden/v1/*.bin`, `crates/codec/tests/spec.rs`
- **Depends on:** T-004
- **Acceptance criterion:** INT-0006 AC-3 (golden files, identical on Windows and Linux) and AC-6 (code and files agree with the spec); INT-0004 AC-1 (identical bytes on both platforms)
- **Success criterion (EARS):**
  - **WHEN** the golden test runs, **THEN** each named corpus value **SHALL** encode byte for byte to `tests/golden/v1/<name>.bin`, and decoding that file **SHALL** yield the corpus value.
  - **WHEN** the golden harness runs with create mode off against a directory where a file is missing, **THEN** it **SHALL** fail and name the file.
  - **WHEN** it runs with create mode off and an existing file has one byte changed, **THEN** it **SHALL** fail and name that file.
  - **WHEN** it runs with create mode on, **THEN** it **SHALL** write only missing files, and any existing file that differs **SHALL** be left unchanged and fail the run, naming that file.
  - **WHEN** `create_mode` is given the value of `PERI_GOLDEN_CREATE`, **THEN** it **SHALL** return true exactly for `Some("1")` (not for `0`, `true`, an empty value or unset).
  - **WHEN** the spec test reads the Worked examples table in `docs/spec/format-v1.md`, **THEN** every golden file's bytes **SHALL** equal the hex the table gives for that name, and every corpus name **SHALL** have a row.
  - **WHEN** the spec test parses the Hash algorithms and Object kinds tables, **THEN** every row — (id, name, digest bytes, status) and (id, kind, status) — **SHALL** equal the code registries (`HashAlg::ALL` with `HashAlg::DEFAULT` as `default`; `ObjectKind::ALL` as `defined`; `RESERVED_KINDS` as `reserved`).
  - **WHEN** `.gitattributes` does not mark `*.bin` files as binary, **THEN** `cargo policy` **SHALL** fail with the new rule `golden`.
- **Notes:**
  - The corpus covers varint edge values, `u32::MAX`, both bools, empty and non-empty bytes, ASCII and multi-byte UTF-8 strings, both `HashRef` algorithms, `None`/`Some`, a sequence, a set, a map, a nested map, a pair and one blob envelope.
  - The spec's hex was written by hand from the rules, independently of the encoder, so the spec test is the encoder's negative control.
  - Freeze rule: once the owner signs off the spec at the checkpoint, files under `v1/` never change, and a changed encoding means a new format version.
  - `PERI_GOLDEN_CREATE` is read once at test entry through `create_mode`. Tests never set it, because edition 2024 makes `set_var` unsafe; they call the harness with the mode they need.

### T-006: Typed-acceptance validator in the book crate
- **Intent:** [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** `Cargo.toml`, `Cargo.lock`, `docs/prior-art.md`, `docs/.sprint-loop-book`, `crates/book/Cargo.toml`, `crates/book/src/lib.rs`, `crates/book/src/main.rs`, `crates/book/tests/book.rs`, `crates/book/tests/cli.rs`, `crates/book/tests/fixtures/sprint-loops/**`
- **Depends on:** T-001, T-003 (`proptest`)
- **Acceptance criterion:** INT-0002 AC-2 (0 violations, at least one criterion per chapter), AC-3 (status consistency), AC-4 (port rule); INT-0003 AC-9 (flagless `cargo book`)
- **Success criterion (EARS):**
  - **WHEN** `cargo book` runs at the repository root with no arguments, **THEN** it **SHALL** check `docs/`, select the strict rule from `typed-acceptance: 1` in `docs/.sprint-loop-book`, print one line per `docs/intents/INT-*.md` chapter with its criterion count, find at least one criterion in each, report 0 violations and exit 0.
  - **WHEN** a typed line in `## Acceptance criteria` breaks a rule, **THEN** the check **SHALL** report exactly one violation naming the file, the 1-based line and the rule:
    - `polarity` — unknown or missing polarity;
    - `modality` — unknown or missing modality;
    - `claim` — no ` — ` separator, or an empty claim;
    - `number` — a non-numeric, unterminated or zero `AC-n`;
    - `duplicate` — a repeated `AC-n`;
    - `detail` — a detail key not allowed for the criterion's modality, or a detail without `:`.
  - **WHEN** a chapter of a declared Book has no `## Acceptance criteria` section, or an empty one, **THEN** the check **SHALL** report rule `missing`.
  - **WHEN** lines appear inside fenced code blocks (``` or ~~~) anywhere in a chapter, including a fenced `## Acceptance criteria` heading outside the real section, **THEN** the parser **SHALL** ignore them.
  - **WHEN** a typed claim wraps onto indented continuation lines, **THEN** the criterion's claim **SHALL** be the trimmed lines joined by single spaces.
  - **WHEN** an untyped top-level list item appears in the criteria section of a declared Book, **THEN** the check **SHALL** report rule `untyped`; **WHEN** the same item appears in an undeclared Book, **THEN** it **SHALL** be read as `necessary` `asserted`, with its wrapped continuation lines joined into the claim.
  - **WHEN** a chapter containing a wrapped criterion with a detail is converted to CRLF in memory, **THEN** the test **SHALL** confirm the converted text contains `\r\n`, and the parsed criteria **SHALL** equal those of the LF original.
  - **WHEN** the marker declares `typed-acceptance:` with any value other than `1`, or the Book directory has no `.sprint-loop-book` marker, **THEN** the check **SHALL** return an error rather than fall back to the port rule, and the binary **SHALL** exit 2 with a message on stderr.
  - **WHEN** `status` is evaluated for any non-empty list of criteria and any assignment of holds/fails/unknown outcomes (property test, 10,000 cases), **THEN** Certified and Refuted **SHALL NOT** both hold; Certified **SHALL** hold exactly when every criterion holds; Refuted **SHALL** hold exactly when some necessary criterion fails. **WHEN** the list is empty, **THEN** `status` **SHALL** return Undetermined.
  - **WHEN** the check runs over the fixture Book holding the vendored Animus_Sprint_Loops chapters `INT-0001-project-book.md` and `INT-0013-verification-integrity.md`, with no declaration, **THEN** it **SHALL** report 0 violations and read exactly 5 and 9 criteria, all `necessary` `asserted`. INT-0001's first claim **SHALL** be its two wrapped lines joined: "`docs/` validates under `check-book.sh`: exactly one `schema-version: 2` marker, unique `INT-NNNN` ids, legal states, and state-appropriate evidence."
  - **WHEN** the binary finds violations, **THEN** it **SHALL** exit 1; **WHEN** it finds none, **THEN** it **SHALL** exit 0; **WHEN** the Book directory is missing, an argument starts with `-`, or more than one argument is given, **THEN** it **SHALL** exit 2 with a non-empty message on stderr and nothing on stdout.
- **Notes:**
  - The binary is `book [BOOK_DIR]` (default `docs`). Checking is idempotent, so there are no flags; the Book's marker selects the rule. The root manifest gains the `book` path entry that T-007 uses.
  - The parser is line-based and std-only. Allowed detail keys per modality: formal — Obligation, Proxy; empirical — Threshold, Sample, Error; preference — Panel, Threshold, Error; judged — Rubric, Quorum; asserted — Signers; every modality — Evidence, Note.
  - Strength is not computed in Sprint 0 (INT-0002 Consequences).
  - The fixtures are copied unchanged from Animus_Sprint_Loops commit `216867f` with an MIT notice; this task adds their Test-data row to prior-art.
  - The `typed-acceptance: 1` marker line is ignored by the Sprint Loops helpers, which read only `schema-version` and `substrate-version`.

### T-007: Canonical suite runner and CI workflow
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- **Touches:** `crates/xtask/Cargo.toml`, `crates/xtask/src/main.rs`, `crates/xtask/src/ci.rs`, `.github/workflows/ci.yml`, `.github/dependabot.yml`, `Cargo.lock`
- **Depends on:** T-001, T-006
- **Acceptance criterion:** INT-0003 AC-2 and AC-4 (locked, fmt- and clippy-clean, locally on both platforms; the GitHub half is carried forward), AC-3 (the deny job in the workflow), AC-9 (flagless idempotent `cargo ci`); INT-0004 AC-1 (both platforms); INT-0002 AC-2 (Book check on every run)
- **Success criterion (EARS):**
  - **WHEN** `cargo ci` runs, **THEN** it **SHALL** execute exactly these steps in order — `fmt` (`cargo fmt --all --check`), `clippy` (`cargo clippy --workspace --all-targets --locked -- -D warnings`), `test` (`cargo test --workspace --locked`), `policy`, `book` — and print one `PASS <step>` or `FAIL <step>: <reason>` line per step.
  - **WHEN** two steps fail, including because a tool is not installed, **THEN** the runner **SHALL** still run every step, exit non-zero and name both failed steps in its summary.
  - **WHEN** `cargo ci` receives any argument besides its command name, including any argument starting with `-`, **THEN** it **SHALL** exit 2 and write a usage message to stderr.
  - **WHEN** the runner starts the `test` step, **THEN** it **SHALL** remove `PERI_GOLDEN_CREATE` from that child's environment, so a variable left set in the shell can never create golden files.
  - **WHEN** `cargo ci` runs on the Windows 11 host checkout, in a WSL Ubuntu ext4 clone and in a fresh Windows clone made with `core.autocrlf=true`, **THEN** every step **SHALL** print `PASS` and the runner **SHALL** exit 0, with each run's OS release and `rustc -Vv` recorded.
  - **WHEN** `cargo ci`, `cargo policy` and `cargo book` each run twice in a row, with no arguments, on a clean checkout, **THEN** every run **SHALL** exit 0, `cargo book` **SHALL** print one counted line per `docs/intents/INT-*.md` chapter, and `git status --porcelain` **SHALL** be empty after every run.
  - **WHEN** the workflow test reads `.github/workflows/ci.yml`, **THEN** it **SHALL** find:
    - push triggers for `dev` and `main`, and a `pull_request` trigger;
    - a job whose matrix holds `ubuntu-latest` and `windows-latest` and that runs `cargo ci`;
    - a `deny` job on `ubuntu-latest` that runs cargo-deny;
    - no `continue-on-error` and no `|| true`.
- **Notes:**
  - The `book` step runs in-process through the `book` library (`book.workspace = true`, from the path entry T-006 adds); policy runs in-process too.
  - cargo-deny stays its own command and CI job, as INT-0003 records. Dependabot gains a `cargo` ecosystem entry targeting `dev`; that is configuration, not a tested promise.
  - The GitHub Actions run starts only at the checkpoint, after the sprint closes. It is recorded by the carried-forward task `T-009 (backlog)` defined in the Intents section, as INT-0003 AC-2 to AC-4 and INT-0006 AC-3 evidence.

### T-008: README and documentation checks
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** `README.md`, `docs/prior-art.md`, `crates/xtask/src/main.rs`, `crates/xtask/src/docs.rs`
- **Depends on:** T-001 to T-007
- **Acceptance criterion:** INT-0003 AC-5 (supporting evidence for the owner's design-credit review) and AC-10 (README)
- **Success criterion (EARS):**
  - **WHEN** the design-credit test reads `docs/prior-art.md`, **THEN** the file **SHALL** name at least: Jujutsu, Mercurial, Sapling, Mononoke, Radicle, Git, JGit, Mergiraf, Spork, TigerBeetle, turmoil, madsim, Zuul, bors, SubmitQueue, Lean 4, comparator, nanoda, lean4export, Aeneas, Pijul, Darcs, Xet, Negentropy, zstd, Agent Trace, in-toto, DSSE, SLSA, Model Context Protocol, Scion, Fossil, Animus_Sprint_Loops, CubiKan, array-test and the BLAKE3 test vectors. Completeness beyond this list is the owner's AC-5 review.
  - **WHEN** the README test reads `README.md`, **THEN** it **SHALL** find:
    - the owner's two original paragraphs verbatim, compared against copies embedded in the test from `README.md` at `620f99c`;
    - a link to `docs/SUMMARY.md`;
    - the commands `cargo ci`, `cargo policy`, `cargo book` and `cargo deny check`;
    - the exact sentence "`cargo ci`, `cargo policy` and `cargo book` take no flags." (`cargo deny check` is a third-party tool, run exactly as shown).
- **Notes:** The README adds status, crate and command sections below the owner's text; INT-0031 is the Book form of the branding paragraph. `docs.rs` is a test-only module.
