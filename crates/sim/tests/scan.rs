//! `Fs::scan` on the simulator: entry kinds, the executable bit, names with
//! no UTF-8 form, and the escaped display (Sprint 2 T-004; INT-0004 AC-7,
//! INT-0005 AC-1, AC-2).

use std::collections::BTreeSet;
use std::path::Path;

use sim::{DirEntry, Fs, Kind, Mode, Name, Node, RawName, Rng, SimFs, is_crash};

fn utf8(name: &str, kind: Kind) -> DirEntry {
    DirEntry {
        name: Name::Utf8(name.to_owned()),
        kind,
    }
}

const FILE: Kind = Kind::File { executable: false };

/// Every kind of entry the simulator models, sorted by display form, with
/// the executable bit following the filesystem's capability.
#[test]
fn scan_lists_kinds_sorted() {
    for has_bit in [true, false] {
        let fs = SimFs::new(3, Mode::Strict);
        fs.set_has_executable_bit(has_bit);
        assert_eq!(fs.has_executable_bit(), has_bit);
        fs.create_dir(Path::new("d")).unwrap();
        fs.create_new(Path::new("d/run")).unwrap();
        fs.make_executable(Path::new("d/run"), true).unwrap();
        fs.create_new(Path::new("d/plain")).unwrap();
        fs.create_dir(Path::new("d/dir")).unwrap();
        fs.make_symlink(Path::new("d/link")).unwrap();
        fs.make_other(Path::new("d/fifo")).unwrap();
        assert_eq!(
            fs.scan(Path::new("d")).unwrap(),
            [
                utf8("dir", Kind::Dir),
                utf8("fifo", Kind::Other),
                utf8("link", Kind::Symlink),
                utf8("plain", FILE),
                utf8(
                    "run",
                    Kind::File {
                        executable: has_bit
                    }
                ),
            ]
        );
        // A symlink or special entry is not a file to read or write.
        assert!(fs.read(Path::new("d/link")).is_err());
        assert!(fs.write(Path::new("d/fifo"), b"x").is_err());
        let snapshot = fs.snapshot();
        assert_eq!(snapshot.get("d/link"), Some(&Node::Symlink));
        assert_eq!(snapshot.get("d/fifo"), Some(&Node::Other));
        assert_eq!(snapshot.get("d/run"), Some(&Node::Executable(Vec::new())));
    }
}

/// Names with no UTF-8 form are kept as given and shown escaped, and they
/// are durable at once. (A UTF-8 name spelled like an escape holds a `\`,
/// which a Windows `Path` takes as a separator, so the display property test
/// below covers that case.)
#[test]
fn scan_reports_raw_names() {
    let fs = SimFs::new(5, Mode::Strict);
    fs.create_dir(Path::new("d")).unwrap();
    fs.sync_dir(Path::new("")).unwrap();
    let bytes = RawName::Bytes(vec![b'a', 0xff, b'b']);
    let wide = RawName::Wide(vec![u16::from(b'w'), 0xd800]);
    fs.make_raw(Path::new("d"), bytes.clone(), b"raw").unwrap();
    fs.make_raw(Path::new("d"), wide.clone(), b"").unwrap();
    fs.create_new(Path::new("d/plain")).unwrap();
    let entries = fs.scan(Path::new("d")).unwrap();
    let shown: Vec<String> = entries.iter().map(|e| e.name.display()).collect();
    assert_eq!(shown, [r"a\xffb", "plain", r"w\u{d800}"]);
    assert_eq!(entries[0].name, Name::Raw(bytes));
    assert_eq!(entries[2].name, Name::Raw(wide));
    assert_eq!(entries[0].name.as_utf8(), None);
    assert_eq!(entries[1].name.as_utf8(), Some("plain"));
    // `plain` was never synced into `d`, so a crash may lose it; the raw
    // names are durable.
    let recovered = fs.recover();
    let kept: Vec<String> = recovered
        .scan(Path::new("d"))
        .unwrap()
        .iter()
        .map(|e| e.name.display())
        .collect();
    assert!(kept.contains(&r"a\xffb".to_owned()) && kept.contains(&r"w\u{d800}".to_owned()));
    assert_eq!(
        recovered.snapshot().get(r"d/a\xffb"),
        Some(&Node::File(b"raw".to_vec()))
    );
}

