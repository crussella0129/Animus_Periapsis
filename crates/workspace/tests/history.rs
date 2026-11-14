//! `ops`, `log`, `undo`, `restore` and `describe` on the simulator (INT-0009
//! AC-1 to AC-3; Sprint 3 T-007; Sprint 6 T-009 and T-010), and the
//! reversibility corpora over every op type (Sprint 3 T-009).

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
fn ops_list_newest_first() {
    let (mut ws, fs) = fresh(1);
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    ws.new_change().unwrap();
    // An edit not yet snapshotted: `ops` snapshots first.
    put(&fs, "a", b"2");
    let log = ws.ops().unwrap();
    assert_eq!(
        log.len(),
        4,
        "root, snapshot, new, and the listing's own snapshot"
    );
    assert_eq!(log[0].id, head(&ws));
    for pair in log.windows(2) {
        assert_eq!(
            pair[0].parents,
            [pair[1].id].into(),
            "children before parents"
        );
    }
    assert!(log[3].parents.is_empty(), "the root op is last");
    // Each entry counts the paths its working-copy change changes: the new
    // change with `a` modified, the new change while still empty, the first
    // change with `a` added, and the root's empty change.
    assert_eq!(
        log.iter().map(|entry| entry.paths).collect::<Vec<_>>(),
        [1, 0, 1, 0]
    );
    let (wc, change_id, _) = ws.working_copy().unwrap();
    assert_eq!((log[0].working_copy, log[0].change_id), (wc, change_id));
    // A second listing with nothing changed adds no op.
    assert_eq!(ws.ops().unwrap().len(), 4);
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
    let [wc] = view.workspaces["default"]
        .working_copy
        .iter()
        .collect::<Vec<_>>()[..]
    else {
        panic!("a pointer of one changeset");
    };
    tree_files(ws, wc)
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
    /// Describe the working-copy change.
    Describe(&'static [u8]),
}

const SCRIPT: [Step; 13] = [
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
    // Added in Sprint 6, after the steps above so that the positions
    // `Restore` names stay what they were.
    Step::Describe(b"Add the runner"),
    Step::Edit(3),
    Step::Describe(b"Add the runner\n\nIt \xff runs.\n"),
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
                let ops = ws.ops().unwrap();
                let id = ops[ops.len() - 1 - k].id;
                ws.restore(&id.to_string()).unwrap();
            }
            Step::Describe(text) => {
                assert!(ws.describe(text).unwrap().op.is_some());
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
    let ops: Vec<HashRef> = ws.ops().unwrap().into_iter().map(|e| e.id).collect();
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
            Step::Describe(_) => "describe",
        });
    }
    assert_eq!(seen.len(), 5, "snapshot, new, undo, restore and describe");
}

