//! The workspace on the simulator: init, open, snapshot, new and status
//! (INT-0014 AC-1, AC-2; INT-0004 AC-7; Sprint 2 T-005).

mod common;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

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

fn fresh(seed: u64) -> Workspace<Arc<SimFs>> {
    let fs = Arc::new(SimFs::new(seed, DirMode::Strict));
    Workspace::init(fs, Path::new(ROOT), seed).unwrap().0
}

fn fs_of(ws: &Workspace<Arc<SimFs>>) -> Arc<SimFs> {
    Arc::clone(ws.store().fs())
}

/// The paths of a tree.
fn paths(tree: &Tree) -> Vec<&str> {
    tree.keys().map(RepoPath::as_str).collect()
}

#[test]
fn init_creates_a_root_change() {
    let fs = Arc::new(SimFs::new(1, DirMode::Strict));
    let (ws, created) = Workspace::init(Arc::clone(&fs), Path::new(ROOT), 1).unwrap();
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
    assert_eq!(
        view.workspaces,
        [(
            "default".to_owned(),
            codec::WorkspaceEntry {
                working_copy: [wc_id].into(),
                heads: [wc_id].into(),
            },
        )]
        .into()
    );
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
    let (again, created) = Workspace::init(Arc::clone(&fs), Path::new(ROOT), 99).unwrap();
    assert!(!created);
    assert_eq!(fs.snapshot(), before, "a second init writes no file");
    assert_eq!(again.store().ops().unwrap().len(), 1);
}

/// A store with its layout but no op, as a crash inside `init` leaves it:
/// running `init` again finishes the job.
#[test]
fn init_completes_an_interrupted_init() {
    let fs = Arc::new(SimFs::new(3, DirMode::Strict));
    fs.create_dir(Path::new(ROOT)).unwrap();
    fs.create_dir(Path::new("repo/.praps")).unwrap();
    Store::init(
        Arc::clone(&fs),
        Path::new("repo/.praps/store"),
        HashAlg::DEFAULT,
        0,
    )
    .unwrap();
    assert!(matches!(
        Workspace::open(Arc::clone(&fs), Path::new(ROOT), 3),
        Err(Error::NotInitialised(_))
    ));
    let (ws, created) = Workspace::init(Arc::clone(&fs), Path::new(ROOT), 3).unwrap();
    assert!(created);
    assert_eq!(ws.store().ops().unwrap().len(), 1);
}

#[test]
fn open_without_ops_fails() {
    let fs = SimFs::new(0, DirMode::Strict);
    fs.create_dir(Path::new(ROOT)).unwrap();
    let error = Workspace::open(&fs, Path::new(ROOT), 0).err().unwrap();
    assert!(matches!(error, Error::NotInitialised(_)), "{error}");
    // The message says what is wrong; what to do is `praps`'s hint.
    assert_eq!(
        error.to_string(),
        "repo is not an initialised Periapsis repository"
    );
}

