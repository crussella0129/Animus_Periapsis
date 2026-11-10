//! Writing the working copy on the simulator: materialise, the record and
//! resume (INT-0014 AC-1, INT-0009 AC-3; Sprint 3 T-006).
//!
//! Each test moves the working copy to an earlier view with
//! `Workspace::restore_view`, which `undo` and `restore` build on.

mod common;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use codec::RepoPath;
use common::{ROOT, delete, files_on_disk, put, writes};
use hash::HashRef;
use sim::{Fs, Mode as DirMode, NameRules, SimFs};
use workspace::materialise;
use workspace::record::{self, Record};
use workspace::tree::Tree;
use workspace::{Error, Workspace};

type Ws = Workspace<Arc<SimFs>>;

fn fresh(seed: u64) -> (Ws, Arc<SimFs>) {
    let fs = Arc::new(SimFs::new(seed, DirMode::Strict));
    let ws = Workspace::init(Arc::clone(&fs), Path::new(ROOT), seed)
        .unwrap()
        .0;
    (ws, fs)
}

fn praps() -> std::path::PathBuf {
    Path::new(ROOT).join(".praps")
}

/// The single head op and its view.
fn head(ws: &Ws) -> (HashRef, HashRef) {
    let ops = ws.store().ops().unwrap();
    let heads = store::heads_of(&ops);
    assert_eq!(heads.len(), 1);
    let head = *heads.first().unwrap();
    (head, ops[&head].view)
}

/// Snapshots and returns the head's view.
fn checkpoint(ws: &mut Ws) -> HashRef {
    ws.snapshot().unwrap();
    head(ws).1
}

/// What the files on disk should be: `(path, content, executable)`.
fn expect(files: &[(&str, &str, bool)]) -> BTreeMap<String, (Vec<u8>, bool)> {
    files
        .iter()
        .map(|(path, content, exec)| ((*path).to_owned(), (content.as_bytes().to_vec(), *exec)))
        .collect()
}

fn path(text: &str) -> RepoPath {
    RepoPath::new(text).unwrap()
}

/// The index of the first trace line starting with `prefix`.
fn index_of(trace: &[String], prefix: &str) -> usize {
    trace
        .iter()
        .position(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("no `{prefix}` in {trace:#?}"))
}

#[test]
fn materialise_writes_removes_and_sets_modes() {
    let (mut ws, fs) = fresh(1);
    put(&fs, "a", b"1");
    put(&fs, "run", b"1");
    fs.make_executable(&Path::new(ROOT).join("run"), true)
        .unwrap();
    put(&fs, "d/x", b"1");
    let v1 = checkpoint(&mut ws);
    put(&fs, "a", b"2");
    put(&fs, "b", b"2");
    put(&fs, "run", b"2");
    fs.make_executable(&Path::new(ROOT).join("run"), false)
        .unwrap();
    delete(&fs, "d/x");
    checkpoint(&mut ws);

    ws.restore_view(&v1).unwrap();
    assert_eq!(
        files_on_disk(&*fs),
        expect(&[("a", "1", false), ("d/x", "1", false), ("run", "1", true)])
    );
    assert_eq!(head(&ws).1, v1, "the new head names the exact view");
    assert_eq!(ws.snapshot().unwrap().op, None, "the files are the view's");
    assert!(!fs.snapshot().contains_key("repo/.praps/working-copy"));
}

#[test]
fn materialise_swaps_file_and_directory() {
    let (mut ws, fs) = fresh(2);
    put(&fs, "a", b"file");
    let file = checkpoint(&mut ws);
    delete(&fs, "a");
    ws.snapshot().unwrap();
    put(&fs, "a/x", b"inside");
    let dir = checkpoint(&mut ws);

    ws.restore_view(&file).unwrap();
    assert_eq!(files_on_disk(&*fs), expect(&[("a", "file", false)]));
    ws.restore_view(&dir).unwrap();
    assert_eq!(files_on_disk(&*fs), expect(&[("a/x", "inside", false)]));
    ws.restore_view(&file).unwrap();
    assert_eq!(files_on_disk(&*fs), expect(&[("a", "file", false)]));
}

