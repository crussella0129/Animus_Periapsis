//! `praps fsck` on real stores (INT-0007 AC-4; exit codes 0, 1 and 2). These
//! tests corrupt files by hand, so they use `std::fs`, which the
//! filesystem boundary allows in `tests/` only.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use codec::{Operation, blob};
use hash::{HashAlg, HashRef};
use sim::{Fs, RealFs, Rng};
use store::Store;
use store::names::file_name;

/// A directory under the OS temp directory that did not exist before.
fn claim(label: &str) -> PathBuf {
    let mut rng = Rng::new(u64::from(std::process::id()) ^ (label.len() as u64) << 32);
    loop {
        let dir = std::env::temp_dir().join(format!("praps-{label}-{:016x}", rng.next_u64()));
        match fs::create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

struct Repo {
    dir: PathBuf,
    object: HashRef,
    op: HashRef,
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// A healthy store on the real filesystem, built through `RealFs`.
fn repo(label: &str) -> Repo {
    let dir = claim(label);
    let root = dir.join("repo");
    let mut store = Store::init(RealFs, &root, HashAlg::DEFAULT, 1).unwrap();
    let object = store.put_object(&blob(b"hello\n")).unwrap();
    let first = store
        .commit_op(&Operation {
            parents: Default::default(),
            objects: [object].into(),
        })
        .unwrap();
    let op = store
        .commit_op(&Operation {
            parents: [first].into(),
            objects: [object].into(),
        })
        .unwrap();
    Repo { dir, object, op }
}

fn praps(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_praps"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn fsck_exits_zero_on_a_healthy_store() {
    let repo = repo("healthy");
    let before = tree(&repo.dir);
    let output = praps(&["fsck", "repo"], &repo.dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(stdout(&output), "clean\n");
    // Checking changes nothing: the tree is the same, and a second run says
    // the same.
    assert_eq!(tree(&repo.dir), before);
    assert_eq!(praps(&["fsck", "repo"], &repo.dir).stdout, output.stdout);
}

/// Every path under `dir` with its content (`None` for a directory).
fn tree(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.push((path.clone(), None));
                stack.push(path);
            } else {
                out.push((path.clone(), Some(fs::read(&path).unwrap())));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn fsck_without_dir_checks_the_current_directory() {
    let repo = repo("cwd");
    let output = praps(&["fsck"], &repo.dir.join("repo"));
    assert_eq!(code(&output), 0, "{output:?}");
}

#[test]
fn fsck_lists_corrupt_files_and_exits_one() {
    let repo = repo("corrupt");
    let root = repo.dir.join("repo");
    let op_path = root.join("ops").join(file_name(&repo.op));
    let object_path = root.join("objects").join(file_name(&repo.object));
    let (op_bytes, object_bytes) = (fs::read(&op_path).unwrap(), fs::read(&object_path).unwrap());

    // Truncate an op and flip a bit in an object, by hand.
    fs::write(&op_path, &op_bytes[..op_bytes.len() - 3]).unwrap();
    let mut flipped = object_bytes.clone();
    flipped[4] ^= 0x01;
    fs::write(&object_path, &flipped).unwrap();
    let output = praps(&["fsck", "repo"], &repo.dir);
    assert_eq!(code(&output), 1, "{output:?}");
    let text = stdout(&output);
    assert!(
        text.contains(&format!("ops/{}", file_name(&repo.op))),
        "{text}"
    );
    assert!(
        text.contains(&format!("objects/{}", file_name(&repo.object))),
        "{text}"
    );

    // Restored, it is clean again.
    fs::write(&op_path, &op_bytes).unwrap();
    fs::write(&object_path, &object_bytes).unwrap();
    assert_eq!(code(&praps(&["fsck", "repo"], &repo.dir)), 0);

    // A crash's leftover temporary file is reported but is not corruption.
    RealFs
        .create_new(&root.join("ops").join("tmp-0000000000000000"))
        .unwrap();
    let output = praps(&["fsck", "repo"], &repo.dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(stdout(&output).contains("temporary file"));
}

#[test]
fn fsck_on_a_non_store_exits_one() {
    let dir = claim("plain");
    fs::write(dir.join("a-file"), b"not a directory").unwrap();
    for target in ["nonexistent", ".", "a-file"] {
        let output = praps(&["fsck", target], &dir);
        assert_eq!(code(&output), 1, "{target}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("praps fsck"));
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn usage_errors_exit_two() {
    let repo = repo("usage");
    for args in [
        &[][..],
        &["--help"][..],
        &["-h"][..],
        &["fsck", "--fix"][..],
        &["check"][..],
        &["fsck", "repo", "extra"][..],
    ] {
        let output = praps(args, &repo.dir);
        assert_eq!(code(&output), 2, "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).starts_with("usage: praps fsck"),
            "{args:?}"
        );
    }
}
