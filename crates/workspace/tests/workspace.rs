//! The workspace on the simulator: init, open, snapshot, new and status
//! (INT-0014 AC-1, AC-2; INT-0004 AC-7; Sprint 2 T-005).

mod common;

use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;

use codec::{Changeset, Mode, Operation, RepoPath, View};
use common::{ROOT, delete, put, writes};
use hash::HashAlg;
use sim::{Fs, Mode as DirMode, RawName, SimFs};
use store::{Store, Variant};
use workspace::tree::Tree;
use workspace::{Change, Error, Reason, Reported, Workspace};

fn path(text: &str) -> RepoPath {
    RepoPath::new(text).unwrap()
}

fn fresh(seed: u64) -> Workspace<Rc<SimFs>> {
    let fs = Rc::new(SimFs::new(seed, DirMode::Strict));
    Workspace::init(fs, Path::new(ROOT), seed).unwrap().0
}

fn fs_of(ws: &Workspace<Rc<SimFs>>) -> Rc<SimFs> {
    Rc::clone(ws.store().fs())
}

/// The paths of a tree.
fn paths(tree: &Tree) -> Vec<&str> {
    tree.keys().map(RepoPath::as_str).collect()
}

#[test]
fn init_creates_a_root_change() {
    let fs = Rc::new(SimFs::new(1, DirMode::Strict));
    let (ws, created) = Workspace::init(Rc::clone(&fs), Path::new(ROOT), 1).unwrap();
    assert!(created);
    let snapshot = fs.snapshot();
    assert!(snapshot.contains_key("repo/.praps/store/objects"));
    assert!(snapshot.contains_key("repo/.praps/store/ops"));
    let ops = ws.store().ops().unwrap();
    assert_eq!(ops.len(), 1);
    let op = ops.values().next().unwrap();
    assert!(op.parents.is_empty());
    let view = View::from_envelope(&ws.store().get_object(&op.view).unwrap()).unwrap();
    let (wc_id, _, tree) = ws.working_copy().unwrap();
    assert_eq!(view.heads, [wc_id].into());
    assert_eq!(view.workspaces, [("default".to_owned(), wc_id)].into());
    let wc = Changeset::from_envelope(&ws.store().get_object(&wc_id).unwrap()).unwrap();
    assert!(wc.parents.is_empty() && wc.changes.is_empty() && wc.description.is_empty());
    assert!(tree.is_empty());
}

/// After an acknowledged init, every recovery of a strict crash keeps the
/// repository: `.praps` and the root's entries were synced (plan critique
/// round 2, C-002).
#[test]
fn init_makes_praps_dir_durable() {
    for seed in 0..40 {
        let fs = SimFs::new(seed, DirMode::Strict);
        Workspace::init(&fs, Path::new(ROOT), seed).unwrap();
        let recovered = fs.recover();
        let ws = Workspace::open(&recovered, Path::new(ROOT), seed)
            .unwrap_or_else(|error| panic!("seed {seed}: {error}"));
        assert!(ws.working_copy().unwrap().2.is_empty());
    }
}

#[test]
fn init_is_idempotent() {
    let ws = fresh(2);
    let fs = fs_of(&ws);
    let before = fs.snapshot();
    let (again, created) = Workspace::init(Rc::clone(&fs), Path::new(ROOT), 99).unwrap();
    assert!(!created);
    assert_eq!(fs.snapshot(), before, "a second init writes no file");
    assert_eq!(again.store().ops().unwrap().len(), 1);
}

/// A store with its layout but no op, as a crash inside `init` leaves it:
/// running `init` again finishes the job.
#[test]
fn init_completes_an_interrupted_init() {
    let fs = Rc::new(SimFs::new(3, DirMode::Strict));
    fs.create_dir(Path::new(ROOT)).unwrap();
    fs.create_dir(Path::new("repo/.praps")).unwrap();
    Store::init(
        Rc::clone(&fs),
        Path::new("repo/.praps/store"),
        HashAlg::DEFAULT,
        0,
    )
    .unwrap();
    assert!(matches!(
        Workspace::open(Rc::clone(&fs), Path::new(ROOT), 3),
        Err(Error::NotInitialised(_))
    ));
    let (ws, created) = Workspace::init(Rc::clone(&fs), Path::new(ROOT), 3).unwrap();
    assert!(created);
    assert_eq!(ws.store().ops().unwrap().len(), 1);
}

