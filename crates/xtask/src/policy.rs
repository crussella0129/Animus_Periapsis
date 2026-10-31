//! `cargo policy`: mechanical checks of the engineering baseline (INT-0003),
//! the hash-collection guard (INT-0005 AC-4), the format spec's presence
//! (INT-0006 AC-6) and byte-exact golden files (INT-0006 AC-3).
//!
//! Each rule is a pure function of the files under the workspace root, so the
//! tests run every rule against small fixture trees.

use std::fs;
use std::path::{Path, PathBuf};

/// A rule: given the workspace root and its member crate directories, the
/// problems found. An empty list means the rule passed.
type Rule = fn(&Path, &[PathBuf]) -> Vec<String>;

/// The rules, in the order they are reported.
pub const RULES: [(&str, Rule); 11] = [
    ("lints", lints),
    ("unsafe", forbid_unsafe),
    ("toolchain", toolchain),
    ("lockfile", lockfile),
    ("publish", publish),
    ("deps", deps),
    ("credits", credits),
    ("clippy", clippy),
    ("names", names),
    ("spec", spec),
    ("golden", golden),
];

const FORBID_UNSAFE: &str = "#![forbid(unsafe_code)]";
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
const DISALLOWED: [&str; 2] = ["std::collections::HashMap", "std::collections::HashSet"];
const C_ALLOWED: &str = "C allowed:";
const SPEC_PATH: &str = "docs/spec/format-v1.md";
const SPEC_SECTIONS: [&str; 11] = [
    "Conventions",
    "Primitives",
    "Canonical order",
    "Hash algorithms",
    "Objects",
    "Object kinds",
    "Object identity",
    "Worked examples",
    "Signatures (reserved)",
    "Decoder requirements",
    "Change control",
];
/// Tables the spec must hold rows in: (section, row prefix, minimum cells).
const SPEC_TABLES: [(&str, &str, usize); 3] = [
    ("Hash algorithms", "| `0x", 4),
    ("Object kinds", "| `0x", 3),
    ("Worked examples", "| `", 3),
];

/// The outcome of one rule: an empty problem list means it passed.
#[derive(Debug)]
pub struct RuleResult {
    pub rule: &'static str,
    pub problems: Vec<String>,
}

/// Runs every rule against the workspace rooted at `root`.
pub fn check(root: &Path) -> Vec<RuleResult> {
    let members = members(root);
    RULES
        .into_iter()
        .map(|(rule, run)| RuleResult {
            rule,
            problems: run(root, &members),
        })
        .collect()
}

