//! Canonical primitives: the encoder, the decoder and the `Encode`/`Decode`
//! implementations for every v1 type.

use std::collections::{BTreeMap, BTreeSet};

use hash::HashRef;

use crate::error::DecodeError;

/// Longest valid unsigned LEB128 encoding of a `u64`.
const MAX_VARINT_LEN: usize = 10;

/// An opaque byte string, encoded with a length prefix. `Vec<T>` encodes a
/// sequence of values, so raw bytes always go through this type.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes(pub Vec<u8>);

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Bytes(bytes)
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Bytes(bytes.to_vec())
    }
}

/// Builds a canonical encoding.
#[derive(Debug, Default)]
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    /// An empty encoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one raw byte.
    pub fn byte(&mut self, byte: u8) {
        self.buf.push(byte);
    }

    /// Appends raw bytes with no length prefix.
    pub fn raw(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// Appends a minimal unsigned LEB128 integer.
    pub fn uvarint(&mut self, mut value: u64) {
        loop {
            let low = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.buf.push(low);
                return;
            }
            self.buf.push(low | 0x80);
        }
    }

    /// Appends `0x00` or `0x01`.
    pub fn bool(&mut self, value: bool) {
        self.buf.push(u8::from(value));
    }

    /// Appends a length-prefixed byte string.
    pub fn bytes(&mut self, bytes: &[u8]) {
        self.uvarint(bytes.len() as u64);
        self.raw(bytes);
    }

    /// Appends a length-prefixed UTF-8 string.
    pub fn str(&mut self, text: &str) {
        self.bytes(text.as_bytes());
    }

    /// Appends a hash in its binary form.
    pub fn hash_ref(&mut self, hash: &HashRef) {
        hash.write_to(&mut self.buf);
    }

    /// The encoded bytes.
    pub fn finish(self) -> Vec<u8> {
        self.buf
    }
}

/// Reads a canonical encoding, rejecting every non-canonical form.
#[derive(Debug)]
pub struct Decoder<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    /// A decoder positioned at the start of `input`.
    pub fn new(input: &'a [u8]) -> Self {
        Decoder { input, pos: 0 }
    }

    /// Offset of the next unread byte.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Number of unread bytes.
    pub fn remaining(&self) -> usize {
        self.input.len() - self.pos
    }

    /// Reads one raw byte.
    pub fn byte(&mut self) -> Result<u8, DecodeError> {
        self.byte_of(self.pos)
    }

    /// Reads the next byte of a value that starts at `start`, so input that
    /// ends inside the value is reported where the value starts.
    fn byte_of(&mut self, start: usize) -> Result<u8, DecodeError> {
        let byte = *self.input.get(self.pos).ok_or(DecodeError::UnexpectedEnd {
            at: start,
            needed: 1,
        })?;
        self.pos += 1;
        Ok(byte)
    }

    /// Reads `len` raw bytes.
    pub fn take(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let remaining = self.remaining();
        if len > remaining {
            return Err(DecodeError::UnexpectedEnd {
                at: self.pos,
                needed: len - remaining,
            });
        }
        let bytes = &self.input[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }

    /// Reads every remaining byte.
    pub fn rest(&mut self) -> &'a [u8] {
        let bytes = &self.input[self.pos..];
        self.pos = self.input.len();
        bytes
    }

    /// Reads a minimal unsigned LEB128 integer.
    pub fn uvarint(&mut self) -> Result<u64, DecodeError> {
        let start = self.pos;
        let mut value = 0u64;
        for index in 0..MAX_VARINT_LEN {
            let byte = self.byte_of(start)?;
            // The tenth byte carries only bit 63, so anything above 0x01
            // either overflows or continues past ten bytes.
            if index == MAX_VARINT_LEN - 1 && byte > 0x01 {
                return Err(DecodeError::VarintOverflow { at: start });
            }
            value |= u64::from(byte & 0x7f) << (7 * index);
            if byte & 0x80 == 0 {
                if byte == 0 && index > 0 {
                    return Err(DecodeError::NonMinimalVarint { at: start });
                }
                return Ok(value);
            }
        }
        Err(DecodeError::VarintOverflow { at: start })
    }

    /// Reads a length or count and checks that the input could hold it.
    /// Every encoded value takes at least one byte, so a count larger than
    /// the remaining input is already known to be truncated.
    pub fn length(&mut self) -> Result<usize, DecodeError> {
        let at = self.pos;
        let len = self.uvarint()?;
        let remaining = self.remaining();
        match usize::try_from(len) {
            Ok(len) if len <= remaining => Ok(len),
            // `len` exceeds `remaining` here, so the subtraction cannot wrap.
            _ => Err(DecodeError::UnexpectedEnd {
                at,
                needed: usize::try_from(len - remaining as u64).unwrap_or(usize::MAX),
            }),
        }
    }

    /// Reads `0x00` or `0x01`.
    pub fn bool(&mut self) -> Result<bool, DecodeError> {
        let at = self.pos;
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            byte => Err(DecodeError::InvalidBool { at, byte }),
        }
    }

    /// Reads a length-prefixed byte string.
    pub fn bytes(&mut self) -> Result<&'a [u8], DecodeError> {
        let len = self.length()?;
        self.take(len)
    }

    /// Reads a length-prefixed UTF-8 string.
    pub fn str(&mut self) -> Result<&'a str, DecodeError> {
        let len = self.length()?;
        let at = self.pos;
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8 { at })
    }

    /// Reads a hash in its binary form.
    pub fn hash_ref(&mut self) -> Result<HashRef, DecodeError> {
        let at = self.pos;
        let (hash, rest) = HashRef::split_prefix(&self.input[self.pos..])
            .map_err(|error| DecodeError::Hash { at, error })?;
        self.pos = self.input.len() - rest.len();
        Ok(hash)
    }

    /// Succeeds only if every byte has been read.
    pub fn finish(self) -> Result<(), DecodeError> {
        match self.remaining() {
            0 => Ok(()),
            remaining => Err(DecodeError::TrailingBytes {
                at: self.pos,
                remaining,
            }),
        }
    }
}

