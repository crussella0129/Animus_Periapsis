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
//!
//! A run with crash point *k* lets syscalls 1 to *k* − 1 take effect; syscall
//! *k* takes no effect and fails, as does every later one. [`SimFs::recover`]
//! then gives the filesystem a restart would see.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};

use crate::fs::{Crash, Fs};
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    File(u64),
    Dir,
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

#[derive(Clone, Debug, Default)]
struct Dir {
    /// What a reader sees now.
    entries: BTreeMap<String, Entry>,
    /// The entries as of the last `sync_dir` (plus lenient-mode promotions).
    durable: BTreeMap<String, Entry>,
    /// Entry changes since then, in order.
    pending: Vec<DirOp>,
}

#[derive(Clone, Debug, Default)]
struct Inode {
    current: Vec<u8>,
    durable: Vec<u8>,
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
}

/// A hook run at a syscall boundary, given the number the next syscall will
/// have. It may itself use the filesystem (the syscalls it makes are counted
/// and traced like any other).
pub type Hook = Box<dyn FnMut(u64)>;

/// The simulated filesystem. Its root is the empty path; paths are relative
/// and use normal components only.
pub struct SimFs {
    state: RefCell<State>,
    hook: RefCell<Option<Hook>>,
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
            state: RefCell::new(State {
                seed,
                mode,
                crash_at: None,
                count: 0,
                crashed: false,
                trace: Vec::new(),
                dirs,
                inodes: BTreeMap::new(),
                next_inode: 0,
            }),
            hook: RefCell::new(None),
        }
    }

    /// Crash at syscall `at` (counting from 1), or never.
    pub fn set_crash_at(&self, at: Option<u64>) {
        self.state.borrow_mut().crash_at = at;
    }

    /// Runs `hook` before every syscall from now on, or stops running it.
    pub fn set_hook(&self, hook: Option<Hook>) {
        *self.hook.borrow_mut() = hook;
    }

    /// Syscalls made so far, the crashing one included.
    pub fn syscalls(&self) -> u64 {
        self.state.borrow().count
    }

    /// Whether the crash point has been reached.
    pub fn crashed(&self) -> bool {
        self.state.borrow().crashed
    }

    /// The directory-entry mode.
    pub fn mode(&self) -> Mode {
        self.state.borrow().mode
    }

    /// One line per syscall made, in order: the operation and its path.
    pub fn trace(&self) -> Vec<String> {
        self.state.borrow().trace.clone()
    }

    /// Whether the name at `path` would survive a crash now: its directory's
    /// durable entries hold it, naming the same file or directory as now.
    /// Not a syscall: it is not counted or traced.
    pub fn is_durable(&self, path: &Path) -> bool {
        let Ok((parent, name)) = split(path) else {
            return false;
        };
        let st = self.state.borrow();
        st.dirs.get(&parent).is_some_and(|dir| {
            dir.entries.contains_key(&name) && dir.durable.get(&name) == dir.entries.get(&name)
        })
    }

    /// Everything visible now, keyed by `/`-separated path (the root
    /// excluded).
    pub fn snapshot(&self) -> BTreeMap<String, Node> {
        let st = self.state.borrow();
        let mut out = BTreeMap::new();
        let mut stack = vec![PathBuf::new()];
        while let Some(dir) = stack.pop() {
            for (name, entry) in &st.dirs[&dir].entries {
                let path = dir.join(name);
                match entry {
                    Entry::Dir => {
                        out.insert(shown(&path), Node::Dir);
                        stack.push(path);
                    }
                    Entry::File(inode) => {
                        out.insert(shown(&path), Node::File(st.inodes[inode].current.clone()));
                    }
                }
            }
        }
        out
    }

    /// The filesystem a restart sees after the crash (or after the run, if it
    /// never crashed): each pending entry change applied or not, each pending
    /// inode kept, dropped or torn, all chosen by the seed. Everything in it
    /// is durable, and it does not crash unless told to.
    pub fn recover(&self) -> SimFs {
        let st = self.state.borrow();
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
            let entries = resolved
                .get(&path)
                .cloned()
                .expect("a durable directory entry names a directory that exists");
            for (name, entry) in &entries {
                match entry {
                    Entry::Dir => stack.push(path.join(name)),
                    Entry::File(id) => {
                        if !inodes.contains_key(id) {
                            let content = settle(&st.inodes[id], &mut rng);
                            inodes.insert(
                                *id,
                                Inode {
                                    current: content.clone(),
                                    durable: content,
                                    truncated: false,
                                },
                            );
                        }
                    }
                }
            }
            dirs.insert(
                path,
                Dir {
                    entries: entries.clone(),
                    durable: entries,
                    pending: Vec::new(),
                },
            );
        }

        SimFs {
            state: RefCell::new(State {
                seed: rng.next_u64(),
                mode: st.mode,
                crash_at: None,
                count: 0,
                crashed: false,
                trace: Vec::new(),
                dirs,
                inodes,
                next_inode: st.next_inode,
            }),
            hook: RefCell::new(None),
        }
    }

    /// Counts and traces one syscall, failing it at and after the crash point.
    fn syscall(&self, op: &str, path: &Path) -> io::Result<()> {
        let next = {
            let st = self.state.borrow();
            if st.crashed {
                return Err(Crash::error());
            }
            st.count + 1
        };
        // The hook runs with no borrow held, so it may use the filesystem.
        let hook = self.hook.borrow_mut().take();
        if let Some(mut hook) = hook {
            hook(next);
            let mut slot = self.hook.borrow_mut();
            if slot.is_none() {
                *slot = Some(hook);
            }
        }
        let mut st = self.state.borrow_mut();
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
        let mut st = self.state.borrow_mut();
        let lenient = st.mode == Mode::Lenient;
        let dir = st.dir_mut(&parent, path)?;
        if dir.entries.contains_key(&name) {
            return Err(error(ErrorKind::AlreadyExists, "already exists", path));
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
        st.dirs.insert(full, Dir::default());
        Ok(())
    }

    fn create_new(&self, path: &Path) -> io::Result<()> {
        self.syscall("create_new", path)?;
        let (parent, name) = split(path)?;
        let mut st = self.state.borrow_mut();
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
        let mut st = self.state.borrow_mut();
        let id = st.file(path)?;
        st.inode(id).current.extend_from_slice(data);
        Ok(())
    }

    fn overwrite(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.syscall("overwrite", path)?;
        let mut st = self.state.borrow_mut();
        let id = st.file(path)?;
        let inode = st.inode(id);
        inode.current = data.to_vec();
        inode.truncated = true;
        Ok(())
    }

    fn sync_file(&self, path: &Path) -> io::Result<()> {
        self.syscall("sync_file", path)?;
        let (parent, name) = split(path)?;
        let mut st = self.state.borrow_mut();
        let id = st.file(path)?;
        let inode = st.inode(id);
        inode.durable = inode.current.clone();
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
        let mut st = self.state.borrow_mut();
        let dir = st.dir_mut(&from_parent, from)?;
        let entry = match dir.entries.get(&from_name) {
            Some(Entry::File(id)) => Entry::File(*id),
            Some(Entry::Dir) => {
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
        let mut st = self.state.borrow_mut();
        let dir = st.dir_mut(&full, path)?;
        dir.durable = dir.entries.clone();
        dir.pending.clear();
        Ok(())
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.syscall("read", path)?;
        let mut st = self.state.borrow_mut();
        let id = st.file(path)?;
        Ok(st.inode(id).current.clone())
    }

    fn list(&self, path: &Path) -> io::Result<Vec<String>> {
        self.syscall("list", path)?;
        let full = key(path)?;
        let st = self.state.borrow();
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
        let mut st = self.state.borrow_mut();
        let entry = Entry::File(st.file(path)?);
        let dir = st.dir_mut(&parent, path)?;
        dir.entries.remove(&name);
        dir.pending.push(DirOp::Remove { name, entry });
        Ok(())
    }
}
