//! The lane matrix (INT-0005 AC-1 to AC-3; Sprint 7 T-006): a crash at
//! every syscall of making a lane, for 40 seeds in both directory modes.
//! `.praps/workspace` is a new durable structure, so the matrix has a
//! negative control of its own, `Variant::LaneFileInPlace`.
//!
//! In each cell the repository is made and edited uncrashed, then the crash
//! falls on one syscall of making lane `lane` at `lane` (or on none, after
//! the last). After recovery:
//! - where the lane's op landed, the next command at the lane finishes its
//!   files, and making the lane again refuses because the name is in use;
//! - where it did not, that command refuses, naming the interrupted lane,
//!   or finds no repository there, and making the lane again succeeds;
//! - after the last syscall the lane is complete;
//! - the store checks clean.

mod common;

use std::path::Path;
use std::rc::Rc;

use hash::{HashAlg, HashRef};
use sim::{Mode, Node, SimFs};
use store::Store;
use workspace::{Error, Variant, Workspace};

const SEEDS: u64 = 40;
const MODES: [Mode; 2] = [Mode::Lenient, Mode::Strict];
const ROOT: &str = "repo";
const STORE: &str = "repo/.praps/store";
const LANE: &str = "lane";

type Ws = Workspace<Rc<SimFs>>;

/// The repository the lane is made from: `a` = "1" recorded, then a change
/// on top of it with `a` = "2". The lane's files are the first change's.
fn setup(fs: &Rc<SimFs>, seed: u64) -> Ws {
    let mut ws = Workspace::init(Rc::clone(fs), Path::new(ROOT), seed)
        .unwrap()
        .0;
    common::put(&**fs, "a", b"1");
    ws.new_change().unwrap();
    common::put(&**fs, "a", b"2");
    ws.snapshot().unwrap();
    ws
}

/// Makes the lane, in `variant`.
fn make(ws: &mut Ws, variant: Variant) -> Result<(), Error> {
    ws.set_variant(variant);
    ws.make_lane(LANE, Path::new(LANE)).map(drop)
}

/// The lane's files outside its `.praps`.
fn lane_files(fs: &SimFs) -> Vec<(String, Vec<u8>)> {
    fs.snapshot()
        .into_iter()
        .filter(|(path, _)| path.starts_with("lane/") && !path.starts_with("lane/.praps"))
        .filter_map(|(path, node)| match node {
            Node::File(content) | Node::Executable(content) => Some((path, content)),
            _ => None,
        })
        .collect()
}

fn expected_files() -> Vec<(String, Vec<u8>)> {
    vec![("lane/a".to_owned(), b"1".to_vec())]
}

