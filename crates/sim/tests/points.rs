//! Named points (Sprint 4 T-003, INT-0005 AC-1, AC-2 and AC-6).

use std::io;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use sim::{Fs, Mode, RealFs, SimFs};

/// Every syscall kind once, with a point before, between and after them
/// when `points` is set.
fn every_kind(fs: &SimFs, points: bool) -> io::Result<()> {
    let mark = |name| {
        if points {
            fs.point(name);
        }
    };
    let (d, a, b, e) = (
        Path::new("d"),
        Path::new("d/a"),
        Path::new("d/b"),
        Path::new("d/e"),
    );
    mark("test.points.start");
    fs.create_dir(d)?;
    mark("test.points.created");
    fs.create_new(a)?;
    fs.write(a, b"one")?;
    mark("test.points.written");
    fs.overwrite(a, b"two")?;
    fs.set_executable(a, true)?;
    fs.sync_file(a)?;
    mark("test.points.synced");
    fs.rename(a, b)?;
    mark("test.points.renamed");
    fs.sync_dir(d)?;
    fs.read(b)?;
    fs.list(d)?;
    fs.scan(d)?;
    mark("test.points.read");
    fs.create_dir(e)?;
    fs.sync_dir(d)?;
    fs.remove_dir(e)?;
    fs.remove_file(b)?;
    mark("test.points.removed");
    fs.sync_dir(d)?;
    mark("test.points.end");
    Ok(())
}

const EVERY_KIND: u64 = 16;

#[test]
fn points_are_not_syscalls() {
    let plain = SimFs::new(0, Mode::Strict);
    every_kind(&plain, false).unwrap();
    let marked = SimFs::new(0, Mode::Strict);
    every_kind(&marked, true).unwrap();
    assert_eq!(plain.syscalls(), EVERY_KIND);
    assert_eq!(marked.syscalls(), EVERY_KIND);
    assert_eq!(plain.trace(), marked.trace());
    for mode in [Mode::Lenient, Mode::Strict] {
        for k in 1..=EVERY_KIND {
            for seed in 0..10 {
                let run = |points| {
                    let fs = SimFs::new(seed, mode);
                    fs.set_crash_at(Some(k));
                    let error = every_kind(&fs, points).unwrap_err();
                    assert!(sim::is_crash(&error));
                    (fs.syscalls(), fs.trace(), fs.recover().snapshot())
                };
                assert_eq!(run(false), run(true), "{mode:?} k {k} seed {seed}");
            }
        }
    }
}

#[test]
fn points_do_not_change_traces() {
    for mode in [Mode::Lenient, Mode::Strict] {
        for seed in 0..40 {
            let plain = SimFs::new(seed, mode);
            every_kind(&plain, false).unwrap();
            let marked = SimFs::new(seed, mode);
            every_kind(&marked, true).unwrap();
            assert_eq!(plain.trace(), marked.trace());
            assert_eq!(plain.snapshot(), marked.snapshot());
            assert_eq!(plain.recover().snapshot(), marked.recover().snapshot());
        }
    }
}

#[test]
fn reach_counts_points() {
    let fs = SimFs::new(1, Mode::Strict);
    assert!(fs.reach().is_empty());
    fs.point("b.x.y");
    fs.point("a.x.y");
    fs.point("b.x.y");
    assert_eq!(fs.reach(), [("a.x.y", 1), ("b.x.y", 2)]);
}

#[test]
fn on_point_actions_run_in_order() {
    let fs = SimFs::new(1, Mode::Strict);
    let log = Arc::new(Mutex::new(Vec::new()));
    for tag in ["first", "second"] {
        let log = Arc::clone(&log);
        fs.on_point("a.x.y", move || log.lock().unwrap().push(tag));
    }
    let other = Arc::clone(&log);
    fs.on_point("b.x.y", move || other.lock().unwrap().push("other"));
    fs.point("a.x.y");
    assert_eq!(*log.lock().unwrap(), ["first", "second"]);
    fs.point("a.x.y");
    assert_eq!(*log.lock().unwrap(), ["first", "second", "first", "second"]);
    fs.clear_points();
    fs.point("a.x.y");
    fs.point("b.x.y");
    assert_eq!(log.lock().unwrap().len(), 4);
}

#[test]
fn on_point_action_is_not_reentered() {
    let fs = Arc::new(SimFs::new(1, Mode::Strict));
    let runs = Arc::new(AtomicU32::new(0));
    let (inner, count) = (Arc::clone(&fs), Arc::clone(&runs));
    fs.on_point("a.x.y", move || {
        count.fetch_add(1, Ordering::SeqCst);
        // Reaching the same point inside its action counts it, but does not
        // run the action again.
        inner.point("a.x.y");
        inner.create_dir(Path::new("made")).unwrap();
    });
    fs.point("a.x.y");
    assert_eq!(runs.load(Ordering::SeqCst), 1);
    assert_eq!(fs.reach(), [("a.x.y", 2)]);
    // The action's syscall is an ordinary one.
    assert_eq!(fs.trace(), ["create_dir made"]);
    fs.clear_points();
    let (inner, count) = (Arc::clone(&fs), Arc::clone(&runs));
    fs.on_point("a.x.y", move || {
        count.fetch_add(1, Ordering::SeqCst);
        inner.point("a.x.y");
    });
    // Once it has ended, the next arrival runs it again.
    fs.point("a.x.y");
    assert_eq!(runs.load(Ordering::SeqCst), 2);
    // Dropping the actions breaks the cycle the action's `Arc` makes.
    fs.clear_points();
}

