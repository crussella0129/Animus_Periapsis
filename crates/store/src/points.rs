//! The named points the store marks (INT-0005 AC-6): its units, as the
//! reach table's columns, and the sprint that built each.

use sim::Point;

/// Every point the store marks, checked against the code by `cargo policy`.
pub const POINTS: &[Point] = &[
    Point {
        name: "store.init.layout_created",
        unit: "store::init",
        sprint: 1,
    },
    Point {
        name: "store.sync_layout.synced",
        unit: "store::sync_layout",
        sprint: 1,
    },
    Point {
        name: "store.put_object.present_synced",
        unit: "store::put_object",
        sprint: 1,
    },
    Point {
        name: "store.put_object.temp_written",
        unit: "store::put_object",
        sprint: 1,
    },
    Point {
        name: "store.put_object.temp_synced",
        unit: "store::put_object",
        sprint: 1,
    },
    Point {
        name: "store.put_object.renamed",
        unit: "store::put_object",
        sprint: 1,
    },
    Point {
        name: "store.put_object.dir_synced",
        unit: "store::put_object",
        sprint: 1,
    },
    Point {
        name: "store.commit_op.references_resynced",
        unit: "store::commit_op",
        sprint: 1,
    },
    Point {
        name: "store.commit_op.present_synced",
        unit: "store::commit_op",
        sprint: 1,
    },
    Point {
        name: "store.commit_op.temp_written",
        unit: "store::commit_op",
        sprint: 1,
    },
    Point {
        name: "store.commit_op.temp_synced",
        unit: "store::commit_op",
        sprint: 1,
    },
    Point {
        name: "store.commit_op.renamed",
        unit: "store::commit_op",
        sprint: 1,
    },
    Point {
        name: "store.commit_op.dir_synced",
        unit: "store::commit_op",
        sprint: 1,
    },
];
