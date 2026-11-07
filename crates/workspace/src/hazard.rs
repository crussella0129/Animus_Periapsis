//! The hazard check: names a write would turn into a different file, or
//! could not create, on the filesystem at hand (INT-0004 AC-3, AC-4).
//!
//! The check is a pure function of the from-tree F, the target tree T, the
//! names that exist now in each directory, and the filesystem's
//! [`NameRules`]. It looks only at the paths a write creates or rewrites (in
//! T, with an entry different from F's), every component of each, so a
//! tracked name that stays unchanged never blocks a write.
//! `docs/spec/working-copy.md` ("Hazards") is the specification.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use codec::RepoPath;
use sim::NameRules;

use crate::tree::Tree;

/// Characters Windows refuses in a name, besides codes 0 to 31.
const WINDOWS_CHARS: [char; 7] = ['<', '>', ':', '"', '|', '?', '*'];

/// Windows device names that are reserved whatever follows a `.`.
const RESERVED: [&str; 6] = ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"];
/// The numbered device names: `COM` or `LPT` followed by one of these.
const RESERVED_NUMBERED: [&str; 2] = ["COM", "LPT"];
const RESERVED_DIGITS: [char; 13] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '¹', '²', '³',
];

/// Why a component is a hazard.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Problem {
    /// Another of T's names in the same directory differs only in case.
    CaseCollision {
        /// That other name.
        other: String,
    },
    /// An entry that exists in the directory differs only in case.
    CaseExisting {
        /// The existing entry's name.
        existing: String,
    },
    /// A character Windows refuses.
    WindowsChar(char),
    /// A trailing `.` or space, which Windows strips.
    TrailingDotOrSpace,
    /// A reserved Windows device name, the base given.
    Reserved(String),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::CaseCollision { other } => {
                write!(f, "differs only in case from `{other}`, also written")
            }
            Problem::CaseExisting { existing } => {
                write!(f, "differs only in case from the existing `{existing}`")
            }
            Problem::WindowsChar(c) => write!(f, "holds {c:?}, which Windows refuses"),
            Problem::TrailingDotOrSpace => {
                f.write_str("ends in a `.` or space, which Windows strips")
            }
            Problem::Reserved(base) => write!(f, "`{base}` is a reserved Windows device name"),
        }
    }
}

/// One hazardous component of a path the write needs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hazard {
    /// The path to write.
    pub path: RepoPath,
    /// The offending component of it.
    pub component: String,
    /// What is wrong with it.
    pub problem: Problem,
}

impl fmt::Display for Hazard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: `{}` {}", self.path, self.component, self.problem)
    }
}

/// Whether two names are equal ignoring case: under `to_lowercase` or under
/// `to_uppercase`. Neither alone matches every pair NTFS folds together
/// (`σ`/`ς` agree only in uppercase, and so do `s`/`ſ`), so the test is
/// deliberately conservative.
pub fn fold_eq(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase() || a.to_uppercase() == b.to_uppercase()
}

/// What Windows refuses in `name`, if anything.
pub fn windows_problem(name: &str) -> Option<Problem> {
    if let Some(c) = name
        .chars()
        .find(|c| WINDOWS_CHARS.contains(c) || u32::from(*c) < 32)
    {
        return Some(Problem::WindowsChar(c));
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Some(Problem::TrailingDotOrSpace);
    }
    let base = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches(' ');
    let reserved = RESERVED.iter().any(|r| base.eq_ignore_ascii_case(r))
        || RESERVED_NUMBERED.iter().any(|prefix| {
            base.get(..prefix.len())
                .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
                && {
                    let mut rest = base[prefix.len()..].chars();
                    matches!((rest.next(), rest.next()), (Some(d), None) if RESERVED_DIGITS.contains(&d))
                }
        });
    reserved.then(|| Problem::Reserved(base.to_owned()))
}

/// The parent directory of the component at index `i` of `components`, as a
/// repository path string (`""` at the top).
fn dir_of(components: &[&str], i: usize) -> String {
    components[..i].join("/")
}

/// Every hazard of writing T over F. `existing` holds, for each directory (a
/// repository path, `""` at the top), the names that exist in it now; a
/// directory not listed is taken as empty or missing.
pub fn check(
    from: &Tree,
    target: &Tree,
    existing: &BTreeMap<String, Vec<String>>,
    rules: NameRules,
) -> Vec<Hazard> {
    // T's names in each directory.
    let mut children: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
    for path in target.keys() {
        let components: Vec<&str> = path.components().collect();
        for (i, name) in components.iter().enumerate() {
            children
                .entry(dir_of(&components, i))
                .or_default()
                .insert(name);
        }
    }
    let mut hazards = BTreeSet::new();
    for (path, file) in target {
        if from.get(path) == Some(file) {
            continue;
        }
        let components: Vec<&str> = path.components().collect();
        for (i, name) in components.iter().enumerate() {
            let dir = dir_of(&components, i);
            let mut found = |problem| {
                hazards.insert(Hazard {
                    path: path.clone(),
                    component: (*name).to_owned(),
                    problem,
                });
            };
            if rules.windows_names
                && let Some(problem) = windows_problem(name)
            {
                found(problem);
            }
            if rules.case_insensitive {
                for other in &children[&dir] {
                    if other != name && fold_eq(other, name) {
                        found(Problem::CaseCollision {
                            other: (*other).to_owned(),
                        });
                    }
                }
                for entry in existing.get(&dir).into_iter().flatten() {
                    if entry != name && fold_eq(entry, name) {
                        found(Problem::CaseExisting {
                            existing: entry.clone(),
                        });
                    }
                }
            }
        }
    }
    hazards.into_iter().collect()
}
