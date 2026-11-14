//! The Concurrency scenario (INT-0010 AC-5 and AC-7, INT-0014 AC-3,
//! INT-0005 AC-1, AC-3 and AC-6; Sprint 7 T-009): eight lanes at once.
//!
//! A repository at `repo` records `base`, starts a change on top of it, and
//! makes eight lanes, `lane-0` to `lane-7`. Then eight simulated processes,
//! one in each lane, run [`SCRIPT`] at once under the seeded scheduler
//! ([`sim::Processes`]). In the one-store run the lanes share the
//! repository's store. In the two-store run lanes 4 to 7 use a second
//! store, `replica`, which the exchange (below) makes from the first before
//! the processes start: two stores in one simulated filesystem stand in for
//! two replicas.
//!
//! Each cell, a directory mode and a seed, runs three times: once to count
//! each process's steps; once with a kill, a process and a step drawn from
//! the seed; and once with that kill and a system crash at a syscall drawn
//! from those the processes made in the second run. The second and third
//! runs are each checked after recovery, the second as a crash after its
//! last syscall. A system crash stops every process, so a command that
//! fails after it is not counted. In the two-store run the two stores then
//! exchange what each lacks, through the store's own `put_object` and
//! `commit_op`, references and parents first. Every op is exchanged, since
//! ops carry no scope yet.
//!
//! The log lives outside the process threads, so it survives kills. An op
//! is *acknowledged* once the command that committed it has returned; a
//! lane whose script did not end has a command *in flight*.
//!
//! The oracles, after recovery and the exchange:
//! - no command failed before the crash, but in the killed process;
//! - a command that changes nothing committed nothing (INT-0010 AC-7);
//! - every acknowledged op is in every store, each store checks clean, and
//!   no lane holds a store of its own;
//! - each lane's writer is its last acknowledged op, or an op of its command
//!   in flight that follows that op;
//! - where the writer is the last acknowledged op, each store's merged view
//!   holds that op's entry for the lane, and every changeset the entry names
//!   is present, with every changeset it follows; where the writer is an op
//!   in flight, the last acknowledged op's entry is still in the store, with
//!   every changeset it names, so `restore` can return to it;
//! - every lane computes the view the heads leave together, the same on
//!   every store, and then runs one more command.

mod common;

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use codec::{Changeset, Operation, View, WorkspaceEntry};
use hash::{HashAlg, HashRef};
use sim::{Fs, KillAt, Mode, Node, Outcome, Processes, Rng, SimFs};
use store::Store;
use workspace::lane::{self, LaneFile};
use workspace::{Error, Variant, Workspace, merge};

const SEEDS: u64 = 40;
const MODES: [Mode; 2] = [Mode::Lenient, Mode::Strict];
const LANES: usize = 8;
const ALG: HashAlg = HashAlg::DEFAULT;
const STORE: &str = "repo/.praps/store";
const REPLICA: &str = "replica";
const MERGED: &str = "workspace.merge.computed";
const MADE: &str = "workspace.lane.made";

type Ws = Workspace<Arc<SimFs>>;
type Ops = BTreeMap<HashRef, Operation>;

thread_local! {
    /// The simulated process the current thread runs, for counting steps.
    static ME: Cell<Option<usize>> = const { Cell::new(None) };
}

/// One step of a process's script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// A new file written into the lane: an edit, not a command.
    Edit(&'static str),
    /// `status`, which records the edit.
    Status,
    /// `describe`, with the lane's own text.
    Describe,
    /// `new`.
    New,
    /// `undo`, which records the edit first, then takes back `new`.
    Undo,
    /// `status` with nothing to record, then `changes`: commands that
    /// change nothing.
    Read,
}

/// Each process's script.
const SCRIPT: [Step; 7] = [
    Step::Edit("one"),
    Step::Status,
    Step::Describe,
    Step::New,
    Step::Edit("two"),
    Step::Undo,
    Step::Read,
];

/// The store and workspace a run is built with: correct, or one negative
/// control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Variants {
    store: store::Variant,
    workspace: Variant,
}

const CORRECT: Variants = Variants {
    store: store::Variant::Correct,
    workspace: Variant::Correct,
};

fn lane_name(p: usize) -> String {
    format!("lane-{p}")
}

/// The store lane `p` uses.
fn store_of(p: usize, two: bool) -> &'static str {
    if two && p >= LANES / 2 {
        REPLICA
    } else {
        STORE
    }
}

/// What one lane's process did, kept outside its thread.
#[derive(Clone, Debug, Default)]
struct LaneLog {
    /// Its acknowledged ops, in order: the op that made the lane first.
    acked: Vec<HashRef>,
    /// Its script ran to the end.
    done: bool,
    /// What failed while the filesystem had not crashed.
    failed: Vec<String>,
    /// A command failed after the crash.
    crashed: bool,
    /// Commands that change nothing but committed an op.
    readers_wrote: u64,
}

