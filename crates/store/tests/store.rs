//! The store on the simulator, without crashes (INT-0007 AC-3, AC-4's checker,
//! and the Consequences on names and re-syncing).

use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;
use std::path::Path;

use codec::{Operation, blob, object_id};
use hash::{HashAlg, HashRef};
use sim::{Fs, Mode, Node, Rng, SimFs};
use store::names::{file_name, parse_name};
use store::{ProblemKind, Store};

const ROOT: &str = "repo";

fn fresh() -> Store<SimFs> {
    Store::init(
        SimFs::new(1, Mode::Strict),
        Path::new(ROOT),
        HashAlg::DEFAULT,
        9,
    )
    .unwrap()
}

fn op(parents: &[&HashRef], objects: &[&HashRef]) -> Operation {
    Operation {
        parents: parents.iter().map(|id| *(*id)).collect(),
        objects: objects.iter().map(|id| *(*id)).collect(),
    }
}

/// The syscalls one store call made.
fn calls<T>(
    store: &mut Store<SimFs>,
    call: impl FnOnce(&mut Store<SimFs>) -> T,
) -> (T, Vec<String>) {
    let before = store.fs().trace().len();
    let value = call(store);
    (value, store.fs().trace()[before..].to_vec())
}

fn writes(trace: &[String]) -> Vec<&String> {
    trace
        .iter()
        .filter(|line| {
            [
                "create_dir",
                "create_new",
                "write",
                "overwrite",
                "rename",
                "remove_file",
            ]
            .iter()
            .any(|op| line.starts_with(&format!("{op} ")))
        })
        .collect()
}

#[test]
fn init_creates_and_syncs_layout() {
    let store = fresh();
    let trace = store.fs().trace();
    for line in [
        "create_dir repo",
        "create_dir repo/objects",
        "create_dir repo/ops",
        "sync_dir repo/objects",
        "sync_dir repo/ops",
        "sync_dir repo",
        "sync_dir repo/..",
    ] {
        assert!(trace.iter().any(|l| l == line), "{line:?} not in {trace:?}");
    }
    assert!(trace.last().unwrap().starts_with("sync_dir"), "{trace:?}");
    // Durable in strict mode: a crash right after init keeps the layout.
    for seed in 0..50 {
        let fs = SimFs::new(seed, Mode::Strict);
        Store::init(&fs, Path::new(ROOT), HashAlg::DEFAULT, 0).unwrap();
        let recovered = fs.recover();
        Store::open(&recovered, Path::new(ROOT), HashAlg::DEFAULT, 0).unwrap();
    }
}

#[test]
fn init_is_idempotent() {
    let fs = SimFs::new(3, Mode::Strict);
    Store::init(&fs, Path::new(ROOT), HashAlg::DEFAULT, 0).unwrap();
    let before = fs.trace().len();
    let snapshot = fs.snapshot();
    Store::init(&fs, Path::new(ROOT), HashAlg::DEFAULT, 0).unwrap();
    assert!(
        writes(&fs.trace()[before..]).is_empty(),
        "{:?}",
        &fs.trace()[before..]
    );
    assert_eq!(fs.snapshot(), snapshot);

    // A half-made layout is completed.
    let fs = SimFs::new(3, Mode::Strict);
    fs.create_dir(Path::new(ROOT)).unwrap();
    fs.create_dir(Path::new("repo/objects")).unwrap();
    Store::init(&fs, Path::new(ROOT), HashAlg::DEFAULT, 0).unwrap();
    assert_eq!(fs.snapshot().get("repo/ops"), Some(&Node::Dir));
}

/// A layout a died process left unsynced is made durable by `open`, so ops
/// acknowledged after it cannot vanish with it (code review, Sprint 1).
#[test]
fn open_syncs_the_layout() {
    for seed in 0..50 {
        let fs = SimFs::new(seed, Mode::Strict);
        for dir in [ROOT, "repo/objects", "repo/ops"] {
            fs.create_dir(Path::new(dir)).unwrap();
        }
        let mut store = Store::open(&fs, Path::new(ROOT), HashAlg::DEFAULT, 0).unwrap();
        let o = store.put_object(&blob(b"o")).unwrap();
        let acked = store.commit_op(&op(&[], &[&o])).unwrap();
        let recovered = fs.recover();
        let store = Store::open(&recovered, Path::new(ROOT), HashAlg::DEFAULT, 0).unwrap();
        assert!(store.ops().unwrap().contains_key(&acked), "seed {seed}");
    }
}

