#![forbid(unsafe_code)]
//! Algorithm-tagged content hashes for Periapsis format v1.
//!
//! Every hash Periapsis stores or sends is a [`HashRef`]: a one-byte algorithm
//! id followed by the digest, so a repository can move to a new algorithm
//! without ambiguity. The registry and both wire forms are specified in
//! `docs/spec/format-v1.md`.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// Length in bytes of every digest in the v1 registry.
pub const DIGEST_LEN: usize = 32;

/// A hash algorithm from the format v1 registry.
///
/// Variants are declared in id order, so the derived `Ord` sorts by id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HashAlg {
    /// BLAKE3 with a 256-bit output. The default.
    Blake3,
    /// SHA-256 (FIPS 180-4).
    Sha256,
}

impl HashAlg {
    /// The algorithm new repositories use.
    pub const DEFAULT: HashAlg = HashAlg::Blake3;

    /// Every registered algorithm, in id order.
    pub const ALL: [HashAlg; 2] = [HashAlg::Blake3, HashAlg::Sha256];

    /// The one-byte registry id.
    pub const fn id(self) -> u8 {
        match self {
            HashAlg::Blake3 => 0x01,
            HashAlg::Sha256 => 0x02,
        }
    }

    /// Looks up a registry id.
    pub fn from_id(id: u8) -> Result<Self, HashError> {
        Self::ALL
            .into_iter()
            .find(|alg| alg.id() == id)
            .ok_or(HashError::UnknownAlgorithm(id))
    }

    /// The name used in the text form.
    pub const fn name(self) -> &'static str {
        match self {
            HashAlg::Blake3 => "blake3",
            HashAlg::Sha256 => "sha256",
        }
    }

    /// Looks up a text-form name.
    pub fn from_name(name: &str) -> Result<Self, HashError> {
        Self::ALL
            .into_iter()
            .find(|alg| alg.name() == name)
            .ok_or_else(|| HashError::UnknownAlgorithmName(name.to_owned()))
    }

    /// Digest length in bytes.
    pub const fn digest_len(self) -> usize {
        DIGEST_LEN
    }

    /// Hashes `data` in one call.
    pub fn hash(self, data: &[u8]) -> HashRef {
        let mut hasher = self.hasher();
        hasher.update(data);
        hasher.finalize()
    }

    /// Starts an incremental hash.
    pub fn hasher(self) -> Hasher {
        let inner = match self {
            HashAlg::Blake3 => Inner::Blake3(Box::new(blake3::Hasher::new())),
            HashAlg::Sha256 => Inner::Sha256(<sha2::Sha256 as sha2::Digest>::new()),
        };
        Hasher { inner }
    }
}

/// An incremental hasher for one registered algorithm.
pub struct Hasher {
    inner: Inner,
}

enum Inner {
    Blake3(Box<blake3::Hasher>),
    Sha256(sha2::Sha256),
}

impl Hasher {
    /// Feeds more input.
    pub fn update(&mut self, data: &[u8]) {
        match &mut self.inner {
            Inner::Blake3(hasher) => {
                hasher.update(data);
            }
            Inner::Sha256(hasher) => sha2::Digest::update(hasher, data),
        }
    }

    /// Finishes the hash.
    pub fn finalize(self) -> HashRef {
        let mut digest = [0u8; DIGEST_LEN];
        let alg = match self.inner {
            Inner::Blake3(hasher) => {
                digest.copy_from_slice(hasher.finalize().as_bytes());
                HashAlg::Blake3
            }
            Inner::Sha256(hasher) => {
                digest.copy_from_slice(&sha2::Digest::finalize(hasher));
                HashAlg::Sha256
            }
        };
        HashRef { alg, digest }
    }
}

/// A digest tagged with the algorithm that produced it.
///
/// Ordering is by algorithm id, then by digest bytes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HashRef {
    alg: HashAlg,
    digest: [u8; DIGEST_LEN],
}

impl HashRef {
    /// Length of the binary form: one id byte plus the digest.
    pub const ENCODED_LEN: usize = 1 + DIGEST_LEN;

    /// Wraps a digest, checking its length against the algorithm.
    pub fn new(alg: HashAlg, digest: &[u8]) -> Result<Self, HashError> {
        if digest.len() != alg.digest_len() {
            return Err(HashError::DigestLength {
                alg,
                expected: alg.digest_len(),
                actual: digest.len(),
            });
        }
        let mut bytes = [0u8; DIGEST_LEN];
        bytes.copy_from_slice(digest);
        Ok(HashRef { alg, digest: bytes })
    }

    /// The algorithm that produced the digest.
    pub fn alg(&self) -> HashAlg {
        self.alg
    }

