//! The edit-and-command corpus on the real filesystem: NTFS on Windows, ext4
//! on Linux (INT-0014 AC-1; INT-0004 AC-1; Sprint 2 T-007).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod common;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use common::{Model, ROOT, edit_sequence};
use sim::{Fs, RealFs, Rng};
use workspace::Workspace;

const SEQUENCES: u64 = 20;
const STEPS: usize = 12;

/// A fresh directory under the OS temp directory. Each sequence's
/// repository is `s<seed>/repo` inside it, reached through [`Under`].
fn claim() -> PathBuf {
    let base = std::env::temp_dir();
    let mut rng = Rng::new(u64::from(std::process::id()) ^ 0xed17);
    loop {
        let dir = base.join(format!("ws-edits-{:016x}", rng.next_u64()));
        match RealFs.create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

/// `RealFs` with every path taken under a base directory, so the corpus
/// helpers' relative `repo/...` paths land in a temp directory.
#[derive(Clone)]
struct Under(PathBuf);

impl Fs for Under {
    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        RealFs.create_dir(&self.0.join(path))
    }
    fn create_new(&self, path: &Path) -> std::io::Result<()> {
        RealFs.create_new(&self.0.join(path))
    }
    fn write(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
        RealFs.write(&self.0.join(path), data)
    }
    fn overwrite(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
        RealFs.overwrite(&self.0.join(path), data)
    }
    fn sync_file(&self, path: &Path) -> std::io::Result<()> {
        RealFs.sync_file(&self.0.join(path))
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        RealFs.rename(&self.0.join(from), &self.0.join(to))
    }
    fn sync_dir(&self, path: &Path) -> std::io::Result<()> {
        RealFs.sync_dir(&self.0.join(path))
    }
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        RealFs.read(&self.0.join(path))
    }
    fn list(&self, path: &Path) -> std::io::Result<Vec<String>> {
        RealFs.list(&self.0.join(path))
    }
    fn remove_file(&self, path: &Path) -> std::io::Result<()> {
        RealFs.remove_file(&self.0.join(path))
    }
    fn scan(&self, path: &Path) -> std::io::Result<Vec<sim::DirEntry>> {
        RealFs.scan(&self.0.join(path))
    }
    fn has_executable_bit(&self) -> bool {
        RealFs.has_executable_bit()
    }
    fn remove_dir(&self, path: &Path) -> std::io::Result<()> {
        RealFs.remove_dir(&self.0.join(path))
    }
    fn set_executable(&self, path: &Path, executable: bool) -> std::io::Result<()> {
        RealFs.set_executable(&self.0.join(path), executable)
    }
    fn name_rules(&self) -> sim::NameRules {
        RealFs.name_rules()
    }
}

#[cfg(unix)]
fn set_exec(fs: &Under, name: &str, executable: bool) {
    use std::os::unix::fs::PermissionsExt;
    let path = name.split('/').fold(fs.0.join(ROOT), |p, c| p.join(c));
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    let mode = permissions.mode();
    permissions.set_mode(if executable {
        mode | 0o100
    } else {
        mode & !0o100
    });
    std::fs::set_permissions(&path, permissions).unwrap();
}

#[cfg(not(unix))]
fn set_exec(_: &Under, _: &str, _: bool) {
    unreachable!("no executable bit here");
}

#[test]
fn real_edit_corpus_loses_nothing() {
    let base = claim();
    let mut checked = 0;
    for seed in 0..SEQUENCES {
        let dir = base.join(format!("s{seed}"));
        RealFs.create_dir(&dir).unwrap();
        let fs = Under(dir);
        let mut ws = Workspace::init(fs, Path::new(ROOT), seed).unwrap().0;
        checked += edit_sequence(&mut ws, Model::default(), seed, STEPS, &set_exec);
    }
    println!("real edit corpus: {SEQUENCES} sequences, {checked} snapshots, 0 lost edits");
    assert_eq!(checked, SEQUENCES as usize * STEPS);
    let _ = std::fs::remove_dir_all(&base);
}
