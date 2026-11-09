//! An in-memory filesystem that can crash at any syscall (INT-0005 AC-1, AC-2).
//!
//! The fault model follows what POSIX promises and what real filesystems
//! were found to do (Pillai et al., "All File Systems Are Not Created Equal",
//! OSDI 2014):
//!
//! - **File data** lives in an inode. Data written since the inode's last
//!   [`Fs::sync_file`] is pending; at a crash each pending inode is,
//!   by the seed, kept, dropped (back to its last synced content) or torn (cut
//!   to a prefix of what was written, never losing synced bytes unless the
//!   file was truncated since). Pending data follows its inode across a
//!   rename: neither `rename` nor `sync_dir` makes data durable.
//! - **Directory entries** (a new name, a rename, a removal) are pending until
//!   [`Fs::sync_dir`] of their directory. At a crash each pending entry change
//!   is, by the seed, applied or not, each atomically (a rename moves its one
//!   name or leaves it). A directory whose own entry is lost takes everything
//!   below it.
//! - [`Mode::Lenient`] adds what common journaling filesystems do: a file's
//!   `sync_file` also makes its own entry durable, together with every entry
//!   change of its directory made before it (the journal commits in order),
//!   and `create_dir` is durable at once. [`Mode::Strict`] assumes only what POSIX promises.
//! - **The executable bit** is inode metadata, pending like data until
//!   [`Fs::sync_file`]; at a crash a pending change is kept or reverted by
//!   the seed. The seed is drawn only when a change is pending.
//! - **Removing a directory** ([`Fs::remove_dir`]) needs it empty, with its
//!   own entry changes all synced, so a removal that a crash reverts leaves an
//!   empty directory. While the removal is pending, the name cannot be made a
//!   directory again.
//!
//! A run with crash point *k* lets syscalls 1 to *k* − 1 take effect; syscall
//! *k* takes no effect and fails, as does every later one. [`SimFs::recover`]
//! then gives the filesystem a restart would see.
//!
//! Test helpers that are not syscalls add what the [`Fs`] methods cannot
//! make: executable bits, symbolic links, other special entries, and names
//! with no UTF-8 form ([`SimFs::make_executable`], [`SimFs::make_symlink`],
//! [`SimFs::make_other`], [`SimFs::make_raw`]). Each takes effect at once and
//! is durable, and none is counted or traced. A raw name is held as its
//! bytes or code units, the same on every host, under a key no `Path` the
//! simulator accepts can name.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};

use crate::fs::{Crash, Fs, NameRules};
use crate::name::{DirEntry, Kind, Name, RawName};
use crate::rng::Rng;

/// Which directory-entry durability the simulator assumes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    /// A file's sync also makes its entry durable; `create_dir` is durable at
    /// once.
    Lenient,
    /// Only `sync_dir` makes entries durable.
    Strict,
}

/// One visible node, as [`SimFs::snapshot`] reports it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    /// A directory.
    Dir,
    /// A file and its content.
    File(Vec<u8>),
    /// A file with its executable bit set, and its content.
    Executable(Vec<u8>),
    /// A symbolic link.
    Symlink,
    /// A special entry: a FIFO, socket or device.
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    File(u64),
    Dir,
    Symlink,
    Other,
}

#[derive(Clone, Debug)]
enum DirOp {
    Insert {
        name: String,
        entry: Entry,
    },
    /// Removes `name` if it still names `entry`.
    Remove {
        name: String,
        entry: Entry,
    },
    Rename {
        from: String,
        to: String,
        entry: Entry,
    },
}

impl DirOp {
    fn apply(&self, entries: &mut BTreeMap<String, Entry>) {
        match self {
            DirOp::Insert { name, entry } => {
                entries.insert(name.clone(), *entry);
            }
            DirOp::Remove { name, entry } => {
                if entries.get(name) == Some(entry) {
                    entries.remove(name);
                }
            }
            DirOp::Rename { from, to, entry } => {
                if entries.get(from) == Some(entry) {
                    entries.remove(from);
                }
                entries.insert(to.clone(), *entry);
            }
        }
    }

    /// Whether this op put `inode` under the name `target`.
    fn targets(&self, target: &str, inode: Entry) -> bool {
        match self {
            DirOp::Insert { name, entry } => name == target && *entry == inode,
            DirOp::Rename { to, entry, .. } => to == target && *entry == inode,
            DirOp::Remove { .. } => false,
        }
    }
}

