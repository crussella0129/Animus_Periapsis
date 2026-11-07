//! Canonicality and totality of the v1 encoding (INT-0006 AC-1, AC-5).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

use codec::{
    Bytes, Changeset, Decode, DecodeError, Decoder, Encode, Encoder, Entry, Mode, ObjectKind,
    Operation, RESERVED_KINDS, RepoPath, View, blob, decode_envelope, encode_object, from_slice,
    object_id, to_vec,
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

/// Half uniform, which almost always takes nine or ten bytes, and half shifted
/// right at random, so every varint length from one to ten bytes is drawn.
fn any_u64() -> impl Strategy<Value = u64> {
    prop_oneof![
        any::<u64>(),
        (any::<u64>(), 0..64u32).prop_map(|(value, shift)| value >> shift),
    ]
}

/// Up to 400 bytes: most lengths take a two-byte prefix, and some do not fit
/// in a byte.
fn any_bytes() -> impl Strategy<Value = Bytes> {
    prop::collection::vec(any::<u8>(), 0..400).prop_map(Bytes)
}

/// Up to three parents and a view, over both hash algorithms.
fn any_operation() -> impl Strategy<Value = Operation> {
    (prop::collection::btree_set(any_hash(), 0..4), any_hash())
        .prop_map(|(parents, view)| Operation { parents, view })
}

/// A repository path of one to three components; no component is `.` or
/// `..`, and some are not ASCII.
fn any_path() -> impl Strategy<Value = RepoPath> {
    prop::collection::vec("[a-zA-Z0-9_π-]{1,4}(\\.[a-z]{1,2})?", 1..4)
        .prop_map(|components| RepoPath::new(components.join("/")).unwrap())
}

fn any_entry() -> impl Strategy<Value = Entry> {
    prop_oneof![
        Just(Entry::Delete),
        (any_hash(), any::<bool>()).prop_map(|(blob, executable)| Entry::Put {
            blob,
            mode: if executable {
                Mode::Executable
            } else {
                Mode::Regular
            },
        }),
    ]
}

/// Up to three parents (order kept, repeats allowed by the seq) and five
/// changed paths.
fn any_changeset() -> impl Strategy<Value = Changeset> {
    (
        prop::collection::vec(any_hash(), 0..4),
        any::<[u8; 16]>(),
        prop::collection::btree_map(any_path(), any_entry(), 0..6),
        prop::collection::vec(any::<u8>(), 0..40),
    )
        .prop_map(|(parents, change_id, changes, description)| Changeset {
            parents,
            change_id,
            changes,
            description,
        })
}

fn any_view() -> impl Strategy<Value = View> {
    (
        prop::collection::btree_set(any_hash(), 0..4),
        prop::collection::btree_map(".{0,8}", any_hash(), 0..4),
    )
        .prop_map(|(heads, workspaces)| View { heads, workspaces })
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
    round_trip_u64, mutants_u64: u64 = any_u64();
    round_trip_u32, mutants_u32: u32 = any::<u32>();
    round_trip_bool, mutants_bool: bool = any::<bool>();
    // Up to 300 characters, so many lengths take two bytes and pass 255.
    round_trip_string, mutants_string: String = ".{0,300}";
    round_trip_bytes, mutants_bytes: Bytes = any_bytes();
    round_trip_hash_ref, mutants_hash_ref: HashRef = any_hash();
    round_trip_option, mutants_option: Option<u64> = prop::option::of(any_u64());
    round_trip_vec, mutants_vec: Vec<u32> = prop::collection::vec(any::<u32>(), 0..8);
    round_trip_set, mutants_set: BTreeSet<HashRef> = prop::collection::btree_set(any_hash(), 0..4);
    round_trip_map, mutants_map: BTreeMap<String, bool> =
        prop::collection::btree_map(".{0,8}", any::<bool>(), 0..6);
    round_trip_pair, mutants_pair: (String, Option<u64>) =
        (".{0,8}", prop::option::of(any_u64()));
    round_trip_nested, mutants_nested: Nested = any_nested();
    operation_round_trips, accepted_operation_bytes_reencode: Operation = any_operation();
    changeset_round_trips, accepted_changeset_bytes_reencode: Changeset = any_changeset();
    view_round_trips, accepted_view_bytes_reencode: View = any_view();
    repo_path_round_trips, accepted_repo_path_bytes_reencode: RepoPath = any_path();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn round_trip_envelope(body in prop::collection::vec(any::<u8>(), 0..400)) {
        let envelope = blob(&body);
        let decoded = decode_envelope(&envelope).unwrap();
        prop_assert_eq!(decoded.kind, ObjectKind::Blob);
        prop_assert_eq!(decoded.body, &body[..]);
        prop_assert_eq!(encode_object(decoded.kind, decoded.body), envelope);
    }

    #[test]
    fn mutants_envelope(
        body in prop::collection::vec(any::<u8>(), 0..400),
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
    fn decode_is_total_on_random_bytes(bytes in prop::collection::vec(any::<u8>(), 0..=256)) {
        // Lengths 0 to 256, both included. Each call must return, Ok or Err,
        // without panicking.
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

    #[test]
    fn operation_decoding_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..=256)) {
        // As a bare body and inside an operation envelope.
        let _ = from_slice::<Operation>(&bytes);
        let _ = Operation::from_envelope(&[&[0x01, 0x03][..], &bytes].concat());
    }

    #[test]
    fn changeset_decoding_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..=256)) {
        let _ = from_slice::<Changeset>(&bytes);
        let _ = Changeset::from_envelope(&[&[0x01, 0x02][..], &bytes].concat());
        let _ = from_slice::<RepoPath>(&bytes);
    }

    #[test]
    fn view_decoding_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..=256)) {
        let _ = from_slice::<View>(&bytes);
        let _ = View::from_envelope(&[&[0x01, 0x04][..], &bytes].concat());
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
    // Both ends of every length: 2^7k is k bytes of `80` then `01`, and
    // 2^7k - 1 is k - 1 bytes of `ff` then `7f`.
    for k in 1..=9 {
        let mut smallest = vec![0x80; k];
        smallest.push(0x01);
        let mut largest = vec![0xff; k - 1];
        largest.push(0x7f);
        for (value, bytes) in [(1u64 << (7 * k), smallest), ((1 << (7 * k)) - 1, largest)] {
            assert_eq!(to_vec(&value), bytes, "{value}");
            assert_eq!(from_slice::<u64>(&bytes), Ok(value));
        }
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
    // A redundant zero group, after one, two or nine continuation bytes, is
    // reported where the varint starts.
    let mut ten = [0x80u8; 10];
    ten[9] = 0x00;
    for redundant in [&[0x80, 0x00][..], &[0x80, 0x80, 0x00], &ten] {
        assert_eq!(
            from_slice::<Vec<u64>>(&second_element(0x01, redundant)),
            Err(DecodeError::NonMinimalVarint { at: 2 }),
            "{redundant:02x?}"
        );
    }
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
    // A bad byte other than 02, or a bad tag at another offset, keeps the
    // offset and the byte apart: `01 05` is Some(5), so the next tag is at 3.
    assert_eq!(
        from_slice::<Vec<bool>>(&second_element(0x00, &[0xff])),
        Err(DecodeError::InvalidBool { at: 2, byte: 0xff })
    );
    assert_eq!(
        from_slice::<Vec<Option<u64>>>(&second_element(0x00, &[0xff])),
        Err(DecodeError::InvalidOptionTag { at: 2, byte: 0xff })
    );
    assert_eq!(
        from_slice::<Vec<Option<u64>>>(&[0x02, 0x01, 0x05, 0x02]),
        Err(DecodeError::InvalidOptionTag { at: 3, byte: 2 })
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

/// `value` encodes to exactly `expected`, and `expected` decodes to `value`.
fn known_answer<T: Encode + Decode + PartialEq + Debug>(value: T, expected: &[u8]) {
    let name = std::any::type_name::<T>();
    assert_eq!(to_vec(&value), expected, "{name}");
    assert_eq!(from_slice::<T>(expected), Ok(value), "{name}");
}

/// Every length and count is a `uvarint`: from 128 (`80 01`) it takes two
/// bytes, and 300 (`ac 02`) does not fit in one.
#[test]
fn lengths_and_counts_are_varints() {
    let prefixed = |prefix: [u8; 2], content: &[u8]| [&prefix[..], content].concat();
    // 0 to 127 each encode as the one byte equal to the value.
    let small: Vec<u8> = (0..128).collect();
    known_answer(
        (0..128u64).collect::<Vec<_>>(),
        &prefixed([0x80, 0x01], &small),
    );
    known_answer(
        (0..128u64).collect::<BTreeSet<_>>(),
        &prefixed([0x80, 0x01], &small),
    );
    let entries: Vec<u8> = (0..128u8)
        .flat_map(|key| [key, u8::from(key % 2 == 0)])
        .collect();
    known_answer(
        (0..128u64)
            .map(|key| (key, key % 2 == 0))
            .collect::<BTreeMap<_, _>>(),
        &prefixed([0x80, 0x01], &entries),
    );
    known_answer(
        Bytes(vec![0xab; 128]),
        &prefixed([0x80, 0x01], &[0xab; 128]),
    );
    // "é" is `c3 a9`, so 64 of them take 128 bytes.
    known_answer(
        "é".repeat(64),
        &prefixed([0x80, 0x01], &[0xc3u8, 0xa9].repeat(64)),
    );
    known_answer("a".repeat(300), &prefixed([0xac, 0x02], &[b'a'; 300]));
    known_answer(
        Bytes(vec![b'a'; 300]),
        &prefixed([0xac, 0x02], &[b'a'; 300]),
    );
    // The spec's example, `"é"` → `02 c3 a9`, through `str` as well as `String`.
    assert_eq!(to_vec("é"), [0x02, 0xc3, 0xa9]);
    known_answer("é".to_owned(), &[0x02, 0xc3, 0xa9]);
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
    // All but the three bytes present are missing (saturating where `usize`
    // is narrower than `u64`).
    let needed = usize::try_from(u64::MAX - 3).unwrap_or(usize::MAX);
    let at_prefix = |result: Result<(), DecodeError>| {
        assert_eq!(result, Err(DecodeError::UnexpectedEnd { at: 2, needed }));
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
    // A prefix of 2 with one byte after it, and of 5 (the whole input's
    // length) with two: the prefix is checked against what follows it, so
    // the error is at the prefix, before any element is read.
    for (second, needed) in [(&[0x02, 0x61][..], 1), (&[0x05, 0x07, 0x08], 3)] {
        let input = second_element(0x00, second);
        for result in [
            from_slice::<Vec<Bytes>>(&input).map(drop),
            from_slice::<Vec<String>>(&input).map(drop),
            from_slice::<Vec<Vec<u64>>>(&input).map(drop),
            from_slice::<Vec<BTreeSet<u64>>>(&input).map(drop),
            from_slice::<Vec<BTreeMap<u64, u64>>>(&input).map(drop),
        ] {
            assert_eq!(
                result,
                Err(DecodeError::UnexpectedEnd { at: 2, needed }),
                "{input:02x?}"
            );
        }
    }
    // `take` from offset 1 with two bytes left: all but two of `len` are
    // missing, up to the largest `len`.
    for (len, needed) in [(5, 3), (usize::MAX, usize::MAX - 2)] {
        let mut dec = Decoder::new(&[0x01, 0x02, 0x03]);
        dec.byte().unwrap();
        assert_eq!(
            dec.take(len),
            Err(DecodeError::UnexpectedEnd { at: 1, needed })
        );
    }
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
    // The key is checked before its value is read, so a bad or missing value
    // after it does not hide it.
    for descending in [
        &[0x02, 0x01, b'b', 0x00, 0x01, b'a', 0x02][..],
        &[0x02, 0x01, b'b', 0x00, 0x01, b'a'],
    ] {
        assert_eq!(
            from_slice::<BTreeMap<String, bool>>(descending),
            Err(DecodeError::UnsortedOrDuplicateKey { at: 4 }),
            "{descending:02x?}"
        );
    }
    let set = [0x02, 0x05, 0x05];
    assert_eq!(
        from_slice::<BTreeSet<u64>>(&set),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 2 })
    );
    // Byte strings order by content, not by length: `01 00` comes before `02`.
    let bytes = BTreeSet::from([Bytes(vec![0x02]), Bytes(vec![0x01, 0x00])]);
    let canonical = [0x02, 0x02, 0x01, 0x00, 0x01, 0x02];
    assert_eq!(to_vec(&bytes), canonical);
    assert_eq!(from_slice::<BTreeSet<Bytes>>(&canonical), Ok(bytes));
    assert_eq!(
        from_slice::<BTreeSet<Bytes>>(&[0x02, 0x01, 0x02, 0x02, 0x01, 0x00]),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 3 })
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
    // `at` is the first trailing byte and `remaining` their count: here three
    // bytes follow the two-byte varint for 128.
    assert_eq!(
        from_slice::<u64>(&[0x80, 0x01, 0xaa, 0xbb, 0xcc]),
        Err(DecodeError::TrailingBytes {
            at: 2,
            remaining: 3,
        })
    );
}

/// The spec's version rule and kind table, written out by hand rather than
/// taken from the code they check: what an envelope with this header decodes
/// to, and its class. The version comes first: under another version the kind
/// byte has no v1 meaning, so a version other than 1 is rejected whatever the
/// kind.
fn expected_header(version: u8, kind: u8) -> (&'static str, Result<ObjectKind, DecodeError>) {
    match (version, kind) {
        (0x01, 0x01) => ("defined", Ok(ObjectKind::Blob)),
        (0x01, 0x02) => ("defined", Ok(ObjectKind::Changeset)),
        (0x01, 0x03) => ("defined", Ok(ObjectKind::Operation)),
        (0x01, 0x04) => ("defined", Ok(ObjectKind::View)),
        (0x01, 0x05) => ("reserved", Err(DecodeError::ReservedObjectKind { kind })),
        // 0x00 is invalid and 0x06 to 0xff are unassigned.
        (0x01, _) => ("unknown", Err(DecodeError::UnknownObjectKind { kind })),
        _ => (
            "unsupported",
            Err(DecodeError::UnsupportedFormatVersion { version }),
        ),
    }
}

/// Every (version, kind) pair, each with an empty, a one-byte and a two-byte
/// body, against `expected_header`; the class counts fail a sweep that skips
/// pairs. Input too short for a header reports truncation, whatever its byte.
#[test]
fn envelope_rejects_unknown_version_and_kind() {
    // A blob body has no length prefix, so any length decodes.
    for content in [&b"content"[..], &[b'a'; 300]] {
        let envelope = blob(content);
        let decoded = decode_envelope(&envelope).unwrap();
        assert_eq!((decoded.kind, decoded.body), (ObjectKind::Blob, content));
    }

    let bodies: [&[u8]; 3] = [&[], &[0x00], &[0x61, 0x62]];
    let mut classes = BTreeMap::new();
    for version in 0..=u8::MAX {
        for kind in 0..=u8::MAX {
            let (class, expected) = expected_header(version, kind);
            for body in bodies {
                let envelope = [&[version, kind][..], body].concat();
                let decoded = decode_envelope(&envelope).map(|envelope| envelope.kind);
                assert_eq!(decoded, expected, "envelope {envelope:02x?}");
            }
            *classes.entry(class).or_insert(0) += 1;
        }
    }
    assert_eq!(
        classes,
        BTreeMap::from([
            ("defined", 4),
            ("reserved", 1),
            ("unknown", 251),
            ("unsupported", 255 * 256),
        ])
    );

    // `from_id` is the lookup the decoder uses, so it must agree id for id.
    for kind in 0..=u8::MAX {
        let (_, expected) = expected_header(0x01, kind);
        assert_eq!(ObjectKind::from_id(kind), expected, "from_id({kind:#04x})");
    }

    // `at` is the first missing field (the version at 0, the kind at 1) and
    // `needed` the bytes missing. One byte is truncation even when it is not
    // version 1 (the locked test plan; code review codec-4).
    assert_eq!(
        decode_envelope(&[]),
        Err(DecodeError::UnexpectedEnd { at: 0, needed: 2 })
    );
    for byte in 0..=u8::MAX {
        assert_eq!(
            decode_envelope(&[byte]),
            Err(DecodeError::UnexpectedEnd { at: 1, needed: 1 }),
            "{byte:#04x}"
        );
    }

    // Each header error says which rule failed: a reserved kind never reads
    // as an unknown one.
    let message = |envelope: &[u8]| decode_envelope(envelope).unwrap_err().to_string();
    assert_eq!(message(&[0x02, 0x01]), "unsupported format version 2");
    assert_eq!(
        message(&[0x01, 0x05]),
        "object kind 0x05 is reserved and not yet specified"
    );
    assert_eq!(message(&[0x01, 0x06]), "unknown object kind 0x06");
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
    // Long envelopes that differ only in their last byte: every byte is part
    // of the identity, however far in.
    for len in [100, 10_000] {
        let long = blob(&vec![b'a'; len]);
        let mut changed = long.clone();
        *changed.last_mut().unwrap() = b'b';
        for alg in HashAlg::ALL {
            assert_eq!(object_id(alg, &long), alg.hash(&long));
            assert_ne!(
                object_id(alg, &long),
                object_id(alg, &changed),
                "{len}-byte blob"
            );
        }
    }
}

#[test]
fn kind_registry_is_complete() {
    let defined: Vec<(u8, &str)> = ObjectKind::ALL
        .into_iter()
        .map(|kind| (kind.id(), kind.name()))
        .collect();
    assert_eq!(
        defined,
        [
            (0x01, "blob"),
            (0x02, "changeset"),
            (0x03, "operation"),
            (0x04, "view")
        ]
    );
    assert_eq!(RESERVED_KINDS, [(0x05, "event")]);
    for (kind, _) in RESERVED_KINDS {
        assert_eq!(
            ObjectKind::from_id(kind),
            Err(DecodeError::ReservedObjectKind { kind })
        );
    }
}

fn hash_of(alg: HashAlg, input: &[u8]) -> HashRef {
    alg.hash(input)
}

/// Two hashes in canonical order: algorithm id first, then digest.
fn ordered_pair() -> (HashRef, HashRef) {
    let (a, b) = (
        hash_of(HashAlg::Sha256, b"abc"),
        hash_of(HashAlg::Sha256, b""),
    );
    assert!(a < b, "ba78… sorts before e3b0…");
    (a, b)
}

#[test]
fn operation_rejects_unsorted_parents() {
    let (low, high) = ordered_pair();
    let mut enc = Encoder::new();
    enc.uvarint(2);
    enc.hash_ref(&high);
    enc.hash_ref(&low);
    enc.uvarint(0);
    let body = enc.finish();
    // The second parent starts after the count byte and the 33-byte first one.
    assert_eq!(
        from_slice::<Operation>(&body),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 34 })
    );
}

