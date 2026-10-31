# Completed Tasks Log (Append-Only)

## T-001 (sprint 0)
- **Description:** Workspace, guardrails, policy checker and credits
- **Intent:** [INT-0003](../intents/INT-0003-engineering-baseline.md), [INT-0005](../intents/INT-0005-deterministic-simulation-testing.md)
- **Completed:** 2026-10-31T21:03:40Z
- **Files modified:** Cargo.toml, Cargo.lock, rust-toolchain.toml, deny.toml, clippy.toml, .gitattributes, .cargo/config.toml, docs/prior-art.md, crates/xtask/Cargo.toml, crates/xtask/src/main.rs, crates/xtask/src/policy.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo policy` → PASS for lints, unsafe, toolchain, lockfile, publish, deps, credits, clippy, names (exit 0); `cargo policy --fix` → usage on stderr, exit 2; `cargo test --workspace --locked` → 15 passed; `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo fmt --all --check` clean; `cargo deny check` (cargo-deny 0.20.2) → advisories ok, bans ok, licenses ok, sources ok; probe in a scratch copy with a planted `std::collections::HashMap` → Clippy fails on `disallowed_types`.
- **Commit:** `f89c74156ea2b0fef17ee52ce29cefec98bd7100`

## T-002 (sprint 0)
- **Description:** Format v1 specification
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md)
- **Completed:** 2026-10-31T21:05:20Z
- **Files modified:** docs/spec/format-v1.md, crates/xtask/src/policy.rs, docs/intents/INT-0006-format-v1.md
- **Evidence:** Windows 11 host, rustc 1.99.1. The spec lands before any hash or codec code. `cargo policy` → PASS for all 10 rules including the new `spec` rule (exit 0); `cargo test --workspace --locked` → 16 passed, including `policy_requires_format_spec` (missing spec, missing Object identity section, empty Object kinds, empty Worked examples → FAIL `spec`); fmt and Clippy clean.
- **Commit:** `951be4fe76da7c7e754f3914b49ac3a4ef09d39d`

## T-003 (sprint 0)
- **Description:** Algorithm-tagged hashes in the hash crate
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md), [INT-0003](../intents/INT-0003-engineering-baseline.md)
- **Completed:** 2026-10-31T21:06:56Z
- **Files modified:** Cargo.toml, Cargo.lock, docs/prior-art.md, crates/hash/Cargo.toml, crates/hash/src/lib.rs, crates/hash/tests/vectors.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p hash` → 11 passed (35 official BLAKE3 vectors, SHA-256 FIPS vectors, registry ids 0..=255, `hash_registry_is_complete`, digest lengths, cross-algorithm order, binary and text round trips and rejections, chunked = one-shot). `cargo tree --workspace --target all -e normal,build -i cc --depth 1` → `cc` ← `blake3` only, the single C-build `yes` row. `cargo tree --workspace -e features -i blake3` → `blake3 feature "pure"`. `cargo policy` → 10 rules PASS. `cargo deny check` → advisories ok, bans ok, licenses ok, sources ok. deny.toml needed no change.
- **Commit:** `0fdb12a3c769dbaeecee195aba3ecbbfee1c75f8`

