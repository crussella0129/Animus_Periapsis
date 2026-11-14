//! Several simulated processes over one [`SimFs`], run one at a time by a
//! seeded scheduler (INT-0005; INT-0007 AC-5).
//!
//! Each process is an OS thread, but a baton lets exactly one run at a time,
//! so a run is a function of its seed. At every boundary of the running
//! process (each syscall, each named point, its end, its kill or its wait on
//! an ordering) the scheduler draws the next runner from the seed among the
//! processes able to run. Orderings, in the form of RocksDB's sync-point
//! dependencies, hold a process at a named point until another process has
//! reached a point or ended. A process can be killed at a boundary: it stops
//! there, makes no further syscall, and everything it wrote stays visible to
//! the others until a system crash settles it.

use std::any::Any;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;

use crate::rng::Rng;
use crate::simfs::{Boundary, SimFs};

/// What a process waits for before it may pass a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum After {
    /// Process `process` has arrived at point `name` for the `nth` time.
    Point {
        process: usize,
        name: &'static str,
        nth: u32,
    },
    /// Process `process` has ended: returned, been killed or panicked.
    End(usize),
}

/// Where a kill takes effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KillAt {
    /// At the process's `k`-th boundary, counting from 1.
    Step(u64),
    /// On the process's `nth` arrival at point `name`, before any wait there.
    Point { name: &'static str, nth: u32 },
}

/// How one process ended.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome<T> {
    /// It returned this value.
    Done(T),
    /// It was killed, or stopped because the run ended early.
    Killed,
}

/// Why a run stopped before every process ended.
pub enum Stopped {
    /// A process panicked; this is the first panic, from that process. Every
    /// other process was unwound at its next boundary.
    Panicked {
        process: usize,
        payload: Box<dyn Any + Send>,
    },
    /// Every live process was waiting on a live process: the orderings
    /// cannot be met. Each waiting point is named, as `process: name`.
    Deadlock(Vec<String>),
}

impl std::fmt::Debug for Stopped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stopped::Panicked { process, payload } => {
                let message = payload
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_owned())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_default();
                write!(f, "process {process} panicked: {message}")
            }
            Stopped::Deadlock(waiting) => write!(f, "ordering deadlock: {waiting:?}"),
        }
    }
}

