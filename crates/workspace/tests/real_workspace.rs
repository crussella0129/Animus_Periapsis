//! The workspace on the real filesystem: the same tree snapshotted on NTFS
//! and on ext4 gives the same changeset id (INT-0004 AC-2; Sprint 2 T-005).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::io::ErrorKind;
use std::path::PathBuf;

use sim::{Fs, RealFs, Rng};
use workspace::Workspace;

/// A fresh directory under the OS temp directory.
fn claim(label: &str) -> PathBuf {
    let base = std::env::temp_dir();
    let mut rng = Rng::new(u64::from(std::process::id()) ^ (label.len() as u64) << 40);
    loop {
        let dir = base.join(format!("ws-{label}-{:016x}", rng.next_u64()));
        match RealFs.create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

/// The working-copy changeset id of this tree, snapshotted with seed 42 on
/// either system. Paths are `/`-separated UTF-8 in the store whatever the
/// host's separator, and no file here is executable.
/// Recorded on Windows 11 (NTFS) and WSL Ubuntu (ext4), Sprint 2.
const EXPECTED: &str = "blake3:9ad9ad907eb641bf35322a3db8ed790384b4b1c8bd6f5f4377fe54c2142228c3";

#[test]
fn real_snapshot_id_is_host_independent() {
    let dir = claim("hostfree");
    let root = dir.join("repo");
    let (mut ws, _) = Workspace::init(RealFs, &root, 42).unwrap();
    for (path, content) in [
        ("README.md", &b"# Periapsis\r\n"[..]),
        ("src/main.rs", b"fn main() {}\n"),
        ("src/π/deep/name with spaces.txt", b"\x00\xff binary"),
        ("docs/a/b/c/d.md", b""),
        (".gitignore", b"target/\n"),
        ("target/ignored.o", b"never stored"),
    ] {
        let file = path.split('/').fold(root.clone(), |p, c| p.join(c));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, content).unwrap();
    }
    ws.snapshot().unwrap();
    let (id, _, tree) = ws.working_copy().unwrap();
    let paths: Vec<&str> = tree.keys().map(|p| p.as_str()).collect();
    assert_eq!(
        paths,
        [
            ".gitignore",
            "README.md",
            "docs/a/b/c/d.md",
            "src/main.rs",
            "src/π/deep/name with spaces.txt"
        ]
    );
    println!("working-copy changeset: {id}");
    assert_eq!(id.to_string(), EXPECTED);
    let _ = std::fs::remove_dir_all(&dir);
}
