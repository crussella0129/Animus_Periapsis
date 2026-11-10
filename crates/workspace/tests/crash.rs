//! The Snapshot crash scenario (INT-0005 AC-1 to AC-3; INT-0007 AC-1, AC-2;
//! INT-0014 AC-1; Sprint 2 T-007). `cargo test -- --nocapture` prints the
//! counts.
//!
//! One scenario: `init`, then four rounds of scripted edits each followed by
//! a snapshot, with a `new` after the second round. It runs with a crash at
//! every syscall (and after the last), for 40 seeds, in both directory modes.
//! On the state a restart sees, it checks:
//! - closure: `fsck` finds no fault (every op's parents and view, and every
//!   object an op reaches, present and whole);
//! - durability: every acknowledged op is present;
//! - recency: once `init` is acknowledged, the single head's working-copy
//!   change has the tree of the last acknowledged command, or of the one in
//!   flight; before that, the store may hold no op;
//! - liveness: `init` then `snapshot` succeed on the recovered state, and the
//!   working-copy change then holds exactly the recovered working copy.

mod common;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::rc::Rc;

use codec::Mode as FileMode;
use common::ROOT;
use hash::{HashAlg, HashRef};
use sim::{Fs, Mode, Node, SimFs};
use store::{Store, Variant};
use workspace::Workspace;

const SEEDS: u64 = 40;
const MODES: [Mode; 2] = [Mode::Lenient, Mode::Strict];
const STORE: &str = "repo/.praps/store";

type Files = BTreeMap<String, (Vec<u8>, FileMode)>;

/// What a run did and what was acknowledged.
#[derive(Clone, Debug, Default)]
struct Log {
    init_acked: bool,
    /// Every op a command returned.
    acked_ops: Vec<HashRef>,
    /// The working-copy tree each acknowledged command left, in order.
    acked: Vec<Files>,
    /// The tree the command in flight would leave.
    in_flight: Option<Files>,
}

/// The working copy as the scenario's edits leave it, for the model. The
/// scenario writes no `.gitignore`. Ignore matching is a pure read of files
/// the snapshot never writes, so a crash cannot change its answer; the
/// Git-recorded corpus and the edit corpora cover it (test critique round
/// 2, C-004).
///
/// It reads the simulator's snapshot, which is not a syscall, so building
/// the model moves no crash point.
fn model(fs: &SimFs) -> Files {
    fs.snapshot()
        .into_iter()
        .filter_map(|(path, node)| {
            let rest = path.strip_prefix("repo/")?;
            if rest.starts_with(".praps") {
                return None;
            }
            match node {
                Node::File(content) => Some((rest.to_owned(), (content, FileMode::Regular))),
                _ => None,
            }
        })
        .collect()
}

/// Writes a file under the repository, creating directories as needed. Its
/// syscalls may hit the crash point like any other, so errors propagate.
fn put(fs: &impl Fs, path: &str, content: &[u8]) -> io::Result<()> {
    let mut dir = Path::new(ROOT).to_path_buf();
    let mut parts: Vec<&str> = path.split('/').collect();
    let name = parts.pop().unwrap();
    for part in parts {
        dir = dir.join(part);
        match fs.create_dir(&dir) {
            Err(error) if error.kind() != io::ErrorKind::AlreadyExists => return Err(error),
            _ => {}
        }
    }
    let file = dir.join(name);
    match fs.create_new(&file) {
        Ok(()) => fs.write(&file, content),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => fs.overwrite(&file, content),
        Err(error) => Err(error),
    }
}

fn delete(fs: &impl Fs, path: &str) -> io::Result<()> {
    let file = path
        .split('/')
        .fold(Path::new(ROOT).to_path_buf(), |p, c| p.join(c));
    fs.remove_file(&file)
}

/// The four rounds of edits.
fn edit(fs: &impl Fs, round: usize) -> io::Result<()> {
    match round {
        0 => {
            put(fs, "a", b"alpha")?;
            put(fs, "src/lib.rs", b"pub fn f() {}")
        }
        1 => {
            put(fs, "a", b"alpha, edited")?;
            put(fs, "src/deep/b", b"beta")
        }
        2 => {
            delete(fs, "src/lib.rs")?;
            put(fs, "c", b"")
        }
        _ => {
            put(fs, "src/deep/b", b"beta, edited")?;
            delete(fs, "a")
        }
    }
}