/// The key a raw name is held under: a NUL, which no name the simulator
/// gives out holds, then its display form.
fn raw_key(raw: &RawName) -> String {
    format!("\0{}", Name::Raw(raw.clone()).display())
}

#[derive(Clone, Debug, Default)]
struct Dir {
    /// What a reader sees now.
    entries: BTreeMap<String, Entry>,
    /// The names behind raw keys (see [`raw_key`]).
    raw: BTreeMap<String, RawName>,
    /// The entries as of the last `sync_dir` (plus lenient-mode promotions).
    durable: BTreeMap<String, Entry>,
    /// Entry changes since then, in order.
    pending: Vec<DirOp>,
}

#[derive(Clone, Debug, Default)]
struct Inode {
    current: Vec<u8>,
    durable: Vec<u8>,
    /// The owner-execute bit as a reader sees it now.
    executable: bool,
    /// The bit as of the last `sync_file`.
    durable_executable: bool,
    /// Whether an `overwrite` truncated the file since its last sync, so a
    /// crash may lose bytes that were durable.
    truncated: bool,
}

#[derive(Debug)]
struct State {
    seed: u64,
    mode: Mode,
    crash_at: Option<u64>,
    count: u64,
    crashed: bool,
    trace: Vec<String>,
    dirs: BTreeMap<PathBuf, Dir>,
    inodes: BTreeMap<u64, Inode>,
    next_inode: u64,
    /// Whether files carry an executable bit ([`Fs::has_executable_bit`]).
    executable_bit: bool,
    /// What [`Fs::name_rules`] reports.
    name_rules: NameRules,
    /// Directories removed while the removal is pending in their parent: a
    /// crash that reverts it brings each back empty.
    removed: BTreeSet<PathBuf>,
}

/// A hook run at a syscall boundary, given the number the next syscall will
/// have. It may itself use the filesystem (the syscalls it makes are counted
/// and traced like any other).
pub type Hook = Box<dyn FnMut(u64) + Send>;

/// An action a test attaches to a named point with [`SimFs::on_point`]. It
/// may itself use the filesystem; its syscalls are counted and traced like
/// any other, as syscalls of the thread that reached the point.
pub type Action = Arc<dyn Fn() + Send + Sync>;

/// A boundary the scheduler of [`crate::Processes`] acts at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Boundary {
    /// Just before a syscall.
    Syscall,
    /// At a named point, after it is counted and before its actions run.
    Point(&'static str),
}

/// The scheduler's callback, run on the thread that reached a boundary with
/// no lock held. It may block (a wait for the baton or on an ordering) or
/// unwind (a kill).
pub(crate) type Switch = Arc<dyn Fn(Boundary) + Send + Sync>;

/// What the simulator keeps about named points ([`Fs::point`]), apart from
/// the filesystem state: points are not syscalls.
#[derive(Default)]
struct Points {
    /// How often each name was reached.
    reach: BTreeMap<&'static str, u64>,
    /// The actions attached to each name, in the order attached.
    actions: BTreeMap<&'static str, Vec<Action>>,
    /// The (thread, name) pairs whose actions are running now, so an action
    /// that reaches its own point is not entered again.
    running: Vec<(ThreadId, &'static str)>,
}

/// Removes a running (thread, name) pair when its actions end, even by
/// unwinding.
struct Running<'a> {
    fs: &'a SimFs,
    key: (ThreadId, &'static str),
}

impl Drop for Running<'_> {
    fn drop(&mut self) {
        self.fs.points().running.retain(|key| *key != self.key);
    }
}

/// The simulated filesystem. Its root is the empty path; paths are relative
/// and use normal components only.
///
/// It is `Send + Sync`, so several threads may share it through an `Arc`.
/// Its state sits behind a lock that is never held while a hook runs. A
/// thread that panics holding the lock does not poison it for the others:
/// the next caller takes the state as it was, so the first panic is the one
/// reported.
pub struct SimFs {
    state: Mutex<State>,
    hook: Mutex<Option<Hook>>,
    points: Mutex<Points>,
    switch: Mutex<Option<Switch>>,
}

