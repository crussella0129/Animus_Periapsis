//! The licence policy in `deny.toml` (INT-0003 AC-3). `cargo deny check`
//! proves that every dependency's licence is on the policy's `allow` lists;
//! these tests prove that every licence on those lists is compatible with
//! GPL-3.0.

use std::fs;
use std::path::Path;

use toml_edit::{DocumentMut, Item, TableLike, Value};

/// Licences, as SPDX expressions, compatible with distributing Periapsis under
/// GPL-3.0: those the Free Software Foundation lists as GPL-3.0-compatible,
/// `Apache-2.0` with the LLVM exception (which only adds permissions), and
/// `Unicode-3.0`, the successor of the listed Unicode data-files licence.
/// Adding one is a reviewed change to this list.
const GPL3_COMPATIBLE: [&str; 13] = [
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "BSL-1.0",
    "CC0-1.0",
    "ISC",
    "MIT",
    "MPL-2.0",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Unlicense",
    "Zlib",
];

/// The licences a `deny.toml` allows that are not on the list, from its
/// `[licenses] allow` list and from every per-crate exception, or why they
/// cannot be read.
fn incompatible(text: &str) -> Result<Vec<String>, String> {
    let config: DocumentMut = text
        .parse()
        .map_err(|error| format!("deny.toml: {error}"))?;
    let licenses = config
        .get("licenses")
        .and_then(Item::as_table_like)
        .ok_or("deny.toml has no [licenses] table")?;
    if !licenses.contains_key("allow") {
        return Err("deny.toml has no [licenses] allow list".to_owned());
    }
    let mut tables: Vec<&dyn TableLike> = vec![licenses];
    match licenses.get("exceptions") {
        None => {}
        Some(Item::ArrayOfTables(exceptions)) => {
            tables.extend(exceptions.iter().map(|table| table as &dyn TableLike));
        }
        Some(Item::Value(Value::Array(exceptions))) => {
            for exception in exceptions {
                let table = exception
                    .as_inline_table()
                    .ok_or("a licence exception is not a table")?;
                tables.push(table);
            }
        }
        Some(_) => return Err("[licenses] exceptions is not a list of tables".to_owned()),
    }
    let mut wrong = Vec::new();
    for table in tables {
        let Some(allow) = table.get("allow") else {
            continue;
        };
        for entry in allow.as_array().ok_or("an allow list is not an array")? {
            let licence = entry.as_str().ok_or("an allow entry is not a string")?;
            if !GPL3_COMPATIBLE.contains(&licence) {
                wrong.push(licence.to_owned());
            }
        }
    }
    Ok(wrong)
}

#[test]
fn deny_allows_only_gpl3_compatible_licences() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deny.toml");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert_eq!(incompatible(&text), Ok(Vec::new()));
}

/// An incompatible licence is named wherever `deny.toml` allows it: in the
/// main list, or in a per-crate exception in either TOML form. A policy that
/// cannot be read is an error, never a pass.
#[test]
fn incompatible_licences_are_named() {
    let main = "[licenses]\nallow = [\"MIT\", \"SSPL-1.0\"]\n";
    assert_eq!(incompatible(main), Ok(vec!["SSPL-1.0".to_owned()]));
    let exception = "[licenses]\nallow = [\"MIT\"]\n\n\
                     [[licenses.exceptions]]\ncrate = \"x\"\nallow = [\"BUSL-1.1\"]\n";
    assert_eq!(incompatible(exception), Ok(vec!["BUSL-1.1".to_owned()]));
    let inline = "[licenses]\nallow = [\"MIT\"]\n\
                  exceptions = [{ crate = \"x\", allow = [\"OpenSSL\"] }]\n";
    assert_eq!(incompatible(inline), Ok(vec!["OpenSSL".to_owned()]));
    for broken in [
        "",
        "[licenses]\n",
        "[licenses]\nallow = \"MIT\"\n",
        "[licenses]\nallow = [1]\n",
        "[licenses]\nallow = [\"MIT\"]\nexceptions = \"x\"\n",
        "licenses = [",
    ] {
        assert!(incompatible(broken).is_err(), "{broken:?}");
    }
}
