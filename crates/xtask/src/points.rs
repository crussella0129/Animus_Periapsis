//! Reading named points from source (INT-0005 AC-6): the `.point("…")`
//! calls in a boundary crate's `src/`, and its registry in `src/points.rs`.
//!
//! The project has no Rust parser, so this reads text. Comments are removed
//! first, so a point in a comment or in rustdoc is not a use, and anything
//! the reader does not recognise is reported: it fails closed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The boundary crates whose points are checked; `sim` defines the trait
/// and its forwarders, which pass names on rather than mark points.
pub const POINT_CRATES: [&str; 3] = ["store", "praps", "workspace"];

/// The one accepted registry declaration.
const REGISTRY_START: &str = "pub const POINTS: &[Point] = &[";

/// One registry entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub unit: String,
    pub sprint: u32,
}

/// `source` with `//` and `/* */` comments replaced by spaces, keeping line
/// breaks. String literals are not tracked: a `//` inside one cuts the rest
/// of its line, which can only hide a use, and an unused registration then
/// fails the rule.
pub fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('/', Some('/')) => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            ('/', Some('*')) => {
                chars.next();
                let mut depth = 1usize;
                let mut last = ' ';
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                    }
                    if last == '/' && c == '*' {
                        depth += 1;
                        last = ' ';
                        continue;
                    }
                    if last == '*' && c == '/' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        last = ' ';
                        continue;
                    }
                    last = c;
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// A string literal at the start of `text` (after whitespace): its value
/// and the text after it. Escapes are not accepted: point names have none.
fn literal(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start().strip_prefix('"')?;
    let end = text.find(['"', '\\'])?;
    (text.as_bytes()[end] == b'"').then(|| (&text[..end], &text[end + 1..]))
}

/// The point names `source` marks, and the calls the rule cannot accept:
/// - a `.point(` whose argument is not a single string literal;
/// - `point` called through a path (`Fs::point(…)`, `<F as Fs>::point(…)`),
///   which the reader would otherwise miss;
/// - a point marked after the file's first `#[cfg(test)]`: test code in
///   `src/` may not mark registered points, nor count as their use
///   (code review CR-03; test modules sit at the end of a file).
pub fn uses(source: &str) -> (Vec<String>, Vec<String>) {
    let code = strip_comments(source);
    let (mut names, mut bad) = (Vec::new(), Vec::new());
    if code.contains("::point(") {
        bad.push("`::point(`: call `point` as a method, `.point(\"…\")`".to_owned());
    }
    let (code, tests) = match code.find("#[cfg(test)]") {
        Some(at) => code.split_at(at),
        None => (code.as_str(), ""),
    };
    if tests.contains(".point(") {
        bad.push("`.point(` in test code after `#[cfg(test)]` under src/".to_owned());
    }
    let mut rest = code;
    while let Some(at) = rest.find(".point(") {
        rest = &rest[at + ".point(".len()..];
        match literal(rest) {
            // rustfmt may leave a trailing comma after a literal on its own line.
            Some((name, after))
                if after
                    .trim_start()
                    .strip_prefix(',')
                    .unwrap_or(after.trim_start())
                    .trim_start()
                    .starts_with(')') =>
            {
                names.push(name.to_owned());
            }
            _ => {
                let shown: String = rest.chars().take(40).collect();
                bad.push(format!(".point({}", shown.split('\n').next().unwrap_or("")));
            }
        }
    }
    (names, bad)
}

/// The entries of a registry file, or what in it is not an entry of the one
/// accepted form.
pub fn registry(source: &str) -> Result<Vec<Entry>, String> {
    let code = strip_comments(source);
    let start = code
        .find(REGISTRY_START)
        .ok_or_else(|| format!("no `{REGISTRY_START}…];`"))?;
    let body = &code[start + REGISTRY_START.len()..];
    let end = body
        .find("];")
        .ok_or("the registry is not closed with `];`")?;
    let mut rest = &body[..end];
    let mut entries = Vec::new();
    loop {
        rest = rest.trim_start().trim_start_matches(',').trim_start();
        if rest.is_empty() {
            return Ok(entries);
        }
        let (entry, after) = entry(rest).ok_or_else(|| {
            let shown: String = rest.chars().take(60).collect();
            format!("an entry not of the form `Point {{ name: \"…\", unit: \"…\", sprint: N }}`: {shown}")
        })?;
        entries.push(entry);
        rest = after;
    }
}

