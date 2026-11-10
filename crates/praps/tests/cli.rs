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
        &["undo", "--force"][..],
        &["undo", "a", "b"][..],
        &["restore", "-f", "abc"][..],
        &["restore", "abc", "--force"][..],
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

/// The 12-hex op prefixes `praps log` prints, newest first.
fn logged_ops(root: &Path) -> Vec<String> {
    let output = praps(&["log"], root);
    assert_eq!(code(&output), 0, "{output:?}");
    lines(&output)
        .iter()
        .map(|line| line.split(' ').next().unwrap().to_owned())
        .collect()
}

#[test]
fn praps_log_lists_ops_newest_first() {
    let dir = claim("log");
    let root = dir.join("repo");
    assert_eq!(code(&praps(&["init", "repo"], &dir)), 0);
    write(&root, "a", b"1");
    write(&root, "b", b"1");
    assert_eq!(code(&praps(&["new", "repo"], &dir)), 0);
    let output = praps(&["log", "repo"], &dir);
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
    assert!(lines[0].ends_with(" 2 files"), "{lines:?}");
    assert!(
        lines[2].ends_with(" 0 files"),
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
            from: head,
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
        "status", "new", "undo", "restore", "init", "help", "version",
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
