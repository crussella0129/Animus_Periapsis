//! `sim::Processes`, the deterministic scheduler (Sprint 4 T-005, INT-0005
//! AC-1; the means for INT-0007 AC-5).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sim::{After, Fs, KillAt, Mode, Node, Outcome, Processes, SimFs, Stopped};

type Run<'a, T> = Vec<Box<dyn FnOnce() -> T + Send + 'a>>;

/// Three processes, each writing two files of its own and reading the
/// others' directory, with a point between.
fn writers(fs: &Arc<SimFs>) -> Run<'_, Vec<String>> {
    (0..3)
        .map(|p| {
            let fs = Arc::clone(fs);
            Box::new(move || {
                let dir = PathBuf::from(format!("p{p}"));
                fs.create_dir(&dir).unwrap();
                for i in 0..2 {
                    let path = dir.join(format!("f{i}"));
                    fs.create_new(&path).unwrap();
                    fs.write(&path, format!("{p}{i}").as_bytes()).unwrap();
                    fs.point("test.writers.written");
                    fs.sync_file(&path).unwrap();
                }
                fs.list(Path::new("")).unwrap()
            }) as Box<dyn FnOnce() -> Vec<String> + Send>
        })
        .collect()
}

fn values<T: Clone>(outcomes: &[Outcome<T>]) -> Vec<T> {
    outcomes
        .iter()
        .map(|o| match o {
            Outcome::Done(v) => v.clone(),
            Outcome::Killed => panic!("killed"),
        })
        .collect()
}

#[test]
fn processes_run_one_at_a_time() {
    let fs = Arc::new(SimFs::new(1, Mode::Strict));
    let inside = Arc::new(AtomicU32::new(0));
    let overlap = Arc::new(AtomicBool::new(false));
    let run: Run<'_, ()> = (0..4)
        .map(|p| {
            let (fs, inside, overlap) =
                (Arc::clone(&fs), Arc::clone(&inside), Arc::clone(&overlap));
            Box::new(move || {
                // Before the first boundary too: a thread starts only with
                // the baton.
                for i in 0..5 {
                    if inside.fetch_add(1, Ordering::SeqCst) != 0 {
                        overlap.store(true, Ordering::SeqCst);
                    }
                    std::thread::sleep(Duration::from_millis(1));
                    inside.fetch_sub(1, Ordering::SeqCst);
                    fs.create_dir(Path::new(&format!("p{p}-{i}"))).unwrap();
                }
            }) as Box<dyn FnOnce() + Send>
        })
        .collect();
    Processes::new(7).run(&fs, run).unwrap();
    assert!(!overlap.load(Ordering::SeqCst));
    assert_eq!(fs.syscalls(), 20);
}

#[test]
fn processes_return_results_in_input_order() {
    let fs = Arc::new(SimFs::new(1, Mode::Strict));
    let run: Run<'_, usize> = (0..4usize)
        .map(|p| {
            let fs = Arc::clone(&fs);
            Box::new(move || {
                fs.create_dir(Path::new(&format!("p{p}"))).unwrap();
                fs.point("test.order.made");
                p * 10
            }) as Box<dyn FnOnce() -> usize + Send>
        })
        .collect();
    let outcomes = Processes::new(3)
        .kill(
            2,
            KillAt::Point {
                name: "test.order.made",
                nth: 1,
            },
        )
        .run(&fs, run)
        .unwrap();
    assert_eq!(
        outcomes,
        [
            Outcome::Done(0),
            Outcome::Done(10),
            Outcome::Killed,
            Outcome::Done(30)
        ]
    );
}

#[test]
fn processes_runs_are_deterministic() {
    for seed in 0..20 {
        let once = || {
            let fs = Arc::new(SimFs::new(seed, Mode::Strict));
            let outcomes = Processes::new(seed).run(&fs, writers(&fs)).unwrap();
            (fs.trace(), fs.snapshot(), values(&outcomes))
        };
        assert_eq!(once(), once(), "seed {seed}");
    }
}

