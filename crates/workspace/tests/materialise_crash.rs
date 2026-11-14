//! The Materialise crash scenario (INT-0005 AC-1 to AC-3; INT-0007 AC-1,
//! AC-2; INT-0014 AC-1; INT-0009 AC-3; Sprint 3 T-009). `cargo test --
//! --nocapture` prints the counts.
//!
//! One scenario: `init`; durable edits; snapshot; `new`; durable edits that
//! swap a file and a directory both ways and change an executable file's
//! content; `undo`; `restore` of the snapshot `undo` took. It runs with a
//! crash at every syscall (and after the last), for 40 seeds, in both
//! directory modes. On the state a restart sees, after `open` (which
//! resumes an interrupted write) and before any further edit, it checks:
//! - closure: `fsck` finds no fault;
//! - durability: every acknowledged op is present;
//! - the working copy: for a crash inside `undo` or `restore`, the files are
//!   exactly T if the command's new op survived and F if it did not; for a
//!   crash anywhere else but an edit step, the last acknowledged state's
//!   files; and no record or temporary name is left;
//! - liveness: `open` succeeds (`init`, before `init` was acknowledged), and
//!   a snapshot then records exactly the files on disk.
//!
//! Every edit syncs its files and the directories it changes, so a user's
//! work cannot revert at a later crash; a crash inside an edit is checked
//! for liveness only. An ignored file stands beside the working copy
//! throughout, and after every crash inside `undo` or `restore` the user
//! edits a file that write changes before the next command: both must come
//! through unchanged (code review CR-06).

mod common;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use std::rc::Rc;

use codec::Mode as FileMode;
use common::ROOT;
use hash::{HashAlg, HashRef};
use sim::{Fs, Mode, NameRules, Node, SimFs};
use store::Store;
use workspace::{Variant, Workspace};

const SEEDS: u64 = 40;
/// Seeds whose resuming cells also get a second crash, inside the resume.
const SECOND_PASS_SEEDS: u64 = 10;
const MODES: [Mode; 2] = [Mode::Lenient, Mode::Strict];
const STORE: &str = "repo/.praps/store";

type Files = BTreeMap<String, (Vec<u8>, FileMode)>;

/// The filesystem semantics a matrix runs under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Host {
    /// An executable bit, case-sensitive names, no Windows naming rules.
    Unix,
    /// No executable bit, so "matches F" compares bytes only; names fold
    /// case, and Windows' naming rules apply (test critique C-004).
    Windows,
}

fn new_fs(seed: u64, mode: Mode, host: Host) -> SimFs {
    let fs = SimFs::new(seed, mode);
    if host == Host::Windows {
        fs.set_has_executable_bit(false);
        fs.set_name_rules(NameRules {
            case_insensitive: true,
            windows_names: true,
        });
    }
    fs
}

/// What the run was doing when it stopped.
#[derive(Clone, Debug, Default)]
enum Phase {
    /// Between commands, or in a command that writes no file.
    #[default]
    Idle,
    /// Inside a scripted edit.
    Edit,
    /// Inside `undo` or `restore`, moving the files from F to T.
    Write {
        from: Files,
        target: Files,
        /// The view the command's new op names.
        target_view: HashRef,
        /// Every op before the command started.
        before: BTreeSet<HashRef>,
    },
}

#[derive(Clone, Debug, Default)]
struct Log {
    init_acked: bool,
    acked_ops: Vec<HashRef>,
    /// The files the last acknowledged step left.
    acked: Files,
    phase: Phase,
}

/// The files on disk, from the simulator's snapshot (not a syscall).
fn model(fs: &SimFs) -> Files {
    fs.snapshot()
        .into_iter()
        .filter_map(|(path, node)| {
            let rest = path.strip_prefix("repo/")?;
            if rest.starts_with(".praps/") || rest == ".praps" {
                return None;
            }
            match node {
                Node::File(content) => Some((rest.to_owned(), (content, FileMode::Regular))),
                Node::Executable(content) => {
                    Some((rest.to_owned(), (content, FileMode::Executable)))
                }
                _ => None,
            }
        })
        .collect()
}

