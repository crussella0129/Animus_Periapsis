//! Workspaces over one store, in the library (INT-0014 AC-3; Sprint 7
//! T-006): making a lane, opening one, and what a lane refuses.

mod common;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use codec::{Changeset, View};
use common::{ROOT, put};
use sim::{Fs, Kind, Mode as DirMode, SimFs};
use workspace::lane::{self, LaneFile};
use workspace::record;
use workspace::{Error, Workspace};

type Ws = Workspace<Arc<SimFs>>;

const STORE: &str = "repo/.praps/store";

/// A repository at `repo` whose working-copy change has a parent holding
/// `a` = "1", and itself `a` = "2" and `b` = "2": the files of a lane made
/// from it are the parent's.
fn repository(seed: u64) -> (Ws, Arc<SimFs>) {
    let fs = Arc::new(SimFs::new(seed, DirMode::Strict));
    let mut ws = Workspace::init(Arc::clone(&fs), Path::new(ROOT), seed)
        .unwrap()
        .0;
    put(&*fs, "a", b"1");
    ws.new_change().unwrap();
    put(&*fs, "a", b"2");
    put(&*fs, "b", b"2");
    ws.snapshot().unwrap();
    (ws, fs)
}

/// Every file under `dir`, but `.praps`, with its content.
fn files_in(fs: &impl Fs, dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut stack = vec![(dir.to_path_buf(), String::new())];
    while let Some((path, shown)) = stack.pop() {
        for entry in fs.scan(&path).unwrap() {
            let name = entry.name.as_utf8().unwrap().to_owned();
            if shown.is_empty() && name == ".praps" {
                continue;
            }
            let child = if shown.is_empty() {
                name.clone()
            } else {
                format!("{shown}/{name}")
            };
            match entry.kind {
                Kind::Dir => stack.push((path.join(&name), child)),
                Kind::File { .. } => {
                    files.insert(child, fs.read(&path.join(&name)).unwrap());
                }
                Kind::Symlink | Kind::Other => {}
            }
        }
    }
    files
}

/// The view `ws`'s store's single head leaves.
fn head_view(ws: &Ws) -> View {
    let ops = ws.store().ops().unwrap();
    let heads = store::heads_of(&ops);
    assert_eq!(heads.len(), 1);
    let view = ops[heads.first().unwrap()].view;
    View::from_envelope(&ws.store().get_object(&view).unwrap()).unwrap()
}

fn changeset(ws: &Ws, id: &hash::HashRef) -> Changeset {
    Changeset::from_envelope(&ws.store().get_object(id).unwrap()).unwrap()
}

/// The position in `trace` of the first line that starts with `prefix` and
/// holds `part`.
fn at(trace: &[String], prefix: &str, part: &str) -> usize {
    trace
        .iter()
        .position(|line| line.starts_with(prefix) && line.contains(part))
        .unwrap_or_else(|| panic!("no `{prefix}` of {part} in {trace:#?}"))
}

#[test]
fn a_lane_is_made_over_the_same_store() {
    let (mut ws, fs) = repository(1);
    let before = fs.trace().len();
    let (op, change_id) = ws.make_lane("lane", Path::new("lane")).unwrap();
    let trace = fs.trace()[before..].to_vec();
    // The steps, in order: the lane's directories, its file, the record, the
    // op, the files, and the record dropped.
    let dirs = at(&trace, "create_dir ", "lane/.praps");
    let file = at(&trace, "rename ", "lane/.praps/workspace.new");
    let record = at(&trace, "rename ", "lane/.praps/working-copy.new");
    let committed = at(&trace, "rename ", "repo/.praps/store/ops/");
    // A file is written under a temporary name, then renamed into place.
    let written = at(&trace, "rename ", "lane/.praps-tmp-");
    let dropped = trace
        .iter()
        .rposition(|line| line == "remove_file lane/.praps/working-copy")
        .expect("the record is dropped");
    assert!(
        dirs < file && file < record && record < committed,
        "{trace:#?}"
    );
    assert!(committed < written && written < dropped, "{trace:#?}");
    // Its `.praps` holds the lane's file and no store.
    assert_eq!(
        fs.list(Path::new("lane/.praps")).unwrap(),
        [lane::WORKSPACE_FILE]
    );
    assert_eq!(
        lane::read(&*fs, Path::new("lane/.praps")).unwrap().unwrap(),
        Ok(LaneFile {
            name: "lane".to_owned(),
            store: STORE.into(),
        })
    );
    // One op, made in the lane, over the head, adding its entry.
    let made = ws.store().ops().unwrap()[&op].clone();
    assert_eq!(made.workspace, "lane");
    let view = head_view(&ws);
    let entry = &view.workspaces["lane"];
    assert_eq!(entry.working_copy, entry.heads);
    let lane_change = *entry.working_copy.first().unwrap();
    assert_eq!(changeset(&ws, &lane_change).change_id, change_id);
    assert_eq!(files_in(&*fs, Path::new("lane")).len(), 1);
}