impl SimFs {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn hook_slot(&self) -> MutexGuard<'_, Option<Hook>> {
        self.hook.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn points(&self) -> MutexGuard<'_, Points> {
        self.points.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Runs `action` each time a thread reaches the point `name`, after the
    /// point is counted, in the order actions were attached. An action is
    /// not entered again for the same name on the same thread while it runs,
    /// and no action runs once the filesystem has crashed.
    pub fn on_point(&self, name: &'static str, action: impl Fn() + Send + Sync + 'static) {
        self.points()
            .actions
            .entry(name)
            .or_default()
            .push(Arc::new(action));
    }

    /// Sets or clears the scheduler's callback (see [`crate::Processes`]).
    pub(crate) fn set_switch(&self, switch: Option<Switch>) {
        *self.switch.lock().unwrap_or_else(PoisonError::into_inner) = switch;
    }

    /// Runs the scheduler's callback, if one is set, with no lock held.
    fn switch_at(&self, at: Boundary) {
        let switch = self
            .switch
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(switch) = switch {
            switch(at);
        }
    }

    /// Removes every action attached to points.
    pub fn clear_points(&self) {
        self.points().actions.clear();
    }

    /// Each point reached so far, with how often, sorted by name.
    pub fn reach(&self) -> Vec<(&'static str, u64)> {
        self.points()
            .reach
            .iter()
            .map(|(name, count)| (*name, *count))
            .collect()
    }

    /// Panics while holding the state lock, so a test can check that the
    /// lock is not left poisoned. Not part of the simulator's model.
    #[doc(hidden)]
    pub fn panic_holding_the_lock(&self) {
        let _held = self.lock();
        panic!("a deliberate panic with the state locked");
    }
}

fn error(kind: ErrorKind, what: &str, path: &Path) -> io::Error {
    io::Error::new(kind, format!("{what}: {}", shown(path)))
}

fn shown(path: &Path) -> String {
    let parts: Vec<String> = path
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    parts.join("/")
}

/// The path as the simulator keys it: normal components only, with `.`
/// dropped and `..` taken lexically (it may not climb above the root).
fn key(path: &Path) -> io::Result<PathBuf> {
    let mut key = PathBuf::new();
    for part in path.components() {
        match part {
            // A NUL is refused, as real filesystems refuse it; it also keeps
            // the raw-name keys (see [`raw_key`]) out of reach (CR-12).
            Component::Normal(name) if name.as_encoded_bytes().contains(&0) => {
                return Err(error(ErrorKind::InvalidInput, "a name holds a NUL", path));
            }
            Component::Normal(name) => key.push(name),
            Component::CurDir => {}
            Component::ParentDir if key.pop() => {}
            _ => {
                return Err(error(
                    ErrorKind::InvalidInput,
                    "the simulator takes relative paths that stay under its root",
                    path,
                ));
            }
        }
    }
    Ok(key)
}

/// A path's parent directory and its name in it.
fn split(path: &Path) -> io::Result<(PathBuf, String)> {
    let key = key(path)?;
    let name = key
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| error(ErrorKind::InvalidInput, "no file name", path))?
        .to_owned();
    let parent = key.parent().map(Path::to_path_buf).unwrap_or_default();
    Ok((parent, name))
}

impl State {
    fn dir(&self, dir: &Path, path: &Path) -> io::Result<&Dir> {
        self.dirs
            .get(dir)
            .ok_or_else(|| error(ErrorKind::NotFound, "no such directory", path))
    }

    fn dir_mut(&mut self, dir: &Path, path: &Path) -> io::Result<&mut Dir> {
        self.dirs
            .get_mut(dir)
            .ok_or_else(|| error(ErrorKind::NotFound, "no such directory", path))
    }

    /// The inode of the file at `path`.
    fn file(&self, path: &Path) -> io::Result<u64> {
        let (parent, name) = split(path)?;
        match self.dir(&parent, path)?.entries.get(&name) {
            Some(Entry::File(inode)) => Ok(*inode),
            Some(Entry::Dir) => Err(error(ErrorKind::IsADirectory, "is a directory", path)),
            Some(Entry::Symlink | Entry::Other) => {
                Err(error(ErrorKind::InvalidInput, "not a regular file", path))
            }
            None => Err(error(ErrorKind::NotFound, "no such file", path)),
        }
    }

