//! Exit-code contract of the `book` binary (INT-0002 AC-2, INT-0003 AC-9).

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn book() -> Command {
    Command::new(env!("CARGO_BIN_EXE_book"))
}

fn repo_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// A Book directory under the OS temp directory, removed when dropped, so a
/// failing assertion leaves nothing behind.
struct TempBook(PathBuf);

impl Drop for TempBook {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A Book directory under the OS temp directory with one chapter.
fn temp_book(name: &str, marker: Option<&str>, chapter: &str) -> TempBook {
    let root = std::env::temp_dir().join(format!("book-cli-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("intents")).unwrap();
    if let Some(marker) = marker {
        fs::write(root.join(".sprint-loop-book"), marker).unwrap();
    }
    fs::write(root.join("intents/INT-0001-x.md"), chapter).unwrap();
    TempBook(root)
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
        .filter(|e| {
            // The chapters the checker reads: every `.md` but `README.md`.
            let name = e.file_name().to_string_lossy().into_owned();
            name.ends_with(".md") && name != "README.md"
        })
        .count();
    let counted = stdout.lines().filter(|l| l.ends_with(" criteria")).count();
    assert_eq!(counted, chapters, "one counted line per chapter: {stdout}");

    // A clean Book named explicitly.
    let explicit = book().arg(repo_root().join("docs")).output().unwrap();
    assert_eq!(explicit.status.code(), Some(0));

    // Violations: a declared Book with an untyped criterion at line 4. The
    // violation names the chapter by the path the binary read it at, the Book
    // argument joined with `intents` and the file name.
    let violating = temp_book(
        "violating",
        Some("schema-version: 2\ntyped-acceptance: 1\n"),
        "# INT-0001 — X\n\n## Acceptance criteria\n- Untyped claim.\n",
    );
    let output = book().arg(&violating.0).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    let chapter = violating.0.join("intents").join("INT-0001-x.md");
    let untyped = format!("{}:4: untyped: ", chapter.display());
    let printed = stdout.lines().filter(|line| line.starts_with(&untyped));
    assert_eq!(
        printed.count(),
        1,
        "one line must start with {untyped:?}: {stdout}"
    );

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
        &book().arg(&future.0).output().unwrap(),
        "typed-acceptance: 2",
    );
    assert_usage_error(
        &book().arg(&unmarked.0).output().unwrap(),
        "a Book without a marker",
    );
    for args in [&["--strict"][..], &["-v"], &["docs", "docs"]] {
        let output = book().current_dir(repo_root()).args(args).output().unwrap();
        assert_usage_error(&output, &format!("{args:?}"));
    }
}

/// book-11: a non-Unicode argument is a Book path like any other, so a
/// missing one is refused with exit 2 rather than a panic.
#[test]
fn non_unicode_argument_is_a_book_path() {
    let output = book().arg(non_unicode_name()).output().unwrap();
    assert_usage_error(&output, "a non-Unicode missing directory");
}

/// `n` followed by a lone surrogate, which no Unicode string holds.
#[cfg(windows)]
fn non_unicode_name() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0x6e, 0xD800])
}

/// `n` followed by a byte that is not UTF-8.
#[cfg(unix)]
fn non_unicode_name() -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(b"n\xff".to_vec())
}

/// book-8: a marker that declares typed-acceptance twice is refused, in
/// either order.
#[test]
fn repeated_declaration_exits_2() {
    for (name, first, second) in [("twice-12", 1, 2), ("twice-21", 2, 1)] {
        let marker =
            format!("schema-version: 2\ntyped-acceptance: {first}\ntyped-acceptance: {second}\n");
        let dir = temp_book(
            name,
            Some(&marker),
            "# INT-0001 — X\n\n## Acceptance criteria\n- **AC-1** `necessary` `asserted` — x\n",
        );
        let output = book().arg(&dir.0).output().unwrap();
        assert_usage_error(&output, &marker);
    }
}

/// book-3: an undeclared Book whose items carry `AC-n` labels but no type
/// tags is clean, because the port rule lifts those items.
#[test]
fn undeclared_labels_are_lifted() {
    let dir = temp_book(
        "labels",
        Some("schema-version: 2\nsubstrate-version: 4\n"),
        "# INT-0001 — X\n\n## Acceptance criteria\n\
         - **AC-1:** Users can sign in.\n- **AC-2** Users can sign out.\n",
    );
    let output = book().arg(&dir.0).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(
        stdout.contains("1 chapters, 2 criteria, 0 violations"),
        "{stdout}"
    );
}