#[test]
fn a_lane_starts_from_the_parent_of_the_change() {
    let (mut ws, fs) = repository(2);
    let (wc, ..) = ws.working_copy().unwrap();
    let parent = changeset(&ws, &wc).parents[0];
    ws.make_lane("lane", Path::new("lane")).unwrap();
    let view = head_view(&ws);
    let lane_change = *view.workspaces["lane"].working_copy.first().unwrap();
    let made = changeset(&ws, &lane_change);
    assert_eq!(made.parents, [parent]);
    assert!(made.changes.is_empty());
    // The parent's tree: `a` = "1", without the edits in progress.
    assert_eq!(
        files_in(&*fs, Path::new("lane")),
        [("a".to_owned(), b"1".to_vec())].into()
    );
    // From a workspace whose change has no parent: the empty tree.
    let fs = Arc::new(SimFs::new(3, DirMode::Strict));
    let mut ws = Workspace::init(Arc::clone(&fs), Path::new(ROOT), 3)
        .unwrap()
        .0;
    put(&*fs, "a", b"1");
    ws.make_lane("empty", Path::new("empty")).unwrap();
    let view = head_view(&ws);
    let lane_change = *view.workspaces["empty"].working_copy.first().unwrap();
    assert!(changeset(&ws, &lane_change).parents.is_empty());
    assert!(files_in(&*fs, Path::new("empty")).is_empty());
}

#[test]
fn open_finds_the_store_a_lane_names() {
    let (mut ws, fs) = repository(4);
    ws.make_lane("lane", Path::new("lane")).unwrap();
    drop(ws);
    let mut lane = Workspace::open(Arc::clone(&fs), Path::new("lane"), 5).unwrap();
    assert_eq!(lane.store().root(), Path::new(STORE));
    // A command there writes the lane's entry, in the one store.
    fs.create_new(Path::new("lane/c")).unwrap();
    fs.write(Path::new("lane/c"), b"3").unwrap();
    let op = lane.snapshot().unwrap().op.expect("the edit is recorded");
    assert_eq!(lane.store().ops().unwrap()[&op].workspace, "lane");
    let view = head_view(&lane);
    let (wc, ..) = lane.working_copy().unwrap();
    assert_eq!(view.workspaces["lane"].working_copy, [wc].into());
    // The repository's own working copy is untouched by it.
    let ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 6).unwrap();
    assert_eq!(ws.working_copy().unwrap().2.len(), 2);
}

#[test]
fn lane_refusals_write_nothing() {
    let (mut ws, fs) = repository(7);
    ws.make_lane("lane", Path::new("lane")).unwrap();
    // A name in use.
    let before = fs.trace().len();
    match ws.make_lane("lane", Path::new("other")) {
        Err(Error::LaneExists(name)) => assert_eq!(name, "lane"),
        other => panic!("{:?}", other.err()),
    }
    assert!(common::writes(&fs.trace()[before..]).is_empty());
    // A directory that is not free.
    fs.create_dir(Path::new("full")).unwrap();
    fs.create_new(Path::new("full/x")).unwrap();
    let before = fs.trace().len();
    match ws.make_lane("full", Path::new("full")) {
        Err(Error::NotFree(dir)) => assert_eq!(dir, Path::new("full")),
        other => panic!("{:?}", other.err()),
    }
    assert!(common::writes(&fs.trace()[before..]).is_empty());
    // A directory holding only what an interrupted making left is free.
    fs.create_dir(Path::new("left")).unwrap();
    fs.create_dir(Path::new("left/.praps")).unwrap();
    fs.create_new(Path::new("left/.praps/workspace.new"))
        .unwrap();
    fs.create_new(Path::new("left/.praps/working-copy.new"))
        .unwrap();
    ws.make_lane("left", Path::new("left")).unwrap();
}

