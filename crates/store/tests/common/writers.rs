//! The four-writer matrix (Sprint 4 T-007; INT-0007 AC-5; INT-0005 AC-2,
//! AC-3).
//!
//! `init` and a base op, then four simulated processes over one [`SimFs`],
//! each with its own `Store` and seed, scheduled by [`sim::Processes`]:
//! - each records a state of its own and commits a first op on the base;
//! - process 1 commits that op twice, as a retry after a lost
//!   acknowledgement would (`store.commit_op.present_synced`);
//! - processes 2 and 3 then list the ops they see and commit a second op
//!   whose parents are their own first op and every listed op no listed op
//!   names.
//!
//! In odd seeds, orderings force the parent race: processes 1 and 3 end, and
//! process 2's first commit finishes, before process 0 writes its op; process
//! 0 is then held between its op's rename and its `ops/` sync while process 2
//! lists and names that op. The kill setting kills process 0 there in odd
//! seeds, and a process and step drawn from the seed in even seeds.
//!
//! The log lives outside the process threads, so it survives kills. An op is
//! *attempted* when logged before `commit_op` and *acknowledged* when
//! `commit_op` returned; the *in-flight* ops are the attempted ids that were
//! never acknowledged. All three are sets of op ids.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use codec::{Operation, object_id};
use hash::{HashAlg, HashRef};
use sim::{After, Fs, KillAt, Mode, Node, Outcome as Ended, Processes, Rng, SimFs};
use store::names::{TMP_PREFIX, file_name, parse_name};
use store::{Store, Variant};

use super::matrix::{
    ALG, MODES, ROOT, SEEDS, Violations, as_object, closure, digest, id_of, op, present, state,
    view_of,
};

/// The processes after the base op.
pub const PROCESSES: usize = 4;
/// Without and with a kill.
pub const KILLS: [bool; 2] = [false, true];
/// The test-local point process 2 and 3 mark before listing ops; the
/// reserved `test.` prefix keeps it out of the reach table.
pub const LISTING: &str = "test.writers.listing";

thread_local! {
    /// The simulated process the current thread runs, for counting steps.
    static ME: Cell<Option<usize>> = const { Cell::new(None) };
}

/// What a run attempted and what the store acknowledged.
#[derive(Clone, Debug, Default)]
pub struct Log {
    pub init_acked: bool,
    pub attempted: BTreeSet<HashRef>,
    pub acked: BTreeSet<HashRef>,
    pub objects_acked: BTreeSet<HashRef>,
    /// Every op attempted, by id: expected heads come from here.
    pub ops: BTreeMap<HashRef, Operation>,
    /// Process 2's second op, once it is computed.
    pub second_of_2: Option<HashRef>,
    /// Process 2's second op names process 0's first op.
    pub names_op0: bool,
    /// At process 2's listing: process 0's op name (visible, durable).
    pub at_listing: Option<(bool, bool)>,
    /// Process 1 began its retry.
    pub retry_started: bool,
    /// Process 1's retry was acknowledged.
    pub retried: bool,
    /// How each process ended, when the run ended.
    pub killed: Vec<bool>,
}

/// A cell's outcome.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub violations: Violations,
    /// Problems `fsck` reports on the recovered store (leftovers are not
    /// problems).
    pub fsck: u64,
    /// Process 0's op was still not durable when process 2's second op's
    /// rename point ran its action. Actions run after the scheduler's draw
    /// at that point, so another process may have synced `ops/` first: this
    /// undercounts, and never overcounts, since a name once durable stays
    /// durable until a crash (code review CR-01).
    pub undurable_at_rename: bool,
    /// Two commits overlapped: an op was renamed into `ops/` while another
    /// op's rename there was not yet followed by a sync of `ops/`. Read from
    /// the trace, in syscall order (code review CR-01).
    pub overlapped: bool,
    pub killed: bool,
    pub state: HashRef,
    pub trace: HashRef,
    /// The named points the scenario's own run reached (not the oracle's).
    pub reach: Vec<(&'static str, u64)>,
}

/// The kill for one run: none, or a process and where.
pub type Kill = Option<(usize, KillAt)>;

fn ops_dir() -> PathBuf {
    Path::new(ROOT).join("ops")
}

fn put(store: &mut Store<Arc<SimFs>>, envelope: &[u8], log: &Mutex<Log>) -> io::Result<HashRef> {
    let id = store.put_object(envelope)?;
    log.lock().unwrap().objects_acked.insert(id);
    Ok(id)
}

