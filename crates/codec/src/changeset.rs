//! The `changeset` object (kind `0x02`), *provisional* until the Phase 1 exit
//! gate (`docs/spec/format-v1.md`, INT-0006).
//!
//! Body: `parents: seq<hashref> ‖ change_id: 16 raw bytes ‖
//! changes: map<path, entry> ‖ description: bytes`, where an entry is
//! `00` (delete) or `01 ‖ blob: hashref ‖ mode: byte` (put).

use std::collections::BTreeMap;

use hash::HashRef;

use crate::envelope::{ObjectKind, decode_as, encode_object};
use crate::error::DecodeError;
use crate::path::RepoPath;
use crate::wire::{Decode, Decoder, Encode, Encoder, to_vec};

/// The length of a change ID in bytes (128 bits, INT-0011).
pub const CHANGE_ID_LEN: usize = 16;

/// A file's mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    /// A regular file (`00`).
    Regular,
    /// An executable file (`01`).
    Executable,
}

impl Mode {
    /// The mode byte reserved for symbolic links, rejected until specified.
    pub const RESERVED_SYMLINK: u8 = 0x02;

    /// The mode's byte.
    pub const fn byte(self) -> u8 {
        match self {
            Mode::Regular => 0x00,
            Mode::Executable => 0x01,
        }
    }
}

impl Encode for Mode {
    fn encode(&self, enc: &mut Encoder) {
        enc.byte(self.byte());
    }
}

impl Decode for Mode {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let at = dec.position();
        match dec.byte()? {
            0x00 => Ok(Mode::Regular),
            0x01 => Ok(Mode::Executable),
            Mode::RESERVED_SYMLINK => Err(DecodeError::ReservedMode {
                at,
                byte: Mode::RESERVED_SYMLINK,
            }),
            byte => Err(DecodeError::UnknownMode { at, byte }),
        }
    }
}

/// What a changeset does to one path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Entry {
    /// The path is removed (`00`).
    Delete,
    /// The path holds this blob with this mode (`01`); added or modified
    /// alike, as Bonsai changesets do not tell them apart.
    Put {
        /// The id of the blob holding the file's content.
        blob: HashRef,
        /// The file's mode.
        mode: Mode,
    },
}

impl Encode for Entry {
    fn encode(&self, enc: &mut Encoder) {
        match self {
            Entry::Delete => enc.byte(0x00),
            Entry::Put { blob, mode } => {
                enc.byte(0x01);
                blob.encode(enc);
                mode.encode(enc);
            }
        }
    }
}

impl Decode for Entry {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let at = dec.position();
        match dec.byte()? {
            0x00 => Ok(Entry::Delete),
            0x01 => Ok(Entry::Put {
                blob: HashRef::decode(dec)?,
                mode: Mode::decode(dec)?,
            }),
            byte => Err(DecodeError::UnknownEntryTag { at, byte }),
        }
    }
}

/// One version of a change: what it changes against its parents.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Changeset {
    /// The changesets this one follows, in order; empty for a root.
    pub parents: Vec<HashRef>,
    /// The change ID, the same across every rewrite of the change.
    pub change_id: [u8; CHANGE_ID_LEN],
    /// What happens to each changed path, against the first parent's tree.
    pub changes: BTreeMap<RepoPath, Entry>,
    /// The change's description, as bytes (INT-0034).
    pub description: Vec<u8>,
}

impl Changeset {
    /// The changeset's envelope, `01 02 ‖ body`.
    pub fn to_envelope(&self) -> Vec<u8> {
        encode_object(ObjectKind::Changeset, &to_vec(self))
    }

    /// Decodes an envelope that must hold a changeset.
    pub fn from_envelope(bytes: &[u8]) -> Result<Self, DecodeError> {
        decode_as(bytes, ObjectKind::Changeset)
    }
}

impl Encode for Changeset {
    fn encode(&self, enc: &mut Encoder) {
        self.parents.encode(enc);
        enc.raw(&self.change_id);
        self.changes.encode(enc);
        enc.bytes(&self.description);
    }
}

impl Decode for Changeset {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let parents = Vec::decode(dec)?;
        let mut change_id = [0; CHANGE_ID_LEN];
        change_id.copy_from_slice(dec.take(CHANGE_ID_LEN)?);
        Ok(Changeset {
            parents,
            change_id,
            changes: BTreeMap::decode(dec)?,
            description: dec.bytes()?.to_vec(),
        })
    }
}
