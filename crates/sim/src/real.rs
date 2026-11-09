//! The operating system's filesystem. This is the only module in the core
//! crates that names `std::fs` (INT-0005; enforced by `cargo policy`, rule
//! `fs-boundary`, and by the crate's Clippy configuration).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::borrow::Cow;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use crate::fs::{Fs, NameRules};
use crate::name::{DirEntry, Kind, Name, RawName};

/// [`Fs`] on the real filesystem, through `std::fs`.
///
/// On Windows every path is turned into a verbatim (`\\?\`) path before
/// use. Win32 path normalisation otherwise strips a trailing `.` or space
/// from a name and turns `CON` or `aux.txt` into a device, so a file whose
/// real NTFS name is `foo.` would be read as `foo` (code review CR-02).
#[derive(Clone, Copy, Debug, Default)]
pub struct RealFs;

/// `path` as the operating system must be given it: unchanged on Unix.
#[cfg(not(windows))]
fn native(path: &Path) -> io::Result<Cow<'_, Path>> {
    Ok(Cow::Borrowed(path))
}

/// `path` as a verbatim Windows path: absolute (relative paths are taken
/// from the current directory), `\`-separated, with `.` dropped and `..`
/// taken lexically, and every name passed through untouched.
#[cfg(windows)]
fn native(path: &Path) -> io::Result<Cow<'_, Path>> {
    use std::path::{Component, PathBuf, Prefix};
    let mut components = path.components().peekable();
    let base: PathBuf = match components.peek() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Verbatim(_)
            | Prefix::VerbatimDisk(_)
            | Prefix::VerbatimUNC(..)
            | Prefix::DeviceNS(_) => return Ok(Cow::Borrowed(path)),
            Prefix::Disk(letter) => {
                components.next();
                if components.peek() != Some(&Component::RootDir) {
                    // A drive-relative path such as `C:foo`: left to Win32.
                    return Ok(Cow::Borrowed(path));
                }
                components.next();
                PathBuf::from(format!("\\\\?\\{}:\\", char::from(letter)))
            }
            Prefix::UNC(server, share) => {
                components.next();
                if components.peek() == Some(&Component::RootDir) {
                    components.next();
                }
                let mut base = PathBuf::from("\\\\?\\UNC");
                base.push(server);
                base.push(share);
                base
            }
        },
        // Rooted but with no drive (`\foo`): left to Win32.
        Some(Component::RootDir) => return Ok(Cow::Borrowed(path)),
        _ => {
            let cwd = std::env::current_dir()?;
            match native(&cwd)? {
                Cow::Owned(verbatim) => verbatim,
                Cow::Borrowed(same) => same.to_path_buf(),
            }
        }
    };
    let floor = base.components().count();
    let mut out = base;
    for component in components {
        match component {
            Component::Normal(name) => out.push(name),
            Component::CurDir => {}
            Component::ParentDir => {
                if out.components().count() > floor {
                    out.pop();
                }
            }
            Component::Prefix(_) | Component::RootDir => {}
        }
    }
    Ok(Cow::Owned(out))
}

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
        let path = native(path)?;
        let path = path.as_ref();
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
        fs::create_dir(native(path)?)
    }

    fn create_new(&self, path: &Path) -> io::Result<()> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(native(path)?)
            .map(drop)
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        OpenOptions::new()
            .append(true)
            .open(native(path)?)?
            .write_all(data)
    }

    fn overwrite(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(native(path)?)?
            .write_all(data)
    }

    fn sync_file(&self, path: &Path) -> io::Result<()> {
        OpenOptions::new()
            .write(true)
            .open(native(path)?)?
            .sync_all()
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(native(from)?, native(to)?)
    }

    fn sync_dir(&self, path: &Path) -> io::Result<()> {
        Self::open_dir_for_sync(path)?.sync_all()
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(native(path)?)
    }

    fn list(&self, path: &Path) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(native(path)?)? {
            // A name with no UTF-8 form is listed lossily, so callers see
            // it (and `fsck` reports it as a bad name) instead of failing
            // the whole listing.
            names.push(entry?.file_name().to_string_lossy().into_owned());
        }
        names.sort();
        Ok(names)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(native(path)?)
    }

    fn scan(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(native(path)?)? {
            let entry = entry?;
            // `DirEntry::file_type` does not follow symbolic links. An entry
            // removed since the listing is left out, as if listed a moment
            // later (code review CR-05).
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            let kind = if file_type.is_symlink() {
                Kind::Symlink
            } else if file_type.is_dir() {
                Kind::Dir
            } else if file_type.is_file() {
                match executable(&entry) {
                    Ok(executable) => Kind::File { executable },
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(error),
                }
            } else {
                Kind::Other
            };
            entries.push(DirEntry {
                name: name_of(entry.file_name()),
                kind,
            });
        }
        entries.sort_by_cached_key(|entry| entry.name.display());
        Ok(entries)
    }

    fn has_executable_bit(&self) -> bool {
        cfg!(unix)
    }

    fn remove_dir(&self, path: &Path) -> io::Result<()> {
        fs::remove_dir(native(path)?)
    }

    fn set_executable(&self, path: &Path, executable: bool) -> io::Result<()> {
        let path = native(path)?;
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("not a regular file: {}", path.display()),
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            // As Git does: execute wherever read is allowed, or nowhere.
            let mode = if executable {
                mode | ((mode & 0o444) >> 2)
            } else {
                mode & !0o111
            };
            fs::set_permissions(&path, fs::Permissions::from_mode(mode))
        }
        #[cfg(not(unix))]
        {
            let _ = executable;
            Ok(())
        }
    }

    /// Does nothing: points exist for the simulator.
    fn point(&self, _name: &'static str) {}

    /// From the platform: Windows' rules and case-insensitive names there,
    /// neither elsewhere. A case-sensitive NTFS directory, or a
    /// case-insensitive mount on Linux, is not detected (backlog T-015).
    fn name_rules(&self) -> NameRules {
        NameRules {
            case_insensitive: cfg!(windows),
            windows_names: cfg!(windows),
        }
    }
}

/// A name exactly as the filesystem gave it.
fn name_of(name: std::ffi::OsString) -> Name {
    match name.into_string() {
        Ok(utf8) => Name::Utf8(utf8),
        #[cfg(unix)]
        Err(raw) => {
            use std::os::unix::ffi::OsStringExt;
            Name::Raw(RawName::Bytes(raw.into_vec()))
        }
        #[cfg(windows)]
        Err(raw) => {
            use std::os::windows::ffi::OsStrExt;
            Name::Raw(RawName::Wide(raw.encode_wide().collect()))
        }
    }
}

/// Whether a regular file's owner-execute bit is set; always `false` where
/// files have no such bit.
fn executable(entry: &fs::DirEntry) -> io::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Ok(entry.metadata()?.permissions().mode() & 0o100 != 0)
    }
    #[cfg(not(unix))]
    {
        let _ = entry;
        Ok(false)
    }
}

/// 64 bits from the operating system's random source, to seed an [`crate::Rng`]
/// outside simulation (change IDs must differ between runs; INT-0011).
pub fn os_seed() -> u64 {
    getrandom::u64().expect("the operating system provides randomness")
}