/// A run's setup: the seed, the orderings and the kills.
#[derive(Clone, Debug, Default)]
pub struct Processes {
    seed: u64,
    orderings: Vec<(After, usize, &'static str, u32)>,
    kills: BTreeMap<usize, KillAt>,
}

/// The marker a killed or stopped process unwinds with.
struct Stop;

thread_local! {
    /// The process the current thread runs, if it is one.
    static PROCESS: Cell<Option<usize>> = const { Cell::new(None) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Life {
    /// Waiting for its first turn or between turns.
    Live,
    /// Returned, killed or panicked.
    Ended,
}

struct Sched {
    rng: Rng,
    running: Option<usize>,
    life: Vec<Life>,
    steps: Vec<u64>,
    arrivals: BTreeMap<(usize, &'static str), u32>,
    /// What each process waits for before it may go on, if anything, and
    /// the point it waits at.
    waiting: Vec<Option<(Vec<After>, &'static str)>>,
    /// The run is ending early: every process unwinds at its next boundary.
    abort: bool,
    panic: Option<(usize, Box<dyn Any + Send>)>,
    deadlock: Option<Vec<String>>,
}

struct Shared {
    sched: Mutex<Sched>,
    /// One for each process, which waits on its own: a boundary wakes only
    /// the process drawn to run next, or every process once the run is
    /// ending (Sprint 7 T-008). Waking every thread at every boundary cost
    /// about 3 ms a boundary at 1,000 processes.
    turns: Vec<Condvar>,
    setup: Processes,
    fs: Arc<SimFs>,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Sched> {
        self.sched.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn met(&self, sched: &Sched, after: &After) -> bool {
        match *after {
            After::Point { process, name, nth } => {
                sched.arrivals.get(&(process, name)).copied().unwrap_or(0) >= nth
                    || sched.life[process] == Life::Ended
            }
            After::End(process) => sched.life[process] == Life::Ended,
        }
    }

    fn runnable(&self, sched: &Sched, crashed: bool, process: usize) -> bool {
        sched.life[process] == Life::Live
            && match &sched.waiting[process] {
                None => true,
                Some((afters, _)) => crashed || afters.iter().all(|a| self.met(sched, a)),
            }
    }

    /// Draws the next runner among the processes able to run. With none
    /// able while some are live, the orderings cannot be met.
    fn draw(&self, sched: &mut Sched) {
        let crashed = self.fs.crashed();
        let able: Vec<usize> = (0..sched.life.len())
            .filter(|&p| self.runnable(sched, crashed, p))
            .collect();
        if able.is_empty() {
            let live: Vec<usize> = (0..sched.life.len())
                .filter(|&p| sched.life[p] == Life::Live)
                .collect();
            if !live.is_empty() && sched.deadlock.is_none() && sched.panic.is_none() {
                sched.deadlock = Some(
                    live.iter()
                        .map(|&p| match &sched.waiting[p] {
                            Some((_, at)) => format!("{p}: {at}"),
                            None => format!("{p}: (not waiting)"),
                        })
                        .collect(),
                );
                sched.abort = true;
            }
            sched.running = None;
        } else {
            let pick = sched.rng.below(able.len() as u64) as usize;
            sched.running = Some(able[pick]);
        }
        self.wake(sched);
    }

    /// Wakes the process that runs next, or, when the run is ending, every
    /// process, so that each unwinds.
    fn wake(&self, sched: &Sched) {
        if sched.abort {
            for turn in &self.turns {
                turn.notify_all();
            }
        } else if let Some(next) = sched.running {
            self.turns[next].notify_all();
        }
    }

    /// Blocks until `me` holds the baton, or the run is ending (then the
    /// caller unwinds).
    fn wait_turn<'a>(
        &'a self,
        mut sched: MutexGuard<'a, Sched>,
        me: usize,
    ) -> MutexGuard<'a, Sched> {
        while sched.running != Some(me) && !sched.abort {
            sched = self.turns[me]
                .wait(sched)
                .unwrap_or_else(PoisonError::into_inner);
        }
        sched
    }

    /// Ends `me`'s turn for good and hands the baton on.
    fn end(&self, me: usize) {
        let mut sched = self.lock();
        sched.life[me] = Life::Ended;
        sched.waiting[me] = None;
        if sched.running == Some(me) || sched.running.is_none() {
            self.draw(&mut sched);
        } else {
            self.wake(&sched);
        }
    }

    /// The scheduler's work at one boundary of the running process.
    fn boundary(&self, me: usize, at: Boundary) {
        let mut sched = self.lock();
        if sched.abort {
            drop(sched);
            panic::resume_unwind(Box::new(Stop));
        }
        sched.steps[me] += 1;
        let step = sched.steps[me];
        let mut arrival = 0;
        if let Boundary::Point(name) = at {
            let count = sched.arrivals.entry((me, name)).or_insert(0);
            *count += 1;
            arrival = *count;
        }
        // A kill set for this arrival takes effect before any wait.
        let killed = match self.setup.kills.get(&me) {
            Some(KillAt::Step(k)) => *k == step,
            Some(KillAt::Point { name, nth }) => {
                matches!(at, Boundary::Point(at) if at == *name) && arrival == *nth
            }
            None => false,
        };
        if killed {
            drop(sched);
            panic::resume_unwind(Box::new(Stop));
        }
        if let Boundary::Point(name) = at {
            let afters: Vec<After> = self
                .setup
                .orderings
                .iter()
                .filter(|(_, process, point, nth)| {
                    *process == me && *point == name && *nth == arrival
                })
                .map(|(after, ..)| *after)
                .collect();
            if !afters.is_empty() {
                sched.waiting[me] = Some((afters, name));
            }
        }
        self.draw(&mut sched);
        let mut sched = self.wait_turn(sched, me);
        if sched.abort {
            drop(sched);
            panic::resume_unwind(Box::new(Stop));
        }
        sched.waiting[me] = None;
    }
}

impl Processes {
    /// A run whose scheduling draws come from `seed`, with no orderings and
    /// no kills.
    pub fn new(seed: u64) -> Self {
        Processes {
            seed,
            ..Processes::default()
        }
    }

    /// Holds process `process` on its `nth` arrival at point `point` until
    /// `after` holds. The wait is released once the system has crashed, or
    /// once the process `after` names has ended.
    pub fn order(mut self, after: After, process: usize, point: &'static str, nth: u32) -> Self {
        self.orderings.push((after, process, point, nth));
        self
    }

    /// Kills process `process` at `at`. A process has at most one kill.
    pub fn kill(mut self, process: usize, at: KillAt) -> Self {
        let previous = self.kills.insert(process, at);
        assert!(
            previous.is_none(),
            "process {process} already has a kill: {previous:?}"
        );
        self
    }

    /// Refuses a setup that could never act as written: a process index
    /// outside the run, a zeroth arrival or step (counts start at 1). An
    /// off-by-one there would otherwise leave a run with no kill or no
    /// forced race, and a test passing without testing (code review CR-02).
    fn validate(&self, n: usize) {
        let process_ok = |p: usize, what: &str| {
            assert!(p < n, "{what} names process {p}, but the run has {n}");
        };
        for (after, process, point, nth) in &self.orderings {
            process_ok(*process, "an ordering");
            assert!(*nth >= 1, "an ordering at {point} names arrival 0");
            match after {
                After::Point { process, name, nth } => {
                    process_ok(*process, "an ordering's predecessor");
                    assert!(
                        *nth >= 1,
                        "an ordering's predecessor {name} names arrival 0"
                    );
                }
                After::End(process) => process_ok(*process, "an ordering's predecessor"),
            }
        }
        for (process, at) in &self.kills {
            process_ok(*process, "a kill");
            match at {
                KillAt::Step(k) => assert!(*k >= 1, "a kill at step 0"),
                KillAt::Point { name, nth } => {
                    assert!(*nth >= 1, "a kill at {name} names arrival 0");
                }
            }
        }
    }

    /// Runs `closures` as processes over `fs` and returns how each ended, in
    /// input order, or why the run stopped early.
    pub fn run<T: Send>(
        &self,
        fs: &Arc<SimFs>,
        closures: Vec<Box<dyn FnOnce() -> T + Send + '_>>,
    ) -> Result<Vec<Outcome<T>>, Stopped> {
        let n = closures.len();
        self.validate(n);
        let mut rng = Rng::new(self.seed);
        let first = if n == 0 {
            None
        } else {
            Some(rng.below(n as u64) as usize)
        };
        let shared = Arc::new(Shared {
            sched: Mutex::new(Sched {
                rng,
                running: first,
                life: vec![Life::Live; n],
                steps: vec![0; n],
                arrivals: BTreeMap::new(),
                waiting: vec![None; n],
                abort: false,
                panic: None,
                deadlock: None,
            }),
            turns: (0..n).map(|_| Condvar::new()).collect(),
            setup: self.clone(),
            fs: Arc::clone(fs),
        });
        let callback = Arc::clone(&shared);
        fs.set_switch(Some(Arc::new(move |at| {
            if let Some(me) = PROCESS.with(Cell::get) {
                callback.boundary(me, at);
            }
        })));
        let outcomes: Vec<Outcome<T>> = thread::scope(|scope| {
            let handles: Vec<_> = closures
                .into_iter()
                .enumerate()
                .map(|(me, closure)| {
                    let shared = Arc::clone(&shared);
                    scope.spawn(move || {
                        PROCESS.with(|p| p.set(Some(me)));
                        let sched = shared.wait_turn(shared.lock(), me);
                        let started = !sched.abort;
                        drop(sched);
                        let result = if started {
                            panic::catch_unwind(AssertUnwindSafe(closure))
                        } else {
                            Err(Box::new(Stop) as Box<dyn Any + Send>)
                        };
                        PROCESS.with(|p| p.set(None));
                        let outcome = match result {
                            Ok(value) => Outcome::Done(value),
                            Err(payload) if payload.is::<Stop>() => Outcome::Killed,
                            Err(payload) => {
                                let mut sched = shared.lock();
                                if sched.panic.is_none() {
                                    sched.panic = Some((me, payload));
                                }
                                sched.abort = true;
                                drop(sched);
                                Outcome::Killed
                            }
                        };
                        shared.end(me);
                        outcome
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap_or(Outcome::Killed))
                .collect()
        });
        fs.set_switch(None);
        let mut sched = shared.lock();
        if let Some((process, payload)) = sched.panic.take() {
            return Err(Stopped::Panicked { process, payload });
        }
        if let Some(waiting) = sched.deadlock.take() {
            return Err(Stopped::Deadlock(waiting));
        }
        Ok(outcomes)
    }
}
