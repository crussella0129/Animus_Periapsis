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

mod ignore;
pub mod tree;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use codec::{CHANGE_ID_LEN, Changeset, Mode, Operation, RepoPath, View, blob, object_id};
use hash::{HashAlg, HashRef};
use sim::{Fs, Kind, Name, Rng};
use store::Store;

pub use crate::ignore::ALWAYS_IGNORED;
use crate::ignore::Ignores;
use crate::tree::{File, Tree};

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
    /// Several op heads: concurrent operations that cannot be merged until
    /// INT-0010's canonical merge exists. Nothing was written.
    SeveralHeads(Vec<HashRef>),
    /// The repository's state does not decode or is incomplete.
    Corrupt(String),
    /// A changeset with more than one parent, whose tree needs INT-0012's
    /// merge.
    Merge(HashRef),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{error}"),
            Error::NotInitialised(root) => write!(
                f,
                "{} is not an initialised Periapsis repository (run `praps init`)",
                root.display()
            ),
            Error::SeveralHeads(heads) => {
                write!(
                    f,
                    "the op log has {} heads, which cannot be merged yet:",
                    heads.len()
                )?;
                for head in heads {
                    write!(f, " {head}")?;
                }
                Ok(())
            }
            Error::Corrupt(what) => write!(f, "the repository is corrupt: {what}"),
            Error::Merge(id) => write!(
                f,
                "changeset {id} has several parents; merges are not supported yet"
            ),
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

/// The state one op leaves, as a workspace reads it.
struct Loaded {
    head: HashRef,
    view: View,
    /// The working-copy changeset's id and value.
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
    has_bit: bool,
    ignores: Ignores,
    out: Scanned,
}

