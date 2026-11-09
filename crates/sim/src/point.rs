//! The registry entry for a named point (INT-0005 AC-6).
//!
//! Code marks a point with [`crate::Fs::point`]. Each boundary crate lists
//! every point it marks in `pub const POINTS: &[Point]` in its `src/points.rs`,
//! one `Point { name: "…", unit: "…", sprint: N }` per point; `cargo policy`
//! checks that the list and the code agree, and `cargo reach` builds the
//! reach table from it.

/// One named point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    /// `crate.function.stage`, in lowercase ASCII with `_`.
    pub name: &'static str,
    /// The function or module of the boundary crate the point marks, as
    /// `crate::name`: a column of the reach table.
    pub unit: &'static str,
    /// The sprint whose build plan delivered the unit.
    pub sprint: u32,
}
