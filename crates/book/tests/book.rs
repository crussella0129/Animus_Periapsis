//! Typed-acceptance grammar, the port rule and status (INT-0002 AC-2, AC-3, AC-4).

use std::fs;
use std::path::{Path, PathBuf};

use book::{
    CheckError, Kind, Modality, Outcome, Polarity, Rule, Status, check, parse_chapter, status,
};
use proptest::prelude::*;

/// Lines 1–5; the criteria heading is line 6, so criteria start at line 7.
const CHAPTER_HEAD: &str = "# INT-9999 — Fixture\n\n## Intent\nSomething.\n\n";

fn chapter(criteria: &str) -> String {
    format!("{CHAPTER_HEAD}## Acceptance criteria\n{criteria}\n## Rationale\nBecause.\n")
}

fn kinds(text: &str, rule: Rule) -> Vec<(Kind, usize)> {
    let (_, violations) = parse_chapter(Path::new("fixture.md"), text, rule);
    violations.iter().map(|v| (v.kind, v.line)).collect()
}

/// A scratch Book on disk, removed when dropped.
struct TempBook(PathBuf);

impl TempBook {
    fn new(name: &str, marker: Option<&str>, chapters: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!("book-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("intents")).unwrap();
        if let Some(marker) = marker {
            fs::write(root.join(".sprint-loop-book"), marker).unwrap();
        }
        for (file, text) in chapters {
            fs::write(root.join("intents").join(file), text).unwrap();
        }
        TempBook(root)
    }
}

impl Drop for TempBook {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn typed_line_violations_are_reported() {
    let cases = [
        ("- **AC-1** `essential` `formal` — x", Kind::Polarity),
        ("- **AC-1** — x", Kind::Polarity),
        ("- **AC-1** `necessary` `vibes` — x", Kind::Modality),
        ("- **AC-1** `necessary` — x", Kind::Modality),
        ("- **AC-1** `necessary` `formal` - x", Kind::Claim),
        ("- **AC-1** `necessary` `formal` — ", Kind::Claim),
        ("- **AC-x** `necessary` `formal` — x", Kind::Number),
        ("- **AC-1 `necessary` `formal` — x", Kind::Number),
        ("- **AC-0** `necessary` `formal` — x", Kind::Number),
    ];
    for (line, kind) in cases {
        assert_eq!(kinds(&chapter(line), Rule::Strict), [(kind, 7)], "{line}");
    }

    let duplicate = "- **AC-1** `necessary` `formal` — x\n- **AC-1** `supporting` `asserted` — y";
    assert_eq!(
        kinds(&chapter(duplicate), Rule::Strict),
        [(Kind::Duplicate, 8)]
    );