#[test]
fn open_without_ops_fails() {
    let fs = SimFs::new(0, DirMode::Strict);
    fs.create_dir(Path::new(ROOT)).unwrap();
    let error = Workspace::open(&fs, Path::new(ROOT), 0).err().unwrap();
    assert!(matches!(error, Error::NotInitialised(_)), "{error}");
    assert!(error.to_string().contains("praps init"));
}

/// Two ops with the same parent: every command stops, names both heads and
/// writes nothing (until INT-0010's merge).
#[test]
fn several_heads_stop_every_command() {
    let ws = fresh(4);
    let fs = fs_of(&ws);
    let mut other = Store::open(
        Rc::clone(&fs),
        Path::new("repo/.praps/store"),
        HashAlg::DEFAULT,
        9,
    )
    .unwrap();
    let head = *other.heads().unwrap().first().unwrap();
    let op = other.ops().unwrap()[&head].clone();
    let mut view = View::from_envelope(&other.get_object(&op.view).unwrap()).unwrap();
    let a = other
        .commit_op(&Operation {
            parents: [head].into(),
            view: op.view,
        })
        .unwrap();
    view.workspaces
        .insert("other".to_owned(), view.workspaces["default"]);
    let second_view = other.put_object(&view.to_envelope()).unwrap();
    let b = other
        .commit_op(&Operation {
            parents: [head].into(),
            view: second_view,
        })
        .unwrap();
    drop(ws);
    let mut ws = match Workspace::open(Rc::clone(&fs), Path::new(ROOT), 4) {
        Err(Error::SeveralHeads(heads)) => {
            let mut expected = vec![a, b];
            expected.sort();
            assert_eq!(heads, expected);
            // `init_with_store` still gives a handle, to try the commands.
            Workspace::init_with_store(other, Path::new(ROOT), 4)
                .unwrap()
                .0
        }
        other => panic!("open should stop: {:?}", other.err()),
    };
    put(&fs, "f", b"x");
    let before = fs.snapshot();
    for result in [
        ws.snapshot().map(drop),
        ws.status().map(drop),
        ws.new_change().map(drop),
    ] {
        let error = result.unwrap_err();
        assert!(
            matches!(&error, Error::SeveralHeads(h) if h.len() == 2),
            "{error}"
        );
        assert!(error.to_string().contains(&a.to_string()));
    }
    assert_eq!(fs.snapshot(), before);
}

/// The constructor tests use to pass a store built with a negative-control
/// variant.
#[test]
fn init_with_store_uses_the_given_store() {
    let fs = Rc::new(SimFs::new(5, DirMode::Strict));
    fs.create_dir(Path::new(ROOT)).unwrap();
    fs.create_dir(Path::new("repo/.praps")).unwrap();
    let store = Store::init_variant(
        Rc::clone(&fs),
        Path::new("repo/.praps/store"),
        HashAlg::DEFAULT,
        5,
        Variant::UnsyncedObjects,
    )
    .unwrap();
    let before = fs.trace().len();
    let (mut ws, created) = Workspace::init_with_store(store, Path::new(ROOT), 5).unwrap();
    assert!(created);
    // The variant skips object data syncs: the init's object writes show no
    // `sync_file` of an object.
    let trace = fs.trace();
    assert!(
        !trace[before..]
            .iter()
            .any(|l| l.starts_with("sync_file repo/.praps/store/objects")),
        "{trace:?}"
    );
    put(&fs, "f", b"x");
    assert!(ws.snapshot().unwrap().op.is_some());
}

#[test]
fn change_ids_follow_the_seed() {
    let ids = |seed: u64| {
        let mut ws = fresh(seed);
        let first = ws.working_copy().unwrap().1;
        let second = ws.new_change().unwrap();
        (first, second)
    };
    assert_eq!(ids(7), ids(7));
    assert_ne!(ids(7), ids(8));
    let (first, second) = ids(7);
    assert_ne!(first, second);
}

#[test]
fn snapshot_replaces_its_head() {
    let mut ws = fresh(6);
    let fs = fs_of(&ws);
    let (old, ..) = ws.working_copy().unwrap();
    put(&fs, "f", b"x");
    let op = ws.snapshot().unwrap().op.unwrap();
    let op = ws.store().ops().unwrap()[&op].clone();
    let view = View::from_envelope(&ws.store().get_object(&op.view).unwrap()).unwrap();
    let (new, ..) = ws.working_copy().unwrap();
    assert_ne!(old, new);
    assert_eq!(view.heads, [new].into());
    assert_eq!(view.workspaces["default"], new);
}