    fn inode(&mut self, inode: u64) -> &mut Inode {
        self.inodes
            .get_mut(&inode)
            .expect("entries name live inodes")
    }
}

impl SimFs {
    /// An empty filesystem whose crash outcomes are determined by `seed`.
    pub fn new(seed: u64, mode: Mode) -> Self {
        let mut dirs = BTreeMap::new();
        dirs.insert(PathBuf::new(), Dir::default());
        SimFs {
            state: Mutex::new(State {
                seed,
                mode,
                crash_at: None,
                count: 0,
                crashed: false,
                trace: Vec::new(),
                dirs,
                inodes: BTreeMap::new(),
                next_inode: 0,
                executable_bit: true,
                name_rules: NameRules::default(),
                removed: BTreeSet::new(),
            }),
            hook: Mutex::new(None),
            points: Mutex::new(Points::default()),
            switch: Mutex::new(None),
        }
    }

    /// Whether files carry an executable bit; `true` unless set otherwise,
    /// the same on every host.
    pub fn set_has_executable_bit(&self, has: bool) {
        self.lock().executable_bit = has;
    }

    /// What [`Fs::name_rules`] reports; neither rule unless set otherwise,
    /// the same on every host.
    pub fn set_name_rules(&self, rules: NameRules) {
        self.lock().name_rules = rules;
    }

    /// Sets a file's executable bit, durably at once. Not a syscall: the
    /// syscall is [`Fs::set_executable`].
    pub fn make_executable(&self, path: &Path, executable: bool) -> io::Result<()> {
        let mut st = self.lock();
        let id = st.file(path)?;
        let inode = st.inode(id);
        inode.executable = executable;
        inode.durable_executable = executable;
        Ok(())
    }

    /// Adds a durable entry `entry` named `name` in `dir`.
    fn make(&self, dir: &Path, name: String, entry: Entry) -> io::Result<()> {
        let full = key(dir)?;
        let mut st = self.lock();
        let dir_state = st.dir_mut(&full, dir)?;
        if dir_state.entries.contains_key(&name) {
            return Err(error(ErrorKind::AlreadyExists, "already exists", dir));
        }
        dir_state.entries.insert(name.clone(), entry);
        dir_state.durable.insert(name, entry);
        Ok(())
    }

    /// Creates a symbolic link at `path`. Not a syscall.
    pub fn make_symlink(&self, path: &Path) -> io::Result<()> {
        let (parent, name) = split(path)?;
        self.make(&parent, name, Entry::Symlink)
    }

    /// Creates a special entry (FIFO, socket, device) at `path`. Not a syscall.
    pub fn make_other(&self, path: &Path) -> io::Result<()> {
        let (parent, name) = split(path)?;
        self.make(&parent, name, Entry::Other)
    }

    /// Creates a file in `dir` whose name has no UTF-8 form, holding
    /// `content`. Not a syscall.
    pub fn make_raw(&self, dir: &Path, raw: RawName, content: &[u8]) -> io::Result<()> {
        // Only a name that really has no UTF-8 form is raw (CR-12).
        let has_utf8_form = match &raw {
            RawName::Bytes(bytes) => std::str::from_utf8(bytes).is_ok(),
            RawName::Wide(units) => char::decode_utf16(units.iter().copied()).all(|c| c.is_ok()),
        };
        if has_utf8_form {
            return Err(error(
                ErrorKind::InvalidInput,
                "the raw name has a UTF-8 form",
                dir,
            ));
        }
        let key_name = raw_key(&raw);
        let id = {
            let mut st = self.lock();
            let id = st.next_inode;
            st.next_inode += 1;
            st.inodes.insert(
                id,
                Inode {
                    current: content.to_vec(),
                    durable: content.to_vec(),
                    ..Inode::default()
                },
            );
            id
        };
        self.make(dir, key_name.clone(), Entry::File(id))?;
        let full = key(dir)?;
        self.lock().dir_mut(&full, dir)?.raw.insert(key_name, raw);
        Ok(())
    }