#[derive(Clone, Debug, Default)]
struct Log {
    lanes: Vec<LaneLog>,
}

/// The repository with the store built in `variant`.
fn init(fs: &Arc<SimFs>, seed: u64, variant: store::Variant) -> Result<Ws, Error> {
    if variant == store::Variant::Correct {
        return Ok(Workspace::init(Arc::clone(fs), Path::new(common::ROOT), seed)?.0);
    }
    // The same steps as `init`, with the store built in the variant.
    fs.create_dir(Path::new(common::ROOT))?;
    fs.sync_dir(Path::new(""))?;
    fs.create_dir(Path::new("repo/.praps"))?;
    fs.sync_dir(Path::new("repo/.praps"))?;
    fs.sync_dir(Path::new(common::ROOT))?;
    let store = Store::init_variant(Arc::clone(fs), Path::new(STORE), ALG, seed, variant)?;
    Ok(Workspace::init_with_store(store, Path::new(common::ROOT), seed)?.0)
}

/// The repository, its eight lanes and, for two stores, the replica, which
/// lanes 4 to 7 then name. Returns the op that made each lane.
fn setup(
    fs: &Arc<SimFs>,
    two: bool,
    seed: u64,
    variants: Variants,
) -> Result<Vec<HashRef>, String> {
    let why = |error: &dyn std::fmt::Display| error.to_string();
    let mut ws = init(fs, seed, variants.store).map_err(|e| why(&e))?;
    common::put(&**fs, "base", b"base");
    ws.new_change().map_err(|e| why(&e))?;
    let mut made = Vec::new();
    for p in 0..LANES {
        let lane = lane_name(p);
        made.push(
            ws.make_lane(&lane, Path::new(&lane))
                .map_err(|e| why(&e))?
                .0,
        );
    }
    drop(ws);
    if two {
        let from = Store::open_variant(
            Arc::clone(fs),
            Path::new(STORE),
            ALG,
            seed + 1,
            variants.store,
        )
        .map_err(|e| why(&e))?;
        let mut to = Store::init_variant(
            Arc::clone(fs),
            Path::new(REPLICA),
            ALG,
            seed + 2,
            variants.store,
        )
        .map_err(|e| why(&e))?;
        copy_into(&from, &mut to).map_err(|e| why(&e))?;
        for p in LANES / 2..LANES {
            let file = LaneFile {
                name: lane_name(p),
                store: PathBuf::from(REPLICA),
            };
            lane::write(&**fs, &Path::new(&file.name).join(".praps"), &file, false)
                .map_err(|e| why(&e))?;
        }
    }
    Ok(made)
}

/// Opens lane `p` in `variants`: through [`Workspace::open`], as `praps`
/// does, unless the store is a broken variant.
fn open_lane(
    fs: &Arc<SimFs>,
    p: usize,
    two: bool,
    seed: u64,
    variants: Variants,
) -> Result<Ws, Error> {
    let lane = lane_name(p);
    let mut ws = if variants.store == store::Variant::Correct {
        Workspace::open(Arc::clone(fs), Path::new(&lane), seed)?
    } else {
        let store = Store::open_variant(
            Arc::clone(fs),
            Path::new(store_of(p, two)),
            ALG,
            seed,
            variants.store,
        )?;
        Workspace::open_lane_with_store(store, Path::new(&lane), seed)?
    };
    ws.set_variant(variants.workspace);
    Ok(ws)
}

/// Process `p`: opens its lane and runs the script, logging as it goes.
/// It stops at the first failure.
fn process(fs: &Arc<SimFs>, p: usize, two: bool, seed: u64, variants: Variants, log: &Mutex<Log>) {
    ME.with(|me| me.set(Some(p)));
    let fail = |what: String| {
        let mut log = log.lock().unwrap();
        if fs.crashed() {
            log.lanes[p].crashed = true;
        } else {
            log.lanes[p].failed.push(what);
        }
    };
    let mut ws = match open_lane(fs, p, two, seed + 300 + p as u64, variants) {
        Ok(ws) => ws,
        Err(error) => return fail(format!("open: {error}")),
    };
    let lane = lane_name(p);
    for step in SCRIPT {
        let before = ws.last_op();
        let result = match step {
            Step::Edit(name) => {
                let path = Path::new(&lane).join(name);
                fs.create_new(&path)
                    .and_then(|()| fs.write(&path, format!("{lane} {name}").as_bytes()))
                    .map_err(Error::from)
            }
            Step::Status => ws.status().map(drop),
            Step::Describe => ws.describe(format!("work of {lane}").as_bytes()).map(drop),
            Step::New => ws.new_change().map(drop),
            Step::Undo => ws.undo().map(drop),
            Step::Read => ws.status().and_then(|_| ws.changes()).map(drop),
        };
        if let Err(error) = result {
            return fail(format!("{step:?}: {error}"));
        }
        let after = ws.last_op();
        if after != before {
            let mut log = log.lock().unwrap();
            let lane_log = &mut log.lanes[p];
            match (step, after) {
                (Step::Read, _) => lane_log.readers_wrote += 1,
                (_, Some(op)) => lane_log.acked.push(op),
                (_, None) => {}
            }
        }
    }
    log.lock().unwrap().lanes[p].done = true;
}

