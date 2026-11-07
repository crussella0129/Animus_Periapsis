//! The provisional `changeset` and `view` bodies and repository paths
//! (`docs/spec/format-v1.md`; INT-0006 AC-1, AC-5; Sprint 2 T-002).

use codec::{
    Changeset, DecodeError, Entry, Mode, ObjectKind, PathError, RepoPath, View, encode_object,
    from_slice, to_vec,
};
use hash::HashAlg;

fn path(text: &str) -> RepoPath {
    RepoPath::new(text).unwrap()
}

/// A changeset body with no parents, change ID zero, the given encoded
/// changes map, and an empty description.
fn changeset_body(changes: &[u8]) -> Vec<u8> {
    [&[0x00][..], &[0u8; 16], changes, &[0x00]].concat()
}

/// One map entry, `"a" → <entry bytes>`.
fn one_change(key: &[u8], entry: &[u8]) -> Vec<u8> {
    [&[0x01, key.len() as u8][..], key, entry].concat()
}

#[test]
fn repo_path_rejects_invalid_forms() {
    for (text, error) in [
        ("", PathError::Empty),
        ("/a", PathError::EmptyComponent),
        ("a/", PathError::EmptyComponent),
        ("a//b", PathError::EmptyComponent),
        (".", PathError::DotComponent),
        ("..", PathError::DotComponent),
        ("a/./b", PathError::DotComponent),
        ("a/..", PathError::DotComponent),
        ("a\0b", PathError::ForbiddenCharacter('\0')),
        ("a\\b", PathError::ForbiddenCharacter('\\')),
    ] {
        assert_eq!(RepoPath::new(text), Err(error), "{text:?}");
    }
}

#[test]
fn repo_path_accepts_valid_forms() {
    for text in [
        "a",
        "a/b.c",
        "ü/x",
        ".gitignore",
        "...",
        "a/.b/c..d",
        "CON",
        "a b",
    ] {
        assert_eq!(path(text).as_str(), text);
    }
    let nested = path("src/π/main.rs");
    assert_eq!(nested.name(), "main.rs");
    assert_eq!(nested.parent(), Some(path("src/π")));
    assert_eq!(path("top").parent(), None);
    assert_eq!(path("src").join("lib.rs"), Ok(path("src/lib.rs")));
    assert!(path("src").join("a/b").is_err());
    assert!(nested.starts_with(&path("src")));
    assert!(!path("srcx/a").starts_with(&path("src")));
}

#[test]
fn changeset_rejects_unsorted_paths() {
    let mut changes = vec![0x02];
    changes.extend_from_slice(&[0x01, b'b', 0x00]);
    changes.extend_from_slice(&[0x01, b'a', 0x00]);
    assert_eq!(
        from_slice::<Changeset>(&changeset_body(&changes)),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 21 })
    );
}

#[test]
fn changeset_rejects_duplicate_paths() {
    let mut changes = vec![0x02];
    changes.extend_from_slice(&[0x01, b'a', 0x00]);
    changes.extend_from_slice(&[0x01, b'a', 0x00]);
    assert_eq!(
        from_slice::<Changeset>(&changeset_body(&changes)),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 21 })
    );
}

#[test]
fn view_rejects_unsorted_heads() {
    let low = HashAlg::Blake3.hash(b"");
    let high = HashAlg::Sha256.hash(b"");
    let body = [
        &[0x02][..],
        &to_vec(&high),
        &to_vec(&low),
        &[0x00], // no workspaces
    ]
    .concat();
    assert_eq!(
        from_slice::<View>(&body),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 34 })
    );
}

#[test]
fn view_rejects_duplicate_workspaces() {
    let id = to_vec(&HashAlg::Blake3.hash(b""));
    let body = [&[0x00, 0x02][..], &[0x01, b'w'], &id, &[0x01, b'w'], &id].concat();
    assert_eq!(
        from_slice::<View>(&body),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 37 })
    );
}

