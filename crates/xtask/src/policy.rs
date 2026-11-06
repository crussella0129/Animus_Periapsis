//! `cargo policy`: mechanical checks of the engineering baseline (INT-0003),
//! the hash-collection guard (INT-0005 AC-4), the format spec's presence
//! (INT-0006 AC-6) and byte-exact golden files (INT-0006 AC-3).
//!
//! Each rule reads the files under the workspace root (`golden` asks Git,
//! which reads them too), so the tests run every rule against small fixture
//! trees. TOML files are parsed, as Cargo, rustup and Clippy parse them, and
//! the spec's Markdown is read for the headings and table rows GitHub shows.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use toml_edit::{DocumentMut, Item, Value};

use crate::attributes;
use crate::manifest::{dependency_tables, is_true, load, lookup, read_text, strings};
use crate::workspace::members;

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
const DISALLOWED: [&str; 2] = ["std::collections::HashMap", "std::collections::HashSet"];
/// The configs Clippy looks for in each directory, the first one winning.
const CLIPPY_CONFIGS: [&str; 2] = [".clippy.toml", "clippy.toml"];
const C_ALLOWED: &str = "C allowed:";
/// A path in the golden corpus for Git to judge; it need not exist.
/// Where the committed golden files live.
const GOLDEN_DIR: &str = "crates/codec/tests/golden/v1";
/// A path that stands for any golden file added later.
const GOLDEN_PROBE: &str = "crates/codec/tests/golden/v1/probe.bin";
/// Attributes under which Git can rewrite a file's bytes on checkout or
/// commit: line endings, filters, `$Id$` expansion and re-encoding.
const BYTE_ATTRIBUTES: [&str; 4] = ["text", "filter", "ident", "working-tree-encoding"];
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
/// Tables the spec must hold rows in: (section, how the code span that
/// starts a row's first cell begins, minimum cells filled).
const SPEC_TABLES: [(&str, &str, usize); 3] = [
    ("Hash algorithms", "0x", 4),
    ("Object kinds", "0x", 3),
    ("Worked examples", "", 3),
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

/// Every member inherits the workspace lints, and they forbid `unsafe_code`:
/// that lint is INT-0003 AC-1's guard for tests, build scripts, binaries and
/// examples, which the `unsafe` rule never reads.
fn lints(root: &Path, members: &[PathBuf]) -> Vec<String> {
    let mut problems = match parse(root, &root.join("Cargo.toml")) {
        Ok(workspace) => {
            let mut problems = unexpandable_members(&workspace);
            if !forbids_unsafe_code(&workspace) {
                problems.push(
                    "Cargo.toml does not set unsafe_code = \"forbid\" in [workspace.lints.rust]"
                        .to_owned(),
                );
            }
            problems
        }
        Err(problem) => vec![problem],
    };
    problems.extend(each_manifest(root, members, |dir, manifest| {
        (!is_true(manifest.as_item(), &["lints", "workspace"]))
            .then(|| format!("{dir} lacks [lints] workspace = true"))
    }));
    problems
}

/// `members` entries that use glob syntax the policy does not expand, `[...]`
/// classes and `**`. The crates Cargo finds through them would escape every
/// member rule, so such an entry fails instead.
fn unexpandable_members(workspace: &DocumentMut) -> Vec<String> {
    strings(workspace.as_item(), &["workspace", "members"])
        .into_iter()
        .filter(|entry| entry.contains(['[', ']']) || entry.contains("**"))
        .map(|entry| {
            format!(
                "Cargo.toml members entry {entry:?} uses a pattern the policy cannot expand; \
                 use literal paths, * and ?"
            )
        })
        .collect()
}

/// `unsafe_code = "forbid"` in `[workspace.lints.rust]`, given as a level or
/// as a table with a `level`.
fn forbids_unsafe_code(workspace: &DocumentMut) -> bool {
    let lint = lookup(
        workspace.as_item(),
        &["workspace", "lints", "rust", "unsafe_code"],
    );
    lint.and_then(|lint| lint.as_str().or_else(|| lookup(lint, &["level"])?.as_str()))
        == Some("forbid")
}

fn forbid_unsafe(root: &Path, members: &[PathBuf]) -> Vec<String> {
    let mut problems = Vec::new();
    for dir in members {
        for crate_root in ["src/lib.rs", "src/main.rs"] {
            let path = dir.join(crate_root);
            if path.is_file() && !opens_with_forbid_unsafe(&read(&path)) {
                problems.push(format!("{} lacks {FORBID_UNSAFE}", relative(root, &path)));
            }
        }
    }
    problems
}

/// Whether a crate root carries `#![forbid(unsafe_code)]` as its own: as its
/// first line that is neither blank nor a line comment. Anywhere else the line
/// could sit inside a block comment, a string or a nested module, where it
/// would not apply to the crate.
fn opens_with_forbid_unsafe(text: &str) -> bool {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("//"))
        == Some(FORBID_UNSAFE)
}

