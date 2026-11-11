//! Helpers shared by the workspace tests.
#![allow(dead_code)]

use std::io::ErrorKind;
use std::path::Path;

use sim::Fs;

pub const ROOT: &str = "repo";

/// Writes `content` to the file at `path` under the repository, creating
/// its directories and the file as needed.
pub fn put(fs: &impl Fs, path: &str, content: &[u8]) {
    let mut dir = Path::new(ROOT).to_path_buf();
    let mut parts: Vec<&str> = path.split('/').collect();
    let name = parts.pop().unwrap();
    for part in parts {
        dir = dir.join(part);
        match fs.create_dir(&dir) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
    let file = dir.join(name);
    match fs.create_new(&file) {
        Ok(()) => fs.write(&file, content).unwrap(),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            fs.overwrite(&file, content).unwrap();
        }
        Err(error) => panic!("{}: {error}", file.display()),
    }
}

/// Removes the file at `path` under the repository.
pub fn delete(fs: &impl Fs, path: &str) {
    let file = path
        .split('/')
        .fold(Path::new(ROOT).to_path_buf(), |p, c| p.join(c));
    fs.remove_file(&file).unwrap();
}

/// The syscalls that write, in a trace.
pub fn writes(trace: &[String]) -> Vec<&String> {
    trace
        .iter()
        .filter(|line| {
            [
                "create_dir ",
                "create_new ",
                "write ",
                "overwrite ",
                "rename ",
                "remove_file ",
                "remove_dir ",
                "set_executable ",
                "sync_file ",
                "sync_dir ",
            ]
            .iter()
            .any(|op| line.starts_with(op))
        })
        .collect()
}

// ------------------------------------------------- the edit-corpus oracle

use std::collections::{BTreeMap, BTreeSet};

use codec::{Changeset, Mode};
use hash::{HashAlg, HashRef};
use sim::{Kind, Rng};
use workspace::{Error, Workspace};

/// The ignore rules an edit sequence may switch on, with an oracle written
/// apart from the workspace's matcher. Two live in the top `.gitignore`;
/// two (written `d2:…`) live in `d2/.gitignore`: a nested whitelist that
/// re-includes `d2/h.tmp` against the top `*.tmp`, and a nested directory
/// rule that excludes `d2/e/`. A file holding no active rule is deleted.
pub const RULES: [&str; 4] = ["*.tmp", "d1/", "d2:!h.tmp", "d2:e/"];

/// Whether `path` is ignored under the active `rules`, by Git's meaning:
/// the deeper `d2/.gitignore` decides first, an excluded directory cannot be
/// re-included, then the top file.
pub fn oracle_ignored(path: &str, rules: &BTreeSet<&str>) -> bool {
    if path.starts_with("d2/") {
        if rules.contains("d2:e/") && path.starts_with("d2/e/") {
            return true;
        }
        if rules.contains("d2:!h.tmp") && path == "d2/h.tmp" {
            return false;
        }
    }
    (rules.contains("*.tmp") && path.rsplit('/').next().unwrap().ends_with(".tmp"))
        || (rules.contains("d1/") && path.starts_with("d1/"))
}

/// Every regular file under the repository (outside `.praps`), with its
/// content and executable bit, read through `Fs`.
pub fn files_on_disk(fs: &impl Fs) -> BTreeMap<String, (Vec<u8>, bool)> {
    let mut files = BTreeMap::new();
    let mut stack = vec![String::new()];
    while let Some(dir) = stack.pop() {
        let fs_dir = dir
            .split('/')
            .filter(|c| !c.is_empty())
            .fold(Path::new(ROOT).to_path_buf(), |p, c| p.join(c));
        for entry in fs.scan(&fs_dir).unwrap() {
            let name = entry.name.as_utf8().unwrap().to_owned();
            if dir.is_empty() && (name == ".praps" || name == ".git") {
                continue;
            }
            let path = if dir.is_empty() {
                name.clone()
            } else {
                format!("{dir}/{name}")
            };
            match entry.kind {
                Kind::Dir => stack.push(path),
                Kind::File { executable } => {
                    files.insert(path, (fs.read(&fs_dir.join(&name)).unwrap(), executable));
                }
                Kind::Symlink | Kind::Other => {}
            }
        }
    }
    files
}

