//! Broken workspace behaviours for the negative controls of the Materialise
//! and Describe crash scenarios (INT-0005 AC-3). This module exists only
//! with the `variants` feature, which only `[dev-dependencies]` may enable
//! (`cargo policy`, rule `deps`): a production build cannot select a broken
//! variant.

/// How a workspace behaves: correctly, or with one known bug.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Variant {
    /// The real workspace.
    Correct,
    /// Writes the working copy without the working-copy record, so a crash
    /// mid-write leaves a half-written working copy that the next command
    /// snapshots as user edits.
    NoRecord,
    /// A snapshot that records an edit writes the change with no
    /// description, so what `describe` set is lost by the next edit.
    DropDescription,
    /// Making a lane writes `.praps/workspace` in place, with no temporary
    /// name and no sync, so a crash can leave it torn or lost while the
    /// lane's op stands.
    LaneFileInPlace,
    /// A load takes only the first op head and ignores the others, so a
    /// command may continue from a state its own workspace has left.
    FirstHeadOnly,
    /// A load that finds several op heads commits their merge as an op, so
    /// a reader writes.
    MergeOnRead,
}