#[test]
fn executable_bit_is_set_before_the_rename() {
    let (mut ws, fs) = fresh(3);
    put(&fs, "run", b"1");
    fs.make_executable(&Path::new(ROOT).join("run"), true)
        .unwrap();
    let v1 = checkpoint(&mut ws);
    put(&fs, "run", b"2");
    checkpoint(&mut ws);
    let start = fs.trace().len();
    ws.restore_view(&v1).unwrap();
    let trace = fs.trace()[start..].to_vec();
    let tmp = "repo/.praps-tmp-0";
    let order: Vec<usize> = [
        format!("create_new {tmp}"),
        format!("write {tmp}"),
        format!("set_executable {tmp}"),
        format!("sync_file {tmp}"),
        format!("rename {tmp}"),
    ]
    .iter()
    .map(|line| index_of(&trace, line))
    .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{trace:#?}");
}

/// Edits are snapshotted before the write, so they stay reachable; ignored
/// files are never touched.
#[test]
fn materialise_leaves_user_edits() {
    let (mut ws, fs) = fresh(4);
    put(&fs, ".gitignore", b"*.log\n");
    put(&fs, "a", b"1");
    let v1 = checkpoint(&mut ws);
    put(&fs, "a", b"2");
    checkpoint(&mut ws);
    put(&fs, "a", b"edit");
    put(&fs, "keep.log", b"untracked");
    ws.restore_view(&v1).unwrap();
    let on_disk = files_on_disk(&*fs);
    assert_eq!(on_disk["a"].0, b"1");
    assert_eq!(on_disk["keep.log"].0, b"untracked");
    // The edit is in the snapshot op, the restore op's parent.
    let ops = ws.store().ops().unwrap();
    let (restore_op, _) = head(&ws);
    let snapshot_op = ops[&restore_op].parents.first().copied().unwrap();
    ws.restore_view(&ops[&snapshot_op].view).unwrap();
    assert_eq!(files_on_disk(&*fs)["a"].0, b"edit");
}

#[test]
fn materialise_removes_only_directories_it_emptied() {
    let (mut ws, fs) = fresh(5);
    put(&fs, ".gitignore", b"*.log\n");
    let v1 = checkpoint(&mut ws);
    put(&fs, "d/x", b"1");
    put(&fs, "e/y", b"1");
    checkpoint(&mut ws);
    put(&fs, "e/z.log", b"ignored");
    fs.create_dir(&Path::new(ROOT).join("u")).unwrap();
    ws.restore_view(&v1).unwrap();
    let snapshot = fs.snapshot();
    assert!(!snapshot.contains_key("repo/d"), "d was emptied");
    assert!(
        snapshot.contains_key("repo/e/z.log"),
        "e holds an ignored file"
    );
    assert!(!snapshot.contains_key("repo/e/y"));
    assert!(snapshot.contains_key("repo/u"), "u was never F's");
}

/// Builds the three blocking cases from ignored, untracked entries that the
/// opening snapshot cannot capture.
#[test]
fn blocking_entries_are_refused() {
    type Plant = fn(&SimFs);
    let cases: [(&str, &str, Plant); 3] = [
        ("x.log", "an untracked entry is where a file", |fs| {
            put(fs, "x.log", b"untracked");
        }),
        ("k.log/f", "where a directory is needed", |fs| {
            // The snapshot left `k.log/` empty; the user replaces it.
            let dir = Path::new(ROOT).join("k.log");
            fs.sync_dir(&dir).unwrap();
            fs.remove_dir(&dir).unwrap();
            fs.sync_dir(Path::new(ROOT)).unwrap();
            put(fs, "k.log", b"untracked file");
        }),
        ("m.log", "a directory is where a file", |fs| {
            put(fs, "m.log/q", b"untracked dir");
        }),
    ];
    for (i, (tracked, needle, plant)) in cases.into_iter().enumerate() {
        let (mut ws, fs) = fresh(10 + i as u64);
        put(&fs, tracked, b"tracked");
        let v1 = checkpoint(&mut ws);
        delete(&fs, tracked);
        put(&fs, ".gitignore", b"*.log\n");
        checkpoint(&mut ws);
        plant(&fs);
        ws.snapshot().unwrap();
        let ops = ws.store().ops().unwrap().len();
        let start = fs.trace().len();
        let before = fs.snapshot();
        match ws.restore_view(&v1) {
            Err(Error::Blocked(entries)) => {
                assert!(entries.iter().any(|e| e.contains(needle)), "{entries:?}");
            }
            other => panic!("{tracked}: {other:?}"),
        }
        assert_eq!(writes(&fs.trace()[start..]), Vec::<&String>::new());
        assert_eq!(fs.snapshot(), before);
        assert_eq!(ws.store().ops().unwrap().len(), ops);
    }
}

#[test]
fn temporary_name_collisions_are_refused() {
    // A tracked file with the temporary name the write needs.
    let (mut ws, fs) = fresh(20);
    put(&fs, ".praps-tmp-0", b"mine");
    put(&fs, "b", b"1");
    let v1 = checkpoint(&mut ws);
    put(&fs, "b", b"2");
    checkpoint(&mut ws);
    assert!(
        matches!(ws.restore_view(&v1), Err(Error::Blocked(e)) if e.iter().any(|e| e.contains("temporary name")))
    );
    // An ignored file with it.
    let (mut ws, fs) = fresh(21);
    put(&fs, "b", b"1");
    let v1 = checkpoint(&mut ws);
    put(&fs, "b", b"2");
    put(&fs, ".gitignore", b".praps-tmp-*\n");
    checkpoint(&mut ws);
    // The diff is `.gitignore` (0), then `b` (1).
    put(&fs, ".praps-tmp-1", b"ignored");
    assert!(
        matches!(ws.restore_view(&v1), Err(Error::Blocked(e)) if e.iter().any(|e| e.contains(".praps-tmp-1")))
    );
}

#[test]
fn temporary_names_are_deterministic() {
    let tree = |files: &[(&str, &str)]| -> Tree {
        files
            .iter()
            .map(|(p, c)| {
                (
                    path(p),
                    workspace::tree::File {
                        blob: codec::object_id(hash::HashAlg::DEFAULT, &codec::blob(c.as_bytes())),
                        mode: codec::Mode::Regular,
                    },
                )
            })
            .collect()
    };
    let from = tree(&[("a", "1"), ("d/b", "1"), ("same", "1")]);
    let target = tree(&[("a", "2"), ("c", "2"), ("d/b", "2"), ("same", "1")]);
    let expected = vec![
        (path("a"), ".praps-tmp-0".to_owned()),
        (path("c"), ".praps-tmp-1".to_owned()),
        (path("d/b"), "d/.praps-tmp-2".to_owned()),
    ];
    assert_eq!(materialise::diff(&from, &target), expected);
    assert_eq!(materialise::diff(&from, &target), expected);
}

/// Syncs every file and directory under the repository, so the edits a
/// test made survive the crash under test, as a user's durable edits would.
fn make_durable(fs: &SimFs) {
    let mut dirs: Vec<String> = Vec::new();
    for (path, _) in fs.snapshot() {
        let Some(rest) = path.strip_prefix("repo/") else {
            continue;
        };
        if rest.starts_with(".praps") {
            continue;
        }
        let on_disk = Path::new(ROOT).join(rest);
        match fs.snapshot()[&path] {
            sim::Node::Dir => dirs.push(rest.to_owned()),
            sim::Node::File(_) | sim::Node::Executable(_) => fs.sync_file(&on_disk).unwrap(),
            _ => {}
        }
    }
    // Deepest first, then the root.
    dirs.sort_by_key(|d| std::cmp::Reverse(d.matches('/').count()));
    for dir in dirs {
        fs.sync_dir(&Path::new(ROOT).join(dir)).unwrap();
    }
    fs.sync_dir(Path::new(ROOT)).unwrap();
}

/// Two views whose working copies differ in every way, on a fresh fs.
fn swap_setup(seed: u64) -> (Ws, Arc<SimFs>, HashRef) {
    let (mut ws, fs) = fresh(seed);
    put(&fs, "a", b"file");
    put(&fs, "keep", b"1");
    let v1 = checkpoint(&mut ws);
    delete(&fs, "a");
    put(&fs, "a/x", b"inside");
    put(&fs, "keep", b"2");
    put(&fs, "n1", b"new");
    put(&fs, "n2", b"new");
    put(&fs, "n3", b"new");
    checkpoint(&mut ws);
    make_durable(&fs);
    (ws, fs, v1)
}

#[test]
fn record_is_durable_before_the_op() {
    let (mut ws, fs, v1) = swap_setup(30);
    let start = fs.trace().len();
    ws.restore_view(&v1).unwrap();
    let trace = fs.trace()[start..].to_vec();
    let rename = index_of(&trace, "rename repo/.praps/working-copy.new");
    let sync = rename + 1;
    assert_eq!(trace[sync], "sync_dir repo/.praps");
    let op = index_of(&trace, "create_new repo/.praps/store/ops/");
    assert!(sync < op, "{trace:#?}");
}

#[test]
fn materialise_syncs_before_dropping_the_record() {
    let (mut ws, fs, v1) = swap_setup(31);
    let start = fs.trace().len();
    ws.restore_view(&v1).unwrap();
    let trace = fs.trace()[start..].to_vec();
    let drop = trace
        .iter()
        .position(|line| line == "remove_file repo/.praps/working-copy")
        .unwrap();
    let last_rename = trace
        .iter()
        .rposition(|line| line.starts_with("rename repo/") && !line.contains(".praps/"))
        .unwrap();
    assert!(last_rename < drop);
    assert_eq!(trace[last_rename + 1], "sync_dir repo", "{trace:#?}");
    assert_eq!(trace.last().unwrap(), "sync_dir repo/.praps");
}

/// Runs `restore_view` on a fresh `swap_setup(seed)` with a crash at the
/// first trace line starting with `at`, then recovers and reopens.
fn crash_at(seed: u64, at: &str) -> (SimFs, HashRef) {
    let (mut ws, fs, v1) = swap_setup(seed);
    let start = fs.trace().len() as u64;
    // A dry run on an identical setup gives the line's syscall number.
    let (mut dry, dry_fs, dry_v1) = swap_setup(seed);
    let dry_start = dry_fs.trace().len();
    dry.restore_view(&dry_v1).unwrap();
    let k = index_of(&dry_fs.trace()[dry_start..], at) as u64;
    fs.set_crash_at(Some(start + k + 1));
    assert!(ws.restore_view(&v1).is_err());
    (fs.recover(), v1)
}

#[test]
fn resume_finishes_an_interrupted_materialise() {
    for seed in 0..10 {
        let (fs, _) = crash_at(40 + seed, "rename repo/.praps-tmp-");
        let ws = Workspace::open(&fs, Path::new(ROOT), seed).unwrap();
        assert!(ws.notes().is_empty(), "{:?}", ws.notes());
        assert_eq!(
            files_on_disk(&fs),
            expect(&[("a", "file", false), ("keep", "1", false)]),
            "seed {seed}"
        );
        assert!(!fs.snapshot().contains_key("repo/.praps/working-copy"));
    }
}

#[test]
fn resume_after_removals_finishes_a_directory_to_file_swap() {
    for seed in 0..10 {
        // Step 1 is done and synced; step 2 has not removed `a/`.
        let (fs, _) = crash_at(50 + seed, "remove_dir repo/a");
        assert!(fs.snapshot().contains_key("repo/a"));
        let ws = Workspace::open(&fs, Path::new(ROOT), seed).unwrap();
        assert!(ws.notes().is_empty(), "{:?}", ws.notes());
        assert_eq!(
            files_on_disk(&fs),
            expect(&[("a", "file", false), ("keep", "1", false)]),
            "seed {seed}"
        );
    }
}

#[test]
fn resume_drops_a_record_whose_op_never_landed() {
    for seed in 0..10 {
        let (fs, _) = crash_at(60 + seed, "create_new repo/.praps/store/ops/");
        assert!(fs.snapshot().contains_key("repo/.praps/working-copy"));
        let mut ws = Workspace::open(&fs, Path::new(ROOT), seed).unwrap();
        assert!(ws.notes().is_empty());
        assert!(!fs.snapshot().contains_key("repo/.praps/working-copy"));
        assert_eq!(
            ws.snapshot().unwrap().op,
            None,
            "the files are F's, untouched"
        );
    }
}

#[test]
fn resume_drops_a_record_whose_target_is_not_the_head() {
    let (mut ws, fs) = fresh(70);
    put(&fs, "a", b"1");
    let v1 = checkpoint(&mut ws);
    let (old_op, _) = head(&ws);
    put(&fs, "a", b"2");
    checkpoint(&mut ws);
    let (now, _) = head(&ws);
    let _ = v1;
    record::write(
        &*fs,
        &praps(),
        &Record {
            from: now,
            target: old_op,
        },
    )
    .unwrap();
    let before = files_on_disk(&*fs);
    let ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 70).unwrap();
    assert_eq!(ws.notes().len(), 1, "{:?}", ws.notes());
    assert!(ws.notes()[0].contains("no longer the only head"));
    assert_eq!(files_on_disk(&*fs), before);
    assert!(!fs.snapshot().contains_key("repo/.praps/working-copy"));
}

