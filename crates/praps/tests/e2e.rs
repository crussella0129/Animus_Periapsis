//! End to end through the built `praps` binary on the real filesystem (NTFS
//! on Windows, ext4 on Linux): a repository's first life, and a name with no
//! UTF-8 form (Sprint 2 test plan, E2E; INT-0014 AC-1, AC-2; INT-0004 AC-7).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

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
    let log = ok(&["ops"], &root);
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
    let after = ok(&["ops"], &root);
    assert_eq!(after.len(), log.len() + 1, "{after:?}");
    for line in &log {
        assert!(after.contains(line), "{line} was removed");
    }
    assert!(!root.join(".praps/working-copy").exists());
    assert_eq!(ok(&["verify"], &root), ["clean"]);
    let _ = fs::remove_dir_all(&dir);
}

/// `ops`, `log`, `describe`, `undo`, `restore` and `new` report a name with
/// no UTF-8 form as `status` does, after their own output, on success and
/// on failure, and keep their exit codes (Sprint 4 T-009, backlog T-019;
/// INT-0004 AC-7).
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

    // `new`, `ops`, `log`, `undo` and `restore`: their own lines, then the
    // report.
    write(&root, "a", b"1");
    let lines = ok(&["new"], &root);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("new change "));
    assert_eq!(lines[1], skipped);
    let lines = ok(&["ops"], &root);
    assert_eq!(lines.last(), Some(&skipped), "{lines:?}");
    assert_eq!(lines.iter().filter(|line| **line == skipped).count(), 1);
    let target = lines[0][..12].to_owned();
    // The list of changes records the working copy too, and reports what
    // it left alone (Sprint 6 T-009).
    let lines = ok(&["log"], &root);
    assert_eq!(lines.last(), Some(&skipped), "{lines:?}");
    assert_eq!(lines.iter().filter(|line| **line == skipped).count(), 1);
    assert!(lines[0].ends_with("(no description)"), "{lines:?}");
    // `describe` too, when it sets a description and when it has nothing
    // to change (Sprint 6 T-010).
    for said in ["described change ", "nothing changed: "] {
        let lines = ok(&["describe", "Add a"], &root);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].starts_with(said), "{lines:?}");
        assert_eq!(lines[1], skipped);
    }
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

/// The history session again, with every command after `init` run from a
/// directory two levels below the top: the same files come and go, the same
/// number of ops is listed, and the store stays clean (INT-0037 AC-3;
/// Sprint 6 T-008).
#[test]
fn praps_end_to_end_from_below() {
    let ops_listed = |from_below: bool| {
        let dir = claim(if from_below { "below" } else { "top" });
        let root = dir.join("project");
        ok(&["init", "project"], &dir);
        // The directory the commands run from is ignored, so that it is in
        // both sessions without being part of either's history.
        write(&root, ".gitignore", b"*.log\nhere/\n");
        write(&root, "here/deep/keep.log", b"a place to stand");
        let cwd = if from_below {
            root.join("here/deep")
        } else {
            root.clone()
        };
        write(&root, "README.md", b"# project\n");
        write(&root, "src/main.rs", b"fn main() {}\n");
        assert_eq!(
            ok(&["status"], &cwd),
            ["A .gitignore", "A README.md", "A src/main.rs"]
        );
        let read = |path: &str| fs::read(root.join(path)).ok();

        ok(&["new"], &cwd);
        write(&root, "README.md", b"# project\n\nMore.\n");
        fs::remove_file(root.join("src/main.rs")).unwrap();
        write(&root, "docs/guide.md", b"guide");
        assert_eq!(
            ok(&["status"], &cwd),
            ["M README.md", "A docs/guide.md", "D src/main.rs"]
        );

        let undone = ok(&["undo"], &cwd);
        assert!(undone[0].starts_with("undid the last op"), "{undone:?}");
        assert_eq!(read("README.md").unwrap(), b"# project\n");
        assert_eq!(read("src/main.rs").unwrap(), b"fn main() {}\n");
        assert_eq!(read("docs/guide.md"), None);
        assert!(!root.join("docs").exists());

        let log = ok(&["ops"], &cwd);
        let edits = log[1].split(' ').next().unwrap();
        ok(&["restore", edits], &cwd);
        assert_eq!(read("README.md").unwrap(), b"# project\n\nMore.\n");
        assert_eq!(read("src/main.rs"), None);
        assert_eq!(read("docs/guide.md").unwrap(), b"guide");
        assert_eq!(
            ok(&["status"], &cwd),
            ["M README.md", "A docs/guide.md", "D src/main.rs"]
        );
        assert_eq!(ok(&["verify"], &cwd), ["clean"]);
        assert!(!root.join(".praps/working-copy").exists());
        // No repository was made where the commands ran.
        assert!(!cwd.join(".praps").exists() || cwd == root);
        let listed = ok(&["ops"], &cwd).len();
        let _ = fs::remove_dir_all(&dir);
        listed
    };
    assert_eq!(ops_listed(true), ops_listed(false));
}