/// A type with exactly one canonical encoding.
pub trait Encode {
    /// Appends the canonical encoding of `self`.
    fn encode(&self, enc: &mut Encoder);
}

/// A type that can be read back from its canonical encoding.
pub trait Decode: Sized {
    /// Reads one value, rejecting non-canonical input.
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError>;
}

impl Encode for u64 {
    fn encode(&self, enc: &mut Encoder) {
        enc.uvarint(*self);
    }
}

impl Decode for u64 {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        dec.uvarint()
    }
}

impl Encode for u32 {
    fn encode(&self, enc: &mut Encoder) {
        enc.uvarint(u64::from(*self));
    }
}

impl Decode for u32 {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let at = dec.position();
        u32::try_from(dec.uvarint()?).map_err(|_| DecodeError::IntegerOutOfRange { at })
    }
}

impl Encode for bool {
    fn encode(&self, enc: &mut Encoder) {
        enc.bool(*self);
    }
}

impl Decode for bool {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        dec.bool()
    }
}

impl Encode for str {
    fn encode(&self, enc: &mut Encoder) {
        enc.str(self);
    }
}

impl Encode for String {
    fn encode(&self, enc: &mut Encoder) {
        enc.str(self);
    }
}

impl Decode for String {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        dec.str().map(str::to_owned)
    }
}

impl Encode for Bytes {
    fn encode(&self, enc: &mut Encoder) {
        enc.bytes(&self.0);
    }
}

impl Decode for Bytes {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        dec.bytes().map(Bytes::from)
    }
}

impl Encode for HashRef {
    fn encode(&self, enc: &mut Encoder) {
        enc.hash_ref(self);
    }
}

impl Decode for HashRef {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        dec.hash_ref()
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, enc: &mut Encoder) {
        match self {
            None => enc.byte(0),
            Some(value) => {
                enc.byte(1);
                value.encode(enc);
            }
        }
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let at = dec.position();
        match dec.byte()? {
            0 => Ok(None),
            1 => T::decode(dec).map(Some),
            byte => Err(DecodeError::InvalidOptionTag { at, byte }),
        }
    }
}

impl<T: Encode> Encode for [T] {
    fn encode(&self, enc: &mut Encoder) {
        enc.uvarint(self.len() as u64);
        for item in self {
            item.encode(enc);
        }
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, enc: &mut Encoder) {
        self.as_slice().encode(enc);
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let count = dec.length()?;
        // Nothing is reserved from `count`: an element can take many times its
        // encoded size, so memory grows only with the elements actually read.
        let mut items = Vec::new();
        for _ in 0..count {
            items.push(T::decode(dec)?);
        }
        Ok(items)
    }
}

impl<T: Encode> Encode for BTreeSet<T> {
    fn encode(&self, enc: &mut Encoder) {
        enc.uvarint(self.len() as u64);
        for item in self {
            item.encode(enc);
        }
    }
}

impl<T: Decode + Ord> Decode for BTreeSet<T> {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let count = dec.length()?;
        let mut items = BTreeSet::new();
        for _ in 0..count {
            let at = dec.position();
            let item = T::decode(dec)?;
            if items.last().is_some_and(|last| &item <= last) {
                return Err(DecodeError::UnsortedOrDuplicateKey { at });
            }
            items.insert(item);
        }
        Ok(items)
    }
}

impl<K: Encode, V: Encode> Encode for BTreeMap<K, V> {
    fn encode(&self, enc: &mut Encoder) {
        enc.uvarint(self.len() as u64);
        for (key, value) in self {
            key.encode(enc);
            value.encode(enc);
        }
    }
}

impl<K: Decode + Ord, V: Decode> Decode for BTreeMap<K, V> {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let count = dec.length()?;
        let mut map = BTreeMap::new();
        for _ in 0..count {
            let at = dec.position();
            let key = K::decode(dec)?;
            if map.last_key_value().is_some_and(|(last, _)| &key <= last) {
                return Err(DecodeError::UnsortedOrDuplicateKey { at });
            }
            let value = V::decode(dec)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<A: Encode, B: Encode> Encode for (A, B) {
    fn encode(&self, enc: &mut Encoder) {
        self.0.encode(enc);
        self.1.encode(enc);
    }
}

impl<A: Decode, B: Decode> Decode for (A, B) {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok((A::decode(dec)?, B::decode(dec)?))
    }
}

/// Encodes one value.
pub fn to_vec<T: Encode + ?Sized>(value: &T) -> Vec<u8> {
    let mut enc = Encoder::new();
    value.encode(&mut enc);
    enc.finish()
}

/// Decodes exactly one value; trailing bytes are an error.
pub fn from_slice<T: Decode>(bytes: &[u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::new(bytes);
    let value = T::decode(&mut dec)?;
    dec.finish()?;
    Ok(value)
}
