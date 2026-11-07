#![forbid(unsafe_code)]
//! The crash-safe object store and op log (INT-0007).
//!
//! A store is a directory holding `objects/` and `ops/`. Every object and
//! every operation is one file, named for its id (see [`names`]), whose
//! content is the object's envelope. There is no HEAD or pointer file: the op
//! heads are derived from the op files (R2-E23b; after Jujutsu's op-heads
//! store, minus the head files). Nothing is ever deleted or changed in place
//! (INT-0009): an id's file only ever holds that id's bytes.
//!
//! Every file is written the same way, after the fault model in Pillai et
//! al., OSDI 2014: write a fresh temporary file, sync it, rename it to its
//! id, sync the directory. A store call returns only after that last sync,
//! and its return is the acknowledgement. All I/O goes through [`sim::Fs`],
//! so the same code runs on the real filesystem and in the crash simulator.

mod fsck;
pub mod names;
#[cfg(feature = "variants")]
mod variant;

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use codec::{Operation, decode_envelope, from_slice, object_id, to_vec};
use hash::{HashAlg, HashRef};
use sim::{Fs, Rng};

pub use fsck::{Problem, ProblemKind, Report};
use names::{TMP_PREFIX, file_name, parse_name};
#[cfg(feature = "variants")]
pub use variant::Variant;

/// The directory of object files.
pub const OBJECTS: &str = "objects";
/// The directory of op files.
pub const OPS: &str = "ops";
/// The pointer file only the pointer-file negative control writes.
const POINTER: &str = "HEAD";

/// A store on a filesystem `F`.
pub struct Store<F: Fs> {
    fs: F,
    root: PathBuf,
    objects: PathBuf,
    ops: PathBuf,
    alg: HashAlg,
    rng: Rng,
    #[cfg(feature = "variants")]
    variant: Variant,
}

/// The error for a directory that is not a store.
fn not_a_store(root: &Path) -> io::Error {
    io::Error::new(
        ErrorKind::NotFound,
        format!(
            "{} is not a Periapsis store (it needs {OBJECTS}/ and {OPS}/)",
            root.display()
        ),
    )
}

impl<F: Fs> Store<F> {
    fn new(fs: F, root: &Path, alg: HashAlg, seed: u64) -> Self {
        Store {
            fs,
            root: root.to_path_buf(),
            objects: root.join(OBJECTS),
            ops: root.join(OPS),
            alg,
            rng: Rng::new(seed),
            #[cfg(feature = "variants")]
            variant: Variant::Correct,
        }
    }

    #[cfg(feature = "variants")]
    fn pointer_file(&self) -> bool {
        self.variant == Variant::PointerFile
    }
    #[cfg(not(feature = "variants"))]
    fn pointer_file(&self) -> bool {
        false
    }
    #[cfg(feature = "variants")]
    fn syncs_object_data(&self) -> bool {
        self.variant != Variant::UnsyncedObjects
    }
    #[cfg(not(feature = "variants"))]
    fn syncs_object_data(&self) -> bool {
        true
    }
    #[cfg(feature = "variants")]
    fn syncs_at_init(&self) -> bool {
        self.variant != Variant::UnsyncedInit
    }
    #[cfg(not(feature = "variants"))]
    fn syncs_at_init(&self) -> bool {
        true
    }

    /// [`Store::init`] with a negative-control variant.
    #[cfg(feature = "variants")]
    pub fn init_variant(
        fs: F,
        root: &Path,
        alg: HashAlg,
        seed: u64,
        variant: Variant,
    ) -> io::Result<Self> {
        let mut store = Store::new(fs, root, alg, seed);
        store.variant = variant;
        store.finish_init()
    }

    /// [`Store::open`] with a negative-control variant.
    #[cfg(feature = "variants")]
    pub fn open_variant(
        fs: F,
        root: &Path,
        alg: HashAlg,
        seed: u64,
        variant: Variant,
    ) -> io::Result<Self> {
        let mut store = Store::open(fs, root, alg, seed)?;
        store.variant = variant;
        Ok(store)
    }

