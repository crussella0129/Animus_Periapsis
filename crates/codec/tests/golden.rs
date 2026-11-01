//! Golden corpus: the v1 encodings pinned as files (INT-0006 AC-3).
//!
//! Each case's encoding must equal `tests/golden/v1/<name>.bin` byte for byte,
//! and decoding that file must give the case's value back. With create mode
//! off (the default), a missing or differing file fails. `PERI_GOLDEN_CREATE=1`
//! turns create mode on: missing files are written, existing ones are never
//! overwritten, and a differing file still fails.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use codec::{Bytes, Decode, Encode, ObjectKind, blob, decode_envelope, from_slice, to_vec};
use hash::HashAlg;

type Check = Box<dyn Fn(&[u8]) -> Result<(), String>>;

struct Case {
    name: &'static str,
    bytes: Vec<u8>,
    decodes: Check,
}

fn case<T: Encode + Decode + PartialEq + Debug + 'static>(name: &'static str, value: T) -> Case {
    let bytes = to_vec(&value);
    let decodes: Check = Box::new(move |file| match from_slice::<T>(file) {
        Ok(decoded) if decoded == value => Ok(()),
        Ok(decoded) => Err(format!("decoded {decoded:?}, expected {value:?}")),
        Err(error) => Err(error.to_string()),
    });
    Case {
        name,
        bytes,
        decodes,
    }
}

/// The corpus. `docs/spec/format-v1.md` gives every value's hex by hand.
fn corpus() -> Vec<Case> {
    let blob_content = b"Periapsis\n".to_vec();
    let blob_case = Case {
        name: "blob-envelope",
        bytes: blob(&blob_content),
        decodes: Box::new(move |file| match decode_envelope(file) {
            Ok(env) if env.kind == ObjectKind::Blob && env.body == blob_content => Ok(()),
            Ok(env) => Err(format!("decoded {env:?}")),
            Err(error) => Err(error.to_string()),
        }),
    };
    vec![
        case("uvarint-0", 0u64),
        case("uvarint-127", 127u64),
        case("uvarint-128", 128u64),
        case("uvarint-300", 300u64),
        case("uvarint-max", u64::MAX),
        case("u32-max", u32::MAX),
        case("bool-false", false),
        case("bool-true", true),
        case("bytes-empty", Bytes::default()),
        case("bytes-short", Bytes(vec![0x00, 0x7f, 0x80, 0xff])),
        case("string-empty", String::new()),
        case("string-ascii", "periapsis".to_owned()),
        case("string-utf8", "π → é 🜨".to_owned()),
        case("hashref-blake3", HashAlg::Blake3.hash(b"")),
        case("hashref-sha256", HashAlg::Sha256.hash(b"abc")),
        case("option-none", None::<u64>),
        case("option-some", Some("present".to_owned())),
        case("seq-u64", vec![3u64, 1, 4, 1, 5, 9, 2, 6]),
        case(
            "set-string",
            BTreeSet::from(["a".to_owned(), "b".to_owned(), "ab".to_owned()]),
        ),
        case(
            "map-string-u64",
            BTreeMap::from([("main".to_owned(), 7u64), ("dev".to_owned(), 12)]),
        ),
        case(
            "map-nested",
            BTreeMap::from([(
                "intent".to_owned(),
                BTreeMap::from([(2u64, Some(Bytes(b"x".to_vec()))), (1, None)]),
            )]),
        ),
        case("pair", ("key".to_owned(), true)),
        blob_case,
    ]
}

/// Whether the value of `PERI_GOLDEN_CREATE` turns create mode on: exactly
/// `1` does, anything else (including unset) does not.
fn create_mode(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}