#[test]
fn open_needs_the_layout() {
    let fs = SimFs::new(3, Mode::Strict);
    fs.create_dir(Path::new(ROOT)).unwrap();
    fs.create_dir(Path::new("repo/objects")).unwrap();
    let error = Store::open(&fs, Path::new(ROOT), HashAlg::DEFAULT, 0)
        .err()
        .unwrap();
    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert!(
        error.to_string().contains("not a Periapsis store"),
        "{error}"
    );
    fs.create_dir(Path::new("repo/ops")).unwrap();
    Store::open(&fs, Path::new(ROOT), HashAlg::DEFAULT, 0).unwrap();
}

#[test]
fn objects_round_trip_by_id() {
    let mut store = fresh();
    let envelope = blob(b"content");
    let id = store.put_object(&envelope).unwrap();
    assert_eq!(id, object_id(HashAlg::DEFAULT, &envelope));
    assert_eq!(store.get_object(&id).unwrap(), envelope);
    let path = format!("repo/objects/{}", file_name(&id));
    assert_eq!(
        store.fs().snapshot().get(&path),
        Some(&Node::File(envelope))
    );
    // Not an envelope: refused.
    assert_eq!(
        store.put_object(&[0x09, 0x01]).unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
}

#[test]
fn file_names_round_trip_and_are_portable() {
    let mut rng = Rng::new(5);
    for alg in HashAlg::ALL {
        for _ in 0..100 {
            let id = alg.hash(&rng.next_u64().to_le_bytes());
            let name = file_name(&id);
            assert_eq!(parse_name(&name), Some(id), "{name}");
            assert_eq!(&name[..3], format!("{:02x}-", alg.id()));
            assert!(
                name.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b) || b == b'-'),
                "{name}"
            );
            assert!(!name.ends_with('.') && !name.ends_with(' '));
        }
    }
    // Equal digests under different algorithms get different names.
    let digest = [7u8; 32];
    assert_ne!(
        file_name(&HashRef::new(HashAlg::Blake3, &digest).unwrap()),
        file_name(&HashRef::new(HashAlg::Sha256, &digest).unwrap())
    );
    for bad in [
        "",
        "01",
        "01-",
        "1-ab",
        "zz-00",
        "01-AB",
        "03-00",
        "01-0",
        "blake3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
        "tmp-0000000000000000",
    ] {
        assert_eq!(parse_name(bad), None, "{bad:?}");
    }
}

#[test]
fn present_files_are_synced_again() {
    let mut store = fresh();
    let envelope = blob(b"twice");
    let id = store.put_object(&envelope).unwrap();
    let name = format!("repo/objects/{}", file_name(&id));
    let (_, trace) = calls(&mut store, |s| s.put_object(&envelope).unwrap());
    assert_eq!(
        trace,
        [
            format!("read {name}"),
            format!("sync_file {name}"),
            "sync_dir repo/objects".to_owned(),
        ]
    );
    // commit_op syncs what it depends on before writing the op.
    let root = store.commit_op(&op(&[], &[&id])).unwrap();
    let (_, trace) = calls(&mut store, |s| s.commit_op(&op(&[&root], &[&id])).unwrap());
    let position = |line: &str| trace.iter().position(|l| l == line).unwrap();
    let op_name = format!("repo/ops/{}", file_name(&root));
    assert!(position(&format!("sync_file {name}")) < position("sync_dir repo/objects"));
    assert!(position(&format!("sync_file {op_name}")) < position("sync_dir repo/ops"));
    let create = trace
        .iter()
        .position(|l| l.starts_with("create_new"))
        .unwrap();
    assert!(position("sync_dir repo/ops") < create, "{trace:?}");
}

/// `inspect` and `fsck` only read: every syscall they make is a `read` or a
/// `list` (test critique round 2, C-002).
#[test]
fn inspect_and_fsck_only_read() {
    let mut store = fresh();
    let o = store.put_object(&blob(b"o")).unwrap();
    store.commit_op(&op(&[], &[&o])).unwrap();
    let fs = store.fs();
    let before = fs.trace().len();
    let inspected = Store::inspect(fs, Path::new(ROOT), HashAlg::DEFAULT).unwrap();
    assert!(inspected.fsck().unwrap().is_clean());
    let trace = &fs.trace()[before..];
    assert!(!trace.is_empty());
    for line in trace {
        assert!(
            line.starts_with("read ") || line.starts_with("list "),
            "{line} is not a read"
        );
    }
    let missing = Store::inspect(fs, Path::new("nowhere"), HashAlg::DEFAULT);
    assert_eq!(missing.err().unwrap().kind(), ErrorKind::NotFound);
}