fn record(
    store: &mut Store<Arc<SimFs>>,
    envelopes: &[Vec<u8>],
    log: &Mutex<Log>,
) -> io::Result<HashRef> {
    let mut last = None;
    for envelope in envelopes {
        last = Some(put(store, envelope, log)?);
    }
    Ok(last.expect("a state has objects"))
}

fn commit(store: &mut Store<Arc<SimFs>>, op: &Operation, log: &Mutex<Log>) -> io::Result<()> {
    let id = id_of(op);
    {
        let mut log = log.lock().unwrap();
        log.attempted.insert(id);
        log.ops.insert(id, op.clone());
    }
    store.commit_op(op)?;
    log.lock().unwrap().acked.insert(id);
    Ok(())
}

fn first_state(p: usize) -> Vec<Vec<u8>> {
    state(&format!("writer {p}"), &[&format!("writer {p}")])
}

fn base_state() -> Vec<Vec<u8>> {
    state("base", &["base"])
}

fn base_op() -> Operation {
    op(&[], view_of(&base_state()))
}

/// Process `p`'s first op.
fn first_op(p: usize) -> Operation {
    op(&[id_of(&base_op())], view_of(&first_state(p)))
}

/// The id of process `p`'s first op.
pub fn first_op_id(p: usize) -> HashRef {
    id_of(&first_op(p))
}

/// The odd seeds' orderings, which force the parent race.
fn forced(setup: Processes) -> Processes {
    let temp = "store.commit_op.temp_written";
    let renamed = "store.commit_op.renamed";
    setup
        .order(After::End(1), 0, temp, 1)
        .order(After::End(3), 0, temp, 1)
        .order(
            After::Point {
                process: 2,
                name: "store.commit_op.dir_synced",
                nth: 1,
            },
            0,
            temp,
            1,
        )
        .order(
            After::Point {
                process: 0,
                name: renamed,
                nth: 1,
            },
            2,
            LISTING,
            1,
        )
        .order(
            After::Point {
                process: 2,
                name: renamed,
                nth: 2,
            },
            0,
            renamed,
            1,
        )
}

/// Runs the scenario until it ends or the crash. Returns how each process
/// ended (all `Killed` if the run never reached the processes).
fn run(
    fs: &Arc<SimFs>,
    variant: Variant,
    seed: u64,
    kill: Kill,
    log: &Arc<Mutex<Log>>,
    watch: &Watch,
) -> io::Result<Vec<bool>> {
    let root = Path::new(ROOT);
    let mut a = Store::init_variant(Arc::clone(fs), root, ALG, seed, variant)?;
    log.lock().unwrap().init_acked = true;
    record(&mut a, &base_state(), log)?;
    commit(&mut a, &base_op(), log)?;
    drop(a);

    watch.install(fs, log);
    let mut setup = Processes::new(seed ^ 0x5eed_0004);
    if seed % 2 == 1 {
        setup = forced(setup);
    }
    if let Some((process, at)) = kill {
        setup = setup.kill(process, at);
    }
    let base = id_of(&base_op());
    let op0 = id_of(&first_op(0));
    let closures: Vec<Box<dyn FnOnce() -> io::Result<()> + Send + '_>> = (0..PROCESSES)
        .map(|p| {
            let (fs, log) = (Arc::clone(fs), Arc::clone(log));
            Box::new(move || {
                ME.with(|me| me.set(Some(p)));
                let mut store = Store::open_variant(
                    Arc::clone(&fs),
                    root,
                    ALG,
                    seed + 100 + p as u64,
                    variant,
                )?;
                record(&mut store, &first_state(p), &log)?;
                let first = first_op(p);
                commit(&mut store, &first, &log)?;
                if p == 1 {
                    log.lock().unwrap().retry_started = true;
                    commit(&mut store, &first, &log)?;
                    log.lock().unwrap().retried = true;
                }
                if p >= 2 {
                    fs.point(LISTING);
                    if p == 2 {
                        let path = ops_dir().join(file_name(&op0));
                        let shown = format!("{ROOT}/ops/{}", file_name(&op0));
                        let seen = (fs.snapshot().contains_key(&shown), fs.is_durable(&path));
                        log.lock().unwrap().at_listing = Some(seen);
                    }
                    let listed = store.ops()?;
                    let named: BTreeSet<HashRef> =
                        listed.values().flat_map(|o| o.parents.clone()).collect();
                    let own = id_of(&first);
                    let mut parents = vec![own];
                    parents.extend(
                        listed
                            .keys()
                            .filter(|id| **id != own && **id != base && !named.contains(id))
                            .copied(),
                    );
                    let second_state = state(
                        &format!("writer {p} second"),
                        &[&format!("writer {p} second")],
                    );
                    let second = op(&parents, view_of(&second_state));
                    if p == 2 {
                        let mut log = log.lock().unwrap();
                        log.second_of_2 = Some(id_of(&second));
                        log.names_op0 = parents.contains(&op0);
                    }
                    record(&mut store, &second_state, &log)?;
                    commit(&mut store, &second, &log)?;
                }
                Ok(())
            }) as Box<dyn FnOnce() -> io::Result<()> + Send>
        })
        .collect();
    let ended = setup
        .run(fs, closures)
        .unwrap_or_else(|stopped| panic!("seed {seed}: {stopped:?}"));
    fs.clear_points();
    Ok(ended
        .iter()
        .map(|ended| matches!(ended, Ended::Killed))
        .collect())
}

