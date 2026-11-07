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

use codec::Mode;
use hash::HashAlg;
use sim::{Kind, Rng};
use workspace::Workspace;

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

/// The oracle's own record of what the repository tracks, kept apart from
/// the workspace: the working-copy change's paths and modes, and its
/// parent's. It never reads the implementation's state.
#[derive(Clone, Debug, Default)]
pub struct Model {
    pub tracked: BTreeMap<String, Mode>,
    pub parent: BTreeMap<String, Mode>,
}

impl Model {
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
        expected
    }

    /// `new`: the change just snapshotted becomes the parent.
    pub fn new_change(&mut self) {
        self.parent = self.tracked.clone();
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

/// One random edit sequence of `steps` steps, each followed by a snapshot
/// (and sometimes a `new`), checked after every step against [`Model`],
/// which starts from `model` and is never fed the workspace's own state.
/// `set_exec` changes a file's executable bit, where the filesystem has one.
/// Returns the number of snapshots checked.
pub fn edit_sequence<F: Fs>(
    ws: &mut Workspace<F>,
    mut model: Model,
    seed: u64,
    steps: usize,
    set_exec: &dyn Fn(&F, &str, bool),
) -> usize {
    let mut rng = Rng::new(seed);
    let mut rules: BTreeSet<&str> = BTreeSet::new();
    let mut checked = 0;
    for step in 0..steps {
        let fs = ws.store().fs();
        let name = NAMES[rng.below(NAMES.len() as u64) as usize];
        let exists = files_on_disk(fs).contains_key(name);
        let mut new = false;
        match rng.below(10) {
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
            _ => put(fs, name, b""),
        }
        if new {
            // `new` snapshots, then starts a change that holds the same
            // tree: the model snapshots, and the tree must not change.
            let expected = model.snapshot(ws.store().fs(), &rules);
            ws.new_change().unwrap();
            model.new_change();
            assert_eq!(recorded_tree(ws), expected, "seed {seed} step {step}: new");
        }
        ws.snapshot().unwrap();
        let expected = model.snapshot(ws.store().fs(), &rules);
        let recorded = recorded_tree(ws);
        assert_eq!(recorded, expected, "seed {seed} step {step}: lost edit");
        checked += 1;
    }
    checked
}