#[test]
fn processes_seeds_vary_interleavings() {
    let mut traces = std::collections::BTreeSet::new();
    for seed in 0..20 {
        let fs = Arc::new(SimFs::new(0, Mode::Strict));
        Processes::new(seed).run(&fs, writers(&fs)).unwrap();
        traces.insert(fs.trace());
    }
    assert!(traces.len() >= 2, "{} distinct interleavings", traces.len());
}

#[test]
fn orderings_hold_back_a_point() {
    for seed in 0..20 {
        let fs = Arc::new(SimFs::new(0, Mode::Strict));
        let log = Arc::new(Mutex::new(Vec::new()));
        let run: Run<'_, ()> = (0..2)
            .map(|p| {
                let (fs, log) = (Arc::clone(&fs), Arc::clone(&log));
                Box::new(move || {
                    fs.create_dir(Path::new(&format!("p{p}"))).unwrap();
                    let name = if p == 0 {
                        "test.order.a"
                    } else {
                        "test.order.b"
                    };
                    fs.point(name);
                    // What process 1 sees as it goes past B: whether A has
                    // been reached. An ordering orders arrivals, not the
                    // code after them.
                    if p == 1 {
                        let reached_a = fs.reach().iter().any(|(n, _)| *n == "test.order.a");
                        log.lock().unwrap().push(reached_a);
                    }
                    fs.create_dir(Path::new(&format!("q{p}"))).unwrap();
                }) as Box<dyn FnOnce() + Send>
            })
            .collect();
        let after_a = After::Point {
            process: 0,
            name: "test.order.a",
            nth: 1,
        };
        Processes::new(seed)
            .order(after_a, 1, "test.order.b", 1)
            .run(&fs, run)
            .unwrap();
        assert_eq!(*log.lock().unwrap(), [true], "seed {seed}");
    }
}

#[test]
fn orderings_wait_on_a_process_end() {
    for seed in 0..10 {
        let fs = Arc::new(SimFs::new(0, Mode::Strict));
        let run: Run<'_, ()> = (0..2)
            .map(|p| {
                let fs = Arc::clone(&fs);
                Box::new(move || {
                    fs.point("test.order.start");
                    for i in 0..3 {
                        fs.create_dir(Path::new(&format!("p{p}-{i}"))).unwrap();
                    }
                }) as Box<dyn FnOnce() + Send>
            })
            .collect();
        Processes::new(seed)
            .order(After::End(0), 1, "test.order.start", 1)
            .run(&fs, run)
            .unwrap();
        let trace = fs.trace();
        assert!(
            trace[..3].iter().all(|l| l.contains("p0-")),
            "seed {seed}: {trace:?}"
        );
    }
}