## T-004 (sprint 0)
- **Description:** Canonical binary encoding in the codec crate
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md)
- **Completed:** 2026-10-31T21:09:14Z
- **Files modified:** Cargo.toml, Cargo.lock, crates/codec/Cargo.toml, crates/codec/src/lib.rs, crates/codec/src/error.rs, crates/codec/src/wire.rs, crates/codec/src/envelope.rs, crates/codec/tests/properties.rs, crates/codec/tests/mutants.rs
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p codec` → 38 passed in 6.4 s: 13 round-trip and 13 mutation properties at 10,000 cases each (u64, u32, bool, String, Bytes, HashRef, Option, Vec, BTreeSet, BTreeMap, pair, nested map, blob envelope); `decode_is_total_on_random_bytes` (100,000 cases, every type and the envelope); `exhaustive_mutants_reencode` (every single-byte substitution, insertion and deletion of one value per type, at least one decoding mutant each); known varint encodings; non-canonical varints, bad bool, option tag and UTF-8, and `u64::MAX` length and count prefixes all rejected at offset 2 (UTF-8 at the first content byte, 3); `u32` range; strictly increasing keys; trailing bytes; reserved versus unknown kinds; whole-envelope object id; kind registry. No new third-party dependency; `cargo policy` 10 PASS; `cargo deny check` ok.
- **Commit:** `1f747f36a54f6d32bd8ffdb9ff0837cd24415e25`

## T-005 (sprint 0)
- **Description:** Golden corpus and spec consistency
- **Intent:** [INT-0006](../intents/INT-0006-format-v1.md), [INT-0004](../intents/INT-0004-windows-and-linux.md)
- **Completed:** 2026-10-31T21:11:01Z
- **Files modified:** .gitattributes, crates/xtask/src/policy.rs, crates/codec/tests/golden.rs, crates/codec/tests/golden/v1/*.bin (23 files), crates/codec/tests/spec.rs, docs/intents/INT-0004-windows-and-linux.md
- **Evidence:** Windows 11 host, rustc 1.99.1. Golden files created once with `PERI_GOLDEN_CREATE=1 cargo test -p codec --test golden golden_v1_corpus_matches`; later runs compare only. `spec_examples_match_golden_files` passed on its first run: all 23 hand-derived worked-example hex strings in the spec equal the encoder output, the encoder's independent check. `spec_registries_match_code` passed (hash and kind tables, reserved rows included). Golden tests 5 passed: corpus matches; one changed byte fails, naming only that file; a missing file fails without create mode; create mode never overwrites; `create_mode` true only for "1". `git check-attr` reports the .bin files as binary. `cargo policy` → 11 rules PASS, including the new `golden` rule (`policy_requires_binary_golden_attr`).
- **Commit:** `8f75733232dc44de4d6776a903ced35ddc6e3afd`

## T-006 (sprint 0)
- **Description:** Typed-acceptance validator in the book crate
- **Intent:** [INT-0002](../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../intents/INT-0003-engineering-baseline.md)
- **Completed:** 2026-10-31T21:15:04Z
- **Files modified:** Cargo.toml, Cargo.lock, docs/prior-art.md, docs/.sprint-loop-book, crates/book/Cargo.toml, crates/book/src/lib.rs, crates/book/src/main.rs, crates/book/tests/book.rs, crates/book/tests/cli.rs, crates/book/tests/fixtures/sprint-loops/ (marker, NOTICE.md, two chapters), docs/intents/INT-0002-typed-acceptance-criteria.md
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p book` → 14 passed: every typed-line rule including AC-0, each with exactly one violation; `missing` for an absent or empty section only; backtick and tilde fences, including a fenced criteria heading; wrapped typed claims; declared versus undeclared; CRLF built in memory; unknown declaration and marker-less directory refused; status truth table and 10,000-case consistency property; this Book strict-clean; fixtures give 5 and 9 criteria with the joined claim; CLI exit codes 0, 1 and 2, with stderr on every 2. `cargo book` → 31 chapters, 164 criteria, 0 violations, exit 0. `cargo book --strict` → exit 2. `cargo book ~/Animus_Sprint_Loops/docs` at e8c3a58 → 13 chapters, 82 criteria (equal to the independent count), 0 violations. Sprint Loops helpers accept `typed-acceptance: 1` (check-book valid, substrate-complete). `cargo policy` 11 PASS.
- **Commit:** `df87da1e8125010eb16c2dcfdf8b9e1022cc4b06`

## T-007 (sprint 0)
- **Description:** Canonical suite runner and CI workflow
- **Intent:** [INT-0003](../intents/INT-0003-engineering-baseline.md), [INT-0004](../intents/INT-0004-windows-and-linux.md), [INT-0002](../intents/INT-0002-typed-acceptance-criteria.md)
- **Completed:** 2026-10-31T21:16:49Z
- **Files modified:** crates/xtask/Cargo.toml, crates/xtask/src/main.rs, crates/xtask/src/ci.rs, .github/workflows/ci.yml, .github/dependabot.yml, Cargo.lock
- **Evidence:** Windows 11 host, rustc 1.99.1. `cargo test -p xtask` → 22 passed, including `ci_steps_are_ordered_and_locked`; `ci_failures_do_not_stop_later_steps` (two failures, every step runs, exit 1, summary names both); `ci_rejects_extra_arguments`; `test_step_removes_golden_create`; `workflow_matches_ci_contract`. `cargo ci` on this checkout → PASS fmt, clippy, test, policy, book; "ci: all 5 steps passed"; exit 0 in 9.4 s. `cargo ci --fast` → usage on stderr, exit 2. The book and policy steps run in-process; the nested `cargo test` did not rebuild the running xtask binary. The WSL ext4 clone, fresh `core.autocrlf=true` clone and idempotence runs need committed state and run in the Test phase.
- **Commit:** `1a0658ed3fa7e7d9b08cf89f2e509119eaafc2d8`
