//! The object envelope: `format_version ‖ kind ‖ body`, and object identity.

use hash::{HashAlg, HashRef};

use crate::error::DecodeError;
use crate::wire::{Decode, from_slice};

/// The format version this build writes and the only one it reads.
pub const FORMAT_VERSION: u8 = 1;

/// Kinds the registry names but whose bodies are not yet specified, as
/// (id, name). They are rejected until the spec defines them.
pub const RESERVED_KINDS: [(u8, &str); 1] = [(0x05, "event")];

/// An object kind whose body format v1 defines.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectKind {
    /// File content. The body is the raw bytes, with no length prefix.
    Blob,
    /// One version of a change, *provisional* until the Phase 1 exit gate;
    /// see [`crate::Changeset`].
    Changeset,
    /// An op-log entry, *provisional* until the Phase 1 exit gate; see
    /// [`crate::Operation`].
    Operation,
    /// The repository state an operation leaves, *provisional* until the
    /// Phase 1 exit gate; see [`crate::View`].
    View,
}

impl ObjectKind {
    /// Every defined kind, in id order.
    pub const ALL: [ObjectKind; 4] = [
        ObjectKind::Blob,
        ObjectKind::Changeset,
        ObjectKind::Operation,
        ObjectKind::View,
    ];

    /// The one-byte registry id.
    pub const fn id(self) -> u8 {
        match self {
            ObjectKind::Blob => 0x01,
            ObjectKind::Changeset => 0x02,
            ObjectKind::Operation => 0x03,
            ObjectKind::View => 0x04,
        }
    }

    /// The registry name.
    pub const fn name(self) -> &'static str {
        match self {
            ObjectKind::Blob => "blob",
            ObjectKind::Changeset => "changeset",
            ObjectKind::Operation => "operation",
            ObjectKind::View => "view",
        }
    }

    /// Looks up a registry id. Reserved ids and unassigned ids are distinct
    /// errors.
    pub fn from_id(kind: u8) -> Result<Self, DecodeError> {
        if let Some(defined) = Self::ALL.into_iter().find(|k| k.id() == kind) {
            return Ok(defined);
        }
        if RESERVED_KINDS.iter().any(|&(id, _)| id == kind) {
            return Err(DecodeError::ReservedObjectKind { kind });
        }
        Err(DecodeError::UnknownObjectKind { kind })
    }
}

/// A decoded envelope whose body still borrows the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Envelope<'a> {
    /// What the body holds.
    pub kind: ObjectKind,
    /// The kind-specific body.
    pub body: &'a [u8],
}

/// Wraps a body in a v1 envelope.
pub fn encode_object(kind: ObjectKind, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + body.len());
    out.push(FORMAT_VERSION);
    out.push(kind.id());
    out.extend_from_slice(body);
    out
}

/// Splits an envelope, checking its version and kind.
pub fn decode_envelope(bytes: &[u8]) -> Result<Envelope<'_>, DecodeError> {
    let [version, kind, body @ ..] = bytes else {
        return Err(DecodeError::UnexpectedEnd {
            at: bytes.len(),
            needed: 2 - bytes.len(),
        });
    };
    if *version != FORMAT_VERSION {
        return Err(DecodeError::UnsupportedFormatVersion { version: *version });
    }
    Ok(Envelope {
        kind: ObjectKind::from_id(*kind)?,
        body,
    })
}

/// Decodes an envelope that must hold `kind`, and its body as `T`. Offsets
/// in body errors are relative to the body.
pub(crate) fn decode_as<T: Decode>(bytes: &[u8], kind: ObjectKind) -> Result<T, DecodeError> {
    let envelope = decode_envelope(bytes)?;
    if envelope.kind != kind {
        return Err(DecodeError::UnexpectedObjectKind {
            expected: kind.id(),
            found: envelope.kind.id(),
        });
    }
    from_slice(envelope.body)
}

/// The envelope of a blob holding `content`.
pub fn blob(content: &[u8]) -> Vec<u8> {
    encode_object(ObjectKind::Blob, content)
}

/// An object's identity: the hash of its whole envelope, so version and kind
/// are part of what is hashed.
pub fn object_id(alg: HashAlg, envelope: &[u8]) -> HashRef {
    alg.hash(envelope)
}