#[test]
fn operation_needs_a_view() {
    // Sprint 2: the body ends with the view's hashref, so a body holding only
    // the parent set fails where the view should start.
    let mut enc = Encoder::new();
    enc.uvarint(0);
    assert!(matches!(
        from_slice::<Operation>(&enc.finish()),
        Err(DecodeError::Hash { at: 1, .. })
    ));
}

#[test]
fn operation_rejects_trailing_bytes() {
    let op = Operation {
        parents: BTreeSet::new(),
        view: hash_of(HashAlg::Blake3, b""),
    };
    let mut body = to_vec(&op);
    let len = body.len();
    body.push(0x00);
    assert_eq!(
        from_slice::<Operation>(&body),
        Err(DecodeError::TrailingBytes {
            at: len,
            remaining: 1,
        })
    );
    let mut envelope = op.to_envelope();
    envelope.push(0x00);
    assert_eq!(
        Operation::from_envelope(&envelope),
        Err(DecodeError::TrailingBytes {
            at: len,
            remaining: 1,
        })
    );
}

#[test]
fn operation_kind_is_defined() {
    assert_eq!(ObjectKind::from_id(0x03), Ok(ObjectKind::Operation));
    assert_eq!(ObjectKind::Operation.name(), "operation");
    let op = Operation {
        parents: BTreeSet::from([hash_of(HashAlg::Blake3, b"p")]),
        view: hash_of(HashAlg::Sha256, b"o"),
    };
    let envelope = op.to_envelope();
    assert_eq!(&envelope[..2], &[0x01, 0x03]);
    assert_eq!(Operation::from_envelope(&envelope), Ok(op.clone()));
    // A blob envelope is not an operation, even when its body would decode.
    let body = to_vec(&op);
    assert_eq!(
        Operation::from_envelope(&blob(&body)),
        Err(DecodeError::UnexpectedObjectKind {
            expected: 0x03,
            found: 0x01,
        })
    );
}

#[test]
fn other_kinds_stay_reserved() {
    // Sprint 2 defines 0x02 and 0x04; only 0x05 is still reserved.
    assert_eq!(
        ObjectKind::from_id(0x05),
        Err(DecodeError::ReservedObjectKind { kind: 0x05 })
    );
    assert_eq!(
        Operation::from_envelope(&[0x01, 0x05]),
        Err(DecodeError::ReservedObjectKind { kind: 0x05 })
    );
    for found in [0x02, 0x04] {
        assert_eq!(
            Operation::from_envelope(&[0x01, found]),
            Err(DecodeError::UnexpectedObjectKind {
                expected: 0x03,
                found
            })
        );
    }
}

#[test]
fn operation_rejects_duplicate_parents() {
    let (low, _) = ordered_pair();
    let mut enc = Encoder::new();
    enc.uvarint(2);
    enc.hash_ref(&low);
    enc.hash_ref(&low);
    enc.uvarint(0);
    assert_eq!(
        from_slice::<Operation>(&enc.finish()),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 34 })
    );
}
