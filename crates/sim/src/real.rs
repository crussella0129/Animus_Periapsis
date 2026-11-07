//! The operating system's filesystem. This is the only module in the core
//! crates that names `std::fs` (INT-0005; enforced by `cargo policy`, rule
//! `fs-boundary`, and by the crate's Clippy configuration).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use crate::fs::Fs;

/// [`Fs`] on the real filesystem, through `std::fs`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RealFs;

impl RealFs {
    /// Opens a directory so that `sync_all` flushes its entries.
    ///
    /// On Unix that is a read-only open. Windows has no POSIX directory fsync:
    /// `FlushFileBuffers` on a directory needs a handle opened with
    /// `FILE_FLAG_BACKUP_SEMANTICS` (required to open a directory at all) and
    /// write access (a read-only handle is refused with "Access is denied").
    /// Microsoft does not document `FlushFileBuffers` for directory handles;
    /// `docs/spec/storage-durability.md` records the evidence that it works.
    pub fn open_dir_for_sync(path: &Path) -> io::Result<File> {
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
            OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
                .open(path)
        }
        #[cfg(not(windows))]
        {
            File::open(path)
        }
    }
}

impl Fs for RealFs {
    fn create_dir(&self, path: &Path) -> io::Result<()> {
        fs::create_dir(path)
    }

    fn create_new(&self, path: &Path) -> io::Result<()> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map(drop)
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        OpenOptions::new().append(true).open(path)?.write_all(data)
    }

    fn overwrite(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?
            .write_all(data)
    }

    fn sync_file(&self, path: &Path) -> io::Result<()> {
        OpenOptions::new().write(true).open(path)?.sync_all()
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn sync_dir(&self, path: &Path) -> io::Result<()> {
        Self::open_dir_for_sync(path)?.sync_all()
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn list(&self, path: &Path) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(path)? {
            // A name with no UTF-8 form is listed lossily, so callers see
            // it (and `fsck` reports it as a bad name) instead of failing
            // the whole listing.
            names.push(entry?.file_name().to_string_lossy().into_owned());
        }
        names.sort();
        Ok(names)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}