/// A first session that uses only words `praps` itself printed: the list,
/// then `help`, `init`, `status`, `describe`, `new`, `log`, `ops`, `undo`,
/// `restore` and `verify`. Every command exits 0 and prints what its step
/// names. With `from_below`, every command after `init` runs from a
/// directory two levels below the top (INT-0037 AC-2, AC-3; Sprint 6
/// T-010).
fn first_session(from_below: bool) {
    let dir = claim(if from_below {
        "first-below"
    } else {
        "first-top"
    });
    // The list comes first, and every word used below is taken from it.
    let list = ok(&[], &dir);
    let listed: Vec<&str> = list
        .iter()
        .filter_map(|line| line.strip_prefix("  "))
        .map(|line| line.split(' ').next().unwrap())
        .collect();
    let word = |name: &'static str| {
        assert!(listed.contains(&name), "`{name}` is not in {listed:?}");
        name
    };
    for name in [
        "init", "status", "describe", "new", "log", "ops", "undo", "restore", "verify",
    ] {
        let help = ok(&[word("help"), word(name)], &dir);
        assert!(
            help[0].starts_with(&format!("usage: praps {name}")),
            "{help:?}"
        );
    }

    assert_eq!(
        ok(&[word("init"), "project"], &dir),
        ["initialised a Periapsis repository in project"]
    );
    let root = dir.join("project");
    write(&root, "README.md", b"# project\n");
    write(&root, "src/app/main.rs", b"fn main() {}\n");
    // The directory the commands run from holds a tracked file in every
    // state the session visits, so it is never removed beneath them.
    let cwd = if from_below {
        root.join("src/app")
    } else {
        root.clone()
    };
    let read = |path: &str| fs::read(root.join(path)).ok();
    // The change a line of `describe` names.
    let described = |lines: &[String], first_line: &str| -> String {
        assert_eq!(lines.len(), 1, "{lines:?}");
        let rest = lines[0].strip_prefix("described change ").unwrap();
        let (id, shown) = rest.split_once(": ").unwrap();
        assert_eq!(id.len(), 12, "{lines:?}");
        assert_eq!(shown, first_line);
        id.to_owned()
    };

    assert_eq!(
        ok(&[word("status")], &cwd),
        ["A README.md", "A src/app/main.rs"]
    );
    let first = described(
        &ok(&[word("describe"), "Start the project"], &cwd),
        "Start the project",
    );
    let new = ok(&[word("new")], &cwd);
    assert!(new[0].starts_with("new change "), "{new:?}");
    assert!(ok(&["status"], &cwd).is_empty());

    write(&root, "README.md", b"# project\n\nMore.\n");
    write(&root, "src/app/lib.rs", b"pub fn f() {}\n");
    assert_eq!(ok(&["status"], &cwd), ["M README.md", "A src/app/lib.rs"]);
    let second = described(
        &ok(
            &["describe", "Add the library\n\nIt has one function."],
            &cwd,
        ),
        "Add the library",
    );
    assert_ne!(first, second);
    assert!(
        new[0].starts_with(&format!("new change {second}")),
        "{new:?}"
    );
    let both = [
        format!("{second} 2 paths Add the library"),
        format!("{first} 2 paths Start the project"),
    ];
    assert_eq!(ok(&[word("log")], &cwd), both);

    // Six operations, newest first: the second description, the recording
    // of the edits, `new`, the first description, the first recording, and
    // `init`.
    let ops = ok(&[word("ops")], &cwd);
    assert_eq!(ops.len(), 6, "{ops:?}");
    let op = |at: usize| ops[at].split(' ').next().unwrap();
    assert!(
        ops[0].ends_with(&format!(" change {second} 2 paths")),
        "{ops:?}"
    );
    assert!(
        ops[3].ends_with(&format!(" change {first} 2 paths")),
        "{ops:?}"
    );
    assert!(ops[5].ends_with(" 0 paths"), "{ops:?}");

    // `undo` takes the second description back, and no file changes.
    assert_eq!(
        ok(&[word("undo")], &cwd),
        [format!(
            "undid the last op: the working copy is back to op {}",
            op(1)
        )]
    );
    assert_eq!(
        ok(&["log"], &cwd),
        [
            format!("{second} 2 paths (no description)"),
            both[1].clone()
        ]
    );
    assert_eq!(read("src/app/lib.rs").unwrap(), b"pub fn f() {}\n");

    // `restore` of the describing operation gives it again.
    assert_eq!(
        ok(&[word("restore"), op(0)], &cwd),
        [format!("restored the working copy to op {}", op(0))]
    );
    assert_eq!(ok(&["log"], &cwd), both);

    // `restore` of the first description's operation writes the files of
    // that time, and `undo` brings the later ones back.
    ok(&["restore", op(3)], &cwd);
    assert_eq!(read("README.md").unwrap(), b"# project\n");
    assert_eq!(read("src/app/lib.rs"), None);
    assert_eq!(read("src/app/main.rs").unwrap(), b"fn main() {}\n");
    assert_eq!(ok(&["log"], &cwd), [both[1].clone()]);
    assert_eq!(ok(&["status"], &cwd), ["A README.md", "A src/app/main.rs"]);
    ok(&["undo"], &cwd);
    assert_eq!(read("README.md").unwrap(), b"# project\n\nMore.\n");
    assert_eq!(read("src/app/lib.rs").unwrap(), b"pub fn f() {}\n");
    assert_eq!(ok(&["log"], &cwd), both);

    assert_eq!(ok(&[word("verify")], &cwd), ["clean"]);
    // Every operation is still listed, with the four the session added
    // since: `undo`, `restore`, `restore` and `undo`.
    let after = ok(&["ops"], &cwd);
    assert_eq!(after.len(), ops.len() + 4, "{after:?}");
    for line in &ops {
        assert!(after.contains(line), "{line} was removed");
    }
    assert!(!root.join(".praps/working-copy").exists());
    assert!(!cwd.join(".praps").exists() || cwd == root);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn praps_first_session_end_to_end() {
    first_session(false);
}

