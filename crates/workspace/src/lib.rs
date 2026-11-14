#![forbid(unsafe_code)]
//! The working copy of a Periapsis repository (INT-0014).
//!
//! A repository is a directory, the working copy, with the store in
//! `.praps/store`. The working copy is itself a change: every command that
//! opens it first snapshots it into the working-copy change, so no command
//! discards uncommitted work. A snapshot stores each file's bytes exactly
//! (INT-0034), follows `.gitignore` rules ([`ignore`]), and reports, without
//! tracking or touching them, entries it cannot store: names with no UTF-8
//! form (INT-0004 AC-7), names that are not valid repository paths, symbolic
//! links and special files.
//!
//! All filesystem I/O goes through [`sim::Fs`] (INT-0005), and every change ID
//! comes from an [`Rng`] seeded by the caller, so a run on the simulator is a
//! function of its seed. `docs/spec/working-copy.md` is the specification.

pub mod hazard;
mod ignore;
pub mod lane;
pub mod materialise;
pub mod merge;
pub mod points;
pub mod record;
pub mod tree;
#[cfg(feature = "variants")]
mod variant;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use codec::{
    CHANGE_ID_LEN, Changeset, Mode, Operation, RepoPath, View, WorkspaceEntry, blob,
    decode_envelope, object_id,
};
use hash::{HashAlg, HashRef};
use sim::{Fs, Kind, Name, Rng, dir_exists, parent_dir};
use store::{OpLog, Store};

pub use crate::ignore::ALWAYS_IGNORED;
use crate::ignore::Ignores;
use crate::record::Record;
use crate::tree::{File, Tree};
#[cfg(feature = "variants")]
pub use crate::variant::Variant;

/// The directory holding the repository's own data.
pub const PRAPS_DIR: &str = ".praps";
/// The store, inside [`PRAPS_DIR`].
pub const STORE_DIR: &str = "store";
/// The name of the one workspace this sprint supports.
pub const DEFAULT_WORKSPACE: &str = "default";
const ALG: HashAlg = HashAlg::DEFAULT;

/// Why a workspace command failed.
#[derive(Debug)]
pub enum Error {
    /// The filesystem failed.
    Io(io::Error),
    /// The directory holds no repository, or one whose `init` never finished.
    NotInitialised(PathBuf),
    /// The repository's state does not decode or is incomplete.
    Corrupt(String),
    /// A changeset with more than one parent, whose tree needs INT-0012's
    /// merge.
    Merge(HashRef),
    /// Writing the working copy would hit these hazards (INT-0004 AC-3,
    /// AC-4). Nothing but the opening snapshot was written.
    Hazards(Vec<hazard::Hazard>),
    /// Entries on disk stand where the write must put files or directories
    /// (each described). Nothing but the opening snapshot was written.
    Blocked(Vec<String>),
    /// `undo` found no earlier op to return to: the op head when it started
    /// was the root op.
    NothingToUndo,
    /// `restore` was given an op id or prefix that names no op.
    UnknownOp(String),
    /// `restore` was given a prefix that names several ops.
    AmbiguousOp(String, Vec<HashRef>),
    /// Making a lane: the name is already a workspace's. Nothing but the
    /// opening snapshot was written.
    LaneExists(String),
    /// Making a lane: the directory holds more than an interrupted making
    /// leaves. Nothing but the opening snapshot was written.
    NotFree(PathBuf),
    /// A lane whose `.praps/workspace` names a workspace no op made: its
    /// making stopped before its op was committed.
    UnfinishedLane(String, PathBuf),
    /// The store a lane's `.praps/workspace` names does not open.
    StoreGone(PathBuf, String),
    /// A lane's `.praps/workspace` cannot be read or does not parse.
    BadWorkspaceFile(PathBuf, String),
    /// `restore` was given an op whose view holds no entry for this
    /// workspace: one made before the workspace was. Nothing but the
    /// opening snapshot was written.
    NotInWorkspace(HashRef, String),
}

/// Each message says only what is wrong. What to do about it is said once,
/// by whoever prints the error: `praps` ends every refusal with a hint
/// (INT-0037 AC-4), and `messages_say_only_what_is_wrong` fixes each text.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{error}"),
            Error::NotInitialised(root) => write!(
                f,
                "{} is not an initialised Periapsis repository",
                root.display()
            ),
            Error::Corrupt(what) => write!(f, "the repository is corrupt: {what}"),
            Error::Merge(id) => write!(
                f,
                "changeset {id} has several parents; merges are not supported yet"
            ),
            Error::Hazards(hazards) => {
                write!(
                    f,
                    "the working copy cannot be written on this filesystem; no file was changed:"
                )?;
                for hazard in hazards {
                    write!(f, "\n  {hazard}")?;
                }
                Ok(())
            }
            Error::Blocked(entries) => {
                write!(
                    f,
                    "untracked entries are in the way, and no file was changed:"
                )?;
                for entry in entries {
                    write!(f, "\n  {entry}")?;
                }
                Ok(())
            }
            Error::NothingToUndo => {
                f.write_str("nothing to undo: this workspace's last op is the one that made it")
            }
            Error::UnknownOp(spec) => write!(f, "no op matches `{spec}`"),
            Error::AmbiguousOp(spec, candidates) => {
                write!(f, "`{spec}` matches {} ops:", candidates.len())?;
                for candidate in candidates {
                    write!(f, "\n  {candidate}")?;
                }
                Ok(())
            }
            Error::LaneExists(name) => write!(f, "{name} is already a workspace"),
            Error::NotFree(dir) => write!(
                f,
                "{} is not empty, so no lane can be made there",
                dir.display()
            ),
            Error::UnfinishedLane(name, dir) => write!(
                f,
                "the lane {name} at {} was never finished: no op made it",
                dir.display()
            ),
            Error::StoreGone(store, why) => {
                write!(
                    f,
                    "the store this lane names, {}, does not open: {why}",
                    store.display()
                )
            }
            Error::BadWorkspaceFile(path, why) => {
                write!(f, "{} cannot be read: {why}", path.display())
            }
            Error::NotInWorkspace(op, name) => {
                write!(f, "op {op} holds nothing of workspace {name}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

/// Why an entry was reported and left alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reason {
    /// Its name has no UTF-8 form (INT-0004 AC-7).
    NotUtf8,
    /// Its name is UTF-8 but not a valid repository path component (it
    /// holds a `\` or a NUL).
    InvalidName,
    /// A symbolic link; symlinks are not specified yet.
    Symlink,
    /// A FIFO, socket or device.
    Special,
}

impl Reason {
    /// The reason as `praps status` prints it.
    pub fn text(self) -> &'static str {
        match self {
            Reason::NotUtf8 => "name is not UTF-8",
            Reason::InvalidName => "name is not a valid repository path",
            Reason::Symlink => "symbolic link",
            Reason::Special => "special file",
        }
    }
}