    let bad_detail = "- **AC-1** `necessary` `empirical` — x\n  - Signers: owner\n  - no colon";
    assert_eq!(
        kinds(&chapter(bad_detail), Rule::Strict),
        [(Kind::Detail, 8), (Kind::Detail, 9)]
    );
}

#[test]
fn missing_criteria_are_reported() {
    let missing = format!("{CHAPTER_HEAD}## Rationale\nNo criteria section.\n");
    assert_eq!(kinds(&missing, Rule::Strict), [(Kind::Missing, 1)]);
    let empty = chapter("");
    assert_eq!(kinds(&empty, Rule::Strict), [(Kind::Missing, 6)]);
    // The port rule reads legacy Books as they are: no criteria is no error.
    assert!(kinds(&missing, Rule::Port).is_empty());
    assert!(kinds(&empty, Rule::Port).is_empty());
}

#[test]
fn typed_criteria_parse_with_details() {
    let text = chapter(
        "- **AC-2** `supporting` `preference` — Judges prefer the new layout   \n  over the old one.\n  - Panel: 12 judges\n  - Error: δ = 1/20",
    );
    let (chapter, violations) = parse_chapter(Path::new("f.md"), &text, Rule::Strict);
    assert!(violations.is_empty(), "{violations:?}");
    let [criterion] = chapter.criteria.as_slice() else {
        panic!("expected one criterion, got {:?}", chapter.criteria);
    };
    assert_eq!(criterion.number, Some(2));
    assert_eq!(criterion.polarity, Polarity::Supporting);
    assert_eq!(criterion.modality, Modality::Preference);
    assert_eq!(
        criterion.claim,
        "Judges prefer the new layout over the old one."
    );
    assert_eq!(
        criterion.details,
        [
            ("Panel".to_owned(), "12 judges".to_owned()),
            ("Error".to_owned(), "δ = 1/20".to_owned()),
        ]
    );
}

#[test]
fn fenced_code_is_ignored() {
    let text = "# INT-9999 — Fixture\n\n## Intent\n\
                ```markdown\n## Acceptance criteria\n- **AC-9** `bogus` `bogus` — a fenced heading\n```\n\n\
                ## Acceptance criteria\n\
                ```text\n- **AC-1** `bogus` `bogus` — in a backtick fence\n```\n\
                ~~~\n- **AC-2** `bogus` `bogus` — in a tilde fence\n~~~\n\
                - **AC-3** `necessary` `asserted` — real\n\n## Rationale\nBecause.\n";
    let (chapter, violations) = parse_chapter(Path::new("f.md"), text, Rule::Strict);
    assert!(violations.is_empty(), "{violations:?}");
    let numbers: Vec<Option<u32>> = chapter.criteria.iter().map(|c| c.number).collect();
    assert_eq!(numbers, [Some(3)]);
}

#[test]
fn declared_book_rejects_untyped_items() {
    let text = chapter("- Users can export a CSV report.");
    assert_eq!(kinds(&text, Rule::Strict), [(Kind::Untyped, 7)]);

    let book = TempBook::new(
        "declared",
        Some("schema-version: 2\ntyped-acceptance: 1\n"),
        &[("INT-0001-a.md", &text)],
    );
    let report = check(&book.0).unwrap();
    assert_eq!(report.rule, Rule::Strict);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].kind, Kind::Untyped);
}

#[test]
fn undeclared_book_lifts_untyped_items() {
    let text = chapter(
        "- Users can export a CSV report\n  from the reports page.\n- Exports finish in 2 s.",
    );
    let book = TempBook::new(
        "undeclared",
        Some("schema-version: 2\nsubstrate-version: 4\n"),
        &[("INT-0001-a.md", &text)],
    );
    let report = check(&book.0).unwrap();
    assert_eq!(report.rule, Rule::Port);
    assert!(report.violations.is_empty(), "{:?}", report.violations);
    let criteria = &report.chapters[0].criteria;
    assert_eq!(criteria.len(), 2);
    for criterion in criteria {
        assert!(!criterion.typed);
        assert_eq!(criterion.polarity, Polarity::Necessary);
        assert_eq!(criterion.modality, Modality::Asserted);
    }
    assert_eq!(
        criteria[0].claim,
        "Users can export a CSV report from the reports page."
    );
}

#[test]
fn crlf_parses_like_lf() {
    let lf = chapter(
        "- **AC-1** `necessary` `formal` — x\n  - Obligation: `unstated`\n- **AC-2** `supporting` `judged` — y\n  wrapped\n  - Quorum: 2 of 3",
    );
    let crlf = lf.replace('\n', "\r\n");
    assert!(
        crlf.contains("\r\n"),
        "the CRLF variant must really contain CRLF"
    );
    let (a, va) = parse_chapter(Path::new("f.md"), &lf, Rule::Strict);
    let (b, vb) = parse_chapter(Path::new("f.md"), &crlf, Rule::Strict);
    assert!(va.is_empty() && vb.is_empty(), "{va:?} {vb:?}");
    assert_eq!(a.criteria.len(), 2);
    assert_eq!(a.criteria[1].claim, "y wrapped");
    assert_eq!(a.criteria, b.criteria);
}

