//! Decoding errors. A variant that carries `at` gives the absolute byte offset
//! where the faulty value starts (for a string that is not UTF-8, its first
//! content byte), so a rejected object can be located exactly. The exceptions,
//! `UnsupportedFormatVersion`, `ReservedObjectKind`, `UnknownObjectKind` and
//! `UnexpectedObjectKind`, carry no offset: they name envelope-header fields at fixed positions, the
//! format version at byte 0 and the kind at byte 1.

use std::error::Error;
use std::fmt;

use hash::HashError;

/// Why a byte string is not a canonical encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The input ended before a value was complete. `at` is the absolute
    /// offset where that value starts, and `needed` is the least number of
    /// further bytes the input must have: 1 for a missing byte or a cut-off
    /// varint, and for a length, count or `Decoder::take` larger than the
    /// remaining input, how far it exceeds it (saturating at `usize::MAX`).
    UnexpectedEnd { at: usize, needed: usize },
    /// A varint has a redundant zero final group.
    NonMinimalVarint { at: usize },
    /// A varint is longer than ten bytes or exceeds `u64::MAX`.
    VarintOverflow { at: usize },
    /// An integer does not fit the type being decoded.
    IntegerOutOfRange { at: usize },
    /// A bool byte other than `0x00` or `0x01`.
    InvalidBool { at: usize, byte: u8 },
    /// An option tag other than `0x00` or `0x01`.
    InvalidOptionTag { at: usize, byte: u8 },
    /// A string is not valid UTF-8.
    InvalidUtf8 { at: usize },
    /// A set element or map key is not strictly greater than the previous one.
    UnsortedOrDuplicateKey { at: usize },
    /// Bytes remain after a complete value.
    TrailingBytes { at: usize, remaining: usize },
    /// A hash is malformed.
    Hash { at: usize, error: HashError },
    /// An object envelope names a format version this build does not know.
    UnsupportedFormatVersion { version: u8 },
    /// An object envelope names a reserved kind whose body is not yet specified.
    ReservedObjectKind { kind: u8 },
    /// An object envelope names an invalid or unassigned kind.
    UnknownObjectKind { kind: u8 },
    /// An envelope holds a defined kind other than the one the caller decodes.
    UnexpectedObjectKind { expected: u8, found: u8 },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnexpectedEnd { at, needed } => {
                write!(
                    f,
                    "value at byte {at} is truncated; {needed} more bytes needed"
                )
            }
            DecodeError::NonMinimalVarint { at } => {
                write!(f, "non-minimal varint at byte {at}")
            }
            DecodeError::VarintOverflow { at } => write!(f, "varint overflows u64 at byte {at}"),
            DecodeError::IntegerOutOfRange { at } => {
                write!(f, "integer at byte {at} is out of range for its type")
            }
            DecodeError::InvalidBool { at, byte } => {
                write!(f, "invalid bool 0x{byte:02x} at byte {at}")
            }
            DecodeError::InvalidOptionTag { at, byte } => {
                write!(f, "invalid option tag 0x{byte:02x} at byte {at}")
            }
            DecodeError::InvalidUtf8 { at } => write!(f, "invalid UTF-8 in string at byte {at}"),
            DecodeError::UnsortedOrDuplicateKey { at } => {
                write!(f, "key at byte {at} is not greater than the previous key")
            }
            DecodeError::TrailingBytes { at, remaining } => {
                write!(
                    f,
                    "{remaining} unexpected bytes after the value, from byte {at}"
                )
            }
            DecodeError::Hash { at, error } => write!(f, "bad hash at byte {at}: {error}"),
            DecodeError::UnsupportedFormatVersion { version } => {
                write!(f, "unsupported format version {version}")
            }
            DecodeError::ReservedObjectKind { kind } => {
                write!(
                    f,
                    "object kind 0x{kind:02x} is reserved and not yet specified"
                )
            }
            DecodeError::UnknownObjectKind { kind } => {
                write!(f, "unknown object kind 0x{kind:02x}")
            }
            DecodeError::UnexpectedObjectKind { expected, found } => {
                write!(
                    f,
                    "expected object kind 0x{expected:02x}, found 0x{found:02x}"
                )
            }
        }
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DecodeError::Hash { error, .. } => Some(error),
            _ => None,
        }
    }
}