/// One entry a snapshot reported and left alone.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reported {
    /// Its path, `/`-separated, with its name in display form
    /// ([`sim::Name::display`]).
    pub path: String,
    /// Why it was not tracked.
    pub reason: Reason,
}

/// How a path differs between the working-copy change and its parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Change {
    /// Present in the change, absent from its parent.
    Added,
    /// Present in both, with other content or mode.
    Modified,
    /// Absent from the change, present in its parent.
    Deleted,
}

impl Change {
    /// The letter `praps status` prints.
    pub fn letter(self) -> char {
        match self {
            Change::Added => 'A',
            Change::Modified => 'M',
            Change::Deleted => 'D',
        }
    }
}

/// What `status` found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    /// The changed paths, sorted.
    pub changes: Vec<(RepoPath, Change)>,
    /// The entries reported and left alone, sorted.
    pub reported: Vec<Reported>,
}

/// What a snapshot did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// The op it committed, or `None` when the working copy was unchanged
    /// and nothing was written.
    pub op: Option<HashRef>,
    /// The entries reported and left alone, sorted.
    pub reported: Vec<Reported>,
}

/// What `describe` did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Described {
    /// The change ID of the change described.
    pub change_id: [u8; CHANGE_ID_LEN],
    /// The op that set the description, or `None` when the change already
    /// had exactly this description and nothing was written for it.
    pub op: Option<HashRef>,
}

/// What a command works from: the op log as one listing read it, and the
/// state its heads leave together. A command reads this once, at its start,
/// and passes it through its steps; a step that commits an op moves it to
/// what was just written, as a fresh read would find it. It never outlives
/// the command ("Reading the op log" in the spec).
struct State {
    /// Every valid op, by id.
    ops: BTreeMap<HashRef, Operation>,
    /// The state the heads leave together.
    at: Loaded,
}

/// The state the op heads leave together, as a workspace reads it.
struct Loaded {
    /// The op heads: one, or several that no command has joined yet.
    heads: BTreeSet<HashRef>,
    /// Their canonical merge ([`merge::merge`]): with one head, its view.
    view: View,
    /// Every changeset this workspace's pointer holds, when it holds
    /// several (commands that moved it at once); empty when it holds one.
    raced: Vec<HashRef>,
    /// The working-copy changeset's id and value: the pointer's one
    /// changeset, or the smallest of several (the race rule, INT-0014).
    wc_id: HashRef,
    wc: Changeset,
    /// The tree of the working-copy change's parent (empty for a root).
    parent_tree: Tree,
    /// The working-copy change's own tree.
    wc_tree: Tree,
}

/// What a scan of the working copy found.
#[derive(Default)]
struct Scanned {
    /// The tracked files: every regular file not ignored, and every file
    /// tracked before even if ignored now.
    tree: Tree,
    /// Each file's content, by blob id.
    contents: BTreeMap<HashRef, Vec<u8>>,
    /// The entries reported and left alone.
    reported: Vec<Reported>,
}

/// One scan in progress.
struct Scan<'a, F: Fs> {
    fs: &'a F,
    root: &'a Path,
    /// The working-copy change's tree: what was tracked before.
    tracked: &'a Tree,
    /// Its parent's tree, for the mode where files carry no executable bit.
    parent: &'a Tree,
    /// Blobs already in the store: their content is not kept.
    known: &'a BTreeSet<HashRef>,
    has_bit: bool,
    ignores: Ignores,
    out: Scanned,
}

impl<F: Fs> Scan<'_, F> {
    /// Scans directory `dir` (a repository path, `""` at the top). In an
    /// ignored directory (`tracked_only`) only paths tracked before count,
    /// and nothing is reported.
    fn dir(&mut self, dir: &str, tracked_only: bool) -> Result<(), Error> {
        let fs_dir = materialise::on_disk(self.root, dir);
        let entries = self.fs.scan(&fs_dir)?;
        let gitignore = match entries
            .iter()
            .find(|e| e.name.as_utf8() == Some(".gitignore"))
        {
            Some(entry) if !tracked_only && matches!(entry.kind, Kind::File { .. }) => {
                Some(self.fs.read(&fs_dir.join(".gitignore"))?)
            }
            _ => None,
        };
        self.ignores.push(dir, gitignore.as_deref());
        let join = |name: &str| {
            if dir.is_empty() {
                name.to_owned()
            } else {
                format!("{dir}/{name}")
            }
        };
        // The directory's components are UTF-8 repository path components;
        // each is shown escaped, as the entry's name is (code review CR-08).
        let shown_dir: Vec<String> = dir
            .split('/')
            .filter(|c| !c.is_empty())
            .map(|c| Name::Utf8(c.to_owned()).display())
            .collect();
        for entry in entries {
            let mut shown = shown_dir.clone();
            shown.push(entry.name.display());
            let shown = shown.join("/");
            let Name::Utf8(name) = &entry.name else {
                if !tracked_only {
                    self.report(shown, Reason::NotUtf8);
                }
                continue;
            };
            // Compared case-insensitively, so `.Git` is ignored on every
            // system, as it names `.git` on a case-insensitive one (CR-13).
            if dir.is_empty()
                && ALWAYS_IGNORED
                    .iter()
                    .any(|ignored| ignored.eq_ignore_ascii_case(name))
            {
                continue;
            }
            let full = join(name);
            let Ok(path) = RepoPath::new(full.clone()) else {
                if !tracked_only {
                    self.report(shown, Reason::InvalidName);
                }
                continue;
            };
            let ignored = tracked_only || self.ignores.is_ignored(&full, entry.kind == Kind::Dir);
            match entry.kind {
                Kind::Dir => {
                    // Paths below `dir` sort right after `dir/`, so one range
                    // probe answers whether any is tracked (CR-04).
                    let below = ignored
                        && self
                            .tracked
                            .range(path.clone()..)
                            .take_while(|(p, _)| p.starts_with(&path))
                            .any(|(p, _)| *p != path);
                    if !ignored || below {
                        self.dir(&full, ignored)?;
                    }
                }
                Kind::File { executable } => {
                    if ignored && !self.tracked.contains_key(&path) {
                        continue;
                    }
                    // A file deleted between the listing and the read is
                    // absent: the snapshot records it as deleted (CR-05).
                    let content = match self.fs.read(&fs_dir.join(name)) {
                        Ok(content) => content,
                        Err(error) if error.kind() == ErrorKind::NotFound => continue,
                        Err(error) => return Err(error.into()),
                    };
                    let id = object_id(ALG, &blob(&content));
                    let mode = if self.has_bit {
                        if executable {
                            Mode::Executable
                        } else {
                            Mode::Regular
                        }
                    } else {
                        // The mode the working-copy change, or else its
                        // parent, last recorded (CR-01).
                        self.tracked
                            .get(&path)
                            .or_else(|| self.parent.get(&path))
                            .map_or(Mode::Regular, |file| file.mode)
                    };
                    self.out.tree.insert(path, File { blob: id, mode });
                    if !self.known.contains(&id) {
                        self.out.contents.insert(id, content);
                    }
                }
                Kind::Symlink if !ignored => self.report(shown, Reason::Symlink),
                Kind::Other if !ignored => self.report(shown, Reason::Special),
                Kind::Symlink | Kind::Other => {}
            }
        }
        self.ignores.pop();
        Ok(())
    }

    fn report(&mut self, path: String, reason: Reason) {
        self.out.reported.push(Reported { path, reason });
    }
}

