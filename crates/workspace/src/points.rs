//! The named points the working copy marks (INT-0005 AC-6): its units, as
//! the reach table's columns, and the sprint that built each.

use sim::Point;

/// Every point the working copy marks, checked against the code by
/// `cargo policy`.
pub const POINTS: &[Point] = &[
    Point {
        name: "workspace.snapshot.scanned",
        unit: "workspace::snapshot",
        sprint: 2,
    },
    Point {
        name: "workspace.snapshot.committed",
        unit: "workspace::snapshot",
        sprint: 2,
    },
    Point {
        name: "workspace.record.written",
        unit: "workspace::record",
        sprint: 3,
    },
    Point {
        name: "workspace.record.dropped",
        unit: "workspace::record",
        sprint: 3,
    },
    Point {
        name: "workspace.apply.path_written",
        unit: "workspace::materialise",
        sprint: 3,
    },
    Point {
        name: "workspace.apply.path_removed",
        unit: "workspace::materialise",
        sprint: 3,
    },
    Point {
        name: "workspace.resume.started",
        unit: "workspace::resume",
        sprint: 3,
    },
    Point {
        name: "workspace.resume.dropped",
        unit: "workspace::resume",
        sprint: 3,
    },
];