#[test]
fn snapshot_records_each_edit_kind() {
    let mut ws = fresh(7);
    let fs = fs_of(&ws);
    put(&fs, "a.txt", b"one");
    put(&fs, "src/lib.rs", b"fn main() {}");
    put(&fs, "src/deep/x", b"");
    put(&fs, "gone", b"bye");
    ws.snapshot().unwrap();
    assert_eq!(
        paths(&ws.working_copy().unwrap().2),
        ["a.txt", "gone", "src/deep/x", "src/lib.rs"]
    );
    put(&fs, "a.txt", b"two");
    delete(&fs, "gone");
    put(&fs, "new", b"n");
    ws.snapshot().unwrap();
    let (id, change_id, tree) = ws.working_copy().unwrap();
    assert_eq!(paths(&tree), ["a.txt", "new", "src/deep/x", "src/lib.rs"]);
    assert_eq!(
        ws.store().get_object(&tree[&path("a.txt")].blob).unwrap(),
        codec::blob(b"two")
    );
    // The working-copy change is a root: its changes hold every file.
    let changeset = Changeset::from_envelope(&ws.store().get_object(&id).unwrap()).unwrap();
    assert_eq!(changeset.changes.len(), 4);
    assert_eq!(change_id, changeset.change_id);
}

#[test]
fn unchanged_working_copy_writes_nothing() {
    let mut ws = fresh(8);
    let fs = fs_of(&ws);
    put(&fs, "f", b"x");
    assert!(ws.snapshot().unwrap().op.is_some());
    let before = fs.trace().len();
    let snapshot = ws.snapshot().unwrap();
    assert_eq!(snapshot.op, None);
    let trace = fs.trace();
    assert!(
        writes(&trace[before..]).is_empty(),
        "{:?}",
        &trace[before..]
    );
}

#[test]
fn new_starts_a_child_change() {
    let mut ws = fresh(9);
    let fs = fs_of(&ws);
    put(&fs, "f", b"x");
    let before_new = fs.snapshot();
    ws.snapshot().unwrap();
    let (parent, parent_change, parent_tree) = ws.working_copy().unwrap();
    let files_before: Vec<_> = before_new
        .keys()
        .filter(|k| !k.contains(".praps"))
        .cloned()
        .collect();
    let trace_before = fs.trace().len();
    let change_id = ws.new_change().unwrap();
    // No working-copy file was written: every write is under .praps.
    for line in writes(&fs.trace()[trace_before..]) {
        assert!(line.contains("repo/.praps"), "{line}");
    }
    let (child, child_change, tree) = ws.working_copy().unwrap();
    assert_ne!(child_change, parent_change);
    assert_eq!(child_change, change_id);
    let changeset = Changeset::from_envelope(&ws.store().get_object(&child).unwrap()).unwrap();
    assert_eq!(changeset.parents, [parent]);
    assert!(changeset.changes.is_empty());
    assert_eq!(tree, parent_tree);
    assert_eq!(ws.parent_tree().unwrap(), parent_tree);
    let files_after: Vec<_> = fs
        .snapshot()
        .keys()
        .filter(|k| !k.contains(".praps"))
        .cloned()
        .collect();
    assert_eq!(files_after, files_before);
    assert!(ws.status().unwrap().changes.is_empty());
}

#[test]
fn status_lists_added_modified_deleted_after_new() {
    let mut ws = fresh(10);
    let fs = fs_of(&ws);
    fs.set_has_executable_bit(true);
    put(&fs, "keep", b"k");
    put(&fs, "edit", b"1");
    put(&fs, "run", b"r");
    put(&fs, "drop", b"d");
    ws.new_change().unwrap();
    put(&fs, "edit", b"2");
    fs.set_executable(&Path::new(ROOT).join("run"), true)
        .unwrap();
    delete(&fs, "drop");
    put(&fs, "added", b"a");
    let status = ws.status().unwrap();
    assert_eq!(
        status.changes,
        [
            (path("added"), Change::Added),
            (path("drop"), Change::Deleted),
            (path("edit"), Change::Modified),
            (path("run"), Change::Modified),
        ]
    );
    let (id, ..) = ws.working_copy().unwrap();
    let changeset = Changeset::from_envelope(&ws.store().get_object(&id).unwrap()).unwrap();
    assert_eq!(changeset.changes.len(), 4);
    assert_eq!(changeset.changes[&path("drop")], codec::Entry::Delete);
    assert!(matches!(
        changeset.changes[&path("run")],
        codec::Entry::Put {
            mode: Mode::Executable,
            ..
        }
    ));
}