/// A repository's working copy.
pub struct Workspace<F: Fs> {
    store: Store<F>,
    root: PathBuf,
    /// The workspace this working copy is, whose view entry it writes.
    name: String,
    rng: Rng,
    /// What resuming an interrupted write left undone, for the user.
    notes: Vec<String>,
    /// What the latest snapshot reported and left alone (INT-0004 AC-7).
    reported: Vec<Reported>,
    /// The state `open` read, for the first command to take: a process that
    /// opens and runs one command then reads the op log once. `None` once a
    /// command has taken it, and on a workspace `init` returned.
    kept: Option<State>,
    #[cfg(feature = "variants")]
    variant: Variant,
}

/// Makes `root` and `root/.praps` exist and their entries durable: creates
/// whichever is missing, then syncs the root's parent, `.praps` and the root.
fn prepare(fs: &impl Fs, root: &Path) -> io::Result<PathBuf> {
    if !dir_exists(fs, root)? {
        fs.create_dir(root)?;
    }
    // Synced every time, not only when created here: a process that died
    // between creating the root and syncing its parent leaves a root that
    // exists but whose entry is not durable (code review CR-06).
    fs.sync_dir(&parent_dir(root))?;
    let praps = root.join(PRAPS_DIR);
    if !dir_exists(fs, &praps)? {
        fs.create_dir(&praps)?;
    }
    fs.sync_dir(&praps)?;
    fs.sync_dir(root)?;
    Ok(praps.join(STORE_DIR))
}

fn corrupt(what: impl fmt::Display) -> Error {
    Error::Corrupt(what.to_string())
}

/// Refuses an op log in which `ops/` holds files that do not hash to their
/// names or do not decode: the valid ops alone would let a damaged head
/// silently give way to its parent, or a store whose ops are all damaged
/// look uninitialised (code review CR-10).
fn refuse_invalid(log: &OpLog) -> Result<(), Error> {
    if log.invalid > 0 {
        return Err(corrupt(format!(
            "{} of {} op files do not hash to their names or do not decode",
            log.invalid,
            log.ops.len() + log.invalid
        )));
    }
    Ok(())
}

impl<F: Fs> Workspace<F> {
    /// Makes `root` a repository, or completes one whose `init` was
    /// interrupted: creates and syncs `root` and `.praps`, initialises the
    /// store, then [`Workspace::init_with_store`]. Returns the workspace and
    /// whether it committed the root op (false: the repository existed).
    pub fn init(fs: F, root: &Path, seed: u64) -> Result<(Self, bool), Error> {
        let store_root = prepare(&fs, root)?;
        let store = Store::init(fs, &store_root, ALG, seed)?;
        Self::init_with_store(store, root, seed)
    }

    /// Commits the root op on `store` if it has no op: an empty changeset
    /// with no parents and a change ID drawn from `seed`, a view whose
    /// `default` entry has that changeset as its working copy and its one
    /// head, and an op made in `default` naming the view.
    /// On a store that has an op it writes nothing. Tests pass a store built
    /// with a negative-control variant here.
    pub fn init_with_store(store: Store<F>, root: &Path, seed: u64) -> Result<(Self, bool), Error> {
        let mut workspace = Self::new(store, root, seed);
        let log = workspace.store.op_log()?;
        refuse_invalid(&log)?;
        if !log.ops.is_empty() {
            return Ok((workspace, false));
        }
        let change = Changeset {
            parents: Vec::new(),
            change_id: workspace.change_id(),
            changes: BTreeMap::new(),
            description: Vec::new(),
        };
        let change_id = workspace.store.put_object(&change.to_envelope())?;
        let view = View {
            workspaces: [(
                DEFAULT_WORKSPACE.to_owned(),
                WorkspaceEntry {
                    working_copy: [change_id].into(),
                    heads: [change_id].into(),
                },
            )]
            .into(),
        };
        let view_id = workspace.store.put_object(&view.to_envelope())?;
        workspace.store.commit_op(&Operation {
            parents: BTreeSet::new(),
            workspace: DEFAULT_WORKSPACE.to_owned(),
            view: view_id,
        })?;
        Ok((workspace, true))
    }

    /// Opens the repository at `root`: makes `.praps` and the root durable
    /// as `init` does, opens the store, checks that every op file is whole,
    /// finishes an interrupted write, and checks that the op log has exactly
    /// one head. It reads the op log once, and keeps what it read for the
    /// first command. `seed` drives the change IDs it mints.
    pub fn open(fs: F, root: &Path, seed: u64) -> Result<Self, Error> {
        let praps = root.join(PRAPS_DIR);
        let mut workspace = if dir_exists(&fs, &praps.join(STORE_DIR))? {
            let store_root = prepare(&fs, root)?;
            let store = Store::open(fs, &store_root, ALG, seed).map_err(|error| {
                if error.kind() == ErrorKind::NotFound {
                    Error::NotInitialised(root.to_path_buf())
                } else {
                    Error::Io(error)
                }
            })?;
            Self::new(store, root, seed)
        } else {
            // A lane: `.praps/workspace` names its workspace and the store
            // it shares. Probed only where there is no store, so opening a
            // repository's own working copy makes no new syscall.
            let file = match lane::read(&fs, &praps)? {
                None => return Err(Error::NotInitialised(root.to_path_buf())),
                Some(Err(why)) => {
                    return Err(Error::BadWorkspaceFile(
                        praps.join(lane::WORKSPACE_FILE),
                        why,
                    ));
                }
                Some(Ok(file)) => file,
            };
            let store = Store::open(fs, &file.store, ALG, seed)
                .map_err(|error| Error::StoreGone(file.store.clone(), error.to_string()))?;
            let mut workspace = Self::new(store, root, seed);
            workspace.name = file.name;
            workspace
        };
        let log = workspace.store.op_log()?;
        // Before resume, which looks for the record's target among the
        // valid ops: a target whose file is damaged would look like a write
        // that never began, and its record would be dropped though files
        // may already have been written.
        refuse_invalid(&log)?;
        workspace.resume(&log.ops)?;
        workspace.kept = Some(workspace.state_of(log)?);
        Ok(workspace)
    }