type Kill = Option<(usize, KillAt)>;

/// The setup, then the eight processes, with `kill` and a crash at syscall
/// `crash_at`, which falls after the setup. Returns how many syscalls the
/// setup made, and how many processes were killed.
fn run(
    fs: &Arc<SimFs>,
    two: bool,
    seed: u64,
    variants: Variants,
    kill: Kill,
    crash_at: Option<u64>,
    log: &Arc<Mutex<Log>>,
) -> (u64, u64) {
    let made = setup(fs, two, seed, variants)
        .unwrap_or_else(|why| panic!("seed {seed}: the setup failed: {why}"));
    let setup_syscalls = fs.syscalls();
    log.lock().unwrap().lanes = made
        .iter()
        .map(|op| LaneLog {
            acked: vec![*op],
            ..LaneLog::default()
        })
        .collect();
    fs.set_crash_at(crash_at);
    let mut processes = Processes::new(seed ^ 0xc0c0_0009);
    if let Some((p, at)) = kill {
        processes = processes.kill(p, at);
    }
    let closures: Vec<Box<dyn FnOnce() + Send>> = (0..LANES)
        .map(|p| {
            let (fs, log) = (Arc::clone(fs), Arc::clone(log));
            Box::new(move || process(&fs, p, two, seed, variants, &log)) as Box<dyn FnOnce() + Send>
        })
        .collect();
    let ended = processes
        .run(fs, closures)
        .unwrap_or_else(|stopped| panic!("seed {seed}: {stopped:?}"));
    fs.clear_points();
    let killed = ended
        .iter()
        .filter(|ended| **ended == Outcome::Killed)
        .count();
    (setup_syscalls, killed as u64)
}

/// Each process's steps (syscall and point boundaries) in the cell's run
/// with no kill and no crash.
fn steps(two: bool, mode: Mode, seed: u64, variants: Variants) -> [u64; LANES] {
    let fs = Arc::new(SimFs::new(seed, mode));
    let counts = Arc::new(Mutex::new([0_u64; LANES]));
    let points = store::points::POINTS
        .iter()
        .chain(workspace::points::POINTS)
        .map(|point| point.name);
    for name in points {
        let counts = Arc::clone(&counts);
        fs.on_point(name, move || {
            if let Some(p) = ME.with(Cell::get) {
                counts.lock().unwrap()[p] += 1;
            }
        });
    }
    let hooked = Arc::clone(&counts);
    fs.set_hook(Some(Box::new(move |_| {
        if let Some(p) = ME.with(Cell::get) {
            hooked.lock().unwrap()[p] += 1;
        }
    })));
    run(&fs, two, seed, variants, None, None, &Arc::default());
    fs.set_hook(None);
    *counts.lock().unwrap()
}

/// What a cell broke.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Violations {
    /// Commands that failed before the crash, but in the killed process.
    failed: u64,
    /// Commands that change nothing but committed an op.
    readers: u64,
    /// Acknowledged ops missing from a store.
    lost: u64,
    /// Stores that do not check clean or did not exchange, and lanes that
    /// hold a store of their own.
    store: u64,
    /// Lanes whose writer is neither their last acknowledged op nor an op of
    /// their command in flight that follows it.
    writer: u64,
    /// Lanes whose last acknowledged entry is not kept as the writer's kind
    /// requires.
    kept: u64,
    /// Lanes that do not compute the view their store's heads leave
    /// together, and stores whose views differ.
    view: u64,
    /// Lanes whose next command after recovery failed.
    next: u64,
}

impl Violations {
    fn total(&self) -> u64 {
        self.failed
            + self.readers
            + self.lost
            + self.store
            + self.writer
            + self.kept
            + self.view
            + self.next
    }

    fn add(&mut self, other: &Violations) {
        self.failed += other.failed;
        self.readers += other.readers;
        self.lost += other.lost;
        self.store += other.store;
        self.writer += other.writer;
        self.kept += other.kept;
        self.view += other.view;
        self.next += other.next;
    }
}

/// What a cell exercised: the scenario's own coverage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Seen {
    /// Lanes whose writer, on some store, was an op of their command in
    /// flight.
    landed: u64,
    /// Lanes whose command failed after the crash.
    crashed: u64,
    /// Processes killed before their script ended.
    killed: u64,
}

impl Seen {
    fn add(&mut self, other: &Seen) {
        self.landed += other.landed;
        self.crashed += other.crashed;
        self.killed += other.killed;
    }
}

