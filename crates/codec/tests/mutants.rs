//! Every single-byte substitution, insertion and deletion of a fixed value of
//! each type either fails to decode or re-encodes to itself (INT-0006 AC-1).
//! Random mutation can miss rare decoding mutants; this covers all of them for
//! one value per type and requires at least one to decode, so the property is
//! actually exercised.

use std::collections::{BTreeMap, BTreeSet};

use codec::{Bytes, Decode, Encode, blob, decode_envelope, encode_object, from_slice, to_vec};
use hash::HashAlg;

/// Applies every single-byte mutation to `encoding`. `reencode` returns the
/// canonical re-encoding of a mutant that decodes, or `None`. Returns how many
/// mutants decoded.
fn exhaustive(encoding: &[u8], reencode: impl Fn(&[u8]) -> Option<Vec<u8>>) -> usize {
    let mut decoded = 0;
    let mut check = |mutant: Vec<u8>| {
        if let Some(again) = reencode(&mutant) {
            assert_eq!(
                again, mutant,
                "a mutant decoded to a value with another encoding"
            );
            decoded += 1;
        }
    };
    for i in 0..=encoding.len() {
        for byte in 0..=u8::MAX {
            let mut mutant = encoding.to_vec();
            mutant.insert(i, byte);
            check(mutant);
        }
    }
    for i in 0..encoding.len() {
        for byte in (0..=u8::MAX).filter(|&byte| byte != encoding[i]) {
            let mut mutant = encoding.to_vec();
            mutant[i] = byte;
            check(mutant);
        }
        let mut mutant = encoding.to_vec();
        mutant.remove(i);
        check(mutant);
    }
    decoded
}

fn value<T: Encode + Decode>(value: &T) -> usize {
    exhaustive(&to_vec(value), |mutant| {
        from_slice::<T>(mutant).ok().map(|decoded| to_vec(&decoded))
    })
}

#[test]
fn exhaustive_mutants_reencode() {
    let nested: BTreeMap<String, BTreeMap<u64, Option<Vec<Bytes>>>> = BTreeMap::from([(
        "k".to_owned(),
        BTreeMap::from([(1, None), (2, Some(vec![Bytes(vec![7])]))]),
    )]);
    let counts = [
        ("u64", value(&300u64)),
        ("u32", value(&70_000u32)),
        ("bool", value(&true)),
        ("string", value(&"pi".to_owned())),
        ("bytes", value(&Bytes(vec![1, 2]))),
        ("hash_ref", value(&HashAlg::Blake3.hash(b""))),
        ("option", value(&Some(5u64))),
        ("vec", value(&vec![1u32, 2])),
        (
            "set",
            value(&BTreeSet::from(["a".to_owned(), "b".to_owned()])),
        ),
        ("map", value(&BTreeMap::from([("k".to_owned(), true)]))),
        ("pair", value(&("a".to_owned(), None::<u64>))),
        ("nested", value(&nested)),
        (
            "envelope",
            exhaustive(&blob(b"x"), |mutant| {
                decode_envelope(mutant)
                    .ok()
                    .map(|envelope| encode_object(envelope.kind, envelope.body))
            }),
        ),
    ];
    for (name, decoded) in counts {
        assert!(
            decoded > 0,
            "{name}: no mutant decoded, so nothing was checked"
        );
    }
}