    /// Creates an empty directory in `dir` whose name has no UTF-8 form. Not
    /// a syscall. Like every raw name, no `Path` reaches inside it.
    pub fn make_raw_dir(&self, dir: &Path, raw: RawName) -> io::Result<()> {
        let has_utf8_form = match &raw {
            RawName::Bytes(bytes) => std::str::from_utf8(bytes).is_ok(),
            RawName::Wide(units) => char::decode_utf16(units.iter().copied()).all(|c| c.is_ok()),
        };
        if has_utf8_form {
            return Err(error(
                ErrorKind::InvalidInput,
                "the raw name has a UTF-8 form",
                dir,
            ));
        }
        let key_name = raw_key(&raw);
        self.make(dir, key_name.clone(), Entry::Dir)?;
        let full = key(dir)?;
        let mut st = self.lock();
        st.dirs.insert(full.join(&key_name), Dir::default());
        st.dir_mut(&full, dir)?.raw.insert(key_name, raw);
        Ok(())
    }

    /// Crash at syscall `at` (counting from 1), or never.
    pub fn set_crash_at(&self, at: Option<u64>) {
        self.lock().crash_at = at;
    }

    /// Runs `hook` before every syscall from now on, or stops running it.
    pub fn set_hook(&self, hook: Option<Hook>) {
        *self.hook_slot() = hook;
    }

    /// Syscalls made so far, the crashing one included.
    pub fn syscalls(&self) -> u64 {
        self.lock().count
    }

    /// Whether the crash point has been reached.
    pub fn crashed(&self) -> bool {
        self.lock().crashed
    }

    /// The directory-entry mode.
    pub fn mode(&self) -> Mode {
        self.lock().mode
    }

    /// One line per syscall made, in order: the operation and its path.
    pub fn trace(&self) -> Vec<String> {
        self.lock().trace.clone()
    }

    /// Whether the name at `path` would survive a crash now: its directory's
    /// durable entries hold it, naming the same file or directory as now.
    /// Not a syscall: it is not counted or traced.
    pub fn is_durable(&self, path: &Path) -> bool {
        let Ok((parent, name)) = split(path) else {
            return false;
        };
        let st = self.lock();
        st.dirs.get(&parent).is_some_and(|dir| {
            dir.entries.contains_key(&name) && dir.durable.get(&name) == dir.entries.get(&name)
        })
    }

    /// Everything visible now, keyed by `/`-separated path (the root
    /// excluded). A raw name appears by its display form.
    pub fn snapshot(&self) -> BTreeMap<String, Node> {
        let st = self.lock();
        let mut out = BTreeMap::new();
        let mut stack = vec![PathBuf::new()];
        while let Some(dir) = stack.pop() {
            let dir_state = &st.dirs[&dir];
            for (name, entry) in &dir_state.entries {
                let path = dir.join(name);
                // Every name by its display form, so a raw name and a UTF-8
                // name spelled like its escape stay apart.
                let display = match dir_state.raw.get(name) {
                    Some(raw) => Name::Raw(raw.clone()).display(),
                    None => Name::Utf8(name.clone()).display(),
                };
                // Joined as text: a display form may hold a `\`, which a
                // Windows `Path` would split on.
                let shown_path = match shown(&dir) {
                    parent if parent.is_empty() => display,
                    parent => format!("{parent}/{display}"),
                };
                let node = match entry {
                    Entry::Dir => {
                        stack.push(path);
                        Node::Dir
                    }
                    Entry::File(inode) => {
                        let inode = &st.inodes[inode];
                        if inode.executable {
                            Node::Executable(inode.current.clone())
                        } else {
                            Node::File(inode.current.clone())
                        }
                    }
                    Entry::Symlink => Node::Symlink,
                    Entry::Other => Node::Other,
                };
                out.insert(shown_path, node);
            }
        }
        out
    }

