//! Typed-acceptance grammar, the port rule and status (INT-0002 AC-2, AC-3, AC-4).

use std::fs;
use std::path::{Path, PathBuf};

use book::{
    CheckError, Criterion, Kind, Modality, Outcome, Polarity, Rule, Status, check, parse_chapter,
    rule_from_marker, status,
};
use proptest::prelude::*;

/// Lines 1–5; the criteria heading is line 6, so criteria start at line 7.
const CHAPTER_HEAD: &str = "# INT-9999 — Fixture\n\n## Intent\nSomething.\n\n";

fn chapter(criteria: &str) -> String {
    format!("{CHAPTER_HEAD}## Acceptance criteria\n{criteria}\n## Rationale\nBecause.\n")
}

/// The violations of `text` as `(kind, line)`. Each must name the path given
/// to `parse_chapter`, and the CRLF form of `text` must give the same ones.
fn kinds(text: &str, rule: Rule) -> Vec<(Kind, usize)> {
    let path = Path::new("fixture.md");
    let (_, violations) = parse_chapter(path, text, rule);
    for violation in &violations {
        assert_eq!(violation.path, path, "{violation}");
    }
    let (_, crlf) = parse_chapter(path, &text.replace('\n', "\r\n"), rule);
    assert_eq!(violations, crlf, "CRLF must parse like LF");
    let (_, cr) = parse_chapter(path, &text.replace('\n', "\r"), rule);
    assert_eq!(violations, cr, "a lone CR must parse like LF");
    violations.iter().map(|v| (v.kind, v.line)).collect()
}

/// The criteria of `text`, which must have no violations under `rule` and
/// must parse the same from its CRLF form.
fn criteria(text: &str, rule: Rule) -> Vec<Criterion> {
    let path = Path::new("fixture.md");
    let (chapter, violations) = parse_chapter(path, text, rule);
    assert!(violations.is_empty(), "{violations:?}");
    let (crlf, _) = parse_chapter(path, &text.replace('\n', "\r\n"), rule);
    assert_eq!(chapter.criteria, crlf.criteria, "CRLF must parse like LF");
    chapter.criteria
}

/// The `AC-n` numbers of `criteria`, in order.
fn numbers(criteria: &[Criterion]) -> Vec<Option<u32>> {
    criteria.iter().map(|c| c.number).collect()
}

