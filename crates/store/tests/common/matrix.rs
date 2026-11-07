//! The crash matrix (INT-0005 AC-1 to AC-3; INT-0007 AC-1 to AC-3).
//!
//! Each cell runs one scenario against a store on a [`SimFs`] with one seed,
//! one directory mode and one crash point, recovers the filesystem a restart
//! would see, and checks the invariants on that recovered state directly,
//! whether or not the store opens on it. A missing `objects/` or `ops/` counts
//! as empty. "Present" means a file whose content hashes to its name and
//! decodes, as `fsck` judges a file.
//!
//! Every op names a view; a view names a changeset; a changeset names blobs
//! (Sprint 2). Closure follows those references.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use codec::{
    Changeset, Entry, Mode as FileMode, ObjectKind, Operation, RepoPath, View, blob,
    decode_envelope, object_id, references,
};
use hash::{HashAlg, HashRef};
use sim::{Mode, Node, SimFs};
use store::names::{TMP_PREFIX, file_name, parse_name};
use store::{Store, Variant, heads_of};

/// Seeds per scenario and mode (INT-0005 AC-2: at least 40).
pub const SEEDS: u64 = 40;
pub const MODES: [Mode; 2] = [Mode::Lenient, Mode::Strict];
pub const ROOT: &str = "repo";
const ALG: HashAlg = HashAlg::DEFAULT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scenario {
    /// `init` alone.
    Init,
    /// `init`, then a chain of five ops, each with a new recorded state.
    Chain,
    /// `init`, a root op, two children, and a merge of both.
    ForkMerge,
    /// `init`, then writer A records state Y and commits a root op naming its
    /// view, records state X and commits a child op naming X's view. Writer
    /// B, opened after the root op, runs at one fixed boundary in every seed
    /// and never writes what it names:
    /// - even seeds (the view race): after A's `rename` of X's view and
    ///   before A's `sync_dir(objects)`, B commits an op naming X's view;
    /// - odd seeds (the parent race): after A's `rename` of its child op and
    ///   before A's `sync_dir(ops)`, B commits an op whose parent is A's
    ///   child op.
    ///
    /// Only `commit_op`'s re-sync of the view and of the parent ops makes B's
    /// op safe (INT-0007 Consequences).
    TwoWriters,
}

impl Scenario {
    pub const ALL: [Scenario; 4] = [
        Scenario::Init,
        Scenario::Chain,
        Scenario::ForkMerge,
        Scenario::TwoWriters,
    ];
}

/// Which race a two-writer cell ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Race {
    /// B named A's view while its name was not yet durable.
    View,
    /// B named A's op as a parent while its name was not yet durable.
    Parent,
}

/// What a run attempted and what the store acknowledged.
#[derive(Clone, Debug, Default)]
pub struct Log {
    pub init_acked: bool,
    /// Every op a commit was attempted for, in order.
    pub attempted: Vec<HashRef>,
    /// Every op whose commit returned, in order.
    pub acked: Vec<HashRef>,
    /// Every object whose `put_object` returned.
    pub objects_acked: Vec<HashRef>,
    /// The intended single-writer chain, for the prefix check.
    pub chain: Vec<HashRef>,
    /// Two writers: B's op was acknowledged while the name it depends on,
    /// which A had renamed into place, was not yet durable.
    pub raced: Option<Race>,
}

/// Invariant violations in one recovered state, by kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Violations {
    /// After an acknowledged init the store does not open, or after an
    /// unacknowledged one it holds ops, or `init` fails on the recovered state.
    pub layout: u64,
    /// A present op names a parent op or a view that is not present, or an
    /// object reachable from a present op names an object that is not.
    pub closure: u64,
    /// An acknowledged op or object is not present.
    pub durability: u64,
    /// A present op was never attempted, or the heads the store reports differ
    /// from the heads of the present ops.
    pub heads: u64,
    /// The present ops of the chain are not a prefix of the intended chain.
    pub atomicity: u64,
    /// A file under an object or op name that is not wholly that object or
    /// op: its content does not hash to its name or does not decode
    /// (INT-0007 AC-3: in-flight ops are wholly present and valid, or absent).
    pub torn: u64,
    /// A file other than objects, ops and temporary files exists.
    pub stray: u64,
}

