//! Decoding holds memory only for elements it has read (format v1, Decoder
//! requirements: Bounded). `allocation-counter` installs a counting global
//! allocator, so these tests have a binary to themselves; it counts only the
//! calling thread, so tests running in parallel do not disturb each other.

use std::collections::{BTreeMap, BTreeSet};

use codec::{Decode, DecodeError, from_slice, to_vec};
use hash::{HashAlg, HashError, HashRef};

/// Elements a hostile input declares. The input carries as many bytes, so the
/// count passes the check against the remaining input.
const COUNT: u64 = 100_000;

/// The most heap a rejected hostile input may hold at once. Reserving room for
/// `COUNT` elements up front takes megabytes.
const MAX_PEAK: u64 = 1_024;

/// `COUNT` as a three-byte varint, then `first`, then `filler` up to `COUNT`
/// bytes after the varint, so `first` starts at offset 3.
fn hostile_after(first: &[u8], filler: u8) -> Vec<u8> {
    let mut input = to_vec(&COUNT);
    let end = input.len() + COUNT as usize;
    input.extend_from_slice(first);
    input.resize(end, filler);
    input
}

/// `COUNT` as a three-byte varint, then `COUNT` bytes of `filler`, so the
/// first element starts at offset 3.
fn hostile(filler: u8) -> Vec<u8> {
    hostile_after(&[], filler)
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
    // A zero byte is not a hash algorithm id, so the first element fails, in
    // a vector, a set or a map.
    let unknown = |at| DecodeError::Hash {
        at,
        error: HashError::UnknownAlgorithm(0),
    };
    rejected_cheaply::<Vec<HashRef>>(&hostile(0x00), unknown(3));
    rejected_cheaply::<BTreeSet<HashRef>>(&hostile(0x00), unknown(3));
    rejected_cheaply::<BTreeMap<HashRef, HashRef>>(&hostile(0x00), unknown(3));
    // After one valid 33-byte element (a key and a value in the map), the
    // next fails: nothing is reserved once an element has decoded either.
    let hash = to_vec(&HashAlg::Blake3.hash(b""));
    rejected_cheaply::<Vec<HashRef>>(&hostile_after(&hash, 0x00), unknown(36));
    rejected_cheaply::<BTreeSet<HashRef>>(&hostile_after(&hash, 0x00), unknown(36));
    rejected_cheaply::<BTreeMap<HashRef, HashRef>>(
        &hostile_after(&[&hash[..], &hash].concat(), 0x00),
        unknown(69),
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
    // One pair of empty strings first: the second pair fails the same way.
    rejected_cheaply::<Vec<(String, String)>>(
        &hostile_after(&[0x00, 0x00], 0xff),
        DecodeError::VarintOverflow { at: 5 },
    );
}

#[test]
fn hostile_small_element_count_reserves_nothing() {
    // Elements of a few bytes are no exception. Ten 0x80 bytes run past the
    // longest varint, and 0x02 is not a bool, so the first element fails.
    rejected_cheaply::<Vec<u64>>(&hostile(0x80), DecodeError::VarintOverflow { at: 3 });
    rejected_cheaply::<Vec<bool>>(
        &hostile(0x02),
        DecodeError::InvalidBool { at: 3, byte: 0x02 },
    );
}

/// What a decoder that trusts a hostile count would do: reserve room for
/// every declared element, then fail on the first one.
fn reserve_then_fail(count: u64) -> Result<Vec<u64>, DecodeError> {
    let mut elements = Vec::with_capacity(count as usize);
    if count > 1 {
        return Err(DecodeError::VarintOverflow { at: 3 });
    }
    elements.push(count);
    Ok(elements)
}

#[test]
fn a_reservation_on_the_error_path_is_measured() {
    // The negative control for `rejected_cheaply` (Sprint 5 T-002). Tests now
    // build optimised, and an optimiser may delete an allocation that is never
    // used. If it deleted this one, a reserving decoder would pass every test
    // above. No `black_box` or other barrier is used, so the control is
    // optimised exactly as a decoder would be.
    let mut result = None;
    let info = allocation_counter::measure(|| result = Some(reserve_then_fail(COUNT)));
    assert_eq!(result, Some(Err(DecodeError::VarintOverflow { at: 3 })));
    assert!(
        info.bytes_max >= COUNT * 8,
        "reserving {COUNT} elements held only {} bytes of heap",
        info.bytes_max
    );
}

#[test]
fn valid_vec_grows_as_it_decodes() {
    let hashes: Vec<HashRef> = (0..3_000u32)
        .map(|i| HashAlg::Blake3.hash(&i.to_le_bytes()))
        .collect();
    let input = to_vec(&hashes);
    let mut result = None;
    let info = allocation_counter::measure(|| result = Some(from_slice::<Vec<HashRef>>(&input)));
    assert_eq!(result, Some(Ok(hashes)));
    // Sized from its count, the vector takes one allocation, or two if the
    // first element comes first. Grown as its elements decode, it takes a new
    // one each time it fills.
    assert!(info.count_total > 2, "{info:?}");
}
