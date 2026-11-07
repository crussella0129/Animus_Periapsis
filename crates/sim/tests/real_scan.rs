//! `Fs::scan` on the real filesystem, on Windows (NTFS) and Linux (ext4):
//! symlinks not followed, special entries, the executable bit, and names with
//! no UTF-8 form (Sprint 2 T-004; INT-0004 AC-1, AC-7).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use sim::{Fs, Kind, Name, RawName, RealFs, Rng, os_seed};

/// A fresh directory under the OS temp directory.
fn claim(label: &str) -> PathBuf {
    let base = std::env::temp_dir();
    let mut rng = Rng::new(u64::from(std::process::id()) ^ (label.len() as u64) << 32);
    loop {
        let dir = base.join(format!("sim-scan-{label}-{:016x}", rng.next_u64()));
        match RealFs.create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

fn kind_of(dir: &Path, name: &str) -> Option<Kind> {
    RealFs
        .scan(dir)
        .unwrap()
        .into_iter()
        .find(|entry| entry.name.as_utf8() == Some(name))
        .map(|entry| entry.kind)
}

/// Files, directories and a symlink (to a file, not followed), plus a socket
/// on Unix. On Windows, creating a symlink needs Developer Mode or a
/// privilege; without it the symlink case is reported as skipped.
#[test]
fn real_scan_meets_the_contract() {
    let dir = claim("contract");
    RealFs.create_new(&dir.join("file")).unwrap();
    RealFs.create_dir(&dir.join("sub")).unwrap();
    assert_eq!(
        kind_of(&dir, "file"),
        Some(Kind::File { executable: false })
    );
    assert_eq!(kind_of(&dir, "sub"), Some(Kind::Dir));

    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(dir.join("file"), dir.join("link"));
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_file(dir.join("file"), dir.join("link"));
    match linked {
        Ok(()) => assert_eq!(kind_of(&dir, "link"), Some(Kind::Symlink)),
        Err(error) => println!("SKIPPED: symlink case: cannot create a symlink here: {error}"),
    }

    #[cfg(unix)]
    {
        let _socket = std::os::unix::net::UnixListener::bind(dir.join("socket")).unwrap();
        assert_eq!(kind_of(&dir, "socket"), Some(Kind::Other));
    }

    // Sorted by display form.
    let names: Vec<String> = RealFs
        .scan(&dir)
        .unwrap()
        .iter()
        .map(|entry| entry.name.display())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A name with no UTF-8 form is returned as it is, never lossily: bytes
/// that are not UTF-8 on Linux, an unpaired surrogate on Windows.
#[test]
fn real_scan_keeps_non_utf8_names() {
    let dir = claim("raw");
    #[cfg(unix)]
    let (os_name, expected) = {
        use std::os::unix::ffi::OsStringExt;
        let bytes = vec![b'a', 0xff, b'b'];
        (
            std::ffi::OsString::from_vec(bytes.clone()),
            RawName::Bytes(bytes),
        )
    };
    #[cfg(windows)]
    let (os_name, expected) = {
        use std::os::windows::ffi::OsStringExt;
        let units = vec![u16::from(b'a'), 0xd800, u16::from(b'b')];
        (std::ffi::OsString::from_wide(&units), RawName::Wide(units))
    };
    std::fs::write(dir.join(&os_name), b"kept").unwrap();
    let entries = RealFs.scan(&dir).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, Name::Raw(expected));
    assert_eq!(entries[0].kind, Kind::File { executable: false });
    let shown = entries[0].name.display();
    assert!(shown == r"a\xffb" || shown == r"a\u{d800}b", "{shown}");
    assert!(!shown.contains('\u{fffd}'), "never lossy");
    // `list` stays lossy, as documented, so it names no file.
    assert_eq!(RealFs.list(&dir).unwrap(), ["a\u{fffd}b"]);
    assert_eq!(std::fs::read(dir.join(&os_name)).unwrap(), b"kept");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The owner-execute bit on Unix; on Windows there is none.
#[test]
fn real_scan_reports_the_executable_bit() {
    let dir = claim("exec");
    RealFs.create_new(&dir.join("run")).unwrap();
    RealFs.create_new(&dir.join("plain")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(RealFs.has_executable_bit());
        let path = dir.join("run");
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(permissions.mode() | 0o100);
        std::fs::set_permissions(&path, permissions).unwrap();
        assert_eq!(kind_of(&dir, "run"), Some(Kind::File { executable: true }));
    }
    #[cfg(windows)]
    {
        assert!(!RealFs.has_executable_bit());
        assert_eq!(kind_of(&dir, "run"), Some(Kind::File { executable: false }));
    }
    assert_eq!(
        kind_of(&dir, "plain"),
        Some(Kind::File { executable: false })
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Operating-system randomness: two seeds differ (a false failure has
/// probability 2^-64).
#[test]
fn os_seed_varies() {
    assert_ne!(os_seed(), os_seed());
}