    fn new(store: Store<F>, root: &Path, seed: u64) -> Self {
        Workspace {
            store,
            root: root.to_path_buf(),
            name: DEFAULT_WORKSPACE.to_owned(),
            rng: Rng::new(seed),
            notes: Vec::new(),
            reported: Vec::new(),
            kept: None,
            #[cfg(feature = "variants")]
            variant: Variant::Correct,
        }
    }

    /// Selects a broken variant, for the crash scenario's negative control.
    #[cfg(feature = "variants")]
    pub fn set_variant(&mut self, variant: Variant) {
        self.variant = variant;
    }

    #[cfg(feature = "variants")]
    fn writes_record(&self) -> bool {
        self.variant != Variant::NoRecord
    }

    #[cfg(not(feature = "variants"))]
    fn writes_record(&self) -> bool {
        true
    }

    #[cfg(feature = "variants")]
    fn writes_lane_file_in_place(&self) -> bool {
        self.variant == Variant::LaneFileInPlace
    }

    #[cfg(not(feature = "variants"))]
    fn writes_lane_file_in_place(&self) -> bool {
        false
    }

    #[cfg(feature = "variants")]
    fn keeps_description(&self) -> bool {
        self.variant != Variant::DropDescription
    }

    #[cfg(not(feature = "variants"))]
    fn keeps_description(&self) -> bool {
        true
    }

    /// What resuming an interrupted write left undone, one line each.
    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// The entries the latest snapshot reported and left alone, sorted, as
    /// [`Status::reported`] gives them (INT-0004 AC-7). Every command that
    /// snapshots (`ops`, `changes`, `undo`, `restore`, `new_change`,
    /// `describe`, `status`) sets it, and it stays set when the command then
    /// fails. Each command snapshots once, so it holds one report.
    pub fn reported(&self) -> &[Reported] {
        &self.reported
    }

    fn praps_dir(&self) -> PathBuf {
        self.root.join(PRAPS_DIR)
    }

    /// The view an op names.
    fn view_of(&self, op: &Operation) -> Result<View, Error> {
        View::from_envelope(&self.store.get_object(&op.view)?)
            .map_err(|error| corrupt(format!("view {}: {error}", op.view)))
    }

    /// The working-copy tree this workspace's entry names in a view that an
    /// operation of this workspace left: its pointer holds one changeset.
    fn wc_tree_of(&self, view: &View) -> Result<Tree, Error> {
        self.tree_of(&working_copy_of(view, &self.name)?)
    }

    /// The view `heads` leave together ([`merge::merge`]), reading each view
    /// it needs from the store.
    fn merged(&self, ops: &merge::Ops, heads: &BTreeSet<HashRef>) -> Result<View, Error> {
        merge::merge(ops, heads, |id| {
            View::from_envelope(&self.store.get_object(id)?)
                .map_err(|error| corrupt(format!("view {id}: {error}")))
        })
    }

    /// A blob's content.
    fn content(&self, id: &HashRef) -> io::Result<Vec<u8>> {
        let bytes = self.store.get_object(id)?;
        decode_envelope(&bytes)
            .map(|envelope| envelope.body.to_vec())
            .map_err(|error| io::Error::new(ErrorKind::InvalidData, format!("blob {id}: {error}")))
    }

    /// Finishes or drops an interrupted write of the working copy
    /// ("Resume" in the spec), given the valid ops of the log.
    fn resume(&mut self, ops: &BTreeMap<HashRef, Operation>) -> Result<(), Error> {
        let praps = self.praps_dir();
        let record = match record::read(self.store.fs(), &praps)? {
            None => return Ok(()),
            Some(Ok(record)) => record,
            Some(Err(why)) => {
                return Err(corrupt(format!("{PRAPS_DIR}/{}: {why}", record::RECORD)));
            }
        };
        self.store.fs().point("workspace.resume.started");
        let Some(target_op) = ops.get(&record.target) else {
            // The target op never landed, so no file was touched.
            record::drop_record(self.store.fs(), &praps)?;
            self.store.fs().point("workspace.resume.dropped");
            return Ok(());
        };
        let from = match &record.from {
            Some(changeset) => self.tree_of(changeset).map_err(|error| match error {
                Error::Io(error) if error.kind() == ErrorKind::NotFound => corrupt(format!(
                    "{PRAPS_DIR}/{} names {changeset}, which is no changeset in the store",
                    record::RECORD
                )),
                error => error,
            })?,
            None => Tree::new(),
        };
        let target = self.wc_tree_of(&self.view_of(target_op)?)?;
        let fs = self.store.fs();
        materialise::remove_temporaries(fs, &self.root, &from, &target)?;
        // The write is finished while its op is still the only latest op of
        // its workspace: an op of another workspace since changes nothing
        // here (INT-0014, "Resume").
        let writers = merge::writers(ops, &store::heads_of(ops));
        if writers.get(&target_op.workspace) == Some(&BTreeSet::from([record.target])) {
            let plan =
                materialise::plan(fs, &self.root, &from, &target, fs.name_rules(), ALG, true)?;
            let kept =
                materialise::apply(fs, &self.root, &from, ALG, &plan, |id| self.content(id))?;
            for skipped in plan.skipped.iter().chain(&kept) {
                self.notes.push(format!(
                    "an interrupted write of the working copy left this as it was: {skipped}"
                ));
            }
        } else {
            self.notes.push(format!(
                "an interrupted write of the working copy (to op {}) was abandoned: \
                 another op of this workspace followed it or was made beside it; \
                 the working copy was left as it was",
                record.target
            ));
        }
        record::drop_record(self.store.fs(), &praps)?;
        Ok(())
    }

