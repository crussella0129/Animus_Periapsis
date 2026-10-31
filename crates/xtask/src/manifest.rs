//! TOML files read as Cargo, rustup and Clippy read them: parsed, so tables,
//! inline tables and dotted keys mean the same thing, and without a leading
//! byte-order mark.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use toml_edit::{DocumentMut, Item, TableLike, Value};

/// The tables a dependency can be declared in, including the underscore
/// spellings Cargo still accepts before edition 2024.
const DEPENDENCY_TABLES: [&str; 5] = [
    "dependencies",
    "dev-dependencies",
    "build-dependencies",
    "dev_dependencies",
    "build_dependencies",
];

/// Reads a UTF-8 file without its leading byte-order mark, if it has one.
pub fn read_text(path: &Path) -> io::Result<String> {
    let text = fs::read_to_string(path)?;
    Ok(match text.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_owned(),
        None => text,
    })
}

/// Parses the TOML file at `path`, or says why it cannot, calling it `name`.
pub fn load(path: &Path, name: &str) -> Result<DocumentMut, String> {
    let text = read_text(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => format!("{name} is missing"),
        _ => format!("{name} could not be read: {error}"),
    })?;
    text.parse::<DocumentMut>().map_err(|error| {
        let at = error.span().map_or(0, |span| span.start);
        let line = 1 + text.bytes().take(at).filter(|&byte| byte == b'\n').count();
        format!(
            "{name} is not valid TOML: {} (line {line})",
            error.message()
        )
    })
}

/// The item at `path` below `item`, through tables, inline tables and dotted
/// keys alike.
pub fn lookup<'a>(item: &'a Item, path: &[&str]) -> Option<&'a Item> {
    path.iter()
        .try_fold(item, |item, key| item.as_table_like()?.get(key))
}

/// Whether the item at `path` below `item` is `true`, as in `workspace = true`.
pub fn is_true(item: &Item, path: &[&str]) -> bool {
    lookup(item, path).and_then(Item::as_bool) == Some(true)
}

/// The strings in the array at `path` below `item`; anything else holds none.
pub fn strings<'a>(item: &'a Item, path: &[&str]) -> Vec<&'a str> {
    lookup(item, path)
        .and_then(Item::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

/// Every dependency table of a manifest, at the top level and under each
/// `[target.<cfg>]`, with the header that names it.
pub fn dependency_tables(manifest: &DocumentMut) -> Vec<(String, &dyn TableLike)> {
    let mut owners = vec![(String::new(), manifest.as_item())];
    if let Some(targets) = manifest.get("target").and_then(Item::as_table_like) {
        owners.extend(
            targets
                .iter()
                .map(|(cfg, target)| (format!("target.'{cfg}'."), target)),
        );
    }
    owners
        .into_iter()
        .flat_map(|(prefix, owner)| {
            DEPENDENCY_TABLES.into_iter().filter_map(move |kind| {
                let table = lookup(owner, &[kind])?.as_table_like()?;
                Some((format!("{prefix}{kind}"), table))
            })
        })
        .collect()
}