/// Random command sequences: no command removes an op (INT-0009 AC-1).
#[test]
fn ops_are_never_removed() {
    for seed in 0..100 {
        let (mut ws, fs) = fresh(seed);
        let mut rng = sim::Rng::new(seed);
        let mut known = std::collections::BTreeSet::new();
        for step in 0..15 {
            match rng.below(6) {
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
                    let ops = ws.ops().unwrap();
                    let id = ops[rng.below(ops.len() as u64) as usize].id;
                    ws.restore(&id.to_string()).unwrap();
                }
                4 => {
                    // One of three texts, so that some repeats change nothing.
                    let text = [&b"one"[..], b"two\nlines", b""][rng.below(3) as usize];
                    ws.describe(text).unwrap();
                }
                _ => {
                    ws.status().unwrap();
                }
            }
            // Checked through `ops`, what a user sees, and against the
            // store's own op files (test critique C-006).
            let now: std::collections::BTreeSet<HashRef> = ws
                .ops()
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
    ws.ops().unwrap();
    assert_eq!(ws.reported(), expected, "ops");
    put(&fs, "a", b"3");
    let mut ws = reopen(5);
    ws.changes().unwrap();
    assert_eq!(ws.reported(), expected, "changes");
    let mut ws = reopen(3);
    let (undone, _) = ws.undo().unwrap();
    // `undo` snapshots once, so its report is one.
    assert_eq!(ws.reported(), expected, "undo");
    let mut ws = reopen(4);
    ws.restore(&undone.to_string()[7..19]).unwrap();
    assert_eq!(ws.reported(), expected, "restore");
    put(&fs, "a", b"4");
    let mut ws = reopen(6);
    assert!(ws.describe(b"why").unwrap().op.is_some());
    assert_eq!(ws.reported(), expected, "describe");
    // And when the text is the one the change has, and nothing is written.
    let mut ws = reopen(7);
    assert!(ws.describe(b"why").unwrap().op.is_none());
    assert_eq!(ws.reported(), expected, "describe, the same text");
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
    // A damaged op file, written around the store: the next load stops on
    // it, before any scan. (Until Sprint 7, two heads stopped it there.)
    let junk = b"not an operation";
    let name = store::names::file_name(&hash::HashAlg::DEFAULT.hash(junk));
    let path = Path::new(ROOT).join(".praps/store/ops").join(name);
    fs.create_new(&path).unwrap();
    fs.write(&path, junk).unwrap();
    assert!(matches!(ws.status(), Err(Error::Corrupt(_))));
    assert!(ws.reported().is_empty(), "{:?}", ws.reported());
}

/// `undo` and `restore` record the working copy once, just before they plan
/// the write: a tracked file the move leaves as it is, which only a
/// snapshot's scan reads, is read once. An edit made before the command is
/// still in an op of its own before the op that moves the working copy, and
/// `restore` brings it back (Sprint 6 T-005; INT-0009 AC-2).
#[test]
fn undo_and_restore_read_an_untouched_file_once() {
    let (mut ws, fs) = fresh(60);
    put(&fs, "keep/k", b"kept");
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    put(&fs, "a", b"2");
    ws.snapshot().unwrap();
    let reads_of_kept = |from: usize| {
        fs.trace()[from..]
            .iter()
            .filter(|line| *line == "read repo/keep/k")
            .count()
    };

    put(&fs, "a", b"edited before undo");
    let start = fs.trace().len();
    let (_, new) = ws.undo().unwrap();
    assert_eq!(reads_of_kept(start), 1, "undo scans once");
    assert_eq!(content(&fs, "a").unwrap(), b"1");
    assert_eq!(content(&fs, "keep/k").unwrap(), b"kept");
    // The edit is in the op before the one that moved the working copy.
    let before_undo = *ws.store().ops().unwrap()[&new].parents.first().unwrap();

    put(&fs, "a", b"edited before restore");
    let start = fs.trace().len();
    let (_, new) = ws.restore(&hex(&before_undo)).unwrap();
    assert_eq!(reads_of_kept(start), 1, "restore scans once");
    assert_eq!(content(&fs, "a").unwrap(), b"edited before undo");
    let before_restore = *ws.store().ops().unwrap()[&new].parents.first().unwrap();
    ws.restore(&hex(&before_restore)).unwrap();
    assert_eq!(content(&fs, "a").unwrap(), b"edited before restore");

    // `restore_view`, called by itself, still records the working copy
    // first, once.
    put(&fs, "a", b"edited before restore_view");
    let view = view_of(&ws, &before_undo);
    let start = fs.trace().len();
    let new = ws.restore_view(&view).unwrap();
    assert_eq!(reads_of_kept(start), 1, "restore_view scans once");
    let recorded = *ws.store().ops().unwrap()[&new].parents.first().unwrap();
    ws.restore(&hex(&recorded)).unwrap();
    assert_eq!(content(&fs, "a").unwrap(), b"edited before restore_view");
}

/// `changes` lists the working-copy change and the changes before it along
/// first parents, newest first, each with its change ID and the number of
/// paths it changes. Like every command that opens the working copy, it
/// records it first: an edit no command has recorded yet is counted
/// (Sprint 6 T-009).
#[test]
fn changes_follow_first_parents_newest_first() {
    let (mut ws, fs) = fresh(61);
    let mut ids = vec![ws.working_copy().unwrap().1];
    for round in 0..3u8 {
        put(&fs, "a", &[b'0' + round]);
        put(&fs, &format!("f{round}"), b"new");
        ws.snapshot().unwrap();
        ids.push(ws.new_change().unwrap());
    }
    // An edit that no command has recorded yet.
    delete(&fs, "f0");
    let ops_before = ws.store().ops().unwrap().len();
    let changes = ws.changes().unwrap();
    assert_eq!(
        ws.store().ops().unwrap().len(),
        ops_before + 1,
        "the listing recorded the working copy first"
    );
    ids.reverse();
    assert_eq!(
        changes
            .iter()
            .map(|change| change.change_id)
            .collect::<Vec<_>>(),
        ids,
        "the working-copy change first, the root change last"
    );
    // The working-copy change deletes `f0`; each change before it changed
    // `a` and added one file.
    assert_eq!(
        changes
            .iter()
            .map(|change| change.paths)
            .collect::<Vec<_>>(),
        [1, 2, 2, 2]
    );
    assert!(changes.iter().all(|change| change.description.is_empty()));
    let (wc, ..) = ws.working_copy().unwrap();
    assert_eq!(changes[0].id, wc);
    // Listing again records nothing and gives the same answer.
    let ops = ws.store().ops().unwrap().len();
    assert_eq!(ws.changes().unwrap(), changes);
    assert_eq!(ws.store().ops().unwrap().len(), ops);
}

/// A change's description is listed exactly as stored, whoever wrote it:
/// here the changeset is written through a second store handle.
#[test]
fn changes_carry_each_description() {
    let (ws, fs) = fresh(62);
    let (root, ..) = ws.working_copy().unwrap();
    let mut store = store::Store::open(
        Rc::clone(&fs),
        &Path::new(ROOT).join(".praps").join("store"),
        hash::HashAlg::DEFAULT,
        63,
    )
    .unwrap();
    let text = b"Add the parser\n\nIt reads \xff one line at a time.\n".to_vec();
    let described = codec::Changeset {
        parents: vec![root],
        change_id: [5; 16],
        changes: Default::default(),
        description: text.clone(),
    };
    let described = store.put_object(&described.to_envelope()).unwrap();
    let view = codec::View {
        workspaces: [(
            "default".to_owned(),
            codec::WorkspaceEntry {
                working_copy: [described].into(),
                heads: [described].into(),
            },
        )]
        .into(),
    };
    let view = store.put_object(&view.to_envelope()).unwrap();
    store
        .commit_op(&codec::Operation {
            parents: [head(&ws)].into(),
            workspace: "default".to_owned(),
            view,
        })
        .unwrap();
    let mut ws = Workspace::open(Rc::clone(&fs), Path::new(ROOT), 64).unwrap();
    let changes = ws.changes().unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].description, text);
    assert_eq!(changes[0].change_id, [5; 16]);
    assert!(changes[1].description.is_empty());
    // The operation list names the same changeset for the head.
    let ops = ws.ops().unwrap();
    assert_eq!(ops[0].working_copy, described);
    assert_eq!((ops[0].change_id, ops[0].paths), ([5; 16], 0));
}

