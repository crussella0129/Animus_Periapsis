//! Canonicality and totality of the v1 encoding (INT-0006 AC-1, AC-5).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

use codec::{
    Bytes, Decode, DecodeError, Decoder, Encode, ObjectKind, RESERVED_KINDS, blob, decode_envelope,
    encode_object, from_slice, object_id, to_vec,
};
use hash::{HashAlg, HashRef};
use proptest::prelude::*;

type Nested = BTreeMap<String, BTreeMap<u64, Option<Vec<Bytes>>>>;

/// Cases per property: every type gets its own 10,000.
const CASES: u32 = 10_000;

/// Random byte strings decoded as every type (AC-5).
const TOTALITY_CASES: u32 = 100_000;

#[test]
fn case_counts_are_not_overridden() {
    // proptest lets PROPTEST_CASES replace a configured count, which would let
    // the properties pass on fewer cases than the record states.
    for cases in [CASES, TOTALITY_CASES] {
        let config = proptest::test_runner::contextualize_config(ProptestConfig::with_cases(cases));
        assert_eq!(config.cases, cases, "unset PROPTEST_CASES to run the suite");
    }
}

fn any_hash() -> impl Strategy<Value = HashRef> {
    (
        prop::sample::select(HashAlg::ALL.to_vec()),
        any::<[u8; 32]>(),
    )
        .prop_map(|(alg, digest)| HashRef::new(alg, &digest).unwrap())
}

fn any_bytes() -> impl Strategy<Value = Bytes> {
    prop::collection::vec(any::<u8>(), 0..64).prop_map(Bytes)
}

fn any_nested() -> impl Strategy<Value = Nested> {
    prop::collection::btree_map(
        ".{0,8}",
        prop::collection::btree_map(
            any::<u64>(),
            prop::option::of(prop::collection::vec(any_bytes(), 0..3)),
            0..3,
        ),
        0..3,
    )
}

/// A valid encoding with one byte flipped, inserted or removed.
fn mutate(mut bytes: Vec<u8>, op: u8, index: prop::sample::Index, byte: u8) -> Vec<u8> {
    match op % 3 {
        0 if !bytes.is_empty() => {
            let i = index.index(bytes.len());
            bytes[i] ^= byte | 1;
        }
        1 => {
            let i = index.index(bytes.len() + 1);
            bytes.insert(i, byte);
        }
        _ if !bytes.is_empty() => {
            let i = index.index(bytes.len());
            bytes.remove(i);
        }
        _ => bytes.push(byte),
    }
    bytes
}

fn round_trips<T: Encode + Decode + PartialEq + Debug>(value: &T) -> Result<(), TestCaseError> {
    let bytes = to_vec(value);
    prop_assert_eq!(&from_slice::<T>(&bytes).unwrap(), value);
    Ok(())
}

/// If a mutated encoding still decodes, it must be that value's canonical
/// encoding: the decoder accepts nothing else.
fn reencodes<T: Encode + Decode>(bytes: &[u8]) -> Result<(), TestCaseError> {
    if let Ok(value) = from_slice::<T>(bytes) {
        prop_assert_eq!(to_vec(&value), bytes);
    }
    Ok(())
}

/// One round-trip and one mutation property per type, each with its own
/// `CASES` cases.
macro_rules! canonical_properties {
    ($($round_trip:ident, $mutants:ident: $ty:ty = $strategy:expr;)*) => {
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(CASES))]
            $(
                #[test]
                fn $round_trip(value in $strategy) {
                    round_trips::<$ty>(&value)?;
                }

                #[test]
                fn $mutants(
                    value in $strategy,
                    op in any::<u8>(),
                    index in any::<prop::sample::Index>(),
                    byte in any::<u8>(),
                ) {
                    reencodes::<$ty>(&mutate(to_vec(&value), op, index, byte))?;
                }
            )*
        }
    };
}

