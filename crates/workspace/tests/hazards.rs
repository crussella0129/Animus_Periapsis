//! The hazard check's corpora (INT-0004 AC-3, AC-4; Sprint 3 T-005).
//!
//! Each case is a pure call of [`hazard::check`] on hand-built trees, so the
//! corpora run the same on every host. The commands' refusal before any write
//! is tested with `undo` and `restore` (`hazards_are_refused_before_any_write`).

use std::collections::BTreeMap;

use codec::{Mode, RepoPath, blob, object_id};
use hash::HashAlg;
use sim::NameRules;
use workspace::hazard::{self, Problem, fold_eq, windows_problem};
use workspace::tree::{File, Tree};

const CASE: NameRules = NameRules {
    case_insensitive: true,
    windows_names: false,
};
const WINDOWS: NameRules = NameRules {
    case_insensitive: false,
    windows_names: true,
};
const NEITHER: NameRules = NameRules {
    case_insensitive: false,
    windows_names: false,
};

fn file(content: &str) -> File {
    File {
        blob: object_id(HashAlg::DEFAULT, &blob(content.as_bytes())),
        mode: Mode::Regular,
    }
}

/// A tree of `(path, content)` pairs.
fn tree(files: &[(&str, &str)]) -> Tree {
    files
        .iter()
        .map(|(path, content)| (RepoPath::new(*path).unwrap(), file(content)))
        .collect()
}

/// Directory listings: `(directory, names)`.
fn listing(dirs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    dirs.iter()
        .map(|(dir, names)| {
            (
                (*dir).to_owned(),
                names.iter().map(|name| (*name).to_owned()).collect(),
            )
        })
        .collect()
}

/// The `(path, component)` pairs found, sorted.
fn found(
    from: &Tree,
    target: &Tree,
    existing: &BTreeMap<String, Vec<String>>,
    rules: NameRules,
) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = hazard::check(from, target, existing, rules)
        .into_iter()
        .map(|h| (h.path.as_str().to_owned(), h.component))
        .collect();
    found.dedup();
    found
}

/// One case of the case-fold corpus.
struct Case {
    name: &'static str,
    from: &'static [(&'static str, &'static str)],
    target: &'static [(&'static str, &'static str)],
    existing: &'static [(&'static str, &'static [&'static str])],
    /// The `(path, component)` pairs the check must report, sorted.
    expect: &'static [(&'static str, &'static str)],
}

const CASE_CORPUS: &[Case] = &[
    Case {
        name: "two new files differing in case",
        from: &[],
        target: &[("README", "a"), ("readme", "b")],
        existing: &[],
        expect: &[("README", "README"), ("readme", "readme")],
    },
    Case {
        name: "file a against directory A/",
        from: &[],
        target: &[("a", "1"), ("A/x", "2")],
        existing: &[],
        expect: &[("A/x", "A"), ("a", "a")],
    },
    Case {
        name: "directories differing in case at the top",
        from: &[],
        target: &[("dir/Foo", "1"), ("DIR/bar", "2")],
        existing: &[],
        expect: &[("DIR/bar", "DIR"), ("dir/Foo", "dir")],
    },
    Case {
        name: "a collision two levels down only",
        from: &[],
        target: &[("p/q/Name", "1"), ("p/q/name", "2")],
        existing: &[],
        expect: &[("p/q/Name", "Name"), ("p/q/name", "name")],
    },
    Case {
        name: "final and medial sigma, equal only in uppercase",
        from: &[],
        target: &[("x/σ", "1"), ("x/ς", "2")],
        existing: &[],
        expect: &[("x/σ", "σ"), ("x/ς", "ς")],
    },
    Case {
        name: "s and long s, equal only in uppercase",
        from: &[],
        target: &[("s", "1"), ("ſ", "2")],
        existing: &[],
        expect: &[("s", "s"), ("ſ", "ſ")],
    },
    Case {
        name: "the Kelvin sign and k, equal in lowercase",
        from: &[],
        target: &[("\u{212a}", "1"), ("k", "2")],
        existing: &[],
        expect: &[("k", "k"), ("\u{212a}", "\u{212a}")],
    },
    Case {
        name: "sharp s and SS, folded by the conservative test",
        from: &[],
        target: &[("ß", "1"), ("SS", "2")],
        existing: &[],
        expect: &[("SS", "SS"), ("ß", "ß")],
    },
    Case {
        name: "a new directory against an existing one",
        from: &[],
        target: &[("docs/x", "1")],
        existing: &[("", &["Docs"])],
        expect: &[("docs/x", "docs")],
    },
    Case {
        name: "a new file against an untracked one",
        from: &[("a", "1")],
        target: &[("a", "1"), ("Notes.txt", "2")],
        existing: &[("", &["a", "notes.TXT"])],
        expect: &[("Notes.txt", "Notes.txt")],
    },
    Case {
        name: "a case-only rename (known limit, T-016)",
        from: &[("README", "1")],
        target: &[("readme", "1")],
        existing: &[("", &["README"])],
        expect: &[("readme", "readme")],
    },
    Case {
        name: "one of two colliding names written",
        from: &[("README", "1")],
        target: &[("README", "1"), ("readme", "2")],
        existing: &[("", &["README"])],
        expect: &[("readme", "readme")],
    },
    Case {
        name: "a rewrite under its own spelling",
        from: &[("a", "1"), ("D/b", "1")],
        target: &[("a", "2"), ("D/b", "2")],
        existing: &[("", &["a", "D"]), ("D", &["b"])],
        expect: &[],
    },
    Case {
        name: "names that differ in more than case",
        from: &[],
        target: &[("a", "1"), ("b", "2"), ("ab/a", "3"), ("aB.txt", "4")],
        existing: &[("", &["c"])],
        expect: &[],
    },
    Case {
        name: "colliding names that are both unchanged",
        from: &[("README", "1"), ("readme", "2")],
        target: &[("README", "1"), ("readme", "2"), ("new", "3")],
        existing: &[("", &["README", "new0", "readme"])],
        expect: &[],
    },
];

