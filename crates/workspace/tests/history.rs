//! `log`, `undo` and `restore` on the simulator (INT-0009 AC-1 to AC-3;
//! Sprint 3 T-007). The corpora over every op type are in `reversible.rs`.

mod common;

use std::path::Path;
use std::rc::Rc;

use common::{ROOT, delete, files_on_disk, put, writes};
use hash::HashRef;
use sim::{Mode as DirMode, NameRules, SimFs};
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