impl Violations {
    /// Violations of what a crash may never do to acknowledged state: the
    /// kinds the negative controls must produce.
    pub fn semantic(&self) -> u64 {
        self.closure + self.durability + self.heads + self.atomicity + self.torn
    }

    pub fn total(&self) -> u64 {
        self.semantic() + self.layout + self.stray
    }

    pub fn add(&mut self, other: &Violations) {
        self.layout += other.layout;
        self.closure += other.closure;
        self.durability += other.durability;
        self.heads += other.heads;
        self.atomicity += other.atomicity;
        self.torn += other.torn;
        self.stray += other.stray;
    }
}

/// One cell's outcome.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub violations: Violations,
    /// The two-writer race the cell ran, if any (see [`Log::raced`]).
    pub raced: Option<Race>,
    /// A hash of the recovered state.
    pub state: HashRef,
    /// A hash of the syscall trace.
    pub trace: HashRef,
}

fn id_of(op: &Operation) -> HashRef {
    object_id(ALG, &op.to_envelope())
}

fn op(parents: &[HashRef], view: HashRef) -> Operation {
    Operation {
        parents: parents.iter().copied().collect(),
        view,
    }
}

/// The objects one recorded state needs, in put order: a blob per file, a
/// changeset holding them, and a view whose head and `default` workspace are
/// that changeset.
fn state(tag: &str, files: &[&str]) -> Vec<Vec<u8>> {
    let mut envelopes = Vec::new();
    let mut changes = BTreeMap::new();
    for (i, content) in files.iter().enumerate() {
        let envelope = blob(content.as_bytes());
        changes.insert(
            RepoPath::new(format!("f{i}")).unwrap(),
            Entry::Put {
                blob: object_id(ALG, &envelope),
                mode: FileMode::Regular,
            },
        );
        envelopes.push(envelope);
    }
    let changeset = Changeset {
        parents: Vec::new(),
        change_id: *ALG.hash(tag.as_bytes()).digest().first_chunk().unwrap(),
        changes,
        description: tag.as_bytes().to_vec(),
    }
    .to_envelope();
    let changeset_id = object_id(ALG, &changeset);
    envelopes.push(changeset);
    envelopes.push(
        View {
            heads: [changeset_id].into(),
            workspaces: [("default".to_owned(), changeset_id)].into(),
        }
        .to_envelope(),
    );
    envelopes
}

fn view_of(state: &[Vec<u8>]) -> HashRef {
    object_id(ALG, state.last().unwrap())
}

fn put(store: &mut Store<Rc<SimFs>>, envelope: &[u8], log: &RefCell<Log>) -> io::Result<HashRef> {
    let id = store.put_object(envelope)?;
    log.borrow_mut().objects_acked.push(id);
    Ok(id)
}

/// Puts a state's objects in order and returns its view.
fn record(
    store: &mut Store<Rc<SimFs>>,
    state: &[Vec<u8>],
    log: &RefCell<Log>,
) -> io::Result<HashRef> {
    for envelope in state {
        put(store, envelope, log)?;
    }
    Ok(view_of(state))
}

fn commit(store: &mut Store<Rc<SimFs>>, op: &Operation, log: &RefCell<Log>) -> io::Result<()> {
    let id = id_of(op);
    log.borrow_mut().attempted.push(id);
    store.commit_op(op)?;
    log.borrow_mut().acked.push(id);
    Ok(())
}

