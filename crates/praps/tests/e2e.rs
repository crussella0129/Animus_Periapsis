//! End to end through the built `praps` binary on the real filesystem (NTFS
//! on Windows, ext4 on Linux): a repository's first life, and a name with no
//! UTF-8 form (Sprint 2 test plan, E2E; INT-0014 AC-1, AC-2; INT-0004 AC-7).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use sim::Rng;

fn claim(label: &str) -> PathBuf {
    let mut rng = Rng::new(u64::from(std::process::id()) ^ (label.len() as u64) << 24);
    loop {
        let dir = std::env::temp_dir().join(format!("praps-e2e-{label}-{:016x}", rng.next_u64()));
        match fs::create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

fn praps(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_praps"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

/// The command's stdout lines, after checking it exited 0.
fn ok(args: &[&str], cwd: &Path) -> Vec<String> {
    let output = praps(args, cwd);
    assert_eq!(output.status.code(), Some(0), "{args:?}: {output:?}");
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn write(root: &Path, path: &str, content: &[u8]) {
    let file = path.split('/').fold(root.to_path_buf(), |p, c| p.join(c));
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, content).unwrap();
}

#[test]
fn praps_end_to_end() {
    let dir = claim("life");
    let root = dir.join("project");
    ok(&["init", "project"], &dir);
    write(&root, "README.md", b"# project\n");
    write(&root, "src/main.rs", b"fn main() {}\n");
    write(&root, ".gitignore", b"target/\n*.log\n");
    write(&root, "target/debug/app", b"binary");
    write(&root, "run.log", b"noise");
    assert_eq!(
        ok(&["status"], &root),
        ["A .gitignore", "A README.md", "A src/main.rs"]
    );
    // Unchanged: the same answer, and nothing new in the op log.
    let ops = |root: &Path| fs::read_dir(root.join(".praps/store/ops")).unwrap().count();
    let before = ops(&root);
    assert_eq!(ok(&["status"], &root).len(), 3);
    assert_eq!(ops(&root), before);

    ok(&["new"], &root);
    assert!(ok(&["status"], &root).is_empty());
    write(&root, "README.md", b"# project\n\nMore.\n");
    fs::remove_file(root.join("src/main.rs")).unwrap();
    write(&root, "src/lib.rs", b"pub fn f() {}\n");
    write(&root, "notes.log", b"still ignored");
    assert_eq!(
        ok(&["status"], &root),
        ["M README.md", "A src/lib.rs", "D src/main.rs"]
    );
    assert_eq!(ok(&["verify"], &root), ["clean"]);
    assert_eq!(ok(&["verify", "project"], &dir), ["clean"]);
    let _ = fs::remove_dir_all(&dir);
}

/// A name with no UTF-8 form, made with the platform's own API: reported by
/// its escaped form, never tracked, and left untouched.
#[test]
fn praps_status_reports_a_non_utf8_name() {
    let dir = claim("raw");
    let root = dir.join("project");
    ok(&["init", "project"], &dir);
    #[cfg(unix)]
    let (name, shown) = {
        use std::os::unix::ffi::OsStringExt;
        (
            std::ffi::OsString::from_vec(vec![b'r', 0xff, b'w']),
            r"r\xffw",
        )
    };
    #[cfg(windows)]
    let (name, shown) = {
        use std::os::windows::ffi::OsStringExt;
        (
            std::ffi::OsString::from_wide(&[u16::from(b'r'), 0xd800, u16::from(b'w')]),
            r"r\u{d800}w",
        )
    };
    fs::write(root.join(&name), b"untouched").unwrap();
    write(&root, "plain", b"p");
    assert_eq!(
        ok(&["status"], &root),
        [
            "A plain".to_owned(),
            format!("skipped {shown} (name is not UTF-8)")
        ]
    );
    assert_eq!(fs::read(root.join(&name)).unwrap(), b"untouched");
    // Twice: still reported, still untracked, still untouched.
    assert_eq!(ok(&["status"], &root).len(), 2);
    assert_eq!(fs::read(root.join(&name)).unwrap(), b"untouched");
    assert_eq!(ok(&["verify"], &root), ["clean"]);
    let _ = fs::remove_dir_all(&dir);
}

/// More names with no UTF-8 form, and a directory with one holding a file:
/// each reported by its escaped name, its contents never tracked, nothing
/// touched, siblings snapshotted (INT-0004 AC-7; test critique C-005).
#[test]
fn praps_status_reports_non_utf8_names_and_directories() {
    let dir = claim("raw-more");
    let root = dir.join("project");
    ok(&["init", "project"], &dir);
    #[cfg(unix)]
    let names: Vec<(std::ffi::OsString, &str)> = {
        use std::os::unix::ffi::OsStringExt;
        vec![
            (std::ffi::OsString::from_vec(vec![0xfe, b'a']), r"\xfea"),
            (std::ffi::OsString::from_vec(vec![b'a', 0x80]), r"a\x80"),
            (std::ffi::OsString::from_vec(vec![b'd', 0xc3]), r"d\xc3"),
        ]
    };
    #[cfg(windows)]
    let names: Vec<(std::ffi::OsString, &str)> = {
        use std::os::windows::ffi::OsStringExt;
        vec![
            (
                std::ffi::OsString::from_wide(&[0xdc00, u16::from(b'a')]),
                r"\u{dc00}a",
            ),
            (
                std::ffi::OsString::from_wide(&[u16::from(b'a'), 0xd800, 0xd801]),
                r"a\u{d800}\u{d801}",
            ),
            (
                std::ffi::OsString::from_wide(&[u16::from(b'd'), 0xdfff]),
                r"d\u{dfff}",
            ),
        ]
    };
    // The first two are files; the last is a directory holding a file.
    fs::write(root.join(&names[0].0), b"one").unwrap();
    fs::write(root.join(&names[1].0), b"two").unwrap();
    fs::create_dir(root.join(&names[2].0)).unwrap();
    fs::write(root.join(&names[2].0).join("inner"), b"inside").unwrap();
    write(&root, "sibling", b"s");
    let mut expected: Vec<String> = names
        .iter()
        .map(|(_, shown)| format!("skipped {shown} (name is not UTF-8)"))
        .collect();
    expected.sort();
    expected.insert(0, "A sibling".to_owned());
    assert_eq!(ok(&["status"], &root), expected);
    assert_eq!(fs::read(root.join(&names[0].0)).unwrap(), b"one");
    assert_eq!(fs::read(root.join(&names[1].0)).unwrap(), b"two");
    assert_eq!(
        fs::read(root.join(&names[2].0).join("inner")).unwrap(),
        b"inside"
    );
    assert_eq!(ok(&["verify"], &root), ["clean"]);
    let _ = fs::remove_dir_all(&dir);
}

/// Undo and restore through the binary on the real filesystem: files come
/// back exactly, every op stays listed, and the store stays clean (INT-0009
/// AC-1 to AC-3; Sprint 3 test plan, E2E).
#[test]
fn praps_history_end_to_end() {
    let dir = claim("history");
    let root = dir.join("project");
    ok(&["init", "project"], &dir);
    write(&root, "README.md", b"# project\n");
    write(&root, "src/main.rs", b"fn main() {}\n");
    write(&root, ".gitignore", b"*.log\n");
    write(&root, "build.log", b"ignored");
    assert_eq!(
        ok(&["status"], &root),
        ["A .gitignore", "A README.md", "A src/main.rs"]
    );
    let read = |path: &str| fs::read(root.join(path)).ok();

    ok(&["new"], &root);
    write(&root, "README.md", b"# project\n\nMore.\n");
    fs::remove_file(root.join("src/main.rs")).unwrap();
    write(&root, "docs/guide.md", b"guide");
    assert_eq!(
        ok(&["status"], &root),
        ["M README.md", "A docs/guide.md", "D src/main.rs"]
    );

    // Undo: back to the files before `new`.
    let undone = ok(&["undo"], &root);
    assert!(undone[0].starts_with("undid the last op"), "{undone:?}");
    assert_eq!(read("README.md").unwrap(), b"# project\n");
    assert_eq!(read("src/main.rs").unwrap(), b"fn main() {}\n");
    assert_eq!(read("docs/guide.md"), None);
    assert!(!root.join("docs").exists());
    assert_eq!(
        read("build.log").unwrap(),
        b"ignored",
        "ignored files are untouched"
    );

    // Restore the snapshot undo took: the edits are back.
    let log = ok(&["log"], &root);
    let edits = log[1].split(' ').next().unwrap();
    ok(&["restore", edits], &root);
    assert_eq!(read("README.md").unwrap(), b"# project\n\nMore.\n");
    assert_eq!(read("src/main.rs"), None);
    assert_eq!(read("docs/guide.md").unwrap(), b"guide");
    assert_eq!(
        ok(&["status"], &root),
        ["M README.md", "A docs/guide.md", "D src/main.rs"]
    );

    // Every op is still listed: none was removed, and `restore` added one.
    let after = ok(&["log"], &root);
    assert_eq!(after.len(), log.len() + 1, "{after:?}");
    for line in &log {
        assert!(after.contains(line), "{line} was removed");
    }
    assert!(!root.join(".praps/working-copy").exists());
    assert_eq!(ok(&["verify"], &root), ["clean"]);
    let _ = fs::remove_dir_all(&dir);
}

/// `log`, `undo`, `restore` and `new` report a name with no UTF-8 form as
/// `status` does, after their own output, on success and on failure, and
/// keep their exit codes (Sprint 4 T-009, backlog T-019; INT-0004 AC-7).
#[test]
fn praps_commands_report_skipped_names() {
    let dir = claim("raw-commands");
    let root = dir.join("project");
    ok(&["init", "project"], &dir);
    #[cfg(unix)]
    let (name, shown) = {
        use std::os::unix::ffi::OsStringExt;
        (std::ffi::OsString::from_vec(vec![b'q', 0xff]), r"q\xff")
    };
    #[cfg(windows)]
    let (name, shown) = {
        use std::os::windows::ffi::OsStringExt;
        (
            std::ffi::OsString::from_wide(&[u16::from(b'q'), 0xd800]),
            r"q\u{d800}",
        )
    };
    let skipped = format!("skipped {shown} (name is not UTF-8)");
    fs::write(root.join(&name), b"untouched").unwrap();

    // A failed `undo` (nothing to undo yet) and a failed `restore` (an
    // unknown op) still report, after nothing, and exit 1 with the error on
    // stderr (test critique C-006).
    let output = praps(&["undo"], &root);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{skipped}\n")
    );
    let output = praps(&["restore", "0000000000000000"], &root);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{skipped}\n")
    );
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("no op matches `0000000000000000`")
    );

    // `new`, `log`, `undo` and `restore`: their own lines, then the report.
    write(&root, "a", b"1");
    let lines = ok(&["new"], &root);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("new change "));
    assert_eq!(lines[1], skipped);
    let lines = ok(&["log"], &root);
    assert_eq!(lines.last(), Some(&skipped), "{lines:?}");
    assert_eq!(lines.iter().filter(|line| **line == skipped).count(), 1);
    let target = lines[0][..12].to_owned();
    let lines = ok(&["undo"], &root);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("undid the last op"));
    assert_eq!(lines[1], skipped);
    let lines = ok(&["restore", &target], &root);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("restored the working copy"));
    assert_eq!(lines[1], skipped);
    assert_eq!(fs::read(root.join(&name)).unwrap(), b"untouched");
    let _ = fs::remove_dir_all(&dir);
}