/// The syscalls one listing makes after `steps` steps of: edit one small
/// file, snapshot, start a new change. Each step adds two ops and one
/// change, and every change sits on top of the last.
fn listing_cost(steps: usize, list: fn(&mut Ws) -> usize) -> (usize, u64) {
    let (mut ws, fs) = fresh(70);
    for step in 0..steps {
        put(&fs, "f", format!("{step}").as_bytes());
        ws.snapshot().unwrap();
        ws.new_change().unwrap();
    }
    let before = fs.syscalls();
    let entries = list(&mut ws);
    (entries, fs.syscalls() - before)
}

/// The cost of a listing follows its length: the syscalls `ops` makes grow
/// by the same amount, within a tenth, from 50 to 100 operations as from
/// 100 to 150, and so do those of `changes` over 50, 100 and 150 changes.
/// Before Sprint 6's T-009 each line of the operation list read the whole
/// chain of changes, and the second growth was far larger than the first
/// (backlog T-018; the run at T-008's commit is in the sprint's unit test
/// record).
#[test]
fn listing_cost_grows_in_a_straight_line() {
    let straight = |what: &str, costs: [u64; 3]| {
        let [a, b, c] = costs;
        let (first, second) = (b - a, c - b);
        println!("{what}: {a}, {b}, {c} syscalls; growth {first} then {second}");
        assert!(
            second * 10 <= first * 11 && second * 10 >= first * 9,
            "{what}: the cost does not grow in a straight line: {first} then {second}"
        );
    };
    let ops = [25, 50, 75].map(|steps| listing_cost(steps, |ws| ws.ops().unwrap().len()));
    assert_eq!(ops.map(|(entries, _)| entries), [51, 101, 151]);
    straight("ops", ops.map(|(_, cost)| cost));
    let changes = [49, 99, 149].map(|steps| listing_cost(steps, |ws| ws.changes().unwrap().len()));
    assert_eq!(changes.map(|(entries, _)| entries), [50, 100, 150]);
    straight("changes", changes.map(|(_, cost)| cost));
}

// ------------------------------------------------------ `describe` (Sprint 6 T-010)

/// Whether a trace line's path is `.praps` or lies inside it.
fn in_praps(line: &str) -> bool {
    line.split_once(' ')
        .is_some_and(|(_, path)| path == "repo/.praps" || path.starts_with("repo/.praps/"))
}

