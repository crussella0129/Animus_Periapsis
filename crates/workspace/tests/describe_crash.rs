//! The Describe crash scenario (INT-0005 AC-1 to AC-3; INT-0014 AC-1;
//! Sprint 6 T-010). `cargo test -- --nocapture` prints the counts.
//!
//! One scenario: `init`; an edit and a snapshot; `describe`; an edit and a
//! snapshot, which must keep the description; a second `describe`, and the
//! same text again, which writes nothing; `new`, whose change starts with no
//! description; an edit left unrecorded and a third `describe`, which
//! records the edit first. It runs with a crash at every syscall (and after
//! the last), for 40 seeds, in both directory modes. On the state a restart
//! sees, it checks:
//! - closure: the store's check finds no fault;
//! - durability: every acknowledged op is present;
//! - recency: once `init` is acknowledged, the single head's working-copy
//!   change holds the files and the description of the last acknowledged
//!   command, or of a step of the command in flight; before that, the store
//!   may hold no op;
//! - liveness: on a copy, `init`, one more edit and a snapshot succeed, and
//!   the working-copy change then holds exactly the recovered working copy
//!   with that edit, under the description it had before the snapshot.
//!
//! Two controls must each be caught: a workspace whose snapshot drops the
//! description, and the store that does not sync objects.

mod common;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::rc::Rc;

use codec::{Changeset, Mode as FileMode};
use common::ROOT;
use hash::{HashAlg, HashRef};
use sim::{Fs, Mode, Node, SimFs};
use store::Store;
use workspace::{Variant, Workspace};

const SEEDS: u64 = 40;
const MODES: [Mode; 2] = [Mode::Lenient, Mode::Strict];
const STORE: &str = "repo/.praps/store";

/// The three descriptions the scenario gives: plain text, text of several
/// lines with a byte that is not UTF-8, and a single word.
const FIRST: &[u8] = b"Add alpha";
const SECOND: &[u8] = b"Add alpha and beta\n\nBoth are \xff placeholders.\n";
const THIRD: &[u8] = b"gamma";

type Files = BTreeMap<String, (Vec<u8>, FileMode)>;

/// What the working-copy change holds: its files and its description.
type Held = (Files, Vec<u8>);

/// What a run did and what was acknowledged.
#[derive(Clone, Debug, Default)]
struct Log {
    init_acked: bool,
    /// Every op a command returned.
    acked_ops: Vec<HashRef>,
    /// What the working-copy change held after the last acknowledged
    /// command.
    acked: Option<Held>,
    /// What it may hold while the command in flight runs: after each op
    /// that command commits.
    in_flight: Vec<Held>,
}

/// The working copy as the scenario's edits leave it, for the model. It
/// reads the simulator's snapshot, which is not a syscall, so building the
/// model moves no crash point.
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

/// What the working-copy change holds, read from the store: each file's
/// content by its blob, and the changeset's description. `None` if the
/// state cannot be read (a missing or damaged object).
fn held<F: Fs>(ws: &Workspace<F>) -> Option<Held> {
    let (id, _, tree) = ws.working_copy().ok()?;
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
    let changeset = Changeset::from_envelope(&ws.store().get_object(&id).ok()?).ok()?;
    Some((files, changeset.description))
}

fn run(
    fs: &Rc<SimFs>,
    store_variant: store::Variant,
    variant: Variant,
    seed: u64,
    log: &RefCell<Log>,
) -> Result<(), workspace::Error> {
    let mut ws = if store_variant == store::Variant::Correct {
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
            store_variant,
        )?;
        Workspace::init_with_store(store, Path::new(ROOT), seed)?.0
    };
    ws.set_variant(variant);
    {
        let mut log = log.borrow_mut();
        log.init_acked = true;
        log.acked = Some((Files::new(), Vec::new()));
    }
    // Reading the heads is itself syscalls, which may crash: the
    // acknowledged state is recorded first.
    let heads = ws.store().heads()?;
    log.borrow_mut().acked_ops.extend(heads);

    // The command in flight may leave the change in any of `steps`; once it
    // returns, the last of them is what was acknowledged.
    let begin = |steps: Vec<Held>| log.borrow_mut().in_flight = steps;
    let end = |ops: Vec<HashRef>| {
        let mut log = log.borrow_mut();
        log.acked_ops.extend(ops);
        let last = log.in_flight.pop().expect("a command in flight");
        log.in_flight.clear();
        log.acked = Some(last);
    };
    let files = || model(fs);

    // An edit and a snapshot: the change has no description yet.
    put(&**fs, "a", b"alpha")?;
    begin(vec![(files(), Vec::new())]);
    let op = ws.snapshot()?.op;
    end(op.into_iter().collect());

    // The first description.
    begin(vec![(files(), FIRST.to_vec())]);
    let op = ws.describe(FIRST)?.op;
    end(op.into_iter().collect());

    // An edit and a snapshot: the description is carried forward.
    put(&**fs, "a", b"alpha, edited")?;
    put(&**fs, "src/b", b"beta")?;
    begin(vec![(files(), FIRST.to_vec())]);
    let op = ws.snapshot()?.op;
    end(op.into_iter().collect());

    // A second description, then the same text again, which writes nothing.
    begin(vec![(files(), SECOND.to_vec())]);
    let op = ws.describe(SECOND)?.op;
    end(op.into_iter().collect());
    begin(vec![(files(), SECOND.to_vec())]);
    let again = ws.describe(SECOND)?.op;
    assert!(again.is_none(), "the same text wrote an op");
    end(Vec::new());

    // `new`: the next change holds the same files and no description.
    begin(vec![(files(), Vec::new())]);
    ws.new_change()?;
    end(Vec::new());
    let heads = ws.store().heads()?;
    log.borrow_mut().acked_ops.extend(heads);

    // An edit no command has recorded, then a description: the edit is
    // recorded first, in an op of its own, and then the description.
    put(&**fs, "c", b"gamma")?;
    begin(vec![(files(), Vec::new()), (files(), THIRD.to_vec())]);
    let op = ws.describe(THIRD)?.op;
    end(op.into_iter().collect());
    // The command returned, so every op it wrote is acknowledged: the op
    // that recorded the edit as well as the one it returned.
    let ops = ws.store().ops()?;
    log.borrow_mut().acked_ops.extend(ops.into_keys());
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

