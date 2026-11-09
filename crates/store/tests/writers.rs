//! Four writers in separate simulated processes, under crash injection and
//! process kills (Sprint 4 T-007; INT-0007 AC-5; INT-0005 AC-2, AC-3).
//! `cargo test -- --nocapture` prints the counts.

mod common;

use std::sync::OnceLock;
use std::time::Instant;

use common::matrix::SEEDS;
use common::writers::{
    Cell4, Summary, cells_digest, first_op_id, kill_for, matrix, modes, run_cell, run_cell_logged,
    syscalls, uncrashed,
};
use sim::Mode;
use store::Variant;

/// The correct matrix, run once and shared by the two tests that need it,
/// so it runs twice in all, not three times (plan critique C-008).
fn correct() -> &'static (Summary, Vec<Cell4>) {
    static CORRECT: OnceLock<(Summary, Vec<Cell4>)> = OnceLock::new();
    CORRECT.get_or_init(|| {
        let start = Instant::now();
        let result = matrix(Variant::Correct, modes());
        println!(
            "writers matrix correct: {} cells in {:.0} s",
            result.1.len(),
            start.elapsed().as_secs_f64()
        );
        result
    })
}

fn print(variant: Variant, summary: &Summary) {
    for ((mode, kill), cells) in &summary.cells {
        let killed = summary.killed.get(&(*mode, *kill)).copied().unwrap_or(0);
        println!(
            "writers {:<20} {mode:?} kill {kill}: {cells} cells, {killed} killed a process",
            variant.name()
        );
    }
    for (mode, v) in &summary.violations {
        println!(
            "writers {:<20} {mode:?}: closure {} durability {} heads {} atomicity {} torn {} \
             layout {} stray {}; odd cells with process 0's op undurable at process 2's \
             rename: {}",
            variant.name(),
            v.closure,
            v.durability,
            v.heads,
            v.atomicity,
            v.torn,
            v.layout,
            v.stray,
            summary.undurable.get(mode).copied().unwrap_or(0)
        );
    }
}

#[test]
fn writers_matrix_finds_no_violations() {
    let (summary, cells) = correct();
    print(Variant::Correct, summary);
    println!("writers fsck problems: {}", summary.fsck);
    for mode in modes() {
        let v = summary.violations[mode];
        assert_eq!(v.total(), 0, "{mode:?}: {v:?}");
    }
    assert_eq!(summary.fsck, 0);
    for mode in modes() {
        for kill in [false, true] {
            assert!(summary.cells.get(&(*mode, kill)).copied().unwrap_or(0) > 0);
        }
        assert!(summary.killed.get(&(*mode, true)).copied().unwrap_or(0) > 0);
    }
    assert!(!cells.is_empty());
    // The designated record of this scenario's reach (INT-0005 AC-6).
    let mut totals = sim::reach::Totals::new();
    for (_, outcome) in cells {
        sim::reach::add(&mut totals, &outcome.reach);
    }
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("reach");
    sim::reach::record(&dir, "four-writers", "four writers", 4, &totals).unwrap();
}

#[test]
fn writers_matrix_is_reproducible() {
    let (_, first) = correct();
    let (_, second) = matrix(Variant::Correct, modes());
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(&second) {
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.state, b.1.state, "cell {:?}", a.0);
        assert_eq!(a.1.trace, b.1.trace, "cell {:?}", a.0);
        assert_eq!(a.1.violations, b.1.violations, "cell {:?}", a.0);
    }
    let digest = cells_digest(first);
    println!("writers matrix digest over {} cells: {digest}", first.len());
    // Asserted, so a change that moves any cell fails on every system. A
    // new point moves it too: under several processes a point is a
    // scheduling boundary (docs/spec/simulation.md).
    assert_eq!(first.len(), 44_836);
    assert_eq!(
        digest.to_string(),
        "blake3:253b1eabc4054f7aebfb6edd2e907fdaccbc76549158410a47fedc610ba0855c"
    );
}

