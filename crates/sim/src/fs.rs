//! The filesystem trait: the syscalls Periapsis storage makes.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use crate::name::DirEntry;

/// The filesystem operations storage code may use. Each method is one
/// syscall-sized step: [`crate::SimFs`] counts one syscall per call and can
/// crash at any of them. Durability is explicit: written data is durable only
/// after [`Fs::sync_file`], and a directory's entries (new names, renames,
/// removals) only after [`Fs::sync_dir`] of that directory.
pub trait Fs {
    /// Creates a directory; fails with `AlreadyExists` if the name exists and
    /// `NotFound` if the parent does not.
    fn create_dir(&self, path: &Path) -> io::Result<()>;
    /// Creates an empty file; fails with `AlreadyExists` if the name exists.
    fn create_new(&self, path: &Path) -> io::Result<()>;
    /// Appends `data` to an existing file.
    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()>;
    /// Replaces an existing file's content in place: truncate, then write.
    fn overwrite(&self, path: &Path, data: &[u8]) -> io::Result<()>;
    /// Makes a file's data durable.
    fn sync_file(&self, path: &Path) -> io::Result<()>;
    /// Renames a file within its directory, replacing any file at `to`.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    /// Makes a directory's entries durable.
    fn sync_dir(&self, path: &Path) -> io::Result<()>;
    /// Reads a whole file.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    /// The names in a directory, sorted by their bytes. A name with no
    /// UTF-8 form is given lossily (with U+FFFD), so it names no file.
    fn list(&self, path: &Path) -> io::Result<Vec<String>>;
    /// Removes a file.
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    /// A directory's entries with their kinds, sorted by their names'
    /// display forms ([`crate::Name::display`]). Symbolic links are not
    /// followed, and a name with no UTF-8 form is returned as
    /// [`crate::Name::Raw`], never lossily.
    fn scan(&self, path: &Path) -> io::Result<Vec<DirEntry>>;
    /// Whether files on this filesystem carry an owner-execute bit. Not a
    /// syscall.
    fn has_executable_bit(&self) -> bool;
    /// Removes an empty directory.
    fn remove_dir(&self, path: &Path) -> io::Result<()>;
    /// Sets or clears a file's executable bit. Where files have no such bit
    /// ([`Fs::has_executable_bit`] is `false`) it succeeds and changes
    /// nothing, once the file is found. Like data, the change is durable only
    /// after [`Fs::sync_file`].
    fn set_executable(&self, path: &Path, executable: bool) -> io::Result<()>;
    /// Which names this filesystem cannot hold apart or hold at all
    /// (INT-0004). Not a syscall.
    fn name_rules(&self) -> NameRules;
    /// Marks that the code has reached the named point (INT-0005 AC-6). Not
    /// a syscall: on a real filesystem it does nothing; [`crate::SimFs`]
    /// counts it and runs the actions a test attached to the name. Every
    /// implementation states what it does, so no forwarder can drop points.
    fn point(&self, name: &'static str);
}

/// The naming rules of a filesystem, which decide what a write must refuse
/// (INT-0004 AC-3, AC-4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NameRules {
    /// Names that differ only in case name the same entry.
    pub case_insensitive: bool,
    /// Windows' naming rules apply: reserved device names, the characters
    /// `<>:"|?*` and 0-31, and no trailing `.` or space.
    pub windows_names: bool,
}

/// Forwards every [`Fs`] method through a pointer to the filesystem behind
/// it. One definition serves each pointer type, so the three cannot drift
/// apart, and since no method has a default body, a method left out here
/// fails to compile instead of being dropped.
macro_rules! forward_fs {
    ($($(#[$doc:meta])* $pointer:ty;)+) => {$(
        $(#[$doc])*
        impl<T: Fs + ?Sized> Fs for $pointer {
            fn create_dir(&self, path: &Path) -> io::Result<()> {
                (**self).create_dir(path)
            }
            fn create_new(&self, path: &Path) -> io::Result<()> {
                (**self).create_new(path)
            }
            fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
                (**self).write(path, data)
            }
            fn overwrite(&self, path: &Path, data: &[u8]) -> io::Result<()> {
                (**self).overwrite(path, data)
            }
            fn sync_file(&self, path: &Path) -> io::Result<()> {
                (**self).sync_file(path)
            }
            fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
                (**self).rename(from, to)
            }
            fn sync_dir(&self, path: &Path) -> io::Result<()> {
                (**self).sync_dir(path)
            }
            fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
                (**self).read(path)
            }
            fn list(&self, path: &Path) -> io::Result<Vec<String>> {
                (**self).list(path)
            }
            fn remove_file(&self, path: &Path) -> io::Result<()> {
                (**self).remove_file(path)
            }
            fn scan(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
                (**self).scan(path)
            }
            fn has_executable_bit(&self) -> bool {
                (**self).has_executable_bit()
            }
            fn remove_dir(&self, path: &Path) -> io::Result<()> {
                (**self).remove_dir(path)
            }
            fn set_executable(&self, path: &Path, executable: bool) -> io::Result<()> {
                (**self).set_executable(path, executable)
            }
            fn name_rules(&self) -> NameRules {
                (**self).name_rules()
            }
            fn point(&self, name: &'static str) {
                (**self).point(name)
            }
        }
    )+};
}

forward_fs! {
    &T;
    /// Shared ownership, so several stores (and a [`crate::SimFs`] hook) can
    /// use one filesystem.
    Rc<T>;
    /// Shared ownership across threads, so several simulated processes
    /// ([`crate::SimFs`] is `Send + Sync`) can use one filesystem.
    Arc<T>;
}

/// The directory holding `root`'s entry: its parent, or `root/..` when the
/// path has none to give (a bare relative name such as `repo`, or `.`).
pub fn parent_dir(root: &Path) -> PathBuf {
    match root.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => root.join(".."),
    }
}

/// Whether `dir` is a directory that can be listed: `false` when nothing is
/// there. [`Fs`] has no cheaper question to ask.
pub fn dir_exists(fs: &(impl Fs + ?Sized), dir: &Path) -> io::Result<bool> {
    match fs.list(dir) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// The error a simulated syscall returns at and after the crash point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crash;

impl fmt::Display for Crash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("simulated crash")
    }
}

impl Error for Crash {}

impl Crash {
    pub(crate) fn error() -> io::Error {
        io::Error::other(Crash)
    }
}

/// Whether an I/O error is a simulated crash.
pub fn is_crash(error: &io::Error) -> bool {
    error.get_ref().is_some_and(|inner| inner.is::<Crash>())
}
