//! Materialise: moving the files on disk from tree F to tree T, leaving every
//! user edit in place (INT-0014 AC-1, INT-0009 AC-3).
//!
//! [`plan`] classifies each path of the F→T diff against what is on disk and
//! finds what must stop the write: blocking entries and hazards. [`apply`]
//! then makes the changes in an order that keeps every intermediate state
//! one that a crash, followed by a resume, can finish.
//! `docs/spec/working-copy.md` ("Classifying paths", "Materialise") is the
//! specification.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use codec::{Mode, RepoPath, blob, object_id};
use hash::{HashAlg, HashRef};
use sim::{Fs, Kind, NameRules};

use crate::hazard::{self, Hazard, fold_eq};
use crate::tree::{File, Tree};

/// The prefix of the temporary names a write uses.
pub const TMP_PREFIX: &str = ".praps-tmp-";

/// What is on disk at a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OnDisk {
    Absent,
    File { executable: bool },
    Dir,
    Other,
}

/// The plan for one write.
#[derive(Debug, Default)]
pub struct Plan {
    /// Files to remove, sorted.
    pub remove: Vec<RepoPath>,
    /// Directories that may be left empty, deepest first: each is removed
    /// if, after the removals, it exists and is empty.
    pub emptied: Vec<String>,
    /// Files to write, sorted, each with its temporary name and T's entry.
    pub write: Vec<(RepoPath, String, File)>,
    /// Blocking entries, described.
    pub blocked: Vec<String>,
    /// Hazards (INT-0004 AC-3, AC-4).
    pub hazards: Vec<Hazard>,
    /// When resuming: the paths left as they are, described.
    pub skipped: Vec<String>,
}

/// A repository path's parent directory (`""` at the top) and name.
fn split(path: &str) -> (&str, &str) {
    match path.rsplit_once('/') {
        Some((dir, name)) => (dir, name),
        None => ("", path),
    }
}

/// Every proper ancestor directory of `path`, shallowest first.
fn ancestors(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/').map(move |(i, _)| &path[..i])
}

/// `path` (a repository path, `""` for the root) under `root` on disk.
pub fn on_disk(root: &Path, path: &str) -> PathBuf {
    path.split('/')
        .filter(|c| !c.is_empty())
        .fold(root.to_path_buf(), |p, c| p.join(c))
}

/// The temporary name of the `n`th path of the diff, in `dir`.
fn tmp_path(dir: &str, n: usize) -> String {
    if dir.is_empty() {
        format!("{TMP_PREFIX}{n}")
    } else {
        format!("{dir}/{TMP_PREFIX}{n}")
    }
}

/// The F→T diff: every path whose entry differs, sorted, with the
/// temporary name of each (a pure function of F and T).
pub fn diff(from: &Tree, target: &Tree) -> Vec<(RepoPath, String)> {
    let paths: BTreeSet<&RepoPath> = from
        .keys()
        .chain(target.keys())
        .filter(|path| from.get(*path) != target.get(*path))
        .collect();
    paths
        .into_iter()
        .enumerate()
        .map(|(n, path)| (path.clone(), tmp_path(split(path.as_str()).0, n)))
        .collect()
}

/// A directory entry's key in a listing: its name where it is UTF-8 (as a
/// repository path spells it), else a NUL and its display form, which no
/// repository path can equal (code review CR-02).
fn listing_key(name: &sim::Name) -> String {
    match name.as_utf8() {
        Some(utf8) => utf8.to_owned(),
        None => format!("\0{}", name.display()),
    }
}

/// Reads the disk lazily, one directory scan each.
struct Disk<'a, F: Fs> {
    fs: &'a F,
    root: &'a Path,
    has_bit: bool,
    /// Each directory's entries by name, or `None` where it is missing.
    dirs: BTreeMap<String, Option<BTreeMap<String, Kind>>>,
}