    /// Snapshots, then moves the working copy to the view `view_id` names:
    /// checks hazards and blocking entries, writes the record, commits an op
    /// over every head whose view is exactly that view, materialises, and
    /// drops the record. Returns the new op. `undo` and `restore` choose the
    /// view ("History commands" in the spec).
    pub fn restore_view(&mut self, view_id: &HashRef) -> Result<HashRef, Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)?;
        let view = View::from_envelope(&self.store.get_object(view_id)?)
            .map_err(|error| corrupt(format!("view {view_id}: {error}")))?;
        self.restore_view_in(state, view_id, &view)
    }

    /// Moves the working copy to the view `view_id` names, from the state a
    /// command already holds, which that command has just snapshotted: the
    /// working copy is scanned once for a command, and the write then
    /// reads each path again before it removes or replaces it. It takes
    /// the state for good: the op it commits joins the heads, and nothing
    /// follows it in a command.
    /// `view` is the view `view_id` names, which the caller has read.
    fn restore_view_in(
        &mut self,
        state: State,
        view_id: &HashRef,
        view: &View,
    ) -> Result<HashRef, Error> {
        let loaded = &state.at;
        let target = self.wc_tree_of(view)?;
        let fs = self.store.fs();
        let plan = materialise::plan(
            fs,
            &self.root,
            &loaded.wc_tree,
            &target,
            fs.name_rules(),
            ALG,
            false,
        )?;
        if !plan.hazards.is_empty() {
            return Err(Error::Hazards(plan.hazards));
        }
        if !plan.blocked.is_empty() {
            return Err(Error::Blocked(plan.blocked));
        }
        let op = Operation {
            parents: loaded.heads.clone(),
            workspace: self.name.clone(),
            view: *view_id,
        };
        let id = object_id(ALG, &op.to_envelope());
        let praps = self.praps_dir();
        let journalled = self.writes_record();
        if journalled {
            record::write(
                self.store.fs(),
                &praps,
                &Record {
                    from: Some(loaded.wc_id),
                    target: id,
                },
            )?;
        }
        let committed = self.store.commit_op(&op)?;
        if committed != id {
            return Err(corrupt(format!(
                "op {committed} was committed where {id} was expected"
            )));
        }
        let fs = self.store.fs();
        let kept = materialise::apply(fs, &self.root, &loaded.wc_tree, ALG, &plan, |blob| {
            self.content(blob)
        })?;
        self.notes.extend(kept);
        if journalled {
            record::drop_record(fs, &praps)?;
        }
        Ok(committed)
    }

    /// The store.
    pub fn store(&self) -> &Store<F> {
        &self.store
    }

    /// The working copy's directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn change_id(&mut self) -> [u8; CHANGE_ID_LEN] {
        let mut id = [0; CHANGE_ID_LEN];
        id[..8].copy_from_slice(&self.rng.next_u64().to_le_bytes());
        id[8..].copy_from_slice(&self.rng.next_u64().to_le_bytes());
        id
    }

    fn changeset(&self, id: &HashRef) -> Result<Changeset, Error> {
        let bytes = self.store.get_object(id)?;
        Changeset::from_envelope(&bytes)
            .map_err(|error| corrupt(format!("changeset {id}: {error}")))
    }

    /// A changeset's tree, from its chain of first parents.
    fn tree_of(&self, id: &HashRef) -> Result<Tree, Error> {
        let mut chain = Vec::new();
        let mut next = Some(*id);
        while let Some(current) = next {
            let changeset = self.changeset(&current)?;
            next = match changeset.parents.as_slice() {
                [] => None,
                [parent] => Some(*parent),
                _ => return Err(Error::Merge(current)),
            };
            chain.push(changeset);
        }
        let mut tree = Tree::new();
        for changeset in chain.iter().rev() {
            tree = tree::of(changeset, &tree);
        }
        Ok(tree)
    }

    /// The state the heads of `log` leave together, or why there is none:
    /// an invalid op file, or no op at all. Several heads are merged in
    /// memory and nothing is written ([`merge::merge`]).
    fn state_of(&self, log: OpLog) -> Result<State, Error> {
        refuse_invalid(&log)?;
        let ops = log.ops;
        let heads = store::heads_of(&ops);
        if heads.is_empty() {
            return Err(Error::NotInitialised(self.root.clone()));
        }
        let view = self.merged(&ops, &heads)?;
        let pointer = &view
            .workspaces
            .get(&self.name)
            .ok_or_else(|| {
                if self.name == DEFAULT_WORKSPACE {
                    corrupt(format!("the view has no workspace {}", self.name))
                } else {
                    Error::UnfinishedLane(self.name.clone(), self.root.clone())
                }
            })?
            .working_copy;
        // The race rule: the smallest of several, the others kept.
        let wc_id = *pointer
            .first()
            .ok_or_else(|| corrupt("a working copy names no changeset"))?;
        let raced = if pointer.len() > 1 {
            pointer.iter().copied().collect()
        } else {
            Vec::new()
        };
        let wc = self.changeset(&wc_id)?;
        let parent_tree = match wc.parents.as_slice() {
            [] => Tree::new(),
            [parent] => self.tree_of(parent)?,
            _ => return Err(Error::Merge(wc_id)),
        };
        let wc_tree = tree::of(&wc, &parent_tree);
        Ok(State {
            ops,
            at: Loaded {
                heads,
                view,
                raced,
                wc_id,
                wc,
                parent_tree,
                wc_tree,
            },
        })
    }

    /// Reads the op log, with one listing of `ops/`, and the state its heads
    /// leave together.
    fn read_state(&self) -> Result<State, Error> {
        self.state_of(self.store.op_log()?)
    }

    /// The state a command starts from: what `open` read, if no command has
    /// taken it yet, and otherwise a fresh read. A command that fails here
    /// reports nothing, rather than leaving an earlier command's entries
    /// (code review CR-07).
    fn begin(&mut self) -> Result<State, Error> {
        self.reported.clear();
        let state = match self.kept.take() {
            Some(state) => state,
            None => self.read_state()?,
        };
        if !state.at.raced.is_empty() {
            let raced: Vec<String> = state.at.raced.iter().map(ToString::to_string).collect();
            self.notes.push(format!(
                "commands moved the working copy at the same time, to {}; \
                 this command continues from {}, and the others stay among the changes",
                raced.join(", "),
                state.at.wc_id
            ));
        }
        Ok(state)
    }

    /// Scans the working copy, putting no blob yet.
    fn scan(
        &self,
        tracked: &Tree,
        parent: &Tree,
        known: &BTreeSet<HashRef>,
    ) -> Result<Scanned, Error> {
        let fs = self.store.fs();
        let mut scan = Scan {
            fs,
            root: &self.root,
            tracked,
            parent,
            known,
            has_bit: fs.has_executable_bit(),
            ignores: Ignores::default(),
            out: Scanned::default(),
        };
        scan.dir("", false)?;
        scan.out.reported.sort();
        Ok(scan.out)
    }

    /// Snapshots the working copy into the working-copy change. If its
    /// tracked content differs from the change's tree, puts the new blobs, a
    /// changeset with the same change ID, parents and description and the
    /// path map against its parent's tree, a view in which it replaces the
    /// old changeset (as head and as the `default` workspace), and an op
    /// whose parent is the single head. Otherwise it writes nothing.
    pub fn snapshot(&mut self) -> Result<Snapshot, Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)
    }

    /// [`Workspace::snapshot`] from the state a command already holds.
    fn snapshot_in(&mut self, state: &mut State) -> Result<Snapshot, Error> {
        // A snapshot that fails before its scan reports nothing, rather than
        // leaving an earlier command's entries (code review CR-07).
        self.reported.clear();
        let loaded = &state.at;
        let known: BTreeSet<HashRef> = loaded
            .wc_tree
            .values()
            .chain(loaded.parent_tree.values())
            .map(|file| file.blob)
            .collect();
        let Scanned {
            tree,
            contents,
            reported,
        } = self.scan(&loaded.wc_tree, &loaded.parent_tree, &known)?;
        self.store.fs().point("workspace.snapshot.scanned");
        self.reported.clone_from(&reported);
        if tree == loaded.wc_tree {
            return Ok(Snapshot { op: None, reported });
        }
        // `contents` holds only blobs the store does not have yet, each in
        // the new tree (CR-04).
        for content in contents.values() {
            self.store.put_object(&blob(content))?;
        }
        let changeset = Changeset {
            parents: loaded.wc.parents.clone(),
            change_id: loaded.wc.change_id,
            changes: tree::diff(&loaded.parent_tree, &tree),
            description: if self.keeps_description() {
                loaded.wc.description.clone()
            } else {
                Vec::new()
            },
        };
        let op = self.replace_working_copy(state, changeset)?;
        self.store.fs().point("workspace.snapshot.committed");
        Ok(Snapshot {
            op: Some(op),
            reported,
        })
    }

    /// Puts `changeset`, a view in which it replaces the working-copy
    /// changeset as a head and as the working copy of this workspace's
    /// entry, and an op made in this workspace over every head. Then moves `state` to what it wrote: the op
    /// joins the log as its head, and the trees are those a fresh read
    /// would build, by [`tree::of`] from the same changesets.
    fn replace_working_copy(
        &mut self,
        state: &mut State,
        changeset: Changeset,
    ) -> Result<HashRef, Error> {
        let id = self.store.put_object(&changeset.to_envelope())?;
        let mut view = state.at.view.clone();
        let entry = view.workspaces.entry(self.name.clone()).or_default();
        entry.heads.remove(&state.at.wc_id);
        entry.heads.insert(id);
        entry.working_copy = [id].into();
        let view_id = self.store.put_object(&view.to_envelope())?;
        let op = Operation {
            parents: std::mem::take(&mut state.at.heads),
            workspace: self.name.clone(),
            view: view_id,
        };
        let head = self.store.commit_op(&op)?;
        let parent_tree = match changeset.parents.as_slice() {
            [] => Tree::new(),
            // A new change on top of the old working-copy change.
            [parent] if *parent == state.at.wc_id => std::mem::take(&mut state.at.wc_tree),
            // The same change, replaced: its parent has not moved.
            parents if parents == state.at.wc.parents.as_slice() => {
                std::mem::take(&mut state.at.parent_tree)
            }
            [parent] => self.tree_of(parent)?,
            _ => return Err(Error::Merge(id)),
        };
        let wc_tree = tree::of(&changeset, &parent_tree);
        state.ops.insert(head, op);
        state.at = Loaded {
            heads: [head].into(),
            view,
            raced: Vec::new(),
            wc_id: id,
            wc: changeset,
            parent_tree,
            wc_tree,
        };
        Ok(head)
    }

    /// Snapshots, then starts a new working-copy change on top of the
    /// current one: a changeset whose only parent is the current change,
    /// with a fresh change ID and no changes. It replaces its parent as a
    /// head and becomes the `default` workspace's change. No working-copy
    /// file is written. Returns the new change ID.
    pub fn new_change(&mut self) -> Result<[u8; CHANGE_ID_LEN], Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)?;
        let change_id = self.change_id();
        let changeset = Changeset {
            parents: vec![state.at.wc_id],
            change_id,
            changes: BTreeMap::new(),
            description: Vec::new(),
        };
        self.replace_working_copy(&mut state, changeset)?;
        Ok(change_id)
    }

    /// Snapshots, then makes lane `name` at `dir`, a working copy over this
    /// store (INT-0014; "Lanes" in the spec), in this order:
    /// 1. refuses unless `name` is no workspace yet and `dir` is free;
    /// 2. forms the lane's first change: no changes, on the parent of this
    ///    workspace's working-copy change (on nothing when it has none);
    /// 3. makes `dir` and `dir/.praps` durable and writes
    ///    `dir/.praps/workspace`;
    /// 4. writes the record, `from empty`, in `dir/.praps`;
    /// 5. commits one op made in `name` over every head, whose view adds
    ///    `name`'s entry;
    /// 6. writes the change's files into `dir`, and drops the record.
    ///
    /// Returns the op and the lane's change ID.
    pub fn make_lane(
        &mut self,
        name: &str,
        dir: &Path,
    ) -> Result<(HashRef, [u8; CHANGE_ID_LEN]), Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)?;
        if state.at.view.workspaces.contains_key(name) {
            return Err(Error::LaneExists(name.to_owned()));
        }
        self.refuse_unless_free(dir, &state.at.view)?;
        let parents = match state.at.wc.parents.as_slice() {
            [] => Vec::new(),
            [parent] => vec![*parent],
            _ => return Err(Error::Merge(state.at.wc_id)),
        };
        let tree = match parents.as_slice() {
            [parent] => self.tree_of(parent)?,
            _ => Tree::new(),
        };
        let change_id = self.change_id();
        let changeset = Changeset {
            parents,
            change_id,
            changes: BTreeMap::new(),
            description: Vec::new(),
        };
        let change = self.store.put_object(&changeset.to_envelope())?;
        let mut view = state.at.view.clone();
        view.workspaces.insert(
            name.to_owned(),
            WorkspaceEntry {
                working_copy: [change].into(),
                heads: [change].into(),
            },
        );
        let view_id = self.store.put_object(&view.to_envelope())?;
        let op = Operation {
            parents: state.at.heads.clone(),
            workspace: name.to_owned(),
            view: view_id,
        };
        let id = object_id(ALG, &op.to_envelope());
        let fs = self.store.fs();
        prepare(fs, dir)?;
        let praps = dir.join(PRAPS_DIR);
        let plan = materialise::plan(fs, dir, &Tree::new(), &tree, fs.name_rules(), ALG, false)?;
        if !plan.hazards.is_empty() {
            return Err(Error::Hazards(plan.hazards));
        }
        lane::write(
            fs,
            &praps,
            &lane::LaneFile {
                name: name.to_owned(),
                store: self.store.root().to_path_buf(),
            },
            self.writes_lane_file_in_place(),
        )?;
        record::write(
            fs,
            &praps,
            &Record {
                from: None,
                target: id,
            },
        )?;
        let committed = self.store.commit_op(&op)?;
        if committed != id {
            return Err(corrupt(format!(
                "op {committed} was committed where {id} was expected"
            )));
        }
        let fs = self.store.fs();
        let kept =
            materialise::apply(fs, dir, &Tree::new(), ALG, &plan, |blob| self.content(blob))?;
        self.notes.extend(kept);
        record::drop_record(fs, &praps)?;
        Ok((committed, change_id))
    }

    /// Refuses `dir` unless it is free for a lane: absent, empty, or holding
    /// nothing but the `.praps` an interrupted making left. Such a `.praps`
    /// holds no `workspace` file, or one naming a workspace `view` has no
    /// entry for, and at most the temporary names and the record beside it.
    fn refuse_unless_free(&self, dir: &Path, view: &View) -> Result<(), Error> {
        let fs = self.store.fs();
        if !dir_exists(fs, dir)? {
            return Ok(());
        }
        let not_free = || Error::NotFree(dir.to_path_buf());
        let entries = fs.list(dir)?;
        if entries.is_empty() {
            return Ok(());
        }
        if entries.len() != 1 || entries[0] != PRAPS_DIR {
            return Err(not_free());
        }
        let praps = dir.join(PRAPS_DIR);
        let left = [
            lane::WORKSPACE_FILE,
            lane::WORKSPACE_FILE_NEW,
            record::RECORD,
            record::RECORD_NEW,
        ];
        if fs
            .list(&praps)?
            .iter()
            .any(|entry| !left.contains(&entry.as_str()))
        {
            return Err(not_free());
        }
        match lane::read(fs, &praps)? {
            None => Ok(()),
            Some(Ok(file)) if !view.workspaces.contains_key(&file.name) => Ok(()),
            Some(_) => Err(not_free()),
        }
    }

    /// Snapshots, then sets the working-copy change's description to
    /// `text`, byte for byte: one op in which a changeset with the same
    /// change ID, parents and changes, and `text` as its description,
    /// replaces the current one. No working-copy file is written. When
    /// `text` is the description the change already has, nothing is written
    /// for it.
    pub fn describe(&mut self, text: &[u8]) -> Result<Described, Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)?;
        let change_id = state.at.wc.change_id;
        if state.at.wc.description == text {
            return Ok(Described {
                change_id,
                op: None,
            });
        }
        let changeset = Changeset {
            description: text.to_vec(),
            ..state.at.wc.clone()
        };
        let op = self.replace_working_copy(&mut state, changeset)?;
        self.store.fs().point("workspace.describe.committed");
        Ok(Described {
            change_id,
            op: Some(op),
        })
    }

    /// Snapshots, then compares the working-copy change with its parent.
    pub fn status(&mut self) -> Result<Status, Error> {
        let mut state = self.begin()?;
        let snapshot = self.snapshot_in(&mut state)?;
        let loaded = &state.at;
        let mut changes = Vec::new();
        for (path, file) in &loaded.wc_tree {
            match loaded.parent_tree.get(path) {
                None => changes.push((path.clone(), Change::Added)),
                Some(parent) if parent != file => changes.push((path.clone(), Change::Modified)),
                Some(_) => {}
            }
        }
        for path in loaded.parent_tree.keys() {
            if !loaded.wc_tree.contains_key(path) {
                changes.push((path.clone(), Change::Deleted));
            }
        }
        changes.sort();
        Ok(Status {
            changes,
            reported: snapshot.reported,
        })
    }

    /// The working-copy change's id, change ID and tree, as the single op
    /// head leaves it. Read from the store each time, never from the state
    /// a command holds, so a test that compares it with a model checks what
    /// is stored.
    pub fn working_copy(&self) -> Result<(HashRef, [u8; CHANGE_ID_LEN], Tree), Error> {
        let loaded = self.read_state()?.at;
        Ok((loaded.wc_id, loaded.wc.change_id, loaded.wc_tree))
    }

    /// The working-copy change's parent tree, read from the store each time.
    pub fn parent_tree(&self) -> Result<Tree, Error> {
        Ok(self.read_state()?.at.parent_tree)
    }

    /// Snapshots, then lists every op, newest first: children before
    /// parents, ties broken by id ("History commands" in the spec). Each
    /// entry costs two object reads, its view and its working-copy
    /// changeset, however long the history is.
    pub fn ops(&mut self) -> Result<Vec<OpEntry>, Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)?;
        let ops = &state.ops;
        let mut children: BTreeMap<&HashRef, usize> = ops.keys().map(|id| (id, 0)).collect();
        for op in ops.values() {
            for parent in &op.parents {
                if let Some(count) = children.get_mut(parent) {
                    *count += 1;
                }
            }
        }
        let mut ready: BTreeSet<&HashRef> = children
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut entries = Vec::with_capacity(ops.len());
        while let Some(id) = ready.pop_first() {
            let op = &ops[id];
            for parent in &op.parents {
                if let Some(count) = children.get_mut(parent) {
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(parent);
                    }
                }
            }
            let view = self.view_of(op)?;
            let working_copy = *view
                .workspaces
                .get(&op.workspace)
                .and_then(|entry| entry.working_copy.first())
                .ok_or_else(|| {
                    corrupt(format!("view {}: no entry for {}", op.view, op.workspace))
                })?;
            let changeset = self.changeset(&working_copy)?;
            entries.push(OpEntry {
                id: *id,
                parents: op.parents.clone(),
                workspace: op.workspace.clone(),
                working_copy,
                change_id: changeset.change_id,
                paths: changeset.changes.len(),
            });
        }
        Ok(entries)
    }

    /// Snapshots, then lists every workspace the heads hold, in name order:
    /// its working-copy change (the smallest, when its pointer holds
    /// several), what that change changes, and the other changes of a
    /// pointer that holds several. With no edit it writes nothing.
    pub fn lanes(&mut self) -> Result<Vec<LaneEntry>, Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)?;
        let mut lanes = Vec::new();
        for (name, entry) in &state.at.view.workspaces {
            let mut changes = Vec::new();
            for id in &entry.working_copy {
                changes.push(self.changeset(id)?);
            }
            let first = changes.remove(0);
            lanes.push(LaneEntry {
                name: name.clone(),
                this: *name == self.name,
                change_id: first.change_id,
                paths: first.changes.len(),
                others: changes.iter().map(|change| change.change_id).collect(),
            });
        }
        Ok(lanes)
    }

    /// The workspace this working copy is.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Snapshots, then lists the working-copy change and the changes before
    /// it along first parents, newest first ("History commands" in the
    /// spec). Each entry costs one object read.
    pub fn changes(&mut self) -> Result<Vec<ChangeEntry>, Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)?;
        let entry = |id: HashRef, changeset: &Changeset| ChangeEntry {
            id,
            change_id: changeset.change_id,
            paths: changeset.changes.len(),
            description: changeset.description.clone(),
        };
        let mut entries = vec![entry(state.at.wc_id, &state.at.wc)];
        let mut next = only_parent(&state.at.wc, &state.at.wc_id)?;
        while let Some(id) = next {
            let changeset = self.changeset(&id)?;
            next = only_parent(&changeset, &id)?;
            entries.push(entry(id, &changeset));
        }
        Ok(entries)
    }

    /// Undoes this workspace's last op (INT-0009, "Undo is scoped"): takes
    /// its writers as they are before the snapshot (with one head, that
    /// head), snapshots once, and returns this workspace's entry to its value
    /// just before them, in the view their parents leave together. No other
    /// entry changes. Returns those parents and the new op. With no parent,
    /// or none that holds this workspace (its writer made it), it fails with
    /// [`Error::NothingToUndo`], having written only the snapshot.
    pub fn undo(&mut self) -> Result<(BTreeSet<HashRef>, HashRef), Error> {
        let mut state = self.begin()?;
        let writers = merge::writers(&state.ops, &state.at.heads)
            .remove(&self.name)
            .unwrap_or_default();
        self.snapshot_in(&mut state)?;
        let mut parents = BTreeSet::new();
        for writer in &writers {
            let op = state
                .ops
                .get(writer)
                .ok_or_else(|| corrupt(format!("op {writer} disappeared")))?;
            parents.extend(op.parents.iter().copied());
        }
        if parents.is_empty() {
            return Err(Error::NothingToUndo);
        }
        let before = self.merged(&state.ops, &parents)?;
        let Some(entry) = before.workspaces.get(&self.name).cloned() else {
            return Err(Error::NothingToUndo);
        };
        let own_view = match parents.iter().collect::<Vec<_>>()[..] {
            [one] => Some(state.ops[one].view),
            _ => None,
        };
        let (view_id, view) = self.scoped_view(&state, entry, own_view)?;
        let new = self.restore_view_in(state, &view_id, &view)?;
        Ok((parents, new))
    }

    /// Snapshots, then returns this workspace's entry to its value in the
    /// view of the op `spec` names: an op id in text form, or a unique
    /// prefix of its hex digest, with or without the algorithm name. No
    /// other entry changes. Returns that op and the new op.
    pub fn restore(&mut self, spec: &str) -> Result<(HashRef, HashRef), Error> {
        let mut state = self.begin()?;
        self.snapshot_in(&mut state)?;
        let target = resolve(&state.ops, spec)?;
        let own_view = state.ops[&target].view;
        let view = self.view_of(&state.ops[&target])?;
        let Some(entry) = view.workspaces.get(&self.name).cloned() else {
            return Err(Error::NotInWorkspace(target, self.name.clone()));
        };
        let (view_id, view) = self.scoped_view(&state, entry, Some(own_view))?;
        let new = self.restore_view_in(state, &view_id, &view)?;
        Ok((target, new))
    }

    /// The view `undo` or `restore` moves to: the state's view with only
    /// this workspace's entry set to `entry`. A pointer of several
    /// changesets keeps the smallest, the others staying among the heads,
    /// with the race note (INT-0014). When that view is exactly the view
    /// `own_view` names, that object is named; otherwise it is written.
    /// Returns the view's id and the view.
    fn scoped_view(
        &mut self,
        state: &State,
        mut entry: WorkspaceEntry,
        own_view: Option<HashRef>,
    ) -> Result<(HashRef, View), Error> {
        if entry.working_copy.len() > 1 {
            let raced: Vec<String> = entry.working_copy.iter().map(ToString::to_string).collect();
            let kept = *entry.working_copy.first().expect("several");
            self.notes.push(format!(
                "the state this command returns to had the working copy at {}; \
                 this command continues from {kept}, and the others stay among the changes",
                raced.join(", ")
            ));
            entry.heads.extend(entry.working_copy.iter().copied());
            entry.working_copy = [kept].into();
        }
        let mut view = state.at.view.clone();
        view.workspaces.insert(self.name.clone(), entry);
        let bytes = view.to_envelope();
        if let Some(id) = own_view.filter(|id| *id == object_id(ALG, &bytes)) {
            return Ok((id, view));
        }
        Ok((self.store.put_object(&bytes)?, view))
    }
}

