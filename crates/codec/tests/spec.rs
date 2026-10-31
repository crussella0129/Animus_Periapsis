//! The code and the golden files must agree with `docs/spec/format-v1.md`
//! (INT-0006 AC-6: the spec is written first and the code follows it). The
//! spec's worked examples were derived by hand, so this is the encoder's check
//! against an independent source.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use codec::{DecodeError, ObjectKind, RESERVED_KINDS};
use hash::HashAlg;

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn spec() -> String {
    let path = manifest_dir().join("../../docs/spec/format-v1.md");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The table rows starting with `prefix` in the section headed `heading`, as
/// trimmed cells without backticks.
fn rows(spec: &str, heading: &str, prefix: &str) -> Vec<Vec<String>> {
    let mut inside = false;
    let mut rows = Vec::new();
    for line in spec.lines() {
        if let Some(rest) = line.strip_prefix('#') {
            inside = rest.trim_start_matches('#').trim() == heading;
        } else if inside && line.starts_with(prefix) {
            rows.push(
                line.trim()
                    .trim_matches('|')
                    .split('|')
                    .map(|cell| cell.trim().trim_matches('`').to_owned())
                    .collect(),
            );
        }
    }
    rows
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

    let spec_hashes: Vec<(u8, String, usize, String)> = rows(&spec, "Hash algorithms", "| `0x")
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

    let spec_kinds: Vec<(u8, String, String)> = rows(&spec, "Object kinds", "| `0x")
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
    let examples: BTreeMap<String, Vec<u8>> = rows(&spec(), "Worked examples", "| `")
        .into_iter()
        .map(|row| (row[0].clone(), hex_bytes(row.last().unwrap())))
        .collect();
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
