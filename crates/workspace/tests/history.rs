//! `log`, `undo` and `restore` on the simulator (INT-0009 AC-1 to AC-3;
//! Sprint 3 T-007), and the reversibility corpora over every op type
//! (Sprint 3 T-009).

mod common;

use std::path::Path;
use std::rc::Rc;

use common::{ROOT, delete, files_on_disk, put, tree_files, writes};
use hash::HashRef;
use sim::{Fs, Mode as DirMode, NameRules, SimFs};
use workspace::{Error, Workspace};

type Ws = Workspace<Rc<SimFs>>;

fn fresh(seed: u64) -> (Ws, Rc<SimFs>) {
    let fs = Rc::new(SimFs::new(seed, DirMode::Strict));
    let ws = Workspace::init(Rc::clone(&fs), Path::new(ROOT), seed)
        .unwrap()
        .0;
    (ws, fs)
}

fn head(ws: &Ws) -> HashRef {
    let heads = ws.store().heads().unwrap();
    assert_eq!(heads.len(), 1);
    *heads.first().unwrap()
}

fn view_of(ws: &Ws, op: &HashRef) -> HashRef {
    ws.store().ops().unwrap()[op].view
}

fn content(fs: &SimFs, path: &str) -> Option<Vec<u8>> {
    files_on_disk(fs).get(path).map(|(bytes, _)| bytes.clone())
}