fn toolchain(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let config = match parse(root, &root.join("rust-toolchain.toml")) {
        Ok(config) => config,
        Err(problem) => return vec![problem],
    };
    match lookup(config.as_item(), &["toolchain", "channel"]).and_then(Item::as_str) {
        Some(channel) if is_exact_version(channel) => Vec::new(),
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
    let (workspace_unpublished, mut problems) = match parse(root, &root.join("Cargo.toml")) {
        Ok(workspace) => {
            let publish = lookup(workspace.as_item(), &["workspace", "package", "publish"]);
            (publish.and_then(Item::as_bool) == Some(false), Vec::new())
        }
        Err(problem) => (false, vec![problem]),
    };
    problems.extend(each_manifest(root, members, |dir, manifest| {
        let publish = lookup(manifest.as_item(), &["package", "publish"]);
        let own = publish.and_then(Item::as_bool) == Some(false);
        let inherited = workspace_unpublished
            && publish.is_some_and(|publish| is_true(publish, &["workspace"]));
        (!(own || inherited)).then(|| format!("{dir} does not resolve to publish = false"))
    }));
    problems
}

/// Every dependency of a member crate, in every dependency table including
/// the target-specific ones, is inherited from the workspace table, which is
/// the one place licences and credits are checked.
fn deps(root: &Path, members: &[PathBuf]) -> Vec<String> {
    each_manifest(root, members, |dir, manifest| {
        let mut problems = Vec::new();
        for (table, entries) in dependency_tables(manifest) {
            for (name, entry) in entries.iter() {
                if !is_true(entry, &["workspace"]) {
                    problems.push(format!(
                        "{dir} declares `{name}` directly in [{table}]; use `{name}.workspace = true`"
                    ));
                }
            }
        }
        problems
    })
}

/// Every third-party workspace dependency has a complete row in the
/// prior-art dependency table: a licence, whether it can build C, and for a
/// crate that can, the feature that keeps it pure or why C is allowed.
fn credits(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let path = root.join("docs/prior-art.md");
    if !path.is_file() {
        return vec!["docs/prior-art.md is missing".to_owned()];
    }
    let rows = table_cells(&read(&path));
    let workspace = match parse(root, &root.join("Cargo.toml")) {
        Ok(workspace) => workspace,
        Err(problem) => return vec![problem],
    };
    let Some(dependencies) =
        lookup(workspace.as_item(), &["workspace", "dependencies"]).and_then(Item::as_table_like)
    else {
        return Vec::new();
    };
    dependencies
        .iter()
        .filter(|(_, entry)| lookup(entry, &["path"]).is_none())
        .filter_map(|(name, entry)| uncredited(&rows, name, entry))
        .collect()
}

/// What the prior-art row of workspace dependency `name` lacks, if anything.
/// Its row is the table body row, as GitHub shows it, whose first cell is
/// the name as code.
fn uncredited(rows: &[Vec<String>], name: &str, entry: &Item) -> Option<String> {
    let first = format!("`{name}`");
    let Some(row) = rows.iter().find(|row| row.first() == Some(&first)) else {
        return Some(format!(
            "dependency `{name}` has no row in docs/prior-art.md"
        ));
    };
    let cells: Vec<&str> = row.iter().map(String::as_str).collect();
    let filled = |i: usize| cells.get(i).is_some_and(|c| !c.is_empty() && *c != "—");
    let c_build = cells.get(4).copied().map(first_word);
    let how = cells.get(5).copied().unwrap_or_default();
    let enabled = strings(entry, &["features"]);
    let complete = filled(2)
        && (c_build == Some("no")
            || (c_build == Some("yes") && builds_pure_or_justified(how, &enabled)));
    (!complete).then(|| {
        format!(
            "dependency `{name}` needs a licence, a yes/no C-build cell and, for yes, \
             an enabled pure feature or a `{C_ALLOWED}` justification in docs/prior-art.md"
        )
    })
}

/// A cell's first word without trailing punctuation: `yes` for "yes", "yes,"
/// or "yes (its build script …)".
fn first_word(cell: &str) -> &str {
    cell.split(|c: char| c.is_whitespace() || c == '(')
        .next()
        .unwrap_or_default()
        .trim_end_matches([',', ';', ':', '.'])
}

/// A `yes` row either names features, as `features = ["…"]`, that the
/// workspace entry enables, or starts with `C allowed:` and a reason.
fn builds_pure_or_justified(how: &str, enabled: &[&str]) -> bool {
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
    !named.is_empty() && named.iter().all(|feature| enabled.contains(feature))
}

/// Clippy rejects the randomly seeded hash collections (INT-0005 AC-4): the
/// root `clippy.toml` lists both under `disallowed-types`, and no other
/// config takes its place for any member.
fn clippy(root: &Path, members: &[PathBuf]) -> Vec<String> {
    let mut problems: Vec<String> = match parse(root, &root.join("clippy.toml")) {
        Ok(config) => {
            let listed = disallowed_types(&config);
            DISALLOWED
                .into_iter()
                .filter(|path| !listed.contains(path))
                .map(|path| format!("clippy.toml does not list {path} under disallowed-types"))
                .collect()
        }
        Err(problem) => vec![problem],
    };
    problems.extend(shadowing_configs(root, members).iter().map(|path| {
        format!(
            "{} would be read instead of clippy.toml",
            relative(root, path)
        )
    }));
    problems
}

/// The paths `disallowed-types` lists, as strings or as tables with a `path`.
fn disallowed_types(config: &DocumentMut) -> Vec<&str> {
    match config.get("disallowed-types") {
        Some(Item::Value(Value::Array(entries))) => entries
            .iter()
            .filter_map(|entry| {
                entry
                    .as_str()
                    .or_else(|| entry.as_inline_table()?.get("path")?.as_str())
            })
            .collect(),
        Some(Item::ArrayOfTables(tables)) => tables
            .iter()
            .filter_map(|table| table.get("path")?.as_str())
            .collect(),
        _ => Vec::new(),
    }
}

/// Configs Clippy would read instead of the root `clippy.toml`. Walking up
/// from a crate's directory, it reads the first config it finds, and it
/// prefers `.clippy.toml` to `clippy.toml` in the same directory.
fn shadowing_configs(root: &Path, members: &[PathBuf]) -> BTreeSet<PathBuf> {
    let mut configs = BTreeSet::new();
    let hidden = root.join(".clippy.toml");
    if hidden.is_file() {
        configs.insert(hidden);
    }
    for dir in members {
        for between in dir
            .ancestors()
            .take_while(|dir| *dir != root && dir.starts_with(root))
        {
            configs.extend(
                CLIPPY_CONFIGS
                    .into_iter()
                    .map(|name| between.join(name))
                    .filter(|path| path.is_file()),
            );
        }
    }
    configs
}

fn names(root: &Path, members: &[PathBuf]) -> Vec<String> {
    each_manifest(root, members, |dir, manifest| {
        let name = lookup(manifest.as_item(), &["package", "name"])
            .and_then(Item::as_str)
            .unwrap_or_default();
        (!is_single_word(name))
            .then(|| format!("{dir} package name {name:?} is not a single lowercase word"))
    })
}

fn is_single_word(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

/// The format spec exists, keeps every required section, and its registry
/// and worked-example tables hold rows (INT-0006 AC-6: the spec comes first).
/// The spec is read as GitHub shows it (see `markdown`): a heading or a row
/// that it shows as code, as a comment or as plain text does not count.
fn spec(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let path = root.join(SPEC_PATH);
    if !path.is_file() {
        return vec![format!("{SPEC_PATH} is missing")];
    }
    let blocks = markdown(&read(&path));
    let mut problems: Vec<String> = SPEC_SECTIONS
        .into_iter()
        .filter(|name| {
            !blocks
                .iter()
                .any(|block| matches!(block, Block::Heading(Some(text)) if text == name))
        })
        .map(|name| format!("{SPEC_PATH} has no \"{name}\" section"))
        .collect();
    for (name, code, cells) in SPEC_TABLES {
        if table_rows(&blocks, name, code, cells) == 0 {
            problems.push(format!("{SPEC_PATH} \"{name}\" has no table rows"));
        }
    }
    problems
}

/// Golden encodings must reach every platform byte for byte, so the
/// workspace's `.gitattributes` files must make Git treat them as binary
/// (INT-0006 AC-3, INT-0004 AC-1): `text` unset, as `binary` sets it, and
/// nothing else that rewrites bytes. Later lines and deeper files override
/// earlier ones, so Git evaluates the files (see `attributes`), for every
/// committed golden file and a probe that stands for any added later, inside
/// a checkout or out of one.
fn golden(root: &Path, _members: &[PathBuf]) -> Vec<String> {
    let mut paths: Vec<String> = fs::read_dir(root.join(GOLDEN_DIR))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".bin"))
        .map(|name| format!("{GOLDEN_DIR}/{name}"))
        .collect();
    paths.sort();
    paths.push(GOLDEN_PROBE.to_owned());
    let values = match attributes::values(root, &BYTE_ATTRIBUTES, &paths) {
        Ok(values) => values,
        Err(problem) => return vec![format!("*.bin files cannot be checked: {problem}")],
    };
    // One problem per attribute and value, naming the first path it affects.
    let mut wrong: BTreeMap<(String, String), (String, usize)> = BTreeMap::new();
    for (path, attribute, value) in values {
        if !keeps_bytes(&attribute, &value) {
            wrong.entry((attribute, value)).or_insert((path, 0)).1 += 1;
        }
    }
    wrong
        .into_iter()
        .map(|((attribute, value), (path, count))| {
            format!(
                "{count} *.bin paths are not binary: the .gitattributes files give {path} \
                 `{attribute}: {value}`"
            )
        })
        .collect()
}

/// Whether `attribute: value` leaves a file's bytes as committed.
fn keeps_bytes(attribute: &str, value: &str) -> bool {
    match attribute {
        "text" => value == "unset",
        "ident" => value == "unset" || value == "unspecified",
        _ => value == "unspecified",
    }
}

/// Counts the table body rows in the section headed `name` whose first cell
/// starts with a code span that starts with `code`, and that fill at least
/// `cells` of their table's columns.
fn table_rows(blocks: &[Block], name: &str, code: &str, cells: usize) -> usize {
    let mut inside = false;
    let mut rows = 0;
    for block in blocks {
        match block {
            Block::Heading(heading) => inside = heading.as_deref() == Some(name),
            Block::Row { lead, filled } => {
                let counts = lead.as_deref().is_some_and(|lead| lead.starts_with(code));
                rows += usize::from(inside && counts && *filled >= cells);
            }
        }
    }
    rows
}

/// What the `spec` rule reads in Markdown, in order.
#[derive(Debug)]
enum Block {
    /// A heading, which ends the section above it, with its text when it is
    /// at the top level: one in a list item or a quote names no section.
    Heading(Option<String>),
    /// A body row of a table at the top level: the code span that starts its
    /// first cell, if one does, and how many of its cells hold anything.
    Row { lead: Option<String>, filled: usize },
}

/// The body rows of a Markdown file's top-level tables as GitHub shows them
/// (see `markdown`), each as its cells' text, code spans in backticks.
fn table_cells(text: &str) -> Vec<Vec<String>> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut rows = Vec::new();
    // How many elements are open, whether the open table is at the top
    // level, and the open row's cells, in such a table.
    let mut depth = 0;
    let mut top_table = false;
    let mut row: Option<Vec<String>> = None;
    for event in Parser::new_ext(&text, Options::ENABLE_TABLES) {
        let top = depth == 0;
        match event {
            Event::Start(_) => depth += 1,
            Event::End(_) => depth -= 1,
            _ => {}
        }
        let cell = row.as_mut().and_then(|cells| cells.last_mut());
        match event {
            Event::Start(Tag::Table(_)) => top_table = top,
            Event::Start(Tag::TableRow) if top_table => row = Some(Vec::new()),
            Event::End(TagEnd::TableRow) => rows.extend(row.take()),
            Event::Start(Tag::TableCell) => {
                if let Some(cells) = &mut row {
                    cells.push(String::new());
                }
            }
            Event::Text(part) => cell.into_iter().for_each(|cell| cell.push_str(&part)),
            Event::Code(code) => cell
                .into_iter()
                .for_each(|cell| cell.push_str(&format!("`{code}`"))),
            _ => {}
        }
    }
    rows
}