#[test]
fn changeset_rejects_unknown_entry_tag() {
    let body = changeset_body(&one_change(b"a", &[0x02]));
    assert_eq!(
        from_slice::<Changeset>(&body),
        Err(DecodeError::UnknownEntryTag { at: 20, byte: 0x02 })
    );
}

#[test]
fn changeset_rejects_unknown_mode() {
    let put = [&[0x01][..], &to_vec(&HashAlg::Blake3.hash(b"")), &[0x03]].concat();
    let body = changeset_body(&one_change(b"a", &put));
    assert_eq!(
        from_slice::<Changeset>(&body),
        Err(DecodeError::UnknownMode { at: 54, byte: 0x03 })
    );
}

#[test]
fn changeset_rejects_symlink_mode() {
    let put = [
        &[0x01][..],
        &to_vec(&HashAlg::Blake3.hash(b"")),
        &[Mode::RESERVED_SYMLINK],
    ]
    .concat();
    let body = changeset_body(&one_change(b"a", &put));
    let error = from_slice::<Changeset>(&body).unwrap_err();
    assert_eq!(error, DecodeError::ReservedMode { at: 54, byte: 0x02 });
    assert_eq!(
        error.to_string(),
        "file mode 0x02 at byte 54 is reserved and not yet specified"
    );
}

#[test]
fn changeset_rejects_invalid_paths() {
    for key in [&b"/a"[..], b"a/", b".", b"a/..", b"a\\b", b"a\0b", b""] {
        let body = changeset_body(&one_change(key, &[0x00]));
        assert_eq!(
            from_slice::<Changeset>(&body),
            Err(DecodeError::InvalidPath { at: 18 }),
            "{key:?}"
        );
    }
}

#[test]
fn bodies_reject_trailing_bytes() {
    let changeset = Changeset {
        parents: Vec::new(),
        change_id: [7; 16],
        changes: [(path("a"), Entry::Delete)].into(),
        description: b"d".to_vec(),
    };
    let view = View::default();
    let mut changeset_bytes = to_vec(&changeset);
    let at = changeset_bytes.len();
    changeset_bytes.push(0x00);
    assert_eq!(
        from_slice::<Changeset>(&changeset_bytes),
        Err(DecodeError::TrailingBytes { at, remaining: 1 })
    );
    let mut view_bytes = to_vec(&view);
    view_bytes.push(0xff);
    assert_eq!(
        from_slice::<View>(&view_bytes),
        Err(DecodeError::TrailingBytes {
            at: 2,
            remaining: 1
        })
    );
}

#[test]
fn defined_kinds_are_accepted() {
    let changeset = Changeset {
        parents: vec![HashAlg::Blake3.hash(b"")],
        change_id: [1; 16],
        changes: Default::default(),
        description: Vec::new(),
    };
    assert_eq!(
        Changeset::from_envelope(&changeset.to_envelope()),
        Ok(changeset.clone())
    );
    let view = View::default();
    assert_eq!(View::from_envelope(&view.to_envelope()), Ok(view.clone()));
    assert_eq!(ObjectKind::from_id(0x02), Ok(ObjectKind::Changeset));
    assert_eq!(ObjectKind::from_id(0x04), Ok(ObjectKind::View));
    // An envelope of another kind is refused by name.
    assert_eq!(
        View::from_envelope(&changeset.to_envelope()),
        Err(DecodeError::UnexpectedObjectKind {
            expected: 0x04,
            found: 0x02
        })
    );
    assert_eq!(
        Changeset::from_envelope(&encode_object(ObjectKind::Blob, b"x")),
        Err(DecodeError::UnexpectedObjectKind {
            expected: 0x02,
            found: 0x01
        })
    );
}

#[test]
fn event_kind_stays_reserved() {
    assert_eq!(
        ObjectKind::from_id(0x05),
        Err(DecodeError::ReservedObjectKind { kind: 0x05 })
    );
}
