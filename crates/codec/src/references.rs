//! The ids an object names (INT-0007: closure follows references inside
//! objects).

use hash::HashRef;

use crate::changeset::{Changeset, Entry};
use crate::envelope::{ObjectKind, decode_envelope};
use crate::error::DecodeError;
use crate::operation::Operation;
use crate::view::View;

/// The ids the object in `envelope` names, in body order, after decoding it
/// in full: a changeset's parents and put blobs; a view's heads and workspace
/// changesets; an operation's parents and view; nothing for a blob. An
/// envelope that does not decode as its kind is an error.
pub fn references(envelope: &[u8]) -> Result<Vec<HashRef>, DecodeError> {
    let kind = decode_envelope(envelope)?.kind;
    Ok(match kind {
        ObjectKind::Blob => Vec::new(),
        ObjectKind::Changeset => {
            let changeset = Changeset::from_envelope(envelope)?;
            let blobs = changeset.changes.values().filter_map(|entry| match entry {
                Entry::Put { blob, .. } => Some(*blob),
                Entry::Delete => None,
            });
            changeset.parents.iter().copied().chain(blobs).collect()
        }
        ObjectKind::View => {
            let view = View::from_envelope(envelope)?;
            view.heads
                .iter()
                .chain(view.workspaces.values())
                .copied()
                .collect()
        }
        ObjectKind::Operation => {
            let op = Operation::from_envelope(envelope)?;
            op.parents.iter().copied().chain([op.view]).collect()
        }
    })
}
