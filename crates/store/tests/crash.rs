//! The crash matrix and its negative controls (INT-0005 AC-1 to AC-3;
//! INT-0007 AC-1 to AC-3). `cargo test -- --nocapture` prints the counts.

mod common;

use common::matrix::{Outcome, Scenario, Summary, run_matrix};
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

fn caught(variant: Variant) {
    let (summary, _) = matrix(variant);
    let strict = summary.violations[&Mode::Strict];
    assert!(
        strict.semantic() > 0,
        "{} produced no closure, durability, heads or atomicity violation: {strict:?}",
        variant.name()
    );
}

#[test]
fn pointer_file_variant_is_caught() {
    caught(Variant::PointerFile);
}

#[test]
fn unsynced_objects_variant_is_caught() {
    caught(Variant::UnsyncedObjects);
}

#[test]
fn unsynced_init_variant_is_caught() {
    caught(Variant::UnsyncedInit);
}
