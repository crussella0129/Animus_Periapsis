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