fn under(path: &str) -> std::path::PathBuf {
    path.split('/')
        .filter(|c| !c.is_empty())
        .fold(Path::new(ROOT).to_path_buf(), |p, c| p.join(c))
}

/// Writes and syncs a file (its directory must exist).
fn put(fs: &impl Fs, path: &str, content: &[u8]) -> io::Result<()> {
    let file = under(path);
    match fs.create_new(&file) {
        Ok(()) => fs.write(&file, content)?,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs.overwrite(&file, content)?
        }
        Err(error) => return Err(error),
    }
    fs.sync_file(&file)
}

/// The two rounds of durable edits.
fn edit(fs: &impl Fs, round: usize) -> io::Result<()> {
    if round == 0 {
        // An ignored file that no write may touch (code review CR-06).
        put(fs, ".gitignore", b"*.ign\n")?;
        put(fs, "scratch.ign", b"mine")?;
        put(fs, "a", b"alpha")?;
        put(fs, "run", b"#!1")?;
        fs.set_executable(&under("run"), true)?;
        fs.sync_file(&under("run"))?;
        put(fs, "keep", b"kept")?;
        fs.create_dir(&under("d"))?;
        fs.sync_dir(&under(""))?;
        put(fs, "d/x", b"x")?;
        fs.sync_dir(&under("d"))?;
        fs.sync_dir(&under(""))
    } else {
        // `a` becomes a directory and `d` a file; `run` changes content.
        fs.remove_file(&under("a"))?;
        fs.remove_file(&under("d/x"))?;
        fs.sync_dir(&under("d"))?;
        fs.sync_dir(&under(""))?;
        fs.remove_dir(&under("d"))?;
        fs.sync_dir(&under(""))?;
        fs.create_dir(&under("a"))?;
        fs.sync_dir(&under(""))?;
        put(fs, "a/inner", b"inside")?;
        fs.sync_dir(&under("a"))?;
        put(fs, "d", b"now a file")?;
        put(fs, "run", b"#!2")?;
        fs.set_executable(&under("run"), true)?;
        fs.sync_file(&under("run"))?;
        fs.sync_dir(&under(""))
    }
}

