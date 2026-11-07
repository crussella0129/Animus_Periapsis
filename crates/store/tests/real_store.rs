//! The store on the real filesystem (INT-0004 AC-1, AC-5; INT-0007).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use codec::{Operation, View, blob};
use hash::{HashAlg, HashRef};
use sim::{RealFs, Rng};
use store::Store;

fn claim(label: &str) -> PathBuf {
    let mut rng = Rng::new(u64::from(std::process::id()) ^ 0x7265_616c);
    loop {
        let dir = std::env::temp_dir().join(format!("store-{label}-{:016x}", rng.next_u64()));
        match fs::create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

#[test]
fn real_store_reopens_clean() {
    let dir = claim("reopen");
    let root = dir.join("repo");
    let (objects, heads) = {
        let mut store = Store::init(RealFs, &root, HashAlg::DEFAULT, 7).unwrap();
        let a = store.put_object(&blob(b"first")).unwrap();
        let b = store.put_object(&blob(b"second")).unwrap();
        let mut commit = |parents: BTreeSet<HashRef>, heads: BTreeSet<HashRef>| {
            let view = View {
                heads,
                workspaces: Default::default(),
            };
            let view = store.put_object(&view.to_envelope()).unwrap();
            store.commit_op(&Operation { parents, view }).unwrap()
        };
        let base = commit(BTreeSet::new(), [a].into());
        let left = commit([base].into(), [b].into());
        let right = commit([base].into(), [a, b].into());
        assert_eq!(store.heads().unwrap(), BTreeSet::from([left, right]));
        (
            [(a, blob(b"first")), (b, blob(b"second"))],
            store.heads().unwrap(),
        )
    };

    let store = Store::open(RealFs, &root, HashAlg::DEFAULT, 8).unwrap();
    assert_eq!(store.heads().unwrap(), heads);
    for (id, envelope) in &objects {
        assert_eq!(&store.get_object(id).unwrap(), envelope);
    }
    let report = store.fsck().unwrap();
    assert!(report.is_clean(), "{:?}", report.problems);
    assert!(report.leftovers.is_empty());
    // init on the existing store changes nothing.
    Store::init(RealFs, &root, HashAlg::DEFAULT, 9).unwrap();
    assert_eq!(
        Store::open(RealFs, &root, HashAlg::DEFAULT, 0)
            .unwrap()
            .heads()
            .unwrap(),
        heads
    );
    let _ = fs::remove_dir_all(&dir);
}