#[test]
fn ordering_deadlock_is_reported() {
    let fs = Arc::new(SimFs::new(0, Mode::Strict));
    let run: Run<'_, ()> = (0..2)
        .map(|p| {
            let fs = Arc::clone(&fs);
            Box::new(move || {
                fs.point(if p == 0 { "test.dead.a" } else { "test.dead.b" });
                fs.create_dir(Path::new(&format!("p{p}"))).unwrap();
            }) as Box<dyn FnOnce() + Send>
        })
        .collect();
    let stopped = Processes::new(1)
        .order(
            After::Point {
                process: 1,
                name: "test.dead.x",
                nth: 1,
            },
            0,
            "test.dead.a",
            1,
        )
        .order(
            After::Point {
                process: 0,
                name: "test.dead.y",
                nth: 1,
            },
            1,
            "test.dead.b",
            1,
        )
        .run(&fs, run)
        .unwrap_err();
    match stopped {
        Stopped::Deadlock(waiting) => {
            assert_eq!(waiting, ["0: test.dead.a", "1: test.dead.b"]);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(fs.syscalls(), 0);
}

#[test]
fn orderings_are_released_by_a_crash() {
    // Each process waits for a point the other never reaches: without the
    // crash this is a deadlock.
    for seed in 0..10 {
        let fs = Arc::new(SimFs::new(seed, Mode::Strict));
        fs.set_crash_at(Some(1));
        let run: Run<'_, bool> = (0..2)
            .map(|p| {
                let fs = Arc::clone(&fs);
                Box::new(move || {
                    if p == 1 {
                        assert!(fs.create_dir(Path::new("p1")).is_err());
                    }
                    fs.point(if p == 0 {
                        "test.crash.held0"
                    } else {
                        "test.crash.held1"
                    });
                    fs.create_dir(Path::new(&format!("q{p}"))).is_err()
                }) as Box<dyn FnOnce() -> bool + Send>
            })
            .collect();
        let outcomes = Processes::new(seed)
            .order(
                After::Point {
                    process: 1,
                    name: "test.crash.unreached",
                    nth: 1,
                },
                0,
                "test.crash.held0",
                1,
            )
            .order(
                After::Point {
                    process: 0,
                    name: "test.crash.unreached",
                    nth: 1,
                },
                1,
                "test.crash.held1",
                1,
            )
            .run(&fs, run)
            .unwrap();
        assert!(fs.crashed());
        assert_eq!(
            outcomes,
            [Outcome::Done(true), Outcome::Done(true)],
            "seed {seed}"
        );
    }
}

#[test]
fn orderings_are_released_when_the_source_process_ends() {
    for (seed, how) in (0..2).enumerate() {
        let fs = Arc::new(SimFs::new(0, Mode::Strict));
        let run: Run<'_, ()> = (0..2)
            .map(|p| {
                let fs = Arc::clone(&fs);
                Box::new(move || {
                    if p == 0 {
                        fs.point("test.end.held");
                        fs.create_dir(Path::new("p0")).unwrap();
                    } else {
                        fs.create_dir(Path::new("p1")).unwrap();
                        fs.point("test.end.dies");
                    }
                }) as Box<dyn FnOnce() + Send>
            })
            .collect();
        let mut setup = Processes::new(seed as u64).order(
            After::Point {
                process: 1,
                name: "test.end.unreached",
                nth: 1,
            },
            0,
            "test.end.held",
            1,
        );
        if how == 1 {
            setup = setup.kill(
                1,
                KillAt::Point {
                    name: "test.end.dies",
                    nth: 1,
                },
            );
        }
        let outcomes = setup.run(&fs, run).unwrap();
        assert_eq!(outcomes[0], Outcome::Done(()), "how {how}");
        assert!(fs.snapshot().contains_key("p0"));
    }
}

#[test]
fn killed_process_state_stays_visible() {
    let fs = Arc::new(SimFs::new(5, Mode::Strict));
    fs.create_dir(Path::new("d")).unwrap();
    fs.sync_dir(Path::new("")).unwrap();
    let seen = Arc::new(Mutex::new(None));
    let run: Run<'_, ()> = (0..2)
        .map(|p| {
            let (fs, seen) = (Arc::clone(&fs), Arc::clone(&seen));
            Box::new(move || {
                if p == 0 {
                    fs.create_new(Path::new("d/a")).unwrap();
                    fs.write(Path::new("d/a"), b"unsynced").unwrap();
                    fs.create_dir(Path::new("never")).unwrap();
                } else {
                    fs.point("test.kill.read");
                    *seen.lock().unwrap() = Some(fs.read(Path::new("d/a")).unwrap());
                }
            }) as Box<dyn FnOnce() + Send>
        })
        .collect();
    let before = fs.syscalls();
    let outcomes = Processes::new(2)
        // Process 0's third boundary is the syscall that would create
        // `never`: it stops there.
        .kill(0, KillAt::Step(3))
        .order(After::End(0), 1, "test.kill.read", 1)
        .run(&fs, run)
        .unwrap();
    assert_eq!(outcomes, [Outcome::Killed, Outcome::Done(())]);
    assert_eq!(seen.lock().unwrap().as_deref(), Some(&b"unsynced"[..]));
    assert!(!fs.snapshot().contains_key("never"));
    assert_eq!(fs.syscalls() - before, 3);
}

#[test]
fn kill_at_a_point() {
    let fs = Arc::new(SimFs::new(5, Mode::Strict));
    let run: Run<'_, ()> = vec![Box::new({
        let fs = Arc::clone(&fs);
        move || {
            for i in 0..3 {
                fs.create_dir(Path::new(&format!("d{i}"))).unwrap();
                fs.point("test.kill.made");
            }
        }
    })];
    let outcomes = Processes::new(1)
        .kill(
            0,
            KillAt::Point {
                name: "test.kill.made",
                nth: 2,
            },
        )
        .run(&fs, run)
        .unwrap();
    assert_eq!(outcomes, [Outcome::Killed]);
    assert_eq!(fs.trace(), ["create_dir d0", "create_dir d1"]);
    assert_eq!(fs.reach(), [("test.kill.made", 2)]);
}

