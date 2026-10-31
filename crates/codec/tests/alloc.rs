//! Decoding holds memory only for elements it has read (format v1, Decoder
//! requirements: Bounded). `allocation-counter` installs a counting global
//! allocator, so these tests have a binary to themselves; it counts only the
//! calling thread, so tests running in parallel do not disturb each other.

use codec::{Decode, DecodeError, from_slice, to_vec};
use hash::{HashAlg, HashError, HashRef};

/// Elements a hostile input declares. The input carries as many bytes, so the
/// count passes the check against the remaining input.
const COUNT: u64 = 100_000;

/// The most heap a rejected hostile input may hold at once. Reserving room for
/// `COUNT` elements up front takes megabytes.
const MAX_PEAK: u64 = 1_024;

/// `COUNT` as a three-byte varint, then `COUNT` bytes of `filler`, so the
/// first element starts at offset 3.
fn hostile(filler: u8) -> Vec<u8> {
    let mut input = to_vec(&COUNT);
    input.resize(input.len() + COUNT as usize, filler);
    input
}

/// Decoding `input` as a `T` fails with `expected` while holding less than
/// `MAX_PEAK` bytes of heap at any moment.
fn rejected_cheaply<T: Decode>(input: &[u8], expected: DecodeError) {
    let mut result = None;
    let info = allocation_counter::measure(|| result = Some(from_slice::<T>(input).map(drop)));
    assert_eq!(result, Some(Err(expected)));
    assert!(
        info.bytes_max < MAX_PEAK,
        "decoding held {} bytes of heap",
        info.bytes_max
    );
}

#[test]
fn hostile_hash_count_reserves_nothing() {
    // A zero byte is not a hash algorithm id, so the first element fails.
    rejected_cheaply::<Vec<HashRef>>(
        &hostile(0x00),
        DecodeError::Hash {
            at: 3,
            error: HashError::UnknownAlgorithm(0),
        },
    );
}

#[test]
fn hostile_pair_count_reserves_nothing() {
    // Zero bytes would decode as pairs of empty strings. With 0xff the first
    // string's length prefix overflows, so the first element fails.
    rejected_cheaply::<Vec<(String, String)>>(
        &hostile(0xff),
        DecodeError::VarintOverflow { at: 3 },
    );
}

#[test]
fn valid_vec_grows_as_it_decodes() {
    let hashes: Vec<HashRef> = (0..3_000u32)
        .map(|i| HashAlg::Blake3.hash(&i.to_le_bytes()))
        .collect();
    assert_eq!(from_slice::<Vec<HashRef>>(&to_vec(&hashes)), Ok(hashes));
}
