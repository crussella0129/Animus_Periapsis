//! The provisional `changeset` and `view` bodies and repository paths
//! (`docs/spec/format-v1.md`; INT-0006 AC-1, AC-5; Sprint 2 T-002; the view's
//! entries, Sprint 7 T-002).

use codec::{
    Changeset, DecodeError, Entry, Mode, ObjectKind, Operation, PathError, RepoPath, View,
    WorkspaceEntry, encode_object, from_slice, to_vec, typed_references,
};
use hash::{HashAlg, HashRef};

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

/// One view entry's bytes: `working_copy`, then `heads`.
fn entry_body(working_copy: &[HashRef], heads: &[HashRef]) -> Vec<u8> {
    let set = |ids: &[HashRef]| -> Vec<u8> {
        let mut bytes = vec![ids.len() as u8];
        for id in ids {
            bytes.extend(to_vec(id));
        }
        bytes
    };
    [set(working_copy), set(heads)].concat()
}

#[test]
fn view_rejects_unsorted_heads() {
    let low = HashAlg::Blake3.hash(b"");
    let high = HashAlg::Sha256.hash(b"");
    // One entry, "w": working copy {low}, heads out of order. The second
    // head starts at 3 + 34 + 1 + 33 = 71.
    let body = [&[0x01, 0x01, b'w'][..], &entry_body(&[low], &[high, low])].concat();
    assert_eq!(
        from_slice::<View>(&body),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 71 })
    );
}

#[test]
fn view_rejects_duplicate_workspaces() {
    let id = HashAlg::Blake3.hash(b"");
    let entry = entry_body(&[id], &[]);
    // The second name starts after the count, the first name and the first
    // entry: at 1 + 2 + 35 = 38.
    let body = [&[0x02, 0x01, b'w'][..], &entry, &[0x01, b'w'], &entry].concat();
    assert_eq!(
        from_slice::<View>(&body),
        Err(DecodeError::UnsortedOrDuplicateKey { at: 38 })
    );
}

#[test]
fn view_holds_one_entry_per_workspace() {
    let a = HashAlg::Blake3.hash(b"a");
    let b = HashAlg::Sha256.hash(b"b");
    let view = View {
        workspaces: [
            (
                "lane".to_owned(),
                WorkspaceEntry {
                    working_copy: [a, b].into(),
                    heads: [a, b].into(),
                },
            ),
            (
                "default".to_owned(),
                WorkspaceEntry {
                    working_copy: [a].into(),
                    heads: [a].into(),
                },
            ),
        ]
        .into(),
    };
    let body = to_vec(&view);
    // The map in name order: "default" first, then "lane", whose pointer
    // holds two changesets.
    let expected = [
        &[0x02, 0x07][..],
        b"default",
        &entry_body(&[a], &[a]),
        &[0x04],
        b"lane",
        &entry_body(&[a, b], &[a, b]),
    ]
    .concat();
    assert_eq!(body, expected);
    assert_eq!(from_slice::<View>(&body), Ok(view.clone()));
    assert_eq!(View::from_envelope(&view.to_envelope()), Ok(view));
}

#[test]
fn empty_workspace_names_are_rejected() {
    let id = HashAlg::Blake3.hash(b"");
    // A view whose one entry is named "": the name starts at 1.
    let body = [&[0x01, 0x00][..], &entry_body(&[id], &[])].concat();
    assert_eq!(
        from_slice::<View>(&body),
        Err(DecodeError::EmptyWorkspaceName { at: 1 })
    );
    // An operation made in "": no parents, then the name, at 1.
    let body = [&[0x00, 0x00][..], &to_vec(&id)].concat();
    assert_eq!(
        from_slice::<Operation>(&body),
        Err(DecodeError::EmptyWorkspaceName { at: 1 })
    );
}

#[test]
fn empty_working_copies_are_rejected() {
    let id = HashAlg::Blake3.hash(b"");
    // Entry "w" with no working copy: the set starts at 3.
    let body = [&[0x01, 0x01, b'w'][..], &entry_body(&[], &[id])].concat();
    assert_eq!(
        from_slice::<View>(&body),
        Err(DecodeError::EmptyWorkingCopy { at: 3 })
    );
}

#[test]
fn view_references_are_each_entry() {
    let h = |text: &[u8]| HashAlg::Blake3.hash(text);
    let view = View {
        workspaces: [
            (
                "b".to_owned(),
                WorkspaceEntry {
                    working_copy: [h(b"b-wc")].into(),
                    heads: [h(b"b-head")].into(),
                },
            ),
            (
                "a".to_owned(),
                WorkspaceEntry {
                    working_copy: [h(b"a-wc")].into(),
                    heads: [h(b"a-head-1"), h(b"a-head-2")].into(),
                },
            ),
        ]
        .into(),
    };
    // Entries in name order, each its working copy and then its heads, every
    // one a changeset.
    let a_heads: Vec<HashRef> = view.workspaces["a"].heads.iter().copied().collect();
    let expected: Vec<(HashRef, ObjectKind)> = [h(b"a-wc")]
        .into_iter()
        .chain(a_heads)
        .chain([h(b"b-wc"), h(b"b-head")])
        .map(|id| (id, ObjectKind::Changeset))
        .collect();
    assert_eq!(typed_references(&view.to_envelope()), Ok(expected));
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
            at: 1,
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

#[test]
fn references_of_each_kind() {
    use codec::{blob, references};
    let h = |s: &[u8]| HashAlg::Blake3.hash(s);
    assert_eq!(references(&blob(b"content")), Ok(Vec::new()));
    let changeset = Changeset {
        parents: vec![h(b"p2"), h(b"p1")],
        change_id: [3; 16],
        changes: [
            (
                path("a"),
                Entry::Put {
                    blob: h(b"a"),
                    mode: Mode::Regular,
                },
            ),
            (path("b"), Entry::Delete),
            (
                path("c"),
                Entry::Put {
                    blob: h(b"c"),
                    mode: Mode::Executable,
                },
            ),
        ]
        .into(),
        description: Vec::new(),
    };
    assert_eq!(
        references(&changeset.to_envelope()),
        Ok(vec![h(b"p2"), h(b"p1"), h(b"a"), h(b"c")])
    );
    let view = View {
        workspaces: [(
            "default".to_owned(),
            WorkspaceEntry {
                working_copy: [h(b"wc")].into(),
                heads: [h(b"head")].into(),
            },
        )]
        .into(),
    };
    assert_eq!(
        references(&view.to_envelope()),
        Ok(vec![h(b"wc"), h(b"head")])
    );
    let op = Operation {
        parents: [h(b"op")].into(),
        workspace: "default".to_owned(),
        view: h(b"view"),
    };
    assert_eq!(
        references(&op.to_envelope()),
        Ok(vec![h(b"op"), h(b"view")])
    );
    // A body that does not decode as its kind is an error, not "no references".
    assert!(references(&encode_object(ObjectKind::View, &[0x01])).is_err());
}