#[test]
fn an_interrupted_lane_is_refused_and_made_again() {
    let (ws, fs) = repository(8);
    drop(ws);
    // A lane whose file and record were written, but whose op never landed.
    fs.create_dir(Path::new("lane")).unwrap();
    fs.create_dir(Path::new("lane/.praps")).unwrap();
    let file = LaneFile {
        name: "lane".to_owned(),
        store: STORE.into(),
    };
    lane::write(&*fs, Path::new("lane/.praps"), &file, false).unwrap();
    let missing: hash::HashRef =
        "blake3:0000000000000000000000000000000000000000000000000000000000000000"
            .parse()
            .unwrap();
    record::write(
        &*fs,
        Path::new("lane/.praps"),
        &record::Record {
            from: None,
            target: missing,
        },
    )
    .unwrap();
    match Workspace::open(Arc::clone(&fs), Path::new("lane"), 9) {
        Err(Error::UnfinishedLane(name, dir)) => {
            assert_eq!((name.as_str(), dir.as_path()), ("lane", Path::new("lane")));
        }
        other => panic!("{:?}", other.err()),
    }
    // The record was dropped, as resume drops one whose target is absent,
    // and nothing else was written.
    assert_eq!(
        fs.list(Path::new("lane/.praps")).unwrap(),
        [lane::WORKSPACE_FILE]
    );
    // Making the lane again at that directory finishes it.
    let mut ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 10).unwrap();
    ws.make_lane("lane", Path::new("lane")).unwrap();
    Workspace::open(Arc::clone(&fs), Path::new("lane"), 11).unwrap();
}

#[test]
fn a_lane_whose_store_is_gone_is_refused() {
    let fs = Arc::new(SimFs::new(12, DirMode::Strict));
    fs.create_dir(Path::new("lane")).unwrap();
    fs.create_dir(Path::new("lane/.praps")).unwrap();
    let file = LaneFile {
        name: "lane".to_owned(),
        store: "gone/.praps/store".into(),
    };
    lane::write(&*fs, Path::new("lane/.praps"), &file, false).unwrap();
    match Workspace::open(Arc::clone(&fs), Path::new("lane"), 13) {
        Err(Error::StoreGone(store, _)) => assert_eq!(store, Path::new("gone/.praps/store")),
        other => panic!("{:?}", other.err()),
    }
}

#[test]
fn a_damaged_workspace_file_is_refused() {
    let fs = Arc::new(SimFs::new(14, DirMode::Strict));
    fs.create_dir(Path::new("lane")).unwrap();
    fs.create_dir(Path::new("lane/.praps")).unwrap();
    fs.create_new(Path::new("lane/.praps/workspace")).unwrap();
    fs.write(Path::new("lane/.praps/workspace"), b"name lane\n")
        .unwrap();
    match Workspace::open(Arc::clone(&fs), Path::new("lane"), 15) {
        Err(Error::BadWorkspaceFile(path, why)) => {
            assert_eq!(path, Path::new("lane/.praps/workspace"));
            assert!(why.contains("store"), "{why}");
        }
        other => panic!("{:?}", other.err()),
    }
}

#[test]
fn eight_lanes_share_one_store() {
    let (mut ws, fs) = repository(16);
    let names: Vec<String> = (0..8).map(|i| format!("lane-{i}")).collect();
    for name in &names {
        ws.make_lane(name, Path::new(name)).unwrap();
    }
    drop(ws);
    // Each lane runs an edit, `status`, `describe` and `new`, in turn.
    for (i, name) in names.iter().enumerate() {
        let mut lane = Workspace::open(Arc::clone(&fs), Path::new(name), 20 + i as u64).unwrap();
        let file = Path::new(name).join(format!("own-{i}"));
        fs.create_new(&file).unwrap();
        fs.write(&file, name.as_bytes()).unwrap();
        lane.status().unwrap();
        lane.describe(name.as_bytes()).unwrap();
        lane.new_change().unwrap();
    }
    // Every op is in the one store, and no lane holds a store of its own.
    for name in &names {
        let praps = Path::new(name).join(".praps");
        assert_eq!(fs.list(&praps).unwrap(), [lane::WORKSPACE_FILE], "{name}");
    }
    // Each object is one file: its name is its id.
    let objects = fs.list(Path::new("repo/.praps/store/objects")).unwrap();
    let unique: std::collections::BTreeSet<&String> = objects.iter().collect();
    assert_eq!(unique.len(), objects.len());
    // Each lane's entry holds its last command's state, and its files equal
    // its working-copy change.
    let ws = Workspace::open(Arc::clone(&fs), Path::new(ROOT), 30).unwrap();
    let view = head_view(&ws);
    for (i, name) in names.iter().enumerate() {
        let lane = Workspace::open(Arc::clone(&fs), Path::new(name), 40 + i as u64).unwrap();
        let (wc, _, tree) = lane.working_copy().unwrap();
        assert_eq!(view.workspaces[name].working_copy, [wc].into(), "{name}");
        let parent = changeset(&ws, &wc).parents[0];
        assert_eq!(changeset(&ws, &parent).description, name.as_bytes());
        let on_disk = files_in(&*fs, Path::new(name));
        assert_eq!(on_disk.len(), tree.len(), "{name}");
        assert_eq!(on_disk[&format!("own-{i}")], name.as_bytes());
    }
}
