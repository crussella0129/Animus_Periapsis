//! The `view` object (kind `0x04`), *provisional* until the Phase 1 exit gate
//! (`docs/spec/format-v1.md`, INT-0006).
//!
//! Body: `workspaces: map<string, entry>`, with
//! `entry = working_copy: set<hashref> ‖ heads: set<hashref>`.

use std::collections::{BTreeMap, BTreeSet};

use hash::HashRef;

use crate::envelope::{ObjectKind, decode_as, encode_object};
use crate::error::DecodeError;
use crate::wire::{Decode, Decoder, Encode, Encoder, to_vec};

/// The repository's state as one operation leaves it: one entry for each
/// workspace, by name.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct View {
    /// Each workspace's entry, by workspace name. No name is empty.
    pub workspaces: BTreeMap<String, WorkspaceEntry>,
}

/// One workspace's part of a view. Only operations made in that workspace
/// write it (INT-0016).
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceEntry {
    /// The working-copy changeset: one, or several when commands moved the
    /// pointer at once and no command of this workspace has chosen yet (the
    /// support-set normal form of INT-0010). Never empty.
    pub working_copy: BTreeSet<HashRef>,
    /// The visible changesets this workspace made that no other visible
    /// changeset follows.
    pub heads: BTreeSet<HashRef>,
}

impl View {
    /// The view's envelope, `01 04 ‖ body`.
    pub fn to_envelope(&self) -> Vec<u8> {
        encode_object(ObjectKind::View, &to_vec(self))
    }

    /// Decodes an envelope that must hold a view.
    pub fn from_envelope(bytes: &[u8]) -> Result<Self, DecodeError> {
        decode_as(bytes, ObjectKind::View)
    }
}

impl Encode for WorkspaceEntry {
    fn encode(&self, enc: &mut Encoder) {
        self.working_copy.encode(enc);
        self.heads.encode(enc);
    }
}

impl Decode for WorkspaceEntry {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let at = dec.position();
        let working_copy = BTreeSet::decode(dec)?;
        if working_copy.is_empty() {
            return Err(DecodeError::EmptyWorkingCopy { at });
        }
        Ok(WorkspaceEntry {
            working_copy,
            heads: BTreeSet::decode(dec)?,
        })
    }
}

impl Encode for View {
    fn encode(&self, enc: &mut Encoder) {
        self.workspaces.encode(enc);
    }
}

impl Decode for View {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        // The map's decoding, with each name checked where it starts.
        let count = dec.length()?;
        let mut workspaces = BTreeMap::new();
        for _ in 0..count {
            let at = dec.position();
            let name = String::decode(dec)?;
            if name.is_empty() {
                return Err(DecodeError::EmptyWorkspaceName { at });
            }
            if workspaces
                .last_key_value()
                .is_some_and(|(last, _): (&String, _)| &name <= last)
            {
                return Err(DecodeError::UnsortedOrDuplicateKey { at });
            }
            let entry = WorkspaceEntry::decode(dec)?;
            workspaces.insert(name, entry);
        }
        Ok(View { workspaces })
    }
}