/// Files with their contents and modes, by path.
pub type Files = BTreeMap<String, (Vec<u8>, Mode)>;

/// The rules that a working copy's `.gitignore` files hold.
pub fn rules_of(files: &Files) -> BTreeSet<&'static str> {
    let lines = |path: &str| -> Vec<String> {
        files
            .get(path)
            .map(|(content, _)| {
                String::from_utf8_lossy(content)
                    .lines()
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    let (top, d2) = (lines(".gitignore"), lines("d2/.gitignore"));
    RULES
        .into_iter()
        .filter(|rule| match rule.strip_prefix("d2:") {
            Some(line) => d2.iter().any(|l| l == line),
            None => top.iter().any(|l| l == rule),
        })
        .collect()
}

/// One op as the model predicts it.
#[derive(Clone, Debug)]
pub struct ModelOp {
    /// The working-copy change's files.
    pub files: Files,
    /// The modes its parent change recorded (for the no-bit mode rule).
    pub parent_modes: BTreeMap<String, Mode>,
    /// The ignore rules its `.gitignore` files hold.
    pub rules: BTreeSet<&'static str>,
    /// Its parent op, by position.
    pub parent: Option<usize>,
    /// Its id, learned from `log` by position once the counts agree.
    pub id: Option<HashRef>,
}

/// The oracle's own record of what the repository tracks, kept apart from
/// the workspace: the working-copy change's paths and modes, its parent's,
/// and its own op history. It never reads the implementation's state; op
/// ids alone are taken from `log`, by position, after the op count agrees.
#[derive(Clone, Debug)]
pub struct Model {
    pub tracked: BTreeMap<String, Mode>,
    pub parent: BTreeMap<String, Mode>,
    pub ops: Vec<ModelOp>,
    pub head: usize,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            tracked: BTreeMap::new(),
            parent: BTreeMap::new(),
            ops: vec![ModelOp {
                files: Files::new(),
                parent_modes: BTreeMap::new(),
                rules: BTreeSet::new(),
                parent: None,
                id: None,
            }],
            head: 0,
        }
    }
}

impl Model {
    fn push(&mut self, files: Files, rules: BTreeSet<&'static str>) {
        self.ops.push(ModelOp {
            files,
            parent_modes: self.parent.clone(),
            rules,
            parent: Some(self.head),
            id: None,
        });
        self.head = self.ops.len() - 1;
    }

    /// Moves to op `k`'s state, as `undo` or `restore` does: a new op over
    /// the head with that op's working copy.
    pub fn goto(&mut self, k: usize) {
        let target = self.ops[k].clone();
        self.tracked = target
            .files
            .iter()
            .map(|(p, (_, m))| (p.clone(), *m))
            .collect();
        self.parent = target.parent_modes.clone();
        self.push(target.files, target.rules);
    }

    /// Whether writing op `k`'s files over the head's would be refused: a
    /// path T writes that the head does not track, where the disk holds
    /// another file (an ignored, untracked one). The names an edit sequence
    /// uses never make a file and a directory share a path.
    pub fn blocks(&self, k: usize, fs: &impl Fs) -> bool {
        let from = &self.ops[self.head].files;
        let disk = files_on_disk(fs);
        self.ops[k].files.iter().any(|(path, (content, mode))| {
            if from.contains_key(path) {
                return false;
            }
            match disk.get(path) {
                None => false,
                Some((on_disk, executable)) => {
                    on_disk != content
                        || (fs.has_executable_bit() && *executable != (*mode == Mode::Executable))
                }
            }
        })
    }

