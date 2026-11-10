//! On-disk names of objects and ops (INT-0007 Consequences).
//!
//! The `HashRef` text form `<name>:<hex>` cannot be a file name: on NTFS a
//! colon names an alternate data stream. A file is named
//! `<algorithm id as two hex digits>-<digest hex>`, which keeps equal digests
//! under different algorithms apart and is legal on Windows and Linux.

use hash::{HashAlg, HashRef, write_hex};

/// The prefix of a file being written, before it is renamed to its id.
pub const TMP_PREFIX: &str = "tmp-";

/// The file name of an object or op with this id.
pub fn file_name(id: &HashRef) -> String {
    let mut name = format!("{:02x}-", id.alg().id());
    // Writing to a `String` cannot fail.
    let _ = write_hex(&mut name, id.digest());
    name
}

/// The id a file name stands for, if it is one: exactly the form
/// [`file_name`] writes, lowercase hex included.
pub fn parse_name(name: &str) -> Option<HashRef> {
    let (alg, digest) = name.split_once('-')?;
    if alg.len() != 2 || !is_lower_hex(alg) || !is_lower_hex(digest) || digest.len() % 2 != 0 {
        return None;
    }
    let alg = HashAlg::from_id(u8::from_str_radix(alg, 16).ok()?).ok()?;
    let bytes: Option<Vec<u8>> = (0..digest.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&digest[i..i + 2], 16).ok())
        .collect();
    HashRef::new(alg, &bytes?).ok()
}

fn is_lower_hex(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
