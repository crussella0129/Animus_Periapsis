//! The `view` object (kind `0x04`), *provisional* until the Phase 1 exit gate
//! (`docs/spec/format-v1.md`, INT-0006).
//!
//! Body: `heads: set<hashref> ‖ workspaces: map<string, hashref>`.

use std::collections::{BTreeMap, BTreeSet};

use hash::HashRef;

use crate::envelope::{ObjectKind, decode_as, encode_object};
use crate::error::DecodeError;
use crate::wire::{Decode, Decoder, Encode, Encoder, to_vec};

/// The repository's state as one operation leaves it.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct View {
    /// The visible changesets that no other visible changeset follows.
    pub heads: BTreeSet<HashRef>,
    /// Each workspace's working-copy changeset, by workspace name.
    pub workspaces: BTreeMap<String, HashRef>,
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

impl Encode for View {
    fn encode(&self, enc: &mut Encoder) {
        self.heads.encode(enc);
        self.workspaces.encode(enc);
    }
}

impl Decode for View {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(View {
            heads: BTreeSet::decode(dec)?,
            workspaces: BTreeMap::decode(dec)?,
        })
    }
}