/// The view object `id`.
fn view_of(store: &Store<Arc<SimFs>>, id: &HashRef) -> Result<View, String> {
    let bytes = store.get_object(id).map_err(|error| error.to_string())?;
    View::from_envelope(&bytes).map_err(|error| error.to_string())
}

/// Whether `later` is `earlier` or follows it.
fn follows(ops: &Ops, later: &HashRef, earlier: &HashRef) -> bool {
    let mut stack = vec![*later];
    let mut seen = BTreeSet::new();
    while let Some(id) = stack.pop() {
        if id == *earlier {
            return true;
        }
        if !seen.insert(id) {
            continue;
        }
        if let Some(op) = ops.get(&id) {
            stack.extend(op.parents.iter().copied());
        }
    }
    false
}

/// Whether every changeset `entry` names is in `store`, with every
/// changeset it follows.
fn present(store: &Store<Arc<SimFs>>, entry: &WorkspaceEntry) -> bool {
    let mut stack: Vec<HashRef> = entry
        .working_copy
        .iter()
        .chain(&entry.heads)
        .copied()
        .collect();
    let mut seen = BTreeSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let read = store.get_object(&id).ok();
        match read.and_then(|bytes| Changeset::from_envelope(&bytes).ok()) {
            Some(changeset) => stack.extend(changeset.parents),
            None => return false,
        }
    }
    true
}

/// The oracles on one store: it checks clean, holds every acknowledged op,
/// and keeps each lane's writer and entry as the oracle requires. Returns
/// the view its heads leave together, when it has one.
fn check_store(
    fs: &Arc<SimFs>,
    root: &str,
    seed: u64,
    log: &Log,
    v: &mut Violations,
    seen: &mut Seen,
) -> Option<View> {
    let Ok(store) = Store::open(Arc::clone(fs), Path::new(root), ALG, seed) else {
        v.store += 1;
        return None;
    };
    if !store.fsck().is_ok_and(|report| report.is_clean()) {
        v.store += 1;
    }
    let Ok(ops) = store.ops() else {
        v.store += 1;
        return None;
    };
    if ops
        .values()
        .any(|op| op.parents.iter().any(|parent| !ops.contains_key(parent)))
    {
        v.store += 1;
        return None;
    }
    let heads = store::heads_of(&ops);
    let merged = merge::merge(&ops, &heads, |id| view_of(&store, id)).ok();
    let writers = merge::writers(&ops, &heads);
    for (p, lane) in log.lanes.iter().enumerate() {
        let name = lane_name(p);
        v.lost += lane.acked.iter().filter(|id| !ops.contains_key(id)).count() as u64;
        let last = *lane.acked.last().expect("the op that made the lane");
        let entry = |id: &HashRef| {
            let view = view_of(&store, &ops.get(id)?.view).ok()?;
            view.workspaces.get(&name).cloned()
        };
        let written: Vec<&HashRef> = writers.get(&name).into_iter().flatten().collect();
        if written == [&last] {
            // The merged view holds the last acknowledged entry.
            let kept = entry(&last);
            let held = merged.as_ref().and_then(|view| view.workspaces.get(&name));
            if kept.is_none() || held != kept.as_ref() || !kept.is_some_and(|e| present(&store, &e))
            {
                v.kept += 1;
            }
            continue;
        }
        let in_flight = match written[..] {
            [op] => {
                !lane.done
                    && !lane.acked.contains(op)
                    && ops[op].workspace == name
                    && follows(&ops, op, &last)
            }
            _ => false,
        };
        if !in_flight {
            v.writer += 1;
            continue;
        }
        seen.landed += 1;
        if !entry(&last).is_some_and(|e| present(&store, &e)) {
            // The last acknowledged entry is still there to restore.
            v.kept += 1;
        }
    }
    merged
}

/// The oracles, on the recovered filesystem, and what the cell exercised.
fn check(
    recovered: SimFs,
    two: bool,
    seed: u64,
    variants: Variants,
    log: &Log,
) -> (Violations, Seen) {
    let fs = Arc::new(recovered);
    let mut v = Violations::default();
    let mut seen = Seen::default();
    for lane in &log.lanes {
        v.failed += lane.failed.len() as u64;
        v.readers += lane.readers_wrote;
        seen.crashed += u64::from(lane.crashed);
    }
    if two && exchange(&fs, seed).is_err() {
        v.store += 1;
    }
    let roots: &[&str] = if two { &[STORE, REPLICA] } else { &[STORE] };
    let views: Vec<Option<View>> = roots
        .iter()
        .map(|root| check_store(&fs, root, seed, log, &mut v, &mut seen))
        .collect();
    if views.iter().any(Option::is_none) || views.windows(2).any(|pair| pair[0] != pair[1]) {
        v.view += 1;
    }
    for p in 0..LANES {
        let lane = lane_name(p);
        let own = fs.list(&Path::new(&lane).join(".praps"));
        if own.is_ok_and(|names| names.iter().any(|name| name == "store")) {
            v.store += 1;
        }
        let expected = views[usize::from(two && p >= LANES / 2)].as_ref();
        let seen = open_lane(&fs, p, two, seed + 500 + p as u64, variants).and_then(|ws| ws.view());
        if !seen.is_ok_and(|view| Some(&view) == expected) {
            v.view += 1;
        }
    }
    // One more command in each lane.
    for p in 0..LANES {
        let next = open_lane(&fs, p, two, seed + 600 + p as u64, variants)
            .and_then(|mut ws| ws.describe(b"after recovery"));
        if next.is_err() {
            v.next += 1;
        }
    }
    (v, seen)
}

