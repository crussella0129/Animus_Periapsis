//! Undo is scoped (INT-0009 AC-6; Sprint 7 T-005). Over 200 seeded
//! interleavings of two lanes' commands, each `undo` and `restore` by one
//! lane leaves every entry the other owns as the merge left it just before,
//! and the other lane's files as they were.

mod common;

use std::convert::Infallible;
use std::path::Path;
use std::rc::Rc;

use codec::{View, WorkspaceEntry};
use common::{ROOT, put};
use sim::{Fs, Mode, Node, Rng, SimFs};
use workspace::{Error, Workspace};

type Ws = Workspace<Rc<SimFs>>;

const INTERLEAVINGS: u64 = 200;
const STEPS: u64 = 30;

/// Workspace `name`'s entry in the view the store's heads leave together.
fn merged_entry(ws: &Ws, name: &str) -> Option<WorkspaceEntry> {
    let ops = ws.store().ops().unwrap();
    let heads = store::heads_of(&ops);
    let Ok(view) = workspace::merge::merge(&ops, &heads, |id| {
        Ok::<_, Infallible>(View::from_envelope(&ws.store().get_object(id).unwrap()).unwrap())
    });
    view.workspaces.get(name).cloned()
}

/// The files of the lane at `dir`, but its `.praps`.
fn lane_files(fs: &SimFs, dir: &str) -> Vec<(String, Vec<u8>)> {
    fs.snapshot()
        .into_iter()
        .filter(|(path, _)| {
            path.starts_with(&format!("{dir}/")) && !path.starts_with(&format!("{dir}/.praps"))
        })
        .filter_map(|(path, node)| match node {
            Node::File(content) | Node::Executable(content) => Some((path, content)),
            _ => None,
        })
        .collect()
}

/// Writes `content` at `dir/name`, made or replaced.
fn write(fs: &SimFs, dir: &str, name: &str, content: &[u8]) {
    let path = Path::new(dir).join(name);
    match fs.create_new(&path) {
        Ok(()) => fs.write(&path, content).unwrap(),
        Err(_) => fs.overwrite(&path, content).unwrap(),
    }
}

#[test]
fn undo_is_scoped_over_200_interleavings() {
    let names = ["la", "lb"];
    let mut checked = 0;
    for seed in 0..INTERLEAVINGS {
        let fs = Rc::new(SimFs::new(seed, Mode::Lenient));
        let mut ws = Workspace::init(Rc::clone(&fs), Path::new(ROOT), seed)
            .unwrap()
            .0;
        put(&*fs, "a", b"1");
        ws.new_change().unwrap();
        for name in names {
            ws.make_lane(name, Path::new(name)).unwrap();
        }
        drop(ws);
        let mut rng = Rng::new(seed ^ 0x5c0_9ed);
        for step in 0..STEPS {
            let me = rng.below(2) as usize;
            let other = names[1 - me];
            // Each command opens its lane, as a `praps` process does.
            let mut lane: Ws =
                Workspace::open(Rc::clone(&fs), Path::new(names[me]), seed * 100 + step).unwrap();
            let entry_before = merged_entry(&lane, other);
            let files_before = lane_files(&fs, other);
            let undoing = match rng.below(5) {
                0 => {
                    let file = format!("f{}", rng.below(3));
                    write(&fs, names[me], &file, format!("{seed}-{step}").as_bytes());
                    lane.status().unwrap();
                    false
                }
                1 => {
                    lane.describe(format!("{seed}-{step}").as_bytes()).unwrap();
                    false
                }
                2 => {
                    lane.new_change().unwrap();
                    false
                }
                3 => {
                    match lane.undo() {
                        Ok(_) | Err(Error::NothingToUndo) => {}
                        Err(error) => panic!("seed {seed} step {step}: undo: {error}"),
                    }
                    true
                }
                _ => {
                    let ops: Vec<_> = lane.store().ops().unwrap().into_keys().collect();
                    let op = ops[rng.below(ops.len() as u64) as usize];
                    match lane.restore(&op.to_string()) {
                        Ok(_) | Err(Error::NotInWorkspace(..)) => {}
                        Err(error) => panic!("seed {seed} step {step}: restore: {error}"),
                    }
                    true
                }
            };
            if undoing {
                let lane = Workspace::open(Rc::clone(&fs), Path::new(names[me]), 7).unwrap();
                assert_eq!(
                    merged_entry(&lane, other),
                    entry_before,
                    "seed {seed} step {step}: {other}'s entry"
                );
                assert_eq!(
                    lane_files(&fs, other),
                    files_before,
                    "seed {seed} step {step}: {other}'s files"
                );
                checked += 1;
            }
        }
    }
    println!("{checked} undo and restore commands checked over {INTERLEAVINGS} interleavings");
    assert!(checked >= 1_000, "{checked}");
}