canonical_properties! {
    round_trip_u64, mutants_u64: u64 = any::<u64>();
    round_trip_u32, mutants_u32: u32 = any::<u32>();
    round_trip_bool, mutants_bool: bool = any::<bool>();
    round_trip_string, mutants_string: String = ".{0,24}";
    round_trip_bytes, mutants_bytes: Bytes = any_bytes();
    round_trip_hash_ref, mutants_hash_ref: HashRef = any_hash();
    round_trip_option, mutants_option: Option<u64> = prop::option::of(any::<u64>());
    round_trip_vec, mutants_vec: Vec<u32> = prop::collection::vec(any::<u32>(), 0..8);
    round_trip_set, mutants_set: BTreeSet<HashRef> = prop::collection::btree_set(any_hash(), 0..4);
    round_trip_map, mutants_map: BTreeMap<String, bool> =
        prop::collection::btree_map(".{0,8}", any::<bool>(), 0..6);
    round_trip_pair, mutants_pair: (String, Option<u64>) =
        (".{0,8}", prop::option::of(any::<u64>()));
    round_trip_nested, mutants_nested: Nested = any_nested();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn round_trip_envelope(body in prop::collection::vec(any::<u8>(), 0..128)) {
        let envelope = blob(&body);
        let decoded = decode_envelope(&envelope).unwrap();
        prop_assert_eq!(decoded.kind, ObjectKind::Blob);
        prop_assert_eq!(decoded.body, &body[..]);
        prop_assert_eq!(encode_object(decoded.kind, decoded.body), envelope);
    }

    #[test]
    fn mutants_envelope(
        body in prop::collection::vec(any::<u8>(), 0..128),
        op in any::<u8>(),
        index in any::<prop::sample::Index>(),
        byte in any::<u8>(),
    ) {
        let mutant = mutate(blob(&body), op, index, byte);
        if let Ok(envelope) = decode_envelope(&mutant) {
            prop_assert_eq!(encode_object(envelope.kind, envelope.body), mutant);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(TOTALITY_CASES))]

    #[test]
    fn decode_is_total_on_random_bytes(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        // Each call must return, Ok or Err, without panicking.
        let _ = from_slice::<u64>(&bytes);
        let _ = from_slice::<u32>(&bytes);
        let _ = from_slice::<bool>(&bytes);
        let _ = from_slice::<String>(&bytes);
        let _ = from_slice::<Bytes>(&bytes);
        let _ = from_slice::<HashRef>(&bytes);
        let _ = from_slice::<Option<u64>>(&bytes);
        let _ = from_slice::<Vec<u32>>(&bytes);
        let _ = from_slice::<BTreeSet<HashRef>>(&bytes);
        let _ = from_slice::<BTreeMap<String, bool>>(&bytes);
        let _ = from_slice::<(String, Option<u64>)>(&bytes);
        let _ = from_slice::<Nested>(&bytes);
        let _ = decode_envelope(&bytes);
    }
}