/// The headings and table body rows of a Markdown file as GitHub shows it:
/// CommonMark with tables, as pulldown-cmark reads them (`book` reads the
/// Book with it too). What it shows as code, in an HTML block or as plain
/// text is neither, and a row has as many cells as its table's header.
fn markdown(text: &str) -> Vec<Block> {
    // CommonMark ends a line at a lone CR too, but pulldown-cmark does not
    // everywhere (see `book::parse_chapter`), so each line ends at an LF.
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut blocks = Vec::new();
    // How many elements are open; whether the open table is at the top level;
    // the open heading, with its text so far when it is at the top level; and
    // the open row's cells, each as the events it holds, in such a table.
    let mut depth = 0;
    let mut top_table = false;
    let mut heading: Option<Option<String>> = None;
    let mut row: Option<Vec<Vec<Event>>> = None;
    for event in Parser::new_ext(&text, Options::ENABLE_TABLES) {
        let top = depth == 0;
        match event {
            Event::Start(_) => depth += 1,
            Event::End(_) => depth -= 1,
            _ => {}
        }
        match event {
            Event::Start(Tag::Heading { .. }) => heading = Some(top.then(String::new)),
            Event::End(TagEnd::Heading(_)) => blocks.extend(heading.take().map(Block::Heading)),
            Event::Start(Tag::Table(_)) => top_table = top,
            Event::Start(Tag::TableRow) if top_table => row = Some(Vec::new()),
            Event::End(TagEnd::TableRow) => {
                blocks.extend(row.take().map(|cells| Block::Row {
                    lead: match cells.first().and_then(|cell| cell.first()) {
                        Some(Event::Code(code)) => Some(code.to_string()),
                        _ => None,
                    },
                    filled: cells.iter().filter(|cell| !cell.is_empty()).count(),
                }));
            }
            Event::Start(Tag::TableCell) => {
                if let Some(cells) = &mut row {
                    cells.push(Vec::new());
                }
            }
            Event::End(TagEnd::TableCell) => {}
            event => {
                if let Some(Some(text)) = &mut heading {
                    match &event {
                        Event::Text(part) | Event::Code(part) => text.push_str(part),
                        Event::SoftBreak | Event::HardBreak => text.push(' '),
                        _ => {}
                    }
                }
                if let Some(cell) = row.as_mut().and_then(|cells| cells.last_mut()) {
                    cell.push(event);
                }
            }
        }
    }
    blocks
}

/// Runs `check` on each member's parsed manifest, passing the member's
/// directory as named relative to the root; a manifest that does not parse
/// is itself the problem.
fn each_manifest<P: IntoIterator<Item = String>>(
    root: &Path,
    members: &[PathBuf],
    mut check: impl FnMut(&str, &DocumentMut) -> P,
) -> Vec<String> {
    let mut problems = Vec::new();
    for dir in members {
        match parse(root, &dir.join("Cargo.toml")) {
            Ok(manifest) => problems.extend(check(&relative(root, dir), &manifest)),
            Err(problem) => problems.push(problem),
        }
    }
    problems
}

/// Parses a TOML file under the root, naming it relative to the root in the
/// problem it reports when it cannot.
fn parse(root: &Path, path: &Path) -> Result<DocumentMut, String> {
    load(path, &relative(root, path))
}

/// Reads a file without its byte-order mark, treating a missing or unreadable
/// file as empty; each rule then reports what it found missing.
fn read(path: &Path) -> String {
    read_text(path).unwrap_or_default()
}

