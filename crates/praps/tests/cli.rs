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

/// The working-copy changeset op `op`'s view names in `default`.
fn wc_of(store: &Store<RealFs>, op: &HashRef) -> HashRef {
    let view = store.ops().unwrap()[op].view;
    let view = View::from_envelope(&store.get_object(&view).unwrap()).unwrap();
    *view.workspaces["default"].working_copy.first().unwrap()
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
        workspaces: [(
            "default".to_owned(),
            codec::WorkspaceEntry {
                working_copy: [changeset].into(),
                heads: Default::default(),
            },
        )]
        .into(),
    };
    let view = store.put_object(&view.to_envelope()).unwrap();
    let first = store
        .commit_op(&Operation {
            parents: Default::default(),
            workspace: "default".to_owned(),
            view,
        })
        .unwrap();
    let op = store
        .commit_op(&Operation {
            parents: [first].into(),
            workspace: "default".to_owned(),
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
fn verify_exits_zero_on_a_healthy_store() {
    let repo = repo("healthy");
    let before = tree(&repo.dir);
    let output = praps(&["verify", "repo"], &repo.dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(stdout(&output), "clean\n");
    // Checking changes nothing: the tree is the same, and a second run says
    // the same.
    assert_eq!(tree(&repo.dir), before);
    assert_eq!(praps(&["verify", "repo"], &repo.dir).stdout, output.stdout);
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
fn verify_without_dir_checks_the_current_directory() {
    let repo = repo("cwd");
    let output = praps(&["verify"], &repo.dir.join("repo"));
    assert_eq!(code(&output), 0, "{output:?}");
}

#[test]
fn verify_lists_corrupt_files_and_exits_one() {
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
    let output = praps(&["verify", "repo"], &repo.dir);
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
    assert_eq!(code(&praps(&["verify", "repo"], &repo.dir)), 0);

    // A crash's leftover temporary file is reported but is not corruption.
    RealFs
        .create_new(&root.join("ops").join("tmp-0000000000000000"))
        .unwrap();
    let output = praps(&["verify", "repo"], &repo.dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(
        stdout(&output).contains("tmp-0000000000000000: left by an interrupted write"),
        "{output:?}"
    );
}

#[test]
fn verify_on_a_non_store_exits_one() {
    let dir = claim("plain");
    fs::write(dir.join("a-file"), b"not a directory").unwrap();
    for target in ["nonexistent", ".", "a-file"] {
        let output = praps(&["verify", target], &dir);
        assert_eq!(code(&output), 1, "{target}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("praps verify"));
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn usage_errors_exit_two() {
    let repo = repo("usage");
    // `praps` alone, `--help` and `-h` now succeed, and a word that is not
    // a command has an answer of its own (Sprint 6 T-006): see
    // `praps_alone_lists_every_command` and
    // `unknown_words_get_the_nearest_commands`.
    for args in [&["verify", "--fix"][..], &["verify", "repo", "extra"][..]] {
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
    // A directory that is not there has a test of its own,
    // `a_missing_directory_is_not_searched_above` (Sprint 6 T-008).
    for args in [&["status"][..], &["new"][..]] {
        let output = praps(args, &dir);
        assert_eq!(code(&output), 1, "{args:?}: {output:?}");
        assert!(output.stdout.is_empty());
        let err = String::from_utf8_lossy(&output.stderr);
        assert!(
            err.contains("no Periapsis repository here or in any directory above"),
            "{err}"
        );
    }
    assert!(
        fs::read_dir(&dir).unwrap().next().is_none(),
        "nothing written"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Two ops with the same parent: `status` works from their merge and exits
/// 0 (INT-0010). Until Sprint 7 it stopped, named both heads and exited 1.
#[test]
fn praps_status_with_several_heads_works() {
    let dir = claim("heads");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let mut store = Store::open(RealFs, &root.join(".praps/store"), HashAlg::DEFAULT, 5).unwrap();
    let head = *store.heads().unwrap().first().unwrap();
    let view = store.ops().unwrap()[&head].view;
    store
        .commit_op(&Operation {
            parents: [head].into(),
            workspace: "default".to_owned(),
            view,
        })
        .unwrap();
    let mut other = View::from_envelope(&store.get_object(&view).unwrap()).unwrap();
    other
        .workspaces
        .insert("other".to_owned(), other.workspaces["default"].clone());
    let other = store.put_object(&other.to_envelope()).unwrap();
    store
        .commit_op(&Operation {
            parents: [head].into(),
            workspace: "default".to_owned(),
            view: other,
        })
        .unwrap();
    let output = praps(&["status", "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_verify_checks_a_working_copy() {
    let dir = claim("verify-wc");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "f", b"x");
    assert_eq!(code(&praps(&["status", "repo"], &dir)), 0);
    let output = praps(&["verify", "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(stdout(&output).trim(), "clean");
    // Corrupt one object of the repository's store: verify on the working copy
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
    let output = praps(&["verify"], &root);
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
        &["log", "-n", "3"][..],
        &["log", "a", "b"][..],
        &["ops", "-n", "3"][..],
        &["ops", "a", "b"][..],
        &["undo", "--force"][..],
        &["undo", "a", "b"][..],
        &["restore", "-f", "abc"][..],
        &["restore", "abc", "--force"][..],
        &["describe"][..],
        &["describe", "-m", "text"][..],
        &["describe", "-text"][..],
        &["describe", "a", "b", "c"][..],
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

/// When `DIR/.praps/store` cannot be opened, `verify DIR` reports that, not a
/// bare-store error about `DIR` (code review CR-09; test critique C-004).
#[test]
fn praps_verify_reports_a_broken_repository_store() {
    let dir = claim("verify-broken");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    fs::remove_dir_all(root.join(".praps/store/ops")).unwrap();
    let output = praps(&["verify", "repo"], &dir);
    assert_eq!(code(&output), 1, "{output:?}");
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains(".praps") && err.contains("store"), "{err}");
    assert!(err.contains("is not a Periapsis store"), "{err}");
    // `praps` names the argument as given (`repo`); the bare-store fallback
    // would report `repo: repo is not …` (test critique round 2, C-001).
    assert!(
        !err.contains("repo: repo is not"),
        "the bare-store fallback ran: {err}"
    );
    assert!(
        err.contains(".praps/store is not") || err.contains(".praps\\store is not"),
        "{err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

// ------------------------------------------- log, undo, restore (Sprint 3 T-008)

/// The 12-hex op prefixes `praps ops` prints, newest first.
fn logged_ops(root: &Path) -> Vec<String> {
    let output = praps(&["ops"], root);
    assert_eq!(code(&output), 0, "{output:?}");
    lines(&output)
        .iter()
        .map(|line| line.split(' ').next().unwrap().to_owned())
        .collect()
}

#[test]
fn praps_ops_lists_operations_newest_first() {
    let dir = claim("ops");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "a", b"1");
    write(&root, "b", b"1");
    assert_eq!(code(&praps(&["new", "repo"], &dir)), 0);
    let output = praps(&["ops", "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    let lines = lines(&output);
    // init, the snapshot `new` took, `new` itself.
    assert_eq!(lines.len(), 3, "{lines:?}");
    for line in &lines {
        let parts: Vec<&str> = line.split(' ').collect();
        assert_eq!(parts.len(), 5, "{line}");
        assert!(parts[0].len() == 12 && parts[0].bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(parts[1], "change");
        assert_eq!(parts[2].len(), 12);
    }
    // Each line counts the paths its working-copy change changes: the
    // change `new` started is empty, the snapshot before it added `a` and
    // `b`, and the root's change is empty (Sprint 6 T-009; the count was
    // the size of the tree, ` 2 files`, ` 2 files` and ` 0 files`).
    assert!(lines[0].ends_with(" 0 paths"), "{lines:?}");
    assert!(lines[1].ends_with(" 2 paths"), "{lines:?}");
    assert!(
        lines[2].ends_with(" 0 paths"),
        "the root op is last: {lines:?}"
    );
    // The newest is the head.
    let store = Store::open(RealFs, &root.join(".praps/store"), HashAlg::DEFAULT, 1).unwrap();
    let head = *store.heads().unwrap().first().unwrap();
    assert!(head.to_string().contains(&lines[0][..12]));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_undo_restores_files() {
    let dir = claim("undo");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "a", b"1");
    assert_eq!(code(&praps(&["new", "repo"], &dir)), 0);
    write(&root, "a", b"2");
    write(&root, "sub/b", b"new");
    let output = praps(&["undo", "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    // The op printed is the parent of H, the head when `undo` started (the
    // `new` op): the log reads undo, undo's snapshot of the edits, H, then
    // H's parent (test critique C-010).
    let ops = logged_ops(&root);
    assert_eq!(
        stdout(&output).trim_end(),
        format!(
            "undid the last op: the working copy is back to op {}",
            ops[3]
        ),
        "{ops:?}"
    );
    assert_eq!(fs::read(root.join("a")).unwrap(), b"1");
    assert!(
        !root.join("sub").exists(),
        "the emptied directory is removed"
    );
    // Undo again: back to the edits.
    assert_eq!(code(&praps(&["undo"], &root)), 0);
    assert_eq!(fs::read(root.join("a")).unwrap(), b"2");
    assert_eq!(fs::read(root.join("sub/b")).unwrap(), b"new");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_undo_with_nothing_to_undo_fails() {
    let dir = claim("undo-root");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let output = praps(&["undo", "repo"], &dir);
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("nothing to undo"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_restore_by_prefix() {
    let dir = claim("restore");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "a", b"1");
    assert_eq!(code(&praps(&["status"], &root)), 0);
    let first = logged_ops(&root)[0].clone();
    write(&root, "a", b"2");
    assert_eq!(code(&praps(&["status"], &root)), 0);
    let output = praps(&["restore", &first, "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(
        stdout(&output).trim(),
        format!("restored the working copy to op {first}")
    );
    assert_eq!(fs::read(root.join("a")).unwrap(), b"1");
    // Without DIR, the current directory; with the `blake3:` form.
    // Newest first: the restore op, the snapshot of `a` = 2, the first.
    let second = logged_ops(&root)[1].clone();
    assert_eq!(
        code(&praps(&["restore", &format!("blake3:{second}")], &root)),
        0
    );
    assert_eq!(fs::read(root.join("a")).unwrap(), b"2");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_restore_unknown_or_ambiguous_fails() {
    let dir = claim("restore-bad");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    for i in 0..20u8 {
        write(&root, "a", &[b'a' + i]);
        assert_eq!(code(&praps(&["status"], &root)), 0);
    }
    let ops = logged_ops(&root);
    let shared = "0123456789abcdef"
        .chars()
        .map(String::from)
        .find(|p| ops.iter().filter(|op| op.starts_with(p.as_str())).count() >= 2)
        .unwrap();
    let output = praps(&["restore", &shared], &root);
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("matches"),
        "{output:?}"
    );
    let output = praps(&["restore", "nothing"], &root);
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("no op matches"));
    assert_eq!(logged_ops(&root).len(), ops.len(), "no op was added");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_restore_without_op_is_a_usage_error() {
    let dir = claim("restore-usage");
    for args in [&["restore"][..], &["restore", "a", "b", "c"][..]] {
        let output = praps(args, &dir);
        assert_eq!(code(&output), 2, "{args:?}: {output:?}");
        assert!(output.stdout.is_empty());
    }
    let _ = fs::remove_dir_all(&dir);
}

/// A restore refused before any write exits 1 with the error on stderr,
/// writes nothing and adds no op (T-008; test critique C-001). An ignored
/// file stands at a path the target would write; on Windows, a case-only
/// rename is refused as a hazard too.
#[test]
fn praps_restore_refusals_exit_one() {
    let dir = claim("refusals");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "b", b"tracked");
    assert_eq!(code(&praps(&["status"], &root)), 0);
    let with_b = logged_ops(&root)[0].clone();
    // `b` must be untracked before it is ignored: a tracked file stays
    // tracked, as in Git.
    fs::remove_file(root.join("b")).unwrap();
    assert_eq!(code(&praps(&["status"], &root)), 0);
    write(&root, ".gitignore", b"b\n");
    write(&root, "b", b"mine");
    assert_eq!(code(&praps(&["status"], &root)), 0);
    let before = logged_ops(&root);

    let output = praps(&["restore", &with_b], &root);
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("untracked entries are in the way"), "{err}");
    assert!(err.contains("\n  b"), "{err}");
    assert_eq!(fs::read(root.join("b")).unwrap(), b"mine");
    assert_eq!(fs::read(root.join(".gitignore")).unwrap(), b"b\n");
    assert!(!root.join(".praps/working-copy").exists());
    assert_eq!(logged_ops(&root), before, "no op was added");

    if cfg!(windows) {
        fs::remove_file(root.join("b")).unwrap();
        fs::remove_file(root.join(".gitignore")).unwrap();
        write(&root, "README", b"upper");
        assert_eq!(code(&praps(&["status"], &root)), 0);
        let upper = logged_ops(&root)[0].clone();
        fs::remove_file(root.join("README")).unwrap();
        write(&root, "readme", b"lower");
        assert_eq!(code(&praps(&["status"], &root)), 0);
        let before = logged_ops(&root);
        let output = praps(&["restore", &upper], &root);
        assert_eq!(code(&output), 1, "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        let err = String::from_utf8_lossy(&output.stderr);
        assert!(
            err.contains("cannot be written on this filesystem"),
            "{err}"
        );
        assert!(err.contains("README"), "{err}");
        let names: Vec<String> = fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .filter(|n| n != ".praps")
            .collect();
        assert_eq!(names, ["readme"]);
        assert_eq!(fs::read(root.join("readme")).unwrap(), b"lower");
        assert_eq!(logged_ops(&root), before, "no op was added");
    }
    let _ = fs::remove_dir_all(&dir);
}

/// A record left by an interrupted write whose target is no longer the head:
/// the next command drops it and says so on stderr.
#[test]
fn praps_prints_notes() {
    let dir = claim("notes");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "a", b"1");
    assert_eq!(code(&praps(&["status"], &root)), 0);
    let store = Store::open(RealFs, &root.join(".praps/store"), HashAlg::DEFAULT, 1).unwrap();
    let head = *store.heads().unwrap().first().unwrap();
    let root_op = *store.ops().unwrap()[&head].parents.first().unwrap();
    workspace::record::write(
        &RealFs,
        &root.join(".praps"),
        &workspace::record::Record {
            from: Some(wc_of(&store, &head)),
            target: root_op,
        },
    )
    .unwrap();
    let output = praps(&["status"], &root);
    assert_eq!(code(&output), 0, "{output:?}");
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.starts_with("note: "), "{err}");
    assert!(err.contains("was abandoned"), "{err}");
    assert!(err.contains(&root_op.to_string()), "{err}");
    assert!(!root.join(".praps/working-copy").exists());
    assert_eq!(fs::read(root.join("a")).unwrap(), b"1");
    let _ = fs::remove_dir_all(&dir);
}

// --------------------- the registry, help and suggestions (Sprint 6 T-006)

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The last line every refusal about a word ends with.
const LIST_HINT: &str = "hint: run `praps help` to list the commands";

/// The commands `praps` lists, each with its summary: the lines of the list
/// that start with two spaces.
fn listed(dir: &Path) -> Vec<(String, String)> {
    let output = praps(&[], dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    stdout(&output)
        .lines()
        .filter(|line| line.starts_with("  "))
        .map(|line| {
            let (name, summary) = line.trim().split_once(' ').unwrap();
            (name.to_owned(), summary.trim().to_owned())
        })
        .collect()
}

/// `praps` alone lists the commands and exits 0, and every name it lists is
/// one `praps help` explains (INT-0037 AC-2).
#[test]
fn praps_alone_lists_every_command() {
    let dir = claim("list");
    let commands = listed(&dir);
    let names: Vec<&str> = commands.iter().map(|(name, _)| name.as_str()).collect();
    for expected in [
        "status", "describe", "new", "log", "undo", "ops", "restore", "init", "verify", "help",
        "version",
    ] {
        assert!(names.contains(&expected), "{expected} is not in {names:?}");
    }
    let list = stdout(&praps(&[], &dir));
    assert!(
        list.find("\nEvery day\n") < list.find("\nLooking back\n")
            && list.find("\nLooking back\n") < list.find("\nStarting and care\n"),
        "{list}"
    );
    assert_eq!(stdout(&praps(&["help"], &dir)), list);
    for (name, _) in &commands {
        let output = praps(&["help", name], &dir);
        assert_eq!(code(&output), 0, "help {name}: {output:?}");
    }
    assert!(
        fs::read_dir(&dir).unwrap().next().is_none(),
        "nothing created"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// `praps help WORD` prints the command's usage line, its summary, its
/// arguments when it takes any, an example, and whether it is idempotent
/// (INT-0037 AC-2; INT-0022 AC-11).
#[test]
fn praps_help_explains_every_command() {
    let dir = claim("help");
    for (name, summary) in listed(&dir) {
        let output = praps(&["help", &name], &dir);
        assert_eq!(code(&output), 0, "{name}: {output:?}");
        assert!(output.stderr.is_empty(), "{name}: {output:?}");
        let help = stdout(&output);
        let usage = help.lines().next().unwrap();
        assert!(
            usage == format!("usage: praps {name}")
                || usage.starts_with(&format!("usage: praps {name} ")),
            "{name}: {usage}"
        );
        assert!(help.contains(&summary), "{name}: {help}");
        assert_eq!(
            help.contains("\nArguments:\n"),
            usage != format!("usage: praps {name}"),
            "{name}: {help}"
        );
        let (_, examples) = help.split_once("\nExamples:\n").unwrap();
        assert!(
            examples
                .lines()
                .any(|line| line.trim() == format!("praps {name}")
                    || line.trim().starts_with(&format!("praps {name} "))),
            "{name}: {examples}"
        );
        let marks = help
            .lines()
            .filter(|line| line.starts_with("Idempotent: ") || line.starts_with("Not idempotent: "))
            .count();
        assert_eq!(marks, 1, "{name}: {help}");
        // The usage error for the command prints the same usage line.
        let output = praps(&[&name, "a", "b", "c", "d"], &dir);
        assert_eq!(code(&output), 2, "{name}");
        assert_eq!(stderr(&output).lines().next(), Some(usage), "{name}");
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_help_for_an_unknown_word_suggests() {
    let dir = claim("help-unknown");
    let output = praps(&["help", "statsu"], &dir);
    assert_eq!(code(&output), 2, "{output:?}");
    assert!(output.stdout.is_empty());
    let err = stderr(&output);
    assert!(err.contains("`statsu` is not a command"), "{err}");
    assert!(err.contains("nearest: `status`"), "{err}");
    assert_eq!(err.lines().last(), Some(LIST_HINT));
    let _ = fs::remove_dir_all(&dir);
}

/// `--help` and `-h` are spellings of `help`, and `--version` and `-V` of
/// `version` (INT-0037).
#[test]
fn help_and_version_spellings() {
    let dir = claim("spellings");
    let list = stdout(&praps(&[], &dir));
    for args in [&["--help"][..], &["-h"][..]] {
        let output = praps(args, &dir);
        assert_eq!(code(&output), 0, "{args:?}");
        assert_eq!(stdout(&output), list, "{args:?}");
    }
    let status = stdout(&praps(&["help", "status"], &dir));
    assert!(status.starts_with("usage: praps status"));
    for args in [
        &["status", "--help"][..],
        &["--help", "status"][..],
        &["status", "-h"][..],
    ] {
        let output = praps(args, &dir);
        assert_eq!(code(&output), 0, "{args:?}");
        assert_eq!(stdout(&output), status, "{args:?}");
    }
    let version = praps(&["version"], &dir);
    assert_eq!(code(&version), 0);
    assert_eq!(
        stdout(&version),
        format!("praps {}\n", env!("CARGO_PKG_VERSION"))
    );
    for args in [&["--version"][..], &["-V"][..]] {
        let output = praps(args, &dir);
        assert_eq!(code(&output), 0, "{args:?}");
        assert_eq!(output.stdout, version.stdout, "{args:?}");
    }
    assert!(
        fs::read_dir(&dir).unwrap().next().is_none(),
        "nothing created"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// A corpus of misspelt names, built from every command `praps` lists: each
/// single deletion, each swap of two neighbouring letters, and the name
/// with `x` in front. Each is within two edits of the name it came from, so
/// `praps` names that command, exits 2 and writes nothing to stdout
/// (INT-0037 AC-2).
#[test]
fn unknown_words_get_the_nearest_commands() {
    let dir = claim("misspelt");
    let names: Vec<String> = listed(&dir).into_iter().map(|(name, _)| name).collect();
    let mut tried = 0;
    for name in &names {
        let letters: Vec<char> = name.chars().collect();
        let mut words = vec![format!("x{name}")];
        for at in 0..letters.len() {
            let mut deleted = letters.clone();
            deleted.remove(at);
            words.push(deleted.into_iter().collect());
            if at + 1 < letters.len() {
                let mut swapped = letters.clone();
                swapped.swap(at, at + 1);
                words.push(swapped.into_iter().collect());
            }
        }
        for word in words {
            if word.is_empty() || names.contains(&word) {
                continue;
            }
            tried += 1;
            let output = praps(&[&word], &dir);
            assert_eq!(code(&output), 2, "{word}: {output:?}");
            assert!(output.stdout.is_empty(), "{word}");
            let err = stderr(&output);
            assert!(
                err.starts_with(&format!("praps: `{word}` is not a command\n")),
                "{word}: {err}"
            );
            let nearest = err
                .lines()
                .find(|line| line.starts_with("nearest: "))
                .unwrap_or_else(|| panic!("{word}: no nearest line in {err}"));
            assert!(
                nearest.contains(&format!("`{name}`")),
                "{word} came from {name}: {nearest}"
            );
            assert_eq!(err.lines().last(), Some(LIST_HINT), "{word}");
        }
    }
    assert!(tried > 5 * names.len(), "{tried} words tried");
    // A word near nothing is still answered, without a nearest line.
    let output = praps(&["snapshot"], &dir);
    assert_eq!(code(&output), 2);
    let output = praps(&["frobnicate"], &dir);
    assert_eq!(code(&output), 2);
    assert_eq!(
        stderr(&output),
        format!("praps: `frobnicate` is not a command\n{LIST_HINT}\n")
    );
    assert!(
        fs::read_dir(&dir).unwrap().next().is_none(),
        "nothing created"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// A word another system uses is answered with what to use in `praps`,
/// naming only commands `praps` lists.
#[test]
fn other_systems_words_are_answered() {
    let dir = claim("other-words");
    let names: Vec<String> = listed(&dir).into_iter().map(|(name, _)| name).collect();
    for word in ["commit", "add", "checkout", "stash"] {
        let output = praps(&[word], &dir);
        assert_eq!(code(&output), 2, "{word}: {output:?}");
        assert!(output.stdout.is_empty(), "{word}");
        let err = stderr(&output);
        let answer = err
            .lines()
            .find(|line| line.starts_with(&format!("`{word}`: ")))
            .unwrap_or_else(|| panic!("{word}: no answer in {err}"));
        let named: Vec<&str> = answer
            .split("`praps ")
            .skip(1)
            .map(|rest| rest.split(['`', ' ']).next().unwrap())
            .collect();
        assert!(!named.is_empty(), "{word}: {answer}");
        for name in named {
            assert!(names.iter().any(|n| n == name), "{word}: `{name}`");
        }
        assert_eq!(err.lines().last(), Some(LIST_HINT), "{word}");
    }
    let _ = fs::remove_dir_all(&dir);
}

/// Every command whose help says it is idempotent repeats exactly: run
/// twice in a row in a repository with an edit not yet recorded, the second
/// run prints what the first printed and leaves every file as the first
/// left it (INT-0022 AC-9, for the commands built so far). The commands are
/// taken from `praps help`, not from a list kept here.
#[test]
fn idempotent_commands_repeat_exactly() {
    let dir = claim("repeat");
    let idempotent: Vec<String> = listed(&dir)
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| {
            stdout(&praps(&["help", name], &dir))
                .lines()
                .any(|line| line.starts_with("Idempotent: "))
        })
        .collect();
    assert!(idempotent.len() >= 5, "{idempotent:?}");
    assert!(idempotent.iter().any(|name| name == "status"));
    for name in &idempotent {
        let root = dir.join(format!("repo-{name}"));
        assert_eq!(code(&praps(&["init", root.to_str().unwrap()], &dir)), 0);
        write(&root, "a", b"1");
        assert_eq!(code(&praps(&["new"], &root)), 0);
        write(&root, "a", b"an edit no command has recorded");
        write(&root, "b", b"new");
        let first = praps(&[name], &root);
        let after_first = tree(&root);
        let second = praps(&[name], &root);
        assert_eq!(code(&first), 0, "{name}: {first:?}");
        assert_eq!(code(&second), 0, "{name}: {second:?}");
        assert_eq!(second.stdout, first.stdout, "{name}: stdout");
        assert_eq!(second.stderr, first.stderr, "{name}: stderr");
        assert_eq!(tree(&root), after_first, "{name}: the second run wrote");
    }
    let _ = fs::remove_dir_all(&dir);
}

// --------------------------------------- `verify` replaces `fsck` (Sprint 6 T-007)

/// The old name of the integrity check is refused like any word that is
/// not a command, and the answer names the new one (INT-0037 AC-1).
#[test]
fn fsck_is_answered_with_verify() {
    let repo = repo("old-name");
    let before = tree(&repo.dir);
    for args in [&["fsck"][..], &["fsck", "repo"][..]] {
        let output = praps(args, &repo.dir);
        assert_eq!(code(&output), 2, "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        let err = stderr(&output);
        assert!(err.starts_with("praps: `fsck` is not a command\n"), "{err}");
        assert!(
            err.lines()
                .any(|line| line.starts_with("`fsck`: ") && line.contains("`praps verify`")),
            "{err}"
        );
        assert_eq!(err.lines().last(), Some(LIST_HINT));
    }
    assert_eq!(tree(&repo.dir), before, "nothing was checked or written");
    let names: Vec<String> = listed(&repo.dir)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert!(names.iter().any(|name| name == "verify"));
    assert!(!names.iter().any(|name| name == "fsck"));
}

/// The documents that tell a reader what to type name only commands `praps`
/// has: every code span that starts `praps WORD` names a command in the
/// list, and the word `fsck` appears only as the store's own function,
/// `Store::fsck`, or in a file's name.
#[test]
fn docs_name_only_commands_praps_has() {
    let dir = claim("docs");
    let names: Vec<String> = listed(&dir).into_iter().map(|(name, _)| name).collect();
    // Commands a document may name as work to come. None is named yet.
    const TO_COME: [&str; 0] = [];
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for doc in [
        "README.md",
        "docs/spec/working-copy.md",
        "docs/spec/storage-durability.md",
    ] {
        let text = fs::read_to_string(root.join(doc)).unwrap();
        let mut spans = 0;
        for (at, _) in text.match_indices("`praps ") {
            let word: String = text[at + "`praps ".len()..]
                .chars()
                .take_while(char::is_ascii_lowercase)
                .collect();
            spans += 1;
            assert!(
                names.contains(&word) || TO_COME.contains(&word.as_str()),
                "{doc}: `praps {word}` is not a command `praps` lists"
            );
        }
        assert!(spans > 0, "{doc} names no command");
        for (number, line) in text.lines().enumerate() {
            for (at, _) in line.match_indices("fsck") {
                let before = &line[..at];
                let after = &line[at + "fsck".len()..];
                assert!(
                    before.ends_with("Store::") || after.starts_with(".rs"),
                    "{doc}:{}: `fsck` names neither `Store::fsck` nor a file: {line}",
                    number + 1
                );
            }
        }
    }
    let _ = fs::remove_dir_all(&dir);
}

// --------------------------- finding the working copy (Sprint 6 T-008)

/// Copies the directory `from` to `to`, byte for byte.
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// Every file under `top` by its path from `top`, with its content.
fn files_under(top: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    tree(top)
        .into_iter()
        .map(|(path, content)| (path.strip_prefix(top).unwrap().to_path_buf(), content))
        .collect()
}

/// The tests make their directories under the system's temporary
/// directory, and several of them rely on no directory above those being a
/// working copy. A stray repository there fails this test by name.
#[test]
fn no_repository_above_the_test_directories() {
    let dir = claim("above");
    for above in dir.ancestors() {
        assert!(
            !above.join(".praps").join("store").is_dir(),
            "{} is a Periapsis repository above the tests' directories",
            above.display()
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

/// `help` and `version` look at no directory, so they need no repository.
#[test]
fn help_and_version_need_no_repository() {
    let dir = claim("no-repo");
    for args in [
        &[][..],
        &["help"][..],
        &["help", "status"][..],
        &["version"][..],
    ] {
        let output = praps(args, &dir);
        assert_eq!(code(&output), 0, "{args:?}: {output:?}");
        assert!(output.stderr.is_empty(), "{args:?}");
    }
    let _ = fs::remove_dir_all(&dir);
}

/// A repository with some history and directories two levels deep, used as
/// the template the next tests copy. Returns its top.
fn template(dir: &Path) -> PathBuf {
    let top = dir.join("template");
    assert_eq!(code(&praps(&["init", "template"], dir)), 0);
    write(&top, "a", b"1");
    write(&top, "d1/d2/deep", b"deep");
    assert_eq!(code(&praps(&["status"], &top)), 0);
    assert_eq!(code(&praps(&["new"], &top)), 0);
    write(&top, "a", b"2");
    assert_eq!(code(&praps(&["status"], &top)), 0);
    top
}

/// A command that opens a working copy does the same thing wherever inside
/// it the command is run: at the top with no directory, from two levels
/// down, and from outside with a directory below the top given. Its
/// stdout, its exit code, its stderr (once each copy's own top path is
/// replaced by a marker) and the files it leaves are equal (INT-0037 AC-3).
#[test]
fn commands_act_the_same_from_below() {
    let dir = claim("below");
    let template = template(&dir);
    let first_op = logged_ops(&template).last().unwrap().clone();
    type Edit = fn(&Path);
    let corpus: Vec<(&str, Vec<String>, Edit)> = vec![
        ("status with an edit", vec!["status".into()], |top| {
            write(top, "d1/new", b"n")
        }),
        ("status with none", vec!["status".into()], |_| {}),
        ("log", vec!["log".into()], |top| write(top, "a", b"3")),
        ("ops", vec!["ops".into()], |top| write(top, "a", b"3")),
        (
            "describe",
            vec!["describe".into(), "Change a\n\nAnd say why.".into()],
            |top| write(top, "a", b"3"),
        ),
        (
            "describe, the text it has",
            vec!["describe".into(), String::new()],
            |_| {},
        ),
        ("undo", vec!["undo".into()], |top| write(top, "a", b"3")),
        ("restore", vec!["restore".into(), first_op.clone()], |_| {}),
        ("verify", vec!["verify".into()], |_| {}),
        ("a refusal", vec!["restore".into(), "0000".into()], |_| {}),
        (
            "a usage error",
            vec!["status".into(), "--short".into()],
            |_| {},
        ),
    ];
    for (n, (name, args, edit)) in corpus.iter().enumerate() {
        let mut results = Vec::new();
        for (place, sub, given) in [
            ("at the top", "", None),
            ("from two levels down", "d1/d2", None),
            ("with a directory below given", "..", Some("d1")),
        ] {
            let top = dir.join(format!("copy-{n}-{}", results.len()));
            copy_tree(&template, &top);
            edit(&top);
            // `..` stands for "outside": the copy's parent, with the
            // directory given by its path from there.
            let (cwd, given) = match given {
                Some(below) => (
                    dir.clone(),
                    Some(top.join(below).to_str().unwrap().to_owned()),
                ),
                None if sub.is_empty() => (top.clone(), None),
                None => (top.join(sub), None),
            };
            let mut line: Vec<&str> = args.iter().map(String::as_str).collect();
            if let Some(given) = &given {
                line.push(given);
            }
            let output = praps(&line, &cwd);
            // The top's path as the test spells it, and as the system does
            // (where a temporary directory has a short and a long name).
            let spelled = top.display().to_string();
            let resolved = fs::canonicalize(&top).unwrap().display().to_string();
            let resolved = resolved.trim_start_matches(r"\\?\").to_owned();
            let marked = |text: &[u8]| {
                String::from_utf8_lossy(text)
                    .replace(&spelled, "<top>")
                    .replace(&resolved, "<top>")
            };
            results.push((
                place,
                code(&output),
                marked(&output.stdout),
                marked(&output.stderr),
                files_under(&top),
            ));
        }
        let (_, code0, out0, err0, files0) = &results[0];
        for (place, code, out, err, files) in &results[1..] {
            assert_eq!(code, code0, "{name}, {place}: exit code");
            assert_eq!(out, out0, "{name}, {place}: stdout");
            assert_eq!(err, err0, "{name}, {place}: stderr");
            assert_eq!(files, files0, "{name}, {place}: the files left");
        }
    }

    // `new` draws a random change ID: the ID is masked, and the state
    // compared is the number of ops, the files outside `.praps` and what
    // `status` then prints.
    let mut results = Vec::new();
    for sub in ["", "d1/d2"] {
        let top = dir.join(format!("copy-new-{}", results.len()));
        copy_tree(&template, &top);
        write(&top, "a", b"3");
        let output = praps(&["new"], &top.join(sub));
        assert_eq!(code(&output), 0, "{output:?}");
        let line = stdout(&output);
        let id = line.trim().strip_prefix("new change ").unwrap();
        assert_eq!(id.len(), 32, "{line}");
        let outside: Vec<_> = files_under(&top)
            .into_iter()
            .filter(|(path, _)| !path.starts_with(".praps"))
            .collect();
        results.push((
            logged_ops(&top).len(),
            outside,
            stdout(&praps(&["status"], &top)),
        ));
    }
    assert_eq!(results[0], results[1], "new: from the top and from below");
    let _ = fs::remove_dir_all(&dir);
}

/// A refusal names the working copy by the directory given, when one was
/// given and it is the top, and by the top's full path otherwise.
#[test]
fn refusals_name_the_working_copy() {
    let dir = claim("naming");
    let top = template(&dir);
    let full = top.display().to_string();
    let refused = |args: &[&str], cwd: &Path| {
        let output = praps(args, cwd);
        assert_eq!(code(&output), 1, "{args:?}: {output:?}");
        stderr(&output)
    };
    // The top given as a directory: named as given.
    let err = refused(&["restore", "0000", "template"], &dir);
    assert!(err.starts_with("praps restore: template: "), "{err}");
    // No directory given, at the top and from below: the top's full path.
    for cwd in [top.clone(), top.join("d1"), top.join("d1/d2")] {
        let err = refused(&["restore", "0000"], &cwd);
        assert!(
            err.starts_with(&format!("praps restore: {full}: ")),
            "from {}: {err}",
            cwd.display()
        );
    }
    // A directory below the top given: the top's full path too.
    let err = refused(&["restore", "0000", "template/d1"], &dir);
    assert!(
        err.starts_with(&format!("praps restore: {full}: ")),
        "{err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// A directory that is not there, or is not a directory, is refused by the
/// name given, and the working copy above it is not acted on.
#[test]
fn a_missing_directory_is_not_searched_above() {
    let dir = claim("missing");
    let top = template(&dir);
    write(&top, "a-file", b"a file, not a directory");
    write(&top, "a", b"an edit that must not be recorded");
    let before = tree(&top);
    for (given, why) in [
        ("nowhere", "no such directory"),
        ("a-file", "not a directory"),
    ] {
        // Every command that opens a working copy, with the directory in
        // the place each takes it; and `verify`, which refuses it too.
        for args in [
            &["status", given][..],
            &["new", given][..],
            &["log", given][..],
            &["ops", given][..],
            &["undo", given][..],
            &["restore", "0000", given][..],
            &["describe", "text", given][..],
            &["verify", given][..],
        ] {
            let command = args[0];
            let output = praps(args, &top);
            assert_eq!(code(&output), 1, "{command} {given}: {output:?}");
            assert!(output.stdout.is_empty(), "{command} {given}");
            let err = stderr(&output);
            assert!(
                err.starts_with(&format!("praps {command}: {given}: {why}")),
                "{command} {given}: {err}"
            );
        }
    }
    assert_eq!(tree(&top), before, "nothing was recorded or written");
    let _ = fs::remove_dir_all(&dir);
}

/// A shell whose current directory has been removed: the command says the
/// current directory cannot be read, and exits 1.
#[cfg(unix)]
#[test]
fn a_removed_current_directory_is_reported() {
    let dir = claim("gone");
    let script = format!(
        "mkdir gone && cd gone && rmdir ../gone && exec '{}' status",
        env!("CARGO_BIN_EXE_praps")
    );
    let output = Command::new("sh")
        .args(["-c", &script])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(output.stdout.is_empty());
    let err = stderr(&output);
    assert!(
        err.starts_with("praps status: the current directory cannot be read"),
        "{err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// `init` never makes or completes a repository inside a working copy,
/// whatever the directory it is given holds; at the top it still says the
/// repository is there (INT-0037 AC-3).
#[test]
fn init_inside_a_working_copy_is_refused() {
    let dir = claim("nested");
    let top = template(&dir);
    // A directory with a half-made store, and a whole repository, both
    // inside the working copy. Each was made outside and moved in.
    let half = dir.join("half");
    fs::create_dir_all(half.join(".praps/store/objects")).unwrap();
    fs::create_dir_all(half.join(".praps/store/ops")).unwrap();
    fs::rename(&half, top.join("half")).unwrap();
    assert_eq!(code(&praps(&["init", "whole"], &dir)), 0);
    fs::rename(dir.join("whole"), top.join("whole")).unwrap();
    let before = tree(&top);
    let full = top.display().to_string();

    let cases: [(&[&str], PathBuf); 6] = [
        (&["init"], top.join("d1")),
        (&["init"], top.join("d1/d2")),
        (&["init", "d1/new"], top.clone()),
        (&["init", "template/d1"], dir.clone()),
        (&["init"], top.join("half")),
        (&["init"], top.join("whole")),
    ];
    for (args, cwd) in cases {
        let output = praps(args, &cwd);
        assert_eq!(
            code(&output),
            1,
            "{args:?} in {}: {output:?}",
            cwd.display()
        );
        assert!(output.stdout.is_empty(), "{args:?}");
        let err = stderr(&output);
        assert!(err.starts_with("praps init: "), "{err}");
        assert!(
            err.contains(&format!("inside the working copy at {full}")),
            "{args:?} in {}: {err}",
            cwd.display()
        );
    }
    assert_eq!(tree(&top), before, "nothing was written");
    // At the top itself, `init` answers as before and writes nothing.
    let output = praps(&["init"], &top);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(stdout(&output).contains("already a Periapsis repository"));
    assert_eq!(tree(&top), before);
    let _ = fs::remove_dir_all(&dir);
}

/// Outside any working copy, `init` completes a repository whose `init`
/// was interrupted: one left with an empty `.praps`, and one left with a
/// store that holds no operation.
#[test]
fn praps_init_completes_an_interrupted_init() {
    let dir = claim("interrupted");
    fs::create_dir_all(dir.join("empty/.praps")).unwrap();
    fs::create_dir_all(dir.join("no-op/.praps/store/objects")).unwrap();
    fs::create_dir_all(dir.join("no-op/.praps/store/ops")).unwrap();
    for name in ["empty", "no-op"] {
        let root = dir.join(name);
        // Before: no command can use it.
        assert_eq!(code(&praps(&["status"], &root)), 1, "{name}");
        let output = praps(&["init", name], &dir);
        assert_eq!(code(&output), 0, "{name}: {output:?}");
        assert!(
            stdout(&output).starts_with("initialised a Periapsis repository"),
            "{name}: {output:?}"
        );
        assert_eq!(
            fs::read_dir(root.join(".praps/store/ops")).unwrap().count(),
            1
        );
        write(&root, "f", b"x");
        assert_eq!(lines(&praps(&["status"], &root)), ["A f"], "{name}");
    }
    let _ = fs::remove_dir_all(&dir);
}

/// The store `verify` checks, by the four rules in order, and a directory
/// that is not there.
#[test]
fn verify_chooses_its_store_in_order() {
    let dir = claim("verify-order");
    let top = template(&dir);
    let flip = |file: &Path| {
        let mut bytes = fs::read(file).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        fs::write(file, bytes).unwrap();
    };
    let first_file = |objects: &Path| {
        fs::read_dir(objects)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
    };

    // A directory that is not there, inside a working copy: refused by
    // name; the working copy is not checked.
    let output = praps(&["verify", "nowhere"], &top);
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        stderr(&output).starts_with("praps verify: nowhere: no such directory"),
        "{output:?}"
    );

    // Rule 3: a directory inside the working copy, and one that holds a
    // `.praps` with no store, each check the working copy's store.
    fs::create_dir_all(top.join("sub/.praps")).unwrap();
    fs::write(top.join("sub/.praps/x"), b"x").unwrap();
    for cwd in [top.join("d1"), top.join("d1/d2"), top.join("sub")] {
        let output = praps(&["verify"], &cwd);
        assert_eq!(code(&output), 0, "from {}: {output:?}", cwd.display());
        assert_eq!(stdout(&output), "clean\n");
    }

    // Rule 2: a bare store inside the working copy, damaged: the check
    // given that directory reports the damage, and the working copy's own
    // store is still clean.
    let bare = Store::init(RealFs, &top.join("d1/store"), HashAlg::DEFAULT, 3).and_then(|mut s| {
        s.put_object(&blob(b"content"))?;
        Ok(())
    });
    bare.unwrap();
    flip(&first_file(&top.join("d1/store/objects")));
    for (args, cwd) in [
        (&["verify", "d1/store"][..], top.clone()),
        (&["verify"][..], top.join("d1/store")),
    ] {
        let output = praps(args, &cwd);
        assert_eq!(code(&output), 1, "{args:?}: {output:?}");
        assert!(
            stdout(&output).contains("does not hash to its name"),
            "{args:?}: {output:?}"
        );
    }
    assert_eq!(stdout(&praps(&["verify"], &top.join("d1"))), "clean\n");

    // Rule 1: a top that also holds ordinary directories named `objects`
    // and `ops`: its `.praps/store` is the store checked.
    fs::create_dir_all(top.join("objects")).unwrap();
    fs::create_dir_all(top.join("ops")).unwrap();
    fs::write(top.join("objects/not-an-object"), b"x").unwrap();
    let output = praps(&["verify"], &top);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(stdout(&output), "clean\n");
    // And damage in that store is what it reports.
    flip(&first_file(&top.join(".praps/store/objects")));
    let output = praps(&["verify", "template"], &dir);
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(stdout(&output).contains("does not hash to its name"));

    // Rule 4: a directory in no working copy that is not a store.
    let plain = dir.join("plain");
    fs::create_dir(&plain).unwrap();
    let output = praps(&["verify"], &plain);
    assert_eq!(code(&output), 1, "{output:?}");
    assert!(
        stderr(&output).contains("is not a Periapsis store"),
        "{output:?}"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// An entry named `.praps` below the top that is a file, or a directory
/// with no `store` in it, is tracked like any other entry, and a command
/// run beside it acts on the working copy above. On Linux the file is the
/// case where a name on the way to `.praps/store` is not a directory.
#[test]
fn a_praps_entry_without_a_store_is_ordinary() {
    let dir = claim("ordinary");
    let top = dir.join("top");
    assert_eq!(code(&praps(&["init", "top"], &dir)), 0);
    write(&top, "sub/.praps/x", b"inside a directory named .praps");
    write(&top, "other/.praps", b"a file named .praps");
    let expected = ["A other/.praps", "A sub/.praps/x"];
    for cwd in [top.join("sub"), top.join("other"), top.clone()] {
        let output = praps(&["status"], &cwd);
        assert_eq!(code(&output), 0, "from {}: {output:?}", cwd.display());
        assert_eq!(lines(&output), expected, "from {}", cwd.display());
    }
    // `init` beside them is still inside the working copy.
    for cwd in [top.join("sub"), top.join("other")] {
        let output = praps(&["init"], &cwd);
        assert_eq!(code(&output), 1, "{output:?}");
        assert!(stderr(&output).contains("inside the working copy at"));
    }
    let _ = fs::remove_dir_all(&dir);
}

// ------------------- `log` lists changes, `ops` lists operations (Sprint 6 T-009)

/// `log` lists the working-copy change and the changes before it, newest
/// first, one line each: 12 hex digits of the change ID, the number of
/// paths the change changes, and `(no description)` for a change without
/// one. It records the working copy first.
#[test]
fn praps_log_lists_changes_newest_first() {
    let dir = claim("log");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "a", b"1");
    write(&root, "b", b"1");
    let first = stdout(&praps(&["new", "repo"], &dir));
    write(&root, "a", b"2");
    let second = stdout(&praps(&["new", "repo"], &dir));
    // An edit no command has recorded: `log` records it and counts it.
    fs::remove_file(root.join("b")).unwrap();
    let ops_before = logged_ops(&root).len();
    write(&root, "c", b"new");
    let output = praps(&["log", "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let listed = lines(&output);
    let id = |line: &str| line.trim().strip_prefix("new change ").unwrap()[..12].to_owned();
    assert_eq!(
        listed,
        [
            format!("{} 2 paths (no description)", id(&second)),
            format!("{} 1 path (no description)", id(&first)),
            // The root change's ID is not printed by any command before
            // this one, so only its shape is checked below.
            listed[2].clone(),
        ]
    );
    let root_line: Vec<&str> = listed[2].splitn(4, ' ').collect();
    assert_eq!(root_line[1..], ["2", "paths", "(no description)"]);
    assert!(root_line[0].len() == 12 && root_line[0].bytes().all(|b| b.is_ascii_hexdigit()));
    // `logged_ops` recorded `b`'s removal; `log` then recorded `c`.
    assert_eq!(logged_ops(&root).len(), ops_before + 1);
    // From the top with no directory it prints the same.
    assert_eq!(lines(&praps(&["log"], &root)), listed);
    let _ = fs::remove_dir_all(&dir);
}

// ------------------------------------------------------ `describe` (Sprint 6 T-010)

/// `praps` with arguments that need not be text.
fn praps_os(args: &[&std::ffi::OsStr], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_praps"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

/// The working-copy changeset the single op head names, read from the store.
fn working_copy_change(root: &Path) -> codec::Changeset {
    let store = Store::open(RealFs, &root.join(".praps/store"), HashAlg::DEFAULT, 1).unwrap();
    let ops = store.ops().unwrap();
    let heads = store::heads_of(&ops);
    assert_eq!(heads.len(), 1);
    let view =
        View::from_envelope(&store.get_object(&ops[heads.first().unwrap()].view).unwrap()).unwrap();
    let wc = view.workspaces["default"].working_copy.first().unwrap();
    codec::Changeset::from_envelope(&store.get_object(wc).unwrap()).unwrap()
}

/// `praps describe TEXT` records the working copy, then leaves the current
/// change with TEXT's bytes as its description and the same change ID,
/// parents and changes. It says so in one line that names the change and
/// shows the first line of the text. A later edit keeps the description,
/// and `new` starts a change with none.
#[test]
fn praps_describe_sets_and_reports() {
    let dir = claim("describe");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "a", b"1");
    write(&root, "sub/b", b"2");
    let text = "Add a and b\n\nThe second line is not shown.\n";
    let output = praps(&["describe", text, "repo"], &dir);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let change = working_copy_change(&root);
    let id = hash::hex(&change.change_id)[..12].to_owned();
    assert_eq!(
        stdout(&output),
        format!("described change {id}: Add a and b\n")
    );
    assert_eq!(change.description, text.as_bytes());
    assert_eq!(change.changes.len(), 2, "the edit was recorded first");
    assert!(change.parents.is_empty());
    // The root op, the recording of the edit, and the description.
    assert_eq!(logged_ops(&root).len(), 3);
    assert_eq!(
        lines(&praps(&["log"], &root)),
        [format!("{id} 2 paths Add a and b")]
    );
    // No working-copy file was touched.
    assert_eq!(fs::read(root.join("a")).unwrap(), b"1");
    assert_eq!(fs::read(root.join("sub/b")).unwrap(), b"2");

    // A later edit, recorded by another command, keeps the description.
    write(&root, "a", b"edited");
    assert_eq!(lines(&praps(&["status"], &root)), ["A a", "A sub/b"]);
    let edited = working_copy_change(&root);
    assert_eq!(edited.description, text.as_bytes());
    assert_eq!(edited.change_id, change.change_id);

    // `new` starts a change with none; the one before keeps its own.
    assert_eq!(code(&praps(&["new"], &root)), 0);
    let log = lines(&praps(&["log"], &root));
    assert_eq!(log.len(), 2, "{log:?}");
    assert!(log[0].ends_with(" 0 paths (no description)"), "{log:?}");
    assert_eq!(log[1], format!("{id} 2 paths Add a and b"));

    // From a directory below, with no directory given.
    let output = praps(&["describe", "Next"], &root.join("sub"));
    assert_eq!(code(&output), 0, "{output:?}");
    let next = working_copy_change(&root);
    assert_eq!(next.description, b"Next");
    assert_eq!(
        stdout(&output),
        format!(
            "described change {}: Next\n",
            &hash::hex(&next.change_id)[..12]
        )
    );
    assert_eq!(code(&praps(&["verify"], &root)), 0);
    let _ = fs::remove_dir_all(&dir);
}

/// With the description the change already has, `describe` says that
/// nothing changed, exits 0 and writes nothing. An empty text on a change
/// with no description is the same case, and an empty text on a described
/// change removes the description.
#[test]
fn praps_describe_says_when_nothing_changed() {
    let dir = claim("describe-same");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let id = hash::hex(&working_copy_change(&root).change_id)[..12].to_owned();
    let unchanged = format!("nothing changed: change {id} already has that description\n");

    let before = tree(&root);
    let output = praps(&["describe", ""], &root);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(stdout(&output), unchanged);
    assert_eq!(tree(&root), before, "nothing was written");

    let output = praps(&["describe", "Start"], &root);
    assert_eq!(stdout(&output), format!("described change {id}: Start\n"));
    let before = tree(&root);
    let ops = logged_ops(&root).len();
    for _ in 0..2 {
        let output = praps(&["describe", "Start"], &root);
        assert_eq!(code(&output), 0, "{output:?}");
        assert_eq!(stdout(&output), unchanged);
        assert!(output.stderr.is_empty());
    }
    assert_eq!(tree(&root), before, "nothing was written");
    assert_eq!(logged_ops(&root).len(), ops);

    // An edit made before it is still recorded, in an op of its own.
    write(&root, "a", b"1");
    let output = praps(&["describe", "Start"], &root);
    assert_eq!(stdout(&output), unchanged);
    assert_eq!(logged_ops(&root).len(), ops + 1);
    assert_eq!(working_copy_change(&root).changes.len(), 1);

    // An empty text removes the description.
    let output = praps(&["describe", ""], &root);
    assert_eq!(
        stdout(&output),
        format!("described change {id}: (no description)\n")
    );
    assert!(working_copy_change(&root).description.is_empty());
    let _ = fs::remove_dir_all(&dir);
}

/// The first line of a description is printed escaped, as names are, by
/// `describe` and by `log`: a control character or a byte that is not UTF-8
/// cannot break the line or reach the terminal, and the bytes stored are
/// the bytes given (INT-0034).
#[test]
fn praps_log_escapes_the_first_line() {
    let dir = claim("describe-escape");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let id = hash::hex(&working_copy_change(&root).change_id)[..12].to_owned();
    let text = "tab\there \u{1b}[31mred back\\slash caf\u{e9}\r\nsecond line";
    let shown = "tab\\u{9}here \\u{1b}[31mred back\\\\slash caf\u{e9}";
    let output = praps(&["describe", text], &root);
    assert_eq!(code(&output), 0, "{output:?}");
    assert_eq!(stdout(&output), format!("described change {id}: {shown}\n"));
    assert_eq!(working_copy_change(&root).description, text.as_bytes());
    let log = praps(&["log"], &root);
    assert_eq!(stdout(&log), format!("{id} 0 paths {shown}\n"));
    assert!(
        !log.stdout.contains(&0x1b),
        "an escape reached the terminal"
    );

    // Bytes that are not UTF-8, which Linux passes as they are.
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let raw = b"caf\xc3\xa9 \xff\xfe end";
        let output = praps_os(
            &["describe".as_ref(), std::ffi::OsStr::from_bytes(raw)],
            &root,
        );
        assert_eq!(code(&output), 0, "{output:?}");
        let shown = "caf\u{e9} \\xff\\xfe end";
        assert_eq!(stdout(&output), format!("described change {id}: {shown}\n"));
        assert_eq!(working_copy_change(&root).description, raw);
        assert_eq!(
            stdout(&praps(&["log"], &root)),
            format!("{id} 0 paths {shown}\n")
        );
    }
    // Text with no UTF-8 form, which only Windows can pass, is refused as a
    // usage error, and nothing is written.
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let raw = std::ffi::OsString::from_wide(&[u16::from(b'a'), 0xd800, u16::from(b'b')]);
        let before = tree(&root);
        let output = praps_os(&["describe".as_ref(), raw.as_os_str()], &root);
        assert_eq!(code(&output), 2, "{output:?}");
        assert!(output.stdout.is_empty());
        let err = stderr(&output);
        assert_eq!(err.lines().next(), Some("usage: praps describe TEXT [DIR]"));
        assert!(err.lines().last().unwrap().starts_with("hint: "), "{err}");
        assert!(err.contains("Unicode"), "{err}");
        assert_eq!(tree(&root), before);
    }
    let _ = fs::remove_dir_all(&dir);
}

// ------------------------- every refusal names the next step (Sprint 6 T-011)

/// The commands a hint names: each word that follows `` `praps `` in it.
fn named_in(hint: &str) -> Vec<String> {
    hint.match_indices("`praps ")
        .map(|(at, found)| {
            hint[at + found.len()..]
                .chars()
                .take_while(char::is_ascii_lowercase)
                .collect()
        })
        .collect()
}

/// Every refusal ends with exactly one line that starts `hint:`, after what
/// was wrong, and a command the hint names is one `praps` lists (INT-0037
/// AC-4). The corpus is the refusals these tests can make happen through
/// the binary: usage errors, words that are not commands, ways of not
/// finding a working copy, every error of the `workspace` crate, and ways
/// `verify` fails. Some are reached on one system only: a hazard and text
/// with no UTF-8 form on Windows; a removed current directory, and the four
/// that need a directory closed by its mode, on Unix. One kind is not
/// reached on either: a top that cannot be made the current directory,
/// which nothing here can close without closing its store too.
#[test]
fn every_refusal_prints_one_hint() {
    let dir = claim("hints");
    let commands: Vec<String> = listed(&dir).into_iter().map(|(name, _)| name).collect();
    let top = template(&dir);
    write(&top, "a-file", b"a file, not a directory");
    assert_eq!(code(&praps(&["status"], &top)), 0);
    let mut kinds = 0;
    // Runs one command line and checks its refusal. `holds` is a part of
    // the hint that only the right hint for the case has.
    let mut refused = |what: &str, output: Output, exit: i32, holds: &str| {
        assert_eq!(code(&output), exit, "{what}: {output:?}");
        let err = stderr(&output);
        let hints: Vec<&str> = err
            .lines()
            .filter(|line| line.starts_with("hint: "))
            .collect();
        assert_eq!(hints.len(), 1, "{what}: one hint line in {err:?}");
        let hint = hints[0];
        assert_eq!(err.lines().last(), Some(hint), "{what}: the hint is last");
        assert!(hint.contains(holds), "{what}: {hint}");
        // What was wrong comes first: on stderr, or for `verify`'s report
        // on stdout.
        assert!(
            err.lines().count() >= 2 || !output.stdout.is_empty(),
            "{what}: nothing says what was wrong: {err:?}"
        );
        for name in named_in(hint) {
            assert!(commands.contains(&name), "{what}: `{name}` in {hint}");
        }
        assert!(!hint.ends_with('.'), "{what}: {hint}");
        kinds += 1;
    };

    // Usage errors and words that are not commands: exit 2.
    refused(
        "too many arguments",
        praps(&["status", "a", "b"], &top),
        2,
        "`praps help status`",
    );
    refused(
        "a flag the command does not take",
        praps(&["status", "--short"], &top),
        2,
        "`praps help status`",
    );
    refused(
        "a flag and no command",
        praps(&["--force"], &top),
        2,
        "`praps help` to list",
    );
    refused(
        "a mistyped word",
        praps(&["statsu"], &top),
        2,
        "`praps help` to list",
    );
    refused(
        "another system's word",
        praps(&["commit"], &top),
        2,
        "`praps help` to list",
    );
    refused(
        "text given as several words",
        praps(&["describe", "Add", "the", "parser"], &top),
        2,
        "`praps describe \"Add the parser\"`",
    );
    #[cfg(unix)]
    let not_text = {
        use std::os::unix::ffi::OsStrExt;
        std::ffi::OsStr::from_bytes(b"\xff\xfe").to_owned()
    };
    #[cfg(windows)]
    let not_text = {
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(&[0xd800])
    };
    refused(
        "an operation id that is not text",
        praps_os(&["restore".as_ref(), not_text.as_os_str()], &top),
        2,
        "`praps ops`",
    );
    #[cfg(windows)]
    refused(
        "text with no UTF-8 form",
        praps_os(&["describe".as_ref(), not_text.as_os_str()], &top),
        2,
        "Unicode",
    );

    // Not finding the working copy: exit 1.
    let plain = dir.join("plain");
    fs::create_dir(&plain).unwrap();
    refused(
        "no repository here or above",
        praps(&["status"], &plain),
        1,
        "`praps init`",
    );
    refused(
        "a directory that is not there",
        praps(&["status", "nowhere"], &top),
        1,
        "give a directory that exists",
    );
    refused(
        "a file given as the directory",
        praps(&["log", "a-file"], &top),
        1,
        "give a directory that exists",
    );
    refused(
        "text given as two words",
        praps(&["describe", "Add", "parser"], &top),
        1,
        "`praps describe \"Add parser\"`",
    );
    refused(
        "init inside a working copy",
        praps(&["init"], &top.join("d1")),
        1,
        "`praps status`",
    );

    // A shell whose current directory has been removed.
    #[cfg(unix)]
    refused(
        "a current directory that cannot be read",
        Command::new("sh")
            .args([
                "-c",
                &format!(
                    "mkdir gone && cd gone && rmdir ../gone && exec '{}' status",
                    env!("CARGO_BIN_EXE_praps")
                ),
            ])
            .current_dir(&plain)
            .output()
            .unwrap(),
        1,
        "change to a directory that exists",
    );

    // The errors of the `workspace` crate: exit 1.
    assert_eq!(code(&praps(&["init", "fresh"], &dir)), 0);
    refused(
        "nothing to undo",
        praps(&["undo", "fresh"], &dir),
        1,
        "`praps ops`",
    );
    refused(
        "an unknown operation",
        praps(&["restore", "0000"], &top),
        1,
        "`praps ops` to list the operations and their ids",
    );
    // A prefix two operations share: with more than 16 operations, some
    // first digit is shared.
    let many = dir.join("many");
    assert_eq!(code(&praps(&["init", "many"], &dir)), 0);
    for round in 0..17u8 {
        write(&many, "a", &[b'a' + round]);
        assert_eq!(code(&praps(&["status"], &many)), 0);
    }
    let ops = logged_ops(&many);
    let shared = "0123456789abcdef"
        .chars()
        .map(String::from)
        .find(|digit| {
            ops.iter()
                .filter(|op| op.starts_with(digit.as_str()))
                .count()
                >= 2
        })
        .unwrap();
    refused(
        "an ambiguous operation",
        praps(&["restore", &shared], &many),
        1,
        "give more digits",
    );
    // A store that holds no operation: `init` was interrupted.
    fs::create_dir_all(dir.join("no-op/.praps/store/objects")).unwrap();
    fs::create_dir_all(dir.join("no-op/.praps/store/ops")).unwrap();
    refused(
        "a repository whose init never finished",
        praps(&["status"], &dir.join("no-op")),
        1,
        "`praps init`",
    );
    // Two op heads.
    let heads = dir.join("heads");
    assert_eq!(code(&praps(&["init", "heads"], &dir)), 0);
    {
        let mut store =
            Store::open(RealFs, &heads.join(".praps/store"), HashAlg::DEFAULT, 5).unwrap();
        let head = *store.heads().unwrap().first().unwrap();
        let view = store.ops().unwrap()[&head].view;
        let mut other = View::from_envelope(&store.get_object(&view).unwrap()).unwrap();
        other
            .workspaces
            .insert("other".to_owned(), other.workspaces["default"].clone());
        let other = store.put_object(&other.to_envelope()).unwrap();
        for view in [view, other] {
            store
                .commit_op(&Operation {
                    parents: [head].into(),
                    workspace: "default".to_owned(),
                    view,
                })
                .unwrap();
        }
    }
    // Two op heads no longer stop a command (INT-0010).
    assert_eq!(code(&praps(&["status"], &heads)), 0);
    // An op file that does not hash to its name.
    let corrupt = dir.join("corrupt");
    assert_eq!(code(&praps(&["init", "corrupt"], &dir)), 0);
    let op_file = fs::read_dir(corrupt.join(".praps/store/ops"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut bytes = fs::read(&op_file).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    fs::write(&op_file, bytes).unwrap();
    refused(
        "a damaged op file",
        praps(&["status"], &corrupt),
        1,
        "`praps verify` to list what is damaged",
    );
    // An object the head needs is gone: the filesystem's own error.
    let gone = dir.join("gone");
    assert_eq!(code(&praps(&["init", "gone"], &dir)), 0);
    {
        let store = Store::open(RealFs, &gone.join(".praps/store"), HashAlg::DEFAULT, 5).unwrap();
        let head = *store.heads().unwrap().first().unwrap();
        let view = store.ops().unwrap()[&head].view;
        fs::remove_file(gone.join(".praps/store/objects").join(file_name(&view))).unwrap();
    }
    refused(
        "a failure of the filesystem",
        praps(&["status"], &gone),
        1,
        "`praps verify` to check what is stored",
    );
    // A working-copy change with two parents, written through the store.
    let merged = dir.join("merged");
    assert_eq!(code(&praps(&["init", "merged"], &dir)), 0);
    write(&merged, "a", b"1");
    assert_eq!(code(&praps(&["new"], &merged)), 0);
    {
        let mut store =
            Store::open(RealFs, &merged.join(".praps/store"), HashAlg::DEFAULT, 5).unwrap();
        let head = *store.heads().unwrap().first().unwrap();
        let view =
            View::from_envelope(&store.get_object(&store.ops().unwrap()[&head].view).unwrap())
                .unwrap();
        let second = *view.workspaces["default"].working_copy.first().unwrap();
        let first = codec::Changeset::from_envelope(&store.get_object(&second).unwrap())
            .unwrap()
            .parents[0];
        let both = codec::Changeset {
            parents: vec![second, first],
            change_id: [9; 16],
            changes: Default::default(),
            description: Vec::new(),
        };
        let both = store.put_object(&both.to_envelope()).unwrap();
        let view = View {
            workspaces: [(
                "default".to_owned(),
                codec::WorkspaceEntry {
                    working_copy: [both].into(),
                    heads: [both].into(),
                },
            )]
            .into(),
        };
        let view = store.put_object(&view.to_envelope()).unwrap();
        store
            .commit_op(&Operation {
                parents: [head].into(),
                workspace: "default".to_owned(),
                view,
            })
            .unwrap();
    }
    refused(
        "a change with several parents",
        praps(&["status"], &merged),
        1,
        "this build cannot read a change that has several parents yet; nothing was lost",
    );

    // Refusals that need a directory the user may not read or write. They
    // are made with file modes, so on Unix only, and not as root, for whom
    // a mode closes nothing.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = |path: &Path, mode: u32| {
            fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
        };
        let locked = dir.join("locked");
        assert_eq!(code(&praps(&["init", "locked"], &dir)), 0);
        fs::create_dir(locked.join("in")).unwrap();
        fs::create_dir(dir.join("closed")).unwrap();
        fs::create_dir_all(top.join("bak/objects")).unwrap();
        fs::create_dir_all(top.join("bak/ops")).unwrap();
        mode(&locked.join(".praps/store"), 0o000);
        mode(&dir.join("closed"), 0o555);
        mode(&top.join("bak/objects"), 0o000);
        let root = fs::read_dir(locked.join(".praps/store")).is_ok();
        assert!(
            !root,
            "this test must not run as root: a mode closes nothing"
        );
        refused(
            "a store that cannot be listed",
            praps(&["status"], &locked),
            1,
            "make that directory readable",
        );
        refused(
            "a directory above that cannot be searched",
            praps(&["status"], &locked.join("in")),
            1,
            "make the store of each repository above that directory readable",
        );
        refused(
            "init stopped by the filesystem",
            praps(&["init", "closed/new"], &dir),
            1,
            "`praps init` again once the cause is put right",
        );
        // A directory given to `verify` that may or may not be a store: it
        // is refused, and the working copy around it is not checked in its
        // place (code review, finding 1).
        let output = praps(&["verify", "bak"], &top);
        assert!(
            output.stdout.is_empty(),
            "nothing was called clean: {output:?}"
        );
        refused(
            "verify stopped by the filesystem",
            output,
            1,
            "make the store's directories readable",
        );
        mode(&locked.join(".praps/store"), 0o755);
        mode(&dir.join("closed"), 0o755);
        mode(&top.join("bak/objects"), 0o755);
        fs::remove_dir_all(top.join("bak")).unwrap();
    }

    // An untracked entry where the write must put a file.
    let blocked = dir.join("blocked");
    assert_eq!(code(&praps(&["init", "blocked"], &dir)), 0);
    write(&blocked, "b", b"tracked");
    assert_eq!(code(&praps(&["status"], &blocked)), 0);
    let with_b = logged_ops(&blocked)[0].clone();
    fs::remove_file(blocked.join("b")).unwrap();
    assert_eq!(code(&praps(&["status"], &blocked)), 0);
    write(&blocked, ".gitignore", b"b\n");
    write(&blocked, "b", b"mine");
    refused(
        "an entry in the way",
        praps(&["restore", &with_b], &blocked),
        1,
        "move those entries out of the way",
    );
    assert_eq!(fs::read(blocked.join("b")).unwrap(), b"mine");
    // A name this filesystem cannot hold beside another (Windows: a
    // case-only difference).
    if cfg!(windows) {
        let hazard = dir.join("hazard");
        assert_eq!(code(&praps(&["init", "hazard"], &dir)), 0);
        write(&hazard, "README", b"upper");
        assert_eq!(code(&praps(&["status"], &hazard)), 0);
        let upper = logged_ops(&hazard)[0].clone();
        fs::remove_file(hazard.join("README")).unwrap();
        write(&hazard, "readme", b"lower");
        refused(
            "a hazard",
            praps(&["restore", &upper], &hazard),
            1,
            "this build cannot write those paths on this filesystem yet; nothing was lost",
        );
    }

    // Each way `verify` fails: exit 1.
    refused(
        "verify: a directory that is not there",
        praps(&["verify", "nowhere"], &top),
        1,
        "give the directory of a working copy or of a store",
    );
    refused(
        "verify: a directory that is no store",
        praps(&["verify"], &plain),
        1,
        "`praps init` to make a repository",
    );
    refused(
        "verify: a damaged op file",
        praps(&["verify"], &corrupt),
        1,
        "put back the files named above",
    );
    fs::remove_dir_all(many.join(".praps/store/ops")).unwrap();
    refused(
        "verify: a working copy's store that is no store",
        praps(&["verify"], &many),
        1,
        "`praps init` there if its `init` was interrupted",
    );
    // 24 everywhere; four more on Unix by file modes and one by a removed
    // current directory; two more on Windows (a hazard, text that is not
    // Unicode).
    let expected = if cfg!(unix) { 29 } else { 26 };
    assert_eq!(kinds, expected, "kinds of refusal checked");
    let _ = fs::remove_dir_all(&dir);
}

/// Text given to `describe` without its quotes. Three or more words are a
/// usage error whose hint shows them as one quoted text. Two words, of
/// which the second is no directory, are refused naming that word, with
/// both shown as one quoted text. Nothing is described in either case, and
/// the command line the hint shows works.
#[test]
fn describe_with_unquoted_words_hints_quotes() {
    let dir = claim("unquoted");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "notes", b"a file");
    assert_eq!(code(&praps(&["status"], &root)), 0);
    let before = tree(&root);
    let usage = "usage: praps describe TEXT [DIR]\n";

    let output = praps(&["describe", "Add", "the", "parser"], &root);
    assert_eq!(code(&output), 2, "{output:?}");
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!(
            "{usage}hint: run `praps describe \"Add the parser\"` to give those words as one text\n"
        )
    );

    for (second, why) in [
        ("parser", "no such directory"),
        ("notes", "not a directory"),
    ] {
        let output = praps(&["describe", "Add", second], &root);
        assert_eq!(code(&output), 1, "{second}: {output:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(
            stderr(&output),
            format!(
                "praps describe: {second}: {why}\nhint: run `praps describe \"Add {second}\"` if both words are the text\n"
            )
        );
    }

    // Words a shell could change inside quotes are not shown: the hint
    // says where the rule is, or what the directory must be.
    let output = praps(&["describe", "say", "\"hi\"", "now"], &root);
    assert_eq!(code(&output), 2, "{output:?}");
    assert_eq!(
        stderr(&output),
        format!(
            "{usage}hint: run `praps help describe` for how to give text of more than one word\n"
        )
    );
    let output = praps(&["describe", "costs", "$5"], &root);
    assert_eq!(code(&output), 1, "{output:?}");
    assert_eq!(
        stderr(&output),
        "praps describe: $5: no such directory\nhint: give a directory that exists, or none for the current directory\n"
    );
    assert_eq!(tree(&root), before, "nothing was described or written");

    // The command line the first hint shows does what was meant.
    let output = praps(&["describe", "Add the parser"], &root);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(
        stdout(&output).ends_with(": Add the parser\n"),
        "{output:?}"
    );
    assert_eq!(working_copy_change(&root).description, b"Add the parser");
    let _ = fs::remove_dir_all(&dir);
}

/// `praps describe update docs`, where `docs` is a directory inside the
/// working copy, describes the change as `update`: a command takes what it
/// acts on and then a directory. The line it prints shows the text it took.
#[test]
fn describe_with_a_directory_as_its_last_word_takes_one_word() {
    let dir = claim("describe-dir");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "docs/guide.md", b"guide");
    let output = praps(&["describe", "update", "docs"], &root);
    assert_eq!(code(&output), 0, "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let change = working_copy_change(&root);
    assert_eq!(change.description, b"update");
    assert_eq!(
        stdout(&output),
        format!(
            "described change {}: update\n",
            &hash::hex(&change.change_id)[..12]
        )
    );
    // The change described is the working copy's, found from `docs`.
    assert_eq!(change.changes.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

// ------------------------------------- the README says what `praps` says (T-013)

/// The README's command table holds one row for each command `praps`
/// lists, in the order `praps` lists them, and no other: each row is the
/// command's usage line, as `praps help` prints it, and its summary, as the
/// list prints it (INT-0037 AC-2; INT-0003 AC-10).
#[test]
fn readme_lists_what_praps_lists() {
    let dir = claim("readme");
    let expected: Vec<(String, String)> = listed(&dir)
        .into_iter()
        .map(|(name, summary)| {
            let help = stdout(&praps(&["help", &name], &dir));
            let usage = help.lines().next().unwrap();
            (usage.strip_prefix("usage: ").unwrap().to_owned(), summary)
        })
        .collect();
    assert!(expected.len() >= 11, "{expected:?}");
    let readme =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md")).unwrap();
    // The rows of the table whose header is `| Command | What it does |`.
    let rows: Vec<(String, String)> = readme
        .lines()
        .skip_while(|line| *line != "| Command | What it does |")
        .skip(2)
        .take_while(|line| line.starts_with("| "))
        .map(|line| {
            let cells: Vec<&str> = line.trim_matches('|').split(" | ").collect();
            assert_eq!(cells.len(), 2, "{line}");
            (
                cells[0].trim().trim_matches('`').to_owned(),
                cells[1].trim().to_owned(),
            )
        })
        .collect();
    assert_eq!(rows, expected);
    let _ = fs::remove_dir_all(&dir);
}

/// After two `new` at once, written through the store, the next command
/// works from the merge and prints one note naming both changes (the race
/// rule, INT-0014).
#[test]
fn praps_notes_a_raced_working_copy() {
    let dir = claim("raced");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let mut store = Store::open(RealFs, &root.join(".praps/store"), HashAlg::DEFAULT, 5).unwrap();
    let head = *store.heads().unwrap().first().unwrap();
    let view = store.ops().unwrap()[&head].view;
    let view = View::from_envelope(&store.get_object(&view).unwrap()).unwrap();
    let wc = *view.workspaces["default"].working_copy.first().unwrap();
    let mut raced = Vec::new();
    for change_id in [[1_u8; 16], [2; 16]] {
        let changeset = codec::Changeset {
            parents: vec![wc],
            change_id,
            changes: Default::default(),
            description: Vec::new(),
        };
        let changeset = store.put_object(&changeset.to_envelope()).unwrap();
        let mut next = view.clone();
        let entry = next.workspaces.get_mut("default").unwrap();
        entry.heads.remove(&wc);
        entry.heads.insert(changeset);
        entry.working_copy = [changeset].into();
        let next = store.put_object(&next.to_envelope()).unwrap();
        store
            .commit_op(&Operation {
                parents: [head].into(),
                workspace: "default".to_owned(),
                view: next,
            })
            .unwrap();
        raced.push(changeset);
    }
    let output = praps(&["status"], &root);
    assert_eq!(code(&output), 0, "{output:?}");
    let err = String::from_utf8_lossy(&output.stderr);
    let notes: Vec<&str> = err
        .lines()
        .filter(|line| line.starts_with("note: "))
        .collect();
    assert_eq!(notes.len(), 1, "{err}");
    for id in &raced {
        assert!(notes[0].contains(&id.to_string()), "{err}");
    }
    let _ = fs::remove_dir_all(&dir);
}

/// Eight `praps new` started together in one working copy each exit 0, and
/// the repository goes on: the measurement that motivated Sprint 7 (3 of 8
/// succeeded at `855eb67`; INT-0010).
#[test]
fn eight_new_at_once_all_succeed() {
    let dir = claim("eight-new");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let children: Vec<_> = (0..8)
        .map(|_| {
            Command::new(env!("CARGO_BIN_EXE_praps"))
                .arg("new")
                .current_dir(&root)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert_eq!(code(&output), 0, "{output:?}");
    }
    let status = praps(&["status"], &root);
    assert_eq!(code(&status), 0, "{status:?}");
    let verify = praps(&["verify"], &root);
    assert_eq!(code(&verify), 0, "{verify:?}");
    assert!(stdout(&verify).contains("clean"), "{verify:?}");
    let _ = fs::remove_dir_all(&dir);
}

/// `undo` after a race that a `describe` resolved goes back to both racing
/// ops, and says so: `back to ops X, Y`, in id order (INT-0009; Sprint 7
/// T-005). With one op it prints the line it always printed.
#[test]
fn praps_undo_reports_every_op_it_went_back_to() {
    let dir = claim("undo-ops");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    let mut store = Store::open(RealFs, &root.join(".praps/store"), HashAlg::DEFAULT, 5).unwrap();
    let head = *store.heads().unwrap().first().unwrap();
    let view = store.ops().unwrap()[&head].view;
    let view = View::from_envelope(&store.get_object(&view).unwrap()).unwrap();
    let wc = *view.workspaces["default"].working_copy.first().unwrap();
    let mut raced = Vec::new();
    for change_id in [[3_u8; 16], [4; 16]] {
        let changeset = codec::Changeset {
            parents: vec![wc],
            change_id,
            changes: Default::default(),
            description: Vec::new(),
        };
        let changeset = store.put_object(&changeset.to_envelope()).unwrap();
        let mut next = view.clone();
        let entry = next.workspaces.get_mut("default").unwrap();
        entry.heads.remove(&wc);
        entry.heads.insert(changeset);
        entry.working_copy = [changeset].into();
        let next = store.put_object(&next.to_envelope()).unwrap();
        raced.push(
            store
                .commit_op(&Operation {
                    parents: [head].into(),
                    workspace: "default".to_owned(),
                    view: next,
                })
                .unwrap(),
        );
    }
    raced.sort();
    assert_eq!(code(&praps(&["describe", "picked"], &root)), 0);
    let output = praps(&["undo"], &root);
    assert_eq!(code(&output), 0, "{output:?}");
    let short = |id: &HashRef| hash::hex(id.digest())[..12].to_owned();
    assert_eq!(
        stdout(&output).lines().next().unwrap(),
        format!(
            "undid the last op: the working copy is back to ops {}, {}",
            short(&raced[0]),
            short(&raced[1])
        )
    );
    let _ = fs::remove_dir_all(&dir);
}