#[test]
fn unsupported_declaration_is_an_error() {
    let book = TempBook::new(
        "future",
        Some("schema-version: 2\ntyped-acceptance: 2\n"),
        &[],
    );
    assert!(matches!(
        check(&book.0),
        Err(CheckError::UnsupportedDeclaration { .. })
    ));
}

#[test]
fn missing_marker_is_an_error() {
    let book = TempBook::new("no-marker", None, &[]);
    assert!(matches!(check(&book.0), Err(CheckError::NotABook { .. })));
}

#[test]
fn status_truth_table() {
    use Outcome::{Fails, Holds, Unknown};
    use Polarity::{Necessary, Supporting};
    assert_eq!(
        status(&[(Necessary, Holds), (Supporting, Holds)]),
        Status::Certified
    );
    assert_eq!(
        status(&[(Necessary, Fails), (Supporting, Holds)]),
        Status::Refuted
    );
    assert_eq!(
        status(&[(Necessary, Holds), (Supporting, Fails)]),
        Status::Undetermined
    );
    assert_eq!(status(&[(Necessary, Unknown)]), Status::Undetermined);
    assert_eq!(status(&[]), Status::Undetermined);
}

fn any_polarity() -> impl Strategy<Value = Polarity> {
    prop_oneof![Just(Polarity::Necessary), Just(Polarity::Supporting)]
}

fn any_outcome() -> impl Strategy<Value = Outcome> {
    prop_oneof![
        Just(Outcome::Holds),
        Just(Outcome::Fails),
        Just(Outcome::Unknown)
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn status_is_consistent(criteria in prop::collection::vec((any_polarity(), any_outcome()), 1..8)) {
        let result = status(&criteria);
        let all_hold = criteria.iter().all(|&(_, o)| o == Outcome::Holds);
        let necessary_fails = criteria
            .iter()
            .any(|&(p, o)| p == Polarity::Necessary && o == Outcome::Fails);
        prop_assert_eq!(result == Status::Certified, all_hold);
        prop_assert_eq!(result == Status::Refuted, necessary_fails);
        prop_assert!(!(result == Status::Certified && result == Status::Refuted));
    }
}

#[test]
fn book_intents_pass_strict() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let report = check(&docs).unwrap();
    assert_eq!(
        report.rule,
        Rule::Strict,
        "docs/.sprint-loop-book declares typed-acceptance: 1"
    );
    let violations: Vec<String> = report.violations.iter().map(ToString::to_string).collect();
    assert!(violations.is_empty(), "{}", violations.join("\n"));
    let chapter_files = fs::read_dir(docs.join("intents"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with("INT-") && name.ends_with(".md")
        })
        .count();
    assert_eq!(report.chapters.len(), chapter_files);
    for chapter in &report.chapters {
        assert!(!chapter.criteria.is_empty(), "{}", chapter.path.display());
    }
}

#[test]
fn sprint_loops_fixtures_pass_port_rule() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sprint-loops");
    let report = check(&fixture).unwrap();
    assert_eq!(report.rule, Rule::Port);
    assert!(report.violations.is_empty(), "{:?}", report.violations);
    let counts: Vec<(String, usize)> = report
        .chapters
        .iter()
        .map(|c| {
            let name = c.path.file_name().unwrap().to_string_lossy().into_owned();
            (name, c.criteria.len())
        })
        .collect();
    assert_eq!(
        counts,
        [
            ("INT-0001-project-book.md".to_owned(), 5),
            ("INT-0013-verification-integrity.md".to_owned(), 9),
        ]
    );
    for chapter in &report.chapters {
        for criterion in &chapter.criteria {
            assert_eq!(criterion.polarity, Polarity::Necessary);
            assert_eq!(criterion.modality, Modality::Asserted);
        }
    }
    assert_eq!(
        report.chapters[0].criteria[0].claim,
        "`docs/` validates under `check-book.sh`: exactly one `schema-version: 2` marker, \
         unique `INT-NNNN` ids, legal states, and state-appropriate evidence."
    );
}