/// `ops` in an order that puts each op after its parents.
fn parents_first(ops: &Ops) -> Vec<HashRef> {
    let mut order = Vec::new();
    let mut placed = BTreeSet::new();
    while order.len() < ops.len() {
        let before = order.len();
        for (id, op) in ops {
            let ready = op
                .parents
                .iter()
                .all(|parent| placed.contains(parent) || !ops.contains_key(parent));
            if !placed.contains(id) && ready {
                placed.insert(*id);
                order.push(*id);
            }
        }
        assert!(order.len() > before, "the op log has a cycle");
    }
    order
}

/// Copies object `id` from `from` into `to`, the objects it names first.
fn copy_object(
    from: &Store<Arc<SimFs>>,
    to: &mut Store<Arc<SimFs>>,
    id: &HashRef,
    copied: &mut BTreeSet<HashRef>,
) -> io::Result<()> {
    if !copied.insert(*id) {
        return Ok(());
    }
    let bytes = from.get_object(id)?;
    for named in codec::references(&bytes).map_err(io::Error::other)? {
        copy_object(from, to, &named, copied)?;
    }
    let put = to.put_object(&bytes)?;
    if put != *id {
        return Err(io::Error::other(format!("object {id} was put as {put}")));
    }
    Ok(())
}

/// Copies into `to` every op of `from` that `to` lacks, parents first, each
/// after the objects its view reaches.
fn copy_into(from: &Store<Arc<SimFs>>, to: &mut Store<Arc<SimFs>>) -> io::Result<()> {
    let ops = from.ops()?;
    let have = to.ops()?;
    let mut copied = BTreeSet::new();
    for id in parents_first(&ops) {
        if have.contains_key(&id) {
            continue;
        }
        let op = &ops[&id];
        copy_object(from, to, &op.view, &mut copied)?;
        let committed = to.commit_op(op)?;
        if committed != id {
            return Err(io::Error::other(format!(
                "op {id} was committed as {committed}"
            )));
        }
    }
    Ok(())
}