/// The working-copy change's tree with each blob's content, or `None` if
/// the state cannot be read (a missing or damaged object).
fn recorded_tree<F: Fs>(ws: &Workspace<F>) -> Option<Files> {
    let (_, _, tree) = ws.working_copy().ok()?;
    let mut files = Files::new();
    for (path, file) in &tree {
        let envelope = ws.store().get_object(&file.blob).ok()?;
        if codec::object_id(HashAlg::DEFAULT, &envelope) != file.blob {
            return None;
        }
        files.insert(
            path.as_str().to_owned(),
            (envelope[2..].to_vec(), file.mode),
        );
    }
    Some(files)
}

fn run(
    fs: &Rc<SimFs>,
    variant: Variant,
    seed: u64,
    log: &RefCell<Log>,
) -> Result<(), workspace::Error> {
    let mut ws = if variant == Variant::Correct {
        Workspace::init(Rc::clone(fs), Path::new(ROOT), seed)?.0
    } else {
        // The same steps as `init`, with the store built in the variant.
        fs.create_dir(Path::new(ROOT))?;
        fs.sync_dir(Path::new(""))?;
        fs.create_dir(Path::new("repo/.praps"))?;
        fs.sync_dir(Path::new("repo/.praps"))?;
        fs.sync_dir(Path::new(ROOT))?;
        let store = Store::init_variant(
            Rc::clone(fs),
            Path::new(STORE),
            HashAlg::DEFAULT,
            seed,
            variant,
        )?;
        Workspace::init_with_store(store, Path::new(ROOT), seed)?.0
    };
    {
        let mut log = log.borrow_mut();
        log.init_acked = true;
        log.acked.push(Files::new());
    }
    // Reading the heads is itself syscalls, which may crash: the acknowledged
    // state is recorded first.
    let heads = ws.store().heads()?;
    log.borrow_mut().acked_ops.extend(heads);
    for round in 0..4 {
        edit(&**fs, round)?;
        log.borrow_mut().in_flight = Some(model(fs));
        if let Some(op) = ws.snapshot()?.op {
            log.borrow_mut().acked_ops.push(op);
        }
        let mut log_mut = log.borrow_mut();
        let done = log_mut.in_flight.take().unwrap();
        log_mut.acked.push(done.clone());
        drop(log_mut);
        if round == 1 {
            log.borrow_mut().in_flight = Some(done.clone());
            ws.new_change()?;
            {
                let mut log_mut = log.borrow_mut();
                log_mut.in_flight = None;
                log_mut.acked.push(done);
            }
            let heads = ws.store().heads()?;
            log.borrow_mut().acked_ops.extend(heads);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Violations {
    closure: u64,
    durability: u64,
    recency: u64,
    liveness: u64,
}

impl Violations {
    fn total(&self) -> u64 {
        self.closure + self.durability + self.recency + self.liveness
    }
}

/// The invariants, and the reach of the next command the scenario models
/// (`init` then `snapshot` on a copy), read before the oracle checks.
fn check(recovered: &SimFs, log: &Log, seed: u64) -> (Violations, Vec<(&'static str, u64)>) {
    let mut v = Violations::default();
    let root = Path::new(ROOT);
    match Store::inspect(recovered, Path::new(STORE), HashAlg::DEFAULT) {
        Ok(store) => {
            let report = store.fsck().unwrap();
            v.closure += report.problems.len() as u64;
            let ops = store.ops().unwrap();
            v.durability += log
                .acked_ops
                .iter()
                .filter(|id| !ops.contains_key(id))
                .count() as u64;
            if log.init_acked || !ops.is_empty() {
                match Workspace::init_with_store(
                    Store::inspect(recovered, Path::new(STORE), HashAlg::DEFAULT).unwrap(),
                    root,
                    seed,
                ) {
                    Ok((ws, false)) => {
                        let tree = recorded_tree(&ws);
                        let ok = tree.is_some()
                            && (log.acked.last() == tree.as_ref()
                                || log.in_flight.as_ref() == tree.as_ref()
                                || (!log.init_acked && tree.as_ref().is_some_and(Files::is_empty)));
                        if !ok {
                            v.recency += 1;
                        }
                    }
                    // A root op that never became durable: only before init
                    // was acknowledged.
                    Ok((_, true)) => v.recency += u64::from(log.init_acked),
                    Err(_) => v.recency += 1,
                }
            }
        }
        Err(_) => v.durability += u64::from(log.init_acked),
    }
    // Liveness, on a copy: `init` then `snapshot` work, and record the
    // recovered working copy exactly.
    let copy = recovered.recover();
    let mut next = Vec::new();
    let live = Workspace::init(&copy, root, seed).and_then(|(mut ws, _)| {
        ws.snapshot()?;
        next = copy.reach();
        Ok(recorded_tree(&ws) == Some(model(&copy)))
    });
    if !matches!(live, Ok(true)) {
        v.liveness += 1;
    }
    (v, next)
}

/// A hash of everything visible in a filesystem.
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
    /// The scenario's own reach and its next command's.
    reach: Vec<(&'static str, u64)>,
}

fn matrix(variant: Variant) -> Vec<Cell> {
    let mut cells = Vec::new();
    for mode in MODES {
        for seed in 0..SEEDS {
            let syscalls = {
                let fs = Rc::new(SimFs::new(seed, mode));
                let log = RefCell::new(Log::default());
                run(&fs, variant, seed, &log).expect("an uncrashed run succeeds");
                fs.syscalls()
            };
            for crash_at in 1..=syscalls + 1 {
                let fs = Rc::new(SimFs::new(seed, mode));
                fs.set_crash_at(Some(crash_at));
                let log = RefCell::new(Log::default());
                let _ = run(&fs, variant, seed, &log);
                let recovered = fs.recover();
                let (violations, next) = check(&fs.recover(), &log.borrow(), seed);
                let mut reach = fs.reach();
                reach.extend(next);
                cells.push(Cell {
                    mode,
                    seed,
                    crash_at,
                    violations,
                    state: digest(&recovered),
                    trace: HashAlg::Blake3.hash(fs.trace().join("\n").as_bytes()),
                    reach,
                });
            }
        }
    }
    let mut sums: BTreeMap<String, (u64, Violations)> = BTreeMap::new();
    for cell in &cells {
        let entry = sums.entry(format!("{:?}", cell.mode)).or_default();
        entry.0 += 1;
        entry.1.closure += cell.violations.closure;
        entry.1.durability += cell.violations.durability;
        entry.1.recency += cell.violations.recency;
        entry.1.liveness += cell.violations.liveness;
    }
    for (mode, (count, v)) in &sums {
        println!(
            "snapshot matrix {:<16} {mode}: {count} cells; violations: closure {} durability {} recency {} liveness {}",
            variant.name(),
            v.closure,
            v.durability,
            v.recency,
            v.liveness
        );
    }
    cells
}

#[test]
fn snapshot_matrix_finds_no_violations() {
    let cells = matrix(Variant::Correct);
    assert!(
        cells.iter().any(|c| c.mode == Mode::Strict)
            && cells.iter().any(|c| c.mode == Mode::Lenient)
    );
    let failing: Vec<_> = cells
        .iter()
        .filter(|c| c.violations.total() > 0)
        .take(5)
        .map(|c| (c.mode, c.seed, c.crash_at, c.violations))
        .collect();
    assert!(failing.is_empty(), "first violating cells: {failing:?}");
    // The designated record of this scenario's reach (INT-0005 AC-6).
    let mut totals = sim::reach::Totals::new();
    for cell in &cells {
        sim::reach::add(&mut totals, &cell.reach);
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("reach");
    sim::reach::record(&dir, "snapshot", "Snapshot", 2, &totals).unwrap();
}

#[test]
fn snapshot_matrix_is_reproducible() {
    let first = matrix(Variant::Correct);
    let second = matrix(Variant::Correct);
    assert_eq!(first.len(), second.len());
    let mut all = Vec::new();
    for (a, b) in first.iter().zip(&second) {
        assert_eq!((a.mode, a.seed, a.crash_at), (b.mode, b.seed, b.crash_at));
        assert_eq!(a.state, b.state, "cell {:?}", (a.mode, a.seed, a.crash_at));
        assert_eq!(a.trace, b.trace, "cell {:?}", (a.mode, a.seed, a.crash_at));
        // The outcome too: the same violations, cell by cell (C-006).
        assert_eq!(
            a.violations,
            b.violations,
            "cell {:?}",
            (a.mode, a.seed, a.crash_at)
        );
        all.extend_from_slice(&a.state.to_bytes());
        all.extend_from_slice(&a.trace.to_bytes());
    }
    let digest = HashAlg::Blake3.hash(&all);
    println!(
        "snapshot matrix digest over {} cells: {digest}",
        first.len()
    );
    // Asserted, not only printed, so a change that moves any cell fails
    // here on every system (Sprint 4 plan critique, C-008).
    assert_eq!(first.len(), 24_320);
    assert_eq!(
        digest.to_string(),
        "blake3:b6fca2372d26d8a0b40000df18c57b226eb976d4b9597d2deb2a95a629e84e1c"
    );
}

#[test]
fn snapshot_catches_unsynced_objects() {
    let cells = matrix(Variant::UnsyncedObjects);
    let strict: u64 = cells
        .iter()
        .filter(|c| c.mode == Mode::Strict)
        // Durability or closure, as the EARS clause names them (C-006).
        .map(|c| c.violations.closure + c.violations.durability)
        .sum();
    assert!(
        strict > 0,
        "the unsynced-objects variant produced no violation"
    );
}