/// A damaged copy under an object's final name is replaced through a
/// temporary file and a rename (the write sequence in
/// `docs/spec/storage-durability.md`; test critique round 2, C-003).
#[test]
fn a_damaged_copy_is_replaced() {
    let mut store = fresh();
    let envelope = blob(b"to be repaired");
    let id = object_id(HashAlg::DEFAULT, &envelope);
    let path = format!("repo/objects/{}", file_name(&id));
    store.fs().create_new(Path::new(&path)).unwrap();
    store.fs().write(Path::new(&path), &envelope[..5]).unwrap();
    let (put, trace) = calls(&mut store, |s| s.put_object(&envelope).unwrap());
    assert_eq!(put, id);
    assert_eq!(store.get_object(&id).unwrap(), envelope);
    let kinds: Vec<&str> = trace.iter().map(|l| l.split(' ').next().unwrap()).collect();
    assert_eq!(
        kinds,
        [
            "read",
            "create_new",
            "write",
            "sync_file",
            "rename",
            "sync_dir"
        ],
        "{trace:?}"
    );
    assert!(
        trace[1].starts_with("create_new repo/objects/tmp-"),
        "{trace:?}"
    );
    assert_eq!(
        trace[4],
        format!(
            "rename repo/objects/{}",
            trace[1].rsplit('/').next().unwrap()
        )
    );
}

#[test]
fn leftover_tmp_does_not_block_a_put() {
    // The store's first nonce comes from Rng::new(9) (`fresh`), so plant a
    // file with exactly that temporary name.
    let mut store = fresh();
    let first = format!("repo/objects/tmp-{:016x}", Rng::new(9).next_u64());
    store.fs().create_new(Path::new(&first)).unwrap();
    store
        .fs()
        .write(Path::new(&first), b"crash debris")
        .unwrap();
    let id = store.put_object(&blob(b"after a crash")).unwrap();
    assert_eq!(store.get_object(&id).unwrap(), blob(b"after a crash"));
    assert_eq!(
        store.fs().snapshot().get(&first),
        Some(&Node::File(b"crash debris".to_vec())),
        "the leftover is left alone"
    );
}

#[test]
fn commit_with_missing_parent_writes_nothing() {
    let mut store = fresh();
    let object = store.put_object(&blob(b"o")).unwrap();
    let ghost = HashAlg::DEFAULT.hash(b"no such op");
    let before = store.fs().snapshot();
    let (result, trace) = calls(&mut store, |s| s.commit_op(&op(&[&ghost], &[&object])));
    let error = result.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert!(error.to_string().contains("parent op"), "{error}");
    assert!(writes(&trace).is_empty(), "{trace:?}");
    assert_eq!(store.fs().snapshot(), before);
}

#[test]
fn commit_with_missing_object_writes_nothing() {
    let mut store = fresh();
    let ghost = HashAlg::DEFAULT.hash(b"no such object");
    let before = store.fs().snapshot();
    let (result, trace) = calls(&mut store, |s| s.commit_op(&op(&[], &[&ghost])));
    assert!(result.unwrap_err().to_string().contains("object"));
    assert!(writes(&trace).is_empty(), "{trace:?}");
    assert_eq!(store.fs().snapshot(), before);
}

#[test]
fn heads_of_a_chain() {
    let mut store = fresh();
    assert_eq!(store.heads().unwrap(), BTreeSet::new());
    let o = store.put_object(&blob(b"o")).unwrap();
    let mut last = store.commit_op(&op(&[], &[&o])).unwrap();
    for _ in 0..4 {
        assert_eq!(store.heads().unwrap(), BTreeSet::from([last]));
        last = store.commit_op(&op(&[&last], &[&o])).unwrap();
    }
    assert_eq!(store.heads().unwrap(), BTreeSet::from([last]));
    assert_eq!(store.ops().unwrap().len(), 5);
}

#[test]
fn heads_of_a_fork_and_merge() {
    let mut store = fresh();
    let (a, b) = (
        store.put_object(&blob(b"a")).unwrap(),
        store.put_object(&blob(b"b")).unwrap(),
    );
    let root = store.commit_op(&op(&[], &[])).unwrap();
    let left = store.commit_op(&op(&[&root], &[&a])).unwrap();
    let right = store.commit_op(&op(&[&root], &[&b])).unwrap();
    assert_eq!(store.heads().unwrap(), BTreeSet::from([left, right]));
    let merge = store.commit_op(&op(&[&left, &right], &[&a, &b])).unwrap();
    assert_eq!(store.heads().unwrap(), BTreeSet::from([merge]));
}

