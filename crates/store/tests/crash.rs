//! The crash matrix and its negative controls (INT-0005 AC-1 to AC-3;
//! INT-0007 AC-1 to AC-3). `cargo test -- --nocapture` prints the counts.

mod common;

use common::matrix::{Outcome, Race, Scenario, Summary, run_matrix};
use hash::HashAlg;
use sim::Mode;
use store::Variant;

type Cells = Vec<(Scenario, Mode, u64, u64, Outcome)>;

fn matrix(variant: Variant) -> (Summary, Cells) {
    let mut cells = Vec::new();
    let summary = run_matrix(variant, |scenario, mode, seed, crash_at, outcome, _| {
        cells.push((scenario, mode, seed, crash_at, outcome.clone()));
    });
    summary.print(variant);
    for mode in [Mode::Lenient, Mode::Strict] {
        assert!(
            summary.cells.get(&mode).copied().unwrap_or(0) > 0,
            "{} ran no {mode:?} cells",
            variant.name()
        );
    }
    (summary, cells)
}

/// Where the designated tests record reach for `cargo reach`.
fn reach_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("reach")
}

#[test]
fn crash_matrix_finds_no_violations() {
    let (summary, cells) = matrix(Variant::Correct);
    let failing: Vec<_> = cells
        .iter()
        .filter(|(.., outcome)| outcome.violations.total() > 0)
        .take(5)
        .map(|(s, m, seed, k, o)| (s, m, seed, k, o.violations))
        .collect();
    assert!(failing.is_empty(), "first violating cells: {failing:?}");
    for violations in summary.violations.values() {
        assert_eq!(violations.total(), 0);
    }
    // Both two-writer races must actually run: some cells acknowledge B's
    // op while the view or parent op it names is not yet durable, so the
    // re-sync is what saves it.
    for mode in [Mode::Lenient, Mode::Strict] {
        for race in [Race::View, Race::Parent] {
            assert!(
                summary.races(mode, race) > 0,
                "no {mode:?} cell ran the {race:?} race"
            );
        }
    }
    // The designated record of this scenario's reach (INT-0005 AC-6): the
    // two writers apart from the single-writer scenarios.
    let (mut store, mut two) = (sim::reach::Totals::new(), sim::reach::Totals::new());
    for (scenario, _, _, _, outcome) in &cells {
        let totals = if *scenario == Scenario::TwoWriters {
            &mut two
        } else {
            &mut store
        };
        sim::reach::add(totals, &outcome.reach);
    }
    sim::reach::record(&reach_dir(), "store", "store", 1, &store).unwrap();
    sim::reach::record(&reach_dir(), "two-writers", "two writers", 1, &two).unwrap();
}

#[test]
fn matrix_runs_are_reproducible() {
    let (_, first) = matrix(Variant::Correct);
    let (_, second) = matrix(Variant::Correct);
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(&second) {
        assert_eq!(a.0..=a.0, b.0..=b.0);
        assert_eq!((a.1, a.2, a.3), (b.1, b.2, b.3));
        assert_eq!(a.4.state, b.4.state, "cell {:?}", (a.0, a.1, a.2, a.3));
        assert_eq!(a.4.trace, b.4.trace, "cell {:?}", (a.0, a.1, a.2, a.3));
    }
    // One digest over every cell, to compare across systems.
    let mut all = Vec::new();
    for (.., outcome) in &first {
        all.extend_from_slice(&outcome.state.to_bytes());
        all.extend_from_slice(&outcome.trace.to_bytes());
    }
    let digest = HashAlg::Blake3.hash(&all);
    println!("matrix digest over {} cells: {digest}", first.len());
    // Asserted, not only printed, so a change that moves any cell fails
    // here on every system (Sprint 4 plan critique, C-008).
    assert_eq!(first.len(), 32_840);
    assert_eq!(
        digest.to_string(),
        "blake3:d36b8c9c77744d485a362ef3d94cebe40dcb67ca7260eb409f5430c2a13ae812"
    );
}

fn caught(variant: Variant) -> Cells {
    let (summary, cells) = matrix(variant);
    let strict = summary.violations[&Mode::Strict];
    assert!(
        strict.semantic() > 0,
        "{} produced no closure, durability, heads or atomicity violation: {strict:?}",
        variant.name()
    );
    cells
}

#[test]
fn pointer_file_variant_is_caught() {
    caught(Variant::PointerFile);
}

/// Every data sync of an object file is skipped: acknowledged objects are
/// lost or torn (Sprint 2 T-003).
#[test]
fn unsynced_objects_variant_is_caught() {
    let cells = caught(Variant::UnsyncedObjects);
    let strict = cells.iter().filter(|c| c.1 == Mode::Strict);
    let durability: u64 = strict.clone().map(|c| c.4.violations.durability).sum();
    let torn: u64 = strict.map(|c| c.4.violations.torn).sum();
    assert!(
        durability > 0 && torn > 0,
        "durability {durability}, torn {torn}"
    );
}

/// `commit_op` does not re-sync the view and parent ops it names: B's op can
/// survive while what it names is lost, in both race kinds (Sprint 2 T-003;
/// INT-0007 Consequences).
#[test]
fn unsynced_references_variant_is_caught() {
    let cells = caught(Variant::UnsyncedReferences);
    for race in [Race::View, Race::Parent] {
        let closure: u64 = cells
            .iter()
            .filter(|c| c.1 == Mode::Strict && c.0 == Scenario::TwoWriters)
            .filter(|c| c.2.is_multiple_of(2) == (race == Race::View))
            .map(|c| c.4.violations.closure)
            .sum();
        assert!(closure > 0, "no closure violation in {race:?}-race cells");
    }
}

#[test]
fn unsynced_init_variant_is_caught() {
    caught(Variant::UnsyncedInit);
}

#[test]
fn write_in_place_variant_is_caught() {
    caught(Variant::WriteInPlace);
}

/// The two-writer race fires from a named point, with no trace inspection,
/// in every seed of both modes (Sprint 4 T-006). `syscalls` asserts that
/// each uncrashed correct run raced; the digest test shows the cells are
/// unchanged from when a hook matched the trace.
#[test]
fn two_writer_race_fires_at_the_named_point() {
    for mode in [Mode::Lenient, Mode::Strict] {
        for seed in 0..common::matrix::SEEDS {
            common::matrix::syscalls(Variant::Correct, Scenario::TwoWriters, mode, seed);
        }
    }
}