    /// Makes `root` a store: creates whichever of the root, `objects/` and
    /// `ops/` is missing, then syncs both, the root and the root's parent,
    /// returning after the last sync. On a complete store it creates
    /// nothing, so `init` is idempotent and completes an interrupted init.
    /// New objects are hashed with `alg`; `seed` drives temporary-file names.
    pub fn init(fs: F, root: &Path, alg: HashAlg, seed: u64) -> io::Result<Self> {
        Store::new(fs, root, alg, seed).finish_init()
    }

    fn finish_init(self) -> io::Result<Self> {
        for dir in [&self.root, &self.objects, &self.ops] {
            if !self.dir_exists(dir)? {
                self.fs.create_dir(dir)?;
            }
        }
        if self.pointer_file() {
            let pointer = self.root.join(POINTER);
            match self.fs.create_new(&pointer) {
                Ok(()) => self
                    .fs
                    .overwrite(&pointer, &to_vec(&BTreeSet::<HashRef>::new()))?,
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        if self.syncs_at_init() {
            self.sync_layout()?;
        }
        Ok(self)
    }

    /// Syncs `objects/`, `ops/`, the root and the root's parent, so the
    /// layout's own entries are durable. `init` ends with it, and `open`
    /// runs it too: a process that died inside `init` may have left a layout
    /// that exists but whose entries were never synced, and acknowledged ops
    /// written into it would be lost with it.
    fn sync_layout(&self) -> io::Result<()> {
        self.fs.sync_dir(&self.objects)?;
        self.fs.sync_dir(&self.ops)?;
        self.fs.sync_dir(&self.root)?;
        self.fs.sync_dir(&parent_dir(&self.root))
    }

    /// Opens an existing store; a directory without `objects/` and `ops/` is
    /// not one.
    pub fn open(fs: F, root: &Path, alg: HashAlg, seed: u64) -> io::Result<Self> {
        let store = Store::new(fs, root, alg, seed);
        if store.dir_exists(&store.objects)? && store.dir_exists(&store.ops)? {
            if store.syncs_at_init() {
                store.sync_layout()?;
            }
            Ok(store)
        } else {
            Err(not_a_store(root))
        }
    }

    /// The filesystem the store uses.
    pub fn fs(&self) -> &F {
        &self.fs
    }

    /// The store's directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn dir_exists(&self, dir: &Path) -> io::Result<bool> {
        match self.fs.list(dir) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Stores an object envelope and returns its id once it is durable.
    pub fn put_object(&mut self, envelope: &[u8]) -> io::Result<HashRef> {
        decode_envelope(envelope)
            .map_err(|error| io::Error::new(ErrorKind::InvalidInput, error))?;
        let id = object_id(self.alg, envelope);
        let dir = self.objects.clone();
        self.write_file(&dir, &file_name(&id), envelope)?;
        Ok(id)
    }

    /// An object's envelope.
    pub fn get_object(&self, id: &HashRef) -> io::Result<Vec<u8>> {
        self.fs.read(&self.objects.join(file_name(id)))
    }

    /// Appends an operation to the op log and returns its id once it is
    /// durable: the acknowledgement. Every parent op and object it names must
    /// already be present (hashing to its id); otherwise nothing is written.
    /// Those files are synced again, with their directories, before the op is
    /// written, since another writer may have renamed one without syncing
    /// yet (INT-0007 Consequences).
    pub fn commit_op(&mut self, op: &Operation) -> io::Result<HashRef> {
        for parent in &op.parents {
            if !self.valid_op(parent)? {
                return Err(io::Error::new(
                    ErrorKind::NotFound,
                    format!("parent op {} is not in the store", file_name(parent)),
                ));
            }
        }
        for object in &op.objects {
            if !self.valid_object(object)? {
                return Err(io::Error::new(
                    ErrorKind::NotFound,
                    format!("object {} is not in the store", file_name(object)),
                ));
            }
        }
        if self.syncs_object_data() {
            for object in &op.objects {
                self.fs.sync_file(&self.objects.join(file_name(object)))?;
            }
        }
        if !op.objects.is_empty() {
            self.fs.sync_dir(&self.objects)?;
        }
        for parent in &op.parents {
            self.fs.sync_file(&self.ops.join(file_name(parent)))?;
        }
        if !op.parents.is_empty() {
            self.fs.sync_dir(&self.ops)?;
        }
        let envelope = op.to_envelope();
        let id = object_id(self.alg, &envelope);
        let dir = self.ops.clone();
        self.write_file(&dir, &file_name(&id), &envelope)?;
        if self.pointer_file() {
            let mut heads = self.heads()?;
            for parent in &op.parents {
                heads.remove(parent);
            }
            heads.insert(id);
            self.fs
                .overwrite(&self.root.join(POINTER), &to_vec(&heads))?;
        }
        Ok(id)
    }

    /// Whether the object file for `id` exists and hashes to `id`.
    fn valid_object(&self, id: &HashRef) -> io::Result<bool> {
        match self.fs.read(&self.objects.join(file_name(id))) {
            Ok(bytes) => Ok(object_id(id.alg(), &bytes) == *id),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Whether the op file for `id` exists, hashes to `id` and decodes.
    fn valid_op(&self, id: &HashRef) -> io::Result<bool> {
        match self.fs.read(&self.ops.join(file_name(id))) {
            Ok(bytes) => {
                Ok(object_id(id.alg(), &bytes) == *id && Operation::from_envelope(&bytes).is_ok())
            }
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Writes `bytes` as `dir/name` durably. A file already there with these
    /// bytes is synced again with its directory; one with other bytes (a copy
    /// a crash damaged) is replaced through the same temporary-file path.
    fn write_file(&mut self, dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
        let path = dir.join(name);
        let sync_data = dir != self.objects || self.syncs_object_data();
        match self.fs.read(&path) {
            Ok(existing) if existing == bytes => {
                if sync_data {
                    self.fs.sync_file(&path)?;
                }
                return self.fs.sync_dir(dir);
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let tmp = loop {
            let tmp = dir.join(format!("{TMP_PREFIX}{:016x}", self.rng.next_u64()));
            match self.fs.create_new(&tmp) {
                Ok(()) => break tmp,
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        };
        self.fs.write(&tmp, bytes)?;
        if sync_data {
            self.fs.sync_file(&tmp)?;
        }
        self.fs.rename(&tmp, &path)?;
        self.fs.sync_dir(dir)
    }

    /// Every valid op in the store: its file name parses, its content hashes
    /// to that id, and it decodes as an operation.
    pub fn ops(&self) -> io::Result<BTreeMap<HashRef, Operation>> {
        let mut ops = BTreeMap::new();
        for name in self.fs.list(&self.ops)? {
            let Some(id) = parse_name(&name) else {
                continue;
            };
            let bytes = self.fs.read(&self.ops.join(&name))?;
            if object_id(id.alg(), &bytes) != id {
                continue;
            }
            if let Ok(op) = Operation::from_envelope(&bytes) {
                ops.insert(id, op);
            }
        }
        Ok(ops)
    }

    /// The op heads, derived from the op files: every valid op that no other
    /// valid op names as a parent.
    pub fn heads(&self) -> io::Result<BTreeSet<HashRef>> {
        if self.pointer_file() {
            let bytes = self.fs.read(&self.root.join(POINTER))?;
            return from_slice(&bytes)
                .map_err(|error| io::Error::new(ErrorKind::InvalidData, error));
        }
        Ok(heads_of(&self.ops()?))
    }

    /// Checks every file in the store; see [`Report`].
    pub fn fsck(&self) -> io::Result<Report> {
        fsck::check(&self.fs, &self.root)
    }
}

/// The directory holding `root`'s entry: its parent, or `root/..` when the
/// path has none to give (a bare relative name such as `repo`, or `.`).
fn parent_dir(root: &Path) -> PathBuf {
    match root.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => root.join(".."),
    }
}

/// The heads of a set of ops: those no op in the set names as a parent.
pub fn heads_of(ops: &BTreeMap<HashRef, Operation>) -> BTreeSet<HashRef> {
    let named: BTreeSet<&HashRef> = ops.values().flat_map(|op| &op.parents).collect();
    ops.keys()
        .filter(|id| !named.contains(id))
        .cloned()
        .collect()
}
