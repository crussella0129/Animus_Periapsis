//! Trees, derived from changesets (INT-0013): a tree is never stored.

use std::collections::BTreeMap;

use codec::{Changeset, Entry, Mode, RepoPath};
use hash::HashRef;

/// One file in a tree: its blob and mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File {
    /// The blob holding the file's content.
    pub blob: HashRef,
    /// The file's mode.
    pub mode: Mode,
}

/// Every file of a change, by path.
pub type Tree = BTreeMap<RepoPath, File>;

/// `base` with `changes` applied.
pub fn apply(base: &Tree, changes: &BTreeMap<RepoPath, Entry>) -> Tree {
    let mut tree = base.clone();
    for (path, entry) in changes {
        match entry {
            Entry::Put { blob, mode } => {
                tree.insert(
                    path.clone(),
                    File {
                        blob: *blob,
                        mode: *mode,
                    },
                );
            }
            Entry::Delete => {
                tree.remove(path);
            }
        }
    }
    tree
}

/// The changes that turn `base` into `target`: a put for every path added
/// or changed, a delete for every path removed.
pub fn diff(base: &Tree, target: &Tree) -> BTreeMap<RepoPath, Entry> {
    let mut changes = BTreeMap::new();
    for (path, file) in target {
        if base.get(path) != Some(file) {
            changes.insert(
                path.clone(),
                Entry::Put {
                    blob: file.blob,
                    mode: file.mode,
                },
            );
        }
    }
    for path in base.keys() {
        if !target.contains_key(path) {
            changes.insert(path.clone(), Entry::Delete);
        }
    }
    changes
}

/// A changeset's tree, given its first parent's.
pub fn of(changeset: &Changeset, parent: &Tree) -> Tree {
    apply(parent, &changeset.changes)
}
