//! Four OS processes committing ops to one store on the real filesystem
//! (Sprint 4 T-007; INT-0007 AC-5; INT-0004 AC-1). The kernel decides the
//! interleavings, so this is evidence, not a reproducible run.
//!
//! The parent test re-runs this test binary four times as children. A child
//! writes only when both its exact test name was passed and
//! `PERI_WRITER_CHILD` is set; `cargo ci` scrubs that variable, so a value
//! left in a shell cannot turn the no-op child test into a writer.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use codec::{
    Changeset, Entry, Mode as FileMode, Operation, RepoPath, View, WorkspaceEntry, blob, object_id,
};
use hash::{HashAlg, HashRef};
use sim::{RealFs, Rng};
use store::Store;
use store::names::{file_name, parse_name};

const CHILD: &str = "PERI_WRITER_CHILD";
const CHILD_TEST: &str = "real_writer_child";
const CHILDREN: usize = 4;
const OPS: usize = 25;
const ALG: HashAlg = HashAlg::DEFAULT;

fn claim(label: &str) -> PathBuf {
    let mut rng = Rng::new(u64::from(std::process::id()) ^ 0x7772_6974);
    loop {
        let dir = env::temp_dir().join(format!("store-{label}-{:016x}", rng.next_u64()));
        match fs::create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

/// Puts a state of its own (a blob, a changeset, a view) and returns the
/// view.
fn state(store: &mut Store<RealFs>, tag: &str) -> HashRef {
    let blob_id = store.put_object(&blob(tag.as_bytes())).unwrap();
    let changeset = Changeset {
        parents: Vec::new(),
        change_id: *ALG.hash(tag.as_bytes()).digest().first_chunk().unwrap(),
        changes: [(
            RepoPath::new("f").unwrap(),
            Entry::Put {
                blob: blob_id,
                mode: FileMode::Regular,
            },
        )]
        .into(),
        description: tag.as_bytes().to_vec(),
    };
    let changeset = store.put_object(&changeset.to_envelope()).unwrap();
    store
        .put_object(
            &View {
                workspaces: [(
                    "default".to_owned(),
                    WorkspaceEntry {
                        working_copy: [changeset].into(),
                        heads: [changeset].into(),
                    },
                )]
                .into(),
            }
            .to_envelope(),
        )
        .unwrap()
}

/// A child: commits `OPS` ops of its own content, each on its previous one
/// (the first on the base), and prints each acknowledged op with its
/// parents.
#[test]
fn real_writer_child() {
    let exact = env::args().any(|arg| arg == CHILD_TEST);
    let Some(spec) = env::var_os(CHILD) else {
        return;
    };
    if !exact {
        return;
    }
    let spec = spec.to_string_lossy().into_owned();
    let mut parts = spec.split('|');
    let root = PathBuf::from(parts.next().unwrap());
    let child: usize = parts.next().unwrap().parse().unwrap();
    let base = parse_name(parts.next().unwrap()).unwrap();
    let mut store = Store::open(RealFs, &root, ALG, 1_000 + child as u64).unwrap();
    let mut previous = base;
    for i in 0..OPS {
        let view = state(&mut store, &format!("child {child} op {i}"));
        let op = Operation {
            parents: [previous].into(),
            workspace: "default".to_owned(),
            view,
        };
        let id = store.commit_op(&op).unwrap();
        println!("ACK {} {}", file_name(&id), file_name(&previous));
        previous = id;
    }
}

#[test]
fn real_four_processes_lose_nothing() {
    if env::var_os(CHILD).is_some() {
        // A child run must not start children of its own.
        return;
    }
    let dir = claim("writers");
    let root = dir.join("repo");
    let base = {
        let mut store = Store::init(RealFs, &root, ALG, 7).unwrap();
        let view = state(&mut store, "base");
        store
            .commit_op(&Operation {
                parents: BTreeSet::new(),
                workspace: "default".to_owned(),
                view,
            })
            .unwrap()
    };
    let exe = env::current_exe().unwrap();
    let children: Vec<_> = (0..CHILDREN)
        .map(|child| {
            Command::new(&exe)
                .args([CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"])
                .env(
                    CHILD,
                    format!("{}|{child}|{}", root.display(), file_name(&base)),
                )
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    // All four run at once; only then is each waited for.
    let children: Vec<_> = children
        .into_iter()
        .map(|child| child.wait_with_output())
        .collect();
    let mut reported: BTreeMap<HashRef, HashRef> = BTreeMap::new();
    for (child, output) in children.into_iter().enumerate() {
        let output = output.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "child {child} failed: {stdout}{}",
            String::from_utf8_lossy(&output.stderr)
        );
        // libtest may print the first one after the test's name.
        let acks: Vec<&str> = stdout
            .lines()
            .filter_map(|line| line.find("ACK ").map(|at| &line[at..]))
            .collect();
        assert_eq!(acks.len(), OPS, "child {child}: {stdout}");
        for line in acks {
            let mut words = line.split(' ').skip(1);
            let id = parse_name(words.next().unwrap()).unwrap();
            let parent = parse_name(words.next().unwrap()).unwrap();
            reported.insert(id, parent);
        }
    }
    assert_eq!(reported.len(), CHILDREN * OPS);

    let store = Store::open(RealFs, &root, ALG, 9).unwrap();
    let ops = store.ops().unwrap();
    for (id, parent) in &reported {
        let op = ops
            .get(id)
            .unwrap_or_else(|| panic!("op {} lost", file_name(id)));
        assert_eq!(op.parents, BTreeSet::from([*parent]));
        assert_eq!(object_id(ALG, &op.to_envelope()), *id);
    }
    assert_eq!(ops.len(), CHILDREN * OPS + 1);
    let report = store.fsck().unwrap();
    assert!(report.is_clean(), "{:?}", report.problems);
    // The heads from the children's reports: every reported op no reported
    // op names. Each child's chain ends in one head.
    let named: BTreeSet<&HashRef> = reported.values().collect();
    let expected: BTreeSet<HashRef> = reported
        .keys()
        .filter(|id| !named.contains(id))
        .copied()
        .collect();
    assert_eq!(expected.len(), CHILDREN);
    assert_eq!(store.heads().unwrap(), expected);
    drop(store);
    fs::remove_dir_all(&dir).unwrap();
}
