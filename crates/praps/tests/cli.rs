//! `praps` on real stores and working copies (INT-0007 AC-4, INT-0014 AC-1;
//! exit codes 0, 1 and 2). These
//! tests corrupt files by hand, so they use `std::fs`, which the
//! filesystem boundary allows in `tests/` only.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use codec::{Operation, View, blob};
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
    let blobs = [object];
    let holder = codec::Changeset {
        parents: Vec::new(),
        change_id: [0; 16],
        changes: blobs
            .iter()
            .enumerate()
            .map(|(i, blob)| {
                (
                    codec::RepoPath::new(format!("f{i}")).unwrap(),
                    codec::Entry::Put {
                        blob: *blob,
                        mode: codec::Mode::Regular,
                    },
                )
            })
            .collect(),
        description: Vec::new(),
    }
    .to_envelope();
    let changeset = store.put_object(&holder).unwrap();
    let view = View {
        heads: [changeset].into(),
        workspaces: Default::default(),
    };
    let view = store.put_object(&view.to_envelope()).unwrap();
    let first = store
        .commit_op(&Operation {
            parents: Default::default(),
            view,
        })
        .unwrap();
    let op = store
        .commit_op(&Operation {
            parents: [first].into(),
            view,
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
    assert!(
        stdout(&output).contains("tmp-0000000000000000: left by an interrupted write"),
        "{output:?}"
    );
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
            String::from_utf8_lossy(&output.stderr).starts_with("usage: praps "),
            "{args:?}"
        );
    }
}

// ------------------------------------------------- init, new, status (T-006)

fn write(root: &Path, path: &str, content: &[u8]) {
    let file = path.split('/').fold(root.to_path_buf(), |p, c| p.join(c));
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, content).unwrap();
}

fn lines(output: &Output) -> Vec<String> {
    stdout(output).lines().map(str::to_owned).collect()
}

#[test]
fn praps_init_creates_a_repository() {
    let dir = claim("init");
    let output = praps(&["init", "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(stdout(&output).starts_with("initialised a Periapsis repository"));
    assert!(dir.join("repo/.praps/store/objects").is_dir());
    assert!(dir.join("repo/.praps/store/ops").is_dir());
    // Without DIR, the current directory.
    let here = dir.join("here");
    fs::create_dir(&here).unwrap();
    assert_eq!(code(&praps(&["init"], &here)), 0);
    assert!(here.join(".praps/store/ops").is_dir());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_init_is_idempotent() {
    let dir = claim("init-again");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let before = tree(&dir.join("repo"));
    let output = praps(&["init", "repo"], &dir);
    assert_eq!(code(&output), 0);
    assert!(stdout(&output).contains("already a Periapsis repository"));
    assert_eq!(tree(&dir.join("repo")), before, "nothing written");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_new_prints_a_change_id() {
    let dir = claim("new");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "f", b"x");
    let first = praps(&["new"], &root);
    let second = praps(&["new", "repo"], &dir);
    for output in [&first, &second] {
        assert_eq!(code(output), 0, "{output:?}");
        let text = stdout(output);
        let hex = text.trim().strip_prefix("new change ").unwrap();
        assert_eq!(hex.len(), 32, "{text}");
        assert!(hex.bytes().all(|b| b.is_ascii_hexdigit()));
    }
    assert_ne!(
        stdout(&first),
        stdout(&second),
        "each change gets its own ID"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_status_lists_changes() {
    let dir = claim("status");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "keep", b"k");
    write(&root, "edit", b"1");
    write(&root, "drop", b"d");
    write(&root, ".gitignore", b"*.o\n");
    write(&root, "x.o", b"ignored");
    let output = praps(&["status", "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(
        lines(&output),
        ["A .gitignore", "A drop", "A edit", "A keep"]
    );
    assert_eq!(code(&praps(&["new", "repo"], &dir)), 0);
    write(&root, "edit", b"2");
    fs::remove_file(root.join("drop")).unwrap();
    write(&root, "sub/added", b"a");
    let output = praps(&["status"], &root);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(lines(&output), ["D drop", "M edit", "A sub/added"]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_status_outside_a_repository_fails() {
    let dir = claim("status-outside");
    for args in [&["status"][..], &["new"][..], &["status", "nowhere"][..]] {
        let output = praps(args, &dir);
        assert_eq!(code(&output), 1, "{args:?}: {output:?}");
        assert!(output.stdout.is_empty());
        let err = String::from_utf8_lossy(&output.stderr);
        assert!(
            err.contains("not an initialised Periapsis repository"),
            "{err}"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

/// Two ops with the same parent: `status` stops, names both heads and exits
/// 1 (until INT-0010's merge).
#[test]
fn praps_status_with_several_heads_fails() {
    let dir = claim("heads");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let mut store = Store::open(RealFs, &root.join(".praps/store"), HashAlg::DEFAULT, 5).unwrap();
    let head = *store.heads().unwrap().first().unwrap();
    let view = store.ops().unwrap()[&head].view;
    let a = store
        .commit_op(&Operation {
            parents: [head].into(),
            view,
        })
        .unwrap();
    let mut other = View::from_envelope(&store.get_object(&view).unwrap()).unwrap();
    other
        .workspaces
        .insert("other".to_owned(), other.workspaces["default"]);
    let other = store.put_object(&other.to_envelope()).unwrap();
    let b = store
        .commit_op(&Operation {
            parents: [head].into(),
            view: other,
        })
        .unwrap();
    let output = praps(&["status", "repo"], &dir);
    assert_eq!(code(&output), 1, "{output:?}");
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("2 heads"), "{err}");
    assert!(
        err.contains(&a.to_string()) && err.contains(&b.to_string()),
        "{err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_fsck_checks_a_working_copy() {
    let dir = claim("fsck-wc");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "f", b"x");
    assert_eq!(code(&praps(&["status", "repo"], &dir)), 0);
    let output = praps(&["fsck", "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(stdout(&output).trim(), "clean");
    // Corrupt one object of the repository's store: fsck on the working copy
    // finds it.
    let objects = root.join(".praps/store/objects");
    let victim = fs::read_dir(&objects)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut bytes = fs::read(&victim).unwrap();
    *bytes.last_mut().unwrap() ^= 0x01;
    fs::write(&victim, &bytes).unwrap();
    let output = praps(&["fsck"], &root);
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(stdout(&output).contains("does not hash to its name"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_rejects_flags_and_extra_arguments() {
    let dir = claim("flags");
    for args in [
        &["init", "--bare"][..],
        &["status", "-s"][..],
        &["new", "-m", "message"][..],
        &["status", "a", "b"][..],
        &["init", "a", "b"][..],
        &["snapshot"][..],
    ] {
        let output = praps(args, &dir);
        assert_eq!(code(&output), 2, "{args:?}: {output:?}");
        assert!(output.stdout.is_empty());
    }
    assert!(
        fs::read_dir(&dir).unwrap().next().is_none(),
        "nothing created"
    );
    let _ = fs::remove_dir_all(&dir);
}