#[test]
fn varint_known_encodings() {
    let cases: [(u64, &[u8]); 5] = [
        (0, &[0x00]),
        (127, &[0x7f]),
        (128, &[0x80, 0x01]),
        (300, &[0xac, 0x02]),
        (
            u64::MAX,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
    ];
    for (value, bytes) in cases {
        assert_eq!(to_vec(&value), bytes, "{value}");
        assert_eq!(from_slice::<u64>(bytes), Ok(value));
    }
}

/// A two-element `Vec` whose first element is `first` and whose second
/// element's bytes are `second`; the second element starts at offset 2.
fn second_element(first: u8, second: &[u8]) -> Vec<u8> {
    let mut input = vec![0x02, first];
    input.extend_from_slice(second);
    input
}

#[test]
fn varint_rejects_non_canonical() {
    assert_eq!(
        from_slice::<Vec<u64>>(&second_element(0x01, &[0x80, 0x00])),
        Err(DecodeError::NonMinimalVarint { at: 2 })
    );
    assert_eq!(
        from_slice::<Vec<u64>>(&second_element(0x01, &[0xff; 11])),
        Err(DecodeError::VarintOverflow { at: 2 })
    );
    let mut tenth = [0xffu8; 10];
    tenth[9] = 0x02;
    assert_eq!(
        from_slice::<Vec<u64>>(&second_element(0x01, &tenth)),
        Err(DecodeError::VarintOverflow { at: 2 })
    );
}

#[test]
fn truncated_varint_located_at_its_start() {
    // The input ends inside the second element's varint, which starts at 2.
    for cut in [&[0x80][..], &[0xff, 0xff, 0xff]] {
        assert_eq!(
            from_slice::<Vec<u64>>(&second_element(0x01, cut)),
            Err(DecodeError::UnexpectedEnd { at: 2, needed: 1 })
        );
    }
    assert_eq!(
        from_slice::<u64>(&[0x80]),
        Err(DecodeError::UnexpectedEnd { at: 0, needed: 1 })
    );
    assert_eq!(
        from_slice::<Bytes>(&[0x80]),
        Err(DecodeError::UnexpectedEnd { at: 0, needed: 1 })
    );
}

#[test]
fn u32_range_is_enforced() {
    assert_eq!(
        from_slice::<u32>(&[0x80, 0x80, 0x80, 0x80, 0x10]),
        Err(DecodeError::IntegerOutOfRange { at: 0 })
    );
    assert_eq!(
        from_slice::<u32>(&[0xff, 0xff, 0xff, 0xff, 0x0f]),
        Ok(u32::MAX)
    );
    assert_eq!(to_vec(&u32::MAX), [0xff, 0xff, 0xff, 0xff, 0x0f]);
}

#[test]
fn decode_rejects_invalid_primitives() {
    assert_eq!(
        from_slice::<Vec<bool>>(&second_element(0x00, &[0x02])),
        Err(DecodeError::InvalidBool { at: 2, byte: 2 })
    );
    assert_eq!(
        from_slice::<Vec<Option<u64>>>(&second_element(0x00, &[0x02, 0x00])),
        Err(DecodeError::InvalidOptionTag { at: 2, byte: 2 })
    );
    // The second string's length prefix is at 2, its first content byte at 3.
    assert_eq!(
        from_slice::<Vec<String>>(&second_element(0x00, &[0x01, 0xff])),
        Err(DecodeError::InvalidUtf8 { at: 3 })
    );
    // The error is at the first content byte, not at the first invalid byte
    // (4 here)...
    assert_eq!(
        from_slice::<Vec<String>>(&second_element(0x00, &[0x02, 0x61, 0xff])),
        Err(DecodeError::InvalidUtf8 { at: 3 })
    );
    // ...and not one past the prefix's start: a two-byte prefix at 2 puts the
    // content at 4.
    let mut long = vec![0x80, 0x01];
    long.extend_from_slice(&[0xff; 128]);
    assert_eq!(
        from_slice::<Vec<String>>(&second_element(0x00, &long)),
        Err(DecodeError::InvalidUtf8 { at: 4 })
    );
}

#[test]
fn oversize_prefixes_rejected_without_allocation() {
    // The second element's length or count is the ten-byte varint for
    // u64::MAX, followed by three bytes. Reserving that much up front would
    // panic with a capacity overflow, so a pass also shows that nothing is
    // allocated before the prefix is checked against the remaining input.
    let mut oversize = vec![0xff; 9];
    oversize.extend_from_slice(&[0x01, 0x61, 0x62, 0x63]);
    let input = second_element(0x00, &oversize);
    let at_prefix = |result: Result<(), DecodeError>| {
        assert!(
            matches!(result, Err(DecodeError::UnexpectedEnd { at: 2, .. })),
            "{result:?}"
        );
    };
    at_prefix(from_slice::<Vec<Bytes>>(&input).map(drop));
    at_prefix(from_slice::<Vec<String>>(&input).map(drop));
    at_prefix(from_slice::<Vec<Vec<u64>>>(&input).map(drop));
    at_prefix(from_slice::<Vec<BTreeSet<u64>>>(&input).map(drop));
    at_prefix(from_slice::<Vec<BTreeMap<u64, u64>>>(&input).map(drop));
}

#[test]
fn overrun_reports_bytes_still_needed() {
    // Five bytes declared and three present: two more are needed.
    let short = from_slice::<Bytes>(&[0x05, 0x01, 0x02, 0x03]);
    assert_eq!(short, Err(DecodeError::UnexpectedEnd { at: 0, needed: 2 }));
    assert_eq!(
        short.unwrap_err().to_string(),
        "value at byte 0 is truncated; 2 more bytes needed"
    );
    // The second element's prefix is at 2, and one of its five bytes is present.
    assert_eq!(
        from_slice::<Vec<Bytes>>(&second_element(0x00, &[0x05, 0x01])),
        Err(DecodeError::UnexpectedEnd { at: 2, needed: 4 })
    );
    // `take` from offset 1 with two bytes left: three of its five are missing.
    let mut dec = Decoder::new(&[0x01, 0x02, 0x03]);
    dec.byte().unwrap();
    assert_eq!(
        dec.take(5),
        Err(DecodeError::UnexpectedEnd { at: 1, needed: 3 })
    );
}

#[test]
fn maps_and_sets_require_strictly_increasing_keys() {
    // Two entries, "b" then "a": the second key starts at offset 4.
    let descending = [0x02, 0x01, b'b', 0x00, 0x01, b'a', 0x00];
    assert_eq!(
        from_slice::<BTreeMap<String, bool>>(&descending),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 4 })
    );
    let duplicate = [0x02, 0x01, b'a', 0x00, 0x01, b'a', 0x01];
    assert_eq!(
        from_slice::<BTreeMap<String, bool>>(&duplicate),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 4 })
    );
    let set = [0x02, 0x05, 0x05];
    assert_eq!(
        from_slice::<BTreeSet<u64>>(&set),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 2 })
    );
}