/// Member crate directories: every `crates/*` holding a `Cargo.toml`, matching
/// the workspace's `members = ["crates/*"]`.
fn members(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(root.join("crates"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("Cargo.toml").is_file())
        .collect();
    dirs.sort();
    dirs
}

fn lints(root: &Path, members: &[PathBuf]) -> Vec<String> {
    members
        .iter()
        .filter(|dir| {
            let manifest = read(&dir.join("Cargo.toml"));
            !section(&manifest, "lints")
                .iter()
                .any(|line| key_value(line) == Some(("workspace", "true")))
        })
        .map(|dir| format!("{} lacks [lints] workspace = true", relative(root, dir)))
        .collect()
}

fn forbid_unsafe(root: &Path, members: &[PathBuf]) -> Vec<String> {
    let mut problems = Vec::new();
    for dir in members {
        for crate_root in ["src/lib.rs", "src/main.rs"] {
            let path = dir.join(crate_root);
            if path.is_file() && !read(&path).lines().any(|line| line.trim() == FORBID_UNSAFE) {
                problems.push(format!("{} lacks {FORBID_UNSAFE}", relative(root, &path)));
            }
        }
    }
    problems
}

fn toolchain(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let text = read(&root.join("rust-toolchain.toml"));
    let channel = section(&text, "toolchain")
        .into_iter()
        .find_map(|line| match key_value(line) {
            Some(("channel", value)) => Some(value.trim_matches('"').to_owned()),
            _ => None,
        });
    match channel {
        Some(channel) if is_exact_version(&channel) => Vec::new(),
        Some(channel) => vec![format!(
            "rust-toolchain.toml channel {channel:?} is not an exact X.Y.Z release"
        )],
        None => vec!["rust-toolchain.toml has no [toolchain] channel".to_owned()],
    }
}

fn is_exact_version(channel: &str) -> bool {
    let parts: Vec<&str> = channel.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

fn lockfile(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    if root.join("Cargo.lock").is_file() {
        Vec::new()
    } else {
        vec!["Cargo.lock is missing".to_owned()]
    }
}

/// Every member resolves to `publish = false`, until INT-0003 records a
/// publishable crate.
fn publish(root: &Path, members: &[PathBuf]) -> Vec<String> {
    let workspace_unpublished = section(&read(&root.join("Cargo.toml")), "workspace.package")
        .iter()
        .any(|line| key_value(line) == Some(("publish", "false")));
    members
        .iter()
        .filter(|dir| {
            let manifest = read(&dir.join("Cargo.toml"));
            let package = section(&manifest, "package");
            let own = package
                .iter()
                .any(|line| key_value(line) == Some(("publish", "false")));
            let inherited = workspace_unpublished
                && package
                    .iter()
                    .any(|line| key_value(line) == Some(("publish.workspace", "true")));
            !(own || inherited)
        })
        .map(|dir| {
            format!(
                "{} does not resolve to publish = false",
                relative(root, dir)
            )
        })
        .collect()
}

/// Every dependency of a member crate is inherited from the workspace table,
/// which is the one place licences and credits are checked.
fn deps(root: &Path, members: &[PathBuf]) -> Vec<String> {
    let mut problems = Vec::new();
    for dir in members {
        let manifest = read(&dir.join("Cargo.toml"));
        for (table, lines) in tables(&manifest) {
            let kind = table.split('.').next().unwrap_or_default();
            if !DEPENDENCY_TABLES.contains(&kind) {
                continue;
            }
            if let Some((_, name)) = table.split_once('.') {
                // A `[dependencies.<name>]` table.
                let inherited = lines
                    .iter()
                    .any(|line| key_value(line) == Some(("workspace", "true")));
                if !inherited {
                    problems.push(direct(root, dir, name));
                }
                continue;
            }
            for line in lines {
                let Some((key, value)) = key_value(line) else {
                    continue;
                };
                let inherited = (key.ends_with(".workspace") && value == "true")
                    || value.replace(' ', "").contains("workspace=true");
                if !inherited {
                    let name = key.split('.').next().unwrap_or(key);
                    problems.push(direct(root, dir, name));
                }
            }
        }
    }
    problems
}

fn direct(root: &Path, dir: &Path, name: &str) -> String {
    format!(
        "{} declares `{name}` directly; use `{name}.workspace = true`",
        relative(root, dir)
    )
}

/// Every third-party workspace dependency has a complete row in the
/// prior-art dependency table: a licence, whether it can build C, and for a
/// crate that can, the feature that keeps it pure or why C is allowed.
fn credits(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let path = root.join("docs/prior-art.md");
    if !path.is_file() {
        return vec!["docs/prior-art.md is missing".to_owned()];
    }
    let credits = read(&path);
    let manifest = read(&root.join("Cargo.toml"));
    let mut problems = Vec::new();
    for line in section(&manifest, "workspace.dependencies") {
        if line.contains("path =") {
            continue;
        }
        let Some((name, entry)) = key_value(line) else {
            continue;
        };
        let row_start = format!("| `{name}` |");
        let Some(row) = credits
            .lines()
            .find(|line| line.trim_start().starts_with(&row_start))
        else {
            problems.push(format!(
                "dependency `{name}` has no row in docs/prior-art.md"
            ));
            continue;
        };
        let cells: Vec<&str> = row
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let filled = |i: usize| cells.get(i).is_some_and(|c| !c.is_empty() && *c != "—");
        let c_build = cells.get(4).copied().unwrap_or_default();
        let how = cells.get(5).copied().unwrap_or_default();
        let complete = filled(2)
            && (c_build.starts_with("no")
                || (c_build.starts_with("yes") && builds_pure_or_justified(how, entry)));
        if !complete {
            problems.push(format!(
                "dependency `{name}` needs a licence, a yes/no C-build cell and, for yes, \
                 an enabled pure feature or a `{C_ALLOWED}` justification in docs/prior-art.md"
            ));
        }
    }
    problems
}

/// A `yes` row either names features, as `features = ["…"]`, that the
/// workspace entry enables, or starts with `C allowed:` and a reason.
fn builds_pure_or_justified(how: &str, entry: &str) -> bool {
    if let Some(reason) = how.strip_prefix(C_ALLOWED) {
        return !reason.trim().is_empty();
    }
    let Some(list) = how
        .split_once("features = [")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list)
    else {
        return false;
    };
    let named: Vec<&str> = list
        .split(',')
        .map(|feature| feature.trim().trim_matches('"'))
        .filter(|feature| !feature.is_empty())
        .collect();
    !named.is_empty()
        && named
            .iter()
            .all(|feature| entry.contains(&format!("\"{feature}\"")))
}

fn clippy(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let text = read(&root.join("clippy.toml"));
    let has_list = text.contains("disallowed-types");
    DISALLOWED
        .into_iter()
        .filter(|path| !(has_list && text.contains(&format!("\"{path}\""))))
        .map(|path| format!("clippy.toml does not list {path} under disallowed-types"))
        .collect()
}

fn names(root: &Path, members: &[PathBuf]) -> Vec<String> {
    members
        .iter()
        .filter_map(|dir| {
            let manifest = read(&dir.join("Cargo.toml"));
            let name = section(&manifest, "package")
                .into_iter()
                .find_map(|line| match key_value(line) {
                    Some(("name", value)) => Some(value.trim_matches('"').to_owned()),
                    _ => None,
                })
                .unwrap_or_default();
            (!is_single_word(&name)).then(|| {
                format!(
                    "{} package name {name:?} is not a single lowercase word",
                    relative(root, dir)
                )
            })
        })
        .collect()
}

fn is_single_word(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

/// The format spec exists, keeps every required section, and its registry
/// and worked-example tables hold rows (INT-0006 AC-6: the spec comes first).
fn spec(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let path = root.join(SPEC_PATH);
    if !path.is_file() {
        return vec![format!("{SPEC_PATH} is missing")];
    }
    let text = read(&path);
    let headings: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix('#'))
        .map(|rest| rest.trim_start_matches('#').trim())
        .collect();
    let mut problems: Vec<String> = SPEC_SECTIONS
        .into_iter()
        .filter(|name| !headings.contains(name))
        .map(|name| format!("{SPEC_PATH} has no \"{name}\" section"))
        .collect();
    for (name, prefix, cells) in SPEC_TABLES {
        if table_rows(&text, name, prefix, cells) == 0 {
            problems.push(format!("{SPEC_PATH} \"{name}\" has no table rows"));
        }
    }
    problems
}

/// Golden encodings must reach every platform byte for byte, so Git must
/// treat them as binary (INT-0006 AC-3, INT-0004 AC-1).
fn golden(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let text = read(&root.join(".gitattributes"));
    let marked = text.lines().any(|line| {
        let mut fields = line.split_whitespace();
        fields.next() == Some("*.bin") && fields.any(|field| field == "binary")
    });
    if marked {
        Vec::new()
    } else {
        vec![".gitattributes does not mark *.bin as binary".to_owned()]
    }
}

/// Counts table rows that start with `prefix` and hold at least `cells`
/// cells, in the section headed `name`.
fn table_rows(text: &str, name: &str, prefix: &str, cells: usize) -> usize {
    let mut inside = false;
    let mut rows = 0;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix('#') {
            inside = rest.trim_start_matches('#').trim() == name;
        } else if inside
            && line.starts_with(prefix)
            && line.trim().trim_matches('|').split('|').count() >= cells
        {
            rows += 1;
        }
    }
    rows
}