/// The two stores exchange what each lacks.
fn exchange(fs: &Arc<SimFs>, seed: u64) -> io::Result<()> {
    let mut first = Store::open(Arc::clone(fs), Path::new(STORE), ALG, seed + 700)?;
    let mut second = Store::open(Arc::clone(fs), Path::new(REPLICA), ALG, seed + 701)?;
    copy_into(&first, &mut second)?;
    copy_into(&second, &mut first)
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

/// One run, checked after recovery.
struct Checked {
    mode: Mode,
    seed: u64,
    /// The crash point, or `None` for a crash after the last syscall.
    crash_at: Option<u64>,
    violations: Violations,
    seen: Seen,
    state: HashRef,
    trace: HashRef,
    reach: Vec<(&'static str, u64)>,
}

/// One run with `kill` and `crash_at`, checked after recovery. Returns the
/// setup's syscalls, the run's, and the cell.
fn checked(
    two: bool,
    mode: Mode,
    seed: u64,
    variants: Variants,
    kill: Kill,
    crash_at: Option<u64>,
) -> (u64, u64, Checked) {
    let fs = Arc::new(SimFs::new(seed, mode));
    let log = Arc::new(Mutex::new(Log::default()));
    let (setup, killed) = run(&fs, two, seed, variants, kill, crash_at, &log);
    let total = fs.syscalls();
    let reach = fs.reach();
    let trace = HashAlg::Blake3.hash(fs.trace().join("\n").as_bytes());
    let recovered = fs.recover();
    let state = digest(&recovered);
    let log = log.lock().unwrap().clone();
    let (violations, mut seen) = check(recovered, two, seed, variants, &log);
    seen.killed = killed;
    let cell = Checked {
        mode,
        seed,
        crash_at,
        violations,
        seen,
        state,
        trace,
        reach,
    };
    (setup, total, cell)
}

/// The run's cells: for each mode and seed, the run with the kill crashed
/// after its last syscall, then at a syscall drawn from the seed.
fn matrix(two: bool, variants: Variants) -> Vec<Checked> {
    cells_until(two, variants, |_| false)
}

/// The run's cells in [`matrix`]'s order, up to and including the first for
/// which `stop` holds.
fn cells_until(
    two: bool,
    variants: Variants,
    mut stop: impl FnMut(&Checked) -> bool,
) -> Vec<Checked> {
    let mut cells = Vec::new();
    for mode in MODES {
        for seed in 0..SEEDS {
            let steps = steps(two, mode, seed, variants);
            let mut rng = Rng::new(seed ^ 0x7e57_0009);
            let killed = rng.below(LANES as u64) as usize;
            let kill = Some((killed, KillAt::Step(1 + rng.below(steps[killed]))));
            let (setup, total, last) = checked(two, mode, seed, variants, kill, None);
            let done = stop(&last);
            cells.push(last);
            if done {
                return cells;
            }
            let crash_at = setup + 1 + rng.below(total - setup);
            let crashed = checked(two, mode, seed, variants, kill, Some(crash_at)).2;
            let done = stop(&crashed);
            cells.push(crashed);
            if done {
                return cells;
            }
        }
    }
    cells
}

/// The violations and coverage summed over `cells`, printed.
fn summary(name: &str, cells: &[Checked]) -> (Violations, Seen) {
    let mut total = Violations::default();
    let mut seen = Seen::default();
    for cell in cells {
        total.add(&cell.violations);
        seen.add(&cell.seen);
    }
    let caught = cells
        .iter()
        .filter(|cell| cell.violations.total() > 0)
        .count();
    println!(
        "{name}: {} cells, {caught} with a violation; violations: {total:?}; seen: {seen:?}",
        cells.len()
    );
    (total, seen)
}

/// The scenario exercised what its oracles are for: a kill, commands
/// stopped by the crash, and a command in flight whose op landed.
fn exercised(seen: &Seen) {
    assert!(seen.killed > 0, "no process was killed: {seen:?}");
    assert!(
        seen.crashed > 0,
        "no command was stopped by the crash: {seen:?}"
    );
    assert!(seen.landed > 0, "no op in flight landed: {seen:?}");
}

fn failing(cells: &[Checked]) -> Vec<(Mode, u64, Option<u64>, Violations)> {
    cells
        .iter()
        .filter(|cell| cell.violations.total() > 0)
        .map(|cell| (cell.mode, cell.seed, cell.crash_at, cell.violations))
        .take(10)
        .collect()
}

static ONE_STORE: OnceLock<Vec<Checked>> = OnceLock::new();
static TWO_STORES: OnceLock<Vec<Checked>> = OnceLock::new();

fn one_store() -> &'static [Checked] {
    ONE_STORE.get_or_init(|| matrix(false, CORRECT))
}

fn two_stores() -> &'static [Checked] {
    TWO_STORES.get_or_init(|| matrix(true, CORRECT))
}

/// INT-0014 AC-3: eight lanes at once over one store.
#[test]
fn concurrency_on_one_store_finds_no_violations() {
    let cells = one_store();
    exercised(&summary("one store", cells).1);
    let failing = failing(cells);
    assert!(failing.is_empty(), "{failing:?}");
}

