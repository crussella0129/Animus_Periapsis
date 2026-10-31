//! Known-answer and property tests for algorithm-tagged hashes (INT-0006 AC-2, AC-4).

use hash::{HashAlg, HashError, HashRef};
use proptest::prelude::*;

/// (input length, first 32 bytes of `hash`) for every case in the official
/// BLAKE3 `test_vectors/test_vectors.json` (BLAKE3-team/BLAKE3,
/// CC0-1.0 OR Apache-2.0). Input byte `i` is `i % 251`.
const BLAKE3_VECTORS: [(usize, &str); 35] = [
    (
        0,
        "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
    ),
    (
        1,
        "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213",
    ),
    (
        2,
        "7b7015bb92cf0b318037702a6cdd81dee41224f734684c2c122cd6359cb1ee63",
    ),
    (
        3,
        "e1be4d7a8ab5560aa4199eea339849ba8e293d55ca0a81006726d184519e647f",
    ),
    (
        4,
        "f30f5ab28fe047904037f77b6da4fea1e27241c5d132638d8bedce9d40494f32",
    ),
    (
        5,
        "b40b44dfd97e7a84a996a91af8b85188c66c126940ba7aad2e7ae6b385402aa2",
    ),
    (
        6,
        "06c4e8ffb6872fad96f9aaca5eee1553eb62aed0ad7198cef42e87f6a616c844",
    ),
    (
        7,
        "3f8770f387faad08faa9d8414e9f449ac68e6ff0417f673f602a646a891419fe",
    ),
    (
        8,
        "2351207d04fc16ade43ccab08600939c7c1fa70a5c0aaca76063d04c3228eaeb",
    ),
    (
        63,
        "e9bc37a594daad83be9470df7f7b3798297c3d834ce80ba85d6e207627b7db7b",
    ),
    (
        64,
        "4eed7141ea4a5cd4b788606bd23f46e212af9cacebacdc7d1f4c6dc7f2511b98",
    ),
    (
        65,
        "de1e5fa0be70df6d2be8fffd0e99ceaa8eb6e8c93a63f2d8d1c30ecb6b263dee",
    ),
    (
        127,
        "d81293fda863f008c09e92fc382a81f5a0b4a1251cba1634016a0f86a6bd640d",
    ),
    (
        128,
        "f17e570564b26578c33bb7f44643f539624b05df1a76c81f30acd548c44b45ef",
    ),
    (
        129,
        "683aaae9f3c5ba37eaaf072aed0f9e30bac0865137bae68b1fde4ca2aebdcb12",
    ),
    (
        1023,
        "10108970eeda3eb932baac1428c7a2163b0e924c9a9e25b35bba72b28f70bd11",
    ),
    (
        1024,
        "42214739f095a406f3fc83deb889744ac00df831c10daa55189b5d121c855af7",
    ),
    (
        1025,
        "d00278ae47eb27b34faecf67b4fe263f82d5412916c1ffd97c8cb7fb814b8444",
    ),
    (
        2048,
        "e776b6028c7cd22a4d0ba182a8bf62205d2ef576467e838ed6f2529b85fba24a",
    ),
    (
        2049,
        "5f4d72f40d7a5f82b15ca2b2e44b1de3c2ef86c426c95c1af0b6879522563030",
    ),
    (
        3072,
        "b98cb0ff3623be03326b373de6b9095218513e64f1ee2edd2525c7ad1e5cffd2",
    ),
    (
        3073,
        "7124b49501012f81cc7f11ca069ec9226cecb8a2c850cfe644e327d22d3e1cd3",
    ),
    (
        4096,
        "015094013f57a5277b59d8475c0501042c0b642e531b0a1c8f58d2163229e969",
    ),
    (
        4097,
        "9b4052b38f1c5fc8b1f9ff7ac7b27cd242487b3d890d15c96a1c25b8aa0fb995",
    ),
    (
        5120,
        "9cadc15fed8b5d854562b26a9536d9707cadeda9b143978f319ab34230535833",
    ),
    (
        5121,
        "628bd2cb2004694adaab7bbd778a25df25c47b9d4155a55f8fbd79f2fe154cff",
    ),
    (
        6144,
        "3e2e5b74e048f3add6d21faab3f83aa44d3b2278afb83b80b3c35164ebeca205",
    ),
    (
        6145,
        "f1323a8631446cc50536a9f705ee5cb619424d46887f3c376c695b70e0f0507f",
    ),
    (
        7168,
        "61da957ec2499a95d6b8023e2b0e604ec7f6b50e80a9678b89d2628e99ada77a",
    ),
    (
        7169,
        "a003fc7a51754a9b3c7fae0367ab3d782dccf28855a03d435f8cfe74605e7817",
    ),
    (
        8192,
        "aae792484c8efe4f19e2ca7d371d8c467ffb10748d8a5a1ae579948f718a2a63",
    ),
    (
        8193,
        "bab6c09cb8ce8cf459261398d2e7aef35700bf488116ceb94a36d0f5f1b7bc3b",
    ),
    (
        16384,
        "f875d6646de28985646f34ee13be9a576fd515f76b5b0a26bb324735041ddde4",
    ),
    (
        31744,
        "62b6960e1a44bcc1eb1a611a8d6235b6b4b78f32e7abc4fb4c6cdcce94895c47",
    ),
    (
        102400,
        "bc3e3d41a1146b069abffad3c0d44860cf664390afce4d9661f7902e7943e085",
    ),
];

