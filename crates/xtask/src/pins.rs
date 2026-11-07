//! Configuration whose every byte decides what the checks check: the CI
//! workflow, the aliases behind `cargo ci`, `cargo policy` and `cargo book`,
//! and the cargo-deny policy. Each file must equal its reviewed copy here, so
//! changing one means changing its copy in the same commit, where both are
//! reviewed. The structural checks in `workflow` and `deny` say why each copy
//! meets its clause; a pin also catches every form they do not read.

use std::fs;
use std::path::Path;

fn repo_file(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("\r\n", "\n")
}

/// The CI workflow (T-007): pushes to dev and main and every pull request run
/// `cargo ci` on Linux and Windows, and a deny job runs every cargo-deny check.
const CI_WORKFLOW: &str = r#"name: ci

on:
  push:
    branches: [dev, main]
  pull_request:

permissions:
  contents: read

jobs:
  ci:
    name: cargo ci (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v7
      # Installs the toolchain and components pinned in rust-toolchain.toml.
      - run: rustup toolchain install
      - run: cargo ci

  deny:
    name: cargo deny check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: EmbarkStudios/cargo-deny-action@v2
"#;

/// The project commands (INT-0003 AC-2 and AC-9): each builds `--locked`, and
/// `book`'s trailing `--` hands every argument to the `book` binary, which
/// refuses flags.
const CARGO_CONFIG: &str = r#"# Project commands. Each is idempotent, so none takes flags (INT-0003 AC-9).
[alias]
ci = "run --locked --quiet --package xtask -- ci"
policy = "run --locked --quiet --package xtask -- policy"
book = "run --locked --quiet --package book --"
"#;

/// The cargo-deny policy (INT-0003 AC-3): GPL-3.0-compatible licences,
/// dev-dependencies included, no advisory ignored.
const DENY_POLICY: &str = r#"# cargo-deny configuration (INT-0003 AC-3). Run with `cargo deny check`.

[graph]
# Check the dependency graph as it resolves on both supported platforms.
targets = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]

[advisories]
# Every RustSec advisory fails the check unless it is listed here with a reason.
ignore = []

[licenses]
# Licences compatible with distributing Periapsis under GPL-3.0. The list is
# policy, so entries no current dependency uses are not reported.
allow = [
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "CC0-1.0",
    "ISC",
    "MIT",
    "Unicode-3.0",
    "Zlib",
]
# Dev-dependencies' licences are checked too; cargo-deny skips them otherwise.
include-dev = true
confidence-threshold = 0.93
unused-allowed-license = "allow"

[licenses.private]
# Workspace crates are unpublished (`publish = false`) and carry the
# repository's own GPL-3.0-only licence.
ignore = true

[bans]
multiple-versions = "warn"
wildcards = "deny"
# Path dependencies between unpublished workspace crates have no version.
allow-wildcard-paths = true

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
"#;

#[test]
fn ci_workflow_is_the_reviewed_one() {
    assert_eq!(repo_file(".github/workflows/ci.yml"), CI_WORKFLOW);
}

#[test]
fn cargo_aliases_are_the_reviewed_ones() {
    assert_eq!(repo_file(".cargo/config.toml"), CARGO_CONFIG);
    // Cargo reads a legacy `.cargo/config` in place of `config.toml` when
    // both exist, so its aliases would replace the pinned ones unseen.
    let legacy = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.cargo/config");
    assert!(
        !legacy.exists(),
        "{} would override .cargo/config.toml",
        legacy.display()
    );
}

#[test]
fn deny_policy_is_the_reviewed_one() {
    assert_eq!(repo_file("deny.toml"), DENY_POLICY);
}
