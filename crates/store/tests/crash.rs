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
    println!(
        "matrix digest over {} cells: {}",
        first.len(),
        HashAlg::Blake3.hash(&all)
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