/// The op `spec` names among `ops`.
fn resolve(ops: &BTreeMap<HashRef, Operation>, spec: &str) -> Result<HashRef, Error> {
    let unknown = || Error::UnknownOp(spec.to_owned());
    let hex = match spec.split_once(':') {
        Some((alg, hex)) if alg == ALG.name() => hex,
        Some(_) => return Err(unknown()),
        None => spec,
    };
    if hex.is_empty() || !hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(unknown());
    }
    let matches: Vec<HashRef> = ops
        .keys()
        .filter(|id| hash::hex(id.digest()).starts_with(hex))
        .copied()
        .collect();
    match matches.as_slice() {
        [] => Err(unknown()),
        [one] => Ok(*one),
        _ => Err(Error::AmbiguousOp(spec.to_owned(), matches)),
    }
}

/// Workspace `name`'s working-copy changeset in `view`, which an operation
/// of that workspace left: every operation a command commits holds one
/// changeset in its own pointer, so one that holds several here is corrupt.
fn working_copy_of(view: &View, name: &str) -> Result<HashRef, Error> {
    let entry = view
        .workspaces
        .get(name)
        .ok_or_else(|| corrupt(format!("the view has no workspace {name}")))?;
    match entry.working_copy.iter().collect::<Vec<_>>()[..] {
        [one] => Ok(*one),
        _ => Err(corrupt(format!(
            "the working copy of {name} names {} changesets",
            entry.working_copy.len()
        ))),
    }
}