fn run(
    fs: &Rc<SimFs>,
    variant: Variant,
    seed: u64,
    log: &RefCell<Log>,
) -> Result<(), workspace::Error> {
    let mut ws = Workspace::init(Rc::clone(fs), Path::new(ROOT), seed)?.0;
    ws.set_variant(variant);
    log.borrow_mut().init_acked = true;
    let heads = ws.store().heads()?;
    log.borrow_mut().acked_ops.extend(heads);

    log.borrow_mut().phase = Phase::Edit;
    edit(&**fs, 0)?;
    let r0 = model(fs);
    {
        let mut log = log.borrow_mut();
        log.phase = Phase::Idle;
        log.acked = r0.clone();
    }
    if let Some(op) = ws.snapshot()?.op {
        log.borrow_mut().acked_ops.push(op);
    }
    ws.new_change()?;
    let heads = ws.store().heads()?;
    log.borrow_mut().acked_ops.extend(heads);

    log.borrow_mut().phase = Phase::Edit;
    edit(&**fs, 1)?;
    let r1 = model(fs);
    {
        let mut log = log.borrow_mut();
        log.phase = Phase::Idle;
        log.acked = r1.clone();
    }

    // `undo`: from the edits (F, snapshotted by `undo`) back to round 0.
    let ops = ws.store().ops()?;
    let h = *ws.store().heads()?.first().unwrap();
    let parent = *ops[&h].parents.first().unwrap();
    log.borrow_mut().phase = Phase::Write {
        from: r1.clone(),
        target: r0.clone(),
        target_view: ops[&parent].view,
        before: ops.keys().copied().collect(),
    };
    let (_, undone) = ws.undo()?;
    {
        let mut log = log.borrow_mut();
        log.phase = Phase::Idle;
        log.acked = r0.clone();
        log.acked_ops.push(undone);
    }

    // `restore` of the snapshot `undo` took: back to the edits.
    let ops = ws.store().ops()?;
    let snapshot = *ops[&undone].parents.first().unwrap();
    log.borrow_mut().phase = Phase::Write {
        from: r0,
        target: r1.clone(),
        target_view: ops[&snapshot].view,
        before: ops.keys().copied().collect(),
    };
    let (_, restored) = ws.restore(&snapshot.to_string())?;
    let mut log = log.borrow_mut();
    log.phase = Phase::Idle;
    log.acked = r1;
    log.acked_ops.push(restored);
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Violations {
    closure: u64,
    durability: u64,
    working_copy: u64,
    liveness: u64,
}

impl Violations {
    fn total(&self) -> u64 {
        self.closure + self.durability + self.working_copy + self.liveness
    }

    fn add(&mut self, other: Violations) {
        self.closure += other.closure;
        self.durability += other.durability;
        self.working_copy += other.working_copy;
        self.liveness += other.liveness;
    }
}

/// The working-copy change's files, or `None` if they cannot be read.
fn recorded<F: Fs>(ws: &Workspace<F>) -> Option<Files> {
    let (_, _, tree) = ws.working_copy().ok()?;
    let mut files = Files::new();
    for (path, file) in &tree {
        let envelope = ws.store().get_object(&file.blob).ok()?;
        files.insert(
            path.as_str().to_owned(),
            (envelope[2..].to_vec(), file.mode),
        );
    }
    Some(files)
}

/// Checks the state a restart sees. It works on copies, so `recovered`
/// stays as the crash left it.
/// The invariants, and the reach of the next command the scenario models
/// (the `open` that resumes, or `init`), read before the oracle checks.
fn check(recovered: &SimFs, log: &Log, seed: u64) -> (Violations, Vec<(&'static str, u64)>) {
    let mut v = Violations::default();
    let root = Path::new(ROOT);
    if let Ok(store) = Store::inspect(recovered, Path::new(STORE), HashAlg::DEFAULT) {
        v.closure += store.fsck().unwrap().problems.len() as u64;
        let ops = store.ops().unwrap();
        v.durability += log
            .acked_ops
            .iter()
            .filter(|id| !ops.contains_key(id))
            .count() as u64;
    } else {
        v.durability += u64::from(log.init_acked);
    }
    let copy = recovered.recover();
    // After a crash inside `undo` or `restore`, the user edits a file the
    // write changes, before the next command: resume must keep it (code
    // review CR-06). `run` exists in F and T, and at every point between.
    let mut user_edit = None;
    if matches!(log.phase, Phase::Write { .. }) {
        let run = Path::new(ROOT).join("run");
        if copy.overwrite(&run, b"edited after the crash").is_ok() && copy.sync_file(&run).is_ok() {
            user_edit = model(&copy).get("run").cloned();
        }
    }
    let opened = if log.init_acked {
        Workspace::open(&copy, root, seed)
    } else {
        Workspace::init(&copy, root, seed).map(|(ws, _)| ws)
    };
    let next = copy.reach();
    let Ok(mut ws) = opened else {
        v.liveness += 1;
        return (v, next);
    };
    let expected = match &log.phase {
        Phase::Edit => None,
        Phase::Idle => Some(log.acked.clone()),
        Phase::Write {
            from,
            target,
            target_view,
            before,
        } => {
            let ops = ws.store().ops().unwrap();
            let survived = ops
                .iter()
                .any(|(id, op)| !before.contains(id) && op.view == *target_view);
            Some(if survived {
                target.clone()
            } else {
                from.clone()
            })
        }
    };
    if let Some(mut expected) = expected {
        if let Some(edited) = user_edit {
            expected.insert("run".to_owned(), edited);
        }
        if model(&copy) != expected {
            v.working_copy += 1;
        }
    }
    if copy.snapshot().contains_key("repo/.praps/working-copy") {
        v.working_copy += 1;
    }
    // The snapshot records every file but the ignored ones. A crash inside
    // the first edit may leave `.gitignore` torn to a prefix of `*.ign\n`,
    // read as Git reads it: `*` ignores everything, `*.ign` (with or without
    // its newline) the ignored file, and the other prefixes nothing here.
    let mut tracked = model(&copy);
    let rule = tracked.get(".gitignore").map(|(content, _)| {
        String::from_utf8_lossy(content.strip_suffix(b"\n").unwrap_or(content)).into_owned()
    });
    match rule.as_deref() {
        Some("*") => tracked.clear(),
        Some("*.ign") => tracked.retain(|path, _| !path.ends_with(".ign")),
        _ => {}
    }
    let live = ws.snapshot().is_ok() && recorded(&ws) == Some(tracked);
    v.liveness += u64::from(!live);
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
            Node::File(content) => {
                bytes.push(b'f');
                bytes.extend_from_slice(&(content.len() as u64).to_le_bytes());
                bytes.extend_from_slice(&content);
            }
            Node::Executable(content) => {
                bytes.push(b'x');
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

/// The second pass for one resuming cell: a crash at every syscall of the
/// recovering `open`, then recovery, `open` again, and the same oracle.
/// Returns the cells run and their violations.
fn second_pass(recovered: &SimFs, log: &Log, seed: u64) -> (u64, Violations) {
    let syscalls = {
        let copy = recovered.recover();
        let _ = Workspace::open(&copy, Path::new(ROOT), seed);
        copy.syscalls()
    };
    let mut v = Violations::default();
    for k in 1..=syscalls {
        let copy = recovered.recover();
        copy.set_crash_at(Some(k));
        let _ = Workspace::open(&copy, Path::new(ROOT), seed);
        v.add(check(&copy.recover(), log, seed).0);
    }
    (syscalls, v)
}

struct Matrix {
    cells: Vec<Cell>,
    resuming: u64,
    second: (u64, Violations),
}

fn matrix(variant: Variant, second: bool) -> Matrix {
    matrix_on(variant, second, Host::Unix)
}

fn matrix_on(variant: Variant, second: bool, host: Host) -> Matrix {
    let mut out = Matrix {
        cells: Vec::new(),
        resuming: 0,
        second: (0, Violations::default()),
    };
    let label = match host {
        Host::Unix => format!("{variant:?}"),
        Host::Windows => format!("{variant:?} (Windows rules)"),
    };
    for mode in MODES {
        for seed in 0..SEEDS {
            let syscalls = {
                let fs = Rc::new(new_fs(seed, mode, host));
                let log = RefCell::new(Log::default());
                run(&fs, variant, seed, &log).expect("an uncrashed run succeeds");
                fs.syscalls()
            };
            for crash_at in 1..=syscalls + 1 {
                let fs = Rc::new(new_fs(seed, mode, host));
                fs.set_crash_at(Some(crash_at));
                let log = RefCell::new(Log::default());
                let _ = run(&fs, variant, seed, &log);
                let recovered = fs.recover();
                let log = log.into_inner();
                if recovered
                    .snapshot()
                    .contains_key("repo/.praps/working-copy")
                {
                    out.resuming += 1;
                    if second && seed < SECOND_PASS_SEEDS {
                        let (cells, v) = second_pass(&recovered, &log, seed);
                        out.second.0 += cells;
                        out.second.1.add(v);
                    }
                }
                let (violations, next) = check(&recovered, &log, seed);
                let mut reach = fs.reach();
                reach.extend(next);
                out.cells.push(Cell {
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
    for cell in &out.cells {
        let entry = sums.entry(format!("{:?}", cell.mode)).or_default();
        entry.0 += 1;
        entry.1.add(cell.violations);
    }
    for (mode, (count, v)) in &sums {
        println!(
            "materialise matrix {label} {mode}: {count} cells; violations: closure {} durability {} working copy {} liveness {}",
            v.closure, v.durability, v.working_copy, v.liveness
        );
    }
    println!(
        "materialise matrix {label}: {} cells resumed a record; second pass {} cells, {} violations",
        out.resuming,
        out.second.0,
        out.second.1.total()
    );
    out
}

#[test]
fn materialise_matrix_finds_no_violations() {
    let m = matrix(Variant::Correct, false);
    assert!(m.resuming > 0, "no crash left a record to resume");
    let failing: Vec<_> = m
        .cells
        .iter()
        .filter(|c| c.violations.total() > 0)
        .take(5)
        .map(|c| (c.mode, c.seed, c.crash_at, c.violations))
        .collect();
    assert!(failing.is_empty(), "first violating cells: {failing:?}");
    // The designated record of this scenario's reach (INT-0005 AC-6).
    let mut totals = sim::reach::Totals::new();
    for cell in &m.cells {
        sim::reach::add(&mut totals, &cell.reach);
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("reach");
    sim::reach::record(&dir, "materialise", "Materialise", 3, &totals).unwrap();
}

/// The same scenario on a filesystem with no executable bit and Windows'
/// name rules: the bytes-only "matches F" rule and the case-folded checks
/// under a crash at every syscall (test critique C-004).
#[test]
fn materialise_matrix_with_windows_rules_finds_no_violations() {
    let m = matrix_on(Variant::Correct, false, Host::Windows);
    assert!(m.resuming > 0, "no crash left a record to resume");
    let failing: Vec<_> = m
        .cells
        .iter()
        .filter(|c| c.violations.total() > 0)
        .take(5)
        .map(|c| (c.mode, c.seed, c.crash_at, c.violations))
        .collect();
    assert!(failing.is_empty(), "first violating cells: {failing:?}");
}

#[test]
fn resume_survives_a_second_crash() {
    let m = matrix(Variant::Correct, true);
    assert!(m.second.0 > 0, "no second-pass cell ran");
    assert_eq!(m.second.1, Violations::default());
}

#[test]
fn materialise_matrix_is_reproducible() {
    let first = matrix(Variant::Correct, false);
    let second = matrix(Variant::Correct, false);
    assert_eq!(first.cells.len(), second.cells.len());
    let mut all = Vec::new();
    for (a, b) in first.cells.iter().zip(&second.cells) {
        let at = (a.mode, a.seed, a.crash_at);
        assert_eq!(at, (b.mode, b.seed, b.crash_at));
        assert_eq!(a.state, b.state, "cell {at:?}");
        assert_eq!(a.trace, b.trace, "cell {at:?}");
        assert_eq!(a.violations, b.violations, "cell {at:?}");
        all.extend_from_slice(&a.state.to_bytes());
        all.extend_from_slice(&a.trace.to_bytes());
    }
    let digest = HashAlg::Blake3.hash(&all);
    println!(
        "materialise matrix digest over {} cells: {digest}",
        first.cells.len()
    );
    // Asserted, not only printed, so a change that moves any cell fails
    // here on every system (Sprint 4 plan critique, C-008).
    assert_eq!(first.cells.len(), 34_400);
    assert_eq!(
        digest.to_string(),
        "blake3:1992301c25ffe9794573fe5c3eacebf0c8dd865333a6e0ae06fced55e6d0319a"
    );
}

/// The negative control: without the record, a crash mid-write leaves a
/// working copy that is neither F nor T, in both modes (INT-0005 AC-3).
#[test]
fn materialise_catches_no_record() {
    let m = matrix(Variant::NoRecord, false);
    for mode in MODES {
        let caught: u64 = m
            .cells
            .iter()
            .filter(|c| c.mode == mode)
            .map(|c| c.violations.working_copy)
            .sum();
        assert!(
            caught > 0,
            "{mode:?}: NoRecord produced no working-copy violation"
        );
    }
}
