//! `SimFs` shared across threads (Sprint 4 T-002, INT-0005).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use sim::{Fs, Mode, SimFs};

#[test]
fn simfs_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SimFs>();
    assert_send_sync::<Arc<SimFs>>();

    let fs = Arc::new(SimFs::new(1, Mode::Strict));
    fs.create_dir(Path::new("d")).unwrap();
    let handles: Vec<_> = (0..2)
        .map(|i| {
            let fs = Arc::clone(&fs);
            thread::spawn(move || {
                let path = Path::new("d").join(format!("f{i}"));
                fs.create_new(&path).unwrap();
                fs.write(&path, &[i]).unwrap();
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(fs.list(Path::new("d")).unwrap(), ["f0", "f1"]);
    assert_eq!(fs.syscalls(), 6);
}

#[test]
fn poisoned_state_is_recovered() {
    let fs = Arc::new(SimFs::new(2, Mode::Lenient));
    fs.create_dir(Path::new("d")).unwrap();

    // A panic with the state locked.
    let held = Arc::clone(&fs);
    let result = thread::spawn(move || held.panic_holding_the_lock()).join();
    assert!(result.is_err());
    fs.create_new(Path::new("d/a")).unwrap();

    // A panic inside a hook, which runs with no lock held.
    fs.set_hook(Some(Box::new(|_| panic!("a deliberate panic in a hook"))));
    let unwound = catch_unwind(AssertUnwindSafe(|| fs.create_new(Path::new("d/b"))));
    assert!(unwound.is_err());
    fs.set_hook(None);
    fs.create_new(Path::new("d/c")).unwrap();

    assert_eq!(fs.list(Path::new("d")).unwrap(), ["a", "c"]);
}

/// A hook that unwinds goes back in its slot, so the next syscall runs it
/// again (code review CR-06).
#[test]
fn a_hook_that_unwinds_is_put_back() {
    use std::sync::atomic::{AtomicU32, Ordering};
    let fs = SimFs::new(3, Mode::Strict);
    let runs = Arc::new(AtomicU32::new(0));
    let count = Arc::clone(&runs);
    fs.set_hook(Some(Box::new(move |_| {
        if count.fetch_add(1, Ordering::SeqCst) == 0 {
            panic!("a deliberate panic in a hook");
        }
    })));
    let unwound = catch_unwind(AssertUnwindSafe(|| fs.create_dir(Path::new("a"))));
    assert!(unwound.is_err());
    fs.create_dir(Path::new("b")).unwrap();
    assert_eq!(runs.load(Ordering::SeqCst), 2);
    fs.set_hook(None);
}