/// Runs a scenario until it ends or the first error (the crash).
fn run(
    fs: &Rc<SimFs>,
    scenario: Scenario,
    variant: Variant,
    seed: u64,
    log: &Rc<RefCell<Log>>,
) -> io::Result<()> {
    let root = Path::new(ROOT);
    let mut a = Store::init_variant(Rc::clone(fs), root, ALG, seed, variant)?;
    log.borrow_mut().init_acked = true;
    match scenario {
        Scenario::Init => Ok(()),
        Scenario::Chain => {
            let states: Vec<_> = (0..5)
                .map(|i| state(&format!("chain {i}"), &[&format!("object {i}")]))
                .collect();
            let mut parents = Vec::new();
            let mut ops = Vec::new();
            for state in &states {
                let next = op(&parents, view_of(state));
                parents = vec![id_of(&next)];
                ops.push(next);
            }
            log.borrow_mut().chain = ops.iter().map(id_of).collect();
            for (state, next) in states.iter().zip(&ops) {
                record(&mut a, state, log)?;
                commit(&mut a, next, log)?;
            }
            Ok(())
        }
        Scenario::ForkMerge => {
            let base = op(&[], record(&mut a, &state("base", &[]), log)?);
            commit(&mut a, &base, log)?;
            let left = op(
                &[id_of(&base)],
                record(&mut a, &state("left", &["left"]), log)?,
            );
            commit(&mut a, &left, log)?;
            let right = op(
                &[id_of(&base)],
                record(&mut a, &state("right", &["right"]), log)?,
            );
            commit(&mut a, &right, log)?;
            let merged = record(&mut a, &state("merge", &["left", "right"]), log)?;
            commit(&mut a, &op(&[id_of(&left), id_of(&right)], merged), log)
        }
        Scenario::TwoWriters => {
            let y = record(&mut a, &state("Y", &["Y"]), log)?;
            let base = op(&[], y);
            commit(&mut a, &base, log)?;
            // B opens now: `open` syncs the layout, which inside A's writes
            // would also make A's pending names durable and hide the race.
            let b = Store::open_variant(
                Rc::clone(fs),
                Path::new(ROOT),
                ALG,
                seed.wrapping_add(1_000),
                variant,
            )?;
            let x = state("X", &["X"]);
            let child = op(&[id_of(&base)], view_of(&x));
            let (race, target, b_op) = if seed.is_multiple_of(2) {
                (
                    Race::View,
                    Path::new(ROOT)
                        .join("objects")
                        .join(file_name(&view_of(&x))),
                    op(&[], view_of(&x)),
                )
            } else {
                (
                    Race::Parent,
                    Path::new(ROOT).join("ops").join(file_name(&id_of(&child))),
                    op(&[id_of(&child)], y),
                )
            };
            install_race(fs, log, race, target, b, b_op);
            let result = (|| -> io::Result<()> {
                record(&mut a, &x, log)?;
                commit(&mut a, &child, log)
            })();
            fs.set_hook(None);
            result
        }
    }
}

/// Fires B once, at the first syscall boundary right after A renamed
/// `target` into place while its name is not yet durable.
fn install_race(
    fs: &Rc<SimFs>,
    log: &Rc<RefCell<Log>>,
    race: Race,
    target: PathBuf,
    mut b: Store<Rc<SimFs>>,
    b_op: Operation,
) {
    let (hook_fs, hook_log) = (Rc::clone(fs), Rc::clone(log));
    let shown = target.to_string_lossy().replace('\\', "/");
    let mut fired = false;
    fs.set_hook(Some(Box::new(move |_next| {
        if fired {
            return;
        }
        // None of these checks is a syscall, so none moves the crash point.
        let after_rename = hook_fs
            .trace()
            .last()
            .is_some_and(|line| line.starts_with("rename "));
        if !after_rename || !hook_fs.snapshot().contains_key(&shown) || hook_fs.is_durable(&target)
        {
            return;
        }
        fired = true;
        // B's commit may fail (the crash); the log shows whether it was
        // acknowledged.
        if commit(&mut b, &b_op, &hook_log).is_ok() {
            hook_log.borrow_mut().raced = Some(race);
        }
    })));
}

/// The syscalls a run makes when nothing crashes.
pub fn syscalls(variant: Variant, scenario: Scenario, mode: Mode, seed: u64) -> u64 {
    let fs = Rc::new(SimFs::new(seed, mode));
    let log = Rc::new(RefCell::new(Log::default()));
    run(&fs, scenario, variant, seed, &log).expect("an uncrashed run succeeds");
    // Every seed races in the correct store; a variant may never reach the
    // boundary (write-in-place has no rename).
    if scenario == Scenario::TwoWriters && variant == Variant::Correct {
        assert!(
            log.borrow().raced.is_some(),
            "seed {seed}: the uncrashed two-writer run must race"
        );
    }
    fs.syscalls()
}

