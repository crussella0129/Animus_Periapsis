//! Broken workspace behaviours for the Materialise crash scenario's negative
//! control (INT-0005 AC-3). This module exists only with the `variants`
//! feature, which only `[dev-dependencies]` may enable (`cargo policy`, rule
//! `deps`): a production build cannot select a broken variant.

/// How a workspace writes the working copy: correctly, or with one known bug.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Variant {
    /// The real workspace.
    Correct,
    /// Writes the working copy without the working-copy record, so a crash
    /// mid-write leaves a half-written working copy that the next command
    /// snapshots as user edits.
    NoRecord,
}