/// The actions a run watches with: process 0's durability at process 2's
/// rename, and step counting. Actions run no syscall.
#[derive(Default)]
pub struct Watch {
    undurable: Arc<AtomicBool>,
    /// Steps per process, when counting.
    counting: Option<Arc<Mutex<[u64; PROCESSES]>>>,
}

impl Watch {
    fn install(&self, fs: &Arc<SimFs>, log: &Arc<Mutex<Log>>) {
        let (watch_fs, watch_log, undurable) =
            (Arc::clone(fs), Arc::clone(log), Arc::clone(&self.undurable));
        let op0 = id_of(&first_op(0));
        fs.on_point("store.commit_op.renamed", move || {
            let Some(second) = watch_log.lock().unwrap().second_of_2 else {
                return;
            };
            let shown = |id: &HashRef| format!("{ROOT}/ops/{}", file_name(id));
            let path = |id: &HashRef| ops_dir().join(file_name(id));
            let visible = watch_fs.snapshot();
            if visible.contains_key(&shown(&second))
                && !watch_fs.is_durable(&path(&second))
                && visible.contains_key(&shown(&op0))
                && !watch_fs.is_durable(&path(&op0))
            {
                undurable.store(true, Ordering::SeqCst);
            }
        });
        if let Some(counts) = &self.counting {
            let names = store::points::POINTS
                .iter()
                .map(|point| point.name)
                .chain([LISTING]);
            for name in names {
                let counts = Arc::clone(counts);
                fs.on_point(name, move || {
                    if let Some(p) = ME.with(Cell::get) {
                        counts.lock().unwrap()[p] += 1;
                    }
                });
            }
            let counts = Arc::clone(counts);
            fs.set_hook(Some(Box::new(move |_| {
                if let Some(p) = ME.with(Cell::get) {
                    counts.lock().unwrap()[p] += 1;
                }
            })));
        }
    }
}

/// Whether an op was renamed into `ops/` while an earlier op's rename there
/// was not yet followed by a sync of `ops/`, in the trace's syscall order.
pub fn overlapped(trace: &[String]) -> bool {
    let renamed = format!("rename {ROOT}/ops/");
    let synced = format!("sync_dir {ROOT}/ops");
    let mut pending = false;
    for line in trace {
        if line.starts_with(&renamed) && !line.ends_with("(crash)") {
            if pending {
                return true;
            }
            pending = true;
        } else if *line == synced {
            pending = false;
        }
    }
    false
}

/// The steps (syscall and point boundaries) each process makes in the
/// seed's uncrashed run without a kill.
fn steps(variant: Variant, mode: Mode, seed: u64) -> [u64; PROCESSES] {
    let fs = Arc::new(SimFs::new(seed, mode));
    let log = Arc::new(Mutex::new(Log::default()));
    let counts = Arc::new(Mutex::new([0; PROCESSES]));
    let watch = Watch {
        counting: Some(Arc::clone(&counts)),
        ..Watch::default()
    };
    run(&fs, variant, seed, None, &log, &watch).expect("an uncrashed run succeeds");
    fs.set_hook(None);
    *counts.lock().unwrap()
}

/// The kill a cell uses: in odd seeds, process 0 on arrival at its held
/// rename; in even seeds, a process and a step within its steps, drawn
/// from the seed.
pub fn kill_for(variant: Variant, mode: Mode, seed: u64, kill: bool) -> Kill {
    if !kill {
        return None;
    }
    if seed % 2 == 1 {
        return Some((
            0,
            KillAt::Point {
                name: "store.commit_op.renamed",
                nth: 1,
            },
        ));
    }
    let steps = steps(variant, mode, seed);
    let mut rng = Rng::new(seed ^ 0x4b11);
    let process = rng.below(PROCESSES as u64) as usize;
    let k = 1 + rng.below(steps[process]);
    Some((process, KillAt::Step(k)))
}

