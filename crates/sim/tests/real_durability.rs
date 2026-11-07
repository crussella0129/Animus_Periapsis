//! The real filesystem's directory flush (INT-0004 AC-5): how `RealFs` makes
//! directory entries durable, and that the Windows way needs write access.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use sim::{Fs, RealFs, Rng};

fn claim(label: &str) -> PathBuf {
    let mut rng = Rng::new(u64::from(std::process::id()) ^ 0x6475_7261);
    loop {
        let dir = std::env::temp_dir().join(format!("sim-{label}-{:016x}", rng.next_u64()));
        match fs::create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

/// The full write sequence the store uses, on the real filesystem: every
/// step, `sync_dir` included, succeeds and the result reads back.
#[test]
fn write_sequence_succeeds_on_the_real_filesystem() {
    let dir = claim("sequence");
    let (tmp, file) = (dir.join("tmp-1"), dir.join("01-ab"));
    RealFs.create_new(&tmp).unwrap();
    RealFs.write(&tmp, b"payload").unwrap();
    RealFs.sync_file(&tmp).unwrap();
    RealFs.rename(&tmp, &file).unwrap();
    RealFs.sync_dir(&dir).unwrap();
    assert_eq!(RealFs.read(&file).unwrap(), b"payload");
    assert_eq!(RealFs.list(&dir).unwrap(), ["01-ab"]);
    let _ = fs::remove_dir_all(&dir);
}

/// On Windows a directory opens only with `FILE_FLAG_BACKUP_SEMANTICS`, and
/// `FlushFileBuffers` on it succeeds only with write access: the handle
/// `RealFs` opens flushes, a read-only one is refused.
#[cfg(windows)]
#[test]
fn directory_flush_needs_write_access() {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

    let dir = claim("flush");
    RealFs::open_dir_for_sync(&dir).unwrap().sync_all().unwrap();

    let read_only = fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(&dir)
        .unwrap();
    let error = read_only.sync_all().unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PermissionDenied, "{error}");

    // Without backup semantics a directory does not open at all.
    let error = fs::File::open(&dir).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PermissionDenied, "{error}");
    let _ = fs::remove_dir_all(&dir);
}

/// On Unix a directory opened read-only flushes.
#[cfg(unix)]
#[test]
fn directory_flush_needs_write_access() {
    let dir = claim("flush");
    RealFs::open_dir_for_sync(&dir).unwrap().sync_all().unwrap();
    fs::File::open(&dir).unwrap().sync_all().unwrap();
    let _ = fs::remove_dir_all(&dir);
}