fn entry(text: &str) -> Option<(Entry, &str)> {
    let text = text.strip_prefix("Point")?.trim_start().strip_prefix('{')?;
    let text = text.trim_start().strip_prefix("name:")?;
    let (name, text) = literal(text)?;
    let text = text.trim_start().strip_prefix(',')?.trim_start();
    let text = text.strip_prefix("unit:")?;
    let (unit, text) = literal(text)?;
    let text = text.trim_start().strip_prefix(',')?.trim_start();
    let text = text.strip_prefix("sprint:")?.trim_start();
    let digits = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    let sprint = text[..digits].parse().ok()?;
    let text = text[digits..].trim_start();
    let text = text.strip_prefix(',').unwrap_or(text).trim_start();
    let text = text.strip_prefix('}')?;
    Some((
        Entry {
            name: name.to_owned(),
            unit: unit.to_owned(),
            sprint,
        },
        text,
    ))
}

/// Whether `name` is `crate.function.stage` for `krate`, in lowercase ASCII
/// with `_`.
pub fn well_formed(name: &str, krate: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() == 3
        && parts[0] == krate
        && parts.iter().all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
}

/// The highest `docs/sprints/sN` under `root`, or 0.
pub fn latest_sprint(root: &Path) -> u32 {
    fs::read_dir(root.join("docs/sprints"))
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| {
                    entry
                        .file_name()
                        .to_str()?
                        .strip_prefix('s')?
                        .parse::<u32>()
                        .ok()
                })
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

/// The registries of the point crates among `members`, by crate name, with
/// the problems found reading them.
pub fn registries(members: &[PathBuf]) -> (BTreeMap<String, Vec<Entry>>, Vec<String>) {
    let mut out = BTreeMap::new();
    let mut problems = Vec::new();
    for member in members {
        let Some(krate) = member.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !POINT_CRATES.contains(&krate) {
            continue;
        }
        let path = member.join("src/points.rs");
        match fs::read_to_string(&path) {
            Ok(text) => match registry(&text) {
                Ok(entries) => {
                    out.insert(krate.to_owned(), entries);
                }
                Err(why) => problems.push(format!("{krate}/src/points.rs: {why}")),
            },
            Err(_) => {
                out.insert(krate.to_owned(), Vec::new());
            }
        }
    }
    (out, problems)
}

#[cfg(test)]
mod tests {
    use super::{registry, strip_comments, uses, well_formed};

    #[test]
    fn comments_are_not_uses() {
        let source = "// fs.point(\"store.a.b\")\n/// fs.point(\"store.c.d\")\n\
                      /* fs.point(\"store.e.f\") */ fs.point(\n    \"store.g.h\",\n);\n";
        assert_eq!(uses(source), (vec!["store.g.h".to_owned()], vec![]));
        assert_eq!(strip_comments("a/* x\ny */b").lines().count(), 2);
    }

    #[test]
    fn non_literal_arguments_are_reported() {
        let (names, bad) = uses("fs.point(NAME);\nfs.point(\"a\" );\nfs.point(\"x\\\"y\");");
        assert_eq!(names, ["a"]);
        assert_eq!(bad.len(), 2, "{bad:?}");
    }

    #[test]
    fn registry_reads_the_one_form() {
        let ok = "use sim::Point;\npub const POINTS: &[Point] = &[\n    Point {\n        name: \"store.a.b\",\n        \
                  unit: \"store::a\",\n        sprint: 1,\n    },\n    // a comment\n    Point { name: \"store.c.d\", unit: \"store::c\", sprint: 2 }\n];\n";
        let entries = registry(ok).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].sprint, 2);
        assert!(registry("pub const POINTS: &[Point] = &[ X ];").is_err());
        assert!(registry("const POINTS: &[Point] = &[];").is_err());
    }

    #[test]
    fn names_are_checked() {
        assert!(well_formed("store.commit_op.renamed", "store"));
        assert!(!well_formed("store.commit_op", "store"));
        assert!(!well_formed("store.Commit.renamed", "store"));
        assert!(!well_formed("workspace.a.b", "store"));
    }
}