#[test]
fn resume_with_a_target_that_is_not_the_head_removes_temporaries() {
    let (mut ws, fs) = fresh(71);
    put(&fs, "a", b"1");
    checkpoint(&mut ws);
    let (old_op, _) = head(&ws);
    put(&fs, "a", b"2");
    checkpoint(&mut ws);
    let (now, _) = head(&ws);
    // The only diff path is `a`, so its temporary name is `.praps-tmp-0`.
    put(&fs, ".praps-tmp-0", b"half written");
    record::write(
        &*fs,
        &praps(),
        &Record {
            from: now,
            target: old_op,
        },
    )
    .unwrap();
    let ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 71).unwrap();
    assert_eq!(ws.notes().len(), 1);
    assert_eq!(files_on_disk(&*fs), expect(&[("a", "2", false)]));
}

#[test]
fn resume_finishes_what_it_can_and_notes_the_rest() {
    let rules = NameRules {
        case_insensitive: true,
        windows_names: false,
    };
    // Restore from v1 to v2 ({n1, n2, n3} written in that order), crashing
    // after the first of them is renamed into place.
    let setup = |seed| {
        let (mut ws, fs) = fresh(seed);
        fs.set_name_rules(rules);
        put(&fs, "keep", b"1");
        let v1 = checkpoint(&mut ws);
        put(&fs, "n1", b"new");
        put(&fs, "n2", b"new");
        put(&fs, "n3", b"new");
        let v2 = checkpoint(&mut ws);
        ws.restore_view(&v1).unwrap();
        make_durable(&fs);
        (ws, fs, v2)
    };
    let (mut dry, dry_fs, dry_v2) = setup(80);
    let dry_start = dry_fs.trace().len();
    dry.restore_view(&dry_v2).unwrap();
    let k = index_of(&dry_fs.trace()[dry_start..], "rename repo/.praps-tmp-1") as u64;
    let (mut ws, fs, v2) = setup(80);
    let start = fs.trace().len() as u64;
    fs.set_crash_at(Some(start + k + 1));
    assert!(ws.restore_view(&v2).is_err());
    let fs = fs.recover();
    // After the crash, the user puts an untracked `n2` (blocking) and an
    // `N3` (a case variant of a path still to write).
    put(&fs, "n2", b"mine");
    put(&fs, "N3", b"mine too");
    make_durable(&fs);
    let ws = Workspace::open(&fs, Path::new(ROOT), 80).unwrap();
    let notes = ws.notes().join("\n");
    assert!(notes.contains("n2"), "{notes}");
    assert!(notes.contains("n3"), "{notes}");
    let on_disk = files_on_disk(&fs);
    assert_eq!(on_disk["n1"].0, b"new");
    assert_eq!(on_disk["n2"].0, b"mine");
    assert_eq!(on_disk["N3"].0, b"mine too");
    assert!(!on_disk.contains_key("n3"));
    assert!(!fs.snapshot().contains_key("repo/.praps/working-copy"));
}

