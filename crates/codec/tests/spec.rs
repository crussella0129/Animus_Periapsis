//! The code and the golden files must agree with `docs/spec/format-v1.md`
//! (INT-0006 AC-6: the spec is written first and the code follows it). The
//! spec's worked examples were derived by hand, so this is the encoder's check
//! against an independent source.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use codec::{DecodeError, ObjectKind, RESERVED_KINDS};
use hash::HashAlg;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn spec() -> String {
    let path = manifest_dir().join("../../docs/spec/format-v1.md");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// What the test reads in the spec, in order.
#[derive(Debug)]
enum Block {
    Heading(String),
    /// A table's rows, its header first, as their cells' text.
    Table(Vec<Vec<String>>),
}

/// The spec's headings and tables as GitHub shows them: CommonMark with
/// tables, as pulldown-cmark reads them (`book` and `cargo policy` read
/// Markdown with it too). What it shows as code or in an HTML block is
/// neither. A text keeps each code span in backticks.
fn blocks(spec: &str) -> Vec<Block> {
    // CommonMark ends a line at a lone CR too, but pulldown-cmark does not
    // everywhere, so each line ends at an LF.
    let spec = spec.replace("\r\n", "\n").replace('\r', "\n");
    let mut blocks = Vec::new();
    // Whether a heading or a cell is open.
    let mut open = false;
    for event in Parser::new_ext(&spec, Options::ENABLE_TABLES) {
        let text = match event {
            Event::Start(Tag::Heading { .. }) => {
                blocks.push(Block::Heading(String::new()));
                open = true;
                continue;
            }
            Event::Start(Tag::Table(_)) => {
                blocks.push(Block::Table(Vec::new()));
                continue;
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => {
                if let Some(Block::Table(rows)) = blocks.last_mut() {
                    rows.push(Vec::new());
                }
                continue;
            }
            Event::Start(Tag::TableCell) => {
                if let Some(Block::Table(rows)) = blocks.last_mut() {
                    rows.last_mut().unwrap().push(String::new());
                }
                open = true;
                continue;
            }
            Event::End(TagEnd::Heading(_) | TagEnd::TableCell) => {
                open = false;
                continue;
            }
            Event::Text(text) if open => text.into_string(),
            Event::Code(code) if open => format!("`{code}`"),
            _ => continue,
        };
        match blocks.last_mut() {
            Some(Block::Heading(heading)) => heading.push_str(&text),
            Some(Block::Table(rows)) => {
                let cell = rows.last_mut().and_then(|row| row.last_mut()).unwrap();
                cell.push_str(&text);
            }
            None => unreachable!("text outside a heading or a cell"),
        }
    }
    blocks
}

/// The body rows of the table in the section headed `heading`, as their
/// cells' text without backticks, or what keeps the test from reading them.
///
/// The test must read the rows a reader sees, so it reads them as GitHub
/// shows them (see `blocks`): a row in a code block, an HTML block or a
/// comment is none, and every heading ends the section above it. The spec
/// must have one such section, and the section one table, whose header names
/// `columns` and whose every row's first cell starts with `prefix`.
fn rows(
    spec: &str,
    heading: &str,
    columns: &[&str],
    prefix: &str,
) -> Result<Vec<Vec<String>>, String> {
    let blocks = blocks(spec);
    let starts: Vec<usize> = (0..blocks.len())
        .filter(|&at| matches!(&blocks[at], Block::Heading(text) if text == heading))
        .collect();
    let [start] = starts[..] else {
        return Err(format!(
            "{heading}: {} such sections, not one",
            starts.len()
        ));
    };
    let tables: Vec<&Block> = blocks[start + 1..]
        .iter()
        .take_while(|block| !matches!(block, Block::Heading(_)))
        .collect();
    let [Block::Table(rows)] = tables[..] else {
        return Err(format!(
            "{heading}: the section needs one table: {tables:?}"
        ));
    };
    let [header, body @ ..] = &rows[..] else {
        unreachable!("a table has a header");
    };
    if header != columns {
        return Err(format!("{heading}: the table's columns are {header:?}"));
    }
    body.iter()
        .map(|row| {
            if row[0].starts_with(prefix) {
                Ok(row
                    .iter()
                    .map(|cell| cell.trim_matches('`').to_owned())
                    .collect())
            } else {
                Err(format!("{heading}: a row the test cannot read: {row:?}"))
            }
        })
        .collect()
}

/// The body rows `rows` reads, or a failure naming what stopped it.
fn table(spec: &str, heading: &str, columns: &[&str], prefix: &str) -> Vec<Vec<String>> {
    rows(spec, heading, columns, prefix).unwrap_or_else(|problem| panic!("{problem}"))
}

fn hex_id(cell: &str) -> u8 {
    u8::from_str_radix(cell.trim_start_matches("0x"), 16).unwrap()
}

fn hex_bytes(cell: &str) -> Vec<u8> {
    let digits: Vec<u8> = cell.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let (pairs, odd) = digits.as_chunks::<2>();
    assert!(odd.is_empty(), "odd hex in spec: {cell}");
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn spec_registries_match_code() {
    let spec = spec();

    let spec_hashes: Vec<(u8, String, usize, String)> = table(
        &spec,
        "Hash algorithms",
        &["id", "name", "digest bytes", "status"],
        "`0x",
    )
    .into_iter()
    .map(|row| {
        (
            hex_id(&row[0]),
            row[1].clone(),
            row[2].parse().unwrap(),
            row[3].clone(),
        )
    })
    .collect();
    let code_hashes: Vec<(u8, String, usize, String)> = HashAlg::ALL
        .into_iter()
        .map(|alg| {
            let status = if alg == HashAlg::DEFAULT {
                "default"
            } else {
                "supported"
            };
            (
                alg.id(),
                alg.name().to_owned(),
                alg.digest_len(),
                status.to_owned(),
            )
        })
        .collect();
    assert_eq!(spec_hashes, code_hashes, "hash algorithm table");

    let spec_kinds: Vec<(u8, String, String)> =
        table(&spec, "Object kinds", &["id", "kind", "status"], "`0x")
            .into_iter()
            .map(|row| (hex_id(&row[0]), row[1].clone(), row[2].clone()))
            .collect();
    let mut code_kinds: Vec<(u8, String, String)> = ObjectKind::ALL
        .into_iter()
        .map(|kind| (kind.id(), kind.name().to_owned(), "defined".to_owned()))
        .chain(
            RESERVED_KINDS
                .into_iter()
                .map(|(id, name)| (id, name.to_owned(), "reserved".to_owned())),
        )
        .collect();
    code_kinds.sort();
    assert_eq!(spec_kinds, code_kinds, "object kind table");

    for (kind, _, status) in &spec_kinds {
        if status == "reserved" {
            assert_eq!(
                ObjectKind::from_id(*kind),
                Err(DecodeError::ReservedObjectKind { kind: *kind })
            );
        }
    }
}

#[test]
fn spec_examples_match_golden_files() {
    let mut examples = BTreeMap::new();
    for row in table(&spec(), "Worked examples", &["name", "value", "hex"], "`") {
        assert!(
            examples
                .insert(row[0].clone(), hex_bytes(&row[2]))
                .is_none(),
            "{}: a worked example is named twice",
            row[0]
        );
    }
    assert!(!examples.is_empty(), "the spec has no worked examples");

    let dir: PathBuf = manifest_dir().join("tests/golden/v1");
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "bin") {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            files.insert(name, fs::read(&path).unwrap());
        }
    }

    let example_names: Vec<&String> = examples.keys().collect();
    let file_names: Vec<&String> = files.keys().collect();
    assert_eq!(
        example_names, file_names,
        "every golden file needs a worked example, and every example a file"
    );
    for (name, bytes) in &files {
        assert_eq!(
            bytes, &examples[name],
            "{name}.bin disagrees with the spec's worked example"
        );
    }
}

/// The test reads what GitHub shows: a table in a code block or a comment, or
/// past the next heading, is none. Each section it cannot read whole stops it:
/// none or two of them, none or two tables, other columns, or a row that does
/// not start as the registry's rows do.
#[test]
fn spec_tables_are_read_as_shown() {
    let read = |spec: &str| rows(spec, "Kinds", &["id", "kind"], "`0x");
    let table = "| id | kind |\n|---|---|\n| `0x01` | `blob` |\n";
    let blob = Ok(vec![vec!["0x01".to_owned(), "blob".to_owned()]]);
    for spec in [
        format!("## Kinds\n{table}"),
        format!("# Format\n## Kinds\n\n{table}\n## Next\n| id | kind |\n|---|---|\n"),
        format!("## Kinds\n<!--\n| id | kind |\n|---|---|\n| `0x02` | `tree` |\n-->\n{table}"),
        format!("## Kinds\n```\n## Next\n| id | kind |\n|---|---|\n```\n{table}"),
        format!("## Kinds\n{table}").replace('\n', "\r"),
        format!("## Kinds\n{table}").replace('\n', "\r\n"),
        // A lone CR ends a line in a fence and in a comment too.
        format!("## Kinds\n```\n## Next\n```\n<!--\n## Next\n-->\n{table}").replace('\n', "\r"),
        format!("```\n## Kinds\n```\n## Kinds\n{table}"),
    ] {
        assert_eq!(read(&spec), blob, "{spec:?}");
    }
    assert_eq!(
        read("## Kinds\n| id | kind |\n|:--|--:|\n| `0x01` | `blob` |\n| `0x02` | `tree` |\n"),
        Ok(vec![
            vec!["0x01".to_owned(), "blob".to_owned()],
            vec!["0x02".to_owned(), "tree".to_owned()],
        ])
    );
    for spec in [
        String::new(),
        table.to_owned(),
        "## Kinds\n".to_owned(),
        format!("## Kinds\n{table}\n{table}"),
        format!("## Kinds\n{table}{table}"),
        format!("## Kinds\n{table}## Kinds\n{table}"),
        format!("## Kinds\n## Next\n{table}"),
        format!("## Kinds\n- note\n\n  ### Next\n\n{table}"),
        format!("<!--\n## Kinds\n{table}-->\n"),
        format!("## Kinds\n<!--\n{table}-->\n"),
        format!("## Kinds\n```\n{table}```\n"),
        format!("## Kinds\n\n    {}", table.replace('\n', "\n    ")),
        format!("## Kinds\n```\n{table}"),
        "## Kinds\n| id | name |\n|---|---|\n| `0x01` | `blob` |\n".to_owned(),
        "## Kinds\n| id |\n|---|\n| `0x01` |\n".to_owned(),
        "## Kinds\n| id | kind |\n|---|---|\n| 0x01 | `blob` |\n".to_owned(),
        "## Kinds\n| id | kind |\n|---|---|\n| `01` | `blob` |\n".to_owned(),
    ] {
        assert!(read(&spec).is_err(), "{spec:?} read as {:?}", read(&spec));
    }
}
