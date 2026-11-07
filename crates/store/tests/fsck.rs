//! `fsck` over a corruption corpus (INT-0007 AC-4: 100% detection, 0 false
//! reports).

mod common;

use std::path::{Path, PathBuf};

use codec::{Operation, blob};
use common::matrix::{ROOT, run_matrix};
use hash::{HashAlg, HashRef};
use sim::{Fs, Mode, SimFs};
use store::names::file_name;
use store::{ProblemKind, Store, Variant};

/// A healthy store: two objects, a root op, a child and a merge.
struct Built {
    store: Store<SimFs>,
    objects: Vec<HashRef>,
    ops: Vec<HashRef>,
}

fn build() -> Built {
    let mut store = Store::init(
        SimFs::new(0, Mode::Strict),
        Path::new(ROOT),
        HashAlg::DEFAULT,
        1,
    )
    .unwrap();
    let a = store.put_object(&blob(b"abc")).unwrap();
    let b = store.put_object(&blob(b"second object")).unwrap();
    let op = |parents: &[HashRef], objects: &[HashRef]| Operation {
        parents: parents.iter().copied().collect(),
        objects: objects.iter().copied().collect(),
    };
    let root = store.commit_op(&op(&[], &[a])).unwrap();
    let child = store.commit_op(&op(&[root], &[b])).unwrap();
    let merge = store.commit_op(&op(&[root, child], &[a, b])).unwrap();
    assert!(store.fsck().unwrap().is_clean());
    Built {
        store,
        objects: vec![a, b],
        ops: vec![root, child, merge],
    }
}

fn object_path(id: &HashRef) -> PathBuf {
    Path::new(ROOT).join("objects").join(file_name(id))
}

fn op_path(id: &HashRef) -> PathBuf {
    Path::new(ROOT).join("ops").join(file_name(id))
}

/// The paths `fsck` reports, relative to the store.
fn reported(store: &Store<SimFs>) -> Vec<String> {
    store
        .fsck()
        .unwrap()
        .problems
        .into_iter()
        .map(|p| p.path)
        .collect()
}

#[test]
fn fsck_detects_every_truncation() {
    let mut cases = 0;
    for i in 0..3 {
        let len = {
            let built = build();
            built
                .store
                .fs()
                .read(&op_path(&built.ops[i]))
                .unwrap()
                .len()
        };
        for cut in 0..len {
            let built = build();
            let path = op_path(&built.ops[i]);
            let bytes = built.store.fs().read(&path).unwrap();
            built.store.fs().overwrite(&path, &bytes[..cut]).unwrap();
            let name = format!("ops/{}", file_name(&built.ops[i]));
            assert!(
                reported(&built.store).contains(&name),
                "op {i} cut to {cut} bytes not reported"
            );
            cases += 1;
        }
    }
    println!("truncations detected: {cases} of {cases}");
}

#[test]
fn fsck_detects_every_bit_flip() {
    let mut cases = 0;
    let len = build()
        .store
        .fs()
        .read(&object_path(&build().objects[0]))
        .unwrap()
        .len();
    for byte in 0..len {
        for bit in 0..8 {
            let built = build();
            let path = object_path(&built.objects[0]);
            let mut bytes = built.store.fs().read(&path).unwrap();
            bytes[byte] ^= 1 << bit;
            built.store.fs().overwrite(&path, &bytes).unwrap();
            let report = built.store.fsck().unwrap();
            let name = format!("objects/{}", file_name(&built.objects[0]));
            assert!(
                report
                    .problems
                    .iter()
                    .any(|p| p.path == name && p.kind == ProblemKind::HashMismatch),
                "flip of bit {bit} in byte {byte} not reported"
            );
            cases += 1;
        }
    }
    println!("bit flips detected: {cases} of {cases}");
}

#[test]
fn fsck_detects_missing_references() {
    // A missing object: the ops naming it report it.
    let built = build();
    built
        .store
        .fs()
        .remove_file(&object_path(&built.objects[1]))
        .unwrap();
    let kinds: Vec<ProblemKind> = built
        .store
        .fsck()
        .unwrap()
        .problems
        .into_iter()
        .map(|p| p.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![ProblemKind::MissingObject(built.objects[1]); 2],
        "the child and the merge name it"
    );

    // A missing parent op.
    let built = build();
    built
        .store
        .fs()
        .remove_file(&op_path(&built.ops[1]))
        .unwrap();
    let report = built.store.fsck().unwrap();
    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    assert_eq!(
        report.problems[0].path,
        format!("ops/{}", file_name(&built.ops[2]))
    );
    assert_eq!(
        report.problems[0].kind,
        ProblemKind::MissingParent(built.ops[1])
    );
}

#[test]
fn fsck_detects_unparsable_names() {
    for bad in [
        "blake3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
        "zz-00",
        "01-ABCD",
    ] {
        let built = build();
        let from = object_path(&built.objects[0]);
        let to = Path::new(ROOT).join("objects").join(bad);
        built.store.fs().rename(&from, &to).unwrap();
        let report = built.store.fsck().unwrap();
        let bad_path = format!("objects/{bad}");
        assert!(
            report
                .problems
                .iter()
                .any(|p| p.path == bad_path && p.kind == ProblemKind::BadName),
            "{bad}: {:?}",
            report.problems
        );
    }
}

#[test]
fn fsck_is_clean_on_matrix_stores() {
    let (mut opened, mut false_reports) = (0u64, Vec::new());
    run_matrix(
        Variant::Correct,
        |scenario, mode, seed, crash_at, _, recovered| {
            // A store whose init was not acknowledged may not exist; every one
            // that opens must check clean.
            if let Ok(store) = Store::open(recovered, Path::new(ROOT), HashAlg::DEFAULT, 0) {
                opened += 1;
                let report = store.fsck().unwrap();
                if !report.is_clean() {
                    false_reports.push((scenario, mode, seed, crash_at, report.problems));
                }
            }
        },
    );
    println!(
        "fsck on {opened} recovered stores: {} reports",
        false_reports.len()
    );
    assert!(opened > 0);
    assert!(false_reports.is_empty(), "{:?}", &false_reports[..1]);
}
