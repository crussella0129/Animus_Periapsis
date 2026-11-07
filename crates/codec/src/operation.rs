//! The `operation` object (kind `0x03`), *provisional* until the Phase 1 exit
//! gate (`docs/spec/format-v1.md`, INT-0006).
//!
//! Body: `parents: set<hashref> ‖ objects: set<hashref>`.

use std::collections::BTreeSet;

use hash::HashRef;

use crate::envelope::{ObjectKind, decode_as, encode_object};
use crate::error::DecodeError;
use crate::wire::{Decode, Decoder, Encode, Encoder, to_vec};

/// One entry of the op log.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Operation {
    /// The operations this one follows; empty for a root operation.
    pub parents: BTreeSet<HashRef>,
    /// The objects this operation makes reachable.
    pub objects: BTreeSet<HashRef>,
}

impl Operation {
    /// The operation's envelope, `01 03 ‖ body`.
    pub fn to_envelope(&self) -> Vec<u8> {
        encode_object(ObjectKind::Operation, &to_vec(self))
    }

    /// Decodes an envelope that must hold an operation.
    pub fn from_envelope(bytes: &[u8]) -> Result<Self, DecodeError> {
        decode_as(bytes, ObjectKind::Operation)
    }
}

impl Encode for Operation {
    fn encode(&self, enc: &mut Encoder) {
        self.parents.encode(enc);
        self.objects.encode(enc);
    }
}

impl Decode for Operation {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Operation {
            parents: BTreeSet::decode(dec)?,
            objects: BTreeSet::decode(dec)?,
        })
    }
}