/// An uncrashed run with the cell's kill: the filesystem, the log and
/// which processes were killed.
pub fn uncrashed(
    variant: Variant,
    mode: Mode,
    seed: u64,
    kill: Kill,
) -> (Arc<SimFs>, Log, Vec<bool>) {
    let fs = Arc::new(SimFs::new(seed, mode));
    let log = Arc::new(Mutex::new(Log::default()));
    let killed =
        run(&fs, variant, seed, kill, &log, &Watch::default()).expect("an uncrashed run succeeds");
    let log = log.lock().unwrap().clone();
    (fs, log, killed)
}

/// Runs one cell, returning its outcome and its log.
pub fn run_cell_logged(
    variant: Variant,
    mode: Mode,
    seed: u64,
    kill: Kill,
    crash_at: u64,
) -> (Outcome, Log) {
    let fs = Arc::new(SimFs::new(seed, mode));
    fs.set_crash_at(Some(crash_at));
    let log = Arc::new(Mutex::new(Log::default()));
    let _ = run(&fs, variant, seed, kill, &log, &Watch::default());
    let log = log.lock().unwrap().clone();
    (run_cell(variant, mode, seed, kill, crash_at), log)
}

/// The syscalls an uncrashed run makes, with the cell's kill.
pub fn syscalls(variant: Variant, mode: Mode, seed: u64, kill: Kill) -> (u64, Log, Vec<bool>) {
    let fs = Arc::new(SimFs::new(seed, mode));
    let log = Arc::new(Mutex::new(Log::default()));
    let killed =
        run(&fs, variant, seed, kill, &log, &Watch::default()).expect("an uncrashed run succeeds");
    let log = log.lock().unwrap().clone();
    (fs.syscalls(), log, killed)
}

/// Runs one cell and checks the recovered state.
pub fn run_cell(variant: Variant, mode: Mode, seed: u64, kill: Kill, crash_at: u64) -> Outcome {
    let fs = Arc::new(SimFs::new(seed, mode));
    fs.set_crash_at(Some(crash_at));
    let log = Arc::new(Mutex::new(Log::default()));
    let watch = Watch::default();
    let killed = run(&fs, variant, seed, kill, &log, &watch).unwrap_or_default();
    let reach = fs.reach();
    let trace = fs.trace().join("\n");
    let recovered = fs.recover();
    let log = log.lock().unwrap().clone();
    let (violations, fsck) = check(&recovered, &log, variant, seed);
    Outcome {
        violations,
        fsck,
        undurable_at_rename: watch.undurable.load(Ordering::SeqCst),
        overlapped: overlapped(&fs.trace()),
        killed: killed.iter().any(|k| *k) && kill.is_some(),
        state: digest(&recovered),
        trace: HashAlg::Blake3.hash(trace.as_bytes()),
        reach,
    }
}

/// The invariants on a recovered state: closure; acknowledged ⊆ present ⊆
/// acknowledged ∪ in-flight (attempted); every acknowledged object present;
/// the heads the store reports equal to the heads of the logged ops that are
/// present; no torn or stray file; `fsck` with no problem.
fn check(recovered: &SimFs, log: &Log, variant: Variant, seed: u64) -> (Violations, u64) {
    let mut v = Violations::default();
    let snapshot = recovered.snapshot();
    let (objects, ops) = present(&snapshot);
    let root = Path::new(ROOT);
    for (path, node) in &snapshot {
        let Node::File(content) = node else {
            continue;
        };
        let (dir, name) = path.rsplit_once('/').unwrap_or(("", path));
        let Some(id) = parse_name(name) else {
            continue;
        };
        let valid = object_id(id.alg(), content) == id
            && if dir == format!("{ROOT}/objects") {
                as_object(content).is_some()
            } else if dir == format!("{ROOT}/ops") {
                Operation::from_envelope(content).is_ok()
            } else {
                true
            };
        if !valid {
            v.torn += 1;
        }
    }
    v.closure += closure(&objects, &ops);
    v.durability += log.acked.iter().filter(|id| !ops.contains_key(id)).count() as u64;
    v.durability += log
        .objects_acked
        .iter()
        .filter(|id| !objects.contains_key(id))
        .count() as u64;
    v.heads += ops.keys().filter(|id| !log.attempted.contains(id)).count() as u64;
    let named: BTreeSet<&HashRef> = ops
        .keys()
        .filter_map(|id| log.ops.get(id))
        .flat_map(|op| op.parents.iter())
        .collect();
    let expected: BTreeSet<HashRef> = ops
        .keys()
        .filter(|id| !named.contains(id))
        .copied()
        .collect();
    let mut fsck = 0;
    match Store::open_variant(recovered, root, ALG, seed, variant) {
        Ok(store) => {
            if log.init_acked || !ops.is_empty() {
                match store.heads() {
                    Ok(heads) if heads == expected => {}
                    _ => v.heads += 1,
                }
            }
            if !log.init_acked && !ops.is_empty() {
                v.layout += 1;
            }
            if variant == Variant::Correct {
                match store.fsck() {
                    Ok(report) => fsck += report.problems.len() as u64,
                    Err(_) => fsck += 1,
                }
            }
        }
        Err(error) => {
            if log.init_acked || error.kind() != io::ErrorKind::NotFound {
                v.layout += 1;
            }
        }
    }
    for path in snapshot.keys() {
        let Some(rest) = path.strip_prefix(&format!("{ROOT}/")) else {
            continue;
        };
        let ok = match rest.split_once('/') {
            None => rest == "objects" || rest == "ops",
            Some((dir, name)) => {
                (dir == "objects" || dir == "ops")
                    && (parse_name(name).is_some() || name.starts_with(TMP_PREFIX))
            }
        };
        if !ok {
            v.stray += 1;
        }
    }
    if Store::init(recovered, root, ALG, seed).is_err() {
        v.layout += 1;
    }
    (v, fsck)
}