#[test]
fn kill_then_crash_settles_by_seed() {
    let once = |seed| {
        let fs = Arc::new(SimFs::new(seed, Mode::Strict));
        fs.create_dir(Path::new("d")).unwrap();
        fs.sync_dir(Path::new("")).unwrap();
        let run: Run<'_, ()> = (0..2)
            .map(|p| {
                let fs = Arc::clone(&fs);
                Box::new(move || {
                    let path = PathBuf::from(format!("d/f{p}"));
                    // The crash may fail any of these.
                    if fs.create_new(&path).is_err() || fs.write(&path, b"pending").is_err() {
                        return;
                    }
                    fs.point("test.settle.written");
                    let _ = fs.create_dir(Path::new(&format!("x{p}")));
                }) as Box<dyn FnOnce() + Send>
            })
            .collect();
        fs.set_crash_at(Some(fs.syscalls() + 5));
        let _ = Processes::new(seed)
            .kill(
                0,
                KillAt::Point {
                    name: "test.settle.written",
                    nth: 1,
                },
            )
            .run(&fs, run)
            .unwrap();
        fs.recover().snapshot()
    };
    let mut outcomes = std::collections::BTreeSet::new();
    for seed in 0..20 {
        let a = once(seed);
        assert_eq!(a, once(seed), "seed {seed}");
        outcomes.insert(a.get("d/f0").cloned());
    }
    // Kept, dropped or torn: more than one outcome over the seeds.
    assert!(outcomes.len() > 1, "{outcomes:?}");
    assert!(
        outcomes
            .iter()
            .flatten()
            .all(|n| matches!(n, Node::File(_)))
    );
}

#[test]
fn crash_in_interleaved_run_stops_every_process() {
    for seed in 0..10 {
        let fs = Arc::new(SimFs::new(seed, Mode::Strict));
        fs.set_crash_at(Some(4));
        let outcomes = Processes::new(seed)
            .run(&fs, {
                (0..3)
                    .map(|p| {
                        let fs = Arc::clone(&fs);
                        Box::new(move || {
                            let mut failed = 0u32;
                            for i in 0..4 {
                                if fs.create_dir(Path::new(&format!("p{p}-{i}"))).is_err() {
                                    failed += 1;
                                }
                            }
                            failed
                        }) as Box<dyn FnOnce() -> u32 + Send>
                    })
                    .collect()
            })
            .unwrap();
        assert_eq!(fs.trace().len(), 4);
        assert_eq!(values(&outcomes).iter().sum::<u32>(), 12 - 3, "seed {seed}");
    }
}

#[test]
fn point_events_are_ordered() {
    // At one point: the count, then a kill, then the ordering wait, then the
    // actions.
    let fs = Arc::new(SimFs::new(0, Mode::Strict));
    let log = Arc::new(Mutex::new(Vec::new()));
    let action_log = Arc::clone(&log);
    let action_fs = Arc::clone(&fs);
    fs.on_point("test.events.p", move || {
        let count = action_fs
            .reach()
            .iter()
            .find(|(n, _)| *n == "test.events.p")
            .map(|(_, c)| *c);
        action_log
            .lock()
            .unwrap()
            .push(format!("action, count {count:?}"));
    });
    let run: Run<'_, ()> = (0..2)
        .map(|p| {
            let (fs, log) = (Arc::clone(&fs), Arc::clone(&log));
            Box::new(move || {
                if p == 0 {
                    fs.point("test.events.p");
                    log.lock().unwrap().push("0 past p".to_owned());
                } else {
                    fs.point("test.events.p");
                    log.lock().unwrap().push("1 past p".to_owned());
                    fs.create_dir(Path::new("x")).unwrap();
                    fs.point("test.events.q");
                }
            }) as Box<dyn FnOnce() + Send>
        })
        .collect();
    let outcomes = Processes::new(4)
        // Process 1 dies on arrival at p: no wait, no action.
        .kill(
            1,
            KillAt::Point {
                name: "test.events.p",
                nth: 1,
            },
        )
        // Process 0 waits at p until process 1 has ended.
        .order(After::End(1), 0, "test.events.p", 1)
        .run(&fs, run)
        .unwrap();
    fs.clear_points();
    assert_eq!(outcomes, [Outcome::Done(()), Outcome::Killed]);
    // Both arrivals were counted before process 0's action ran; process 1's
    // action never ran, and process 0's ran only after its wait.
    assert_eq!(*log.lock().unwrap(), ["action, count Some(2)", "0 past p"]);
}

