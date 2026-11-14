//! A thousand simulated processes in one run (Sprint 7 T-008; INT-0005).
//!
//! In a test binary of its own: its 3,001 handoffs between threads take tens
//! of seconds on GitHub's Windows runners under the suite's load, and while
//! it shared a process with the scheduler's other tests, one of their
//! 60-second deadlock guards ran out (Sprint 7 Test, pull request run
//! 43628379803).

use std::path::Path;
use std::sync::Arc;

use sim::{Fs, Mode, Outcome, Processes, SimFs};

/// A thousand processes, three syscalls each, all end done: a boundary
/// wakes only the process drawn to run next (Sprint 7 T-008).
#[test]
fn a_thousand_processes_run() {
    let fs = Arc::new(SimFs::new(1, Mode::Strict));
    fs.create_dir(Path::new("d")).unwrap();
    let closures: Vec<Box<dyn FnOnce() -> usize + Send>> = (0..1_000)
        .map(|me| {
            let fs = Arc::clone(&fs);
            Box::new(move || {
                fs.list(Path::new("d")).unwrap();
                fs.create_new(&Path::new("d").join(format!("f{me}")))
                    .unwrap();
                fs.list(Path::new("d")).unwrap().len()
            }) as Box<dyn FnOnce() -> usize + Send>
        })
        .collect();
    let Ok(outcomes) = Processes::new(7).run(&fs, closures) else {
        panic!("the run stopped early");
    };
    assert!(outcomes.iter().all(|o| matches!(o, Outcome::Done(_))));
    assert_eq!(fs.list(Path::new("d")).unwrap().len(), 1_000);
}
