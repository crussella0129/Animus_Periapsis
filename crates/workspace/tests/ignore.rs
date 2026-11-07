//! The ignore-rule corpus (INT-0014 AC-2): every case of
//! `tests/fixtures/ignore-corpus.txt`, whose expected results Git recorded,
//! replayed against a snapshot on the simulator.

mod common;

use std::path::Path;
use std::rc::Rc;

use codec::RepoPath;
use common::{ROOT, put};
use sim::{Mode, SimFs};
use workspace::Workspace;

const CORPUS: &str = include_str!("fixtures/ignore-corpus.txt");

#[derive(Default)]
struct Case {
    name: String,
    gitignores: Vec<(String, String)>,
    tracked: Vec<String>,
    /// (path, Git says ignored, Periapsis must track it)
    paths: Vec<(String, bool, bool)>,
}

fn parse() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    let mut lines = CORPUS.lines();
    while let Some(line) = lines.next() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (word, rest) = line.split_once(' ').unwrap();
        match word {
            "case" => cases.push(Case {
                name: rest.to_owned(),
                ..Case::default()
            }),
            "gitignore" => {
                let mut content = String::new();
                for line in lines.by_ref() {
                    if line == "end" {
                        break;
                    }
                    content.push_str(&line.replace(r"\r", "\r"));
                    content.push('\n');
                }
                cases
                    .last_mut()
                    .unwrap()
                    .gitignores
                    .push((rest.to_owned(), content));
            }
            "tracked" => cases.last_mut().unwrap().tracked.push(rest.to_owned()),
            "path" => {
                let (rest, snapshot) = rest.rsplit_once(" snapshot=").unwrap();
                let (path, git) = rest.rsplit_once(" git=").unwrap();
                cases.last_mut().unwrap().paths.push((
                    path.to_owned(),
                    git == "ignored",
                    snapshot == "tracked",
                ));
            }
            other => panic!("corpus: unknown line kind {other:?}"),
        }
    }
    cases
}

#[test]
fn ignore_rule_corpus() {
    let cases = parse();
    assert!(cases.len() >= 15, "{} cases", cases.len());
    let mut checked = 0;
    for (seed, case) in cases.iter().enumerate() {
        let fs = Rc::new(SimFs::new(seed as u64, Mode::Strict));
        let mut ws = Workspace::init(Rc::clone(&fs), Path::new(ROOT), seed as u64)
            .unwrap()
            .0;
        for path in &case.tracked {
            put(&fs, path, b"tracked\n");
        }
        ws.snapshot().unwrap();
        for (path, content) in &case.gitignores {
            put(&fs, path, content.as_bytes());
        }
        for (path, ..) in &case.paths {
            if !case.tracked.contains(path) {
                put(&fs, path, b"x\n");
            }
        }
        ws.snapshot().unwrap();
        let tree = ws.working_copy().unwrap().2;
        for (path, git_ignored, tracks) in &case.paths {
            // The two recorded columns agree with the rule: tracked before,
            // or not ignored.
            let was_tracked = case.tracked.contains(path);
            assert_eq!(
                *tracks,
                was_tracked || !git_ignored,
                "{}: {path}: corpus columns",
                case.name
            );
            let tracked = tree.contains_key(&RepoPath::new(path.clone()).unwrap());
            assert_eq!(tracked, *tracks, "case {}: {path}", case.name);
            checked += 1;
        }
    }
    println!(
        "ignore corpus: {checked} paths in {} cases agree with Git",
        cases.len()
    );
}
