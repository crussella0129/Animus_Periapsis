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

use codec::{Mode, RepoPath};
use hash::HashAlg;
use sim::{Kind, Rng};
use workspace::Workspace;
use workspace::tree::Tree;

/// The ignore rules an edit sequence may switch on, with an oracle written
/// apart from the workspace's matcher.
pub const RULES: [&str; 2] = ["*.tmp", "d1/"];

/// Whether `path` is ignored under the active `rules` (a subset of
/// [`RULES`]), by their plain meaning.
pub fn oracle_ignored(path: &str, rules: &BTreeSet<&str>) -> bool {
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

/// What the working-copy change must hold after a snapshot: every regular
/// file that is not ignored or was tracked before, with its content, and
/// its mode under the filesystem's executable-bit rule as the spec states it
/// (`docs/spec/working-copy.md`): from the bit where there is one, else the
/// mode the working-copy change, or else its parent, last recorded.
pub fn expected_tree(
    fs: &impl Fs,
    tracked_before: &Tree,
    parent: &Tree,
    rules: &BTreeSet<&str>,
) -> BTreeMap<String, (Vec<u8>, Mode)> {
    let mut expected = BTreeMap::new();
    for (path, (content, executable)) in files_on_disk(fs) {
        let repo_path = RepoPath::new(path.clone()).unwrap();
        let before = tracked_before.get(&repo_path);
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
                .or_else(|| parent.get(&repo_path))
                .map_or(Mode::Regular, |file| file.mode)
        };
        expected.insert(path, (content, mode));
    }
    expected
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
/// some of which the rules ignore.
const NAMES: [&str; 9] = [
    "a",
    "b.tmp",
    "d1/x",
    "d1/y.tmp",
    "d2/z",
    "d2/e/f",
    "d1/deep/g",
    "c.txt",
    "d2/h.tmp",
];

/// One random edit sequence of `steps` steps, each followed by a snapshot
/// (and sometimes a `new`), checked against the oracle after every step.
/// `set_exec` changes a file's executable bit, where the filesystem has one.
/// Returns the number of snapshots checked.
pub fn edit_sequence<F: Fs>(
    ws: &mut Workspace<F>,
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
                let text: String = rules.iter().map(|r| format!("{r}\n")).collect();
                put(fs, ".gitignore", text.as_bytes());
            }
            8 => {
                ws.new_change().unwrap();
            }
            _ => put(fs, name, b""),
        }
        let tracked_before = ws.working_copy().unwrap().2;
        let parent = ws.parent_tree().unwrap();
        ws.snapshot().unwrap();
        let expected = expected_tree(ws.store().fs(), &tracked_before, &parent, &rules);
        let recorded = recorded_tree(ws);
        assert_eq!(recorded, expected, "seed {seed} step {step}: lost edit");
        checked += 1;
    }
    checked
}