    /// The filesystem a restart sees after the crash (or after the run, if it
    /// never crashed): each pending entry change applied or not, each pending
    /// inode kept, dropped or torn, all chosen by the seed. Everything in it
    /// is durable, and it does not crash unless told to.
    pub fn recover(&self) -> SimFs {
        let st = self.lock();
        let mut rng = Rng::new(st.seed);
        let mut resolved: BTreeMap<&PathBuf, BTreeMap<String, Entry>> = BTreeMap::new();
        for (path, dir) in &st.dirs {
            let mut entries = dir.durable.clone();
            for op in &dir.pending {
                if rng.coin() {
                    op.apply(&mut entries);
                }
            }
            resolved.insert(path, entries);
        }

        let mut dirs = BTreeMap::new();
        let mut inodes = BTreeMap::new();
        let mut stack = vec![PathBuf::new()];
        while let Some(path) = stack.pop() {
            let entries = match resolved.get(&path) {
                Some(entries) => entries.clone(),
                // A removal the crash reverted: the directory was empty.
                None if st.removed.contains(&path) => BTreeMap::new(),
                None => panic!("a durable directory entry names a directory that exists"),
            };
            for (name, entry) in &entries {
                match entry {
                    Entry::Dir => stack.push(path.join(name)),
                    Entry::File(id) => {
                        if !inodes.contains_key(id) {
                            let inode = &st.inodes[id];
                            let content = settle(inode, &mut rng);
                            let executable =
                                if inode.executable == inode.durable_executable || rng.coin() {
                                    inode.executable
                                } else {
                                    inode.durable_executable
                                };
                            inodes.insert(
                                *id,
                                Inode {
                                    current: content.clone(),
                                    durable: content,
                                    truncated: false,
                                    executable,
                                    durable_executable: executable,
                                },
                            );
                        }
                    }
                    Entry::Symlink | Entry::Other => {}
                }
            }
            let raw = st
                .dirs
                .get(&path)
                .map(|dir| {
                    dir.raw
                        .iter()
                        .filter(|(key, _)| entries.contains_key(*key))
                        .map(|(key, raw)| (key.clone(), raw.clone()))
                        .collect()
                })
                .unwrap_or_default();
            dirs.insert(
                path,
                Dir {
                    entries: entries.clone(),
                    raw,
                    durable: entries,
                    pending: Vec::new(),
                },
            );
        }

        SimFs {
            state: Mutex::new(State {
                seed: rng.next_u64(),
                mode: st.mode,
                crash_at: None,
                count: 0,
                crashed: false,
                trace: Vec::new(),
                dirs,
                inodes,
                next_inode: st.next_inode,
                executable_bit: st.executable_bit,
                name_rules: st.name_rules,
                removed: BTreeSet::new(),
            }),
            hook: Mutex::new(None),
            points: Mutex::new(Points::default()),
            switch: Mutex::new(None),
        }
    }

    /// Counts and traces one syscall, failing it at and after the crash point.
    fn syscall(&self, op: &str, path: &Path) -> io::Result<()> {
        // Several simulated processes switch here, before anything else.
        self.switch_at(Boundary::Syscall);
        let next = {
            let st = self.lock();
            if st.crashed {
                return Err(Crash::error());
            }
            st.count + 1
        };
        // The hook runs with no lock held, so it may use the filesystem.
        let hook = self.hook_slot().take();
        if let Some(mut hook) = hook {
            hook(next);
            let mut slot = self.hook_slot();
            if slot.is_none() {
                *slot = Some(hook);
            }
        }
        let mut st = self.lock();
        if st.crashed {
            return Err(Crash::error());
        }
        st.count += 1;
        let line = format!("{op} {}", shown(path));
        if st.crash_at == Some(st.count) {
            st.crashed = true;
            st.trace.push(format!("{line} (crash)"));
            return Err(Crash::error());
        }
        st.trace.push(line);
        Ok(())
    }
}

/// What a crash leaves of one inode.
fn settle(inode: &Inode, rng: &mut Rng) -> Vec<u8> {
    if inode.current == inode.durable && !inode.truncated {
        return inode.durable.clone();
    }
    match rng.below(3) {
        0 => inode.durable.clone(),
        1 => inode.current.clone(),
        _ => {
            let floor = if inode.truncated {
                0
            } else {
                inode.durable.len()
            };
            let len = inode.current.len();
            let cut = if len > floor {
                floor + rng.below((len - floor) as u64) as usize
            } else {
                len
            };
            inode.current[..cut].to_vec()
        }
    }
}

