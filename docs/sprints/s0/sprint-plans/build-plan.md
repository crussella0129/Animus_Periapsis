# Sprint 0 Build Plan

## Intents
- [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — state: planned; acceptance criteria covered: AC-2 (0 grammar violations, including the `missing` rule), AC-3 (status consistency, by proxy), AC-4 (port rule over the Animus_Sprint_Loops corpus). AC-1 and AC-5 are sign-offs outside this sprint.
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: planned; acceptance criteria covered: AC-1, AC-2, AC-3, AC-4, AC-6, AC-8, AC-9, plus supporting evidence for AC-5 (the design-credit list). AC-5 itself is the owner's review at the checkpoint, and AC-7 (crates.io reservation) is an owner action. AC-2 to AC-4 are claims about CI: they are proven only once the checkpoint's GitHub Actions run is recorded.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: planned; acceptance criteria covered: AC-1 for this sprint's code, including a fresh Windows clone with `core.autocrlf=true`. AC-2 to AC-6 need paths, a working copy, storage or Nighthawk access.
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: planned; acceptance criteria covered: AC-4 (no randomly seeded hash collections, enforced by Clippy). AC-1 to AC-3 and AC-5 arrive with the first storage code.
- [INT-0006](../../../intents/INT-0006-format-v1.md) — state: planned; acceptance criteria covered: AC-1 to AC-6. AC-7 waits for signatures (INT-0016).
- Context, not advanced: [INT-0001](../../../intents/INT-0001-periapsis-charter.md) stays `proposed` until the owner signs off its decomposition (its AC-1). [INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md) holds the product form of the owner's naming and flag rule; this sprint's tooling follows it through INT-0003 AC-9. [INT-0008](../../../intents/INT-0008-compact-history-storage.md) was reworded (AC-2, AC-5) to agree with INT-0006's object identity and staged freezing.

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
- **Acceptance criterion:** INT-0003 AC-1 (no `unsafe`), AC-2 (pinned and locked), AC-3 (deny configuration), AC-6 (dependencies credited with licence and C-build status), AC-8 (single-word crate names), AC-9 (flagless tooling); INT-0005 AC-4 (no randomly seeded hash collections)
- **Success criterion (EARS):**
  - **WHEN** `cargo policy` runs at the repository root after this task, **THEN** it **SHALL** exit 0 and print `PASS <rule>` for each of `lints`, `unsafe`, `toolchain`, `lockfile`, `golden`, `deps`, `credits`, `clippy` and `names`.
  - **WHEN** a member crate's `Cargo.toml` lacks `[lints]` with `workspace = true`, **THEN** policy **SHALL** fail with rule `lints` and name the crate.
  - **WHEN** a member crate root (`src/lib.rs` or `src/main.rs`) lacks `#![forbid(unsafe_code)]`, **THEN** policy **SHALL** fail with rule `unsafe` and name the file.
  - **WHEN** `rust-toolchain.toml` names a channel that is not an exact `X.Y.Z` release, **THEN** policy **SHALL** fail with rule `toolchain`.
  - **WHEN** `Cargo.lock` is absent, **THEN** policy **SHALL** fail with rule `lockfile`.
  - **WHEN** `.gitattributes` does not mark `*.bin` files as binary, **THEN** policy **SHALL** fail with rule `golden`.
  - **WHEN** a member manifest declares a normal, dev or build dependency without `workspace = true`, **THEN** policy **SHALL** fail with rule `deps` and name the crate and the dependency.
  - **WHEN** `docs/prior-art.md` is missing, or a non-path entry of `[workspace.dependencies]` has no row in its dependency table whose licence cell is non-empty and whose C-build cell starts with `yes` or `no`, **THEN** policy **SHALL** fail with rule `credits` and name the dependency or the missing file.
  - **WHEN** `clippy.toml` does not list both `std::collections::HashMap` and `std::collections::HashSet` under `disallowed-types`, **THEN** policy **SHALL** fail with rule `clippy`.
  - **WHEN** a workspace package name does not match `[a-z][a-z0-9]*`, **THEN** policy **SHALL** fail with rule `names` and name the package.
  - **WHEN** the policy command receives any argument besides its command name, including any argument starting with `-`, **THEN** it **SHALL** exit 2 with a usage message on stderr.
  - **WHEN** `cargo deny check` runs after this task, **THEN** advisories, bans, licenses and sources **SHALL** all report ok.
- **Notes:**
  - Workspace: resolver 3, edition 2024, `rust-version = "1.99"`, `license = "GPL-3.0-only"`, `publish = false`, `[workspace.lints.rust] unsafe_code = "forbid"`. The toolchain is pinned to 1.99.1 with rustfmt and clippy.
  - `.gitattributes`: `* text=auto eol=lf` and `*.bin binary`. The index is already all-LF, so no file renormalises.
  - `.cargo/config.toml` defines the aliases `ci`, `policy` (both `run --locked --quiet --package xtask -- <command>`) and `book` (`run --locked --quiet --package book --`). The `ci` and `book` aliases start working in T-007 and T-006.
  - `deny.toml` starts from `cargo deny init` (cargo-deny 0.20.2), with a GPL-3.0-compatible licence allowlist and unpublished workspace crates ignored. Every later task that adds a dependency updates it and re-runs `cargo deny check`.
  - `docs/prior-art.md` credits every system whose design a Book intent adopts or evaluates, and the owner's own projects; its dependency table starts empty. Each later task that adds a dependency or third-party test data adds its row in the same commit.
  - Policy rules are pure functions over a root path, so tests use temporary fixture trees. The workspace run must pass at every task boundary.

### T-002: Format v1 specification
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** `docs/spec/format-v1.md`, `crates/xtask/src/policy.rs`
- **Depends on:** T-001
- **Acceptance criterion:** INT-0006 AC-6 (each object type's encoding specified before its implementation lands)
- **Success criterion (EARS):**
  - **WHEN** `cargo policy` runs after this task, **THEN** a new rule `spec` **SHALL** pass. The rule requires that `docs/spec/format-v1.md` exists and has the sections Conventions, Primitives, Canonical order, Hash algorithms, Objects, Object kinds, Object identity, Signatures (reserved), Decoder requirements and Change control, and that the Hash algorithms and Object kinds sections each hold at least one registry row of the form ``| `0x..` |`` with a status column.
  - **WHEN** the spec file is missing, a required section is absent or a registry table has no rows, **THEN** policy **SHALL** fail with rule `spec` and name what is missing.
- **Notes:** The spec is the source for T-003 and T-004. It defines:
  - the hash registry (0x01 `blake3`, 32 bytes, default; 0x02 `sha256`, 32 bytes, supported) with the binary form `id ‖ digest` and the text form `<name>:<lowercase hex>`;
  - `uvarint` (minimal LEB128, at most 10 bytes, tenth byte ≤ `01`), `u32`, `bool`, `bytes`, `string`, `option`, `seq`, `set`, `map` and `pair`, with canonical key order per type;
  - the envelope `01 ‖ kind ‖ body`, and the kind registry (0x01 `blob` defined, raw-content body; 0x02 `changeset`, 0x03 `operation`, 0x04 `view` and 0x05 `event` reserved and rejected with `ReservedObjectKind`; other ids rejected with `UnknownObjectKind`);
  - object identity as the hash of the whole envelope, the reserved signature-payload prefix, the decoder requirements, and staged freezing as recorded in INT-0006.

### T-003: Algorithm-tagged hashes in the hash crate
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** `Cargo.toml`, `Cargo.lock`, `deny.toml`, `docs/prior-art.md`, `crates/hash/Cargo.toml`, `crates/hash/src/lib.rs`, `crates/hash/tests/vectors.rs`
- **Depends on:** T-002
- **Acceptance criterion:** INT-0006 AC-2 (hash agility) and AC-4 (BLAKE3 official vectors); INT-0003 AC-3 and AC-6 for the new dependencies
- **Success criterion (EARS):**
  - **WHEN** `HashAlg::Blake3.hash(input)` runs for each of the 35 official BLAKE3 test-vector inputs (lengths 0 to 102,400, byte `i` = `i mod 251`), **THEN** it **SHALL** return the first 32 bytes of that vector's `hash` output.
  - **WHEN** `HashAlg::Sha256.hash` runs on `""` and `"abc"`, **THEN** it **SHALL** return the FIPS 180-2 digests `e3b0c442…b855` and `ba7816bf…15ad`.
  - **WHEN** `HashAlg::from_id(id)` runs for every `id` in 0..=255, **THEN** it **SHALL** succeed exactly for 0x01 and 0x02 and return `UnknownAlgorithm(id)` otherwise.
  - **WHEN** `HashRef::new(alg, digest)` receives a digest whose length differs from `alg.digest_len()`, **THEN** it **SHALL** return `DigestLength { expected, actual }`.
  - **WHEN** two `HashRef`s hold identical digest bytes under different algorithms, **THEN** they **SHALL** compare unequal and order by algorithm id first.
  - **WHEN** a `HashRef` goes to binary `[id][digest]` and back, **THEN** it **SHALL** equal the original; **WHEN** the binary input has an unknown id, a truncated digest or trailing bytes, **THEN** parsing **SHALL** fail.
  - **WHEN** a `HashRef` is rendered as text, **THEN** it **SHALL** read `<name>:<lowercase hex>` and parse back to itself; **WHEN** text has uppercase hex, an unknown name, odd or wrong-length hex, or no `:`, **THEN** parsing **SHALL** fail.
  - **WHEN** input is fed to `Hasher::update` in arbitrary chunks, **THEN** `finalize` **SHALL** equal the one-shot hash for both algorithms.
  - **WHEN** `cargo policy` and `cargo deny check` run after this task, **THEN** both **SHALL** pass, with `blake3` and `sha2` in the prior-art dependency table.
- **Notes:**
  - Dependencies: `blake3 = { version = "1.8.7", default-features = false, features = ["std", "pure"] }` and `sha2 = { version = "0.11.1", default-features = false }`.
  - sha2 0.11.1 was verified with `cargo info` on 2026-10-31: MIT OR Apache-2.0, MSRV 1.88, features `alloc`/`oid`/`zeroize`, no assembly feature.
  - Text names are `blake3` and `sha256`. The in-memory digest is a fixed `[u8; 32]`; the wire form is what the spec freezes.
  - The vectors come from BLAKE3-team/BLAKE3 `test_vectors.json` (CC0-1.0 OR Apache-2.0); this task adds their Test-data row to prior-art.

### T-004: Canonical binary encoding in the codec crate
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** `Cargo.toml`, `Cargo.lock`, `deny.toml`, `docs/prior-art.md`, `crates/codec/Cargo.toml`, `crates/codec/src/lib.rs`, `crates/codec/src/error.rs`, `crates/codec/src/wire.rs`, `crates/codec/src/envelope.rs`, `crates/codec/tests/properties.rs`
- **Depends on:** T-003
- **Acceptance criterion:** INT-0006 AC-1 (canonical round trip) and AC-5 (total decoders); INT-0003 AC-3 and AC-6 for the new dev-dependency
- **Success criterion (EARS):**
  - **WHEN** any `u64` is encoded, **THEN** the encoder **SHALL** emit minimal unsigned LEB128 of at most 10 bytes: 0 → `00`, 127 → `7f`, 128 → `80 01`, 300 → `ac 02`, `u64::MAX` → `ff ff ff ff ff ff ff ff ff 01`.
  - **WHEN** the decoder reads a varint with a redundant zero final group (`80 00`), more than 10 bytes, or a tenth byte above `01`, **THEN** it **SHALL** fail with `NonMinimalVarint` or `VarintOverflow` at the varint's offset.
  - **WHEN** a `u32` is decoded from `80 80 80 80 10` (2³²), **THEN** decoding **SHALL** fail with `IntegerOutOfRange`; **WHEN** it is decoded from `ff ff ff ff 0f` (2³² − 1), **THEN** it **SHALL** succeed.
  - **WHEN** the decoder reads a bool byte other than `00`/`01`, an option tag other than `00`/`01`, or invalid UTF-8 in a string, **THEN** it **SHALL** fail with `InvalidBool`, `InvalidOptionTag` or `InvalidUtf8` carrying the byte offset.
  - **WHEN** a byte-string length, or the element count of a `Vec`, `BTreeSet` or `BTreeMap`, is the ten-byte varint for `u64::MAX` followed by fewer bytes than declared, **THEN** decoding **SHALL** fail with `UnexpectedEnd` at the prefix's offset without attempting the allocation; a pre-allocation of that size would panic and fail the test.
  - **WHEN** a map or set encoding has keys that are not strictly increasing in the key type's canonical order, **THEN** decoding **SHALL** fail with `UnsortedOrDuplicateKey` at the offending key's offset.
  - **WHEN** `from_slice` decodes input that has bytes left after the value, **THEN** it **SHALL** fail with `TrailingBytes`.
  - **WHEN** a property test generates 10,000 values of each supported type (u64, u32, bool, String, Bytes, HashRef, Option, Vec, BTreeSet, BTreeMap, 2-tuples, nested maps, and blob envelopes with arbitrary bodies), **THEN** decoding the encoding **SHALL** give the value back in every case.
  - **WHEN** a valid encoding of any of those types, the envelope included, is mutated (byte flip, insertion or deletion) and the mutant still decodes, **THEN** re-encoding the decoded value **SHALL** reproduce the mutant byte for byte, over 10,000 cases.
  - **WHEN** 100,000 random byte strings (lengths 0 to 256) are decoded as each of those types and as an envelope, **THEN** every call **SHALL** return `Ok` or `Err` without panicking.
  - **WHEN** an envelope is decoded, **THEN** a format version other than 1 **SHALL** fail with `UnsupportedFormatVersion`, kinds 0x02 to 0x05 **SHALL** fail with `ReservedObjectKind`, and kind 0x00 or 0x06 to 0xff **SHALL** fail with `UnknownObjectKind`; **WHEN** `object_id(alg, envelope)` is computed, **THEN** it **SHALL** equal `alg.hash(envelope)` over the whole envelope.
  - **WHEN** `cargo policy` and `cargo deny check` run after this task, **THEN** both **SHALL** pass, with `proptest` in the prior-art dependency table.
- **Notes:** No serde. Canonical key order is Rust `Ord` per key type: strings and bytes lexicographic by raw content, integers numeric, `HashRef` by (id, digest). `proptest = { version = "1.12.0", default-features = false, features = ["std"] }` is a dev-dependency.

### T-005: Golden corpus and spec consistency
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** `crates/codec/tests/golden.rs`, `crates/codec/tests/golden/v1/*.bin`, `crates/codec/tests/spec.rs`
- **Depends on:** T-004
- **Acceptance criterion:** INT-0006 AC-3 (golden files on Windows and Linux) and AC-6 (code agrees with the spec); INT-0004 AC-1 (identical bytes on both platforms)
- **Success criterion (EARS):**
  - **WHEN** the golden test runs, **THEN** each named corpus value **SHALL** encode byte for byte to `tests/golden/v1/<name>.bin`, and decoding that file **SHALL** yield the corpus value.
  - **WHEN** the golden harness runs with create mode off and a corpus file is missing, **THEN** it **SHALL** fail and name the file; **WHEN** it runs with create mode on, **THEN** it **SHALL** write only missing files and **SHALL NOT** overwrite an existing file, even one that differs, which it still reports.
  - **WHEN** the golden test starts, **THEN** it **SHALL** read `PERI_GOLDEN_CREATE` once and pass create mode (`1` means on) to the harness as a parameter.
  - **WHEN** the spec test parses the Hash algorithms and Object kinds tables in `docs/spec/format-v1.md`, **THEN** every row — (id, name, digest bytes, status) and (id, kind, status) — **SHALL** equal the code registry, where `default` marks `HashAlg::DEFAULT`, `defined` rows equal `ObjectKind::ALL`, and every `reserved` id is rejected with `ReservedObjectKind`.
- **Notes:**
  - The corpus covers varint edge values, `u32::MAX`, both bools, empty and non-empty bytes, ASCII and multi-byte UTF-8 strings, both `HashRef` algorithms, `None`/`Some`, a sequence, a set, a map, a nested map, a pair and one blob envelope.
  - Freeze rule: files under `v1/` never change. A changed existing encoding means a new format version, a new directory and an INT-0006 revision.
  - `PERI_GOLDEN_CREATE` is an explicit opt-in because creating files changes the repository. Edition 2024 makes `set_var` unsafe, so tests never set it; they call the harness with the mode they need.

### T-006: Typed-acceptance validator in the book crate
- **Intent:** [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- **Touches:** `Cargo.toml`, `Cargo.lock`, `docs/prior-art.md`, `docs/.sprint-loop-book`, `crates/book/Cargo.toml`, `crates/book/src/lib.rs`, `crates/book/src/main.rs`, `crates/book/tests/book.rs`, `crates/book/tests/cli.rs`, `crates/book/tests/fixtures/sprint-loops/**`
- **Depends on:** T-001
- **Acceptance criterion:** INT-0002 AC-2 (0 violations, at least one criterion per chapter), AC-3 (status consistency), AC-4 (port rule)
- **Success criterion (EARS):**
  - **WHEN** `cargo book` runs at the repository root with no arguments, **THEN** it **SHALL** check `docs/`, select the strict rule from `typed-acceptance: 1` in `docs/.sprint-loop-book`, report one chapter for each `docs/intents/INT-*.md` file, find at least one criterion in each, report 0 violations and exit 0.
  - **WHEN** a typed line in `## Acceptance criteria` breaks a rule, **THEN** the check **SHALL** report exactly one violation naming the file, the 1-based line and the rule:
    - `polarity` — unknown or missing polarity;
    - `modality` — unknown or missing modality;
    - `claim` — no ` — ` separator, or an empty claim;
    - `number` — a non-numeric, unterminated or zero `AC-n`;
    - `duplicate` — a repeated `AC-n`;
    - `detail` — a detail key not allowed for the criterion's modality, or a detail without `:`.
  - **WHEN** a chapter of a declared Book has no `## Acceptance criteria` section, or an empty one, **THEN** the check **SHALL** report rule `missing`.
  - **WHEN** lines appear inside fenced code blocks (``` or ~~~) anywhere in a chapter, **THEN** the parser **SHALL** ignore them.
  - **WHEN** a typed claim in a declared Book wraps onto indented continuation lines, **THEN** the criterion's claim **SHALL** be the lines joined by single spaces.
  - **WHEN** an untyped top-level list item appears in the criteria section of a declared Book, **THEN** the check **SHALL** report rule `untyped`; **WHEN** the same item appears in an undeclared Book, **THEN** it **SHALL** be read as `necessary` `asserted`, with its wrapped continuation lines joined into the claim.
  - **WHEN** a chapter's text is converted to CRLF in memory, **THEN** the test **SHALL** first confirm the converted text contains `\r\n`, and the parsed criteria **SHALL** equal those of the LF original.
  - **WHEN** the marker declares `typed-acceptance:` with any value other than `1`, **THEN** the check **SHALL** return an error rather than fall back to the port rule, and the binary **SHALL** exit 2.
  - **WHEN** `status` is evaluated for any non-empty list of criteria and any assignment of holds/fails/unknown outcomes (property test, 10,000 cases), **THEN** Certified and Refuted **SHALL NOT** both hold; Certified **SHALL** hold exactly when every criterion holds; Refuted **SHALL** hold exactly when some necessary criterion fails. **WHEN** the list is empty, **THEN** `status` **SHALL** return Undetermined.
  - **WHEN** the check runs over the fixture Book of two vendored Animus_Sprint_Loops chapters with no declaration, **THEN** it **SHALL** report 0 violations and read every criterion as `necessary` `asserted`.
  - **WHEN** the binary finds violations, **THEN** it **SHALL** exit 1; **WHEN** it finds none, **THEN** it **SHALL** exit 0; **WHEN** the Book directory is missing, an argument starts with `-`, or more than one argument is given, **THEN** it **SHALL** exit 2 with a message on stderr.
- **Notes:**
  - The binary is `book [BOOK_DIR]` (default `docs`). Checking is idempotent, so there are no flags; the Book's marker selects the rule.
  - The parser is line-based and std-only. Allowed detail keys per modality: formal — Obligation, Proxy; empirical — Threshold, Sample, Error; preference — Panel, Threshold, Error; judged — Rubric, Quorum; asserted — Signers; every modality — Evidence, Note.
  - The fixtures are copied unchanged from Animus_Sprint_Loops commit `216867f` with an MIT notice; this task adds their Test-data row to prior-art.
  - The `typed-acceptance: 1` marker line is ignored by the Sprint Loops helpers, which read only `schema-version` and `substrate-version`.

### T-007: Canonical suite runner and CI workflow
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- **Touches:** `crates/xtask/Cargo.toml`, `crates/xtask/src/main.rs`, `crates/xtask/src/ci.rs`, `.github/workflows/ci.yml`, `.github/dependabot.yml`, `Cargo.lock`
- **Depends on:** T-001, T-006
- **Acceptance criterion:** INT-0003 AC-2 (locked builds), AC-3 (deny job), AC-4 (fmt and clippy), AC-9 (flagless idempotent tooling); INT-0004 AC-1 (both platforms); INT-0002 AC-2 (Book check on every run)
- **Success criterion (EARS):**
  - **WHEN** `cargo ci` runs, **THEN** it **SHALL** execute exactly these steps in order — `fmt` (`cargo fmt --all --check`), `clippy` (`cargo clippy --workspace --all-targets --locked -- -D warnings`), `test` (`cargo test --workspace --locked`), `policy`, `book` — and print one `PASS <step>` or `FAIL <step>: <reason>` line per step.
  - **WHEN** any step fails, including because its tool is not installed, **THEN** the runner **SHALL** still run the remaining steps and exit non-zero, naming every failed step.
  - **WHEN** `cargo ci` receives any argument besides its command name, including any argument starting with `-`, **THEN** it **SHALL** exit 2 with a usage message on stderr.
  - **WHEN** `cargo ci` runs twice in a row on a clean checkout, **THEN** `git status --porcelain` **SHALL** be empty after each run.
  - **WHEN** the workflow test reads `.github/workflows/ci.yml`, **THEN** it **SHALL** find:
    - push triggers for `dev` and `main`, and a `pull_request` trigger;
    - a job whose matrix holds `ubuntu-latest` and `windows-latest` and that runs `cargo ci`;
    - a `deny` job on `ubuntu-latest` that runs cargo-deny;
    - no `continue-on-error` and no `|| true`.
  - **WHEN** the Dependabot test reads `.github/dependabot.yml`, **THEN** it **SHALL** find a `cargo` and a `github-actions` ecosystem, each with `target-branch: "dev"`.
- **Notes:** cargo-deny stays its own command (`cargo deny check`, the tool's simplest form) and its own CI job rather than a runner step, as INT-0003 records, so `cargo ci` needs no skip flag on machines without it. The workflow and Dependabot checks are std-only text checks in `xtask`.

### T-008: README and documentation checks
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** `README.md`, `docs/prior-art.md`, `crates/xtask/src/main.rs`, `crates/xtask/src/docs.rs`
- **Depends on:** T-001 to T-007
- **Acceptance criterion:** INT-0003 AC-5 (supporting evidence for the owner's design-credit review) and AC-9 (the README documents the flagless commands)
- **Success criterion (EARS):**
  - **WHEN** the design-credit test reads `docs/prior-art.md`, **THEN** the file **SHALL** name every system the Book's intents adopt or evaluate, and the owner's projects and test-data sources: Jujutsu, Mercurial, Sapling, Mononoke, Radicle, Git, JGit, Mergiraf, Spork, TigerBeetle, turmoil, madsim, Zuul, bors, SubmitQueue, Lean 4, comparator, nanoda, lean4export, Aeneas, Pijul, Darcs, Xet, Negentropy, Animus_Sprint_Loops, CubiKan, array-test and the BLAKE3 test vectors.
  - **WHEN** the README test reads `README.md`, **THEN** the owner's two original paragraphs **SHALL** be present verbatim, along with a link to `docs/SUMMARY.md` and the command `cargo ci`.
- **Notes:** The README adds status, crate and command sections below the owner's text; INT-0031 is the Book form of the branding paragraph. `docs.rs` is a test-only module.