    /// What a snapshot must record, by the spec (`docs/spec/working-copy.md`):
    /// every regular file not ignored or tracked before, with its content;
    /// its mode from the bit where there is one, else the mode the
    /// working-copy change, or else its parent, last recorded.
    pub fn snapshot(
        &mut self,
        fs: &impl Fs,
        rules: &BTreeSet<&str>,
    ) -> BTreeMap<String, (Vec<u8>, Mode)> {
        let mut expected = BTreeMap::new();
        for (path, (content, executable)) in files_on_disk(fs) {
            let before = self.tracked.get(&path).copied();
            if oracle_ignored(&path, rules) && before.is_none() {
                continue;
            }
            let mode = if fs.has_executable_bit() {
                if executable {
                    Mode::Executable
                } else {
                    Mode::Regular
                }
            } else {
                before
                    .or_else(|| self.parent.get(&path).copied())
                    .unwrap_or(Mode::Regular)
            };
            expected.insert(path, (content, mode));
        }
        self.tracked = expected.iter().map(|(p, (_, m))| (p.clone(), *m)).collect();
        if expected != self.ops[self.head].files {
            let rules = rules_of(&expected);
            self.push(expected.clone(), rules);
        }
        expected
    }

    /// `new`: the change just snapshotted becomes the parent, in a new op.
    pub fn new_change(&mut self) {
        self.parent = self.tracked.clone();
        let head = self.ops[self.head].clone();
        self.push(head.files, head.rules);
    }
}

/// The working-copy change's tree with each blob's content.
pub fn recorded_tree<F: Fs>(ws: &Workspace<F>) -> BTreeMap<String, (Vec<u8>, Mode)> {
    let tree = ws.working_copy().unwrap().2;
    tree.iter()
        .map(|(path, file)| {
            let envelope = ws.store().get_object(&file.blob).unwrap();
            assert_eq!(codec::object_id(HashAlg::DEFAULT, &envelope), file.blob);
            (
                path.as_str().to_owned(),
                (envelope[2..].to_vec(), file.mode),
            )
        })
        .collect()
}

/// The paths an edit sequence works on: a few names in a few directories,
/// some of which the rules ignore or re-include.
const NAMES: [&str; 11] = [
    "a",
    "b.tmp",
    "d1/x",
    "d1/y.tmp",
    "d2/z",
    "d2/e/f",
    "d1/deep/g",
    "c.txt",
    "d2/h.tmp",
    "d2/e/k.tmp",
    "d2/i.tmp",
];

/// Writes the two `.gitignore` files for the active rules, deleting one that
/// holds no rule.
fn write_rules(fs: &impl Fs, rules: &BTreeSet<&str>) {
    let on_disk = files_on_disk(fs);
    for (file, prefix) in [(".gitignore", ""), ("d2/.gitignore", "d2:")] {
        let lines: String = rules
            .iter()
            .filter(|r| {
                if prefix.is_empty() {
                    !r.contains(':')
                } else {
                    r.starts_with(prefix)
                }
            })
            .map(|r| format!("{}\n", r.trim_start_matches(prefix)))
            .collect();
        if lines.is_empty() {
            if on_disk.contains_key(file) {
                delete(fs, file);
            }
        } else {
            put(fs, file, lines.as_bytes());
        }
    }
}

/// The files of the tree a working-copy changeset id names, read through
/// the store and `workspace::tree`.
pub fn tree_files<F: Fs>(ws: &Workspace<F>, id: &HashRef) -> Files {
    let mut chain = Vec::new();
    let mut next = Some(*id);
    while let Some(current) = next {
        let changeset =
            Changeset::from_envelope(&ws.store().get_object(&current).unwrap()).unwrap();
        next = changeset.parents.first().copied();
        chain.push(changeset);
    }
    let mut tree = workspace::tree::Tree::new();
    for changeset in chain.iter().rev() {
        tree = workspace::tree::of(changeset, &tree);
    }
    tree.iter()
        .map(|(path, file)| {
            let envelope = ws.store().get_object(&file.blob).unwrap();
            (
                path.as_str().to_owned(),
                (envelope[2..].to_vec(), file.mode),
            )
        })
        .collect()
}