/// A changeset's one parent, if it has any. A changeset with several needs
/// INT-0012's merge.
fn only_parent(changeset: &Changeset, id: &HashRef) -> Result<Option<HashRef>, Error> {
    match changeset.parents.as_slice() {
        [] => Ok(None),
        [parent] => Ok(Some(*parent)),
        _ => Err(Error::Merge(*id)),
    }
}

/// One op, as `ops` lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpEntry {
    /// The op's id.
    pub id: HashRef,
    /// Its parent ops.
    pub parents: BTreeSet<HashRef>,
    /// The workspace that made it.
    pub workspace: String,
    /// That workspace's working-copy changeset in the op's view: the
    /// smallest, when the pointer holds several.
    pub working_copy: HashRef,
    /// That changeset's change ID.
    pub change_id: [u8; CHANGE_ID_LEN],
    /// How many paths that changeset changes against its parent. Counting
    /// the files of its whole tree would need the chain of changes behind
    /// it, for every op listed.
    pub paths: usize,
}

/// One workspace, as `lanes` lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneEntry {
    /// The workspace's name.
    pub name: String,
    /// Whether it is the workspace the command ran in.
    pub this: bool,
    /// Its working-copy change's change ID.
    pub change_id: [u8; CHANGE_ID_LEN],
    /// How many paths that change changes against its parent.
    pub paths: usize,
    /// The change IDs of the pointer's other changes, when commands moved it
    /// at the same time.
    pub others: Vec<[u8; CHANGE_ID_LEN]>,
}

/// One change, as `log` lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeEntry {
    /// The changeset's id.
    pub id: HashRef,
    /// Its change ID.
    pub change_id: [u8; CHANGE_ID_LEN],
    /// How many paths it changes against its parent.
    pub paths: usize,
    /// Its description, exactly as stored.
    pub description: Vec<u8>,
}
