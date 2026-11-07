#![forbid(unsafe_code)]
//! Canonical binary encoding for Periapsis format v1.
//!
//! Every value has exactly one encoding, and the decoder rejects everything
//! else, so `decode(encode(v)) == v` and `encode(decode(b)) == b` both hold.
//! The rules are specified in `docs/spec/format-v1.md`.

mod changeset;
mod envelope;
mod error;
mod operation;
mod path;
mod references;
mod view;
mod wire;

pub use changeset::{CHANGE_ID_LEN, Changeset, Entry, Mode};
pub use envelope::{
    Envelope, FORMAT_VERSION, ObjectKind, RESERVED_KINDS, blob, decode_envelope, encode_object,
    object_id,
};
pub use error::DecodeError;
pub use operation::Operation;
pub use path::{PathError, RepoPath};
pub use references::references;
pub use view::View;
pub use wire::{Bytes, Decode, Decoder, Encode, Encoder, from_slice, to_vec};