impl Fs for SimFs {
    fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.syscall("create_dir", path)?;
        let (parent, name) = split(path)?;
        let full = key(path)?;
        let mut st = self.lock();
        let lenient = st.mode == Mode::Lenient;
        let dir = st.dir_mut(&parent, path)?;
        if dir.entries.contains_key(&name) {
            return Err(error(ErrorKind::AlreadyExists, "already exists", path));
        }
        let removal_pending = dir.pending.iter().any(|op| {
            matches!(op, DirOp::Remove { name: removed, entry: Entry::Dir } if *removed == name)
        });
        if removal_pending {
            return Err(error(
                ErrorKind::ResourceBusy,
                "the directory's removal is not yet durable; sync its parent first",
                path,
            ));
        }
        dir.entries.insert(name.clone(), Entry::Dir);
        if lenient {
            dir.durable.insert(name, Entry::Dir);
        } else {
            dir.pending.push(DirOp::Insert {
                name,
                entry: Entry::Dir,
            });
        }
        st.removed.remove(&full);
        st.dirs.insert(full, Dir::default());
        Ok(())
    }

    fn create_new(&self, path: &Path) -> io::Result<()> {
        self.syscall("create_new", path)?;
        let (parent, name) = split(path)?;
        let mut st = self.lock();
        let id = st.next_inode;
        let dir = st.dir_mut(&parent, path)?;
        if dir.entries.contains_key(&name) {
            return Err(error(ErrorKind::AlreadyExists, "already exists", path));
        }
        let entry = Entry::File(id);
        dir.entries.insert(name.clone(), entry);
        dir.pending.push(DirOp::Insert { name, entry });
        st.next_inode += 1;
        st.inodes.insert(id, Inode::default());
        Ok(())
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.syscall("write", path)?;
        let mut st = self.lock();
        let id = st.file(path)?;
        st.inode(id).current.extend_from_slice(data);
        Ok(())
    }

    fn overwrite(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.syscall("overwrite", path)?;
        let mut st = self.lock();
        let id = st.file(path)?;
        let inode = st.inode(id);
        inode.current = data.to_vec();
        inode.truncated = true;
        Ok(())
    }

    fn sync_file(&self, path: &Path) -> io::Result<()> {
        self.syscall("sync_file", path)?;
        let (parent, name) = split(path)?;
        let mut st = self.lock();
        let id = st.file(path)?;
        let inode = st.inode(id);
        inode.durable = inode.current.clone();
        inode.durable_executable = inode.executable;
        inode.truncated = false;
        if st.mode == Mode::Lenient {
            // A journaling filesystem commits a directory's entry changes in
            // order: making this file's entry durable commits every pending
            // change of the directory up to the one that gave it this name.
            let dir = st.dir_mut(&parent, path)?;
            let inode = Entry::File(id);
            if let Some(last) = dir.pending.iter().rposition(|op| op.targets(&name, inode)) {
                for op in dir.pending.drain(..=last) {
                    op.apply(&mut dir.durable);
                }
            }
        }
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.syscall("rename", from)?;
        let (from_parent, from_name) = split(from)?;
        let (to_parent, to_name) = split(to)?;
        if from_parent != to_parent {
            return Err(error(
                ErrorKind::Unsupported,
                "the simulator renames within one directory",
                to,
            ));
        }
        let mut st = self.lock();
        let dir = st.dir_mut(&from_parent, from)?;
        let entry = match dir.entries.get(&from_name) {
            Some(Entry::File(id)) => Entry::File(*id),
            Some(Entry::Dir | Entry::Symlink | Entry::Other) => {
                return Err(error(
                    ErrorKind::Unsupported,
                    "the simulator renames files only",
                    from,
                ));
            }
            None => return Err(error(ErrorKind::NotFound, "no such file", from)),
        };
        if dir.entries.get(&to_name) == Some(&Entry::Dir) {
            return Err(error(ErrorKind::IsADirectory, "is a directory", to));
        }
        dir.entries.remove(&from_name);
        dir.entries.insert(to_name.clone(), entry);
        dir.pending.push(DirOp::Rename {
            from: from_name,
            to: to_name,
            entry,
        });
        Ok(())
    }

    fn sync_dir(&self, path: &Path) -> io::Result<()> {
        self.syscall("sync_dir", path)?;
        let full = key(path)?;
        let mut st = self.lock();
        let dir = st.dir_mut(&full, path)?;
        dir.durable = dir.entries.clone();
        dir.pending.clear();
        Ok(())
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.syscall("read", path)?;
        let mut st = self.lock();
        let id = st.file(path)?;
        Ok(st.inode(id).current.clone())
    }

    fn list(&self, path: &Path) -> io::Result<Vec<String>> {
        self.syscall("list", path)?;
        let full = key(path)?;
        let st = self.lock();
        if let Some(dir) = st.dirs.get(&full) {
            return Ok(dir.entries.keys().cloned().collect());
        }
        match st.file(path) {
            Ok(_) => Err(error(ErrorKind::NotADirectory, "not a directory", path)),
            Err(error) => Err(error),
        }
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.syscall("remove_file", path)?;
        let (parent, name) = split(path)?;
        let mut st = self.lock();
        let entry = Entry::File(st.file(path)?);
        let dir = st.dir_mut(&parent, path)?;
        dir.entries.remove(&name);
        dir.pending.push(DirOp::Remove { name, entry });
        Ok(())
    }

    fn scan(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
        self.syscall("scan", path)?;
        let full = key(path)?;
        let st = self.lock();
        let Some(dir) = st.dirs.get(&full) else {
            return match st.file(path) {
                Ok(_) => Err(error(ErrorKind::NotADirectory, "not a directory", path)),
                Err(error) => Err(error),
            };
        };
        let executable_bit = st.executable_bit;
        let mut entries: Vec<DirEntry> = dir
            .entries
            .iter()
            .map(|(name, entry)| DirEntry {
                name: match dir.raw.get(name) {
                    Some(raw) => Name::Raw(raw.clone()),
                    None => Name::Utf8(name.clone()),
                },
                kind: match entry {
                    Entry::File(id) => Kind::File {
                        executable: executable_bit && st.inodes[id].executable,
                    },
                    Entry::Dir => Kind::Dir,
                    Entry::Symlink => Kind::Symlink,
                    Entry::Other => Kind::Other,
                },
            })
            .collect();
        entries.sort_by_cached_key(|entry| entry.name.display());
        Ok(entries)
    }

    fn has_executable_bit(&self) -> bool {
        self.lock().executable_bit
    }

    fn remove_dir(&self, path: &Path) -> io::Result<()> {
        self.syscall("remove_dir", path)?;
        let (parent, name) = split(path)?;
        let full = key(path)?;
        let mut st = self.lock();
        match st.dir(&parent, path)?.entries.get(&name) {
            Some(Entry::Dir) => {}
            Some(_) => return Err(error(ErrorKind::NotADirectory, "not a directory", path)),
            None => return Err(error(ErrorKind::NotFound, "no such directory", path)),
        }
        let target = st.dir(&full, path)?;
        if !target.entries.is_empty() {
            return Err(error(
                ErrorKind::DirectoryNotEmpty,
                "directory not empty",
                path,
            ));
        }
        if !target.pending.is_empty() || !target.durable.is_empty() {
            return Err(error(
                ErrorKind::ResourceBusy,
                "the directory's own entry changes are not yet durable; sync it first",
                path,
            ));
        }
        let dir = st.dir_mut(&parent, path)?;
        dir.entries.remove(&name);
        dir.pending.push(DirOp::Remove {
            name,
            entry: Entry::Dir,
        });
        st.dirs.remove(&full);
        st.removed.insert(full);
        Ok(())
    }

    fn set_executable(&self, path: &Path, executable: bool) -> io::Result<()> {
        self.syscall("set_executable", path)?;
        let mut st = self.lock();
        let id = st.file(path)?;
        if st.executable_bit {
            st.inode(id).executable = executable;
        }
        Ok(())
    }

    fn name_rules(&self) -> NameRules {
        self.lock().name_rules
    }

    fn point(&self, name: &'static str) {
        // Counted first, with no syscall, trace line or seed draw, so a point
        // changes nothing a single-process run does.
        self.points()
            .reach
            .entry(name)
            .and_modify(|n| *n += 1)
            .or_insert(1);
        // Under several simulated processes: a kill, then any ordering wait,
        // happen here, after the count and before the actions.
        self.switch_at(Boundary::Point(name));
        let crashed = self.lock().crashed;
        let me = thread::current().id();
        let actions = {
            let points = self.points();
            if crashed || points.running.contains(&(me, name)) {
                return;
            }
            match points.actions.get(name) {
                Some(actions) if !actions.is_empty() => actions.clone(),
                _ => return,
            }
        };
        self.points().running.push((me, name));
        let _running = Running {
            fs: self,
            key: (me, name),
        };
        // No lock is held while the actions run.
        for action in &actions {
            action();
        }
    }
}