impl<F: Fs> Scan<'_, F> {
    /// Scans directory `dir` (a repository path, `""` at the top). In an
    /// ignored directory (`tracked_only`) only paths tracked before count,
    /// and nothing is reported.
    fn dir(&mut self, dir: &str, tracked_only: bool) -> Result<(), Error> {
        let fs_dir = dir
            .split('/')
            .filter(|c| !c.is_empty())
            .fold(self.root.to_path_buf(), |p, c| p.join(c));
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
        for entry in entries {
            let shown = join(&entry.name.display());
            let Name::Utf8(name) = &entry.name else {
                if !tracked_only {
                    self.report(shown, Reason::NotUtf8);
                }
                continue;
            };
            if dir.is_empty() && ALWAYS_IGNORED.contains(&name.as_str()) {
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
                    let below = self
                        .tracked
                        .keys()
                        .any(|p| p.starts_with(&path) && *p != path);
                    if !ignored || below {
                        self.dir(&full, ignored)?;
                    }
                }
                Kind::File { executable } => {
                    if ignored && !self.tracked.contains_key(&path) {
                        continue;
                    }
                    let content = self.fs.read(&fs_dir.join(name))?;
                    let id = object_id(ALG, &blob(&content));
                    let mode = if self.has_bit {
                        if executable {
                            Mode::Executable
                        } else {
                            Mode::Regular
                        }
                    } else {
                        self.tracked
                            .get(&path)
                            .map_or(Mode::Regular, |file| file.mode)
                    };
                    self.out.tree.insert(path, File { blob: id, mode });
                    self.out.contents.insert(id, content);
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
    rng: Rng,
}

/// The directory holding `root`'s entry: its parent, or `root/..` when the
/// path has none to give (a bare relative name such as `repo`, or `.`).
fn parent_dir(root: &Path) -> PathBuf {
    match root.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => root.join(".."),
    }
}

fn dir_exists(fs: &impl Fs, dir: &Path) -> io::Result<bool> {
    match fs.list(dir) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// Makes `root` and `root/.praps` exist and their entries durable: creates
/// whichever is missing (syncing the root's parent if it created the root),
/// then syncs `.praps` and the root.
fn prepare(fs: &impl Fs, root: &Path) -> io::Result<PathBuf> {
    if !dir_exists(fs, root)? {
        fs.create_dir(root)?;
        fs.sync_dir(&parent_dir(root))?;
    }
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
    /// with no parents and a change ID drawn from `seed`, a view whose head
    /// and `default` workspace are that changeset, and an op naming the view.
    /// On a store that has an op it writes nothing. Tests pass a store built
    /// with a negative-control variant here.
    pub fn init_with_store(store: Store<F>, root: &Path, seed: u64) -> Result<(Self, bool), Error> {
        let mut workspace = Workspace {
            store,
            root: root.to_path_buf(),
            rng: Rng::new(seed),
        };
        if !workspace.store.ops()?.is_empty() {
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
            heads: [change_id].into(),
            workspaces: [(DEFAULT_WORKSPACE.to_owned(), change_id)].into(),
        };
        let view_id = workspace.store.put_object(&view.to_envelope())?;
        workspace.store.commit_op(&Operation {
            parents: BTreeSet::new(),
            view: view_id,
        })?;
        Ok((workspace, true))
    }

    /// Opens the repository at `root`: makes `.praps` and the root durable
    /// as `init` does, opens the store, and checks that it has exactly one
    /// op head. `seed` drives the change IDs it mints.
    pub fn open(fs: F, root: &Path, seed: u64) -> Result<Self, Error> {
        if !dir_exists(&fs, &root.join(PRAPS_DIR).join(STORE_DIR))? {
            return Err(Error::NotInitialised(root.to_path_buf()));
        }
        let store_root = prepare(&fs, root)?;
        let store = Store::open(fs, &store_root, ALG, seed).map_err(|error| {
            if error.kind() == ErrorKind::NotFound {
                Error::NotInitialised(root.to_path_buf())
            } else {
                Error::Io(error)
            }
        })?;
        let workspace = Workspace {
            store,
            root: root.to_path_buf(),
            rng: Rng::new(seed),
        };
        workspace.load()?;
        Ok(workspace)
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

    /// The working-copy change as the single op head leaves it.
    fn load(&self) -> Result<Loaded, Error> {
        let ops = self.store.ops()?;
        let heads = store::heads_of(&ops);
        let head = match heads.len() {
            0 => return Err(Error::NotInitialised(self.root.clone())),
            1 => *heads.first().expect("one head"),
            _ => return Err(Error::SeveralHeads(heads.into_iter().collect())),
        };
        let op = &ops[&head];
        let view = View::from_envelope(&self.store.get_object(&op.view)?)
            .map_err(|error| corrupt(format!("view {}: {error}", op.view)))?;
        let wc_id = *view
            .workspaces
            .get(DEFAULT_WORKSPACE)
            .ok_or_else(|| corrupt("the view has no default workspace"))?;
        let wc = self.changeset(&wc_id)?;
        let parent_tree = match wc.parents.as_slice() {
            [] => Tree::new(),
            [parent] => self.tree_of(parent)?,
            _ => return Err(Error::Merge(wc_id)),
        };
        let wc_tree = tree::of(&wc, &parent_tree);
        Ok(Loaded {
            head,
            view,
            wc_id,
            wc,
            parent_tree,
            wc_tree,
        })
    }

    /// Scans the working copy, putting no blob yet.
    fn scan(&self, tracked: &Tree) -> Result<Scanned, Error> {
        let fs = self.store.fs();
        let mut scan = Scan {
            fs,
            root: &self.root,
            tracked,
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
        let loaded = self.load()?;
        let Scanned {
            tree,
            contents,
            reported,
        } = self.scan(&loaded.wc_tree)?;
        if tree == loaded.wc_tree {
            return Ok(Snapshot { op: None, reported });
        }
        let known: BTreeSet<HashRef> = loaded
            .wc_tree
            .values()
            .chain(loaded.parent_tree.values())
            .map(|file| file.blob)
            .collect();
        for (id, content) in &contents {
            if !known.contains(id) && tree.values().any(|file| file.blob == *id) {
                self.store.put_object(&blob(content))?;
            }
        }
        let changeset = Changeset {
            parents: loaded.wc.parents.clone(),
            change_id: loaded.wc.change_id,
            changes: tree::diff(&loaded.parent_tree, &tree),
            description: loaded.wc.description.clone(),
        };
        let op = self.replace_working_copy(&loaded, &changeset)?;
        Ok(Snapshot {
            op: Some(op),
            reported,
        })
    }

    /// Puts `changeset`, a view in which it replaces the working-copy
    /// changeset as a head and as the `default` workspace's change, and an
    /// op over the single head.
    fn replace_working_copy(
        &mut self,
        loaded: &Loaded,
        changeset: &Changeset,
    ) -> Result<HashRef, Error> {
        let id = self.store.put_object(&changeset.to_envelope())?;
        let mut view = loaded.view.clone();
        view.heads.remove(&loaded.wc_id);
        view.heads.insert(id);
        view.workspaces.insert(DEFAULT_WORKSPACE.to_owned(), id);
        let view_id = self.store.put_object(&view.to_envelope())?;
        Ok(self.store.commit_op(&Operation {
            parents: [loaded.head].into(),
            view: view_id,
        })?)
    }

    /// Snapshots, then starts a new working-copy change on top of the
    /// current one: a changeset whose only parent is the current change,
    /// with a fresh change ID and no changes. It replaces its parent as a
    /// head and becomes the `default` workspace's change. No working-copy
    /// file is written. Returns the new change ID.
    pub fn new_change(&mut self) -> Result<[u8; CHANGE_ID_LEN], Error> {
        self.snapshot()?;
        let loaded = self.load()?;
        let change_id = self.change_id();
        let changeset = Changeset {
            parents: vec![loaded.wc_id],
            change_id,
            changes: BTreeMap::new(),
            description: Vec::new(),
        };
        self.replace_working_copy(&loaded, &changeset)?;
        Ok(change_id)
    }

    /// Snapshots, then compares the working-copy change with its parent.
    pub fn status(&mut self) -> Result<Status, Error> {
        let snapshot = self.snapshot()?;
        let loaded = self.load()?;
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
    /// head leaves it.
    pub fn working_copy(&self) -> Result<(HashRef, [u8; CHANGE_ID_LEN], Tree), Error> {
        let loaded = self.load()?;
        Ok((loaded.wc_id, loaded.wc.change_id, loaded.wc_tree))
    }

    /// The working-copy change's parent tree.
    pub fn parent_tree(&self) -> Result<Tree, Error> {
        Ok(self.load()?.parent_tree)
    }
}