/// `scan` is one syscall, traced, and fails at the crash point like the rest.
#[test]
fn scan_is_one_syscall() {
    let fs = SimFs::new(0, Mode::Strict);
    fs.create_dir(Path::new("d")).unwrap();
    let before = fs.syscalls();
    fs.scan(Path::new("d")).unwrap();
    assert_eq!(fs.syscalls(), before + 1);
    assert_eq!(fs.trace().last().map(String::as_str), Some("scan d"));
    fs.set_crash_at(Some(before + 2));
    assert!(is_crash(&fs.scan(Path::new("d")).unwrap_err()));
}

/// A raw name's display never equals a UTF-8 name's: every `\` a UTF-8 name
/// holds is doubled, so the `\xNN` and `\u{dNNN}` escapes appear only in raw
/// names. Random names over an alphabet rich in backslashes and escape-like
/// text, both forms of raw name.
#[test]
fn raw_display_never_equals_a_utf8_display() {
    let mut rng = Rng::new(42);
    let alphabet: Vec<char> = r"ab\x{}ud80f".chars().chain(['\u{1}', 'é']).collect();
    let pick = |rng: &mut Rng| alphabet[rng.below(alphabet.len() as u64) as usize];
    let mut utf8_shown = BTreeSet::new();
    for _ in 0..20_000 {
        let len = rng.below(7) as usize;
        let name: String = (0..len).map(|_| pick(&mut rng)).collect();
        utf8_shown.insert(Name::Utf8(name).display());
    }
    let mut checked = 0;
    for _ in 0..20_000 {
        let len = rng.below(6) as usize;
        let mut text: String = (0..len).map(|_| pick(&mut rng)).collect();
        // Bytes: the text with one invalid byte somewhere.
        let mut bytes = text.clone().into_bytes();
        let at = rng.below(bytes.len() as u64 + 1) as usize;
        bytes.insert(at, 0xff);
        let shown = Name::Raw(RawName::Bytes(bytes)).display();
        assert!(!utf8_shown.contains(&shown), "{shown}");
        // Code units: the text with one unpaired surrogate.
        text.push('q');
        let mut units: Vec<u16> = text.encode_utf16().collect();
        units.insert(rng.below(units.len() as u64) as usize, 0xdc00);
        let shown = Name::Raw(RawName::Wide(units)).display();
        assert!(!utf8_shown.contains(&shown), "{shown}");
        checked += 2;
    }
    assert_eq!(checked, 40_000);
    // The case the plan critique named.
    assert_ne!(
        Name::Utf8(r"a\xffb".to_owned()).display(),
        Name::Raw(RawName::Bytes(vec![b'a', 0xff, b'b'])).display()
    );
}

/// The simulator refuses a NUL in a name, and a "raw" name that has a UTF-8
/// form, so raw-name keys can never collide with a real name (code review
/// CR-12).
#[test]
fn simfs_refuses_nul_names_and_false_raw_names() {
    let fs = SimFs::new(0, Mode::Strict);
    fs.create_dir(Path::new("d")).unwrap();
    assert_eq!(
        fs.create_new(Path::new("d/a\0b")).unwrap_err().kind(),
        std::io::ErrorKind::InvalidInput
    );
    for raw in [
        RawName::Bytes(b"plain".to_vec()),
        RawName::Wide("plain".encode_utf16().collect()),
    ] {
        assert_eq!(
            fs.make_raw(Path::new("d"), raw, b"").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }
    assert!(fs.scan(Path::new("d")).unwrap().is_empty());
}