/// Checks `log` against the model's op list: the count first, then each op
/// by position (the history is linear, so newest first is creation order
/// reversed), learning each op's id. With `trees`, also each op's files.
fn check_ops<F: Fs>(ws: &mut Workspace<F>, model: &mut Model, trees: bool, at: &str) {
    let log = ws.ops().unwrap();
    assert_eq!(log.len(), model.ops.len(), "{at}: op count");
    for (i, entry) in log.iter().rev().enumerate() {
        let parent = model.ops[i]
            .parent
            .map(|p| model.ops[p].id.unwrap_or(entry.id));
        assert_eq!(
            entry.parents.first().copied(),
            parent,
            "{at}: op {i}'s parent"
        );
        if let Some(id) = model.ops[i].id {
            assert_eq!(id, entry.id, "{at}: op {i} changed");
        }
        model.ops[i].id = Some(entry.id);
        if trees {
            assert_eq!(
                tree_files(ws, &entry.working_copy),
                model.ops[i].files,
                "{at}: op {i}'s files"
            );
            assert_eq!(rules_of(&model.ops[i].files), model.ops[i].rules);
        }
    }
}

/// After a successful `undo` or `restore` to model op `k`: every file op `k`
/// tracks is on disk with its content (and mode, where there is a bit), and
/// every file the head before it tracked and `k` does not is gone.
fn check_written(fs: &impl Fs, from: &Files, target: &Files, at: &str) {
    let disk = files_on_disk(fs);
    for (path, (content, mode)) in target {
        let (on_disk, executable) = disk
            .get(path)
            .unwrap_or_else(|| panic!("{at}: {path} was not written"));
        assert_eq!(on_disk, content, "{at}: {path}");
        if fs.has_executable_bit() {
            assert_eq!(
                *executable,
                *mode == Mode::Executable,
                "{at}: {path}'s mode"
            );
        }
    }
    for path in from.keys().filter(|p| !target.contains_key(*p)) {
        assert!(!disk.contains_key(path), "{at}: {path} was not removed");
    }
}