/// Two ops with the same parent: every command stops, names both heads and
/// writes nothing (until INT-0010's merge).
#[test]
fn several_heads_stop_every_command() {
    let ws = fresh(4);
    let fs = fs_of(&ws);
    let mut other = Store::open(
        Arc::clone(&fs),
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
            workspace: "default".to_owned(),
            view: op.view,
        })
        .unwrap();
    view.workspaces
        .insert("other".to_owned(), view.workspaces["default"].clone());
    let second_view = other.put_object(&view.to_envelope()).unwrap();
    let b = other
        .commit_op(&Operation {
            parents: [head].into(),
            workspace: "default".to_owned(),
            view: second_view,
        })
        .unwrap();
    drop(ws);
    let mut ws = match Workspace::open(Arc::clone(&fs), Path::new(ROOT), 4) {
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
    let fs = Arc::new(SimFs::new(5, DirMode::Strict));
    fs.create_dir(Path::new(ROOT)).unwrap();
    fs.create_dir(Path::new("repo/.praps")).unwrap();
    let store = Store::init_variant(
        Arc::clone(&fs),
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
    assert_eq!(view.workspaces["default"].heads, [new].into());
    assert_eq!(view.workspaces["default"].working_copy, [new].into());
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
    fs.make_executable(&Path::new(ROOT).join("run"), true)
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
    fs.make_executable(&Path::new(ROOT).join("run"), true)
        .unwrap();
    ws.snapshot().unwrap();
    assert_eq!(
        ws.working_copy().unwrap().2[&path("run")].mode,
        Mode::Executable
    );
    fs.make_executable(&Path::new(ROOT).join("run"), false)
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
    fs.make_executable(&Path::new(ROOT).join("run"), true)
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
        Arc::clone(&fs),
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
        workspaces: [(
            "default".to_owned(),
            codec::WorkspaceEntry {
                working_copy: [merge].into(),
                heads: [merge].into(),
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

// ------------------------------------------------ code-review regressions

/// Where files carry no executable bit, a file the working-copy change
/// deleted and the user then restores takes its parent's mode back
/// (code review CR-01).
#[test]
fn no_executable_bit_restores_the_parents_mode() {
    let mut ws = fresh(20);
    let fs = fs_of(&ws);
    put(&fs, "run", b"#!");
    fs.make_executable(&Path::new(ROOT).join("run"), true)
        .unwrap();
    ws.new_change().unwrap();
    fs.set_has_executable_bit(false);
    delete(&fs, "run");
    assert_eq!(
        ws.status().unwrap().changes,
        [(path("run"), Change::Deleted)]
    );
    put(&fs, "run", b"#!");
    assert!(
        ws.status().unwrap().changes.is_empty(),
        "restored as it was"
    );
    assert_eq!(
        ws.working_copy().unwrap().2[&path("run")].mode,
        Mode::Executable
    );
}

/// A file deleted between the scan and its read counts as deleted; the
/// snapshot does not fail (code review CR-05).
#[test]
fn a_file_vanishing_mid_scan_is_deleted() {
    let mut ws = fresh(21);
    let fs = fs_of(&ws);
    put(&fs, "stays", b"s");
    put(&fs, "zz.swp", b"swap");
    ws.snapshot().unwrap();
    put(&fs, "zz.swp", b"changed");
    let hook_fs = Arc::clone(&fs);
    let mut fired = false;
    fs.set_hook(Some(Box::new(move |_| {
        let scanned_top = hook_fs.trace().last().is_some_and(|l| l == "scan repo");
        if !fired && scanned_top {
            fired = true;
            hook_fs.remove_file(Path::new("repo/zz.swp")).unwrap();
        }
    })));
    let result = ws.snapshot();
    fs.set_hook(None);
    result.unwrap();
    assert_eq!(paths(&ws.working_copy().unwrap().2), ["stays"]);
}

/// `init` and `open` sync the root's parent every time, not only when they
/// create the root (code review CR-06).
#[test]
fn init_and_open_sync_the_roots_parent() {
    let ws = fresh(22);
    let fs = fs_of(&ws);
    drop(ws);
    for open in [false, true] {
        let before = fs.trace().len();
        if open {
            Workspace::open(Arc::clone(&fs), Path::new(ROOT), 22).unwrap();
        } else {
            Workspace::init(Arc::clone(&fs), Path::new(ROOT), 22).unwrap();
        }
        let trace = fs.trace();
        assert!(
            trace[before..].iter().any(|l| l == "sync_dir repo/.."),
            "open {open}: {:?}",
            &trace[before..]
        );
    }
}

/// Every component of a reported path is shown escaped, not only the last
/// (code review CR-08).
#[test]
fn reported_paths_are_escaped_in_full() {
    let mut ws = fresh(23);
    let fs = fs_of(&ws);
    let dir = Path::new(ROOT).join("a\u{1}b");
    fs.create_dir(&dir).unwrap();
    fs.make_raw(&dir, RawName::Bytes(vec![0xff]), b"").unwrap();
    let reported = ws.snapshot().unwrap().reported;
    assert_eq!(
        reported,
        [Reported {
            path: r"a\u{1}b/\xff".to_owned(),
            reason: Reason::NotUtf8
        }]
    );
}

/// An op file that does not hash to its name stops every command, rather
/// than letting its parent silently become the head (code review CR-10).
#[test]
fn invalid_op_files_are_refused() {
    let mut ws = fresh(24);
    let fs = fs_of(&ws);
    put(&fs, "f", b"x");
    let op = ws.snapshot().unwrap().op.unwrap();
    let op_path = Path::new("repo/.praps/store/ops").join(store::names::file_name(&op));
    let mut bytes = fs.read(&op_path).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    fs.overwrite(&op_path, &bytes).unwrap();
    let error = ws.snapshot().unwrap_err();
    assert!(
        matches!(&error, Error::Corrupt(what)
            if what == "1 of 2 op files do not hash to their names or do not decode"),
        "{error}"
    );
    let error = Workspace::init(Arc::clone(&fs), Path::new(ROOT), 24)
        .err()
        .unwrap();
    assert!(matches!(error, Error::Corrupt(_)), "{error}");
}

/// `.praps` and `.git` at the top are ignored whatever their case, as a
/// case-insensitive filesystem treats them (code review CR-13).
#[test]
fn always_ignored_names_ignore_case() {
    let mut ws = fresh(25);
    let fs = fs_of(&ws);
    put(&fs, ".Git/config", b"x");
    put(&fs, ".PRAPS/x", b"x");
    put(&fs, "kept", b"x");
    ws.snapshot().unwrap();
    assert_eq!(paths(&ws.working_copy().unwrap().2), ["kept"]);
}

// -------------------------------------------- test-critique regressions

/// A snapshot replaces only its own changeset among the view's heads: any
/// other head is kept (test critique C-003).
#[test]
fn snapshot_keeps_other_heads() {
    let ws = fresh(30);
    let fs = fs_of(&ws);
    let (wc, ..) = ws.working_copy().unwrap();
    let mut store = Store::open(
        Arc::clone(&fs),
        Path::new("repo/.praps/store"),
        HashAlg::DEFAULT,
        30,
    )
    .unwrap();
    let head = *store.heads().unwrap().first().unwrap();
    let other = Changeset {
        parents: Vec::new(),
        change_id: [9; 16],
        changes: BTreeMap::new(),
        description: b"another head".to_vec(),
    };
    let other = store.put_object(&other.to_envelope()).unwrap();
    let view = View {
        workspaces: [(
            "default".to_owned(),
            codec::WorkspaceEntry {
                working_copy: [wc].into(),
                heads: [wc, other].into(),
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
    let mut ws = Workspace::init_with_store(store, Path::new(ROOT), 30)
        .unwrap()
        .0;
    put(&fs, "f", b"x");
    let op = ws.snapshot().unwrap().op.unwrap();
    let op = ws.store().ops().unwrap()[&op].clone();
    let view = View::from_envelope(&ws.store().get_object(&op.view).unwrap()).unwrap();
    let (new, ..) = ws.working_copy().unwrap();
    assert_eq!(view.workspaces["default"].heads, [new, other].into());
}

/// A directory whose name has no UTF-8 form is reported and not entered,
/// nothing outside `.praps` changes, and its siblings are still snapshotted
/// (INT-0004 AC-7; test critique C-005). The simulator's raw directories are
/// empty; `praps_status_reports_non_utf8_names_and_directories` covers one
/// holding a file on the real filesystem.
#[test]
fn raw_named_directories_are_reported_and_left_alone() {
    let mut ws = fresh(31);
    let fs = fs_of(&ws);
    put(&fs, "sibling", b"s");
    for raw in [
        RawName::Bytes(vec![0xfe, b'd']),
        RawName::Bytes(vec![b'd', 0x80]),
        RawName::Wide(vec![0xdc00, 0xd800]),
    ] {
        fs.make_raw_dir(Path::new(ROOT), raw).unwrap();
    }
    let before = fs.snapshot();
    let reported = ws.snapshot().unwrap().reported;
    assert_eq!(
        reported
            .iter()
            .map(|r| (r.path.as_str(), r.reason))
            .collect::<Vec<_>>(),
        [
            (r"\u{dc00}\u{d800}", Reason::NotUtf8),
            (r"\xfed", Reason::NotUtf8),
            (r"d\x80", Reason::NotUtf8),
        ]
    );
    assert_eq!(paths(&ws.working_copy().unwrap().2), ["sibling"]);
    let after = fs.snapshot();
    for (key, node) in &before {
        if !key.starts_with("repo/.praps") {
            assert_eq!(after.get(key), Some(node), "{key} changed");
        }
    }
}

/// A carriage return ends a `.gitignore` line without hiding an escaped
/// trailing space before it (test critique C-008).
#[test]
fn ignore_strips_cr_before_an_escaped_space() {
    let mut ws = fresh(32);
    let fs = fs_of(&ws);
    put(&fs, ".gitignore", b"x\\ \r\n");
    put(&fs, "x ", b"ignored");
    put(&fs, "x", b"kept");
    ws.snapshot().unwrap();
    assert_eq!(paths(&ws.working_copy().unwrap().2), [".gitignore", "x"]);
}

// ------------------------------------------- reading the op log (Sprint 6 T-003)

const OPS: &str = "repo/.praps/store/ops";

/// The listings of `ops/` and the reads of op files in a trace.
fn op_log_calls(trace: &[String]) -> (usize, Vec<String>) {
    let lists = trace
        .iter()
        .filter(|line| *line == &format!("list {OPS}"))
        .count();
    let reads = trace
        .iter()
        .filter_map(|line| line.strip_prefix(&format!("read {OPS}/")))
        .map(str::to_owned)
        .collect();
    (lists, reads)
}

/// A repository with five ops: init, three snapshots and a `new`. Returns
/// its filesystem and the view an early op left, to restore to.
fn five_ops(seed: u64) -> (Arc<SimFs>, hash::HashRef) {
    let mut ws = fresh(seed);
    let fs = fs_of(&ws);
    put(&fs, "keep", b"k");
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    let early = {
        let ops = ws.store().ops().unwrap();
        let head = *store::heads_of(&ops).first().unwrap();
        ops[&head].view
    };
    put(&fs, "a", b"2");
    ws.snapshot().unwrap();
    ws.new_change().unwrap();
    put(&fs, "b", b"1");
    ws.snapshot().unwrap();
    assert_eq!(ws.store().ops().unwrap().len(), 5);
    (fs, early)
}

/// The op files there now, by name, sorted.
fn op_files(fs: &SimFs) -> Vec<String> {
    let mut names = fs.list(Path::new(OPS)).unwrap();
    names.sort();
    names
}

/// `open` and one command read the op log once between them: `ops/` is
/// listed twice (the store lists it to see that it exists, and the log is
/// read with one listing), each op file there at the start is read once,
/// and each op the command commits costs two more reads, its parent and its
/// own name (Sprint 6 T-003).
#[test]
fn open_and_one_command_read_the_op_log_once() {
    type Command = fn(&mut Workspace<Arc<SimFs>>, &SimFs, &hash::HashRef);
    let commands: [(&str, Command, usize); 11] = [
        (
            "snapshot, nothing to record",
            |ws, _, _| drop(ws.snapshot().unwrap()),
            0,
        ),
        (
            "snapshot of an edit",
            |ws, fs, _| {
                put(fs, "c", b"new");
                assert!(ws.snapshot().unwrap().op.is_some());
            },
            1,
        ),
        ("status", |ws, _, _| drop(ws.status().unwrap()), 0),
        ("new_change", |ws, _, _| _ = ws.new_change().unwrap(), 1),
        ("ops", |ws, _, _| drop(ws.ops().unwrap()), 0),
        ("changes", |ws, _, _| drop(ws.changes().unwrap()), 0),
        (
            "describe",
            |ws, _, _| assert!(ws.describe(b"why").unwrap().op.is_some()),
            1,
        ),
        (
            "describe, the text it has",
            |ws, _, _| assert!(ws.describe(b"").unwrap().op.is_none()),
            0,
        ),
        ("undo", |ws, _, _| _ = ws.undo().unwrap(), 1),
        (
            "restore",
            |ws, _, _| {
                let first = ws.store().ops().unwrap();
                let root = first
                    .iter()
                    .find(|(_, op)| op.parents.is_empty())
                    .map(|(id, _)| id.to_string())
                    .unwrap();
                ws.restore(&root).unwrap();
            },
            1,
        ),
        (
            "restore_view",
            |ws, _, early| _ = ws.restore_view(early).unwrap(),
            1,
        ),
    ];
    for (name, command, commits) in commands {
        let (fs, early) = five_ops(40);
        let at_start = op_files(&fs);
        assert_eq!(at_start.len(), 5);
        let before = fs.trace().len();
        let mut ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 41).unwrap();
        let opened = fs.trace().len();
        command(&mut ws, &fs, &early);
        let trace = fs.trace();
        // `restore` looks up the root op through the store, which is the
        // test's own read: the command's calls are counted from after it.
        let (lists, mut reads) = if name == "restore" {
            let own = trace[opened..]
                .iter()
                .position(|line| *line == format!("list {OPS}"))
                .map(|at| opened + at)
                .unwrap();
            let after_own = own + 1 + at_start.len();
            let (open_lists, mut open_reads) = op_log_calls(&trace[before..opened]);
            let (more_lists, more_reads) = op_log_calls(&trace[after_own..]);
            open_reads.extend(more_reads);
            (open_lists + more_lists, open_reads)
        } else {
            op_log_calls(&trace[before..])
        };
        assert_eq!(lists, 2, "{name}: listings of ops/");
        let committed: Vec<String> = op_files(&fs)
            .into_iter()
            .filter(|file| !at_start.contains(file))
            .collect();
        assert_eq!(committed.len(), commits, "{name}: ops committed");
        // Each op file there at the start is read once to load the log. A
        // commit reads its parent once more, and its own name once.
        let mut expected = at_start.clone();
        let mut heads: Vec<String> = Vec::new();
        for new in &committed {
            let parent = ws.store().ops().unwrap()[&store::names::parse_name(new).unwrap()]
                .parents
                .first()
                .map(store::names::file_name)
                .unwrap();
            heads.push(parent);
            expected.push(new.clone());
        }
        expected.extend(heads);
        expected.sort();
        reads.sort();
        // The reads the lookup above made through `ws.store().ops()` came
        // after `trace` was taken, so they are not in `reads`.
        assert_eq!(reads, expected, "{name}: reads under ops/");
    }
}

/// A later command on the same workspace, and the first on a workspace
/// `init` returned, each read the op log once more: one listing and one
/// read of each op file (Sprint 6 T-003).
#[test]
fn a_later_command_reads_the_op_log_once_more() {
    let (fs, _) = five_ops(42);
    let mut ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 43).unwrap();
    ws.status().unwrap();
    for round in 0..2 {
        let before = fs.trace().len();
        ws.status().unwrap();
        let (lists, mut reads) = op_log_calls(&fs.trace()[before..]);
        assert_eq!(lists, 1, "round {round}");
        reads.sort();
        assert_eq!(reads, op_files(&fs), "round {round}");
    }
    // A later command that commits: it too lists `ops/` once and reads each
    // op file once, all of it before its first write. The commit then reads
    // its parent once more and its own name once, and nothing is read again
    // after it (test critique C-008).
    let at_start = op_files(&fs);
    put(&fs, "later", b"edit");
    let before = fs.trace().len();
    assert!(ws.snapshot().unwrap().op.is_some());
    let part = fs.trace()[before..].to_vec();
    let (lists, mut reads) = op_log_calls(&part);
    assert_eq!(lists, 1);
    let first_write = part
        .iter()
        .position(|line| writes(std::slice::from_ref(line)).len() == 1)
        .unwrap();
    let (lists_at_start, mut reads_at_start) = op_log_calls(&part[..first_write]);
    assert_eq!(lists_at_start, 1, "the listing comes before any write");
    reads_at_start.sort();
    assert_eq!(reads_at_start, at_start, "and so does each op file's read");
    let committed: Vec<String> = op_files(&fs)
        .into_iter()
        .filter(|file| !at_start.contains(file))
        .collect();
    assert_eq!(committed.len(), 1);
    let parent = ws.store().ops().unwrap()[&store::names::parse_name(&committed[0]).unwrap()]
        .parents
        .first()
        .map(store::names::file_name)
        .unwrap();
    let mut expected = at_start.clone();
    expected.push(parent);
    expected.push(committed[0].clone());
    expected.sort();
    reads.sort();
    assert_eq!(reads, expected);
    // A workspace `init` returned keeps no state: its first command reads.
    let mut ws = fresh(44);
    let fs = fs_of(&ws);
    let before = fs.trace().len();
    ws.status().unwrap();
    let (lists, reads) = op_log_calls(&fs.trace()[before..]);
    assert_eq!(lists, 1);
    assert_eq!(reads, op_files(&fs));
}

/// A second handle on the same store, as another process would hold.
fn other_handle(fs: &Arc<SimFs>, seed: u64) -> Store<Arc<SimFs>> {
    Store::open(
        Arc::clone(fs),
        Path::new("repo/.praps/store"),
        HashAlg::DEFAULT,
        seed,
    )
    .unwrap()
}

/// Commits, through `other`, one op over the single head whose view is the
/// head's own. Returns the head it was committed over and the new op.
fn commit_over_head(other: &mut Store<Arc<SimFs>>) -> (hash::HashRef, hash::HashRef) {
    let ops = other.ops().unwrap();
    let heads = store::heads_of(&ops);
    assert_eq!(heads.len(), 1);
    let head = *heads.first().unwrap();
    let mut view = View::from_envelope(&other.get_object(&ops[&head].view).unwrap()).unwrap();
    view.workspaces
        .insert("other".to_owned(), view.workspaces["default"].clone());
    let view = other.put_object(&view.to_envelope()).unwrap();
    let new = other
        .commit_op(&Operation {
            parents: [head].into(),
            workspace: "default".to_owned(),
            view,
        })
        .unwrap();
    (head, new)
}

/// The one case in which a command does not start from the log on disk:
/// an op another handle commits between `open` and the first command. The
/// command succeeds, both ops stand over the old head, and the next
/// command says the log has two heads (INT-0014; Sprint 6 T-003).
#[test]
fn an_operation_written_after_open_leaves_two_heads() {
    let (fs, _) = five_ops(45);
    let mut ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 46).unwrap();
    let (old_head, theirs) = commit_over_head(&mut other_handle(&fs, 47));
    put(&fs, "mine", b"edit");
    let ours = ws.snapshot().unwrap().op.expect("the edit is recorded");
    let ops = ws.store().ops().unwrap();
    assert_eq!(ops.len(), 7);
    for id in [ours, theirs] {
        assert_eq!(
            ops[&id].parents.iter().collect::<Vec<_>>(),
            [&old_head],
            "both stand over the head `open` read"
        );
    }
    let before = fs.snapshot();
    let error = ws.status().unwrap_err();
    let mut expected = vec![ours, theirs];
    expected.sort();
    assert!(
        matches!(&error, Error::SeveralHeads(heads) if *heads == expected),
        "{error}"
    );
    assert_eq!(fs.snapshot(), before, "the stopped command wrote nothing");
}

/// After a command has finished, the next one starts from the log on disk:
/// an op another handle committed over the head is the head it builds on
/// (Sprint 6 T-003).
#[test]
fn the_next_command_builds_on_another_handles_op() {
    let (fs, _) = five_ops(48);
    let mut ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 49).unwrap();
    ws.status().unwrap();
    let (_, theirs) = commit_over_head(&mut other_handle(&fs, 50));
    put(&fs, "mine", b"edit");
    let ours = ws.snapshot().unwrap().op.expect("the edit is recorded");
    let ops = ws.store().ops().unwrap();
    assert_eq!(ops[&ours].parents.iter().collect::<Vec<_>>(), [&theirs]);
    assert_eq!(store::heads_of(&ops).len(), 1);
    // The other handle's view entry is kept by the snapshot that follows it.
    let view = View::from_envelope(&ws.store().get_object(&ops[&ours].view).unwrap()).unwrap();
    assert!(view.workspaces.contains_key("other"));
}

/// `working_copy` and `parent_tree` read the store each time: straight
/// after `open`, with no command in between, they show what another handle
/// has since committed, not the state `open` kept (Sprint 6 T-003).
#[test]
fn accessors_read_what_is_stored() {
    let (fs, _) = five_ops(51);
    let ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 52).unwrap();
    let (opened_wc, ..) = ws.working_copy().unwrap();
    let opened_parent = ws.parent_tree().unwrap();
    // Another handle commits an op whose view names a new working-copy
    // changeset: a child of the old one that deletes `b`.
    let mut other = other_handle(&fs, 53);
    let ops = other.ops().unwrap();
    let head = *store::heads_of(&ops).first().unwrap();
    let child = Changeset {
        parents: vec![opened_wc],
        change_id: [7; 16],
        changes: [(path("b"), codec::Entry::Delete)].into(),
        description: Vec::new(),
    };
    let child = other.put_object(&child.to_envelope()).unwrap();
    let view = View {
        workspaces: [(
            "default".to_owned(),
            codec::WorkspaceEntry {
                working_copy: [child].into(),
                heads: [child].into(),
            },
        )]
        .into(),
    };
    let view = other.put_object(&view.to_envelope()).unwrap();
    other
        .commit_op(&Operation {
            parents: [head].into(),
            workspace: "default".to_owned(),
            view,
        })
        .unwrap();

    let (wc, change_id, tree) = ws.working_copy().unwrap();
    assert_eq!(wc, child);
    assert_eq!(change_id, [7; 16]);
    assert_eq!(paths(&tree), ["a", "keep"]);
    let parent = ws.parent_tree().unwrap();
    assert_eq!(paths(&parent), ["a", "b", "keep"]);
    assert_ne!(parent, opened_parent);
}

/// Each error says only what is wrong. Its text is fixed here, variant by
/// variant, and none holds advice: what to do about it is said once, by
/// the hint `praps` prints after it (Sprint 6 T-011; INT-0037 AC-4).
#[test]
fn messages_say_only_what_is_wrong() {
    let id = |text: &[u8]| HashAlg::DEFAULT.hash(text);
    let (one, two) = (id(b"one"), id(b"two"));
    let hazard = workspace::hazard::Hazard {
        path: path("dir/aux.txt"),
        component: "aux.txt".to_owned(),
        problem: workspace::hazard::Problem::Reserved("AUX".to_owned()),
    };
    let cases: Vec<(Error, String)> = vec![
        (
            Error::Io(std::io::Error::other("the disk said no")),
            "the disk said no".to_owned(),
        ),
        (
            Error::NotInitialised("project".into()),
            "project is not an initialised Periapsis repository".to_owned(),
        ),
        (
            Error::SeveralHeads(vec![one, two]),
            format!("the op log has 2 heads, which cannot be merged yet: {one} {two}"),
        ),
        (
            Error::Corrupt("view x: truncated".to_owned()),
            "the repository is corrupt: view x: truncated".to_owned(),
        ),
        (
            Error::Merge(one),
            format!("changeset {one} has several parents; merges are not supported yet"),
        ),
        (
            Error::Hazards(vec![hazard.clone()]),
            format!(
                "the working copy cannot be written on this filesystem; no file was changed:\n  {hazard}"
            ),
        ),
        (
            Error::Blocked(vec!["b: a directory is where a file must be written".to_owned()]),
            "untracked entries are in the way, and no file was changed:\n  b: a directory is where a file must be written"
                .to_owned(),
        ),
        (
            Error::NothingToUndo,
            "nothing to undo: the last op is the root op".to_owned(),
        ),
        (
            Error::UnknownOp("3f".to_owned()),
            "no op matches `3f`".to_owned(),
        ),
        (
            Error::AmbiguousOp("3".to_owned(), vec![one, two]),
            format!("`3` matches 2 ops:\n  {one}\n  {two}"),
        ),
    ];
    // Every variant is here: this match has no catch-all arm, so a variant
    // added to the crate must be named, and then counted below.
    let name = |error: &Error| match error {
        Error::Io(_) => "Io",
        Error::NotInitialised(_) => "NotInitialised",
        Error::SeveralHeads(_) => "SeveralHeads",
        Error::Corrupt(_) => "Corrupt",
        Error::Merge(_) => "Merge",
        Error::Hazards(_) => "Hazards",
        Error::Blocked(_) => "Blocked",
        Error::NothingToUndo => "NothingToUndo",
        Error::UnknownOp(_) => "UnknownOp",
        Error::AmbiguousOp(..) => "AmbiguousOp",
    };
    let names: std::collections::BTreeSet<&str> =
        cases.iter().map(|(error, _)| name(error)).collect();
    assert_eq!(names.len(), 10, "{names:?}");
    assert_eq!(cases.len(), 10);
    for (error, text) in &cases {
        let shown = error.to_string();
        assert_eq!(&shown, text, "{}", name(error));
        for advice in ["praps", "run ", "try again", "hint", "move them", "(run"] {
            assert!(
                !shown.contains(advice),
                "{}: {shown:?} holds {advice:?}",
                name(error)
            );
        }
    }
}