/// Checks every case against `dir`, creating missing files when `create` is
/// set. Returns one message per failure.
fn check_golden(dir: &Path, cases: &[Case], create: bool) -> Vec<String> {
    let mut failures = Vec::new();
    for case in cases {
        let path: PathBuf = dir.join(format!("{}.bin", case.name));
        match fs::read(&path) {
            Ok(file) => {
                if file != case.bytes {
                    failures.push(format!(
                        "{}.bin: encoding changed (file {} bytes, encoder {} bytes)",
                        case.name,
                        file.len(),
                        case.bytes.len()
                    ));
                }
                if let Err(error) = (case.decodes)(&file) {
                    failures.push(format!("{}.bin: file does not decode: {error}", case.name));
                }
            }
            Err(_) if create => {
                fs::create_dir_all(dir).unwrap();
                fs::write(&path, &case.bytes).unwrap();
            }
            Err(error) => failures.push(format!(
                "{}.bin: {} unreadable ({error}); set PERI_GOLDEN_CREATE=1 to create missing files",
                case.name,
                path.display()
            )),
        }
    }
    failures
}

/// An empty directory under the OS temp directory, unique to this process.
fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("codec-golden-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn golden_v1_corpus_matches() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/v1");
    let create = create_mode(std::env::var_os("PERI_GOLDEN_CREATE").as_deref());
    let failures = check_golden(&dir, &corpus(), create);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn golden_changed_file_fails() {
    let dir = scratch_dir("changed");
    let cases = corpus();
    assert!(check_golden(&dir, &cases, true).is_empty());
    let changed = dir.join("string-ascii.bin");
    let mut bytes = fs::read(&changed).unwrap();
    bytes[3] ^= 0x01;
    fs::write(&changed, &bytes).unwrap();

    // The file no longer matches the encoder, and no longer holds the value.
    let failures = check_golden(&dir, &cases, false);
    assert_eq!(failures.len(), 2, "{failures:?}");
    assert!(
        failures[0].starts_with("string-ascii.bin: encoding changed"),
        "{failures:?}"
    );
    assert!(
        failures[1].starts_with("string-ascii.bin: file does not decode"),
        "{failures:?}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn golden_encoder_change_fails() {
    // The encoder's output changed while the committed file still decodes to
    // the value, so only the byte comparison can catch it.
    let dir = scratch_dir("encoder");
    assert!(check_golden(&dir, &corpus(), true).is_empty());
    let mut changed = corpus();
    let case = changed
        .iter_mut()
        .find(|case| case.name == "uvarint-0")
        .unwrap();
    case.bytes = vec![0x01];
    assert_eq!(
        check_golden(&dir, &changed, false),
        ["uvarint-0.bin: encoding changed (file 1 bytes, encoder 1 bytes)"]
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn golden_missing_file_fails_without_create_mode() {
    let dir = scratch_dir("missing");
    let cases = corpus();
    let failures = check_golden(&dir, &cases, false);

    assert_eq!(failures.len(), cases.len(), "{failures:?}");
    for (case, failure) in cases.iter().zip(&failures) {
        assert!(
            failure.starts_with(&format!("{}.bin:", case.name)),
            "{failure} should name {}.bin",
            case.name
        );
    }
    assert_eq!(
        fs::read_dir(&dir).unwrap().count(),
        0,
        "nothing may be written"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn golden_create_mode_never_overwrites() {
    let dir = scratch_dir("create");
    let stale = dir.join("uvarint-0.bin");
    fs::write(&stale, [0xee]).unwrap();

    let cases = corpus();
    let failures = check_golden(&dir, &cases, true);

    assert_eq!(
        fs::read(&stale).unwrap(),
        [0xee],
        "an existing file was overwritten"
    );
    assert!(
        failures.iter().any(|f| f.starts_with("uvarint-0.bin:")),
        "the differing file must fail the run: {failures:?}"
    );
    for case in cases.iter().filter(|case| case.name != "uvarint-0") {
        assert_eq!(
            fs::read(dir.join(format!("{}.bin", case.name))).unwrap(),
            case.bytes
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn create_mode_is_exactly_one() {
    assert!(create_mode(Some(OsStr::new("1"))));
    for value in ["0", "true", "", " 1", "yes"] {
        assert!(!create_mode(Some(OsStr::new(value))), "{value:?}");
    }
    assert!(!create_mode(None));
}