/// `path` relative to the root, with `/` separators; the root itself is `.`.
fn relative(root: &Path, path: &Path) -> String {
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
    if relative.is_empty() {
        ".".to_owned()
    } else {
        relative
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{check, golden, is_exact_version, is_single_word};

    const SPEC: &str = "\
# Format\n## Conventions\n## Primitives\n### Canonical order\n## Hash algorithms\n\
| id | name | digest bytes | status |\n|---|---|---|---|\n| `0x01` | `blake3` | 32 | default |\n\
## Objects\n### Object kinds\n| id | kind | status |\n|---|---|---|\n| `0x01` | `blob` | defined |\n\
### Object identity\n## Worked examples\n| name | value | hex |\n|---|---|---|\n| `bool-true` | true | `01` |\n\
## Signatures (reserved)\n## Decoder requirements\n## Change control\n";

    /// The clean fixture's root manifest.
    const WORKSPACE: &str = "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.package]\npublish = false\n\n\
                             [workspace.dependencies]\nblake3 = { version = \"1\", features = [\"pure\"] }\n\
                             hash = { path = \"crates/hash\" }\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n";

    /// A fixture workspace, in its own directory under the OS temp directory,
    /// that passes every rule until a test breaks one.
    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("xtask-policy-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).unwrap();
            let fixture = Fixture(root);
            fixture.write("Cargo.toml", WORKSPACE);
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

        fn read(&self, path: &str) -> String {
            fs::read_to_string(self.0.join(path)).unwrap()
        }

        /// The rules that fail, with their problems.
        fn failures(&self) -> Vec<(&'static str, Vec<String>)> {
            check(&self.0)
                .into_iter()
                .filter(|result| !result.problems.is_empty())
                .map(|result| (result.rule, result.problems))
                .collect()
        }

        fn passes(&self) {
            let failures = self.failures();
            assert!(failures.is_empty(), "{failures:?}");
        }

        /// Exactly `rules` fail, in report order, each with a problem that
        /// mentions every one of `needles`.
        fn fails_exactly(&self, rules: &[&str], needles: &[&str]) {
            let failures = self.failures();
            let failed: Vec<&str> = failures.iter().map(|(rule, _)| *rule).collect();
            assert_eq!(failed, rules, "{failures:?}");
            for (rule, problems) in &failures {
                assert!(
                    problems
                        .iter()
                        .any(|problem| needles.iter().all(|needle| problem.contains(needle))),
                    "{rule}: {problems:?} should mention {needles:?}"
                );
            }
        }

        fn fails_only(&self, rule: &str, needle: &str) {
            self.fails_exactly(&[rule], &[needle]);
        }

        /// `cargo policy` run on the fixture: its exit code, stdout and stderr.
        fn command(&self) -> (u8, String, String) {
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let code = crate::cli(&[OsString::from("policy")], &self.0, &mut out, &mut err);
            (
                code,
                String::from_utf8(out).unwrap(),
                String::from_utf8(err).unwrap(),
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            // On Windows the emptied directory can stay in use for a moment
            // after Git has run in it, so removal backs off for up to about
            // 9.5 s. The first try normally succeeds.
            for attempt in 0..100 {
                if fs::remove_dir_all(&self.0).is_ok() || !self.0.exists() {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(10 * attempt.min(10)));
            }
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

    /// `text` with `from` replaced by `to`. `from` must occur, so that a test
    /// cannot silently stop planting its defect.
    fn replace(text: &str, from: &str, to: &str) -> String {
        assert!(text.contains(from), "{from:?} is not in {text:?}");
        text.replace(from, to)
    }

    #[test]
    fn fixture_passes_every_rule() {
        Fixture::new("clean").passes();
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

        // A member's own `[lints]` table replaces the workspace lints.
        let fixture = Fixture::new("lints-own-table");
        fixture.write(
            "crates/hash/Cargo.toml",
            &replace(
                &member("hash", "blake3.workspace = true"),
                "[lints]\nworkspace = true\n",
                "[lints.rust]\nmissing_docs = \"warn\"\n",
            ),
        );
        fixture.fails_only("lints", "crates/hash");

        // The inherited lint is the guard for tests, build scripts, binaries
        // and examples, which the `unsafe` rule never reads. Only
        // `unsafe_code` itself, at level `forbid`, is that guard.
        let forbid = "[workspace.lints.rust]\nunsafe_code = \"forbid\"\n";
        for (i, lints) in [
            "",
            "[workspace.lints.rust]\nunsafe_code = \"allow\"\n",
            "[workspace.lints.rust]\nunsafe_code = \"deny\"\n",
            "[workspace.lints.rust]\nunsafe_code = { level = \"allow\", priority = -1 }\n",
            "[workspace.lints.rust]\n# unsafe_code = \"forbid\"\n",
            "[workspace.lints.rust]\nunsafe_op_in_unsafe_fn = \"forbid\"\n",
            "[workspace.lints.clippy]\nunsafe_code = \"forbid\"\n",
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("lints-workspace-{i}"));
            fixture.write("Cargo.toml", &replace(WORKSPACE, forbid, lints));
            fixture.fails_only("lints", "unsafe_code");
        }

        let fixture = Fixture::new("lints-workspace-level");
        fixture.write(
            "Cargo.toml",
            &replace(
                WORKSPACE,
                "unsafe_code = \"forbid\"",
                "unsafe_code = { level = \"forbid\", priority = -1 }",
            ),
        );
        fixture.passes();
    }

    #[test]
    fn policy_flags_missing_forbid_unsafe() {
        let fixture = Fixture::new("unsafe");
        fixture.write("crates/hash/src/main.rs", "fn main() {}\n");
        fixture.fails_only("unsafe", "crates/hash/src/main.rs");

        let fixture = Fixture::new("unsafe-lib");
        fixture.write(
            "crates/hash/src/lib.rs",
            "//! A library without the attribute.\n",
        );
        fixture.fails_only("unsafe", "crates/hash/src/lib.rs");

        // A crate with both roots needs the attribute in each.
        let fixture = Fixture::new("unsafe-lib-beside-main");
        fixture.write(
            "crates/hash/src/main.rs",
            "#![forbid(unsafe_code)]\nfn main() {}\n",
        );
        fixture.write("crates/hash/src/lib.rs", "pub fn f() {}\n");
        fixture.fails_only("unsafe", "crates/hash/src/lib.rs");

        // The line counts only as the crate root's first line that is neither
        // blank nor a line comment: anywhere else it could sit inside a block
        // comment, a string or a nested module.
        for (i, text) in [
            "/*\n#![forbid(unsafe_code)]\n*/\npub fn f() {}\n",
            "mod inner {\n    #![forbid(unsafe_code)]\n}\n",
            "#![allow(dead_code)] /*\n#![forbid(unsafe_code)]\n*/\n",
            "#![doc = \"\n#![forbid(unsafe_code)]\n\"]\n",
            "#![doc = \"]\n#![forbid(unsafe_code)]\n\"]\n",
            "#![deny(missing_docs)]\n#![forbid(unsafe_code)]\n",
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("unsafe-hidden-{i}"));
            fixture.write("crates/hash/src/lib.rs", text);
            fixture.fails_only("unsafe", "crates/hash/src/lib.rs");
        }
        let fixture = Fixture::new("unsafe-after-docs");
        fixture.write(
            "crates/hash/src/lib.rs",
            "//! Docs come first.\n\n// A plain comment too.\n#![forbid(unsafe_code)]\n#![deny(missing_docs)]\n",
        );
        fixture.passes();
    }

    #[test]
    fn policy_requires_exact_toolchain() {
        // Exactly three parts, each a run of decimal digits.
        for (i, channel) in [
            "stable",
            "1.99",
            "nightly-2026-10-01",
            "1.99.1-beta",
            "1.99.1.0",
            "1.99.",
            "1.99.x",
        ]
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

        // Without a channel nothing is pinned (INT-0003 AC-2): rustup falls
        // back to its default toolchain.
        let fixture = Fixture::new("toolchain-missing");
        fs::remove_file(fixture.0.join("rust-toolchain.toml")).unwrap();
        fixture.fails_only("toolchain", "rust-toolchain.toml is missing");

        let fixture = Fixture::new("toolchain-no-channel");
        fixture.write(
            "rust-toolchain.toml",
            "[toolchain]\ncomponents = [\"clippy\"]\n",
        );
        fixture.fails_only(
            "toolchain",
            "rust-toolchain.toml has no [toolchain] channel",
        );
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

        // `true`, or a list of registries to publish to.
        for (i, publish) in ["publish = true", "publish = [\"crates-io\"]"]
            .into_iter()
            .enumerate()
        {
            let fixture = Fixture::new(&format!("publish-own-{i}"));
            fixture.write(
                "crates/hash/Cargo.toml",
                &replace(
                    &member("hash", "blake3.workspace = true"),
                    "publish.workspace = true",
                    publish,
                ),
            );
            fixture.fails_only("publish", "crates/hash");
        }

        // A member that inherits gets the workspace value, whatever it is.
        for (i, package) in [
            "[workspace.package]\npublish = true\n",
            "[workspace.package]\n",
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("publish-workspace-{i}"));
            fixture.write(
                "Cargo.toml",
                &replace(WORKSPACE, "[workspace.package]\npublish = false\n", package),
            );
            fixture.fails_only("publish", "crates/hash");
        }

        for (i, publish) in ["publish = false", "publish = { workspace = true }"]
            .into_iter()
            .enumerate()
        {
            let fixture = Fixture::new(&format!("publish-ok-{i}"));
            fixture.write(
                "crates/hash/Cargo.toml",
                &replace(
                    &member("hash", "blake3.workspace = true"),
                    "publish.workspace = true",
                    publish,
                ),
            );
            fixture.passes();
        }
    }

    #[test]
    fn policy_flags_direct_dependency() {
        for table in [
            "dependencies",
            "dev-dependencies",
            "build-dependencies",
            "dev_dependencies",
            "build_dependencies",
        ] {
            let fixture = Fixture::new(&format!("deps-{table}"));
            fixture.write(
                "crates/hash/Cargo.toml",
                &format!(
                    "[package]\nname = \"hash\"\npublish.workspace = true\n\n[{table}]\nfoo = \"1\"\n\n\
                     [lints]\nworkspace = true\n"
                ),
            );
            let header = format!("[{table}]");
            fixture.fails_exactly(&["deps"], &["crates/hash", "`foo`", header.as_str()]);
        }
        // Each form names the crate, the dependency and the table it is in.
        for (i, (table, name, header)) in [
            (
                "[dependencies.foo]\nversion = \"1\"\n",
                "`foo`",
                "[dependencies]",
            ),
            (
                "[dependencies]\nfoo = { version = \"1\", workspace = false }\n",
                "`foo`",
                "[dependencies]",
            ),
            // An inherited entry does not excuse the entries after it.
            (
                "[dependencies]\nblake3.workspace = true\nfoo = \"1\"\n",
                "`foo`",
                "[dependencies]",
            ),
            (
                "[target.'cfg(windows)'.dependencies]\nwindows-sys = \"0.59\"\n",
                "`windows-sys`",
                "[target.'cfg(windows)'.dependencies]",
            ),
            (
                "[target.'cfg(unix)'.build-dependencies]\ncc = \"1\"\n",
                "`cc`",
                "[target.'cfg(unix)'.build-dependencies]",
            ),
            (
                "[target.'cfg(unix)'.dev-dependencies.libc]\nversion = \"0.2\"\n",
                "`libc`",
                "[target.'cfg(unix)'.dev-dependencies]",
            ),
            // Before edition 2024, Cargo reads the underscore spellings under
            // a target too.
            (
                "[target.'cfg(unix)'.dev_dependencies]\nlibc = \"0.2\"\n",
                "`libc`",
                "[target.'cfg(unix)'.dev_dependencies]",
            ),
            (
                "[target.'cfg(unix)'.build_dependencies]\ncc = \"1\"\n",
                "`cc`",
                "[target.'cfg(unix)'.build_dependencies]",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("deps-table-{i}"));
            fixture.write(
                "crates/hash/Cargo.toml",
                &format!(
                    "[package]\nname = \"hash\"\npublish.workspace = true\n\n{table}\n[lints]\nworkspace = true\n"
                ),
            );
            fixture.fails_exactly(&["deps"], &["crates/hash", name, header]);
        }

        // A top-level inline table declares dependencies too.
        let fixture = Fixture::new("deps-inline-table");
        fixture.write(
            "crates/hash/Cargo.toml",
            "dependencies = { foo = \"1\" }\n\n[package]\nname = \"hash\"\npublish.workspace = true\n\n\
             [lints]\nworkspace = true\n",
        );
        fixture.fails_exactly(&["deps"], &["crates/hash", "`foo`", "[dependencies]"]);

        let fixture = Fixture::new("deps-target-inherited");
        fixture.write(
            "crates/hash/Cargo.toml",
            &format!(
                "{}\n[target.'cfg(windows)'.dependencies]\nwindows-sys.workspace = true\n",
                member("hash", "blake3.workspace = true")
            ),
        );
        fixture.passes();
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

        // A row counts as GitHub shows it: in a comment, a fence or a quote,
        // or with the name not as code, it is no row.
        let header = "| Crate | Version | Licence | Used by | Can build C | How |\n|---|---|---|---|---|---|\n";
        let row =
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | `features = [\"pure\"]`: Rust only |\n";
        let table = format!("{header}{row}");
        for (i, text) in [
            format!("{header}\n<!--\n{row}-->\n"),
            format!("{header}\n```\n{row}```\n"),
            table.lines().map(|line| format!("> {line}\n")).collect(),
            format!("- note\n\n  {}", table.replace('\n', "\n  ")),
            table.replace("| `blake3` |", "| blake3 |"),
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("credits-hidden-{i}"));
            assert_eq!(fixture.read("docs/prior-art.md"), table);
            fixture.write("docs/prior-art.md", &text);
            fixture.fails_only("credits", "`blake3` has no row");
        }
        // The same table after other text, or with CRLF line ends, or with a
        // lone CR ending each line of a fence and a comment before it, is read.
        for (i, text) in [
            format!("# Credits\n\n{table}"),
            table.replace('\n', "\r\n"),
            format!("```\ncode\n```\n<!--\nnote\n-->\n{table}").replace('\n', "\r"),
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("credits-shown-{i}"));
            fixture.write("docs/prior-art.md", &text);
            fixture.passes();
        }

        // A `[workspace.dependencies.<name>]` table declares a dependency too.
        let fixture = Fixture::new("credits-table");
        fixture.write(
            "Cargo.toml",
            &format!("{WORKSPACE}\n[workspace.dependencies.zstd]\nversion = \"0.13\"\n"),
        );
        fixture.fails_only("credits", "`zstd`");

        // So do a bare version and a Git source: only `path` entries are
        // workspace crates.
        for (i, entry) in [
            "zstd = \"0.13\"",
            "zstd = { git = \"https://github.com/gyscos/zstd-rs\" }",
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("credits-source-{i}"));
            fixture.write(
                "Cargo.toml",
                &replace(
                    WORKSPACE,
                    "[workspace.dependencies]\n",
                    &format!("[workspace.dependencies]\n{entry}\n"),
                ),
            );
            fixture.fails_only("credits", "`zstd`");
        }

        // The `blake3` row is not a row for `blake`.
        let fixture = Fixture::new("credits-prefix-name");
        fixture.write(
            "Cargo.toml",
            &replace(
                WORKSPACE,
                "[workspace.dependencies]\n",
                "[workspace.dependencies]\nblake = { version = \"1\", features = [\"pure\"] }\n",
            ),
        );
        fixture.fails_only("credits", "`blake`");
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
            "| `blake3` | 1 | CC0-1.0 | `hash` | not verified | — |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | none | — |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | not checked | — |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | nope | — |",
            // "—" marks an empty cell, as in the designs tables.
            "| `blake3` | 1 | — | `hash` | yes | `features = [\"pure\"]` |",
            // A `yes` row's last cell starts with `C allowed:`, or names
            // features the entry enables: at least one, each in full.
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | Rust only (C allowed: never) |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | `features = []` |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | `features = [\"pur\"]` |",
        ];
        let header = "| Crate | Version | Licence | Used by | Can build C | How |\n|---|---|---|---|---|---|\n";
        for (i, row) in failing.into_iter().enumerate() {
            let fixture = Fixture::new(&format!("credits-row-{i}"));
            fixture.write("docs/prior-art.md", &format!("{header}{row}\n"));
            fixture.fails_only("credits", "`blake3` needs a licence");
        }
        let passing = [
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | `features = [\"pure\"]` |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes | C allowed: no pure-Rust option exists |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | no | — |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | yes (its build script compiles C) | `features = [\"pure\"]` |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | no (its build script only generates tests) | — |",
            "| `blake3` | 1 | CC0-1.0 | `hash` | no. | — |",
        ];
        for (i, row) in passing.into_iter().enumerate() {
            let fixture = Fixture::new(&format!("credits-ok-{i}"));
            fixture.write("docs/prior-art.md", &format!("{header}{row}\n"));
            assert!(
                fixture.failures().is_empty(),
                "{row}: {:?}",
                fixture.failures()
            );
        }

        // The enabled features come from the workspace entry in any TOML form.
        let fixture = Fixture::new("credits-features-table");
        fixture.write(
            "Cargo.toml",
            &format!(
                "{}\n[workspace.dependencies.blake3]\nversion = \"1\"\nfeatures = [\"std\"]\n",
                replace(
                    WORKSPACE,
                    "blake3 = { version = \"1\", features = [\"pure\"] }\n",
                    ""
                )
            ),
        );
        fixture.fails_only("credits", "`blake3`");
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

        // Clippy reads TOML: a commented-out entry, or one under another key,
        // disallows nothing.
        for (name, config, missing) in [
            (
                "commented",
                "disallowed-types = [\n  { path = \"std::collections::HashMap\" },\n  \
                 # { path = \"std::collections::HashSet\" },\n]\n",
                "HashSet",
            ),
            (
                "other-key",
                "disallowed-types = []\ndisallowed-methods = [\n  \"std::collections::HashMap\",\n  \
                 \"std::collections::HashSet\",\n]\n",
                "HashMap",
            ),
        ] {
            let fixture = Fixture::new(&format!("clippy-{name}"));
            fixture.write("clippy.toml", config);
            fixture.fails_only("clippy", missing);
        }

        // Only the `std` paths ban the `std` types: neither another crate's
        // types of the same names nor the bare names do.
        for (i, config) in [
            "disallowed-types = [\"hashbrown::HashMap\", \"hashbrown::HashSet\"]\n",
            "disallowed-types = [\"HashMap\", \"HashSet\"]\n",
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("clippy-not-std-{i}"));
            fixture.write("clippy.toml", config);
            for path in ["std::collections::HashMap", "std::collections::HashSet"] {
                fixture.fails_only("clippy", path);
            }
        }

        // Clippy reads the first config it finds walking up from a crate,
        // preferring `.clippy.toml`, so any other config on that path wins.
        for shadow in [
            ".clippy.toml",
            "crates/clippy.toml",
            "crates/hash/clippy.toml",
            "crates/hash/.clippy.toml",
        ] {
            let fixture = Fixture::new(&format!("clippy-shadow-{}", shadow.replace('/', "-")));
            fixture.write(shadow, &fixture.read("clippy.toml"));
            fixture.fails_only("clippy", shadow);
        }

        for (i, config) in [
            "disallowed-types = [\"std::collections::HashMap\", \"std::collections::HashSet\"]\n",
            "[[disallowed-types]]\npath = \"std::collections::HashMap\"\n\n\
             [[disallowed-types]]\npath = \"std::collections::HashSet\"\n",
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("clippy-ok-{i}"));
            fixture.write("clippy.toml", config);
            fixture.passes();
        }
    }

    #[test]
    fn policy_requires_single_word_names() {
        // ASCII lowercase letters and digits only, from the first letter on.
        for (i, name) in ["peri-hash", "Hash", "hash_ref", "9hash", "hashRef", "café"]
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
        fixture.fails_exactly(&["golden"], &["*.bin", "text: auto"]);

        let fixture = Fixture::new("golden-text");
        fixture.write(".gitattributes", "* text=auto eol=lf\n*.bin text\n");
        fixture.fails_exactly(&["golden"], &["*.bin", "text: set"]);

        // `text=false` gives `text` a value, which is not unsetting it: the
        // `eol` that `*` sets still converts line endings.
        let fixture = Fixture::new("golden-text-false");
        fixture.write(".gitattributes", "* text=auto eol=lf\n*.bin text=false\n");
        fixture.fails_exactly(&["golden"], &["*.bin", "text: false"]);

        // Git lets a later line, or a deeper `.gitattributes`, override
        // `*.bin binary`.
        let fixture = Fixture::new("golden-later-line");
        fixture.write(
            ".gitattributes",
            "* text=auto eol=lf\n*.bin binary\ncrates/codec/tests/golden/**/*.bin text eol=crlf\n",
        );
        fixture.fails_exactly(&["golden"], &["*.bin", "text: set"]);

        let fixture = Fixture::new("golden-nested");
        fixture.write(
            "crates/codec/tests/golden/.gitattributes",
            "*.bin text eol=crlf\n",
        );
        fixture.fails_exactly(&["golden"], &["*.bin", "text: set"]);

        // Only the files decide: the fixtures are not checkouts, and a broken
        // `.git` changes nothing.
        let fixture = Fixture::new("golden-broken-checkout");
        fixture.write(".git", "gitdir: nowhere\n");
        fixture.passes();
        fixture.write(".gitattributes", "* text=auto eol=lf\n");
        fixture.fails_exactly(&["golden"], &["*.bin", "text: auto"]);

        // Nothing marks the files at all.
        for (i, attributes) in ["", "* eol=lf\n"].into_iter().enumerate() {
            let fixture = Fixture::new(&format!("golden-unspecified-{i}"));
            fixture.write(".gitattributes", attributes);
            fixture.fails_exactly(&["golden"], &["*.bin", "text: unspecified"]);
        }

        // The deepest level, where the files themselves live.
        let fixture = Fixture::new("golden-nested-v1");
        fixture.write(
            "crates/codec/tests/golden/v1/.gitattributes",
            "*.bin text\n",
        );
        fixture.fails_exactly(&["golden"], &["*.bin", "text: set"]);

        // `binary` alone is not enough when another attribute rewrites bytes.
        for (i, (extra, needle)) in [
            ("filter=lfs", "filter: lfs"),
            ("ident", "ident: set"),
            (
                "working-tree-encoding=UTF-16LE",
                "working-tree-encoding: UTF-16LE",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("golden-rewrites-{i}"));
            fixture.write(
                ".gitattributes",
                &format!("* text=auto eol=lf\n*.bin binary {extra}\n"),
            );
            fixture.fails_exactly(&["golden"], &["*.bin", needle]);
        }

        // A committed golden file is checked, not only the probe.
        let fixture = Fixture::new("golden-real-file");
        fixture.write("crates/codec/tests/golden/v1/u32-max.bin", "x");
        fixture.write(".gitattributes", "* text=auto eol=lf\nprobe.bin binary\n");
        fixture.fails_exactly(&["golden"], &["u32-max.bin", "text: auto"]);

        // Every committed golden file, not only the first or the last.
        let fixture = Fixture::new("golden-every-file");
        for name in ["blob-envelope", "u32-max", "uvarint-0"] {
            fixture.write(&format!("crates/codec/tests/golden/v1/{name}.bin"), "x");
        }
        fixture.write(
            ".gitattributes",
            "* text=auto eol=lf\n*.bin binary\nu32-max.bin text\n",
        );
        fixture.fails_exactly(&["golden"], &["u32-max.bin", "text: set"]);

        // A file Git warns about, here a macro defined below the root (which
        // Git ignores), fails rather than passing on what Git chose to skip.
        let fixture = Fixture::new("golden-warning");
        fixture.write("crates/.gitattributes", "[attr]golden binary\n");
        fixture.fails_exactly(&["golden"], &["*.bin", "not allowed"]);

        // When Git cannot answer, the rule fails.
        let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("no-such-workspace");
        let problems = golden(&missing, &[]);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with("*.bin files cannot be checked: "),
            "{problems:?}"
        );
    }

    /// The command itself, not only the rules: "`cargo policy` SHALL fail
    /// with the new rule `golden`" (T-005). It prints one line per rule, in
    /// the order the T-001, T-002 and T-005 clauses add them, a `FAIL` line
    /// for each problem, nothing on stderr, and exits 1.
    #[test]
    fn policy_command_fails_naming_each_failing_rule() {
        const GOLDEN: &str = "FAIL golden: 1 *.bin paths are not binary: the .gitattributes \
                              files give crates/codec/tests/golden/v1/probe.bin `text: auto`\n";
        let fixture = Fixture::new("command-fails");
        fixture.write(".gitattributes", "* text=auto eol=lf\n");
        let report = format!(
            "PASS lints\nPASS unsafe\nPASS toolchain\nPASS lockfile\nPASS publish\nPASS deps\n\
             PASS credits\nPASS clippy\nPASS names\nPASS spec\n{GOLDEN}"
        );
        assert_eq!(fixture.command(), (1, report, String::new()));

        // Two failing rules: each FAIL line takes its rule's place.
        fs::remove_file(fixture.0.join("Cargo.lock")).unwrap();
        let report = format!(
            "PASS lints\nPASS unsafe\nPASS toolchain\nFAIL lockfile: Cargo.lock is missing\n\
             PASS publish\nPASS deps\nPASS credits\nPASS clippy\nPASS names\nPASS spec\n{GOLDEN}"
        );
        assert_eq!(fixture.command(), (1, report, String::new()));

        // One rule with two problems: a FAIL line for each.
        let fixture = Fixture::new("command-two-problems");
        fixture.write(".gitattributes", "* text=auto eol=lf\n*.bin filter=lfs\n");
        let report = format!(
            "PASS lints\nPASS unsafe\nPASS toolchain\nPASS lockfile\nPASS publish\nPASS deps\n\
             PASS credits\nPASS clippy\nPASS names\nPASS spec\n\
             FAIL golden: 1 *.bin paths are not binary: the .gitattributes files give \
             crates/codec/tests/golden/v1/probe.bin `filter: lfs`\n{GOLDEN}"
        );
        assert_eq!(fixture.command(), (1, report, String::new()));
    }

    #[test]
    fn policy_requires_format_spec() {
        let fixture = Fixture::new("spec-missing");
        fs::remove_file(fixture.0.join("docs/spec/format-v1.md")).unwrap();
        fixture.fails_only("spec", "format-v1.md is missing");

        let fixture = Fixture::new("spec-section");
        fixture.write(
            "docs/spec/format-v1.md",
            &replace(SPEC, "### Object identity\n", ""),
        );
        fixture.fails_only("spec", "Object identity");

        let fixture = Fixture::new("spec-algorithms");
        fixture.write(
            "docs/spec/format-v1.md",
            &replace(SPEC, "| `0x01` | `blake3` | 32 | default |\n", ""),
        );
        fixture.fails_only("spec", "Hash algorithms");

        let fixture = Fixture::new("spec-kinds");
        fixture.write(
            "docs/spec/format-v1.md",
            &replace(SPEC, "| `0x01` | `blob` | defined |\n", ""),
        );
        fixture.fails_only("spec", "Object kinds");

        let fixture = Fixture::new("spec-examples");
        fixture.write(
            "docs/spec/format-v1.md",
            &replace(SPEC, "| `bool-true` | true | `01` |\n", ""),
        );
        fixture.fails_only("spec", "Worked examples");

        // A row without its status cell, or with that cell empty, is not a
        // row; nor is a worked example without its hex, nor a registry row
        // that does not start with a `0x..` id.
        let short_rows = [
            (
                "| `0x01` | `blake3` | 32 | default |",
                "| `01` | `blake3` | 32 | default |",
                "Hash algorithms",
            ),
            (
                "| `0x01` | `blob` | defined |",
                "| `01` | `blob` | defined |",
                "Object kinds",
            ),
            (
                "| `0x01` | `blob` | defined |",
                "| `blob` | `0x01` | defined |",
                "Object kinds",
            ),
            (
                "| `0x01` | `blake3` | 32 | default |",
                "| `0x01` | `blake3` | 32 |",
                "Hash algorithms",
            ),
            (
                "| `0x01` | `blake3` | 32 | default |",
                "| `0x01` | `blake3` | 32 |  |",
                "Hash algorithms",
            ),
            (
                "| `0x01` | `blob` | defined |",
                "| `0x01` | `blob` |",
                "Object kinds",
            ),
            (
                "| `0x01` | `blob` | defined |",
                "| `0x01` | `blob` |  |",
                "Object kinds",
            ),
            (
                "| `bool-true` | true | `01` |",
                "| `bool-true` | true |",
                "Worked examples",
            ),
        ];
        for (i, (row, short, table)) in short_rows.into_iter().enumerate() {
            let fixture = Fixture::new(&format!("spec-short-row-{i}"));
            fixture.write("docs/spec/format-v1.md", &replace(SPEC, row, short));
            fixture.fails_only("spec", &format!("\"{table}\" has no table rows"));
        }

        // Every section the T-002 clause names is required, each on its own.
        let sections = [
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
        for (i, section) in sections.into_iter().enumerate() {
            let heading = SPEC
                .lines()
                .find(|line| line.trim_start_matches('#').trim() == section)
                .unwrap();
            let fixture = Fixture::new(&format!("spec-section-{i}"));
            fixture.write(
                "docs/spec/format-v1.md",
                &replace(SPEC, &format!("{heading}\n"), ""),
            );
            fixture.fails_only("spec", &format!("no \"{section}\" section"));

            // A heading that only starts with the name, or mentions it, is
            // another section.
            let fixture = Fixture::new(&format!("spec-renamed-{i}"));
            fixture.write(
                "docs/spec/format-v1.md",
                &replace(
                    SPEC,
                    &format!("{heading}\n"),
                    &format!("## {section} notes\n"),
                ),
            );
            fixture.fails_only("spec", &format!("no \"{section}\" section"));
        }

        // A row counts only in its own section: not above the first heading,
        // nor under a later heading that starts with the section's name.
        let row = "| `0x01` | `blob` | defined |\n";
        let table = format!("| id | kind | status |\n|---|---|---|\n{row}");
        let fixture = Fixture::new("spec-row-outside");
        fixture.write(
            "docs/spec/format-v1.md",
            &format!(
                "{table}\n{}### Object kinds (retired)\n{table}",
                replace(SPEC, row, "")
            ),
        );
        fixture.fails_only("spec", "\"Object kinds\" has no table rows");

        // A spec that is not UTF-8 has none of the sections.
        let fixture = Fixture::new("spec-not-utf8");
        fs::write(fixture.0.join("docs/spec/format-v1.md"), [0xff, 0xfe, 0x00]).unwrap();
        fixture.fails_only("spec", "docs/spec/format-v1.md");

        // Each missing section and each empty table is named, and nothing
        // else.
        let fixture = Fixture::new("spec-every-problem");
        let text = replace(SPEC, "## Conventions\n", "");
        let text = replace(&text, "## Change control\n", "");
        fixture.write("docs/spec/format-v1.md", &replace(&text, row, ""));
        let problems = [
            "docs/spec/format-v1.md has no \"Conventions\" section",
            "docs/spec/format-v1.md has no \"Change control\" section",
            "docs/spec/format-v1.md \"Object kinds\" has no table rows",
        ];
        assert_eq!(
            fixture.failures(),
            [("spec", problems.map(str::to_owned).to_vec())]
        );

        // Nothing more is required: a byte-order mark before the first
        // heading, a closing run of `#`, trailing spaces, a deeper heading
        // and more columns than the rule needs all pass.
        let hash_table = "| id | name | digest bytes | status |\n|---|---|---|---|\n\
                          | `0x01` | `blake3` | 32 | default |";
        for (i, text) in [
            format!("\u{feff}{}", replace(SPEC, "# Format\n", "")),
            replace(SPEC, "## Change control\n", "## Change control ##\n"),
            replace(SPEC, "## Change control\n", "## Change control \n"),
            replace(SPEC, "## Change control\n", "## Change control ## \n"),
            replace(SPEC, "## Change control\n", "#### Change control\n"),
            replace(
                SPEC,
                hash_table,
                "| id | name | digest bytes | status | notes |\n|---|---|---|---|---|\n\
                 | `0x01` | `blake3` | 32 | default | BLAKE3-256 |",
            ),
            // A cell past the header's columns is not shown, but the row
            // still has its four.
            replace(
                SPEC,
                "| `0x01` | `blake3` | 32 | default |",
                "| `0x01` | `blake3` | 32 | default | BLAKE3-256 |",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("spec-ok-{i}"));
            fixture.write("docs/spec/format-v1.md", &text);
            fixture.passes();
        }
    }

    /// The spec counts as GitHub shows it, CommonMark with tables: what it
    /// shows as code, as a comment, as plain text or not at all is no section
    /// and no row.
    #[test]
    fn policy_reads_the_spec_as_markdown() {
        let change = "## Change control\n";
        let row = "| `0x01` | `blob` | defined |\n";
        let kinds = "| id | kind | status |\n|---|---|---|\n";
        let no_kinds = "\"Object kinds\" has no table rows";
        let missing = [
            // A heading takes one to six `#` and then a space.
            (
                "## Conventions\n",
                "##Conventions\n",
                "no \"Conventions\" section",
            ),
            (
                "### Object kinds\n",
                "###Object kinds\n",
                "no \"Object kinds\" section",
            ),
            (
                "## Signatures (reserved)\n",
                "####### Signatures (reserved)\n",
                "no \"Signatures (reserved)\" section",
            ),
            // Inside a fenced code block, closed or not, or an HTML comment.
            (
                change,
                "```text\n## Change control\n```\n",
                "Change control",
            ),
            (change, "~~~\n## Change control\n", "Change control"),
            (change, "<!--\n## Change control\n-->\n", "Change control"),
            // Only a run of at least as many of the same marker, indented
            // less than four columns, closes a fence.
            (
                change,
                "````\n```\n## Change control\n````\n",
                "Change control",
            ),
            (
                change,
                "~~~\n```\n## Change control\n~~~\n",
                "Change control",
            ),
            (
                change,
                "```\n    ```\n## Change control\n```\n",
                "Change control",
            ),
            (row, "```\n| `0x01` | `blob` | defined |\n```\n", no_kinds),
            (row, "<!--\n| `0x01` | `blob` | defined |\n-->\n", no_kinds),
            (
                "| id | name | digest bytes | status |\n",
                "```text\n| id | name | digest bytes | status |\n",
                "\"Hash algorithms\" has no table rows",
            ),
            // Indented four columns, or by a tab, after a blank line: code.
            (change, "\n    ## Change control\n", "Change control"),
            (change, "\n\t## Change control\n", "Change control"),
            (row, "\n    | `0x01` | `blob` | defined |\n", no_kinds),
            // A row needs a table: a header row, then a delimiter row with as
            // many columns indented less than four, and then rows up to a
            // blank line or a row without a cell. Lines that go on with a
            // list item's text are none of these.
            (kinds, "", no_kinds),
            (kinds, "| id | kind | status |\n|---|---|\n", no_kinds),
            (kinds, "| id | kind | status |\n|---||---|\n", no_kinds),
            (
                kinds,
                "| id | kind | status |\n    |---|---|---|\n",
                no_kinds,
            ),
            (kinds, "| id | kind | status |\n|---|---|---|\n\n", no_kinds),
            (row, "|\n| `0x01` | `blob` | defined |\n", no_kinds),
            (
                kinds,
                "- note\n| id | kind | status |\n|---|---|---|\n",
                no_kinds,
            ),
            // An escaped pipe splits no cells.
            (kinds, "| id \\| kind | status |\n|---|---|---|\n", no_kinds),
            (row, "| `0x01` \\| `blob` | defined |\n", no_kinds),
            // A cell past the header's columns is not shown: here the status.
            (
                "| id | name | digest bytes | status |\n|---|---|---|---|\n",
                "| id | name | digest bytes |\n|---|---|---|\n",
                "\"Hash algorithms\" has no table rows",
            ),
            // Other HTML hides what follows it up to a blank line, so the
            // reading stops there.
            (
                change,
                "<div>\n## Change control\n</div>\n",
                "Change control",
            ),
            // A fence or comment indented as if in a list might be in one,
            // and then a later fence at the top level is an opening fence.
            (
                change,
                "- note\n  ```\ntext\n```\n## Change control\n```\n",
                "Change control",
            ),
            (
                change,
                "- note\n  <!--\n```\n-->\n## Change control\n```\n",
                "Change control",
            ),
            (
                change,
                "- ```\n  ## Change control\n  ```\n",
                "Change control",
            ),
            // Renderers disagree on whether a tab may follow a closing fence.
            (change, "```\n```\t\n## Change control\n", "Change control"),
            (
                change,
                "<div>\n-->\n## Change control\n</div>\n",
                "Change control",
            ),
            (
                change,
                "<!--\nretired\n## Change control\n-->\n",
                "Change control",
            ),
            (change, "</div>\n## Change control\n", "Change control"),
            (change, "<?x\n## Change control\n?>\n", "Change control"),
            (
                change,
                "\n\t```\n```\n## Change control\n```\n",
                "Change control",
            ),
            (kinds, "| id | kind |\n|---|---|---|\n", no_kinds),
            (
                kinds,
                "\n    | id | kind | status\n|---|---|---|---|\n",
                no_kinds,
            ),
            // A heading in a list or a quote, or underlined, ends the section
            // above it, and one in a list or a quote names none.
            (row, &format!("- ### Retired\n\n{kinds}{row}"), no_kinds),
            (row, &format!("* ### Retired\n\n{kinds}{row}"), no_kinds),
            (row, &format!("1. ### Retired\n\n{kinds}{row}"), no_kinds),
            (row, &format!("> ### Retired\n\n{kinds}{row}"), no_kinds),
            (row, &format!("\nRetired\n---\n\n{kinds}{row}"), no_kinds),
            (row, &format!("\nRetired\n===\n\n{kinds}{row}"), no_kinds),
            (change, "> ## Change control\n", "Change control"),
            (change, "- ## Change control\n", "Change control"),
            // A table in a list or a quote holds no rows the rule counts.
            (
                &format!("{kinds}{row}"),
                "> | id | kind | status |\n> |---|---|---|\n> | `0x01` | `blob` | defined |\n",
                no_kinds,
            ),
            (
                &format!("{kinds}{row}"),
                "- | id | kind | status |\n  |---|---|---|\n  | `0x01` | `blob` | defined |\n",
                no_kinds,
            ),
            // A registry row starts with its id as code.
            (row, "| 0x01 | `blob` | defined |\n", no_kinds),
            (row, "| *`0x01`* | `blob` | defined |\n", no_kinds),
        ];
        for (i, (from, to, needle)) in missing.into_iter().enumerate() {
            let fixture = Fixture::new(&format!("spec-markdown-{i}"));
            fixture.write("docs/spec/format-v1.md", &replace(SPEC, from, to));
            fixture.fails_only("spec", needle);
        }

        // Each of these shows every section and row.
        for (i, text) in [
            replace(SPEC, change, " ## Change control\n"),
            replace(SPEC, change, "##\tChange control\n"),
            // A heading's text is what it shows, code spans and lines joined.
            replace(SPEC, change, "## `Change control`\n"),
            replace(SPEC, change, "## Change `control`\n"),
            replace(SPEC, change, "Change\ncontrol\n---\n"),
            replace(SPEC, change, "## Change *control*\n"),
            replace(
                SPEC,
                change,
                "```text\n## Fenced\n````\n<!-- a -->\n<!--\nb\n-->\n## Change control\n",
            ),
            replace(SPEC, change, "``` `not a fence` ```\n## Change control\n"),
            replace(SPEC, kinds, &format!("\n{kinds}")),
            replace(SPEC, kinds, &format!("```\ncode\n```\n{kinds}")),
            replace(SPEC, kinds, &format!("<!-- note -->\n{kinds}")),
            replace(SPEC, kinds, "| id | kind | status |\n|:---|:---:|---:|\n"),
            replace(SPEC, kinds, "| id | kind | status\n|---|---|---\n"),
            replace(SPEC, row, &format!("  {row}")),
            SPEC.replace('\n', "\r\n"),
            SPEC.replace('\n', "\r"),
            // A lone CR ends a line in a fence and in a comment too.
            replace(
                SPEC,
                change,
                "```\n## Fenced\n```\n<!--\n## Hidden\n-->\n## Change control\n",
            )
            .replace('\n', "\r"),
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("spec-markdown-ok-{i}"));
            fixture.write("docs/spec/format-v1.md", &text);
            fixture.passes();
        }
    }

    #[test]
    fn policy_refuses_member_patterns_it_cannot_expand() {
        for (i, pattern) in ["crates/[ab]*", "crates/**"].into_iter().enumerate() {
            let fixture = Fixture::new(&format!("members-pattern-{i}"));
            fixture.write(
                "Cargo.toml",
                &replace(WORKSPACE, "\"crates/*\"", &format!("\"{pattern}\"")),
            );
            fixture.fails_only("lints", "cannot expand");
        }
    }

    #[test]
    fn policy_reads_manifests_as_toml() {
        // An array may span lines, and `path=` needs no spaces.
        let fixture = Fixture::new("toml-workspace");
        let workspace = replace(
            WORKSPACE,
            "features = [\"pure\"]",
            "features = [\n  \"std\",\n  \"pure\",\n]",
        );
        let workspace = replace(&workspace, "path = \"crates/hash\"", "path=\"crates/hash\"");
        fixture.write("Cargo.toml", &workspace);
        fixture.passes();

        // Dotted keys, inline tables and tables mean the same thing.
        for (i, manifest) in [
            "[package]\nname = \"hash\"\npublish = { workspace = true }\n\n[dependencies]\n\
             blake3.workspace = true\nblake3.features = [\"std\"]\n\n[lints]\nworkspace = true\n",
            "package.name = \"hash\"\npackage.publish.workspace = true\nlints = { workspace = true }\n\
             dependencies.blake3 = { workspace = true }\n",
            "[package]\nname = \"hash\"\npublish.workspace = true\n\n[dependencies.blake3]\n\
             workspace = true\n\n[lints]\nworkspace = true\n",
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("toml-member-{i}"));
            fixture.write("crates/hash/Cargo.toml", manifest);
            fixture.passes();
        }
    }

    #[test]
    fn policy_flags_invalid_toml() {
        let fixture = Fixture::new("invalid-member");
        fixture.write("crates/hash/Cargo.toml", "[package\nname = \"hash\"\n");
        fixture.fails_exactly(
            &["lints", "publish", "deps", "names"],
            &["crates/hash/Cargo.toml", "not valid TOML"],
        );

        let fixture = Fixture::new("invalid-workspace");
        fixture.write(
            "Cargo.toml",
            &format!("{WORKSPACE}unsafe_code = \"forbid\"\n"),
        );
        fixture.fails_exactly(
            &["lints", "publish", "credits"],
            &["Cargo.toml", "not valid TOML"],
        );

        for (rule, path) in [
            ("toolchain", "rust-toolchain.toml"),
            ("clippy", "clippy.toml"),
        ] {
            let fixture = Fixture::new(&format!("invalid-{rule}"));
            fixture.write(path, "[toolchain\n");
            fixture.fails_only(rule, &format!("{path} is not valid TOML"));
        }
    }

    #[test]
    fn policy_ignores_byte_order_marks() {
        const BOM: &str = "\u{feff}";
        let fixture = Fixture::new("bom");
        for path in [
            "Cargo.toml",
            "rust-toolchain.toml",
            "clippy.toml",
            ".gitattributes",
            "docs/prior-art.md",
            "docs/spec/format-v1.md",
            "crates/hash/Cargo.toml",
            "crates/hash/src/lib.rs",
        ] {
            fixture.write(path, &format!("{BOM}{}", fixture.read(path)));
        }
        fixture.passes();

        let fixture = Fixture::new("bom-deps");
        fixture.write(
            "crates/hash/Cargo.toml",
            &format!(
                "{BOM}[dependencies]\nfoo = \"1\"\n\n[package]\nname = \"hash\"\npublish.workspace = true\n\n\
                 [lints]\nworkspace = true\n"
            ),
        );
        fixture.fails_exactly(&["deps"], &["crates/hash", "`foo`"]);
    }

    #[test]
    fn policy_finds_members_like_cargo() {
        // A literal member outside `crates/` gets every member rule.
        let fixture = Fixture::new("members-literal");
        fixture.write(
            "Cargo.toml",
            &replace(WORKSPACE, "\"crates/*\"]", "\"crates/*\", \"fuzz\"]"),
        );
        fixture.write(
            "fuzz/Cargo.toml",
            "[package]\nname = \"peri-fuzz\"\npublish = true\n\n[dependencies]\nlibfuzzer-sys = \"0.4\"\n",
        );
        fixture.write("fuzz/src/main.rs", "fn main() {}\n");
        fixture.fails_exactly(&["lints", "unsafe", "publish", "deps", "names"], &["fuzz"]);

        // `?` and `*` match inside a path component; a directory without a
        // manifest is not a member.
        let fixture = Fixture::new("members-wildcards");
        fixture.write(
            "Cargo.toml",
            &replace(WORKSPACE, "\"crates/*\"", "\"crat?s/*\""),
        );
        fixture.write("cratos/bad/Cargo.toml", &member("Bad", ""));
        fixture.write("crates/notes/README.md", "Not a crate.\n");
        fixture.fails_only("names", "cratos/bad");

        // `exclude` removes a crate a pattern finds, but not one named literally.
        let fixture = Fixture::new("members-exclude");
        fixture.write(
            "Cargo.toml",
            &replace(
                WORKSPACE,
                "\"crates/*\"]",
                "\"crates/*\"]\nexclude = [\"crates/old\"]",
            ),
        );
        fixture.write("crates/old/Cargo.toml", &member("Old", ""));
        fixture.passes();
        fixture.write(
            "Cargo.toml",
            &replace(
                WORKSPACE,
                "\"crates/*\"]",
                "\"crates/*\", \"crates/old\"]\nexclude = [\"crates/old\"]",
            ),
        );
        fixture.fails_only("names", "crates/old");

        // An `exclude` entry covers whole path components: `crates/old` does
        // not exclude `crates/older`.
        let fixture = Fixture::new("members-exclude-prefix");
        fixture.write(
            "Cargo.toml",
            &replace(
                WORKSPACE,
                "\"crates/*\"]",
                "\"crates/*\"]\nexclude = [\"crates/old\"]",
            ),
        );
        fixture.write("crates/older/Cargo.toml", &member("Older", ""));
        fixture.fails_only("names", "crates/older");

        // The root package is a member, and so is a path dependency inside the root.
        let fixture = Fixture::new("members-root");
        fixture.write(
            "Cargo.toml",
            &format!("[package]\nname = \"Root\"\npublish = false\n\n[lints]\nworkspace = true\n\n{WORKSPACE}"),
        );
        fixture.fails_only("names", "\"Root\"");

        let fixture = Fixture::new("members-path");
        fixture.write(
            "Cargo.toml",
            &replace(
                WORKSPACE,
                "hash = { path = \"crates/hash\" }",
                "hash = { path = \"crates/hash\" }\ntool = { path = \"tools/tool\" }",
            ),
        );
        fixture.write(
            "crates/hash/Cargo.toml",
            &member("hash", "tool.workspace = true"),
        );
        fixture.write("tools/tool/Cargo.toml", &member("Tool", ""));
        fixture.fails_only("names", "tools/tool");

        // Cargo follows path dependencies from each member it adds, at any depth.
        let fixture = Fixture::new("members-nested-path");
        fixture.write(
            "Cargo.toml",
            &replace(
                WORKSPACE,
                "hash = { path = \"crates/hash\" }",
                "hash = { path = \"crates/hash\" }\ntool = { path = \"tools/tool\" }\n\
                 util = { path = \"tools/util\" }",
            ),
        );
        fixture.write(
            "crates/hash/Cargo.toml",
            &member("hash", "tool.workspace = true"),
        );
        fixture.write(
            "tools/tool/Cargo.toml",
            &member("tool", "util.workspace = true"),
        );
        fixture.write("tools/util/Cargo.toml", &member("Util", ""));
        fixture.fails_only("names", "tools/util");

        // A member's own `path` dependency is a member too; `deps` flags the
        // entry itself.
        let fixture = Fixture::new("members-own-path");
        fixture.write(
            "crates/hash/Cargo.toml",
            &member("hash", "tool = { path = \"../../tools/tool\" }"),
        );
        fixture.write("tools/tool/Cargo.toml", &member("Tool", ""));
        fixture.fails_exactly(&["deps", "names"], &["tool"]);
    }
}