fn hex(id: &HashRef) -> String {
    id.digest().iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn log_lists_ops_newest_first() {
    let (mut ws, fs) = fresh(1);
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    ws.new_change().unwrap();
    // An edit not yet snapshotted: `log` snapshots first.
    put(&fs, "a", b"2");
    let log = ws.log().unwrap();
    assert_eq!(log.len(), 4, "root, snapshot, new, and log's own snapshot");
    assert_eq!(log[0].id, head(&ws));
    for pair in log.windows(2) {
        assert_eq!(
            pair[0].parents,
            [pair[1].id].into(),
            "children before parents"
        );
    }
    assert!(log[3].parents.is_empty(), "the root op is last");
    assert_eq!(log[0].files, 1);
    assert_eq!(log[3].files, 0);
    let (wc, change_id, _) = ws.working_copy().unwrap();
    assert_eq!((log[0].working_copy, log[0].change_id), (wc, change_id));
    // A second `log` with nothing changed adds no op.
    assert_eq!(ws.log().unwrap().len(), 4);
}

#[test]
fn undo_returns_to_the_parent_view() {
    let (mut ws, fs) = fresh(2);
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    let before_new = head(&ws);
    ws.new_change().unwrap();
    let h = head(&ws);
    put(&fs, "a", b"2");
    put(&fs, "b", b"new");
    let (parent, new) = ws.undo().unwrap();
    assert_eq!(parent, before_new, "H is the head before undo's snapshot");
    assert_eq!(view_of(&ws, &new), view_of(&ws, &before_new));
    assert_eq!(content(&fs, "a").unwrap(), b"1");
    assert_eq!(content(&fs, "b"), None);
    // The edits are in undo's snapshot op, the new op's parent, a child of H.
    let ops = ws.store().ops().unwrap();
    let snapshot = *ops[&new].parents.first().unwrap();
    assert_eq!(ops[&snapshot].parents, [h].into());
    ws.restore(&hex(&snapshot)).unwrap();
    assert_eq!(content(&fs, "a").unwrap(), b"2");
    assert_eq!(content(&fs, "b").unwrap(), b"new");
}

#[test]
fn undo_twice_toggles() {
    let (mut ws, fs) = fresh(3);
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    put(&fs, "a", b"2");
    ws.snapshot().unwrap();
    let after = view_of(&ws, &head(&ws));
    ws.undo().unwrap();
    assert_eq!(content(&fs, "a").unwrap(), b"1");
    let (_, new) = ws.undo().unwrap();
    assert_eq!(content(&fs, "a").unwrap(), b"2");
    assert_eq!(view_of(&ws, &new), after);
}

#[test]
fn undo_on_the_root_op_fails() {
    // Clean: nothing at all is written.
    let (mut ws, fs) = fresh(4);
    let start = fs.trace().len();
    assert!(matches!(ws.undo(), Err(Error::NothingToUndo)));
    assert_eq!(writes(&fs.trace()[start..]), Vec::<&String>::new());
    assert_eq!(ws.store().ops().unwrap().len(), 1);
    // With an edit: only the snapshot is written, and it keeps the edit.
    put(&fs, "a", b"1");
    assert!(matches!(ws.undo(), Err(Error::NothingToUndo)));
    assert_eq!(ws.store().ops().unwrap().len(), 2);
    assert_eq!(content(&fs, "a").unwrap(), b"1");
    assert_eq!(ws.working_copy().unwrap().2.len(), 1);
}

#[test]
fn undo_and_restore_snapshot_first() {
    let (mut ws, fs) = fresh(5);
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    let first = head(&ws);
    put(&fs, "a", b"2");
    ws.snapshot().unwrap();
    put(&fs, "a", b"unsnapshotted");
    let (_, new) = ws.restore(&hex(&first)).unwrap();
    let ops = ws.store().ops().unwrap();
    let snapshot = *ops[&new].parents.first().unwrap();
    ws.restore(&hex(&snapshot)).unwrap();
    assert_eq!(content(&fs, "a").unwrap(), b"unsnapshotted");
    put(&fs, "a", b"again");
    let (_, new) = ws.undo().unwrap();
    let snapshot = *ws.store().ops().unwrap()[&new].parents.first().unwrap();
    ws.restore(&hex(&snapshot)).unwrap();
    assert_eq!(content(&fs, "a").unwrap(), b"again");
}

#[test]
fn restore_by_unique_prefix() {
    let (mut ws, fs) = fresh(6);
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    let target = head(&ws);
    put(&fs, "a", b"2");
    ws.snapshot().unwrap();
    let ops = ws.store().ops().unwrap();
    // The shortest prefix that only `target` has.
    let full = hex(&target);
    let unique = (1..=full.len())
        .map(|n| &full[..n])
        .find(|prefix| ops.keys().filter(|id| hex(id).starts_with(prefix)).count() == 1)
        .unwrap()
        .to_owned();
    let (found, _) = ws.restore(&unique).unwrap();
    assert_eq!(found, target);
    assert_eq!(content(&fs, "a").unwrap(), b"1");
    // The full text form and the `blake3:` prefix form work too.
    put(&fs, "a", b"3");
    assert_eq!(ws.restore(&target.to_string()).unwrap().0, target);
    assert_eq!(ws.restore(&format!("blake3:{unique}")).unwrap().0, target);
}

#[test]
fn restore_unknown_or_ambiguous_prefix_fails() {
    let (mut ws, fs) = fresh(7);
    for i in 0..20u8 {
        put(&fs, "a", &[i]);
        ws.snapshot().unwrap();
    }
    let ops = ws.store().ops().unwrap();
    // Some one-digit prefix is shared by at least two of 21 ops.
    let shared = "0123456789abcdef"
        .chars()
        .map(|c| c.to_string())
        .find(|p| {
            ops.keys()
                .filter(|id| hex(id).starts_with(p.as_str()))
                .count()
                >= 2
        })
        .unwrap();
    let count = ops.len();
    let start = fs.trace().len();
    match ws.restore(&shared) {
        Err(Error::AmbiguousOp(spec, candidates)) => {
            assert_eq!(spec, shared);
            assert!(candidates.len() >= 2);
            assert!(candidates.iter().all(|c| hex(c).starts_with(&shared)));
        }
        other => panic!("{other:?}"),
    }
    for bad in [
        "zz",
        "",
        "sha256:00",
        "0000000000000000000000000000000000000000000000000000000000000000",
    ] {
        assert!(matches!(ws.restore(bad), Err(Error::UnknownOp(_))), "{bad}");
    }
    assert_eq!(writes(&fs.trace()[start..]), Vec::<&String>::new());
    assert_eq!(ws.store().ops().unwrap().len(), count);
}

/// A hazard stops the write before anything but the opening snapshot: from
/// a clean working copy, nothing at all (INT-0004 AC-3, AC-4; T-005).
#[test]
fn hazards_are_refused_before_any_write() {
    for (name, rules, tracked) in [
        (
            "windows",
            NameRules {
                case_insensitive: false,
                windows_names: true,
            },
            vec!["CON", "ok"],
        ),
        (
            "case",
            NameRules {
                case_insensitive: true,
                windows_names: false,
            },
            vec!["README", "readme"],
        ),
    ] {
        let (mut ws, fs) = fresh(8);
        for path in &tracked {
            put(&fs, path, b"x");
        }
        ws.snapshot().unwrap();
        let target = head(&ws);
        for path in &tracked {
            delete(&fs, path);
        }
        ws.snapshot().unwrap();
        fs.set_name_rules(rules);
        let ops = ws.store().ops().unwrap().len();
        let start = fs.trace().len();
        match ws.restore(&hex(&target)) {
            Err(Error::Hazards(hazards)) => assert!(!hazards.is_empty(), "{name}"),
            other => panic!("{name}: {other:?}"),
        }
        assert_eq!(
            writes(&fs.trace()[start..]),
            Vec::<&String>::new(),
            "{name}"
        );
        assert_eq!(ws.store().ops().unwrap().len(), ops, "{name}");
        assert!(files_on_disk(&*fs).is_empty(), "{name}");
    }
}

// ------------------------------------------------- reversibility corpora (T-009)

/// The tracked working copy on disk, as `(path, (content, mode))`, for a
/// repository with no ignored files.
fn disk_files(fs: &SimFs) -> common::Files {
    files_on_disk(fs)
        .into_iter()
        .map(|(path, (content, executable))| {
            let mode = if executable {
                codec::Mode::Executable
            } else {
                codec::Mode::Regular
            };
            (path, (content, mode))
        })
        .collect()
}

/// The files of the working copy that op `op`'s view names.
fn op_files(ws: &Ws, op: &HashRef) -> common::Files {
    let view_id = view_of(ws, op);
    let view = codec::View::from_envelope(&ws.store().get_object(&view_id).unwrap()).unwrap();
    tree_files(ws, &view.workspaces["default"])
}

/// 200 pairs of working copies that differ only in uncommitted content,
/// through `undo` and through `restore`: their snapshot ops never collapse,
/// and `restore` of each snapshot op gives back its pre-state exactly
/// (INT-0009 AC-2).
#[test]
fn two_hundred_pairs_stay_distinguishable() {
    let mut runs = 0;
    for pair in 0..200u64 {
        for restore in [false, true] {
            let mut snapshots = Vec::new();
            for side in ["x", "y"] {
                let (mut ws, fs) = fresh(pair);
                put(&fs, "base", b"shared");
                put(&fs, "f", b"committed");
                ws.snapshot().unwrap();
                ws.new_change().unwrap();
                let earlier = head(&ws);
                // Uncommitted: the sides differ in this content only.
                put(&fs, "f", format!("{side}{pair}").as_bytes());
                let pre = disk_files(&fs);
                let new = if restore {
                    ws.restore(&earlier.to_string()).unwrap().1
                } else {
                    ws.undo().unwrap().1
                };
                let snapshot = *ws.store().ops().unwrap()[&new].parents.first().unwrap();
                assert_eq!(op_files(&ws, &snapshot), pre, "pair {pair}");
                ws.restore(&snapshot.to_string()).unwrap();
                assert_eq!(
                    disk_files(&fs),
                    pre,
                    "pair {pair} {side}: recovered exactly"
                );
                snapshots.push(op_files(&ws, &snapshot));
                runs += 1;
            }
            assert_ne!(snapshots[0], snapshots[1], "pair {pair}: collapsed");
        }
    }
    println!("200 pairs: {runs} runs, 0 collapsed");
}

/// One step of the scripted history that covers every op type, with
/// file-directory swaps and an executable file whose content changes.
#[derive(Clone, Copy, Debug)]
enum Step {
    /// Edit, then snapshot.
    Edit(u8),
    New,
    Undo,
    /// Restore the op at this position in creation order.
    Restore(usize),
}

const SCRIPT: [Step; 10] = [
    Step::Edit(0),
    Step::New,
    Step::Edit(1),
    Step::Undo,
    Step::Edit(2),
    Step::Restore(3),
    Step::New,
    Step::Edit(3),
    Step::Undo,
    Step::Restore(1),
];

fn edit(fs: &SimFs, round: u8) {
    let exe = |on| {
        fs.make_executable(&Path::new(ROOT).join("run"), on)
            .unwrap()
    };
    match round {
        0 => {
            put(fs, "a", b"a file");
            put(fs, "run", b"#!1");
            exe(true);
            put(fs, "d/x", b"x");
        }
        1 => {
            // `a` becomes a directory; `run` changes content, keeps its bit.
            delete(fs, "a");
            put(fs, "a/inner", b"inside");
            put(fs, "run", b"#!2");
            exe(true);
        }
        2 => {
            // `d` becomes a file; `run` loses its bit.
            delete(fs, "d/x");
            fs.sync_dir(&Path::new(ROOT).join("d")).unwrap();
            fs.remove_dir(&Path::new(ROOT).join("d")).unwrap();
            fs.sync_dir(Path::new(ROOT)).unwrap();
            put(fs, "d", b"d file");
            put(fs, "run", b"#!3");
            exe(false);
        }
        _ => {
            put(fs, "new", b"n");
            put(fs, "run", b"#!4");
            exe(true);
        }
    }
}

/// Replays the first `n` steps of [`SCRIPT`] on a fresh repository.
fn replay(n: usize) -> (Ws, Rc<SimFs>) {
    let (mut ws, fs) = fresh(100);
    for step in &SCRIPT[..n] {
        match *step {
            Step::Edit(round) => {
                edit(&fs, round);
                ws.snapshot().unwrap();
            }
            Step::New => {
                ws.new_change().unwrap();
            }
            Step::Undo => {
                ws.undo().unwrap();
            }
            Step::Restore(k) => {
                let ops = ws.log().unwrap();
                let id = ops[ops.len() - 1 - k].id;
                ws.restore(&id.to_string()).unwrap();
            }
        }
    }
    (ws, fs)
}

/// Every op of a history covering every op type, restored: the new head
/// names that op's exact view, and the working copy equals its tree byte for
/// byte, with modes (INT-0009 AC-3).
#[test]
fn restore_reaches_every_earlier_state() {
    let (mut ws, fs) = replay(SCRIPT.len());
    let ops: Vec<HashRef> = ws.log().unwrap().into_iter().map(|e| e.id).collect();
    assert!(ops.len() >= 10, "{}", ops.len());
    for op in &ops {
        let (_, new) = ws.restore(&op.to_string()).unwrap();
        assert_eq!(view_of(&ws, &new), view_of(&ws, op));
        assert_eq!(disk_files(&fs), op_files(&ws, op), "op {op}");
    }
}

/// `undo` right after each op type but `init`: back to the view of the
/// op's parent, exactly (INT-0009 AC-3; `undo` right after `init` is
/// `undo_on_the_root_op_fails`).
#[test]
fn undo_after_every_op_type() {
    let mut seen = std::collections::BTreeSet::new();
    for n in 1..=SCRIPT.len() {
        let (mut ws, fs) = replay(n);
        let h = head(&ws);
        let parent = *ws.store().ops().unwrap()[&h].parents.first().unwrap();
        let (restored, new) = ws.undo().unwrap();
        assert_eq!(restored, parent);
        let after = SCRIPT[n - 1];
        assert_eq!(view_of(&ws, &new), view_of(&ws, &parent), "after {after:?}");
        assert_eq!(disk_files(&fs), op_files(&ws, &parent), "after {after:?}");
        seen.insert(match after {
            Step::Edit(_) => "snapshot",
            Step::New => "new",
            Step::Undo => "undo",
            Step::Restore(_) => "restore",
        });
    }
    assert_eq!(seen.len(), 4, "snapshot, new, undo and restore");
}

/// Random command sequences: no command removes an op (INT-0009 AC-1).
#[test]
fn ops_are_never_removed() {
    for seed in 0..100 {
        let (mut ws, fs) = fresh(seed);
        let mut rng = sim::Rng::new(seed);
        let mut known = std::collections::BTreeSet::new();
        for step in 0..15 {
            match rng.below(5) {
                0 => {
                    let name = ["a", "b", "c/d"][rng.below(3) as usize];
                    put(&fs, name, &rng.next_u64().to_le_bytes());
                    ws.snapshot().unwrap();
                }
                1 => {
                    ws.new_change().unwrap();
                }
                2 => match ws.undo() {
                    Ok(_) | Err(Error::NothingToUndo) => {}
                    Err(error) => panic!("seed {seed} step {step}: undo: {error}"),
                },
                3 => {
                    let ops = ws.log().unwrap();
                    let id = ops[rng.below(ops.len() as u64) as usize].id;
                    ws.restore(&id.to_string()).unwrap();
                }
                _ => {
                    ws.status().unwrap();
                }
            }
            // Checked through `log`, what a user sees, and against the
            // store's own op files (test critique C-006).
            let now: std::collections::BTreeSet<HashRef> = ws
                .log()
                .unwrap()
                .into_iter()
                .map(|entry| entry.id)
                .collect();
            let stored: std::collections::BTreeSet<HashRef> =
                ws.store().ops().unwrap().into_keys().collect();
            assert_eq!(now, stored, "seed {seed} step {step}: log is not every op");
            assert!(
                now.is_superset(&known),
                "seed {seed} step {step}: an op was removed"
            );
            known = now;
        }
    }
}

/// Every command that snapshots reports what it left alone, once, in
/// `status`'s form and order (Sprint 4 T-009, backlog T-019; INT-0004
/// AC-7). Each command runs on a freshly opened `Workspace` that has
/// reported nothing yet, so its report can only come from its own snapshot;
/// two names make the order checked (test critique C-005).
#[test]
fn snapshotting_commands_return_reports() {
    let (mut ws, fs) = fresh(41);
    for raw in [vec![b'r', 0xff], vec![b'q', 0xfe]] {
        fs.make_raw(Path::new(ROOT), sim::RawName::Bytes(raw), b"x")
            .unwrap();
    }
    put(&fs, "a", b"1");
    let expected = ws.status().unwrap().reported;
    assert_eq!(expected.len(), 2);
    assert_eq!(ws.reported(), expected);
    let reopen = |seed: u64| {
        let ws = Workspace::open(Rc::clone(&fs), Path::new(ROOT), seed).unwrap();
        assert!(ws.reported().is_empty());
        ws
    };
    let mut ws = reopen(1);
    ws.new_change().unwrap();
    assert_eq!(ws.reported(), expected, "new_change");
    put(&fs, "a", b"2");
    let mut ws = reopen(2);
    ws.log().unwrap();
    assert_eq!(ws.reported(), expected, "log");
    let mut ws = reopen(3);
    let (undone, _) = ws.undo().unwrap();
    // `undo` snapshots twice; its report is one, not two.
    assert_eq!(ws.reported(), expected, "undo");
    let mut ws = reopen(4);
    ws.restore(&undone.to_string()[7..19]).unwrap();
    assert_eq!(ws.reported(), expected, "restore");
}

/// A command that fails after its snapshot still reports what it left
/// alone: `undo` with nothing to undo, and `restore` of an unknown op, of a
/// view with a hazard, and over a blocking entry (test critique C-006).
#[test]
fn failed_commands_carry_reports() {
    let raw = |fs: &SimFs| {
        fs.make_raw(Path::new(ROOT), sim::RawName::Bytes(vec![0xfe]), b"x")
            .unwrap();
    };
    let carries = |ws: &Ws, what: &str| {
        assert_eq!(ws.reported().len(), 1, "{what}");
        assert!(ws.reported()[0].path.contains("\\xfe"), "{what}");
    };

    let (mut ws, fs) = fresh(42);
    raw(&fs);
    assert!(matches!(ws.undo(), Err(Error::NothingToUndo)));
    carries(&ws, "nothing to undo");

    let (mut ws, fs) = fresh(43);
    raw(&fs);
    assert!(matches!(
        ws.restore("0000000000000000"),
        Err(Error::UnknownOp(_))
    ));
    carries(&ws, "unknown op");

    // A hazard: two names that fold together on a case-insensitive
    // filesystem.
    let (mut ws, fs) = fresh(44);
    put(&fs, "README", b"x");
    put(&fs, "readme", b"x");
    ws.snapshot().unwrap();
    let target = head(&ws);
    delete(&fs, "README");
    delete(&fs, "readme");
    ws.snapshot().unwrap();
    fs.set_name_rules(NameRules {
        case_insensitive: true,
        windows_names: false,
    });
    raw(&fs);
    assert!(matches!(ws.restore(&hex(&target)), Err(Error::Hazards(_))));
    carries(&ws, "hazard");

    // A blocking entry: a tracked name, deleted, then ignored and recreated
    // with the user's own content.
    let (mut ws, fs) = fresh(45);
    put(&fs, "b", b"tracked");
    ws.snapshot().unwrap();
    let target = head(&ws);
    delete(&fs, "b");
    put(&fs, ".gitignore", b"b\n");
    ws.snapshot().unwrap();
    put(&fs, "b", b"mine");
    raw(&fs);
    assert!(matches!(ws.restore(&hex(&target)), Err(Error::Blocked(_))));
    carries(&ws, "blocking entry");
}

/// A snapshot that fails before its scan reports nothing, so a `Workspace`
/// used for a second command never shows the first command's entries as its
/// own (code review CR-07; test critique round 2, C-001).
#[test]
fn a_failed_snapshot_reports_nothing_stale() {
    let (mut ws, fs) = fresh(46);
    fs.make_raw(Path::new(ROOT), sim::RawName::Bytes(vec![0xfe]), b"x")
        .unwrap();
    ws.status().unwrap();
    assert_eq!(ws.reported().len(), 1);
    // Two ops on the head, written through a second store handle: the next
    // load stops with several heads, before any scan.
    let base = head(&ws);
    let view = view_of(&ws, &base);
    let mut other = store::Store::open(
        Rc::clone(&fs),
        &Path::new(ROOT).join(".praps").join("store"),
        hash::HashAlg::DEFAULT,
        47,
    )
    .unwrap();
    let mut second = codec::View::from_envelope(&other.get_object(&view).unwrap()).unwrap();
    second
        .workspaces
        .insert("other".to_owned(), second.workspaces["default"]);
    let second = other.put_object(&second.to_envelope()).unwrap();
    for view in [view, second] {
        other
            .commit_op(&codec::Operation {
                parents: [base].into(),
                view,
            })
            .unwrap();
    }
    assert!(matches!(ws.status(), Err(Error::SeveralHeads(h)) if h.len() == 2));
    assert!(ws.reported().is_empty(), "{:?}", ws.reported());
}