/// Runs one cell: the scenario with a crash at `crash_at`, then recovery and
/// the checks. Returns the outcome and the recovered filesystem.
pub fn run_cell(
    variant: Variant,
    scenario: Scenario,
    mode: Mode,
    seed: u64,
    crash_at: u64,
) -> (Outcome, SimFs) {
    let fs = Rc::new(SimFs::new(seed, mode));
    fs.set_crash_at(Some(crash_at));
    let log = Rc::new(RefCell::new(Log::default()));
    let _ = run(&fs, scenario, variant, seed, &log);
    let trace = fs.trace().join("\n");
    // Recovery is a function of the crashed state and the seed, so the
    // checks (whose final `init` writes) get their own copy and the state
    // returned, hashed and handed on is the one the crash left.
    let recovered = fs.recover();
    let log = log.borrow().clone();
    let violations = check(&fs.recover(), &log, scenario, variant, seed);
    let outcome = Outcome {
        violations,
        raced: log.raced,
        state: digest(&recovered),
        trace: HashAlg::Blake3.hash(trace.as_bytes()),
    };
    (outcome, recovered)
}

/// A hash of everything visible in a filesystem.
pub fn digest(fs: &SimFs) -> HashRef {
    let mut bytes = Vec::new();
    for (path, node) in fs.snapshot() {
        bytes.extend_from_slice(path.as_bytes());
        bytes.push(0);
        match node {
            Node::Dir => bytes.push(b'd'),
            Node::File(content) => {
                bytes.push(b'f');
                bytes.extend_from_slice(&(content.len() as u64).to_le_bytes());
                bytes.extend_from_slice(&content);
            }
        }
    }
    HashAlg::Blake3.hash(&bytes)
}

/// A valid object: its kind and the ids it names.
struct Object {
    kind: ObjectKind,
    refs: Vec<HashRef>,
}

/// An object file's content, if it is wholly a valid object (not an op).
fn as_object(content: &[u8]) -> Option<Object> {
    let kind = decode_envelope(content).ok()?.kind;
    if kind == ObjectKind::Operation {
        return None;
    }
    Some(Object {
        kind,
        refs: references(content).ok()?,
    })
}

/// The present objects and ops under the store root, read from the raw state.
fn present(
    snapshot: &BTreeMap<String, Node>,
) -> (BTreeMap<HashRef, Object>, BTreeMap<HashRef, Operation>) {
    let mut objects = BTreeMap::new();
    let mut ops = BTreeMap::new();
    for (path, node) in snapshot {
        let Node::File(content) = node else {
            continue;
        };
        let (dir, name) = path.rsplit_once('/').unwrap_or(("", path));
        let Some(id) = parse_name(name) else {
            continue;
        };
        if object_id(id.alg(), content) != id {
            continue;
        }
        if dir == format!("{ROOT}/objects")
            && let Some(object) = as_object(content)
        {
            objects.insert(id, object);
        } else if dir == format!("{ROOT}/ops")
            && let Ok(op) = Operation::from_envelope(content)
        {
            ops.insert(id, op);
        }
    }
    (objects, ops)
}

/// Closure violations: missing parents and views of present ops, and missing
/// references of every object those views reach.
fn closure(objects: &BTreeMap<HashRef, Object>, ops: &BTreeMap<HashRef, Operation>) -> u64 {
    let mut count = 0;
    let mut stack = Vec::new();
    for op in ops.values() {
        count += op.parents.iter().filter(|p| !ops.contains_key(p)).count() as u64;
        match objects.get(&op.view) {
            Some(object) if object.kind == ObjectKind::View => stack.push(op.view),
            _ => count += 1,
        }
    }
    let mut seen = BTreeSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        for reference in &objects[&id].refs {
            if objects.contains_key(reference) {
                stack.push(*reference);
            } else {
                count += 1;
            }
        }
    }
    count
}

