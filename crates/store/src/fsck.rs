//! `fsck`: checksums and referential integrity of a store (INT-0007 AC-4).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io;
use std::path::Path;

use codec::{ObjectKind, Operation, decode_envelope, object_id, references};
use hash::HashRef;
use sim::Fs;

use crate::names::{TMP_PREFIX, file_name, parse_name};
use crate::{OBJECTS, OPS, not_a_store};

/// What is wrong with one file.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProblemKind {
    /// The name is not `<algorithm id>-<digest>` (and not a temporary file).
    BadName,
    /// The file could not be read.
    Unreadable(String),
    /// The content does not hash to the id the name gives.
    HashMismatch,
    /// The content does not decode as the object or operation it should be.
    Undecodable(String),
    /// A valid op names a parent op that is missing or invalid.
    MissingParent(HashRef),
    /// A valid op names a view, or an object reachable from a valid op names
    /// an object, that is missing, invalid, or (for an op's view) not a view.
    MissingReference(HashRef),
    /// The store's directory holds something other than `objects/` and
    /// `ops/`, such as a pointer file.
    Unexpected,
}

/// One problem, at a path relative to the store's directory.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Problem {
    /// The file, `/`-separated, relative to the store.
    pub path: String,
    /// What is wrong with it.
    pub kind: ProblemKind,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = &self.path;
        match &self.kind {
            ProblemKind::BadName => write!(f, "{path}: not an object or op file name"),
            ProblemKind::Unreadable(error) => write!(f, "{path}: unreadable: {error}"),
            ProblemKind::HashMismatch => write!(f, "{path}: content does not hash to its name"),
            ProblemKind::Undecodable(error) => write!(f, "{path}: does not decode: {error}"),
            ProblemKind::MissingParent(id) => {
                write!(f, "{path}: parent op {} is missing", file_name(id))
            }
            ProblemKind::MissingReference(id) => {
                write!(f, "{path}: referenced object {} is missing", file_name(id))
            }
            ProblemKind::Unexpected => write!(f, "{path}: not part of a store"),
        }
    }
}

/// What `fsck` found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Corruption, in path order. Empty means the store is healthy.
    pub problems: Vec<Problem>,
    /// What a crash may leave behind that is not corruption: temporary files,
    /// which the store never reads, and objects that no op reaches whose own
    /// references are missing (an interrupted write; INT-0007 Consequences).
    pub leftovers: Vec<String>,
}

impl Report {
    /// Whether no corruption was found.
    pub fn is_clean(&self) -> bool {
        self.problems.is_empty()
    }
}

/// Checks the store at `root`.
pub(crate) fn check(fs: &impl Fs, root: &Path) -> io::Result<Report> {
    let list = |dir: &str| {
        fs.list(&root.join(dir)).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                not_a_store(root)
            } else {
                error
            }
        })
    };
    let object_names = list(OBJECTS)?;
    let op_names = list(OPS)?;
    let mut report = Report::default();

    for name in fs.list(root)? {
        if name != OBJECTS && name != OPS {
            report.problems.push(Problem {
                path: name,
                kind: ProblemKind::Unexpected,
            });
        }
    }

    // Each valid object: its file name, its kind and the ids it names.
    let mut objects: BTreeMap<HashRef, (String, ObjectKind, Vec<HashRef>)> = BTreeMap::new();
    for name in object_names {
        let mut decoded = None;
        if let Some(id) = file(fs, root, OBJECTS, &name, &mut report, |bytes| {
            let kind = decode_envelope(bytes).map_err(|e| e.to_string())?.kind;
            if kind == ObjectKind::Operation {
                return Err("an operation is not an object".to_owned());
            }
            let refs = references(bytes).map_err(|e| e.to_string())?;
            decoded = Some((kind, refs));
            Ok(())
        }) {
            let (kind, refs) = decoded.expect("a valid object decoded");
            objects.insert(id, (name, kind, refs));
        }
    }

    let mut ops = BTreeMap::new();
    for name in op_names {
        let mut decoded = None;
        if let Some(id) = file(fs, root, OPS, &name, &mut report, |bytes| {
            Operation::from_envelope(bytes)
                .map(|op| decoded = Some(op))
                .map_err(|e| e.to_string())
        }) {
            ops.insert((id, name), decoded.expect("a valid op decoded"));
        }
    }

    let valid_ops: BTreeSet<&HashRef> = ops.keys().map(|(id, _)| id).collect();
    let mut reachable = BTreeSet::new();
    let mut stack = Vec::new();
    for ((_, name), op) in &ops {
        let path = format!("{OPS}/{name}");
        for parent in &op.parents {
            if !valid_ops.contains(parent) {
                report.problems.push(Problem {
                    path: path.clone(),
                    kind: ProblemKind::MissingParent(*parent),
                });
            }
        }
        match objects.get(&op.view) {
            Some((_, ObjectKind::View, _)) => stack.push(op.view),
            _ => report.problems.push(Problem {
                path,
                kind: ProblemKind::MissingReference(op.view),
            }),
        }
    }
    // Every object reachable from a valid op must have its references.
    while let Some(id) = stack.pop() {
        if !reachable.insert(id) {
            continue;
        }
        let (name, _, refs) = &objects[&id];
        for reference in refs {
            if objects.contains_key(reference) {
                stack.push(*reference);
            } else {
                report.problems.push(Problem {
                    path: format!("{OBJECTS}/{name}"),
                    kind: ProblemKind::MissingReference(*reference),
                });
            }
        }
    }
    // An object no op reaches, with a reference missing, is a leftover.
    for (id, (name, _, refs)) in &objects {
        if !reachable.contains(id) && refs.iter().any(|r| !objects.contains_key(r)) {
            report.leftovers.push(format!("{OBJECTS}/{name}"));
        }
    }
    report.problems.sort();
    report.problems.dedup();
    report.leftovers.sort();
    Ok(report)
}

/// Checks one file's name, hash and decoding; returns its id if all hold.
fn file(
    fs: &impl Fs,
    root: &Path,
    dir: &str,
    name: &str,
    report: &mut Report,
    mut decodes: impl FnMut(&[u8]) -> Result<(), String>,
) -> Option<HashRef> {
    let path = format!("{dir}/{name}");
    if name.starts_with(TMP_PREFIX) {
        report.leftovers.push(path);
        return None;
    }
    let mut problem = |kind| {
        report.problems.push(Problem {
            path: path.clone(),
            kind,
        });
        None
    };
    let Some(id) = parse_name(name) else {
        return problem(ProblemKind::BadName);
    };
    let bytes = match fs.read(&root.join(dir).join(name)) {
        Ok(bytes) => bytes,
        Err(error) => return problem(ProblemKind::Unreadable(error.to_string())),
    };
    if object_id(id.alg(), &bytes) != id {
        return problem(ProblemKind::HashMismatch);
    }
    match decodes(&bytes) {
        Ok(()) => Some(id),
        Err(error) => problem(ProblemKind::Undecodable(error)),
    }
}
