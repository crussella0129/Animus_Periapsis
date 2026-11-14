//! Golden corpus: the v1 encodings pinned as files (INT-0006 AC-3).
//!
//! Each case's encoding must equal `tests/golden/v1/<name>.bin` byte for byte,
//! and decoding that file must give the case's value back. With create mode
//! off (the default), a missing or differing file fails. `PERI_GOLDEN_CREATE=1`
//! turns create mode on: missing files are written, existing ones are never
//! overwritten, and a differing file still fails.

use std::any::type_name;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use codec::{
    Bytes, Changeset, Decode, DecodeError, Encode, Entry, Envelope, Mode, ObjectKind, Operation,
    RepoPath, View, WorkspaceEntry, blob, decode_envelope, from_slice, to_vec,
};
use hash::{HashAlg, HashRef};

type Check = Box<dyn Fn(&[u8]) -> Result<(), String>>;

struct Case {
    name: &'static str,
    /// The value's type. Values of different types can share an encoding, so
    /// the bytes alone do not pin it.
    ty: &'static str,
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
        ty: type_name::<T>(),
        bytes,
        decodes,
    }
}

/// The corpus. `docs/spec/format-v1.md` gives every value's hex by hand.
fn corpus() -> Vec<Case> {
    let blob_content = b"Periapsis\n".to_vec();
    let blob_case = Case {
        name: "blob-envelope",
        ty: type_name::<Envelope<'static>>(),
        bytes: blob(&blob_content),
        decodes: Box::new(move |file| match decode_envelope(file) {
            Ok(env) if env.kind == ObjectKind::Blob && env.body == blob_content => Ok(()),
            Ok(env) => Err(format!("decoded {env:?}")),
            Err(error) => Err(error.to_string()),
        }),
    };
    let operation_root = Operation {
        parents: BTreeSet::new(),
        workspace: "default".to_owned(),
        view: HashAlg::Blake3.hash(b""),
    };
    let operation_merge = Operation {
        parents: BTreeSet::from([HashAlg::Blake3.hash(b""), HashAlg::Sha256.hash(b"abc")]),
        workspace: "default".to_owned(),
        view: HashAlg::Sha256.hash(b""),
    };
    let path = |p: &str| RepoPath::new(p).unwrap();
    let changeset_root = Changeset {
        parents: Vec::new(),
        change_id: std::array::from_fn(|i| i as u8),
        changes: BTreeMap::new(),
        description: Vec::new(),
    };
    let changeset_edit = Changeset {
        parents: vec![HashAlg::Blake3.hash(b"")],
        change_id: [0xff; 16],
        changes: BTreeMap::from([
            (
                path("README.md"),
                Entry::Put {
                    blob: HashAlg::Sha256.hash(b"abc"),
                    mode: Mode::Regular,
                },
            ),
            (
                path("bin/run"),
                Entry::Put {
                    blob: HashAlg::Blake3.hash(b""),
                    mode: Mode::Executable,
                },
            ),
            (path("src/old.rs"), Entry::Delete),
        ]),
        description: b"fix".to_vec(),
    };
    let one = BTreeSet::from([HashAlg::Blake3.hash(b"")]);
    let view = View {
        workspaces: BTreeMap::from([(
            "default".to_owned(),
            WorkspaceEntry {
                working_copy: one.clone(),
                heads: one,
            },
        )]),
    };
    let two = BTreeSet::from([HashAlg::Blake3.hash(b""), HashAlg::Sha256.hash(b"abc")]);
    let view_conflict = View {
        workspaces: BTreeMap::from([(
            "default".to_owned(),
            WorkspaceEntry {
                working_copy: two.clone(),
                heads: two,
            },
        )]),
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
        case("path-nested", path("src/π/main.rs")),
        blob_case,
        object_case(
            "changeset-root",
            changeset_root,
            Changeset::to_envelope,
            Changeset::from_envelope,
        ),
        object_case(
            "changeset-edit",
            changeset_edit,
            Changeset::to_envelope,
            Changeset::from_envelope,
        ),
        object_case("view", view, View::to_envelope, View::from_envelope),
        object_case(
            "view-conflict",
            view_conflict,
            View::to_envelope,
            View::from_envelope,
        ),
        object_case(
            "operation-root",
            operation_root,
            Operation::to_envelope,
            Operation::from_envelope,
        ),
        object_case(
            "operation-merge",
            operation_merge,
            Operation::to_envelope,
            Operation::from_envelope,
        ),
    ]
}