fn check(
    recovered: &SimFs,
    log: &Log,
    scenario: Scenario,
    variant: Variant,
    seed: u64,
) -> Violations {
    let mut v = Violations::default();
    let snapshot = recovered.snapshot();
    let (objects, ops) = present(&snapshot);
    let root = Path::new(ROOT);

    // Torn: a file under an object or op name that is not wholly it.
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
    // Durability, of acknowledged ops and of acknowledged objects.
    v.durability += log.acked.iter().filter(|id| !ops.contains_key(id)).count() as u64;
    v.durability += log
        .objects_acked
        .iter()
        .filter(|id| !objects.contains_key(id))
        .count() as u64;
    // Heads: only attempted ops, and the heads the store reports.
    let attempted: BTreeSet<&HashRef> = log.attempted.iter().collect();
    v.heads += ops.keys().filter(|id| !attempted.contains(id)).count() as u64;
    match Store::open_variant(recovered, root, ALG, seed, variant) {
        Ok(store) => {
            if log.init_acked || !ops.is_empty() {
                match store.heads() {
                    Ok(heads) if heads == heads_of(&ops) => {}
                    _ => v.heads += 1,
                }
            }
            if !log.init_acked && !ops.is_empty() {
                v.layout += 1;
            }
        }
        Err(error) => {
            if log.init_acked || error.kind() != io::ErrorKind::NotFound {
                v.layout += 1;
            }
        }
    }
    // Atomicity: the present part of the chain is a prefix of it.
    if scenario == Scenario::Chain {
        let kept: Vec<bool> = log.chain.iter().map(|id| ops.contains_key(id)).collect();
        let prefix = kept.iter().take_while(|kept| **kept).count();
        if kept[prefix..].iter().any(|kept| *kept) {
            v.atomicity += 1;
        }
    }
    // Stray files.
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
    // Init always completes the recovered state.
    if Store::init(recovered, root, ALG, seed).is_err() {
        v.layout += 1;
    }
    v
}

/// Per mode: cells run and violations found.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub cells: BTreeMap<Mode, u64>,
    /// Cells per mode and race kind that ran a two-writer race.
    pub races: BTreeMap<(Mode, Race), u64>,
    pub violations: BTreeMap<Mode, Violations>,
}

impl Summary {
    pub fn races(&self, mode: Mode, race: Race) -> u64 {
        self.races.get(&(mode, race)).copied().unwrap_or(0)
    }

    pub fn print(&self, variant: Variant) {
        for (mode, cells) in &self.cells {
            let v = self.violations[mode];
            println!(
                "matrix {:<16} {mode:?}: {cells} cells ({} view races, {} parent races); \
                 violations: closure {} durability {} heads {} atomicity {} torn {} layout {} \
                 stray {}",
                variant.name(),
                self.races(*mode, Race::View),
                self.races(*mode, Race::Parent),
                v.closure,
                v.durability,
                v.heads,
                v.atomicity,
                v.torn,
                v.layout,
                v.stray
            );
        }
    }
}

/// Runs every cell for `variant`: each scenario, mode and seed, with a crash
/// at every syscall of the uncrashed run and once after its last syscall.
/// `visit` sees each cell's outcome and recovered filesystem.
pub fn run_matrix(
    variant: Variant,
    mut visit: impl FnMut(Scenario, Mode, u64, u64, &Outcome, &SimFs),
) -> Summary {
    let mut summary = Summary::default();
    for scenario in Scenario::ALL {
        for mode in MODES {
            for seed in 0..SEEDS {
                let n = syscalls(variant, scenario, mode, seed);
                for crash_at in 1..=n + 1 {
                    let (outcome, recovered) = run_cell(variant, scenario, mode, seed, crash_at);
                    *summary.cells.entry(mode).or_default() += 1;
                    if let Some(race) = outcome.raced {
                        *summary.races.entry((mode, race)).or_default() += 1;
                    }
                    summary
                        .violations
                        .entry(mode)
                        .or_default()
                        .add(&outcome.violations);
                    visit(scenario, mode, seed, crash_at, &outcome, &recovered);
                }
            }
        }
    }
    summary
}