/// Per mode and kill setting: cells, killed cells and violations.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub cells: BTreeMap<(Mode, bool), u64>,
    pub killed: BTreeMap<(Mode, bool), u64>,
    pub violations: BTreeMap<Mode, Violations>,
    pub fsck: u64,
    /// Odd cells in which process 0's op was not durable at process 2's
    /// rename.
    pub undurable: BTreeMap<Mode, u64>,
}

/// One cell, keyed canonically: (mode, seed, kill setting, crash step).
pub type Cell4 = ((Mode, u64, bool, u64), Outcome);

/// The cells of one (mode, seed): both kill settings, every crash step.
fn seed_cells(variant: Variant, mode: Mode, seed: u64) -> Vec<Cell4> {
    let mut cells = Vec::new();
    for kill_setting in KILLS {
        let kill = kill_for(variant, mode, seed, kill_setting);
        let (n, _, _) = syscalls(variant, mode, seed, kill);
        for crash_at in 1..=n + 1 {
            let outcome = run_cell(variant, mode, seed, kill, crash_at);
            cells.push(((mode, seed, kill_setting, crash_at), outcome));
        }
    }
    cells
}

/// Every cell for `variant` over `modes`, in canonical order: (mode, seed,
/// kill setting, crash step). Seeds run in parallel worker threads, each
/// cell on its own filesystem, so the order they finish in changes nothing
/// once the cells are sorted (plan critique C-001, C-008: the first cost
/// control).
pub fn matrix(variant: Variant, modes: &[Mode]) -> (Summary, Vec<Cell4>) {
    let jobs: Vec<(Mode, u64)> = modes
        .iter()
        .flat_map(|mode| (0..SEEDS).map(move |seed| (*mode, seed)))
        .collect();
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(jobs.len());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    let Some(&(mode, seed)) = jobs.get(i) else {
                        break;
                    };
                    let cells = seed_cells(variant, mode, seed);
                    done.lock().unwrap().extend(cells);
                }
            });
        }
    });
    let mut cells = done.into_inner().unwrap();
    cells.sort_by_key(|(key, _)| *key);
    let mut summary = Summary::default();
    for ((mode, seed, kill_setting, _), outcome) in &cells {
        *summary.cells.entry((*mode, *kill_setting)).or_default() += 1;
        if outcome.killed {
            *summary.killed.entry((*mode, *kill_setting)).or_default() += 1;
        }
        if seed % 2 == 1 && outcome.undurable_at_rename {
            *summary.undurable.entry(*mode).or_default() += 1;
        }
        summary
            .violations
            .entry(*mode)
            .or_default()
            .add(&outcome.violations);
        summary.fsck += outcome.fsck;
    }
    (summary, cells)
}

/// All modes.
pub fn modes() -> &'static [Mode] {
    &MODES
}

/// A digest over every cell's recovered state and trace, in canonical order.
pub fn cells_digest(cells: &[Cell4]) -> HashRef {
    let mut all = Vec::new();
    for (_, outcome) in cells {
        all.extend_from_slice(&outcome.state.to_bytes());
        all.extend_from_slice(&outcome.trace.to_bytes());
    }
    HashAlg::Blake3.hash(&all)
}