/// An object is pinned as its whole envelope, like the blob.
fn object_case<T: PartialEq + Debug + 'static>(
    name: &'static str,
    value: T,
    to_envelope: fn(&T) -> Vec<u8>,
    from_envelope: fn(&[u8]) -> Result<T, DecodeError>,
) -> Case {
    Case {
        name,
        ty: type_name::<T>(),
        bytes: to_envelope(&value),
        decodes: Box::new(move |file| match from_envelope(file) {
            Ok(decoded) if decoded == value => Ok(()),
            Ok(decoded) => Err(format!("decoded {decoded:?}, expected {value:?}")),
            Err(error) => Err(error.to_string()),
        }),
    }
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
                    failures.push(format!(
                        "{}.bin: file does not decode to its value: {error}",
                        case.name
                    ));
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

/// The committed golden files.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/v1")
}

#[test]
fn golden_v1_corpus_matches() {
    let value = std::env::var_os("PERI_GOLDEN_CREATE");
    let create = create_mode(value.as_deref());
    // The mode follows the variable, so a plain run fails on a missing file
    // instead of writing it.
    assert_eq!(
        create,
        value.as_deref() == Some(OsStr::new("1")),
        "create mode must follow PERI_GOLDEN_CREATE"
    );
    let failures = check_golden(&golden_dir(), &corpus(), create);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A committed file with no case would pin nothing: no test would compare it
/// with the encoder. (`golden_v1_corpus_matches` shows the other direction,
/// that each case has a file; checking it here too would fail create mode,
/// which writes a missing file while this test lists them.)
#[test]
fn every_golden_file_is_a_corpus_case() {
    let cases: BTreeSet<String> = corpus()
        .iter()
        .map(|case| format!("{}.bin", case.name))
        .collect();
    let strays: Vec<String> = fs::read_dir(golden_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "bin"))
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .filter(|name| !cases.contains(name))
        .collect();
    assert!(
        strays.is_empty(),
        "golden files with no corpus case: {strays:?}"
    );
}