#[test]
fn praps_first_session_from_below() {
    first_session(true);
}

/// The variable that makes this test binary, started again, a process that
/// stands in its current directory: it names a directory for two signal
/// files.
const STAND: &str = "PRAPS_E2E_STAND";

/// Not a test of its own. `undo_from_a_directory_another_process_stands_in`
/// starts this binary again with [`STAND`] set. That process writes
/// `standing`, then keeps its current directory until `release` appears.
/// Run without the variable, as a test, it does nothing.
#[test]
fn stand_in_the_current_directory() {
    let Some(signals) = std::env::var_os(STAND) else {
        return;
    };
    let signals = PathBuf::from(signals);
    fs::write(signals.join("standing"), b"").unwrap();
    let started = Instant::now();
    while !signals.join("release").exists() && started.elapsed() < Duration::from_secs(120) {
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// `undo`, run from a directory that its write empties, while another
/// process has that directory as its current directory. Where the system
/// then refuses to remove the directory (Windows), the write goes on: the
/// working copy's files equal the target's, the directory is left empty,
/// and a note names it. Where the system removes it (Linux), it is gone
/// and there is no note. Either way `praps` exits 0 and the record is
/// dropped (INT-0037 AC-3; INT-0014; Sprint 6 T-012). Run with
/// `--nocapture`, it prints which happened.
#[test]
fn undo_from_a_directory_another_process_stands_in() {
    let dir = claim("stood-in");
    let root = dir.join("project");
    ok(&["init", "project"], &dir);
    write(&root, "keep", b"k");
    assert_eq!(ok(&["status"], &root), ["A keep"]);
    write(&root, "work/file", b"w");
    write(&root, "work/deep/other", b"o");
    assert_eq!(ok(&["status"], &root).len(), 3);
    let work = root.join("work");

    let signals = dir.join("signals");
    fs::create_dir(&signals).unwrap();
    let mut holder = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "stand_in_the_current_directory"])
        .env(STAND, &signals)
        .current_dir(&work)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let started = Instant::now();
    while !signals.join("standing").exists() {
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "the process that stands in the directory did not start"
        );
        std::thread::sleep(Duration::from_millis(20));
    }

    // The undo takes back the recording of `work/…`, so its write removes
    // both files and then the two directories they leave empty.
    let output = praps(&["undo"], &work);
    fs::write(signals.join("release"), b"").unwrap();
    assert!(holder.wait().unwrap().success());
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let err = String::from_utf8_lossy(&output.stderr).into_owned();
    let notes: Vec<&str> = err
        .lines()
        .filter(|line| line.starts_with("note: "))
        .collect();
    assert_eq!(fs::read(root.join("keep")).unwrap(), b"k");
    assert!(!work.join("file").exists(), "{err}");
    assert!(!work.join("deep").exists(), "{err}");
    if work.exists() {
        assert_eq!(fs::read_dir(&work).unwrap().count(), 0, "left empty");
        assert_eq!(notes.len(), 1, "{err}");
        assert!(
            notes[0].starts_with("note: work: the directory could not be removed, and was left"),
            "{err}"
        );
        println!(
            "undo from a directory another process stands in: the directory was left; {}",
            notes[0]
        );
    } else {
        assert!(notes.is_empty(), "{err}");
        println!("undo from a directory another process stands in: the directory was removed");
    }
    assert_eq!(
        err.lines().count(),
        notes.len(),
        "nothing else on stderr: {err}"
    );
    assert!(
        !root.join(".praps/working-copy").exists(),
        "the record was dropped"
    );

    // The write is finished: the next command has nothing to resume, and
    // sees the target's files and nothing else.
    let status = praps(&["status"], &root);
    assert_eq!(status.status.code(), Some(0), "{status:?}");
    assert!(status.stderr.is_empty(), "{status:?}");
    assert_eq!(
        String::from_utf8_lossy(&status.stdout),
        "A keep
"
    );
    assert_eq!(ok(&["verify"], &root), ["clean"]);
    // What was undone is still there to return to.
    ok(&["undo"], &root);
    assert_eq!(fs::read(work.join("file")).unwrap(), b"w");
    assert_eq!(fs::read(work.join("deep/other")).unwrap(), b"o");
    let _ = fs::remove_dir_all(&dir);
}