/// Whether an op made in the lane stands in the store.
fn landed(fs: &Rc<SimFs>) -> bool {
    let store = Store::open(Rc::clone(fs), Path::new(STORE), HashAlg::DEFAULT, 0).unwrap();
    store.ops().unwrap().values().any(|op| op.workspace == LANE)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Violations {
    /// The next command at the lane did not do what the cell allows.
    outcome: u64,
    /// The lane's files differ from its change, or its record is left.
    files: u64,
    /// Making the lane again did not do what the cell allows.
    again: u64,
    /// The store does not check clean.
    store: u64,
    /// After the last syscall, the lane is not complete.
    durability: u64,
}

impl Violations {
    fn total(&self) -> u64 {
        self.outcome + self.files + self.again + self.store + self.durability
    }
}

/// The oracle, on a copy of the recovered filesystem.
fn check(recovered: SimFs, seed: u64, last: bool) -> Violations {
    let fs = Rc::new(recovered);
    let mut v = Violations::default();
    let landed = landed(&fs);
    let opened = Workspace::open(Rc::clone(&fs), Path::new(LANE), seed + 1);
    let record_left = fs.snapshot().contains_key("lane/.praps/working-copy");
    match (landed, opened) {
        (true, Ok(_)) => {
            if lane_files(&fs) != expected_files() || record_left {
                v.files += 1;
            }
        }
        (false, Err(Error::UnfinishedLane(..) | Error::NotInitialised(_))) => {}
        _ => v.outcome += 1,
    }
    if last && !(landed && lane_files(&fs) == expected_files() && !record_left) {
        v.durability += 1;
    }
    match Workspace::open(Rc::clone(&fs), Path::new(ROOT), seed + 2) {
        Ok(mut ws) => match (landed, ws.make_lane(LANE, Path::new(LANE))) {
            (true, Err(Error::LaneExists(_))) => {}
            (false, Ok(_)) => {
                let finished = Workspace::open(Rc::clone(&fs), Path::new(LANE), seed + 3);
                if finished.is_err() || lane_files(&fs) != expected_files() {
                    v.again += 1;
                }
            }
            _ => v.again += 1,
        },
        Err(_) => v.again += 1,
    }
    let store = Store::open(Rc::clone(&fs), Path::new(STORE), HashAlg::DEFAULT, 0).unwrap();
    if !store.fsck().unwrap().is_clean() {
        v.store += 1;
    }
    v
}

fn digest(fs: &SimFs) -> HashRef {
    let mut bytes = Vec::new();
    for (path, node) in fs.snapshot() {
        bytes.extend_from_slice(path.as_bytes());
        bytes.push(0);
        match node {
            Node::Dir => bytes.push(b'd'),
            Node::File(content) | Node::Executable(content) => {
                bytes.push(b'f');
                bytes.extend_from_slice(&(content.len() as u64).to_le_bytes());
                bytes.extend_from_slice(&content);
            }
            Node::Symlink => bytes.push(b'l'),
            Node::Other => bytes.push(b'o'),
        }
    }
    HashAlg::Blake3.hash(&bytes)
}

struct Cell {
    mode: Mode,
    seed: u64,
    crash_at: u64,
    violations: Violations,
    state: HashRef,
    trace: HashRef,
}

fn matrix(variant: Variant) -> Vec<Cell> {
    let mut cells = Vec::new();
    for mode in MODES {
        for seed in 0..SEEDS {
            // Where making the lane starts and ends, uncrashed.
            let (from, to) = {
                let fs = Rc::new(SimFs::new(seed, mode));
                let mut ws = setup(&fs, seed);
                let from = fs.syscalls();
                make(&mut ws, variant).expect("an uncrashed making succeeds");
                (from, fs.syscalls())
            };
            for crash_at in from + 1..=to + 1 {
                let fs = Rc::new(SimFs::new(seed, mode));
                let mut ws = setup(&fs, seed);
                fs.set_crash_at(Some(crash_at));
                let _ = make(&mut ws, variant);
                drop(ws);
                let recovered = fs.recover();
                let violations = check(fs.recover(), seed, crash_at == to + 1);
                cells.push(Cell {
                    mode,
                    seed,
                    crash_at,
                    violations,
                    state: digest(&recovered),
                    trace: HashAlg::Blake3.hash(fs.trace().join("\n").as_bytes()),
                });
            }
        }
    }
    let total = cells.iter().fold(Violations::default(), |mut sum, cell| {
        sum.outcome += cell.violations.outcome;
        sum.files += cell.violations.files;
        sum.again += cell.violations.again;
        sum.store += cell.violations.store;
        sum.durability += cell.violations.durability;
        sum
    });
    println!(
        "lane matrix {variant:?}: {} cells; violations: {total:?}",
        cells.len()
    );
    cells
}

#[test]
fn a_crash_while_making_a_lane_is_finished_or_refused() {
    let cells = matrix(Variant::Correct);
    assert!(cells.iter().any(|c| c.mode == Mode::Strict));
    assert!(cells.iter().any(|c| c.mode == Mode::Lenient));
    let failing: Vec<_> = cells
        .iter()
        .filter(|cell| cell.violations.total() > 0)
        .map(|cell| (cell.mode, cell.seed, cell.crash_at, cell.violations))
        .take(10)
        .collect();
    assert!(failing.is_empty(), "{failing:?}");
}

#[test]
fn lane_matrix_is_reproducible() {
    let first = matrix(Variant::Correct);
    let second = matrix(Variant::Correct);
    assert_eq!(first.len(), second.len());
    let mut all = Vec::new();
    for (a, b) in first.iter().zip(&second) {
        let at = (a.mode, a.seed, a.crash_at);
        assert_eq!(at, (b.mode, b.seed, b.crash_at));
        assert_eq!(a.state, b.state, "cell {at:?}");
        assert_eq!(a.trace, b.trace, "cell {at:?}");
        assert_eq!(a.violations, b.violations, "cell {at:?}");
        all.extend_from_slice(&a.state.to_bytes());
        all.extend_from_slice(&a.trace.to_bytes());
    }
    let digest = HashAlg::Blake3.hash(&all);
    println!("lane matrix digest over {} cells: {digest}", first.len());
    assert_eq!(first.len(), CELLS);
    assert_eq!(digest.to_string(), DIGEST);
}

const CELLS: usize = 5_760;
const DIGEST: &str = "blake3:e67e05ecad31d03c26475a742d45eb9e9d40a1f9e2c06f278ffee3ff3b501724";

#[test]
fn lane_matrix_catches_a_file_written_in_place() {
    let cells = matrix(Variant::LaneFileInPlace);
    let caught = cells
        .iter()
        .filter(|cell| cell.violations.total() > 0)
        .count();
    println!("LaneFileInPlace: {caught} of {} cells caught", cells.len());
    assert!(caught > 0);
}