/// One random edit sequence of `steps` steps, each followed by a snapshot
/// (and sometimes a `new`, an `undo` or a `restore` of a random earlier
/// op), checked after every step against [`Model`], which starts from
/// `model` and is never fed the workspace's own state. `set_exec` changes a
/// file's executable bit, where the filesystem has one. Returns what it
/// checked.
pub fn edit_sequence<F: Fs>(
    ws: &mut Workspace<F>,
    mut model: Model,
    seed: u64,
    steps: usize,
    set_exec: &dyn Fn(&F, &str, bool),
) -> Counts {
    let mut rng = Rng::new(seed);
    let mut rules: BTreeSet<&str> = rules_of(&model.ops[model.head].files);
    let mut counts = Counts::default();
    check_ops(ws, &mut model, false, &format!("seed {seed} start"));
    for step in 0..steps {
        let at = format!("seed {seed} step {step}");
        let fs = ws.store().fs();
        let name = NAMES[rng.below(NAMES.len() as u64) as usize];
        let exists = files_on_disk(fs).contains_key(name);
        // An earlier op to restore, chosen before any snapshot adds one.
        let target = rng.below(model.ops.len() as u64) as usize;
        let mut new = false;
        let mut history = None;
        match rng.below(12) {
            0..=3 => put(
                fs,
                name,
                format!("{seed}:{step}:{}", rng.next_u64()).as_bytes(),
            ),
            4 | 5 if exists => delete(fs, name),
            6 if exists && fs.has_executable_bit() => set_exec(fs, name, rng.coin()),
            7 => {
                let rule = RULES[rng.below(RULES.len() as u64) as usize];
                if !rules.remove(rule) {
                    rules.insert(rule);
                }
                write_rules(fs, &rules);
            }
            8 => new = true,
            9 => history = Some(None),
            10 => history = Some(Some(target)),
            _ => put(fs, name, b""),
        }
        if new {
            // `new` snapshots, then starts a change that holds the same
            // tree: the model snapshots, and the tree must not change.
            let expected = model.snapshot(ws.store().fs(), &rules);
            ws.new_change().unwrap();
            model.new_change();
            assert_eq!(recorded_tree(ws), expected, "{at}: new");
        }
        if let Some(restore) = history {
            // Both snapshot first. `undo` goes to the parent of the head
            // before that snapshot; `restore` to the chosen op.
            let h = model.head;
            let id = model.ops[target].id.expect("ids are learned every step");
            model.snapshot(ws.store().fs(), &rules);
            let before = files_on_disk(ws.store().fs());
            let goal = match restore {
                None => model.ops[h].parent,
                Some(k) => Some(k),
            };
            let result = match restore {
                None => ws.undo().map(|_| ()),
                Some(_) => ws.restore(&id.to_string()).map(|_| ()),
            };
            match goal {
                None => {
                    assert!(
                        matches!(result, Err(Error::NothingToUndo)),
                        "{at}: {result:?}"
                    );
                    counts.refused += 1;
                }
                Some(k) if model.blocks(k, ws.store().fs()) => {
                    assert!(matches!(result, Err(Error::Blocked(_))), "{at}: {result:?}");
                    assert_eq!(
                        files_on_disk(ws.store().fs()),
                        before,
                        "{at}: refused, unchanged"
                    );
                    counts.refused += 1;
                    counts.blocked += 1;
                }
                Some(k) => {
                    result.unwrap_or_else(|error| panic!("{at}: {error}"));
                    if restore.is_some() {
                        counts.restores += 1;
                    } else {
                        counts.undos += 1;
                    }
                    let from = model.ops[model.head].files.clone();
                    model.goto(k);
                    rules = model.ops[k].rules.clone();
                    check_written(ws.store().fs(), &from, &model.ops[k].files, &at);
                }
            }
        }
        ws.snapshot().unwrap();
        let expected = model.snapshot(ws.store().fs(), &rules);
        let recorded = recorded_tree(ws);
        assert_eq!(recorded, expected, "{at}: lost edit");
        check_ops(ws, &mut model, false, &at);
        counts.snapshots += 1;
    }
    check_ops(ws, &mut model, true, &format!("seed {seed} end"));
    counts.ops += model.ops.len();
    counts
}

/// What edit sequences checked.
#[derive(Clone, Copy, Debug, Default)]
pub struct Counts {
    /// Steps, each ending with a checked snapshot.
    pub snapshots: usize,
    /// Successful `undo`s and `restore`s, each checked file by file.
    pub undos: usize,
    pub restores: usize,
    /// Refusals asserted: nothing to undo, or blocked.
    pub refused: usize,
    /// Of those, refusals because an entry was in the way.
    pub blocked: usize,
    /// Ops whose files were checked against `log`.
    pub ops: usize,
}

impl std::ops::AddAssign for Counts {
    fn add_assign(&mut self, other: Counts) {
        self.snapshots += other.snapshots;
        self.undos += other.undos;
        self.restores += other.restores;
        self.refused += other.refused;
        self.blocked += other.blocked;
        self.ops += other.ops;
    }
}

impl std::fmt::Display for Counts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} snapshots, {} undos, {} restores, {} refusals ({} blocked), {} ops checked, 0 lost edits",
            self.snapshots, self.undos, self.restores, self.refused, self.blocked, self.ops
        )
    }
}
