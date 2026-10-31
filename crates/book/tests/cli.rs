//! Exit-code contract of the `book` binary (INT-0002 AC-2, INT-0003 AC-9).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn book() -> Command {
    Command::new(env!("CARGO_BIN_EXE_book"))
}

fn repo_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// A Book directory under the OS temp directory with one chapter.
fn temp_book(name: &str, marker: Option<&str>, chapter: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("book-cli-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("intents")).unwrap();
    if let Some(marker) = marker {
        fs::write(root.join(".sprint-loop-book"), marker).unwrap();
    }
    fs::write(root.join("intents/INT-0001-x.md"), chapter).unwrap();
    root
}

fn assert_usage_error(output: &Output, what: &str) {
    assert_eq!(output.status.code(), Some(2), "{what}");
    assert!(
        !String::from_utf8_lossy(&output.stderr).trim().is_empty(),
        "{what} must explain itself on stderr"
    );
    assert!(output.stdout.is_empty(), "{what} must not print a report");
}

#[test]
fn cli_exit_codes() {
    // No arguments: checks ./docs. Run in the repository root through
    // `Command::current_dir`, so parallel tests are unaffected.
    let default = book().current_dir(repo_root()).output().unwrap();
    let stdout = String::from_utf8_lossy(&default.stdout);
    assert_eq!(default.status.code(), Some(0), "{stdout}");
    let chapters = fs::read_dir(repo_root().join("docs/intents"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with("INT-"))
        .count();
    let counted = stdout.lines().filter(|l| l.ends_with(" criteria")).count();
    assert_eq!(counted, chapters, "one counted line per chapter: {stdout}");

    // A clean Book named explicitly.
    let explicit = book().arg(repo_root().join("docs")).output().unwrap();
    assert_eq!(explicit.status.code(), Some(0));

    // Violations: a declared Book with an untyped criterion.
    let violating = temp_book(
        "violating",
        Some("schema-version: 2\ntyped-acceptance: 1\n"),
        "# INT-0001 — X\n\n## Acceptance criteria\n- Untyped claim.\n",
    );
    let output = book().arg(&violating).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains(": untyped: "));

    // Usage and refusal errors: exit 2, a message on stderr, nothing on stdout.
    let future = temp_book(
        "future",
        Some("schema-version: 2\ntyped-acceptance: 2\n"),
        "# INT-0001 — X\n",
    );
    let unmarked = temp_book("unmarked", None, "# INT-0001 — X\n");
    let missing = repo_root().join("no-such-book");
    assert_usage_error(
        &book().arg(&missing).output().unwrap(),
        "a missing directory",
    );
    assert_usage_error(
        &book().arg(&future).output().unwrap(),
        "typed-acceptance: 2",
    );
    assert_usage_error(
        &book().arg(&unmarked).output().unwrap(),
        "a Book without a marker",
    );
    for args in [&["--strict"][..], &["-v"], &["docs", "docs"]] {
        let output = book().current_dir(repo_root()).args(args).output().unwrap();
        assert_usage_error(&output, &format!("{args:?}"));
    }

    for dir in [violating, future, unmarked] {
        fs::remove_dir_all(dir).unwrap();
    }
}