#[test]
fn materialise_without_an_executable_bit_compares_bytes_only() {
    let (mut ws, fs) = fresh(90);
    put(&fs, "run", b"1");
    fs.make_executable(&Path::new(ROOT).join("run"), true)
        .unwrap();
    let v1 = checkpoint(&mut ws);
    put(&fs, "run", b"2");
    checkpoint(&mut ws);
    fs.set_has_executable_bit(false);
    ws.restore_view(&v1).unwrap();
    assert_eq!(files_on_disk(&*fs)["run"].0, b"1");
    assert_eq!(
        ws.snapshot().unwrap().op,
        None,
        "the mode is kept from the view"
    );
    let (_, _, tree) = ws.working_copy().unwrap();
    assert_eq!(tree[&path("run")].mode, codec::Mode::Executable);
}

#[test]
fn corrupt_record_is_an_error() {
    let (_, fs) = fresh(91);
    put(&fs, ".praps/working-copy", b"from nothing\n");
    match Workspace::open(Arc::clone(&fs), Path::new(ROOT), 91) {
        Err(Error::Corrupt(what)) => assert!(what.contains("working-copy"), "{what}"),
        other => panic!("{:?}", other.err()),
    }
}

#[test]
fn init_never_resumes() {
    let (ws, fs) = fresh(92);
    let (op, _) = head(&ws);
    // A record whose target never landed: `open` would drop it.
    let missing: HashRef =
        "blake3:0000000000000000000000000000000000000000000000000000000000000000"
            .parse()
            .unwrap();
    record::write(
        &*fs,
        &praps(),
        &Record {
            from: op,
            target: missing,
        },
    )
    .unwrap();
    let before = fs.snapshot();
    Workspace::init(Arc::clone(&fs), Path::new(ROOT), 92).unwrap();
    assert_eq!(fs.snapshot(), before, "a repeated init writes nothing");
    Workspace::open(Arc::clone(&fs), Path::new(ROOT), 92).unwrap();
    assert!(!fs.snapshot().contains_key("repo/.praps/working-copy"));
}

