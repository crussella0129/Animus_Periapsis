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

/// The CI workflow (T-007, revised in Sprint 5 T-004): every pull request,
/// every push to `main` and a manual dispatch run separate `lint` and
/// `cargo ci` checks on Linux and Windows and a checksum-verified cargo-deny
/// check; superseded pull-request runs are cancelled.
const CI_WORKFLOW: &str = r#"name: ci

on:
  push:
    branches: [main]
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

# A newer push to a pull request cancels the run it replaces. Each push to
# main has a group of its own, so no later push cancels or replaces it.
concurrency:
  group: ci-${{ github.ref }}-${{ github.event_name == 'pull_request' || github.sha }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

jobs:
  lint:
    name: lint (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
      # Installs the toolchain and components pinned in rust-toolchain.toml.
      - run: rustup toolchain install
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets --locked -- -D warnings

  ci:
    name: cargo ci (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
      - run: rustup toolchain install
      - uses: Swatinem/rust-cache@v2
      - run: cargo ci

  deny:
    name: cargo deny check
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v7
      - run: rustup toolchain install
      # Checksum-verified, with retried downloads.
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-deny@0.20.2
      - run: cargo deny check
"#;

/// The project commands (INT-0003 AC-2 and AC-9): each builds `--locked`, and
/// `book`'s trailing `--` hands every argument to the `book` binary, which
/// refuses flags.
const CARGO_CONFIG: &str = r#"# Project commands. Each is idempotent, so none takes flags (INT-0003 AC-9).
[alias]
ci = "run --locked --quiet --package xtask -- ci"
reach = "run --locked --quiet --package xtask -- reach"
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

/// The root manifest's profile settings (Sprint 5 T-002): tests build
/// optimised, and nothing else about a profile changes, so debug assertions
/// and overflow checks keep their `dev` defaults (on) in every build.
const PROFILE: [&str; 2] = ["dev.opt-level = 1", "dev.package.\"*\".opt-level = 3"];

/// Every setting under `[profile]` in `manifest`, as `dotted.key = value`,
/// sorted. Tables and inline tables are walked alike, so no form hides one.
fn profile_settings(manifest: &str) -> Vec<String> {
    fn walk(prefix: &str, item: &toml_edit::Item, out: &mut Vec<String>) {
        if let Some(table) = item.as_table_like() {
            for (key, child) in table.iter() {
                let key = if key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    key.to_owned()
                } else {
                    format!("\"{key}\"")
                };
                let path = if prefix.is_empty() {
                    key
                } else {
                    format!("{prefix}.{key}")
                };
                walk(&path, child, out);
            }
        } else if let Some(value) = item.as_value() {
            // Only the `parse` feature is enabled, so values are rendered
            // here: any value that is not an integer, a boolean or a string
            // still shows up, just not as its literal.
            let value = match value {
                toml_edit::Value::Integer(n) => n.value().to_string(),
                toml_edit::Value::Boolean(b) => b.value().to_string(),
                toml_edit::Value::String(s) => format!("{:?}", s.value()),
                _ => "<other>".to_owned(),
            };
            out.push(format!("{prefix} = {value}"));
        }
    }
    let manifest: toml_edit::DocumentMut = manifest.parse().expect("the root manifest parses");
    let mut settings = Vec::new();
    if let Some(profile) = manifest.get("profile") {
        walk("", profile, &mut settings);
    }
    settings.sort();
    settings
}

#[test]
fn optimised_test_profile_is_the_reviewed_one() {
    let manifest = repo_file("Cargo.toml");
    assert_eq!(profile_settings(&manifest), PROFILE);

    // Each change to the profile is caught, whatever form it takes.
    let without = |table: &str| {
        assert!(manifest.contains(table), "{table:?}");
        manifest.replace(table, "")
    };
    let mutants = [
        without("[profile.dev]\nopt-level = 1\n"),
        without("[profile.dev.package.\"*\"]\nopt-level = 3"),
        format!("{manifest}\n[profile.test]\nopt-level = 0\n"),
        manifest.replace(
            "[profile.dev]\nopt-level = 1\n",
            "[profile.dev]\nopt-level = 1\noverflow-checks = false\n",
        ),
        format!("{manifest}\n[profile.dev.package.codec]\nopt-level = 0\n"),
        format!("{manifest}\n[profile.dev.build-override]\nopt-level = 0\n"),
        format!("{manifest}\n[profile.release]\ndebug-assertions = false\n"),
        manifest.replace(
            "[profile.dev]\nopt-level = 1\n",
            "[profile.dev]\nopt-level = 1\ndebug-assertions = false\n",
        ),
    ];
    for mutant in mutants {
        assert_ne!(profile_settings(&mutant), PROFILE, "{mutant}");
    }
}

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
