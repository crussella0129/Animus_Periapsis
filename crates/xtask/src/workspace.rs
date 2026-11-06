//! Workspace members, found as Cargo finds them.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use toml_edit::DocumentMut;

use crate::manifest::{dependency_tables, is_true, load, lookup, strings};

/// The member crate directories of the workspace at `root`, as Cargo finds
/// them: the root package, if the root manifest has one; each directory that
/// a `[workspace] members` entry names, literally or with `*` and `?` inside
/// its path components, and that holds a `Cargo.toml`; and each path
/// dependency of a member that lies inside the root. A crate under an
/// `exclude` entry is left out unless a `members` entry names it, or a
/// directory above it, literally. A root manifest that cannot be read has no
/// members; the rules that read it say why.
pub fn members(root: &Path) -> Vec<PathBuf> {
    let Ok(workspace) = load(&root.join("Cargo.toml"), "Cargo.toml") else {
        return Vec::new();
    };
    let listed = strings(workspace.as_item(), &["workspace", "members"]);
    let exclude = strings(workspace.as_item(), &["workspace", "exclude"]);
    let member = |dir: &PathBuf| {
        root.join(dir).join("Cargo.toml").is_file() && !excluded(root, dir, &listed, &exclude)
    };
    let mut found: BTreeSet<PathBuf> = listed
        .iter()
        .flat_map(|pattern| expand(root, pattern))
        .filter(member)
        .collect();
    if workspace.contains_key("package") {
        found.insert(PathBuf::new());
    }
    let mut unvisited: Vec<PathBuf> = found.iter().cloned().collect();
    while let Some(dir) = unvisited.pop() {
        for dependency in path_dependencies(root, &workspace, &dir) {
            if member(&dependency) && found.insert(dependency.clone()) {
                unvisited.push(dependency);
            }
        }
    }
    found
        .into_iter()
        .map(|dir| {
            if dir.as_os_str().is_empty() {
                root.to_path_buf()
            } else {
                root.join(dir)
            }
        })
        .collect()
}

/// The directories, relative to the root, that a `members` entry names: a
/// literal path, or one with `*` (any run of characters) and `?` (any one
/// character) inside its components.
fn expand(root: &Path, pattern: &str) -> Vec<PathBuf> {
    let Some(pattern) = inside(root, Path::new(""), pattern) else {
        return Vec::new();
    };
    pattern.iter().fold(vec![PathBuf::new()], |dirs, part| {
        let part = part.to_string_lossy();
        if part.contains(['*', '?']) {
            dirs.iter()
                .flat_map(|dir| children(root, dir, &part))
                .collect()
        } else {
            dirs.into_iter().map(|dir| dir.join(&*part)).collect()
        }
    })
}

/// The entries of `dir`, relative to the root, whose names match `pattern`.
fn children(root: &Path, dir: &Path, pattern: &str) -> Vec<PathBuf> {
    fs::read_dir(root.join(dir))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| wildcard(pattern, name))
        .map(|name| dir.join(name))
        .collect()
}

/// Whether `name` matches `pattern`, where `*` matches any run of characters
/// and `?` exactly one.
fn wildcard(pattern: &str, name: &str) -> bool {
    let Some(first) = pattern.chars().next() else {
        return name.is_empty();
    };
    let pattern_rest = &pattern[first.len_utf8()..];
    if first == '*' {
        return name
            .char_indices()
            .map(|(at, _)| at)
            .chain([name.len()])
            .any(|at| wildcard(pattern_rest, &name[at..]));
    }
    let mut name_rest = name.chars();
    name_rest
        .next()
        .is_some_and(|found| first == '?' || first == found)
        && wildcard(pattern_rest, name_rest.as_str())
}

/// Cargo's exclusion rule: a crate under an `exclude` entry is not a member
/// unless a `members` entry names it, or a directory above it, literally.
fn excluded(root: &Path, dir: &Path, listed: &[&str], exclude: &[&str]) -> bool {
    let manifest = dir.join("Cargo.toml");
    let under = |entries: &[&str]| {
        entries
            .iter()
            .filter_map(|entry| inside(root, Path::new(""), entry))
            .any(|entry| manifest.starts_with(entry))
    };
    under(exclude) && !under(listed)
}

/// The directories, relative to the root, of a member's path dependencies:
/// its own `path` entries, and the workspace entries it inherits that have
/// one.
fn path_dependencies(root: &Path, workspace: &DocumentMut, dir: &Path) -> Vec<PathBuf> {
    let Ok(manifest) = load(&root.join(dir).join("Cargo.toml"), "Cargo.toml") else {
        return Vec::new();
    };
    let inherited = lookup(workspace.as_item(), &["workspace", "dependencies"]);
    dependency_tables(&manifest)
        .into_iter()
        .flat_map(|(_, table)| table.iter())
        .filter_map(|(name, entry)| {
            let (base, path) = if is_true(entry, &["workspace"]) {
                (Path::new(""), lookup(inherited?, &[name, "path"])?)
            } else {
                (dir, lookup(entry, &["path"])?)
            };
            inside(root, base, path.as_str()?)
        })
        .collect()
}

/// `path`, taken relative to `base` (itself relative to the root), as a path
/// relative to the root with `.` and `..` resolved; `None` when it leads
/// outside the root.
fn inside(root: &Path, base: &Path, path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    let joined = match path.strip_prefix(root) {
        Ok(rest) => rest.to_path_buf(),
        Err(_) => base.join(path),
    };
    let mut resolved = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !resolved.pop() {
                    return None;
                }
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(resolved)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{inside, wildcard};

    #[test]
    fn wildcards_match_within_a_component() {
        for (pattern, name) in [
            ("*", "hash"),
            ("*", ""),
            ("h*", "hash"),
            ("*h", "hash"),
            ("h?sh", "hash"),
            ("*a*", "hash"),
            ("h*s*h", "hash"),
            ("?", "é"),
        ] {
            assert!(wildcard(pattern, name), "{pattern:?} should match {name:?}");
        }
        for (pattern, name) in [
            ("h?sh", "hsh"),
            ("h?sh", "haash"),
            ("*x", "hash"),
            ("hash", "hashes"),
            ("?", ""),
            ("hash", "Hash"),
        ] {
            assert!(
                !wildcard(pattern, name),
                "{pattern:?} should not match {name:?}"
            );
        }
    }

    #[test]
    fn paths_resolve_inside_the_root() {
        let root = Path::new("/workspace");
        let resolve = |base: &str, path: &str| inside(root, Path::new(base), path);
        assert_eq!(
            resolve("", "crates/hash"),
            Some(PathBuf::from("crates/hash"))
        );
        assert_eq!(
            resolve("", "./crates/./hash/"),
            Some(PathBuf::from("crates/hash"))
        );
        assert_eq!(
            resolve("crates/codec", "../hash"),
            Some(PathBuf::from("crates/hash"))
        );
        assert_eq!(resolve("crates", ".."), Some(PathBuf::new()));
        assert_eq!(
            resolve("", "/workspace/crates/hash"),
            Some(PathBuf::from("crates/hash"))
        );
        assert_eq!(resolve("crates/codec", "../../../elsewhere"), None);
        assert_eq!(resolve("", ".."), None);
        assert_eq!(resolve("", "/elsewhere/hash"), None);
    }
}