/// The working-copy changeset as stored.
fn stored_change(ws: &Ws) -> (HashRef, codec::Changeset) {
    let (id, ..) = ws.working_copy().unwrap();
    let changeset = codec::Changeset::from_envelope(&ws.store().get_object(&id).unwrap()).unwrap();
    (id, changeset)
}

/// `describe` leaves the working-copy change with the same change ID,
/// parents and changes and with the text's bytes, whatever they are, as its
/// description, in one op over the head. No working-copy file is written.
#[test]
fn describe_sets_the_description() {
    let (mut ws, fs) = fresh(80);
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    ws.new_change().unwrap();
    put(&fs, "a", b"2");
    put(&fs, "b", b"new");
    ws.snapshot().unwrap();
    let before_head = head(&ws);
    let (before_id, before) = stored_change(&ws);
    assert!(before.description.is_empty());
    let text = b"Change a, add b\n\nNot UTF-8: \xff\xfe. A NUL: \0.\r\n";
    let disk = fs.snapshot();
    let start = fs.trace().len();
    let described = ws.describe(text).unwrap();
    let op = described.op.expect("an op sets the description");
    assert_eq!(described.change_id, before.change_id);
    assert_eq!(head(&ws), op);
    let ops = ws.store().ops().unwrap();
    assert_eq!(
        ops[&op].parents,
        [before_head].into(),
        "one op, over the head"
    );
    let (id, after) = stored_change(&ws);
    assert_ne!(id, before_id);
    assert_eq!(after.description, text);
    assert_eq!(
        (after.change_id, &after.parents, &after.changes),
        (before.change_id, &before.parents, &before.changes)
    );
    // The view names the new changeset in place of the old one.
    let view = codec::View::from_envelope(&ws.store().get_object(&ops[&op].view).unwrap()).unwrap();
    assert_eq!(view.workspaces["default"].heads, [id].into());
    assert_eq!(view.workspaces["default"].working_copy, [id].into());
    // Nothing outside `.praps` was written, and no file there changed.
    assert!(
        writes(&fs.trace()[start..])
            .iter()
            .all(|line| in_praps(line)),
        "{:?}",
        writes(&fs.trace()[start..])
    );
    let outside = |files: std::collections::BTreeMap<String, sim::Node>| {
        files
            .into_iter()
            .filter(|(path, _)| !path.starts_with("repo/.praps"))
            .collect::<Vec<_>>()
    };
    assert_eq!(outside(fs.snapshot()), outside(disk));
    // The listing shows it, on the working-copy change only.
    let changes = ws.changes().unwrap();
    assert_eq!(changes[0].description, text);
    assert!(
        changes[1..]
            .iter()
            .all(|change| change.description.is_empty())
    );
}

/// Like every command that opens the working copy, `describe` records it
/// first: an edit no command has recorded is in an op of its own, before
/// the op that sets the description, and the described change holds it.
#[test]
fn describe_records_the_working_copy_first() {
    let (mut ws, fs) = fresh(81);
    put(&fs, "a", b"1");
    ws.snapshot().unwrap();
    let before = head(&ws);
    put(&fs, "a", b"edited");
    put(&fs, "b", b"added");
    let op = ws.describe(b"Edit a and add b").unwrap().op.unwrap();
    let ops = ws.store().ops().unwrap();
    assert_eq!(
        ops.len(),
        4,
        "root, snapshot, the recording, the description"
    );
    let recording = *ops[&op].parents.first().unwrap();
    assert_eq!(ops[&recording].parents, [before].into());
    // The recording holds the edit under the old, empty description.
    let files = op_files(&ws, &recording);
    assert_eq!(files["a"].0, b"edited");
    assert_eq!(files["b"].0, b"added");
    // So does the described change.
    assert_eq!(op_files(&ws, &op), files);
    let (_, change) = stored_change(&ws);
    assert_eq!(change.description, b"Edit a and add b");
    assert_eq!(change.changes.len(), 2);
}