#[test]
fn status_lists_changes_sorted() {
    let mut ws = fresh(11);
    let fs = fs_of(&ws);
    for name in ["z", "b/c", "a", "b/a", "é"] {
        put(&fs, name, b"x");
    }
    fs.make_symlink(&Path::new(ROOT).join("link")).unwrap();
    let status = ws.status().unwrap();
    let shown: Vec<(&str, char)> = status
        .changes
        .iter()
        .map(|(p, c)| (p.as_str(), c.letter()))
        .collect();
    assert_eq!(
        shown,
        [
            ("a", 'A'),
            ("b/a", 'A'),
            ("b/c", 'A'),
            ("z", 'A'),
            ("é", 'A')
        ]
    );
    assert_eq!(
        status.reported,
        [Reported {
            path: "link".to_owned(),
            reason: Reason::Symlink
        }]
    );
}

#[test]
fn executable_bit_is_recorded() {
    let mut ws = fresh(12);
    let fs = fs_of(&ws);
    put(&fs, "run", b"#!");
    fs.set_executable(&Path::new(ROOT).join("run"), true)
        .unwrap();
    ws.snapshot().unwrap();
    assert_eq!(
        ws.working_copy().unwrap().2[&path("run")].mode,
        Mode::Executable
    );
    fs.set_executable(&Path::new(ROOT).join("run"), false)
        .unwrap();
    ws.snapshot().unwrap();
    assert_eq!(
        ws.working_copy().unwrap().2[&path("run")].mode,
        Mode::Regular
    );
}

/// Where files carry no executable bit (Windows), a tracked file keeps the
/// mode the working-copy change, or else its parent, last recorded, and a
/// new file is regular.
#[test]
fn no_executable_bit_keeps_the_recorded_mode() {
    let mut ws = fresh(13);
    let fs = fs_of(&ws);
    put(&fs, "run", b"#!");
    fs.set_executable(&Path::new(ROOT).join("run"), true)
        .unwrap();
    ws.snapshot().unwrap();
    ws.new_change().unwrap();
    fs.set_has_executable_bit(false);
    put(&fs, "run", b"#!edited");
    put(&fs, "new", b"n");
    ws.snapshot().unwrap();
    let tree = ws.working_copy().unwrap().2;
    assert_eq!(
        tree[&path("run")].mode,
        Mode::Executable,
        "kept from the parent"
    );
    assert_eq!(tree[&path("new")].mode, Mode::Regular);
}

#[test]
fn merge_changesets_are_refused() {
    let ws = fresh(14);
    let fs = fs_of(&ws);
    let (wc, ..) = ws.working_copy().unwrap();
    let mut store = Store::open(
        Rc::clone(&fs),
        Path::new("repo/.praps/store"),
        HashAlg::DEFAULT,
        3,
    )
    .unwrap();
    let head = *store.heads().unwrap().first().unwrap();
    let merge = Changeset {
        parents: vec![wc, wc],
        change_id: [1; 16],
        changes: BTreeMap::new(),
        description: Vec::new(),
    };
    let merge = store.put_object(&merge.to_envelope()).unwrap();
    let view = View {
        heads: [merge].into(),
        workspaces: [("default".to_owned(), merge)].into(),
    };
    let view = store.put_object(&view.to_envelope()).unwrap();
    store
        .commit_op(&Operation {
            parents: [head].into(),
            view,
        })
        .unwrap();
    let mut ws = Workspace::init_with_store(store, Path::new(ROOT), 14)
        .unwrap()
        .0;
    assert!(matches!(ws.snapshot(), Err(Error::Merge(id)) if id == merge));
}