/// The claims of `criteria`, in order.
fn claims(criteria: &[Criterion]) -> Vec<&str> {
    criteria.iter().map(|c| c.claim.as_str()).collect()
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

/// The port rule types an item whose label is followed by a type tag
/// (book-3), so a malformed one is reported there as under the strict rule.
#[test]
fn typed_line_violations_are_reported() {
    let both_rules = |criteria: &str, expected: &[(Kind, usize)]| {
        for rule in [Rule::Strict, Rule::Port] {
            assert_eq!(
                kinds(&chapter(criteria), rule),
                expected,
                "{rule:?}: {criteria}"
            );
        }
    };
    let tagged = [
        ("- **AC-1** `essential` `formal` — x", Kind::Polarity),
        ("- **AC-1** `necessary` `vibes` — x", Kind::Modality),
        ("- **AC-1** `necessary` — x", Kind::Modality),
        // The claim follows ` — `, an em dash between single spaces: not a
        // hyphen, an en dash (U+2013) or a dash without its spaces.
        ("- **AC-1** `necessary` `formal` - x", Kind::Claim),
        ("- **AC-1** `necessary` `formal` \u{2013} x", Kind::Claim),
        ("- **AC-1** `necessary` `formal`—x", Kind::Claim),
        ("- **AC-1** `necessary` `formal` —x", Kind::Claim),
        ("- **AC-1** `necessary` `formal` — ", Kind::Claim),
        ("- **AC-x** `necessary` `formal` — x", Kind::Number),
        ("- **AC-0** `necessary` `formal` — x", Kind::Number),
    ];
    for (line, kind) in tagged {
        both_rules(line, &[(kind, 7)]);
    }
    // Type words match exactly, letter case and spaces included.
    for word in ["Necessary", "necessary ", "Supporting", "supporting "] {
        let line = format!("- **AC-1** `{word}` `formal` — x");
        both_rules(&line, &[(Kind::Polarity, 7)]);
    }
    for word in [
        "Formal",
        "formal ",
        "Empirical",
        "empirical ",
        "Preference",
        "preference ",
        "Judged",
        "judged ",
        "Asserted",
        "asserted ",
    ] {
        let line = format!("- **AC-1** `necessary` `{word}` — x");
        both_rules(&line, &[(Kind::Modality, 7)]);
    }
    // With no type tag after a whole label, only the strict rule types these.
    let untagged = [
        ("- **AC-1** — x", Kind::Polarity),
        ("- **AC-1 `necessary` `formal` — x", Kind::Number),
        ("- **AC-1", Kind::Number),
    ];
    for (line, kind) in untagged {
        assert_eq!(kinds(&chapter(line), Rule::Strict), [(kind, 7)], "{line}");
    }

    // A repeated number is reported where it repeats, next to the first or not.
    let duplicate = "- **AC-1** `necessary` `formal` — x\n- **AC-1** `supporting` `asserted` — y";
    both_rules(duplicate, &[(Kind::Duplicate, 8)]);
    let apart = "- **AC-1** `necessary` `formal` — x\n- **AC-2** `necessary` `formal` — y\n\
                 - **AC-1** `supporting` `asserted` — z";
    both_rules(apart, &[(Kind::Duplicate, 9)]);

    let bad_detail = "- **AC-1** `necessary` `empirical` — x\n  - Signers: owner\n  - no colon";
    both_rules(bad_detail, &[(Kind::Detail, 8), (Kind::Detail, 9)]);
}

/// Each modality allows the detail keys the T-006 Notes give it, and every
/// modality allows `Evidence` and `Note`. Any other key, a key no modality
/// names included, is one `detail` violation at the detail's line.
#[test]
fn detail_keys_follow_each_modality() {
    let own_keys: [(&str, &[&str]); 5] = [
        ("formal", &["Obligation", "Proxy"]),
        ("empirical", &["Threshold", "Sample", "Error"]),
        ("preference", &["Panel", "Threshold", "Error"]),
        ("judged", &["Rubric", "Quorum"]),
        ("asserted", &["Signers"]),
    ];
    let every_modality = ["Evidence", "Note"];
    // The eleven keys the Notes name, then near misses that no modality names.
    // Keys match exactly, letter case included.
    let keys = [
        "Obligation",
        "Proxy",
        "Threshold",
        "Sample",
        "Error",
        "Panel",
        "Rubric",
        "Quorum",
        "Signers",
        "Evidence",
        "Note",
        "Treshold",
        "Signer",
        "Notes",
        "threshold",
    ];
    for (modality, own) in own_keys {
        for key in keys {
            // The criterion is line 7 and its one detail line 8.
            let text = chapter(&format!(
                "- **AC-1** `necessary` `{modality}` — x\n  - {key}: value"
            ));
            let violations = kinds(&text, Rule::Strict);
            if own.contains(&key) || every_modality.contains(&key) {
                assert!(
                    violations.is_empty(),
                    "{key} under {modality}: {violations:?}"
                );
                let parsed = criteria(&text, Rule::Strict);
                let [criterion] = parsed.as_slice() else {
                    panic!("{key} under {modality}: expected one criterion, got {parsed:?}");
                };
                assert_eq!(
                    criterion.details,
                    [(key.to_owned(), "value".to_owned())],
                    "{key} under {modality}"
                );
            } else {
                assert_eq!(violations, [(Kind::Detail, 8)], "{key} under {modality}");
            }
        }
    }
}

/// book-7: `<n>` is ASCII digits with no sign, space or leading zero, from 1
/// up to `u32::MAX`.
#[test]
fn numbers_are_plain_digits() {
    for label in ["+1", "02", "0", "00", " 1", "1 ", "4294967296"] {
        let text = chapter(&format!("- **AC-{label}** `necessary` `formal` — x"));
        assert_eq!(
            kinds(&text, Rule::Strict),
            [(Kind::Number, 7)],
            "AC-{label}"
        );
    }
    let largest = chapter("- **AC-4294967295** `necessary` `formal` — x");
    assert_eq!(numbers(&criteria(&largest, Rule::Strict)), [Some(u32::MAX)]);
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

    // Only a level-2 heading reading exactly `Acceptance criteria` opens the
    // section, so items under any other heading leave the chapter without one.
    for heading in [
        "## Acceptance Criteria",
        "## Acceptance criteria (retired)",
        "# Acceptance criteria",
        "### Acceptance criteria",
    ] {
        let text = format!("{CHAPTER_HEAD}{heading}\n- **AC-1** `necessary` `asserted` — x\n");
        assert_eq!(
            kinds(&text, Rule::Strict),
            [(Kind::Missing, 1)],
            "{heading}"
        );
    }
}

/// A violation of each rule prints the whole path it was read from, its
/// 1-based line and the rule's name, before its message.
#[test]
fn printed_violations_name_file_line_and_rule() {
    let path = Path::new("dir").join("INT-0001-x.md");
    let cases = [
        ("polarity", 7, chapter("- **AC-1** `bogus` `formal` — x")),
        ("modality", 7, chapter("- **AC-1** `necessary` `bogus` — x")),
        ("claim", 7, chapter("- **AC-1** `necessary` `formal` - x")),
        ("number", 7, chapter("- **AC-x** `necessary` `formal` — x")),
        (
            "duplicate",
            8,
            chapter("- **AC-1** `necessary` `formal` — x\n- **AC-1** `supporting` `asserted` — y"),
        ),
        (
            "detail",
            8,
            chapter("- **AC-1** `necessary` `empirical` — x\n  - Signers: owner"),
        ),
        ("untyped", 7, chapter("- Users can export a CSV report.")),
        // A chapter with no criteria section is reported at its first line.
        (
            "missing",
            1,
            format!("{CHAPTER_HEAD}## Rationale\nNo criteria section.\n"),
        ),
    ];
    for (rule, line, text) in cases {
        let (_, violations) = parse_chapter(&path, &text, Rule::Strict);
        let [violation] = violations.as_slice() else {
            panic!("{rule}: expected one violation, got {violations:?}");
        };
        let printed = violation.to_string();
        let prefix = format!("{}:{line}: {rule}: ", path.display());
        assert!(
            printed.starts_with(&prefix),
            "{printed:?} must start with {prefix:?}"
        );
        // CRLF and a lone CR give the same violation.
        assert_eq!(
            kinds(&text, Rule::Strict),
            [(violation.kind, line)],
            "{rule}"
        );
    }
}

#[test]
fn typed_criteria_parse_with_details() {
    // Each type word reads as its own polarity or modality.
    let polarities = [
        ("necessary", Polarity::Necessary),
        ("supporting", Polarity::Supporting),
    ];
    let modalities = [
        ("formal", Modality::Formal),
        ("empirical", Modality::Empirical),
        ("preference", Modality::Preference),
        ("judged", Modality::Judged),
        ("asserted", Modality::Asserted),
    ];
    for (polarity_word, polarity) in polarities {
        for (modality_word, modality) in modalities {
            let text = chapter(&format!(
                "- **AC-1** `{polarity_word}` `{modality_word}` — x"
            ));
            let types: Vec<(Polarity, Modality)> = criteria(&text, Rule::Strict)
                .iter()
                .map(|c| (c.polarity, c.modality))
                .collect();
            assert_eq!(
                types,
                [(polarity, modality)],
                "{polarity_word} {modality_word}"
            );
        }
    }

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

/// book-4: a detail's wrapped and lazy lines belong to its value, not to the
/// criterion's claim.
#[test]
fn wrapped_details_keep_their_lines() {
    let text = chapter(
        &[
            "- **AC-3** `necessary` `formal` — The status function is consistent.",
            "  - Obligation: `unstated` (R5 proved the analogous",
            "    `Certified → ¬ Refuted` for the intent calculus).",
            "  - Proxy: unit",
            "tests.",
        ]
        .join("\n"),
    );
    let parsed = criteria(&text, Rule::Strict);
    let [criterion] = parsed.as_slice() else {
        panic!("expected one criterion, got {parsed:?}");
    };
    assert_eq!(criterion.claim, "The status function is consistent.");
    let obligation =
        "`unstated` (R5 proved the analogous `Certified → ¬ Refuted` for the intent calculus).";
    assert_eq!(
        criterion.details,
        [
            ("Obligation".to_owned(), obligation.to_owned()),
            ("Proxy".to_owned(), "unit tests.".to_owned()),
        ]
    );
}

/// A block quote inside a criterion joins its claim without its `>` markers,
/// and a code block inside it never does. A list in the quote is the quote's,
/// not one of the criterion's details. Links, emphasis and images are claim
/// text like any other.
#[test]
fn quotes_join_a_claim_and_code_does_not() {
    let text = chapter(
        &[
            "- **AC-1** `necessary` `asserted` — The report reads:",
            "  > Exports finish",
            "  > in 2 s.",
            "  > - Note: a quoted list item",
            "  ```text",
            "  - **AC-2** `bogus` `bogus` — sample output",
            "  ```",
        ]
        .join("\n"),
    );
    let parsed = criteria(&text, Rule::Strict);
    assert_eq!(
        claims(&parsed),
        ["The report reads: Exports finish in 2 s."]
    );
    assert!(parsed[0].details.is_empty(), "{parsed:?}");

    let inline = chapter(
        "- **AC-1** `necessary` `asserted` — See [the spec](spec.md), *never*\n  ![ok](ok.png) lost.",
    );
    assert_eq!(
        claims(&criteria(&inline, Rule::Strict)),
        ["See [the spec](spec.md), *never* ![ok](ok.png) lost."]
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

/// book-1, tests-9: a fence closes only on a run of its own character at
/// least as long as its opener, with no info string.
#[test]
fn long_and_tagged_fences_follow_commonmark() {
    // A four-backtick fence in AC-1 holds a three-backtick line; the items
    // after the fence are still checked.
    let in_item = chapter(
        &[
            "- **AC-1** `necessary` `asserted` — Shows an example:",
            "  ````markdown",
            "  ```text",
            "  ````",
            "- **AC-2** `bogus` `bogus` — malformed",
            "- **AC-1** `necessary` `asserted` — duplicate",
        ]
        .join("\n"),
    );
    assert_eq!(
        kinds(&in_item, Rule::Strict),
        [(Kind::Polarity, 11), (Kind::Duplicate, 12)]
    );
    let (parsed, _) = parse_chapter(Path::new("f.md"), &in_item, Rule::Strict);
    assert_eq!(claims(&parsed.criteria), ["Shows an example:"]);

    // A four-backtick example of a whole criteria section is code.
    let example = [
        "## Intent",
        "````markdown",
        "```markdown",
        "## Acceptance criteria",
        "- **AC-9** `bogus` `bogus` — x",
        "```",
        "````",
        "",
        "## Acceptance criteria",
        "- **AC-1** `necessary` `asserted` — real",
    ]
    .join("\n");
    assert_eq!(numbers(&criteria(&example, Rule::Strict)), [Some(1)]);

    // A three-backtick line inside a four-backtick fence in the Intent leaves
    // the fence closed by its own closer, so the real section is read.
    let unbalanced = [
        "## Intent",
        "````markdown",
        "```text",
        "````",
        "",
        "## Acceptance criteria",
        "- **AC-1** `necessary` `asserted` — real",
    ]
    .join("\n");
    assert_eq!(numbers(&criteria(&unbalanced, Rule::Strict)), [Some(1)]);

    // A line with an info string, or a shorter run, does not close a fence.
    let not_closers = chapter(
        &[
            "```",
            "```rust",
            "- **AC-8** `bogus` `bogus` — in a backtick fence",
            "```",
            "~~~~ toml",
            "~~~",
            "- **AC-9** `bogus` `bogus` — in a tilde fence",
            "~~~~",
            "- **AC-1** `necessary` `asserted` — real",
        ]
        .join("\n"),
    );
    assert_eq!(numbers(&criteria(&not_closers, Rule::Strict)), [Some(1)]);
}

/// book-2: a fence left open inside a list item ends with the item, so it
/// hides neither the next criterion nor the next section.
#[test]
fn unclosed_fence_ends_with_its_item() {
    let in_criterion = chapter(
        &[
            "- **AC-1** `necessary` `asserted` — Output is:",
            "  ```text",
            "  ok",
            "- **AC-2** `bogus` `bogus` — never checked",
        ]
        .join("\n"),
    );
    assert_eq!(kinds(&in_criterion, Rule::Strict), [(Kind::Polarity, 10)]);

    let in_intent = [
        "## Intent",
        "- Example:",
        "  ```text",
        "  unclosed",
        "",
        "## Acceptance criteria",
        "- **AC-1** `necessary` `asserted` — real",
    ]
    .join("\n");
    assert_eq!(numbers(&criteria(&in_intent, Rule::Strict)), [Some(1)]);
}

/// book-10: an indented code block is code even when it holds a fence line,
/// so it opens no fence.
#[test]
fn indented_code_opens_no_fence() {
    let text = [
        "## Intent",
        "An example:",
        "",
        "    ```markdown",
        "    - **AC-<n>** `bogus` `bogus` — x",
        "",
        "## Acceptance criteria",
        "- **AC-1** `necessary` `asserted` — real",
    ]
    .join("\n");
    assert_eq!(numbers(&criteria(&text, Rule::Strict)), [Some(1)]);
}

/// book-6: HTML comments, a multi-line one holding a retired criterion
/// included, and thematic breaks are never criteria. A lead-in paragraph is
/// ignored by the port rule and is one `untyped` violation under the strict
/// rule.
#[test]
fn comments_breaks_and_prose_are_not_criteria() {
    let text = chapter(
        &[
            "<!-- criteria are listed below -->",
            "---",
            "<!--",
            "- **AC-1** `necessary` `asserted` — retired",
            "-->",
            "- **AC-2** `necessary` `asserted` — kept",
        ]
        .join("\n"),
    );
    for rule in [Rule::Strict, Rule::Port] {
        assert_eq!(numbers(&criteria(&text, rule)), [Some(2)], "{rule:?}");
    }

    let lead_in = chapter("All of the following hold:\n- **AC-1** `necessary` `asserted` — x");
    assert_eq!(numbers(&criteria(&lead_in, Rule::Port)), [Some(1)]);
    assert_eq!(kinds(&lead_in, Rule::Strict), [(Kind::Untyped, 7)]);
    let (parsed, violations) = parse_chapter(Path::new("f.md"), &lead_in, Rule::Strict);
    assert_eq!(numbers(&parsed.criteria), [Some(1)]);
    assert_eq!(violations[0].message, "criteria must be typed list items");
}

#[test]
fn declared_book_rejects_untyped_items() {
    let text = chapter("- Users can export a CSV report.");
    assert_eq!(kinds(&text, Rule::Strict), [(Kind::Untyped, 7)]);

    // The chapters are the `.md` files but `README.md`: neither the index
    // nor a merge backup beside a chapter is read.
    let book = TempBook::new(
        "declared",
        Some("schema-version: 2\ntyped-acceptance: 1\n"),
        &[
            ("INT-0001-a.md", &text),
            ("INT-0001-a.md.orig", &text),
            ("README.md", "# Intents\n"),
        ],
    );
    let report = check(&book.0).unwrap();
    assert_eq!(report.rule, Rule::Strict);
    assert_eq!(report.chapters.len(), 1);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].kind, Kind::Untyped);
    assert_eq!(
        report.violations[0].path,
        book.0.join("intents").join("INT-0001-a.md")
    );
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

/// book-3: under the port rule only an item with type tags gets the typed
/// checks. `**AC-1:**` and `**AC-2**` labels without tags are lifted whole,
/// and the one violation names the chapter it is in.
#[test]
fn undeclared_book_lifts_untagged_labels() {
    let labels = chapter("- **AC-1:** Users can sign in.\n- **AC-2** Users can sign out.");
    let tagged = chapter("- **AC-3** `bogus` `asserted` — x");
    let book = TempBook::new(
        "labels",
        Some("schema-version: 2\nsubstrate-version: 4\n"),
        &[("INT-0001-a.md", &labels), ("INT-0002-b.md", &tagged)],
    );
    let report = check(&book.0).unwrap();
    assert_eq!(report.rule, Rule::Port);
    let lifted = &report.chapters[0].criteria;
    assert_eq!(
        claims(lifted),
        [
            "**AC-1:** Users can sign in.",
            "**AC-2** Users can sign out."
        ]
    );
    for criterion in lifted {
        assert_eq!(criterion.number, None);
        assert!(!criterion.typed);
        assert_eq!(criterion.polarity, Polarity::Necessary);
        assert_eq!(criterion.modality, Modality::Asserted);
    }

    // A tagged but malformed item is checked even in an undeclared Book.
    let [violation] = report.violations.as_slice() else {
        panic!("expected one violation, got {:?}", report.violations);
    };
    assert_eq!((violation.kind, violation.line), (Kind::Polarity, 7));
    assert_eq!(violation.path, book.0.join("intents").join("INT-0002-b.md"));

    // A code span later in the claim is no type tag.
    let code_later = chapter("- **AC-1** Users run `cargo test`.");
    assert_eq!(
        claims(&criteria(&code_later, Rule::Port)),
        ["**AC-1** Users run `cargo test`."]
    );
}

/// book-5: every CommonMark list marker starts a criterion, indented by up to
/// three spaces, tight or loose; a lazy line continues its item, and a nested
/// list is not part of a lifted claim. Under the strict rule each top-level
/// item is one `untyped` violation.
#[test]
fn every_list_form_holds_criteria() {
    let cases: [(&str, &[&str], &[usize]); 9] = [
        (
            "1. Users can sign in.\n2. Users can sign out.\n3. Sessions expire.",
            &[
                "Users can sign in.",
                "Users can sign out.",
                "Sessions expire.",
            ],
            &[7, 8, 9],
        ),
        (
            "1) Sign in.\n2) Sign out.",
            &["Sign in.", "Sign out."],
            &[7, 8],
        ),
        ("* A\n* B", &["A", "B"], &[7, 8]),
        ("+ A\n+ B", &["A", "B"], &[7, 8]),
        (
            "- Users can export a CSV\nreport from the reports page.\n- Second.",
            &[
                "Users can export a CSV report from the reports page.",
                "Second.",
            ],
            &[7, 9],
        ),
        (
            " - Users can sign in.\n - Users can sign out.",
            &["Users can sign in.", "Users can sign out."],
            &[7, 8],
        ),
        ("   - A\n   - B", &["A", "B"], &[7, 8]),
        ("- A\n\n- B", &["A", "B"], &[7, 9]),
        ("- A\n  - a sub-point", &["A"], &[7]),
    ];
    for (list, expected, lines) in cases {
        let text = chapter(list);
        let lifted = criteria(&text, Rule::Port);
        assert_eq!(claims(&lifted), expected, "{list}");
        let at: Vec<usize> = lifted.iter().map(|c| c.line).collect();
        assert_eq!(at, lines, "{list}");
        let untyped: Vec<(Kind, usize)> = lines.iter().map(|&line| (Kind::Untyped, line)).collect();
        assert_eq!(kinds(&text, Rule::Strict), untyped, "{list}");
    }
}

/// book-9: the criteria heading may take any CommonMark form, a setext
/// heading ends the section like an ATX one, and a heading inside a list item
/// or block quote belongs to that container.
#[test]
fn headings_follow_commonmark() {
    for heading in [
        "## Acceptance criteria ##",
        "##  Acceptance criteria",
        "   ## Acceptance criteria",
        "##\tAcceptance criteria",
        "Acceptance criteria\n---",
    ] {
        let text = format!("{CHAPTER_HEAD}{heading}\n- **AC-1** `necessary` `asserted` — x\n");
        assert_eq!(
            numbers(&criteria(&text, Rule::Strict)),
            [Some(1)],
            "{heading:?}"
        );
    }
    let empty_setext = format!("{CHAPTER_HEAD}Acceptance criteria\n---\n\n## Rationale\n");
    assert_eq!(kinds(&empty_setext, Rule::Strict), [(Kind::Missing, 6)]);

    let setext_end = format!(
        "{CHAPTER_HEAD}## Acceptance criteria\n- **AC-1** `necessary` `asserted` — x\n\n\
         Rationale\n=========\n- Not a criterion.\n"
    );
    assert_eq!(numbers(&criteria(&setext_end, Rule::Strict)), [Some(1)]);

    // `  ## Rationale` is a heading inside AC-1's item: its text joins AC-1,
    // and AC-2 is still in the section.
    let in_item = chapter(
        &[
            "- **AC-1** `necessary` `asserted` — x",
            "  ## Rationale",
            "- **AC-2** `necessary` `asserted` — y",
        ]
        .join("\n"),
    );
    assert_eq!(
        claims(&criteria(&in_item, Rule::Strict)),
        ["x Rationale", "y"]
    );

    // `> ## Rationale` is a block quote: prose in the section, not its end.
    let in_quote = chapter("> ## Rationale\n- **AC-1** `necessary` `asserted` — x");
    assert_eq!(numbers(&criteria(&in_quote, Rule::Port)), [Some(1)]);
    assert_eq!(kinds(&in_quote, Rule::Strict), [(Kind::Untyped, 7)]);
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

    // CommonMark also ends a line at a lone CR: same lines, no CR in claims.
    let (c, vc) = parse_chapter(Path::new("f.md"), &lf.replace('\n', "\r"), Rule::Strict);
    assert!(vc.is_empty(), "{vc:?}");
    assert_eq!(a.criteria, c.criteria);
}

#[test]
fn tab_indented_detail_is_located_at_its_line() {
    // The criterion is line 7 and the detail line 8 (line 9 after a blank).
    let tabbed = chapter("- **AC-1** `necessary` `asserted` — x\n\t- no colon here\n");
    assert_eq!(kinds(&tabbed, Rule::Strict), [(Kind::Detail, 8)]);
    let spaced = chapter("- **AC-1** `necessary` `asserted` — x\n\n\t- no colon here\n");
    assert_eq!(kinds(&spaced, Rule::Strict), [(Kind::Detail, 9)]);
}

#[test]
fn prose_alone_is_untyped_not_missing() {
    // A paragraph is content, so the strict rule reports it, not `missing`.
    assert_eq!(
        kinds(&chapter("Just prose.\n"), Rule::Strict),
        [(Kind::Untyped, 7)]
    );
}

#[test]
fn level_three_heading_stays_in_the_section() {
    let text = chapter(
        "- **AC-1** `necessary` `asserted` — a\n### Sub\n- **AC-2** `necessary` `asserted` — b\n",
    );
    let (parsed, violations) = parse_chapter(Path::new("f.md"), &text, Rule::Strict);
    assert!(violations.is_empty(), "{violations:?}");
    let numbers: Vec<Option<u32>> = parsed.criteria.iter().map(|c| c.number).collect();
    assert_eq!(numbers, [Some(1), Some(2)]);
}

#[test]
fn deep_nesting_cannot_exhaust_the_stack() {
    // Before the reader bounded its recursion, about 850 nested list levels
    // overflowed the main thread's stack on Windows. A small, fixed stack makes
    // an unbounded reader fail here on every machine.
    const LEVELS: usize = 20_000;
    let parse = std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let lists = chapter(&format!(
                "- **AC-1** `necessary` `asserted` — x\n{}deep\n",
                "- ".repeat(LEVELS)
            ));
            let quotes = chapter(&format!(
                "- **AC-1** `necessary` `asserted` — x\n  {}deep\n",
                "> ".repeat(LEVELS)
            ));
            [lists, quotes].map(|text| parse_chapter(Path::new("f.md"), &text, Rule::Strict))
        })
        .unwrap();
    let [(lists, list_violations), (quotes, quote_violations)] = parse.join().unwrap();
    assert_eq!(lists.criteria.len(), 1);
    assert_eq!(list_violations.len(), 1, "the empty second item is untyped");
    assert!(quote_violations.is_empty(), "{quote_violations:?}");
    assert_eq!(quotes.criteria[0].claim, "x");
}

#[test]
fn quotes_are_read_eight_deep() {
    let claim = |depth: usize| {
        let text = chapter(&format!(
            "- **AC-1** `necessary` `asserted` — x\n  {}deep\n",
            "> ".repeat(depth)
        ));
        let (parsed, _) = parse_chapter(Path::new("f.md"), &text, Rule::Strict);
        parsed.criteria[0].claim.clone()
    };
    assert_eq!(claim(1), "x deep");
    assert_eq!(claim(8), "x deep");
    assert_eq!(claim(9), "x", "quotes deeper than eight are skipped");
}

#[test]
fn line_endings_parse_alike() {
    // Fences, HTML blocks, indented code, quotes and details under LF, CRLF
    // and a lone CR: pulldown-cmark alone reads some of these differently.
    let chapters = [
        "- **AC-1** `necessary` `asserted` — Exports finish.\n\n<!-- reviewed -->\n- Users can export a CSV report.\n",
        "- **AC-1** `necessary` `asserted` — Shows:\n  ```text\n  ok\n  ```\n- **AC-2** `bogus` `asserted` — b\n",
        "- **AC-1** `necessary` `asserted` — a\n\n      code\n- **AC-2** `necessary` `formal` — b\n  - Proxy: tests\n  > quoted\n",
    ];
    for criteria in chapters {
        let lf = chapter(criteria);
        for rule in [Rule::Strict, Rule::Port] {
            let expected = parse_chapter(Path::new("f.md"), &lf, rule);
            for ending in ["\r\n", "\r"] {
                let text = lf.replace('\n', ending);
                assert_eq!(
                    parse_chapter(Path::new("f.md"), &text, rule),
                    expected,
                    "{ending:?} {rule:?}: {criteria:?}"
                );
            }
        }
    }
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
    // Only exactly `1` declares the contract: an empty or blank value is not
    // a missing one, and `10` or `1.1` is not version 1.
    for value in ["", "   ", "10", "1.1", "01", "2"] {
        let marker = format!("schema-version: 2\ntyped-acceptance: {value}\n");
        assert!(
            matches!(
                rule_from_marker(&marker),
                Err(CheckError::UnsupportedDeclaration { value: found }) if found == value.trim()
            ),
            "{value:?}"
        );
    }
    assert!(matches!(
        rule_from_marker("schema-version: 2\ntyped-acceptance: 1\n"),
        Ok(Rule::Strict)
    ));
}

/// book-8: a marker that declares typed-acceptance twice names no single
/// contract, so it is refused whatever the order or the values.
#[test]
fn repeated_declaration_is_an_error() {
    for marker in [
        "schema-version: 2\ntyped-acceptance: 1\ntyped-acceptance: 2\n",
        "schema-version: 2\ntyped-acceptance: 2\ntyped-acceptance: 1\n",
        "typed-acceptance: 1\ntyped-acceptance: 1\n",
    ] {
        assert!(
            matches!(
                rule_from_marker(marker),
                Err(CheckError::DuplicateDeclaration)
            ),
            "{marker}"
        );
    }
    assert!(matches!(
        rule_from_marker("schema-version: 2\ntyped-acceptance: 1\n"),
        Ok(Rule::Strict)
    ));
    assert!(matches!(
        rule_from_marker("typed-acceptance: 01\n"),
        Err(CheckError::UnsupportedDeclaration { .. })
    ));

    let book = TempBook::new(
        "declared-twice",
        Some("schema-version: 2\ntyped-acceptance: 1\ntyped-acceptance: 2\n"),
        &[],
    );
    assert!(matches!(
        check(&book.0),
        Err(CheckError::DuplicateDeclaration)
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

/// Cases for the status property.
const CASES: u32 = 10_000;

#[test]
fn case_count_is_not_overridden() {
    // proptest lets PROPTEST_CASES replace a configured count, which would let
    // the property pass on fewer cases than the record states.
    let config = proptest::test_runner::contextualize_config(ProptestConfig::with_cases(CASES));
    assert_eq!(config.cases, CASES, "unset PROPTEST_CASES to run the suite");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// INT-0002 AC-3's proxy. `status` returns a single `Status` variant, so
    /// an intent both Certified and Refuted cannot be represented at all; the
    /// two iff assertions pin which variant it returns.
    #[test]
    fn status_is_consistent(criteria in prop::collection::vec((any_polarity(), any_outcome()), 1..8)) {
        let result = status(&criteria);
        let all_hold = criteria.iter().all(|&(_, o)| o == Outcome::Holds);
        let necessary_fails = criteria
            .iter()
            .any(|&(p, o)| p == Polarity::Necessary && o == Outcome::Fails);
        prop_assert_eq!(result == Status::Certified, all_hold);
        prop_assert_eq!(result == Status::Refuted, necessary_fails);
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