fn vector_input(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn blake3_matches_official_vectors() {
    for (len, expected) in BLAKE3_VECTORS {
        let hash = HashAlg::Blake3.hash(&vector_input(len));
        assert_eq!(hex(hash.digest()), expected, "input length {len}");
    }
}

#[test]
fn sha256_matches_fips_vectors() {
    let cases = [
        (
            &b""[..],
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            &b"abc"[..],
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(hex(HashAlg::Sha256.hash(input).digest()), expected);
    }
}

#[test]
fn only_registered_alg_ids_decode() {
    for id in 0..=u8::MAX {
        match HashAlg::from_id(id) {
            Ok(alg) => {
                assert!(id == 0x01 || id == 0x02, "unexpected id {id:#04x}");
                assert_eq!(alg.id(), id);
            }
            Err(error) => {
                assert!(id != 0x01 && id != 0x02, "id {id:#04x} should decode");
                assert_eq!(error, HashError::UnknownAlgorithm(id));
            }
        }
    }
}

#[test]
fn hash_registry_is_complete() {
    let registry: Vec<(u8, &str, usize)> = HashAlg::ALL
        .into_iter()
        .map(|alg| (alg.id(), alg.name(), alg.digest_len()))
        .collect();
    assert_eq!(registry, [(0x01, "blake3", 32), (0x02, "sha256", 32)]);
    assert_eq!(HashAlg::DEFAULT, HashAlg::Blake3);
    for alg in HashAlg::ALL {
        assert_eq!(HashAlg::from_name(alg.name()), Ok(alg));
    }
}

#[test]
fn wrong_digest_length_rejected() {
    for len in [0, 31, 33] {
        for alg in HashAlg::ALL {
            assert_eq!(
                HashRef::new(alg, &vec![0u8; len]),
                Err(HashError::DigestLength {
                    alg,
                    expected: 32,
                    actual: len,
                })
            );
        }
    }
}

#[test]
fn equal_digest_different_alg_unequal() {
    let digest = [7u8; 32];
    let blake3 = HashRef::new(HashAlg::Blake3, &digest).unwrap();
    let sha256 = HashRef::new(HashAlg::Sha256, &digest).unwrap();
    assert_ne!(blake3, sha256);
    assert!(
        blake3 < sha256,
        "BLAKE3 (id 0x01) orders before SHA-256 (id 0x02)"
    );
    let higher_digest = HashRef::new(HashAlg::Blake3, &[0xff; 32]).unwrap();
    assert!(
        higher_digest < sha256,
        "algorithm id is compared before the digest"
    );
}

#[test]
fn binary_rejects_malformed() {
    let valid = HashAlg::Blake3.hash(b"x").to_bytes();
    assert_eq!(
        HashRef::from_bytes(&[0x09; 33]),
        Err(HashError::UnknownAlgorithm(0x09))
    );
    assert_eq!(
        HashRef::from_bytes(&valid[..20]),
        Err(HashError::Truncated {
            needed: 32,
            available: 19,
        })
    );
    assert_eq!(
        HashRef::from_bytes(&[]),
        Err(HashError::Truncated {
            needed: 1,
            available: 0,
        })
    );
    let mut trailing = valid.clone();
    trailing.push(0);
    assert_eq!(
        HashRef::from_bytes(&trailing),
        Err(HashError::TrailingBytes { extra: 1 })
    );
}

#[test]
fn text_rejects_malformed() {
    let valid = HashAlg::Blake3.hash(b"x").to_string();
    let digest_hex = valid.split_once(':').unwrap().1;
    let cases = [
        valid.to_uppercase(),
        format!("blake3:{}", digest_hex.to_uppercase()),
        format!("md5:{digest_hex}"),
        format!("blake3:{}", &digest_hex[1..]),
        format!("blake3:{}", &digest_hex[2..]),
        format!("blake3:{digest_hex}00"),
        digest_hex.to_owned(),
        format!("blake3:{}zz", &digest_hex[2..]),
    ];
    for text in cases {
        assert!(
            text.parse::<HashRef>().is_err(),
            "{text:?} should be rejected"
        );
    }
    assert_eq!(
        "blake3".parse::<HashRef>(),
        Err(HashError::MissingSeparator)
    );
}

fn any_hash() -> impl Strategy<Value = HashRef> {
    (
        prop::sample::select(HashAlg::ALL.to_vec()),
        any::<[u8; 32]>(),
    )
        .prop_map(|(alg, digest)| HashRef::new(alg, &digest).unwrap())
}

proptest! {
    #[test]
    fn binary_round_trip(hash in any_hash()) {
        let bytes = hash.to_bytes();
        prop_assert_eq!(bytes.len(), HashRef::ENCODED_LEN);
        prop_assert_eq!(bytes[0], hash.alg().id());
        prop_assert_eq!(HashRef::from_bytes(&bytes), Ok(hash));
    }

    #[test]
    fn text_round_trip(hash in any_hash()) {
        let text = hash.to_string();
        let prefix = format!("{}:", hash.alg().name());
        prop_assert!(text.starts_with(&prefix));
        prop_assert_eq!(text.parse::<HashRef>(), Ok(hash));
    }

    #[test]
    fn incremental_matches_one_shot(
        data in prop::collection::vec(any::<u8>(), 0..4096),
        cuts in prop::collection::vec(any::<prop::sample::Index>(), 0..8),
    ) {
        let mut cuts: Vec<usize> = cuts.iter().map(|cut| cut.index(data.len() + 1)).collect();
        cuts.sort_unstable();
        for alg in HashAlg::ALL {
            let mut hasher = alg.hasher();
            let mut start = 0;
            for &cut in &cuts {
                hasher.update(&data[start..cut]);
                start = cut;
            }
            hasher.update(&data[start..]);
            prop_assert_eq!(hasher.finalize(), alg.hash(&data));
        }
    }
}