#[test]
fn snapshot_reports_and_leaves_raw_names() {
    let mut ws = fresh(15);
    let fs = fs_of(&ws);
    put(&fs, "dir/kept", b"k");
    let dir = Path::new(ROOT).join("dir");
    fs.make_raw(&dir, RawName::Bytes(vec![b'a', 0xff]), b"raw")
        .unwrap();
    fs.make_raw(Path::new(ROOT), RawName::Wide(vec![0xd800]), b"")
        .unwrap();
    fs.make_symlink(&dir.join("link")).unwrap();
    fs.make_other(&dir.join("fifo")).unwrap();
    let before = fs.snapshot();
    let snapshot = ws.snapshot().unwrap();
    assert_eq!(
        snapshot.reported,
        // Sorted by path text: `\` (0x5c) sorts before `d`.
        [
            Reported {
                path: r"\u{d800}".to_owned(),
                reason: Reason::NotUtf8
            },
            Reported {
                path: r"dir/a\xff".to_owned(),
                reason: Reason::NotUtf8
            },
            Reported {
                path: "dir/fifo".to_owned(),
                reason: Reason::Special
            },
            Reported {
                path: "dir/link".to_owned(),
                reason: Reason::Symlink
            },
        ]
    );
    assert_eq!(paths(&ws.working_copy().unwrap().2), ["dir/kept"]);
    // Nothing outside .praps changed.
    let outside = |s: &BTreeMap<String, sim::Node>| {
        s.iter()
            .filter(|(k, _)| !k.starts_with("repo/.praps"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(outside(&fs.snapshot()), outside(&before));
}

#[test]
fn praps_and_git_dirs_are_always_ignored() {
    let mut ws = fresh(16);
    let fs = fs_of(&ws);
    put(&fs, ".git/config", b"[core]");
    put(
        &fs,
        "sub/.git",
        b"a file named .git below the top is tracked",
    );
    put(&fs, "sub/.praps/x", b"so is a .praps below the top");
    put(&fs, ".gitignore", b"!.git\n!.praps\n");
    ws.snapshot().unwrap();
    assert_eq!(
        paths(&ws.working_copy().unwrap().2),
        [".gitignore", "sub/.git", "sub/.praps/x"]
    );
}

#[test]
fn tracked_file_stays_tracked_when_ignored() {
    let mut ws = fresh(17);
    let fs = fs_of(&ws);
    put(&fs, "log.txt", b"1");
    put(&fs, "other.txt", b"1");
    ws.snapshot().unwrap();
    put(&fs, ".gitignore", b"*.txt\n");
    put(&fs, "log.txt", b"2");
    put(&fs, "fresh.txt", b"never tracked");
    ws.snapshot().unwrap();
    let tree = ws.working_copy().unwrap().2;
    assert_eq!(paths(&tree), [".gitignore", "log.txt", "other.txt"]);
    assert_eq!(
        ws.store().get_object(&tree[&path("log.txt")].blob).unwrap(),
        codec::blob(b"2")
    );
    delete(&fs, "log.txt");
    put(&fs, "log.txt", b"3");
    ws.snapshot().unwrap();
    // Recreated after the delete was snapshotted? No: the delete and the
    // re-creation fell between two snapshots, so it is still tracked.
    assert!(ws.working_copy().unwrap().2.contains_key(&path("log.txt")));
    delete(&fs, "log.txt");
    ws.snapshot().unwrap();
    put(&fs, "log.txt", b"4");
    ws.snapshot().unwrap();
    assert!(
        !ws.working_copy().unwrap().2.contains_key(&path("log.txt")),
        "once deleted, ignored again"
    );
}

#[test]
fn tracked_file_under_ignored_dir_is_snapshotted() {
    let mut ws = fresh(18);
    let fs = fs_of(&ws);
    put(&fs, "build/keep", b"1");
    ws.snapshot().unwrap();
    put(&fs, ".gitignore", b"build/\n");
    put(&fs, "build/keep", b"2");
    put(&fs, "build/new", b"untracked");
    put(&fs, "build/sub/deep", b"untracked");
    ws.snapshot().unwrap();
    let tree = ws.working_copy().unwrap().2;
    assert_eq!(paths(&tree), [".gitignore", "build/keep"]);
    assert_eq!(
        ws.store()
            .get_object(&tree[&path("build/keep")].blob)
            .unwrap(),
        codec::blob(b"2")
    );
}

#[test]
fn ignore_is_case_sensitive() {
    let mut ws = fresh(19);
    let fs = fs_of(&ws);
    put(&fs, ".gitignore", b"*.LOG\nBuild/\n");
    put(&fs, "a.log", b"x");
    put(&fs, "b.LOG", b"x");
    put(&fs, "build/x", b"x");
    put(&fs, "Build/y", b"x");
    ws.snapshot().unwrap();
    assert_eq!(
        paths(&ws.working_copy().unwrap().2),
        [".gitignore", "a.log", "build/x"]
    );
}