#[test]
fn prefix_names_are_tracked() {
    let (mut ws, fs) = fresh(93);
    put(&fs, ".praps-tmp-x", b"mine");
    let status = ws.status().unwrap();
    assert_eq!(
        status.changes,
        [(path(".praps-tmp-x"), workspace::Change::Added)]
    );
}

#[test]
fn record_round_trips() {
    let a: HashRef = "blake3:1111111111111111111111111111111111111111111111111111111111111111"
        .parse()
        .unwrap();
    let b: HashRef = "blake3:2222222222222222222222222222222222222222222222222222222222222222"
        .parse()
        .unwrap();
    let record = Record { from: a, target: b };
    assert_eq!(Record::decode(&record.encode()), Ok(record));
    for bad in [&b""[..], b"from x\n", b"target 1\nfrom 2\n", b"\xff"] {
        assert!(Record::decode(bad).is_err(), "{bad:?}");
    }
    let mut extra = record.encode();
    extra.extend_from_slice(b"more\n");
    assert!(Record::decode(&extra).is_err());
}

// ------------------------------------------------ code review fixes (Test)

/// Every temporary name of the diff is checked, not only those of paths to
/// write: resume removes them all (code review CR-01).
#[test]
fn removal_temporary_names_are_checked() {
    let (mut ws, fs) = fresh(100);
    put(&fs, ".gitignore", b".praps-tmp-*\n");
    put(&fs, "b", b"2");
    let target = checkpoint(&mut ws);
    put(&fs, "a", b"1");
    put(&fs, "b", b"1");
    checkpoint(&mut ws);
    // The diff is `a` (0, to remove), then `b` (1, to write).
    put(&fs, ".praps-tmp-0", b"ignored, mine");
    match ws.restore_view(&target) {
        Err(Error::Blocked(entries)) => {
            assert!(
                entries.iter().any(|e| e.contains(".praps-tmp-0")),
                "{entries:?}"
            );
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(files_on_disk(&*fs)[".praps-tmp-0"].0, b"ignored, mine");
}

/// Resume never removes a tracked file at a temporary name.
#[test]
fn resume_keeps_a_tracked_file_at_a_temporary_name() {
    let (mut ws, fs) = fresh(101);
    put(&fs, ".praps-tmp-0", b"tracked");
    put(&fs, "a", b"1");
    checkpoint(&mut ws);
    let (old_op, _) = head(&ws);
    put(&fs, "a", b"2");
    checkpoint(&mut ws);
    let (now, _) = head(&ws);
    // An interrupted write back to `old_op`'s view: its op, committed over
    // the head without writing a file, and the record naming it.
    let view = ws.store().ops().unwrap()[&old_op].view;
    let mut store = store::Store::open(
        Arc::clone(&fs),
        &Path::new(ROOT).join(".praps/store"),
        hash::HashAlg::DEFAULT,
        1,
    )
    .unwrap();
    let restore_op = store
        .commit_op(&codec::Operation {
            parents: [now].into(),
            view,
        })
        .unwrap();
    record::write(
        &*fs,
        &praps(),
        &Record {
            from: now,
            target: restore_op,
        },
    )
    .unwrap();
    let ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 101).unwrap();
    assert_eq!(files_on_disk(&*fs)[".praps-tmp-0"].0, b"tracked");
    // `a` (diff index 0) has `.praps-tmp-0` as its temporary name: skipped.
    assert!(
        ws.notes().iter().any(|n| n.contains("a:")),
        "{:?}",
        ws.notes()
    );
}

/// Names with control characters are found on disk (code review CR-02).
#[test]
fn control_characters_in_names_are_seen() {
    let (mut ws, fs) = fresh(102);
    put(&fs, ".gitignore", b"x*\n");
    put(&fs, "c", b"c");
    let target = checkpoint(&mut ws);
    put(&fs, "a\tb", b"tabbed");
    checkpoint(&mut ws);
    ws.restore_view(&target).unwrap();
    assert!(!files_on_disk(&*fs).contains_key("a\tb"), "removed");
    // A tracked name with a tab, where an ignored file of that name stands.
    let (mut ws, fs) = fresh(103);
    put(&fs, "x\ty", b"tracked");
    let target = checkpoint(&mut ws);
    delete(&fs, "x\ty");
    put(&fs, ".gitignore", b"x*\n");
    checkpoint(&mut ws);
    put(&fs, "x\ty", b"ignored, mine");
    assert!(matches!(ws.restore_view(&target), Err(Error::Blocked(_))));
    assert_eq!(files_on_disk(&*fs)["x\ty"].0, b"ignored, mine");
}

/// An edit saved between the plan and the write is kept, and noted (code
/// review CR-03): for a path to remove and for a path to write.
#[test]
fn edits_during_the_write_are_kept() {
    let (mut ws, fs) = fresh(104);
    put(&fs, "a", b"1");
    put(&fs, "b", b"1");
    let target = checkpoint(&mut ws);
    put(&fs, "a", b"2");
    put(&fs, "b", b"2");
    put(&fs, "c", b"2");
    checkpoint(&mut ws);
    let hook_fs = Arc::clone(&fs);
    let done = Arc::new(std::sync::Mutex::new((false, false)));
    let flags = Arc::clone(&done);
    fs.set_hook(Some(Box::new(move |_| {
        let last = hook_fs.trace().last().cloned().unwrap_or_default();
        let (c_done, a_done) = *flags.lock().unwrap();
        // After the op is written, before step 1 re-checks `c`.
        if !c_done && last.starts_with("create_new repo/.praps/store/ops/") {
            *flags.lock().unwrap() = (true, a_done);
            hook_fs
                .overwrite(&Path::new(ROOT).join("c"), b"saved by the user")
                .unwrap();
        }
        // After `a`'s temporary file is synced, before it replaces `a`.
        if !a_done && last == "sync_file repo/.praps-tmp-0" {
            flags.lock().unwrap().1 = true;
            hook_fs
                .overwrite(&Path::new(ROOT).join("a"), b"saved too")
                .unwrap();
        }
    })));
    ws.restore_view(&target).unwrap();
    fs.set_hook(None);
    assert_eq!(*done.lock().unwrap(), (true, true));
    let on_disk = files_on_disk(&*fs);
    assert_eq!(on_disk["a"].0, b"saved too");
    assert_eq!(on_disk["b"].0, b"1");
    assert_eq!(on_disk["c"].0, b"saved by the user");
    assert!(!on_disk.contains_key(".praps-tmp-0"));
    let notes = ws.notes().join("\n");
    assert!(
        notes.contains("a: it changed") && notes.contains("c: it changed"),
        "{notes}"
    );
    // The next snapshot keeps both edits.
    ws.snapshot().unwrap();
    let (_, _, tree) = ws.working_copy().unwrap();
    assert!(tree.contains_key(&path("c")));
}

/// A directory at a temporary name does not stop a resume: its path is
/// left and noted, and the rest is finished (code review CR-04).
#[test]
fn resume_skips_a_directory_at_a_temporary_name() {
    // The diff of the swap setup's restore is a, a/x, keep, n1, n2, n3:
    // `keep`'s temporary name is `.praps-tmp-2`.
    let (fs, _) = crash_at(105, "rename repo/.praps-tmp-0");
    fs.create_dir(&Path::new(ROOT).join(".praps-tmp-2"))
        .unwrap();
    make_durable(&fs);
    let ws = Workspace::open(&fs, Path::new(ROOT), 105).unwrap();
    assert!(
        ws.notes().iter().any(|n| n.contains("keep")),
        "{:?}",
        ws.notes()
    );
    let on_disk = files_on_disk(&fs);
    assert_eq!(on_disk["a"].0, b"file");
    assert_eq!(on_disk["keep"].0, b"2", "left as F had it");
    assert!(fs.snapshot().contains_key("repo/.praps-tmp-2"));
    assert!(!fs.snapshot().contains_key("repo/.praps/working-copy"));
}

/// Each stage of the working copy marks its named point (Sprint 4 T-006):
/// a snapshot, a write of the working copy, and both ways a resume ends.
#[test]
fn workspace_points_mark_each_stage() {
    use std::sync::Mutex;
    let watch = |fs: &SimFs| {
        let seen = Arc::new(Mutex::new(Vec::new()));
        for point in workspace::points::POINTS {
            let seen = Arc::clone(&seen);
            fs.on_point(point.name, move || seen.lock().unwrap().push(point.name));
        }
        seen
    };
    let take = |seen: &Arc<Mutex<Vec<&'static str>>>| std::mem::take(&mut *seen.lock().unwrap());

    let (mut ws, fs) = fresh(105);
    let seen = watch(&fs);
    put(&fs, "a", b"1");
    let target = checkpoint(&mut ws);
    assert_eq!(
        take(&seen),
        ["workspace.snapshot.scanned", "workspace.snapshot.committed"]
    );
    delete(&fs, "a");
    put(&fs, "b", b"2");
    checkpoint(&mut ws);
    take(&seen);
    ws.restore_view(&target).unwrap();
    let marked = take(&seen);
    for stage in [
        "workspace.record.written",
        "workspace.apply.path_written",
        "workspace.apply.path_removed",
        "workspace.record.dropped",
    ] {
        assert!(marked.contains(&stage), "{stage} not in {marked:?}");
    }
    assert_eq!(marked.first(), Some(&"workspace.snapshot.scanned"));
    fs.clear_points();

    // A crash right after the record is written, before the target op: the
    // next open drops the record.
    for (finish, expected) in [
        (false, "workspace.resume.dropped"),
        (true, "workspace.apply.path_written"),
    ] {
        // Two paths, so a crash after the first is written leaves the second
        // for resume.
        let (mut ws, fs) = fresh(106);
        put(&fs, "a", b"1");
        put(&fs, "b", b"1");
        let target = checkpoint(&mut ws);
        put(&fs, "a", b"2");
        put(&fs, "b", b"2");
        // Durable, so the crash keeps the user's files as they were.
        for name in ["a", "b"] {
            fs.sync_file(&Path::new(ROOT).join(name)).unwrap();
        }
        fs.sync_dir(Path::new(ROOT)).unwrap();
        checkpoint(&mut ws);
        let crash_fs = Arc::clone(&fs);
        let crash_on = if finish {
            "workspace.apply.path_written"
        } else {
            "workspace.record.written"
        };
        fs.on_point(crash_on, move || {
            crash_fs.set_crash_at(Some(crash_fs.syscalls() + 1));
        });
        assert!(ws.restore_view(&target).is_err());
        fs.clear_points();
        drop(ws);
        let restarted = Arc::new(fs.recover());
        let seen = watch(&restarted);
        Workspace::open(Arc::clone(&restarted), Path::new(ROOT), 7).unwrap();
        let marked = take(&seen);
        assert_eq!(
            marked.first(),
            Some(&"workspace.resume.started"),
            "{marked:?}"
        );
        assert!(marked.contains(&expected), "{expected} not in {marked:?}");
        restarted.clear_points();
    }
}

/// Everything on the filesystem outside `.praps`, by path.
fn outside_praps(fs: &SimFs) -> BTreeMap<String, sim::Node> {
    fs.snapshot()
        .into_iter()
        .filter(|(path, _)| !path.starts_with("repo/.praps"))
        .collect()
}

/// `open` checks that every op file is whole before it resumes an
/// interrupted write. With the record's target op damaged, resume would
/// take the write for one that never began and drop the record, though
/// files were already written; the half-written working copy would then be
/// recorded as the user's edits once the op file was repaired. Instead
/// `open` stops, the record stays, and no file changes; with the op file
/// put right, it resumes and finishes (Sprint 6 T-004; INT-0014 AC-1).
#[test]
fn a_damaged_op_stops_open_before_resume() {
    for seed in 0..10 {
        // A restore that crashed after its op was committed and its removals
        // were made, as it renamed the first written file into place.
        let (fs, _) = crash_at(140 + seed, "rename repo/.praps-tmp-");
        let record_path = praps().join(record::RECORD);
        let record_bytes = fs.read(&record_path).unwrap();
        let record = record::read(&fs, &praps()).unwrap().unwrap().unwrap();
        let op_path = Path::new(ROOT)
            .join(".praps/store/ops")
            .join(store::names::file_name(&record.target));
        let whole = fs.read(&op_path).unwrap();
        let mut damaged = whole.clone();
        *damaged.last_mut().unwrap() ^= 1;
        fs.overwrite(&op_path, &damaged).unwrap();
        let before = outside_praps(&fs);
        assert_ne!(
            files_on_disk(&fs),
            expect(&[("a", "file", false), ("keep", "1", false)]),
            "seed {seed}: the write was interrupted part way"
        );

        let error = Workspace::open(&fs, Path::new(ROOT), seed).err().unwrap();
        assert!(matches!(error, Error::Corrupt(_)), "seed {seed}: {error}");
        assert_eq!(
            fs.read(&record_path).unwrap(),
            record_bytes,
            "seed {seed}: the record stays as it was"
        );
        assert_eq!(outside_praps(&fs), before, "seed {seed}: no file changed");

        // With the op file's bytes put back, the next open finishes the write.
        fs.overwrite(&op_path, &whole).unwrap();
        let ws = Workspace::open(&fs, Path::new(ROOT), seed).unwrap();
        assert!(ws.notes().is_empty(), "seed {seed}: {:?}", ws.notes());
        assert_eq!(
            files_on_disk(&fs),
            expect(&[("a", "file", false), ("keep", "1", false)]),
            "seed {seed}"
        );
        assert!(!fs.snapshot().contains_key("repo/.praps/working-copy"));
    }
}