/// The invariants, and the reach of the next commands the scenario models
/// (`init`, an edit and a snapshot on a copy), read before the oracle
/// checks.
fn check(
    recovered: &SimFs,
    log: &Log,
    variant: Variant,
    seed: u64,
) -> (Violations, Vec<(&'static str, u64)>) {
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
                        let now = held(&ws);
                        let empty = (Files::new(), Vec::new());
                        let ok = now.as_ref().is_some_and(|now| {
                            log.acked.as_ref() == Some(now)
                                || log.in_flight.contains(now)
                                || (!log.init_acked && *now == empty)
                        });
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
    // Liveness, on a copy: `init`, one more edit and a snapshot work; the
    // change then holds the recovered working copy with that edit, and the
    // description it had before.
    let copy = recovered.recover();
    let mut next = Vec::new();
    let live = Workspace::init(&copy, root, seed).and_then(|(mut ws, _)| {
        ws.set_variant(variant);
        let before = held(&ws);
        put(&copy, "after-the-crash", b"one more edit")?;
        ws.snapshot()?;
        next = copy.reach();
        let after = held(&ws);
        Ok(match (before, after) {
            (Some((_, description)), Some((files, kept))) => {
                files == model(&copy) && kept == description
            }
            _ => false,
        })
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
    /// The scenario's own reach and its next commands'.
    reach: Vec<(&'static str, u64)>,
}

fn matrix(store_variant: store::Variant, variant: Variant) -> Vec<Cell> {
    let mut cells = Vec::new();
    for mode in MODES {
        for seed in 0..SEEDS {
            let syscalls = {
                let fs = Rc::new(SimFs::new(seed, mode));
                let log = RefCell::new(Log::default());
                run(&fs, store_variant, variant, seed, &log).expect("an uncrashed run succeeds");
                fs.syscalls()
            };
            for crash_at in 1..=syscalls + 1 {
                let fs = Rc::new(SimFs::new(seed, mode));
                fs.set_crash_at(Some(crash_at));
                let log = RefCell::new(Log::default());
                let _ = run(&fs, store_variant, variant, seed, &log);
                let recovered = fs.recover();
                let (violations, next) = check(&fs.recover(), &log.borrow(), variant, seed);
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
            "describe matrix {:<16} {variant:?} {mode}: {count} cells; violations: closure {} durability {} recency {} liveness {}",
            store_variant.name(),
            v.closure,
            v.durability,
            v.recency,
            v.liveness
        );
    }
    cells
}

#[test]
fn describe_matrix_finds_no_violations() {
    let cells = matrix(store::Variant::Correct, Variant::Correct);
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
    sim::reach::record(&dir, "describe", "Describe", 6, &totals).unwrap();
}

#[test]
fn describe_matrix_is_reproducible() {
    let first = matrix(store::Variant::Correct, Variant::Correct);
    let second = matrix(store::Variant::Correct, Variant::Correct);
    assert_eq!(first.len(), second.len());
    let mut all = Vec::new();
    for (a, b) in first.iter().zip(&second) {
        assert_eq!((a.mode, a.seed, a.crash_at), (b.mode, b.seed, b.crash_at));
        assert_eq!(a.state, b.state, "cell {:?}", (a.mode, a.seed, a.crash_at));
        assert_eq!(a.trace, b.trace, "cell {:?}", (a.mode, a.seed, a.crash_at));
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
        "describe matrix digest over {} cells: {digest}",
        first.len()
    );
    // Asserted, not only printed, so a change that moves any cell fails
    // here on every system.
    assert_eq!(first.len(), CELLS);
    assert_eq!(digest.to_string(), DIGEST);
}

/// The matrix's size and digest, the same on Windows and Linux.
const CELLS: usize = 28_720;
const DIGEST: &str = "blake3:919cd713956767f84cdad2c512bb34c2247f6c439793a59dd0dd2afcb986dd29";

/// A workspace whose snapshot drops the description is caught: the change
/// no longer holds what the last acknowledged command left.
#[test]
fn describe_catches_a_dropped_description() {
    let cells = matrix(store::Variant::Correct, Variant::DropDescription);
    let caught: u64 = cells
        .iter()
        .map(|c| c.violations.recency + c.violations.liveness)
        .sum();
    assert!(
        caught > 0,
        "the dropped-description variant produced no violation"
    );
    // Nothing else is wrong with that workspace: the store stays whole.
    assert_eq!(
        cells
            .iter()
            .map(|c| c.violations.closure + c.violations.durability)
            .sum::<u64>(),
        0
    );
}

#[test]
fn describe_catches_unsynced_objects() {
    let cells = matrix(store::Variant::UnsyncedObjects, Variant::Correct);
    let strict: u64 = cells
        .iter()
        .filter(|c| c.mode == Mode::Strict)
        .map(|c| c.violations.closure + c.violations.durability)
        .sum();
    assert!(
        strict > 0,
        "the unsynced-objects variant produced no violation"
    );
}
