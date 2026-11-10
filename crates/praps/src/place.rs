//! Finding the working copy a command acts on (INT-0037 AC-3).
//!
//! A directory is the **top** of a working copy when listing its
//! `.praps/store` succeeds. A command that opens a working copy acts on the
//! nearest top among the directory it is given and those above it.
//!
//! The search itself is here, over [`sim::Fs`], so it runs on the simulator.
//! Turning a directory into a full path needs the process's current
//! directory, which the caller reads.

use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};

use sim::Fs;
use workspace::{PRAPS_DIR, STORE_DIR};

/// Whether `dir` is the top of a working copy. Listing `dir/.praps/store`
/// does not list `dir` itself, which may be large; [`Fs`] has no cheaper
/// question to ask.
///
/// Absence is `Ok(false)`: nothing is there, or a name on the way is not a
/// directory (a file named `.praps`, say). Any other failure is returned,
/// so that a `.praps` which cannot be read is never passed over for a
/// repository further up.
pub fn is_top(fs: &impl Fs, dir: &Path) -> io::Result<bool> {
    match fs.list(&dir.join(PRAPS_DIR).join(STORE_DIR)) {
        Ok(_) => Ok(true),
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

/// The first of `candidates` that is a top, by position: the nearest, when
/// they are given nearest first. It asks about no directory after the one
/// it finds, or after one whose question fails.
pub fn nearest_top(fs: &impl Fs, candidates: &[PathBuf]) -> io::Result<Option<usize>> {
    for (at, dir) in candidates.iter().enumerate() {
        if is_top(fs, dir)? {
            return Ok(Some(at));
        }
    }
    Ok(None)
}

/// `dir` as a full path, taken from `cwd` as written: `.` is dropped and
/// `..` removes the name before it. No symbolic link is resolved.
pub fn full_path(cwd: &Path, dir: &Path) -> PathBuf {
    let mut out = cwd.to_path_buf();
    for component in dir.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            // A prefix or a root starts the path again; a name extends it.
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Every directory above `full`, nearest first, up to the root.
pub fn above(full: &Path) -> Vec<PathBuf> {
    full.ancestors().skip(1).map(Path::to_path_buf).collect()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use sim::{DirEntry, Mode, NameRules, SimFs};

    use super::*;

    /// Makes each of `dirs`, with every directory on the way.
    fn dirs(fs: &SimFs, dirs: &[&str]) {
        for dir in dirs {
            let mut at = PathBuf::new();
            for name in dir.split('/') {
                at.push(name);
                match fs.create_dir(&at) {
                    Ok(()) => {}
                    Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                    Err(error) => panic!("{}: {error}", at.display()),
                }
            }
        }
    }

    /// `start` and every directory above it, as the caller lists them.
    fn upward(start: &str) -> Vec<PathBuf> {
        let start = Path::new(start);
        let mut all = vec![start.to_path_buf()];
        all.extend(
            above(start)
                .into_iter()
                .filter(|dir| !dir.as_os_str().is_empty()),
        );
        all
    }

    fn found(fs: &impl Fs, start: &str) -> Option<String> {
        let candidates = upward(start);
        nearest_top(fs, &candidates)
            .unwrap()
            .map(|at| candidates[at].to_string_lossy().replace('\\', "/"))
    }

    #[test]
    fn find_stops_at_the_nearest_top() {
        let fs = SimFs::new(1, Mode::Strict);
        dirs(
            &fs,
            &["a/.praps/store", "a/b/c/.praps/store", "a/b/c/d/e", "a/x"],
        );
        assert_eq!(found(&fs, "a/b/c/d/e").as_deref(), Some("a/b/c"));
        assert_eq!(found(&fs, "a/b/c/d").as_deref(), Some("a/b/c"));
        assert_eq!(found(&fs, "a/b/c").as_deref(), Some("a/b/c"));
        assert_eq!(found(&fs, "a/b").as_deref(), Some("a"));
        assert_eq!(found(&fs, "a/x").as_deref(), Some("a"));
        assert_eq!(found(&fs, "a").as_deref(), Some("a"));
    }

    #[test]
    fn find_reports_nowhere_without_a_repository() {
        let fs = SimFs::new(2, Mode::Strict);
        dirs(&fs, &["a/b/c", "a/.praps"]);
        assert_eq!(found(&fs, "a/b/c"), None);
        // A directory that is not there has no top above it either, as far
        // as the search goes: the caller checks that it exists.
        assert_eq!(found(&fs, "a/b/missing"), None);
    }

    /// A `.praps` that holds no `store`, or is a file, does not make its
    /// directory a top: the search goes on to the working copy above.
    #[test]
    fn a_praps_directory_without_a_store_is_not_a_top() {
        let fs = SimFs::new(3, Mode::Strict);
        dirs(&fs, &["a/.praps/store", "a/b/.praps", "a/c"]);
        fs.create_new(Path::new("a/b/.praps/x")).unwrap();
        fs.create_new(Path::new("a/c/.praps")).unwrap();
        assert!(!is_top(&fs, Path::new("a/b")).unwrap());
        assert!(!is_top(&fs, Path::new("a/c")).unwrap());
        assert_eq!(found(&fs, "a/b").as_deref(), Some("a"));
        assert_eq!(found(&fs, "a/c").as_deref(), Some("a"));
    }

    /// A filesystem that refuses to list one path and records every path it
    /// is asked to list.
    struct Refusing<'a> {
        under: &'a SimFs,
        refused: PathBuf,
        asked: RefCell<Vec<PathBuf>>,
    }

    impl Fs for Refusing<'_> {
        fn create_dir(&self, path: &Path) -> io::Result<()> {
            self.under.create_dir(path)
        }
        fn create_new(&self, path: &Path) -> io::Result<()> {
            self.under.create_new(path)
        }
        fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
            self.under.write(path, data)
        }
        fn overwrite(&self, path: &Path, data: &[u8]) -> io::Result<()> {
            self.under.overwrite(path, data)
        }
        fn sync_file(&self, path: &Path) -> io::Result<()> {
            self.under.sync_file(path)
        }
        fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
            self.under.rename(from, to)
        }
        fn sync_dir(&self, path: &Path) -> io::Result<()> {
            self.under.sync_dir(path)
        }
        fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
            self.under.read(path)
        }
        fn list(&self, path: &Path) -> io::Result<Vec<String>> {
            self.asked.borrow_mut().push(path.to_path_buf());
            if path == self.refused {
                return Err(io::Error::new(ErrorKind::PermissionDenied, "refused"));
            }
            self.under.list(path)
        }
        fn remove_file(&self, path: &Path) -> io::Result<()> {
            self.under.remove_file(path)
        }
        fn scan(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
            self.under.scan(path)
        }
        fn has_executable_bit(&self) -> bool {
            self.under.has_executable_bit()
        }
        fn remove_dir(&self, path: &Path) -> io::Result<()> {
            self.under.remove_dir(path)
        }
        fn set_executable(&self, path: &Path, executable: bool) -> io::Result<()> {
            self.under.set_executable(path, executable)
        }
        fn name_rules(&self) -> NameRules {
            self.under.name_rules()
        }
        // The search marks no point, and a point is named by a string
        // literal where it is marked (`cargo policy`, rule `points`), so
        // this wrapper passes none on.
        fn point(&self, _name: &'static str) {}
    }

    /// A failure that is not absence stops the search: it is returned, and
    /// no directory further up is asked about, though one of them is a top.
    #[test]
    fn find_stops_on_a_failure_that_is_not_absence() {
        let under = SimFs::new(4, Mode::Strict);
        dirs(&under, &["a/.praps/store", "a/b/.praps/store"]);
        let fs = Refusing {
            under: &under,
            refused: Path::new("a/b").join(PRAPS_DIR).join(STORE_DIR),
            asked: RefCell::new(Vec::new()),
        };
        let error = nearest_top(&fs, &upward("a/b")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(
            fs.asked.borrow().as_slice(),
            std::slice::from_ref(&fs.refused)
        );
        // Without the refusal the same search finds `a/b`, and from above
        // it, `a`.
        assert_eq!(found(&under, "a/b").as_deref(), Some("a/b"));
    }

    /// The directories searched are taken from the path as written.
    #[test]
    fn ancestors_are_taken_from_the_path_as_written() {
        let full = |cwd: &str, dir: &str| full_path(Path::new(cwd), Path::new(dir));
        if cfg!(windows) {
            assert_eq!(full(r"C:\a\b", "."), Path::new(r"C:\a\b"));
            assert_eq!(full(r"C:\a\b", r"c\d"), Path::new(r"C:\a\b\c\d"));
            assert_eq!(full(r"C:\a\b", r"..\x"), Path::new(r"C:\a\x"));
            assert_eq!(full(r"C:\a\b", r".\c\..\d"), Path::new(r"C:\a\b\d"));
            assert_eq!(full(r"C:\a", r"..\..\.."), Path::new(r"C:\"));
            assert_eq!(full(r"C:\a\b", r"D:\x\y"), Path::new(r"D:\x\y"));
            assert_eq!(
                above(Path::new(r"C:\a\b")),
                [Path::new(r"C:\a"), Path::new(r"C:\")]
            );
        } else {
            assert_eq!(full("/a/b", "."), Path::new("/a/b"));
            assert_eq!(full("/a/b", "c/d"), Path::new("/a/b/c/d"));
            assert_eq!(full("/a/b", "../x"), Path::new("/a/x"));
            assert_eq!(full("/a/b", "./c/../d"), Path::new("/a/b/d"));
            assert_eq!(full("/a", "../../.."), Path::new("/"));
            assert_eq!(full("/a/b", "/x/y"), Path::new("/x/y"));
            assert_eq!(above(Path::new("/a/b")), [Path::new("/a"), Path::new("/")]);
        }
        assert!(above(Path::new(if cfg!(windows) { r"C:\" } else { "/" })).is_empty());
    }
}