#[test]
fn case_fold_corpus() {
    assert!(CASE_CORPUS.len() >= 12);
    for case in CASE_CORPUS {
        let (from, target) = (tree(case.from), tree(case.target));
        let existing = listing(case.existing);
        let mut expect: Vec<(String, String)> = case
            .expect
            .iter()
            .map(|(p, c)| ((*p).to_owned(), (*c).to_owned()))
            .collect();
        expect.sort();
        assert_eq!(
            found(&from, &target, &existing, CASE),
            expect,
            "{}",
            case.name
        );
        // Where names are case-sensitive, none of these is a hazard.
        assert_eq!(
            found(&from, &target, &existing, NEITHER),
            Vec::<(String, String)>::new(),
            "{}",
            case.name
        );
    }
}

#[test]
fn case_only_rename_is_refused() {
    let hazards = hazard::check(
        &tree(&[("README", "1")]),
        &tree(&[("readme", "1")]),
        &listing(&[("", &["README"])]),
        CASE,
    );
    assert_eq!(hazards.len(), 1, "{hazards:?}");
    assert_eq!(
        hazards[0].problem,
        Problem::CaseExisting {
            existing: "README".to_owned()
        }
    );
}

#[test]
fn fold_eq_is_conservative() {
    for (a, b) in [
        ("a", "A"),
        ("σ", "ς"),
        ("s", "ſ"),
        ("k", "\u{212a}"),
        ("ß", "SS"),
    ] {
        assert!(fold_eq(a, b), "{a} {b}");
    }
    for (a, b) in [("a", "b"), ("a", "aa"), ("é", "e")] {
        assert!(!fold_eq(a, b), "{a} {b}");
    }
}

/// Names Windows cannot hold: every reserved base, with and without an
/// extension and in mixed case, and every refused character.
const WINDOWS_HAZARDS: &[&str] = &[
    "CON",
    "con",
    "Con.txt",
    "PRN",
    "prn.log",
    "AUX",
    "aux.tar.gz",
    "NUL",
    "nul .txt",
    "CONIN$",
    "conout$",
    "CONOUT$.x",
    "COM0",
    "COM1",
    "com2.c",
    "COM3",
    "COM4",
    "COM5",
    "COM6",
    "COM7",
    "COM8",
    "COM9",
    "COM¹",
    "com².txt",
    "COM³",
    "LPT0",
    "LPT1",
    "lpt2",
    "LPT3.doc",
    "LPT4",
    "LPT5",
    "LPT6",
    "LPT7",
    "LPT8",
    "LPT9",
    "LPT¹",
    "lpt²",
    "LPT³.log",
    "NUL  .txt",
    "a<b",
    "a>b",
    "a:b",
    "a\"b",
    "a|b",
    "a?b",
    "a*b",
    "tab\there",
    "bell\u{7}",
    "nl\u{1f}",
    "dot.",
    "space ",
    "dots..",
];

/// Names Windows can hold, though some look like device names.
const WINDOWS_SAFE: &[&str] = &[
    "CONSOLE",
    "COM10",
    "LPT",
    "COM",
    "COMx",
    "con_",
    "auxiliary",
    "nul-",
    ".con",
    "COM⁴",
    "LPT¹0",
    "a.b",
    "a b",
    "naïve",
    "\u{7f}del",
    "CONIN",
    "CON$",
];

#[test]
fn windows_name_corpus() {
    assert!(WINDOWS_HAZARDS.len() >= 34);
    for name in WINDOWS_HAZARDS {
        assert!(
            windows_problem(name).is_some(),
            "{name:?} should be a hazard"
        );
        let target = tree(&[(*name, "x")]);
        let empty = Tree::new();
        let none = BTreeMap::new();
        assert_eq!(
            found(&empty, &target, &none, WINDOWS),
            [((*name).to_owned(), (*name).to_owned())],
            "{name:?}"
        );
        // As a directory component too, and not where the rules are off.
        let deep = tree(&[(&format!("d/{name}/f") as &str, "x")]);
        assert_eq!(
            found(&empty, &deep, &none, WINDOWS),
            [(format!("d/{name}/f"), (*name).to_owned())],
            "{name:?}"
        );
        assert!(
            found(&empty, &target, &none, NEITHER).is_empty(),
            "{name:?}"
        );
    }
    for name in WINDOWS_SAFE {
        assert_eq!(windows_problem(name), None, "{name:?}");
    }
}

#[test]
fn unchanged_hazardous_names_are_not_refused() {
    let from = tree(&[
        ("CON", "1"),
        ("a:b", "1"),
        ("aux/x", "1"),
        ("README", "1"),
        ("readme", "1"),
    ]);
    let mut target = from.clone();
    target.insert(RepoPath::new("ok").unwrap(), file("2"));
    let existing = listing(&[
        ("", &["CON", "README", "a:b", "aux", "readme"]),
        ("aux", &["x"]),
    ]);
    let both = NameRules {
        case_insensitive: true,
        windows_names: true,
    };
    assert!(hazard::check(&from, &target, &existing, both).is_empty());
    // Rewriting one of them is checked.
    target.insert(RepoPath::new("aux/x").unwrap(), file("2"));
    assert_eq!(
        found(&from, &target, &existing, both),
        [("aux/x".to_owned(), "aux".to_owned())]
    );
}