#[test]
fn store_creates_no_other_files() {
    let mut store = fresh();
    let o = store.put_object(&blob(b"o")).unwrap();
    let root = store.commit_op(&op(&[], &[&o])).unwrap();
    store.commit_op(&op(&[&root], &[&o])).unwrap();
    store.put_object(&blob(b"o")).unwrap();
    for path in store.fs().snapshot().keys() {
        let ok = path == ROOT
            || path == "repo/objects"
            || path == "repo/ops"
            || ((path.starts_with("repo/objects/") || path.starts_with("repo/ops/"))
                && parse_name(path.rsplit('/').next().unwrap()).is_some());
        assert!(ok, "unexpected {path}");
    }
}

#[test]
fn fsck_reports_each_fault_kind() {
    let mut store = fresh();
    let kept = store.put_object(&blob(b"kept")).unwrap();
    let lost = store.put_object(&blob(b"lost")).unwrap();
    let root = store.commit_op(&op(&[], &[&kept])).unwrap();
    let child = store.commit_op(&op(&[&root], &[&lost])).unwrap();
    assert!(store.fsck().unwrap().is_clean());

    let fs = store.fs();
    // Missing object, and through it a missing reference.
    fs.remove_file(&Path::new("repo/objects").join(file_name(&lost)))
        .unwrap();
    // Flip a bit in the root op: it no longer hashes to its name, so the
    // child's parent is missing too.
    let root_path = Path::new("repo/ops").join(file_name(&root));
    let mut bytes = fs.read(&root_path).unwrap();
    bytes[3] ^= 0x10;
    fs.overwrite(&root_path, &bytes).unwrap();
    // A name that does not parse, a file that hashes right but is no
    // envelope, a pointer file at the root.
    fs.create_new(Path::new("repo/objects/zz-00")).unwrap();
    let junk = [0x02, 0x01];
    let junk_name = file_name(&object_id(HashAlg::DEFAULT, &junk));
    fs.create_new(&Path::new("repo/objects").join(&junk_name))
        .unwrap();
    fs.write(&Path::new("repo/objects").join(&junk_name), &junk)
        .unwrap();
    fs.create_new(Path::new("repo/HEAD")).unwrap();

    let report = store.fsck().unwrap();
    let found: BTreeMap<String, ProblemKind> = report
        .problems
        .iter()
        .map(|p| (p.path.clone(), p.kind.clone()))
        .collect();
    let child_path = format!("ops/{}", file_name(&child));
    assert_eq!(found.get("HEAD"), Some(&ProblemKind::Unexpected));
    assert_eq!(found.get("objects/zz-00"), Some(&ProblemKind::BadName));
    assert_eq!(
        found.get(&format!("ops/{}", file_name(&root))),
        Some(&ProblemKind::HashMismatch)
    );
    assert!(matches!(
        found.get(&format!("objects/{junk_name}")),
        Some(ProblemKind::Undecodable(_))
    ));
    let child_problems: Vec<&ProblemKind> = report
        .problems
        .iter()
        .filter(|p| p.path == child_path)
        .map(|p| &p.kind)
        .collect();
    assert_eq!(
        child_problems,
        [
            &ProblemKind::MissingParent(root),
            &ProblemKind::MissingObject(lost)
        ]
    );
    assert!(report.leftovers.is_empty());
    for problem in &report.problems {
        assert!(!problem.to_string().is_empty());
    }
}

#[test]
fn fsck_lists_tmp_files_as_leftovers() {
    let store = fresh();
    store
        .fs()
        .create_new(Path::new("repo/ops/tmp-0123456789abcdef"))
        .unwrap();
    let report = store.fsck().unwrap();
    assert!(report.is_clean(), "{:?}", report.problems);
    assert_eq!(report.leftovers, ["ops/tmp-0123456789abcdef"]);
    // On a directory that is not a store, fsck is an error, not a report.
    let fs = SimFs::new(0, Mode::Strict);
    fs.create_dir(Path::new("plain")).unwrap();
    let opened = Store::open(&fs, Path::new("plain"), HashAlg::DEFAULT, 0);
    assert!(opened.is_err());
}