    /// The digest bytes.
    pub fn digest(&self) -> &[u8] {
        &self.digest[..self.alg.digest_len()]
    }

    /// Appends the binary form, `id ‖ digest`.
    pub fn write_to(&self, out: &mut Vec<u8>) {
        out.push(self.alg.id());
        out.extend_from_slice(self.digest());
    }

    /// The binary form, `id ‖ digest`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::ENCODED_LEN);
        self.write_to(&mut out);
        out
    }

    /// Parses a binary form from the front of `bytes`, returning the rest.
    pub fn split_prefix(bytes: &[u8]) -> Result<(Self, &[u8]), HashError> {
        let (&id, rest) = bytes.split_first().ok_or(HashError::Truncated {
            needed: 1,
            available: 0,
        })?;
        let alg = HashAlg::from_id(id)?;
        let len = alg.digest_len();
        if rest.len() < len {
            return Err(HashError::Truncated {
                needed: len,
                available: rest.len(),
            });
        }
        let (digest, rest) = rest.split_at(len);
        Ok((HashRef::new(alg, digest)?, rest))
    }

    /// Parses exactly one binary form.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        let (hash, rest) = Self::split_prefix(bytes)?;
        if !rest.is_empty() {
            return Err(HashError::TrailingBytes { extra: rest.len() });
        }
        Ok(hash)
    }
}

impl fmt::Display for HashRef {
    /// The text form: `<name>:<lowercase hex digest>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.alg.name())?;
        f.write_str(":")?;
        write_hex(f, self.digest())
    }
}

/// Writes `bytes` to `out` as lowercase hex, two digits for each byte, in
/// order. The one definition of the hex form of a string of bytes.
pub fn write_hex(out: &mut impl fmt::Write, bytes: &[u8]) -> fmt::Result {
    for byte in bytes {
        write!(out, "{byte:02x}")?;
    }
    Ok(())
}

/// `bytes` as lowercase hex, two digits for each byte, in order.
pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    // Writing to a `String` cannot fail.
    let _ = write_hex(&mut out, bytes);
    out
}

impl fmt::Debug for HashRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashRef({self})")
    }
}

impl FromStr for HashRef {
    type Err = HashError;

    /// Parses the text form. Only lowercase hex is accepted, so every value
    /// has exactly one text form.
    fn from_str(text: &str) -> Result<Self, HashError> {
        let (name, hex) = text.split_once(':').ok_or(HashError::MissingSeparator)?;
        let alg = HashAlg::from_name(name)?;
        let digest = decode_lower_hex(hex)?;
        HashRef::new(alg, &digest)
    }
}

fn decode_lower_hex(hex: &str) -> Result<Vec<u8>, HashError> {
    fn nibble(c: u8) -> Result<u8, HashError> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            _ => Err(HashError::InvalidHex),
        }
    }
    let (pairs, odd) = hex.as_bytes().as_chunks::<2>();
    if !odd.is_empty() {
        return Err(HashError::InvalidHex);
    }
    pairs
        .iter()
        .map(|&[high, low]| Ok((nibble(high)? << 4) | nibble(low)?))
        .collect()
}

/// Why a hash could not be built or parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HashError {
    /// The id byte is not in the registry.
    UnknownAlgorithm(u8),
    /// The text-form name is not in the registry.
    UnknownAlgorithmName(String),
    /// The digest length does not match the algorithm.
    DigestLength {
        alg: HashAlg,
        expected: usize,
        actual: usize,
    },
    /// The input ended inside a binary hash.
    Truncated { needed: usize, available: usize },
    /// Bytes followed a complete binary hash.
    TrailingBytes { extra: usize },
    /// The text form has no `:` separator.
    MissingSeparator,
    /// The text form's digest is not lowercase hex with an even length.
    InvalidHex,
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashError::UnknownAlgorithm(id) => write!(f, "unknown hash algorithm id 0x{id:02x}"),
            HashError::UnknownAlgorithmName(name) => {
                write!(f, "unknown hash algorithm name {name:?}")
            }
            HashError::DigestLength {
                alg,
                expected,
                actual,
            } => write!(
                f,
                "{} digest must be {expected} bytes, got {actual}",
                alg.name()
            ),
            HashError::Truncated { needed, available } => {
                write!(
                    f,
                    "hash truncated: needed {needed} bytes, {available} available"
                )
            }
            HashError::TrailingBytes { extra } => {
                write!(f, "{extra} unexpected bytes after the hash")
            }
            HashError::MissingSeparator => f.write_str("hash text has no ':' separator"),
            HashError::InvalidHex => f.write_str("hash digest is not lowercase hex"),
        }
    }
}

impl Error for HashError {}