/// Reads a file, treating a missing or unreadable file as empty; each rule
/// then reports what it found missing.
fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Every TOML table as (header, content lines), without comments or blanks.
fn tables(text: &str) -> Vec<(String, Vec<&str>)> {
    let mut tables: Vec<(String, Vec<&str>)> = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            tables.push((header.trim().to_owned(), Vec::new()));
        } else if !line.is_empty()
            && let Some((_, lines)) = tables.last_mut()
        {
            lines.push(line);
        }
    }
    tables
}

/// The content lines of one TOML table.
fn section<'t>(text: &'t str, name: &str) -> Vec<&'t str> {
    tables(text)
        .into_iter()
        .filter(|(header, _)| header == name)
        .flat_map(|(_, lines)| lines)
        .collect()
}

/// Splits `key = value` into trimmed parts.
fn key_value(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    Some((key.trim(), value.trim()))
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{check, is_exact_version, is_single_word};

    const SPEC: &str = "\
# Format\n## Conventions\n## Primitives\n### Canonical order\n## Hash algorithms\n\
| id | name | digest bytes | status |\n|---|---|---|---|\n| `0x01` | `blake3` | 32 | default |\n\
## Objects\n### Object kinds\n| id | kind | status |\n|---|---|---|\n| `0x01` | `blob` | defined |\n\
### Object identity\n## Worked examples\n| name | value | hex |\n|---|---|---|\n| `bool-true` | true | `01` |\n\
## Signatures (reserved)\n## Decoder requirements\n## Change control\n";

    /// A fixture workspace that passes every rule until a test breaks one.
    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("xtask-policy-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            let fixture = Fixture(root);
            fixture.write(
                "Cargo.toml",
                "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.package]\npublish = false\n\n\
                 [workspace.dependencies]\nblake3 = { version = \"1\", features = [\"pure\"] }\n\
                 hash = { path = \"crates/hash\" }\n",
            );
            fixture.write("Cargo.lock", "version = 4\n");
            fixture.write("rust-toolchain.toml", "[toolchain]\nchannel = \"1.99.1\"\n");
            fixture.write(".gitattributes", "* text=auto eol=lf\n*.bin binary\n");
            fixture.write(
                "clippy.toml",
                "disallowed-types = [\n  { path = \"std::collections::HashMap\" },\n  \
                 { path = \"std::collections::HashSet\" },\n]\n",
            );
            fixture.write(
                "docs/prior-art.md",
                "| Crate | Version | Licence | Used by | Can build C | How |\n|---|---|---|---|---|---|\n\
                 | `blake3` | 1 | CC0-1.0 | `hash` | yes | `features = [\"pure\"]`: Rust only |\n",
            );
            fixture.write("docs/spec/format-v1.md", SPEC);
            fixture.write(
                "crates/hash/Cargo.toml",
                &member("hash", "blake3.workspace = true"),
            );
            fixture.write("crates/hash/src/lib.rs", "#![forbid(unsafe_code)]\n");
            fixture
        }

        fn write(&self, path: &str, text: &str) {
            let path = self.0.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }

        /// The rules that fail, with their problems.
        fn failures(&self) -> Vec<(&'static str, Vec<String>)> {
            check(&self.0)
                .into_iter()
                .filter(|result| !result.problems.is_empty())
                .map(|result| (result.rule, result.problems))
                .collect()
        }

        fn fails_only(&self, rule: &str, needle: &str) {
            let failures = self.failures();
            assert_eq!(failures.len(), 1, "{failures:?}");
            assert_eq!(failures[0].0, rule, "{failures:?}");
            assert!(
                failures[0].1.iter().any(|p| p.contains(needle)),
                "{failures:?} should mention {needle:?}"
            );
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// A member manifest that passes every rule, with `dependency` as its one
    /// `[dependencies]` line.
    fn member(name: &str, dependency: &str) -> String {
        format!(
            "[package]\nname = \"{name}\"\npublish.workspace = true\n\n[dependencies]\n{dependency}\n\n\
             [lints]\nworkspace = true\n"
        )
    }

    #[test]
    fn fixture_passes_every_rule() {
        let failures = Fixture::new("clean").failures();
        assert!(failures.is_empty(), "{failures:?}");
    }

    #[test]
    fn policy_passes_on_workspace() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let failures: Vec<_> = check(&root)
            .into_iter()
            .filter(|result| !result.problems.is_empty())
            .collect();
        assert!(failures.is_empty(), "{failures:?}");
    }

    #[test]
    fn policy_flags_missing_workspace_lints() {
        let fixture = Fixture::new("lints");
        fixture.write(
            "crates/hash/Cargo.toml",
            "[package]\nname = \"hash\"\npublish.workspace = true\n\n[dependencies]\nblake3.workspace = true\n",
        );
        fixture.fails_only("lints", "crates/hash");
    }

    #[test]
    fn policy_flags_missing_forbid_unsafe() {
        let fixture = Fixture::new("unsafe");
        fixture.write("crates/hash/src/main.rs", "fn main() {}\n");
        fixture.fails_only("unsafe", "crates/hash/src/main.rs");
    }

    #[test]
    fn policy_requires_exact_toolchain() {
        for (i, channel) in ["stable", "1.99", "nightly-2026-10-01", "1.99.1-beta"]
            .into_iter()
            .enumerate()
        {
            let fixture = Fixture::new(&format!("toolchain-{i}"));
            fixture.write(
                "rust-toolchain.toml",
                &format!("[toolchain]\nchannel = \"{channel}\"\n"),
            );
            fixture.fails_only("toolchain", channel);
        }
        assert!(is_exact_version("1.99.1"));
    }

    #[test]
    fn policy_requires_lockfile() {
        let fixture = Fixture::new("lockfile");
        fs::remove_file(fixture.0.join("Cargo.lock")).unwrap();
        fixture.fails_only("lockfile", "Cargo.lock");
    }

    #[test]
    fn policy_requires_unpublished_members() {
        let fixture = Fixture::new("publish");
        fixture.write(
            "crates/hash/Cargo.toml",
            "[package]\nname = \"hash\"\n\n[dependencies]\nblake3.workspace = true\n\n[lints]\nworkspace = true\n",
        );
        fixture.fails_only("publish", "crates/hash");

        let fixture = Fixture::new("publish-own");
        fixture.write(
            "crates/hash/Cargo.toml",
            "[package]\nname = \"hash\"\npublish = false\n\n[dependencies]\nblake3.workspace = true\n\n\
             [lints]\nworkspace = true\n",
        );
        assert!(fixture.failures().is_empty(), "{:?}", fixture.failures());
    }

    #[test]
    fn policy_flags_direct_dependency() {
        for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let fixture = Fixture::new(&format!("deps-{table}"));
            fixture.write(
                "crates/hash/Cargo.toml",
                &format!(
                    "[package]\nname = \"hash\"\npublish.workspace = true\n\n[dependencies]\n\
                     blake3.workspace = true\n\n[{table}]\nfoo = \"1\"\n\n[lints]\nworkspace = true\n"
                ),
            );
            fixture.fails_only("deps", "`foo`");
        }
        let fixture = Fixture::new("deps-table");
        fixture.write(
            "crates/hash/Cargo.toml",
            "[package]\nname = \"hash\"\npublish.workspace = true\n\n[dependencies]\nblake3.workspace = true\n\n\
             [dependencies.foo]\nversion = \"1\"\n\n[lints]\nworkspace = true\n",
        );
        fixture.fails_only("deps", "`foo`");
    }

    #[test]
    fn policy_requires_prior_art_file() {
        let fixture = Fixture::new("credits-file");
        fs::remove_file(fixture.0.join("docs/prior-art.md")).unwrap();
        fixture.fails_only("credits", "docs/prior-art.md");
    }

    #[test]
    fn policy_flags_uncredited_dependency() {
        let fixture = Fixture::new("credits-missing");
        fixture.write("docs/prior-art.md", "Nothing credited.\n");
        fixture.fails_only("credits", "`blake3`");
    }

    #[test]
    fn policy_requires_complete_dependency_rows() {
        let failing = [
            "| `blake3` | 1 |  | `hash` | yes | `features = [\"pure\"]` |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | maybe | `features = [\"pure\"]` |",
            "| `blake3` | 1 | CC0-1.0 |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes |  |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | `features = [\"neon\"]` |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | C allowed: |",
        ];
        for (i, row) in failing.into_iter().enumerate() {
            let fixture = Fixture::new(&format!("credits-row-{i}"));
            fixture.write("docs/prior-art.md", &format!("{row}\n"));
            fixture.fails_only("credits", "`blake3`");
        }
        let passing = [
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | `features = [\"pure\"]` |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | C allowed: no pure-Rust option exists |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | no | — |",
        ];
        for (i, row) in passing.into_iter().enumerate() {
            let fixture = Fixture::new(&format!("credits-ok-{i}"));
            fixture.write("docs/prior-art.md", &format!("{row}\n"));
            assert!(
                fixture.failures().is_empty(),
                "{row}: {:?}",
                fixture.failures()
            );
        }
    }

    #[test]
    fn policy_requires_disallowed_hash_collections() {
        for (missing, kept) in [("HashMap", "HashSet"), ("HashSet", "HashMap")] {
            let fixture = Fixture::new(&format!("clippy-{missing}"));
            fixture.write(
                "clippy.toml",
                &format!("disallowed-types = [{{ path = \"std::collections::{kept}\" }}]\n"),
            );
            fixture.fails_only("clippy", missing);
        }
    }

    #[test]
    fn policy_requires_single_word_names() {
        for (i, name) in ["peri-hash", "Hash", "hash_ref", "9hash"]
            .into_iter()
            .enumerate()
        {
            let fixture = Fixture::new(&format!("names-{i}"));
            fixture.write(
                "crates/hash/Cargo.toml",
                &member(name, "blake3.workspace = true"),
            );
            fixture.fails_only("names", name);
        }
        assert!(is_single_word("hash"));
        assert!(is_single_word("sha256"));
    }

    #[test]
    fn policy_requires_binary_golden_attr() {
        let fixture = Fixture::new("golden");
        fixture.write(".gitattributes", "* text=auto eol=lf\n");
        fixture.fails_only("golden", "*.bin");

        let fixture = Fixture::new("golden-text");
        fixture.write(".gitattributes", "* text=auto eol=lf\n*.bin text\n");
        fixture.fails_only("golden", "*.bin");
    }

    #[test]
    fn policy_requires_format_spec() {
        let fixture = Fixture::new("spec-missing");
        fs::remove_file(fixture.0.join("docs/spec/format-v1.md")).unwrap();
        fixture.fails_only("spec", "format-v1.md is missing");

        let fixture = Fixture::new("spec-section");
        fixture.write(
            "docs/spec/format-v1.md",
            &SPEC.replace("### Object identity\n", ""),
        );
        fixture.fails_only("spec", "Object identity");

        let fixture = Fixture::new("spec-kinds");
        fixture.write(
            "docs/spec/format-v1.md",
            &SPEC.replace("| `0x01` | `blob` | defined |\n", ""),
        );
        fixture.fails_only("spec", "Object kinds");

        let fixture = Fixture::new("spec-examples");
        fixture.write(
            "docs/spec/format-v1.md",
            &SPEC.replace("| `bool-true` | true | `01` |\n", ""),
        );
        fixture.fails_only("spec", "Worked examples");
    }
}