#[test]
fn trailing_bytes_rejected() {
    let mut bytes = to_vec(&42u64);
    bytes.push(0);
    assert_eq!(
        from_slice::<u64>(&bytes),
        Err(DecodeError::TrailingBytes {
            at: 1,
            remaining: 1,
        })
    );
}

/// Every version and kind byte, each with an empty and a one-byte body. The
/// expected results are the spec's version rule and kind table written out by
/// hand, not taken from the code they check, and the class counts fail a
/// sweep that skips ids.
#[test]
fn envelope_rejects_unknown_version_and_kind() {
    let blob = blob(b"content");
    assert_eq!(decode_envelope(&blob).unwrap().kind, ObjectKind::Blob);
    assert_eq!(decode_envelope(&blob).unwrap().body, b"content");

    let mut versions = BTreeMap::new();
    for version in 0..=u8::MAX {
        let (class, expected) = match version {
            0x01 => ("supported", Ok(ObjectKind::Blob)),
            _ => (
                "unsupported",
                Err(DecodeError::UnsupportedFormatVersion { version }),
            ),
        };
        for envelope in [&[version, 0x01][..], &[version, 0x01, 0x00]] {
            let decoded = decode_envelope(envelope).map(|envelope| envelope.kind);
            assert_eq!(
                decoded, expected,
                "version {version:#04x}, envelope {envelope:02x?}"
            );
        }
        *versions.entry(class).or_insert(0) += 1;
    }
    assert_eq!(
        versions,
        BTreeMap::from([("supported", 1), ("unsupported", 255)])
    );

    // `from_id` is the lookup the decoder uses, so it must agree id for id.
    let mut kinds = BTreeMap::new();
    for kind in 0..=u8::MAX {
        let (class, expected) = match kind {
            0x01 => ("defined", Ok(ObjectKind::Blob)),
            0x02..=0x05 => ("reserved", Err(DecodeError::ReservedObjectKind { kind })),
            // 0x00 is invalid and 0x06 to 0xff are unassigned.
            _ => ("unknown", Err(DecodeError::UnknownObjectKind { kind })),
        };
        for envelope in [&[0x01, kind][..], &[0x01, kind, 0x00]] {
            let decoded = decode_envelope(envelope).map(|envelope| envelope.kind);
            assert_eq!(
                decoded, expected,
                "kind {kind:#04x}, envelope {envelope:02x?}"
            );
        }
        assert_eq!(ObjectKind::from_id(kind), expected, "from_id({kind:#04x})");
        *kinds.entry(class).or_insert(0) += 1;
    }
    assert_eq!(
        kinds,
        BTreeMap::from([("defined", 1), ("reserved", 4), ("unknown", 251)])
    );

    assert!(matches!(
        decode_envelope(&[0x01]),
        Err(DecodeError::UnexpectedEnd { .. })
    ));
}

#[test]
fn object_id_hashes_whole_envelope() {
    let envelope = encode_object(ObjectKind::Blob, b"hello\n");
    for alg in HashAlg::ALL {
        assert_eq!(object_id(alg, &envelope), alg.hash(&envelope));
        assert_ne!(
            object_id(alg, &envelope),
            alg.hash(b"hello\n"),
            "the version and kind bytes are part of the identity"
        );
    }
}

#[test]
fn kind_registry_is_complete() {
    let defined: Vec<(u8, &str)> = ObjectKind::ALL
        .into_iter()
        .map(|kind| (kind.id(), kind.name()))
        .collect();
    assert_eq!(defined, [(0x01, "blob")]);
    assert_eq!(
        RESERVED_KINDS,
        [
            (0x02, "changeset"),
            (0x03, "operation"),
            (0x04, "view"),
            (0x05, "event"),
        ]
    );
    for (kind, _) in RESERVED_KINDS {
        assert_eq!(
            ObjectKind::from_id(kind),
            Err(DecodeError::ReservedObjectKind { kind })
        );
    }
}
