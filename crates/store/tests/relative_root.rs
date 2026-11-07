//! `init` and `open` with a relative root on the real filesystem, whose
//! parent path is empty (code review, Sprint 1). This file holds one test, so
//! changing the process's current directory affects nothing else.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use codec::blob;
use hash::HashAlg;
use sim::{RealFs, Rng};
use store::Store;

#[test]
fn relative_roots_work_on_the_real_filesystem() {
    let mut rng = Rng::new(u64::from(std::process::id()) ^ 0x7265_6c61);
    let dir = loop {
        let dir = std::env::temp_dir().join(format!("store-relative-{:016x}", rng.next_u64()));
        match fs::create_dir(&dir) {
            Ok(()) => break dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    };
    std::env::set_current_dir(&dir).unwrap();
    let mut store = Store::init(RealFs, Path::new("repo"), HashAlg::DEFAULT, 0).unwrap();
    let id = store.put_object(&blob(b"relative")).unwrap();
    Store::open(RealFs, Path::new("repo"), HashAlg::DEFAULT, 0).unwrap();
    std::env::set_current_dir(dir.join("repo")).unwrap();
    let store = Store::init(RealFs, Path::new("."), HashAlg::DEFAULT, 0).unwrap();
    assert_eq!(store.get_object(&id).unwrap(), blob(b"relative"));
    assert!(store.fsck().unwrap().is_clean());
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    let _ = fs::remove_dir_all(&dir);
}