/// Each value has the type its worked example names: `uvarint` is `u64`,
/// `seq` is `Vec`, `set` is `BTreeSet`, `map` is `BTreeMap` and `pair` is a
/// tuple. The bytes cannot show it: `u64::from(u32::MAX)` encodes exactly like
/// `u32::MAX`.
#[test]
fn golden_corpus_has_the_spec_types() {
    let types: Vec<(&str, &str)> = corpus().iter().map(|case| (case.name, case.ty)).collect();
    assert_eq!(
        types,
        [
            ("uvarint-0", type_name::<u64>()),
            ("uvarint-127", type_name::<u64>()),
            ("uvarint-128", type_name::<u64>()),
            ("uvarint-300", type_name::<u64>()),
            ("uvarint-max", type_name::<u64>()),
            ("u32-max", type_name::<u32>()),
            ("bool-false", type_name::<bool>()),
            ("bool-true", type_name::<bool>()),
            ("bytes-empty", type_name::<Bytes>()),
            ("bytes-short", type_name::<Bytes>()),
            ("string-empty", type_name::<String>()),
            ("string-ascii", type_name::<String>()),
            ("string-utf8", type_name::<String>()),
            ("hashref-blake3", type_name::<HashRef>()),
            ("hashref-sha256", type_name::<HashRef>()),
            ("option-none", type_name::<Option<u64>>()),
            ("option-some", type_name::<Option<String>>()),
            ("seq-u64", type_name::<Vec<u64>>()),
            ("set-string", type_name::<BTreeSet<String>>()),
            ("map-string-u64", type_name::<BTreeMap<String, u64>>()),
            (
                "map-nested",
                type_name::<BTreeMap<String, BTreeMap<u64, Option<Bytes>>>>()
            ),
            ("pair", type_name::<(String, bool)>()),
            ("path-nested", type_name::<RepoPath>()),
            ("blob-envelope", type_name::<Envelope<'static>>()),
            ("changeset-root", type_name::<Changeset>()),
            ("changeset-edit", type_name::<Changeset>()),
            ("view", type_name::<View>()),
            ("view-conflict", type_name::<View>()),
            ("operation-root", type_name::<Operation>()),
            ("operation-merge", type_name::<Operation>()),
        ]
    );
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

    // The harness compares the bytes first, then decodes the file: the file
    // no longer matches the encoder, and no longer holds the value.
    assert_eq!(
        check_golden(&dir, &cases, false),
        [
            "string-ascii.bin: encoding changed (file 10 bytes, encoder 10 bytes)",
            "string-ascii.bin: file does not decode to its value: decoded \"pesiapsis\", expected \"periapsis\"",
        ]
    );
    let _ = fs::remove_dir_all(&dir);
}

/// The corpus with the encoder's output for four cases changed while their
/// committed files still decode to the values: only the byte comparison can
/// catch it. The new output has the file's length, is longer, is the file's
/// bytes and one more, and is the file's bytes less the last.
fn changed_encoder() -> (Vec<Case>, [&'static str; 4]) {
    let mut cases = corpus();
    for case in &mut cases {
        match case.name {
            "uvarint-0" => case.bytes = vec![0x01],
            "uvarint-127" => case.bytes = vec![0xff, 0x00],
            "uvarint-128" => case.bytes = vec![0x80, 0x01, 0x00],
            "uvarint-300" => case.bytes = vec![0xac],
            _ => {}
        }
    }
    let failures = [
        "uvarint-0.bin: encoding changed (file 1 bytes, encoder 1 bytes)",
        "uvarint-127.bin: encoding changed (file 1 bytes, encoder 2 bytes)",
        "uvarint-128.bin: encoding changed (file 2 bytes, encoder 3 bytes)",
        "uvarint-300.bin: encoding changed (file 2 bytes, encoder 1 bytes)",
    ];
    (cases, failures)
}

#[test]
fn golden_encoder_change_fails() {
    let dir = scratch_dir("encoder");
    assert!(check_golden(&dir, &corpus(), true).is_empty());
    let (changed, failures) = changed_encoder();
    assert_eq!(check_golden(&dir, &changed, false), failures);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn golden_encoder_change_fails_in_create_mode() {
    // Create mode writes only missing files: existing ones still fail, and
    // are left as committed.
    let dir = scratch_dir("encoder-create");
    assert!(check_golden(&dir, &corpus(), true).is_empty());
    let (changed, failures) = changed_encoder();
    assert_eq!(check_golden(&dir, &changed, true), failures);
    assert_eq!(fs::read(dir.join("uvarint-0.bin")).unwrap(), [0x00]);
    assert_eq!(fs::read(dir.join("uvarint-127.bin")).unwrap(), [0x7f]);
    assert_eq!(fs::read(dir.join("uvarint-128.bin")).unwrap(), [0x80, 0x01]);
    assert_eq!(fs::read(dir.join("uvarint-300.bin")).unwrap(), [0xac, 0x02]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn golden_undecodable_file_fails() {
    // The encoder and the file agree on bytes that are not the value: only
    // the decode check can catch it. A typed case must use the whole file, and
    // the blob case must hold a blob with the corpus content.
    let undecodable: [(&str, Vec<u8>, &str); 4] = [
        (
            "uvarint-0",
            vec![0x80],
            "value at byte 0 is truncated; 1 more bytes needed",
        ),
        (
            "uvarint-0",
            vec![0x00, 0x00],
            "1 unexpected bytes after the value, from byte 1",
        ),
        // The blob's worked example without its final newline.
        (
            "blob-envelope",
            vec![
                0x01, 0x01, 0x50, 0x65, 0x72, 0x69, 0x61, 0x70, 0x73, 0x69, 0x73,
            ],
            "decoded Envelope { kind: Blob, body: [80, 101, 114, 105, 97, 112, 115, 105, 115] }",
        ),
        // A reserved kind, which fails to decode.
        (
            "blob-envelope",
            vec![0x01, 0x05],
            "object kind 0x05 is reserved and not yet specified",
        ),
    ];
    for (name, bytes, failure) in undecodable {
        let dir = scratch_dir("undecodable");
        let mut cases = corpus();
        cases.retain(|case| case.name == name);
        cases[0].bytes = bytes;
        assert!(check_golden(&dir, &cases, true).is_empty());
        assert_eq!(
            check_golden(&dir, &cases, false),
            [format!(
                "{name}.bin: file does not decode to its value: {failure}"
            )]
        );
        let _ = fs::remove_dir_all(&dir);
    }
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
fn golden_create_mode_makes_a_missing_directory() {
    // A new golden directory does not exist until create mode writes its
    // first file.
    let parent = scratch_dir("new-dir");
    let dir = parent.join("v1");
    let cases = corpus();
    assert!(check_golden(&dir, &cases, true).is_empty());
    for case in &cases {
        assert_eq!(
            fs::read(dir.join(format!("{}.bin", case.name))).unwrap(),
            case.bytes
        );
    }
    let _ = fs::remove_dir_all(&parent);
}

#[test]
fn create_mode_is_exactly_one() {
    assert!(create_mode(Some(OsStr::new("1"))));
    for value in ["0", "2", "true", "", " 1", "1 ", "01", "10", "yes"] {
        assert!(!create_mode(Some(OsStr::new(value))), "{value:?}");
    }
    assert!(!create_mode(None));
}