#[test]
fn writers_forced_race_runs() {
    for mode in modes() {
        for seed in (1..SEEDS).step_by(2) {
            let (_, log, _) = uncrashed(Variant::Correct, *mode, seed, None);
            assert!(
                log.names_op0,
                "{mode:?} seed {seed}: process 2 did not name process 0's op"
            );
            assert_eq!(
                log.at_listing,
                Some((true, false)),
                "{mode:?} seed {seed}: (visible, durable) at the listing"
            );
        }
    }
}

#[test]
fn writers_overlap_in_some_even_seed() {
    let overlapped = modes().iter().any(|mode| {
        (0..SEEDS).step_by(2).any(|seed| {
            let (n, ..) = syscalls(Variant::Correct, *mode, seed, None);
            run_cell(Variant::Correct, *mode, seed, None, n + 1).overlapped
        })
    });
    assert!(overlapped, "no even seed overlapped two commits");
}

#[test]
fn writers_kills_happen() {
    for mode in modes() {
        for seed in 0..SEEDS {
            let kill = kill_for(Variant::Correct, *mode, seed, true);
            let (process, _) = kill.unwrap();
            let (_, _, killed) = uncrashed(Variant::Correct, *mode, seed, kill);
            assert!(
                killed[process],
                "{mode:?} seed {seed}: process {process} was not killed"
            );
        }
    }
}

#[test]
fn writers_retry_reaches_present() {
    let (fs, log, _) = uncrashed(Variant::Correct, Mode::Strict, 0, None);
    assert!(log.retried);
    assert!(
        fs.reach()
            .iter()
            .any(|(name, count)| *name == "store.commit_op.present_synced" && *count > 0)
    );
    // A crash during the retry: the op stays acknowledged, and present.
    let op1 = first_op_id(1);
    let mut interrupted = 0;
    for mode in modes() {
        for seed in 0..4 {
            let (n, ..) = syscalls(Variant::Correct, *mode, seed, None);
            for crash_at in 1..=n {
                let (outcome, log) = run_cell_logged(Variant::Correct, *mode, seed, None, crash_at);
                if log.retry_started && !log.retried {
                    interrupted += 1;
                    assert!(log.acked.contains(&op1));
                    assert_eq!(outcome.violations.durability, 0);
                }
            }
        }
    }
    println!("cells that crashed during the retry: {interrupted}");
    assert!(interrupted > 0);
}

fn caught(variant: Variant) -> Summary {
    let (summary, _) = matrix(variant, &[Mode::Strict]);
    print(variant, &summary);
    let strict = summary.violations[&Mode::Strict];
    assert!(
        strict.semantic() > 0,
        "{} produced no semantic violation: {strict:?}",
        variant.name()
    );
    summary
}

#[test]
fn writers_catch_pointer_file() {
    caught(Variant::PointerFile);
}

#[test]
fn writers_catch_unsynced_objects() {
    let strict = caught(Variant::UnsyncedObjects).violations[&Mode::Strict];
    assert!(strict.durability > 0 && strict.torn > 0, "{strict:?}");
}

#[test]
fn writers_catch_unsynced_init() {
    caught(Variant::UnsyncedInit);
}

#[test]
fn writers_catch_write_in_place() {
    caught(Variant::WriteInPlace);
}

#[test]
fn writers_catch_unsynced_references() {
    let (summary, cells) = matrix(Variant::UnsyncedReferences, &[Mode::Strict]);
    print(Variant::UnsyncedReferences, &summary);
    let closure: u64 = cells
        .iter()
        .filter(|((_, seed, ..), _)| seed % 2 == 1)
        .map(|(_, outcome)| outcome.violations.closure)
        .sum();
    assert!(closure > 0, "no closure violation in the forced-race cells");
    assert!(summary.undurable.get(&Mode::Strict).copied().unwrap_or(0) > 0);
}
