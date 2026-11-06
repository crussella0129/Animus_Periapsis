//! The filesystem trait: the syscalls Periapsis storage makes.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;
use std::rc::Rc;

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
    /// The names in a directory, sorted by their bytes.
    fn list(&self, path: &Path) -> io::Result<Vec<String>>;
    /// Removes a file.
    fn remove_file(&self, path: &Path) -> io::Result<()>;
}

impl<T: Fs + ?Sized> Fs for &T {
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
}

/// Shared ownership, so several stores (and a [`crate::SimFs`] hook) can use
/// one filesystem.
impl<T: Fs + ?Sized> Fs for Rc<T> {
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