#[test]
fn panic_in_a_process_ends_the_run() {
    let fs = Arc::new(SimFs::new(0, Mode::Strict));
    let start = Instant::now();
    let run: Run<'_, ()> = (0..3)
        .map(|p| {
            let fs = Arc::clone(&fs);
            Box::new(move || {
                for i in 0..50 {
                    fs.create_dir(Path::new(&format!("p{p}-{i}"))).unwrap();
                    if p == 1 && i == 3 {
                        panic!("process 1 fails");
                    }
                }
            }) as Box<dyn FnOnce() + Send>
        })
        .collect();
    let stopped = Processes::new(9).run(&fs, run).unwrap_err();
    match stopped {
        Stopped::Panicked { process, payload } => {
            assert_eq!(process, 1);
            assert_eq!(payload.downcast_ref::<&str>(), Some(&"process 1 fails"));
        }
        other => panic!("{other:?}"),
    }
    assert!(fs.syscalls() < 150);
    assert!(start.elapsed() < Duration::from_secs(30));
}

/// A setup that could never act as written is refused before any thread
/// starts (code review CR-02).
#[test]
fn invalid_setups_are_refused() {
    let point = |process, nth| After::Point {
        process,
        name: "test.invalid.p",
        nth,
    };
    type Setup = Box<dyn Fn() -> Processes>;
    let setups: Vec<(&str, Setup)> = vec![
        (
            "kill index",
            Box::new(|| Processes::new(0).kill(2, KillAt::Step(1))),
        ),
        (
            "kill step 0",
            Box::new(|| Processes::new(0).kill(0, KillAt::Step(0))),
        ),
        (
            "kill arrival 0",
            Box::new(|| {
                Processes::new(0).kill(
                    0,
                    KillAt::Point {
                        name: "test.invalid.p",
                        nth: 0,
                    },
                )
            }),
        ),
        (
            "ordering index",
            Box::new(move || Processes::new(0).order(point(0, 1), 5, "test.invalid.p", 1)),
        ),
        (
            "ordering arrival 0",
            Box::new(move || Processes::new(0).order(point(0, 1), 1, "test.invalid.p", 0)),
        ),
        (
            "predecessor index",
            Box::new(move || Processes::new(0).order(point(9, 1), 1, "test.invalid.p", 1)),
        ),
        (
            "predecessor arrival 0",
            Box::new(move || Processes::new(0).order(point(0, 0), 1, "test.invalid.p", 1)),
        ),
        (
            "end index",
            Box::new(|| Processes::new(0).order(After::End(7), 1, "test.invalid.p", 1)),
        ),
    ];
    for (what, setup) in setups {
        let fs = Arc::new(SimFs::new(0, Mode::Strict));
        let started = Arc::new(AtomicBool::new(false));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let run: Run<'_, ()> = (0..2)
                .map(|_| {
                    let started = Arc::clone(&started);
                    Box::new(move || started.store(true, Ordering::SeqCst))
                        as Box<dyn FnOnce() + Send>
                })
                .collect();
            let _ = setup().run(&fs, run);
        }));
        assert!(result.is_err(), "{what} was accepted");
        assert!(!started.load(Ordering::SeqCst), "{what}: a process started");
    }
    let twice = std::panic::catch_unwind(|| {
        Processes::new(0)
            .kill(0, KillAt::Step(1))
            .kill(0, KillAt::Step(2))
    });
    assert!(twice.is_err(), "a second kill for one process was accepted");
}