#[test]
fn points_count_but_do_not_act_after_a_crash() {
    let fs = SimFs::new(1, Mode::Strict);
    let runs = Arc::new(AtomicU32::new(0));
    let count = Arc::clone(&runs);
    fs.on_point("a.x.y", move || {
        count.fetch_add(1, Ordering::SeqCst);
    });
    fs.set_crash_at(Some(1));
    assert!(fs.create_dir(Path::new("d")).is_err());
    assert!(fs.crashed());
    fs.point("a.x.y");
    assert_eq!(runs.load(Ordering::SeqCst), 0);
    assert_eq!(fs.reach(), [("a.x.y", 1)]);
}

#[test]
fn recover_drops_actions_and_reach() {
    let fs = SimFs::new(1, Mode::Strict);
    let runs = Arc::new(AtomicU32::new(0));
    let count = Arc::clone(&runs);
    fs.on_point("a.x.y", move || {
        count.fetch_add(1, Ordering::SeqCst);
    });
    fs.point("a.x.y");
    let restarted = fs.recover();
    assert!(restarted.reach().is_empty());
    restarted.point("a.x.y");
    assert_eq!(runs.load(Ordering::SeqCst), 1);
    assert_eq!(restarted.reach(), [("a.x.y", 1)]);
    fs.clear_points();
}

#[test]
fn points_reach_through_every_forwarder() {
    fn mark(fs: impl Fs) {
        fs.point("a.x.y");
    }
    let fs = Arc::new(SimFs::new(1, Mode::Strict));
    mark(&*fs);
    mark(Arc::clone(&fs));
    // The `&T` forwarder over an `Arc`.
    let arc = Arc::clone(&fs);
    let by_ref: &Arc<SimFs> = &arc;
    mark(by_ref);
    let rc = Rc::new(SimFs::new(1, Mode::Strict));
    mark(Rc::clone(&rc));
    mark(&*rc);
    assert_eq!(fs.reach(), [("a.x.y", 3)]);
    assert_eq!(rc.reach(), [("a.x.y", 2)]);
}

#[test]
fn real_fs_point_is_a_no_op() {
    fn mark(fs: impl Fs) {
        fs.point("a.x.y");
    }
    let dir = std::env::temp_dir().join(format!("praps-point-{}", std::process::id()));
    RealFs.create_dir(&dir).unwrap();
    RealFs.point("a.x.y");
    mark(RealFs);
    let by_ref: &RealFs = &RealFs;
    mark(by_ref);
    assert!(RealFs.list(&dir).unwrap().is_empty());
    RealFs.remove_dir(&dir).unwrap();
}

#[test]
fn reach_record_round_trips() {
    let dir = std::env::temp_dir().join(format!("praps-reach-{}", std::process::id()));
    let mut totals = sim::reach::Totals::new();
    sim::reach::add(&mut totals, &[("b.x.y", 2), ("a.x.y", 1)]);
    sim::reach::add(&mut totals, &[("b.x.y", 3)]);
    sim::reach::record(&dir, "scenario-file", "a scenario", 4, &totals).unwrap();
    // Written again: the temporary name is replaced, the file renamed over.
    sim::reach::record(&dir, "scenario-file", "a scenario", 4, &totals).unwrap();
    let text = RealFs.read(&dir.join("scenario-file.txt")).unwrap();
    assert_eq!(text, b"scenario a scenario\nsprint 4\na.x.y 1\nb.x.y 5\n");
    assert_eq!(RealFs.list(&dir).unwrap(), ["scenario-file.txt"]);
    RealFs.remove_file(&dir.join("scenario-file.txt")).unwrap();
    RealFs.remove_dir(&dir).unwrap();
}

/// `Fs::point` has no default body. With one, a forwarder or a wrapper that
/// left `point` out would still compile, and would drop every point marked
/// through it (Sprint 6 T-002).
#[test]
fn point_has_no_default_body() {
    let source = RealFs
        .read(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("fs.rs"),
        )
        .unwrap();
    let source = String::from_utf8(source).unwrap();
    // The trait's own lines: after its opening line, up to the first line
    // that is a closing brace alone.
    let in_trait: Vec<&str> = source
        .lines()
        .skip_while(|line| *line != "pub trait Fs {")
        .skip(1)
        .take_while(|line| *line != "}")
        .map(str::trim)
        .collect();
    assert!(!in_trait.is_empty(), "the trait was not found");
    let declarations: Vec<&str> = in_trait
        .iter()
        .copied()
        .filter(|line| line.starts_with("fn point("))
        .collect();
    assert_eq!(declarations, ["fn point(&self, name: &'static str);"]);
    // Every method of the trait is a declaration only.
    let methods: Vec<&str> = in_trait
        .iter()
        .copied()
        .filter(|line| line.starts_with("fn "))
        .collect();
    assert_eq!(methods.len(), 16, "{methods:#?}");
    assert!(
        methods.iter().all(|line| line.ends_with(';')),
        "{methods:#?}"
    );
}