/// INT-0010 AC-5: eight writers on two replicas lose no work. Also the
/// designated record of the scenario's reach (INT-0005 AC-6), over both
/// runs, which merge op heads and make lanes.
#[test]
fn concurrency_finds_no_violations() {
    let cells = two_stores();
    exercised(&summary("two stores", cells).1);
    let failing = failing(cells);
    assert!(failing.is_empty(), "{failing:?}");
    let mut totals = sim::reach::Totals::new();
    for cell in one_store().iter().chain(two_stores()) {
        sim::reach::add(&mut totals, &cell.reach);
    }
    for point in [MERGED, MADE] {
        assert!(totals.get(point).copied().unwrap_or(0) > 0, "{point}");
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("reach");
    sim::reach::record(&dir, "concurrency", "Concurrency", 7, &totals).unwrap();
}

/// Compares `second`, a run of `two` again, with `first`, cell by cell.
/// Returns the cell count and the digest over every cell's state and trace.
fn compare(two: bool, first: &[Checked], second: &[Checked]) -> (usize, String) {
    assert_eq!(first.len(), second.len());
    let mut all = Vec::new();
    for (a, b) in first.iter().zip(second) {
        let at = (a.mode, a.seed, a.crash_at);
        assert_eq!(at, (b.mode, b.seed, b.crash_at));
        assert_eq!(a.state, b.state, "cell {at:?}");
        assert_eq!(a.trace, b.trace, "cell {at:?}");
        assert_eq!(a.violations, b.violations, "cell {at:?}");
        assert_eq!(a.seen, b.seen, "cell {at:?}");
        assert_eq!(a.reach, b.reach, "cell {at:?}");
        all.extend_from_slice(&a.state.to_bytes());
        all.extend_from_slice(&a.trace.to_bytes());
    }
    let digest = HashAlg::Blake3.hash(&all).to_string();
    let run = if two { "two stores" } else { "one store" };
    println!("{run}: digest over {} cells: {digest}", first.len());
    (first.len(), digest)
}

/// INT-0005 AC-1: each run, repeated, gives equal cells; each run's cell
/// count and digest are literals.
#[test]
fn concurrency_is_reproducible() {
    let (one, two) = std::thread::scope(|scope| {
        let one = scope.spawn(|| matrix(false, CORRECT));
        let two = scope.spawn(|| matrix(true, CORRECT));
        (one.join().unwrap(), two.join().unwrap())
    });
    // Both runs are compared, and both digests printed, before either
    // literal is asserted.
    let one = compare(false, one_store(), &one);
    let two = compare(true, two_stores(), &two);
    assert_eq!(one, (ONE_STORE_CELLS, ONE_STORE_DIGEST.to_owned()));
    assert_eq!(two, (TWO_STORE_CELLS, TWO_STORE_DIGEST.to_owned()));
}

const ONE_STORE_CELLS: usize = 160;
const TWO_STORE_CELLS: usize = 160;
// The digests differ between Windows and other systems, and nothing else
// does: each lane's `.praps/workspace` names the store's path as the system
// spells it (`repo\.praps\store` on Windows, `repo/.praps/store` elsewhere),
// and that file is part of every cell's state.
#[cfg(windows)]
const ONE_STORE_DIGEST: &str =
    "blake3:588a08e5c6fc02fb9f71d77961a93916b51be0cad0e1ee389872149aa8efe283";
#[cfg(windows)]
const TWO_STORE_DIGEST: &str =
    "blake3:459ab6fd17fe97ed20f0c82520a2366badedfbb601ffec9cf36e5a78e0375473";
#[cfg(not(windows))]
const ONE_STORE_DIGEST: &str =
    "blake3:a6e9c08352e0e4707468e26fa34b412382b9570c9327c3a1d401c911006e1f28";
#[cfg(not(windows))]
const TWO_STORE_DIGEST: &str =
    "blake3:ab960f27cd5085c5456ff6d1575e4fbabde6bbfcc090f2cfb88b236c000be8f7";

/// A negative control: the one-store run's cells and then the two-store
/// run's, in order, up to the first that breaks the oracle the control is
/// meant to break. Returns that cell's run, mode, seed and crash point.
/// A control needs one such cell; running every cell costs as much as the
/// scenario itself (the counts over all 320 are in Sprint 7's T-009
/// record).
fn control(
    variants: Variants,
    breaks: fn(&Violations) -> bool,
) -> Option<(bool, Mode, u64, Option<u64>)> {
    for two in [false, true] {
        let cells = cells_until(two, variants, |cell| breaks(&cell.violations));
        let run = if two { "two stores" } else { "one store" };
        summary(&format!("{variants:?}, {run}"), &cells);
        let last = cells.last().expect("a run has cells");
        if breaks(&last.violations) {
            return Some((two, last.mode, last.seed, last.crash_at));
        }
    }
    None
}

/// A load of the first head only continues from a state its own workspace
/// has left, so the lane ends with two writers.
#[test]
fn concurrency_catches_first_head_only() {
    let variants = Variants {
        workspace: Variant::FirstHeadOnly,
        ..CORRECT
    };
    let caught = control(variants, |v| v.writer > 0);
    assert!(caught.is_some(), "no cell has two writers for a lane");
}

/// A load that commits a merge makes a command that changes nothing write.
#[test]
fn concurrency_catches_merge_on_read() {
    let variants = Variants {
        workspace: Variant::MergeOnRead,
        ..CORRECT
    };
    let caught = control(variants, |v| v.readers > 0);
    assert!(caught.is_some(), "no command that changes nothing wrote");
}

/// Objects whose data is never synced do not survive the crash: the store
/// does not check clean.
#[test]
fn concurrency_catches_unsynced_objects() {
    let variants = Variants {
        store: store::Variant::UnsyncedObjects,
        ..CORRECT
    };
    let caught = control(variants, |v| v.store > 0);
    assert!(caught.is_some(), "every store checked clean");
}

/// The op files of the repository's store.
fn op_files(fs: &SimFs) -> Vec<String> {
    fs.snapshot()
        .into_keys()
        .filter(|path| path.starts_with("repo/.praps/store/ops/"))
        .collect()
}

type Reader = Box<dyn FnOnce() -> Result<View, String> + Send>;

/// `n` readers over three heads (INT-0010 AC-7): each opens one of the
/// eight lanes, computes the view, and runs `status` and `changes`, which
/// change nothing here, at once with the others. Returns why they failed
/// the criterion, if they did: an op file added or removed, a reader that
/// failed, or a view other than the merge of the heads.
fn readers(n: usize, variant: Variant) -> Result<(), String> {
    let seed = n as u64;
    let fs = Arc::new(SimFs::new(seed, Mode::Strict));
    setup(&fs, false, seed, CORRECT)?;
    // Three heads: three handles open over one head, then each commits.
    let mut handles = Vec::new();
    for p in 0..3 {
        handles.push(open_lane(&fs, p, false, 800 + p as u64, CORRECT).map_err(|e| e.to_string())?);
    }
    for (p, ws) in handles.iter_mut().enumerate() {
        ws.describe(format!("head {p}").as_bytes())
            .map_err(|e| e.to_string())?;
    }
    drop(handles);
    let store =
        Store::open(Arc::clone(&fs), Path::new(STORE), ALG, seed).map_err(|e| e.to_string())?;
    let ops = store.ops().map_err(|e| e.to_string())?;
    let heads = store::heads_of(&ops);
    assert_eq!(heads.len(), 3);
    let expected = merge::merge(&ops, &heads, |id| view_of(&store, id))?;
    let before = op_files(&fs);
    let variants = Variants {
        workspace: variant,
        ..CORRECT
    };
    let closures: Vec<Reader> = (0..n)
        .map(|r| {
            let fs = Arc::clone(&fs);
            Box::new(move || {
                let why = |error: Error| format!("reader {r}: {error}");
                let mut ws =
                    open_lane(&fs, r % LANES, false, 1_000 + r as u64, variants).map_err(why)?;
                let view = ws.view().map_err(why)?;
                ws.status().map_err(why)?;
                ws.changes().map_err(why)?;
                Ok(view)
            }) as Reader
        })
        .collect();
    let ended = Processes::new(seed)
        .run(&fs, closures)
        .map_err(|stopped| format!("{n} readers: {stopped:?}"))?;
    let after = op_files(&fs);
    if after != before {
        return Err(format!(
            "{n} readers: the op files went from {} to {}",
            before.len(),
            after.len()
        ));
    }
    for (r, ended) in ended.into_iter().enumerate() {
        match ended {
            Outcome::Done(Ok(view)) if view == expected => {}
            Outcome::Done(Ok(_)) => return Err(format!("reader {r} computed another view")),
            Outcome::Done(Err(why)) => return Err(why),
            Outcome::Killed => return Err(format!("reader {r} was stopped")),
        }
    }
    Ok(())
}

/// INT-0010 AC-7: readers add nothing, at 2, 8 and 1,000.
#[test]
fn readers_add_nothing_at_scale() {
    for n in [2, 8, 1_000] {
        if let Err(why) = readers(n, Variant::Correct) {
            panic!("{why}");
        }
    }
}

/// The readers' check catches a workspace that commits a merge when it
/// reads.
#[test]
fn readers_catch_merge_on_read() {
    for n in [2, 8] {
        let caught = readers(n, Variant::MergeOnRead);
        assert!(caught.is_err(), "{n} readers");
    }
}

fn hits(fs: &SimFs, point: &str) -> u64 {
    fs.reach()
        .iter()
        .find(|(name, _)| *name == point)
        .map_or(0, |(_, count)| *count)
}

/// A command over two op heads marks `workspace.merge.computed` once; a
/// command over one head marks nothing.
#[test]
fn merging_marks_its_point() {
    let fs = Arc::new(SimFs::new(3, Mode::Strict));
    setup(&fs, false, 3, CORRECT).unwrap();
    open_lane(&fs, 0, false, 30, CORRECT)
        .unwrap()
        .status()
        .unwrap();
    assert_eq!(hits(&fs, MERGED), 0, "a command over one head");
    // Two heads: two handles each commit over the one head they loaded.
    let mut first = open_lane(&fs, 1, false, 31, CORRECT).unwrap();
    let mut second = open_lane(&fs, 2, false, 32, CORRECT).unwrap();
    first.describe(b"first").unwrap();
    second.describe(b"second").unwrap();
    assert_eq!(
        hits(&fs, MERGED),
        0,
        "each committed over the one head it loaded"
    );
    open_lane(&fs, 3, false, 33, CORRECT)
        .unwrap()
        .status()
        .unwrap();
    assert_eq!(hits(&fs, MERGED), 1, "a command over two heads");
}

/// Making a lane marks `workspace.lane.made` once its op is committed; a
/// lane refused marks nothing.
#[test]
fn making_a_lane_marks_its_point() {
    let fs = Arc::new(SimFs::new(4, Mode::Strict));
    let mut ws = Workspace::init(Arc::clone(&fs), Path::new(common::ROOT), 4)
        .unwrap()
        .0;
    ws.make_lane("lane-a", Path::new("lane-a")).unwrap();
    assert_eq!(hits(&fs, MADE), 1);
    let again = ws.make_lane("lane-a", Path::new("lane-b"));
    assert!(matches!(again, Err(Error::LaneExists(_))), "{again:?}");
    assert_eq!(hits(&fs, MADE), 1);
}