impl<F: Fs> Disk<'_, F> {
    fn listing(&mut self, dir: &str) -> io::Result<Option<&BTreeMap<String, Kind>>> {
        if !self.dirs.contains_key(dir) {
            let listed = match self.fs.scan(&on_disk(self.root, dir)) {
                Ok(entries) => Some(
                    entries
                        .into_iter()
                        .map(|entry| (listing_key(&entry.name), entry.kind))
                        .collect(),
                ),
                Err(error)
                    if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
            self.dirs.insert(dir.to_owned(), listed);
        }
        Ok(self.dirs[dir].as_ref())
    }

    fn at(&mut self, path: &str) -> io::Result<OnDisk> {
        let (dir, name) = split(path);
        Ok(
            match self.listing(dir)?.and_then(|entries| entries.get(name)) {
                None => OnDisk::Absent,
                Some(Kind::File { executable }) => OnDisk::File {
                    executable: *executable,
                },
                Some(Kind::Dir) => OnDisk::Dir,
                Some(Kind::Symlink | Kind::Other) => OnDisk::Other,
            },
        )
    }

    /// Whether the file at `path` holds `file`: its bytes, and its mode
    /// where files carry an executable bit.
    fn holds(&mut self, path: &str, file: &File, alg: HashAlg) -> io::Result<bool> {
        let OnDisk::File { executable } = self.at(path)? else {
            return Ok(false);
        };
        if self.has_bit && executable != (file.mode == Mode::Executable) {
            return Ok(false);
        }
        match self.fs.read(&on_disk(self.root, path)) {
            Ok(content) => Ok(object_id(alg, &blob(&content)) == file.blob),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Whether everything in directory `dir` belongs to F: files to remove,
    /// and directories that hold only those or are empty where F had files.
    fn f_owned_dir(
        &mut self,
        dir: &str,
        remove: &BTreeSet<&str>,
        f_dirs: &BTreeSet<&str>,
    ) -> io::Result<bool> {
        let Some(entries) = self.listing(dir)?.cloned() else {
            return Ok(false);
        };
        if entries.is_empty() {
            return Ok(f_dirs.contains(dir));
        }
        for (name, kind) in entries {
            let path = format!("{dir}/{name}");
            let owned = match kind {
                Kind::File { .. } => remove.contains(path.as_str()),
                Kind::Dir => self.f_owned_dir(&path, remove, f_dirs)?,
                Kind::Symlink | Kind::Other => false,
            };
            if !owned {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Plans the write of T over F. When `resume` is set, blocking entries and
/// hazards do not stop the write: their paths are left as they are and
/// listed in [`Plan::skipped`].
#[allow(clippy::too_many_arguments)]
pub fn plan(
    fs: &impl Fs,
    root: &Path,
    from: &Tree,
    target: &Tree,
    rules: NameRules,
    alg: HashAlg,
    resume: bool,
) -> io::Result<Plan> {
    let mut disk = Disk {
        fs,
        root,
        has_bit: fs.has_executable_bit(),
        dirs: BTreeMap::new(),
    };
    let diff = diff(from, target);
    // Every directory F had files under.
    let f_dirs: BTreeSet<&str> = from.keys().flat_map(|p| ancestors(p.as_str())).collect();
    let t_dirs: BTreeSet<&str> = target.keys().flat_map(|p| ancestors(p.as_str())).collect();

    let mut plan = Plan::default();
    // Files to remove: tracked by F, not by T, still holding F's entry, and
    // reached through real directories only, so a directory replaced by a
    // symbolic link is never followed (code review CR-05).
    let mut to_remove = Vec::new();
    for (path, _) in &diff {
        if let (Some(file), None) = (from.get(path), target.get(path))
            && disk.holds(path.as_str(), file, alg)?
        {
            let mut real = true;
            for dir in ancestors(path.as_str()) {
                real &= disk.at(dir)? == OnDisk::Dir;
            }
            if real {
                to_remove.push(path.clone());
            }
        }
    }
    let remove: BTreeSet<&str> = to_remove.iter().map(RepoPath::as_str).collect();

    // Directories F had files under that hold no T path: deepest first.
    let mut emptied: Vec<&str> = Vec::new();
    for dir in f_dirs.iter().copied().filter(|dir| !t_dirs.contains(dir)) {
        let mut real = true;
        for at in ancestors(dir).chain(std::iter::once(dir)) {
            real &= disk.at(at)? == OnDisk::Dir;
        }
        if real {
            emptied.push(dir);
        }
    }
    emptied.sort_by_key(|dir| std::cmp::Reverse((dir.matches('/').count(), *dir)));
    plan.emptied = emptied.iter().map(|dir| (*dir).to_owned()).collect();

    // Names that a temporary name must not equal: every F and T path.
    let names: BTreeSet<&str> = from
        .keys()
        .chain(target.keys())
        .map(RepoPath::as_str)
        .collect();
    let mut blocked: Vec<String> = Vec::new();
    let mut block = |path: &RepoPath, why: String| blocked.push(format!("{path}: {why}"));
    // Every temporary name of the diff belongs to this write, and resume
    // removes it: each must be free on disk and name no F or T path, for
    // every diff path, written or not (code review CR-01). In a resume the
    // files left at these names were already removed, so an entry still
    // there is not a file and is left alone (CR-04).
    let mut taken = BTreeSet::new();
    for (path, tmp) in &diff {
        let (dir, tmp_name) = split(tmp);
        let taken_on_disk = disk.listing(dir)?.is_some_and(|entries| {
            entries
                .keys()
                .any(|name| name == tmp_name || (rules.case_insensitive && fold_eq(name, tmp_name)))
        });
        let taken_by_path = names.iter().any(|name| {
            let (name_dir, base) = split(name);
            name_dir == dir
                && (base == tmp_name || (rules.case_insensitive && fold_eq(base, tmp_name)))
        });
        if taken_on_disk || taken_by_path {
            block(path, format!("its temporary name `{tmp}` is taken"));
            taken.insert(path.clone());
        }
    }
    let mut writes = Vec::new();
    for (path, tmp) in &diff {
        let Some(file) = target.get(path) else {
            continue;
        };
        if taken.contains(path) {
            continue;
        }
        let p = path.as_str();
        if disk.holds(p, file, alg)? {
            continue; // done
        }
        let at = disk.at(p)?;
        let matches_f = match from.get(path) {
            Some(old) => disk.holds(p, old, alg)?,
            None => false,
        };
        let write = match (from.get(path), at) {
            (Some(_), _) if matches_f => true,
            (None, OnDisk::Absent) => true,
            // A user edit or deletion: kept.
            (Some(_), _) => false,
            (None, OnDisk::Dir) if disk.f_owned_dir(p, &remove, &f_dirs)? => true,
            (None, OnDisk::Dir) => {
                block(path, "a directory is where a file must be written".into());
                false
            }
            (None, _) => {
                block(
                    path,
                    "an untracked entry is where a file must be written".into(),
                );
                false
            }
        };
        if !write {
            continue;
        }
        // Each directory the file needs must be one, or be creatable.
        let mut clear = true;
        for dir in ancestors(p) {
            match disk.at(dir)? {
                OnDisk::Absent | OnDisk::Dir => {}
                OnDisk::File { .. } if remove.contains(dir) => {}
                OnDisk::File { .. } | OnDisk::Other => {
                    block(
                        path,
                        format!("an untracked entry `{dir}` is where a directory is needed"),
                    );
                    clear = false;
                    break;
                }
            }
        }
        if !clear {
            continue;
        }
        writes.push((path.clone(), tmp.clone(), *file));
    }

    // A removal whose temporary name is taken is left too, so that nothing
    // of this write ever needs that name.
    plan.remove = to_remove
        .into_iter()
        .filter(|p| !taken.contains(p))
        .collect();
    plan.blocked = blocked;
    // Hazards, from what each directory a written path passes through holds.
    let mut existing = BTreeMap::new();
    for (path, _) in &diff {
        if target.contains_key(path) {
            let p = path.as_str();
            for dir in std::iter::once("").chain(ancestors(p)) {
                if !existing.contains_key(dir) {
                    let names: Vec<String> = disk
                        .listing(dir)?
                        .map(|entries| entries.keys().cloned().collect())
                        .unwrap_or_default();
                    existing.insert(dir.to_owned(), names);
                }
            }
        }
    }
    plan.hazards = hazard::check(from, target, &existing, rules);
    let hazardous: BTreeSet<&RepoPath> = plan.hazards.iter().map(|h| &h.path).collect();
    plan.write = writes
        .into_iter()
        .filter(|(path, _, _)| !hazardous.contains(path))
        .collect();
    if resume {
        let mut skipped: Vec<String> = std::mem::take(&mut plan.blocked);
        skipped.extend(plan.hazards.drain(..).map(|h| h.to_string()));
        plan.skipped = skipped;
    }
    Ok(plan)
}

/// Removes the files at the temporary names of the F→T diff, syncing each
/// directory one was removed from. A name that is an F or T path, or holds
/// anything but a file, is left alone: the plan then skips its path (code
/// review CR-01, CR-04).
pub fn remove_temporaries(fs: &impl Fs, root: &Path, from: &Tree, target: &Tree) -> io::Result<()> {
    let mut touched = BTreeSet::new();
    let mut listings: BTreeMap<String, Option<BTreeMap<String, Kind>>> = BTreeMap::new();
    for (_, tmp) in diff(from, target) {
        let tracked = RepoPath::new(tmp.clone())
            .is_ok_and(|path| from.contains_key(&path) || target.contains_key(&path));
        if tracked {
            continue;
        }
        let (dir, name) = split(&tmp);
        if !listings.contains_key(dir) {
            let listed = match fs.scan(&on_disk(root, dir)) {
                Ok(entries) => Some(
                    entries
                        .into_iter()
                        .map(|entry| (listing_key(&entry.name), entry.kind))
                        .collect(),
                ),
                Err(error)
                    if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
            listings.insert(dir.to_owned(), listed);
        }
        let is_file = listings[dir]
            .as_ref()
            .is_some_and(|entries| matches!(entries.get(name), Some(Kind::File { .. })));
        if !is_file {
            continue;
        }
        match fs.remove_file(&on_disk(root, &tmp)) {
            Ok(()) => {
                touched.insert(dir.to_owned());
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    for dir in touched {
        fs.sync_dir(&on_disk(root, &dir))?;
    }
    Ok(())
}

/// What is at `path` now, compared with F's entry: `Ok(true)` when it still
/// holds F's bytes (or is absent, where F has none). Checked just before a
/// path is removed or replaced, so an edit saved since the plan is kept
/// (code review CR-03); the mode is not re-read.
fn unchanged(fs: &impl Fs, path: &Path, from: Option<&File>, alg: HashAlg) -> io::Result<bool> {
    match (fs.read(path), from) {
        (Ok(content), Some(file)) => Ok(object_id(alg, &blob(&content)) == file.blob),
        (Ok(_), None) => Ok(false),
        (Err(error), None) if error.kind() == ErrorKind::NotFound => Ok(true),
        (Err(error), _)
            if matches!(
                error.kind(),
                ErrorKind::NotFound | ErrorKind::IsADirectory | ErrorKind::PermissionDenied
            ) =>
        {
            Ok(false)
        }
        (Err(error), _) => Err(error),
    }
}

/// Applies `plan`: steps 1 to 5 of "Materialise". `content` gives each
/// blob's bytes. Returns a note for each path left because it changed after
/// the plan. The caller then drops the record and syncs `.praps`.
pub fn apply(
    fs: &impl Fs,
    root: &Path,
    from: &Tree,
    alg: HashAlg,
    plan: &Plan,
    content: impl Fn(&HashRef) -> io::Result<Vec<u8>>,
) -> io::Result<Vec<String>> {
    let mut notes = Vec::new();
    let changed = |path: &RepoPath| format!("{path}: it changed during the write and was kept");
    // 1. Remove the files to remove, then sync each parent touched.
    let mut parents = BTreeSet::new();
    for path in &plan.remove {
        let at = on_disk(root, path.as_str());
        if !unchanged(fs, &at, from.get(path), alg)? {
            if fs.read(&at).is_ok() {
                notes.push(changed(path));
            }
            continue;
        }
        match fs.remove_file(&at) {
            Ok(()) => fs.point("workspace.apply.path_removed"),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        parents.insert(split(path.as_str()).0);
    }
    for dir in &parents {
        fs.sync_dir(&on_disk(root, dir))?;
    }
    // 2. Remove the emptied directories, deepest first.
    for dir in &plan.emptied {
        let path = on_disk(root, dir);
        match fs.list(&path) {
            Ok(names) if names.is_empty() => {}
            Ok(_) => continue,
            Err(error)
                if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
            {
                continue;
            }
            Err(error) => return Err(error),
        }
        fs.sync_dir(&path)?;
        fs.remove_dir(&path)?;
        fs.sync_dir(&on_disk(root, split(dir).0))?;
    }
    // 3. Create the missing directories, shallowest first.
    let needed: BTreeSet<(usize, &str)> = plan
        .write
        .iter()
        .flat_map(|(path, _, _)| ancestors(path.as_str()))
        .map(|dir| (dir.matches('/').count(), dir))
        .collect();
    for (_, dir) in needed {
        let path = on_disk(root, dir);
        match fs.list(&path) {
            Ok(_) => continue,
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            // A file where a directory is needed: the plan only allows a
            // file to remove there, which step 1 removed, so another entry
            // appeared since. Nothing of the user's is touched.
            Err(error) => return Err(error),
        }
        fs.create_dir(&path)?;
        fs.sync_dir(&on_disk(root, split(dir).0))?;
    }
    // 4. Write each file through its temporary name.
    let mut touched = BTreeSet::new();
    for (path, tmp, file) in &plan.write {
        let tmp_path = on_disk(root, tmp);
        fs.create_new(&tmp_path)?;
        fs.write(&tmp_path, &content(&file.blob)?)?;
        fs.set_executable(&tmp_path, file.mode == Mode::Executable)?;
        fs.sync_file(&tmp_path)?;
        let at = on_disk(root, path.as_str());
        if !unchanged(fs, &at, from.get(path), alg)? {
            notes.push(changed(path));
            fs.remove_file(&tmp_path)?;
            touched.insert(split(path.as_str()).0);
            continue;
        }
        fs.rename(&tmp_path, &at)?;
        fs.point("workspace.apply.path_written");
        touched.insert(split(path.as_str()).0);
    }
    // 5. Sync every directory a file was renamed into.
    for dir in touched {
        fs.sync_dir(&on_disk(root, dir))?;
    }
    Ok(notes)
}