/// With the description the change already has, `describe` writes no op of
/// its own: nothing at all when the working copy is unchanged, and only the
/// recording of an edit otherwise. An empty text on a change with no
/// description is the same case.
#[test]
fn describe_with_the_same_text_writes_nothing() {
    let (mut ws, fs) = fresh(82);
    put(&fs, "a", b"1");
    let start = fs.trace().len();
    let described = ws.describe(b"").unwrap();
    assert_eq!(described.op, None, "no description, and none given");
    assert_eq!(
        ws.store().ops().unwrap().len(),
        2,
        "only the edit was recorded"
    );
    assert!(!writes(&fs.trace()[start..]).is_empty());

    assert!(ws.describe(b"Add a").unwrap().op.is_some());
    let ops = ws.store().ops().unwrap().len();
    let start = fs.trace().len();
    let again = ws.describe(b"Add a").unwrap();
    assert_eq!(again.op, None);
    assert_eq!(again.change_id, described.change_id);
    assert_eq!(writes(&fs.trace()[start..]), Vec::<&String>::new());
    assert_eq!(ws.store().ops().unwrap().len(), ops);
    // Text that differs by one byte is another description.
    assert!(ws.describe(b"Add a\n").unwrap().op.is_some());
    // And an empty text removes it.
    assert!(ws.describe(b"").unwrap().op.is_some());
    assert!(stored_change(&ws).1.description.is_empty());
}

/// A later snapshot that records an edit keeps the description.
#[test]
fn a_snapshot_keeps_the_description() {
    let (mut ws, fs) = fresh(83);
    put(&fs, "a", b"1");
    ws.describe(b"Add a").unwrap();
    for (round, command) in ["snapshot", "status", "ops", "changes"].iter().enumerate() {
        put(&fs, "a", format!("edit {round}").as_bytes());
        let ops = ws.store().ops().unwrap().len();
        match *command {
            "snapshot" => drop(ws.snapshot().unwrap()),
            "status" => drop(ws.status().unwrap()),
            "ops" => drop(ws.ops().unwrap()),
            _ => drop(ws.changes().unwrap()),
        }
        assert_eq!(ws.store().ops().unwrap().len(), ops + 1, "{command}");
        let (_, change) = stored_change(&ws);
        assert_eq!(change.description, b"Add a", "{command}");
        assert_eq!(
            op_files(&ws, &head(&ws))["a"].0,
            format!("edit {round}").as_bytes()
        );
    }
}

/// `new` starts a change with no description, and the change before it keeps
/// its own.
#[test]
fn new_starts_without_a_description() {
    let (mut ws, fs) = fresh(84);
    put(&fs, "a", b"1");
    ws.describe(b"Add a").unwrap();
    let (described, _) = stored_change(&ws);
    ws.new_change().unwrap();
    let (_, new) = stored_change(&ws);
    assert!(new.description.is_empty());
    assert_eq!(new.parents, [described]);
    let changes = ws.changes().unwrap();
    assert_eq!(
        changes
            .iter()
            .map(|change| change.description.as_slice())
            .collect::<Vec<_>>(),
        [&b""[..], b"Add a"]
    );
    // Describing the new change leaves the one before it alone.
    ws.describe(b"Next").unwrap();
    let changes = ws.changes().unwrap();
    assert_eq!(changes[0].description, b"Next");
    assert_eq!(changes[1].description, b"Add a");
    assert_eq!(changes[1].id, described);
}

/// `undo` after `describe` gives back the description the change had, and
/// `restore` of the describing op gives the new one again. No working-copy
/// file changes in either (INT-0009 AC-3).
#[test]
fn describe_is_undone_and_restored() {
    let (mut ws, fs) = fresh(85);
    put(&fs, "a", b"1");
    put(&fs, "d/b", b"2");
    ws.describe(b"first").unwrap();
    let first = head(&ws);
    let described = ws.describe(b"second").unwrap().op.unwrap();
    let disk = disk_files(&fs);
    let untouched = |fs: &SimFs, from: usize, what: &str| {
        assert!(
            writes(&fs.trace()[from..])
                .iter()
                .all(|line| in_praps(line)),
            "{what}: {:?}",
            writes(&fs.trace()[from..])
        );
        assert_eq!(disk_files(fs), disk, "{what}");
    };

    let start = fs.trace().len();
    let (back_to, undo) = ws.undo().unwrap();
    assert_eq!(back_to, first);
    assert_eq!(stored_change(&ws).1.description, b"first");
    assert_eq!(view_of(&ws, &undo), view_of(&ws, &first));
    untouched(&fs, start, "undo");

    let start = fs.trace().len();
    let (target, restored) = ws.restore(&hex(&described)).unwrap();
    assert_eq!(target, described);
    assert_eq!(stored_change(&ws).1.description, b"second");
    assert_eq!(view_of(&ws, &restored), view_of(&ws, &described));
    untouched(&fs, start, "restore");

    // A second undo takes the restore back.
    ws.undo().unwrap();
    assert_eq!(stored_change(&ws).1.description, b"first");
}
