//! Broken store behaviours for the crash matrix's negative controls
//! (INT-0005 AC-3). This module exists only with the `variants` feature,
//! which only `[dev-dependencies]` may enable (`cargo policy`, rule `deps`):
//! a production build cannot select a broken variant.

/// How a store behaves: correctly, or with one known durability bug.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Variant {
    /// The real store.
    Correct,
    /// Keeps a `HEAD` file holding the head set, rewritten in place without a
    /// temporary file, rename or sync, and takes heads from it.
    PointerFile,
    /// Never syncs object files' data (it still renames them and syncs
    /// `objects/`).
    UnsyncedObjects,
    /// `init` syncs no directory.
    UnsyncedInit,
}

impl Variant {
    /// Every variant, the correct one first.
    pub const ALL: [Variant; 4] = [
        Variant::Correct,
        Variant::PointerFile,
        Variant::UnsyncedObjects,
        Variant::UnsyncedInit,
    ];

    /// A short name for reports.
    pub const fn name(self) -> &'static str {
        match self {
            Variant::Correct => "correct",
            Variant::PointerFile => "pointer-file",
            Variant::UnsyncedObjects => "unsynced-objects",
            Variant::UnsyncedInit => "unsynced-init",
        }
    }
}
