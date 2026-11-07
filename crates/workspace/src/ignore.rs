//! Ignore rules (INT-0014 AC-2; `docs/spec/working-copy.md`).
//!
//! The rules come from the `.gitignore` files inside the working copy, with
//! Git's pattern semantics:
//! - each file applies to its own directory and below;
//! - the deepest file with a match decides, and within one file the last
//!   matching pattern wins;
//! - matching is case-sensitive on every system;
//! - nothing under an ignored directory can be re-included (the scan does not
//!   descend into one, except to visit tracked paths).
//!
//! `.praps` and `.git` at the top are always ignored. Git's global excludes
//! file and `.git/info/exclude` are not read, so the result does not depend
//! on the machine.

use std::path::Path;

use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Names at the top of the working copy that are never snapshotted.
pub const ALWAYS_IGNORED: [&str; 2] = [".praps", ".git"];

/// One `.gitignore`: the directory holding it (as a repository path, `""` at
/// the top) and its patterns.
struct Rules {
    dir: String,
    matcher: Gitignore,
}

/// The `.gitignore` files on the path from the top to the directory being
/// scanned, outermost first.
#[derive(Default)]
pub struct Ignores {
    stack: Vec<Rules>,
}

/// Builds the matcher for one `.gitignore`'s bytes. The matcher's root is
/// `.`, so it strips nothing and is always given paths relative to the
/// file's own directory; the crate's prefix stripping would otherwise also
/// cut a sibling such as `ab/x` under a root `a`.
fn matcher(content: &[u8]) -> Gitignore {
    let mut builder = GitignoreBuilder::new(".");
    builder
        .case_insensitive(false)
        .expect("case-sensitive matching builds");
    for line in String::from_utf8_lossy(content).split('\n') {
        // A CRLF file's lines end in `\r`, which is not part of a pattern.
        let line = line.strip_suffix('\r').unwrap_or(line);
        // A pattern Git cannot parse either is skipped, as Git skips it.
        let _ = builder.add_line(None, line);
    }
    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

impl Ignores {
    /// Enters directory `dir` (a repository path, `""` at the top), with the
    /// content of its `.gitignore` if it has one.
    pub fn push(&mut self, dir: &str, gitignore: Option<&[u8]>) {
        let matcher = gitignore.map_or_else(Gitignore::empty, matcher);
        self.stack.push(Rules {
            dir: dir.to_owned(),
            matcher,
        });
    }

    /// Leaves the directory entered last.
    pub fn pop(&mut self) {
        self.stack.pop();
    }

    /// Whether the entry at repository path `path` (directly inside the
    /// directory entered last) is ignored.
    pub fn is_ignored(&self, path: &str, is_dir: bool) -> bool {
        if !path.contains('/') && ALWAYS_IGNORED.contains(&path) {
            return true;
        }
        for rules in self.stack.iter().rev() {
            let relative = if rules.dir.is_empty() {
                path
            } else {
                match path
                    .strip_prefix(&rules.dir)
                    .and_then(|rest| rest.strip_prefix('/'))
                {
                    Some(rest) => rest,
                    None => continue,
                }
            };
            match rules.matcher.matched(Path::new(relative), is_dir) {
                Match::Ignore(_) => return true,
                Match::Whitelist(_) => return false,
                Match::None => {}
            }
        }
        false
    }
}
