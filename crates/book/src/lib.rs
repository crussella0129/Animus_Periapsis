#![forbid(unsafe_code)]
//! Reads the acceptance criteria of a Sprint Loops Book and checks them
//! against the typed-acceptance grammar (INT-0002).
//!
//! A Book that declares `typed-acceptance: 1` in `docs/.sprint-loop-book` is
//! checked strictly: every criterion must be typed. A Book without the
//! declaration is read with the port rule, which lifts each untyped criterion
//! to `necessary` `asserted`. Any other declared value, or a directory with no
//! marker, is refused.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MARKER_FILE: &str = ".sprint-loop-book";
const DECLARATION_KEY: &str = "typed-acceptance:";
const CRITERIA_HEADING: &str = "## Acceptance criteria";
const TYPED_PREFIX: &str = "- **AC-";
const CLAIM_SEPARATOR: &str = " — ";

/// Whether a criterion is a falsifier or sufficiency evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polarity {
    /// Failing it refutes the intent.
    Necessary,
    /// Failing it leaves the intent undetermined.
    Supporting,
}

impl Polarity {
    fn parse(word: &str) -> Option<Self> {
        match word {
            "necessary" => Some(Polarity::Necessary),
            "supporting" => Some(Polarity::Supporting),
            _ => None,
        }
    }
}

/// The kind of evidence a criterion needs, strongest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modality {
    Formal,
    Empirical,
    Preference,
    Judged,
    Asserted,
}

impl Modality {
    fn parse(word: &str) -> Option<Self> {
        match word {
            "formal" => Some(Modality::Formal),
            "empirical" => Some(Modality::Empirical),
            "preference" => Some(Modality::Preference),
            "judged" => Some(Modality::Judged),
            "asserted" => Some(Modality::Asserted),
            _ => None,
        }
    }

    /// The grammar word for this modality.
    pub fn name(self) -> &'static str {
        match self {
            Modality::Formal => "formal",
            Modality::Empirical => "empirical",
            Modality::Preference => "preference",
            Modality::Judged => "judged",
            Modality::Asserted => "asserted",
        }
    }

    /// Detail keys this modality allows, besides `Evidence` and `Note`.
    fn detail_keys(self) -> &'static [&'static str] {
        match self {
            Modality::Formal => &["Obligation", "Proxy"],
            Modality::Empirical => &["Threshold", "Sample", "Error"],
            Modality::Preference => &["Panel", "Threshold", "Error"],
            Modality::Judged => &["Rubric", "Quorum"],
            Modality::Asserted => &["Signers"],
        }
    }

    fn allows_detail(self, key: &str) -> bool {
        matches!(key, "Evidence" | "Note") || self.detail_keys().contains(&key)
    }
}

/// How a Book's criteria are read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// The Book declares `typed-acceptance: 1`: every criterion must be typed.
    Strict,
    /// No declaration: untyped criteria are lifted to `necessary` `asserted`.
    Port,
}

/// One acceptance criterion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Criterion {
    /// The `AC-n` number, absent for a lifted untyped criterion.
    pub number: Option<u32>,
    pub polarity: Polarity,
    pub modality: Modality,
    /// The claim, with wrapped lines trimmed and joined by single spaces.
    pub claim: String,
    /// `(key, value)` detail items.
    pub details: Vec<(String, String)>,
    /// 1-based line of the criterion's first line.
    pub line: usize,
    /// False when the criterion was lifted by the port rule.
    pub typed: bool,
}

/// The parsed criteria of one intent chapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chapter {
    pub path: PathBuf,
    pub criteria: Vec<Criterion>,
}

/// Which grammar rule a violation breaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Polarity,
    Modality,
    Claim,
    Number,
    Duplicate,
    Detail,
    Untyped,
    Missing,
}

impl Kind {
    /// The rule's name as printed.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Polarity => "polarity",
            Kind::Modality => "modality",
            Kind::Claim => "claim",
            Kind::Number => "number",
            Kind::Duplicate => "duplicate",
            Kind::Detail => "detail",
            Kind::Untyped => "untyped",
            Kind::Missing => "missing",
        }
    }
}

/// A grammar violation at a specific line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub path: PathBuf,
    /// 1-based line number; the heading line (or 1) for `missing`.
    pub line: usize,
    pub kind: Kind,
    pub message: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}: {}",
            self.path.display(),
            self.line,
            self.kind.name(),
            self.message
        )
    }
}

/// The result of checking a whole Book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub rule: Rule,
    pub chapters: Vec<Chapter>,
    pub violations: Vec<Violation>,
}

/// Why a Book could not be checked at all.
#[derive(Debug)]
pub enum CheckError {
    /// The directory has no `.sprint-loop-book` marker, so it is not a Book.
    NotABook { dir: PathBuf },
    /// The marker declares a typed-acceptance version this build does not know.
    UnsupportedDeclaration { value: String },
    /// A file or directory could not be read.
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::NotABook { dir } => write!(
                f,
                "{} is not a Book: it has no {MARKER_FILE} marker",
                dir.display()
            ),
            CheckError::UnsupportedDeclaration { value } => write!(
                f,
                "unsupported typed-acceptance version {value:?}; this build knows only 1"
            ),
            CheckError::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for CheckError {}

/// Checks every intent chapter of the Book rooted at `book_dir`.
pub fn check(book_dir: &Path) -> Result<Report, CheckError> {
    let marker_path = book_dir.join(MARKER_FILE);
    if !marker_path.is_file() {
        return Err(CheckError::NotABook {
            dir: book_dir.to_path_buf(),
        });
    }
    let rule = rule_from_marker(&read(&marker_path)?)?;

    let intents_dir = book_dir.join("intents");
    let io_error = |source| CheckError::Io {
        path: intents_dir.clone(),
        source,
    };
    let mut paths = Vec::new();
    for entry in fs::read_dir(&intents_dir).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        let is_chapter = path.extension().is_some_and(|ext| ext == "md")
            && path.file_name().is_some_and(|name| name != "README.md");
        if is_chapter {
            paths.push(path);
        }
    }
    paths.sort();

    let mut report = Report {
        rule,
        chapters: Vec::new(),
        violations: Vec::new(),
    };
    for path in paths {
        let text = read(&path)?;
        let (chapter, violations) = parse_chapter(&path, &text, rule);
        report.chapters.push(chapter);
        report.violations.extend(violations);
    }
    Ok(report)
}

fn read(path: &Path) -> Result<String, CheckError> {
    fs::read_to_string(path).map_err(|source| CheckError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Reads the rule a Book marker declares.
pub fn rule_from_marker(marker: &str) -> Result<Rule, CheckError> {
    for line in marker.lines() {
        if let Some(value) = line.trim().strip_prefix(DECLARATION_KEY) {
            return match value.trim() {
                "1" => Ok(Rule::Strict),
                other => Err(CheckError::UnsupportedDeclaration {
                    value: other.to_owned(),
                }),
            };
        }
    }
    Ok(Rule::Port)
}

/// Parses the acceptance criteria of one chapter.
pub fn parse_chapter(path: &Path, text: &str, rule: Rule) -> (Chapter, Vec<Violation>) {
    let mut parser = Parser {
        path,
        rule,
        criteria: Vec::new(),
        violations: Vec::new(),
        current: None,
        paragraph: false,
        items: 0,
    };
    let mut fence: Option<char> = None;
    let mut heading_line = None;
    let mut in_section = false;

    for (index, raw) in text.lines().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let number = index + 1;

        let trimmed = line.trim_start();
        if let Some(marker) = fence {
            if trimmed.starts_with(&marker.to_string().repeat(3)) {
                fence = None;
            }
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fence = trimmed.chars().next();
            continue;
        }

        if line.starts_with("###") {
            parser.close();
            continue;
        }
        if line.starts_with("# ") || line.starts_with("## ") {
            parser.close();
            in_section = line.trim_end() == CRITERIA_HEADING;
            if in_section {
                heading_line = Some(number);
            }
            continue;
        }
        if in_section {
            parser.line(line, number);
        }
    }
    parser.close();

    // `missing` is for a section that is absent or holds no items at all; a
    // malformed item already has its own violation.
    if rule == Rule::Strict && parser.items == 0 {
        parser.violations.push(Violation {
            path: path.to_path_buf(),
            line: heading_line.unwrap_or(1),
            kind: Kind::Missing,
            message: "chapter has no acceptance criteria".to_owned(),
        });
    }

    let chapter = Chapter {
        path: path.to_path_buf(),
        criteria: parser.criteria,
    };
    (chapter, parser.violations)
}

struct Parser<'p> {
    path: &'p Path,
    rule: Rule,
    criteria: Vec<Criterion>,
    violations: Vec<Violation>,
    /// The criterion being read; `None` after a malformed or refused line.
    current: Option<Criterion>,
    /// Whether the previous line was column-0 prose, so the next prose line
    /// continues the same paragraph.
    paragraph: bool,
    /// Items started in the criteria section, valid or not.
    items: usize,
}

impl Parser<'_> {
    fn line(&mut self, line: &str, number: usize) {
        if line.trim().is_empty() {
            self.paragraph = false;
            return;
        }
        if let Some(rest) = line.strip_prefix("- ") {
            self.close();
            self.items += 1;
            if line.starts_with(TYPED_PREFIX) {
                self.current = self.typed(line, number);
            } else {
                self.untyped(rest, number);
            }
            return;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            // A column-0 paragraph is one untyped criterion: its first line
            // starts it and the lines after it continue it.
            if self.paragraph {
                if let Some(current) = self.current.as_mut() {
                    current.claim.push(' ');
                    current.claim.push_str(line.trim());
                }
                return;
            }
            self.close();
            self.items += 1;
            self.untyped(line, number);
            self.paragraph = true;
            return;
        }
        let Some(current) = self.current.as_mut() else {
            return;
        };
        let inner = line.trim();
        match inner.strip_prefix("- ") {
            Some(detail) if current.typed => {
                let modality = current.modality;
                match detail.split_once(':') {
                    Some((key, value)) if modality.allows_detail(key.trim()) => current
                        .details
                        .push((key.trim().to_owned(), value.trim().to_owned())),
                    Some((key, _)) => {
                        let message = format!(
                            "detail key {:?} is not allowed for {} criteria",
                            key.trim(),
                            modality.name()
                        );
                        self.violation(number, Kind::Detail, message);
                    }
                    None => self.violation(
                        number,
                        Kind::Detail,
                        "detail item must read `Key: value`".to_owned(),
                    ),
                }
            }
            _ => {
                current.claim.push(' ');
                current.claim.push_str(inner);
            }
        }
    }

    fn typed(&mut self, line: &str, number: usize) -> Option<Criterion> {
        let rest = &line[TYPED_PREFIX.len()..];
        let Some((digits, rest)) = rest.split_once("**") else {
            self.violation(number, Kind::Number, "unterminated `**AC-n**`".to_owned());
            return None;
        };
        let Some(ac) = digits.parse::<u32>().ok().filter(|&n| n > 0) else {
            let message = format!("{digits:?} is not a positive number");
            self.violation(number, Kind::Number, message);
            return None;
        };
        if self.criteria.iter().any(|c| c.number == Some(ac)) {
            self.violation(number, Kind::Duplicate, format!("AC-{ac} appears twice"));
            return None;
        }

        let Some((polarity_word, rest)) = code_span(rest) else {
            self.violation(number, Kind::Polarity, "missing `polarity`".to_owned());
            return None;
        };
        let Some(polarity) = Polarity::parse(polarity_word) else {
            let message = format!("{polarity_word:?} is not necessary or supporting");
            self.violation(number, Kind::Polarity, message);
            return None;
        };

        let Some((modality_word, rest)) = code_span(rest) else {
            self.violation(number, Kind::Modality, "missing `modality`".to_owned());
            return None;
        };
        let Some(modality) = Modality::parse(modality_word) else {
            let message = format!("{modality_word:?} is not a modality");
            self.violation(number, Kind::Modality, message);
            return None;
        };

        let claim = rest.strip_prefix(CLAIM_SEPARATOR).map(str::trim);
        let Some(claim) = claim.filter(|claim| !claim.is_empty()) else {
            let message = format!("expected `{}` followed by a claim", CLAIM_SEPARATOR.trim());
            self.violation(number, Kind::Claim, message);
            return None;
        };

        Some(Criterion {
            number: Some(ac),
            polarity,
            modality,
            claim: claim.to_owned(),
            details: Vec::new(),
            line: number,
            typed: true,
        })
    }

    fn untyped(&mut self, text: &str, number: usize) {
        match self.rule {
            Rule::Strict => {
                self.violation(
                    number,
                    Kind::Untyped,
                    "criterion lacks `**AC-n**` `polarity` `modality`".to_owned(),
                );
                self.current = None;
            }
            Rule::Port => {
                self.current = Some(Criterion {
                    number: None,
                    polarity: Polarity::Necessary,
                    modality: Modality::Asserted,
                    claim: text.trim().to_owned(),
                    details: Vec::new(),
                    line: number,
                    typed: false,
                });
            }
        }
    }

    fn close(&mut self) {
        self.paragraph = false;
        if let Some(criterion) = self.current.take() {
            self.criteria.push(criterion);
        }
    }

    fn violation(&mut self, line: usize, kind: Kind, message: String) {
        self.violations.push(Violation {
            path: self.path.to_path_buf(),
            line,
            kind,
            message,
        });
    }
}

/// Splits `` `word` rest`` (after a leading space) into `word` and `rest`.
fn code_span(text: &str) -> Option<(&str, &str)> {
    let inner = text.strip_prefix(" `")?;
    inner.split_once('`')
}

/// Whether a criterion currently holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Holds,
    Fails,
    Unknown,
}

/// An intent's derived status (INT-0002 semantics).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Every criterion holds.
    Certified,
    /// Some necessary criterion fails.
    Refuted,
    /// Neither.
    Undetermined,
}

/// Derives an intent's status from its criteria's outcomes. An intent with no
/// criteria is undetermined, never vacuously certified.
pub fn status(criteria: &[(Polarity, Outcome)]) -> Status {
    let refuted = criteria
        .iter()
        .any(|&(polarity, outcome)| polarity == Polarity::Necessary && outcome == Outcome::Fails);
    if refuted {
        return Status::Refuted;
    }
    let all_hold = criteria
        .iter()
        .all(|&(_, outcome)| outcome == Outcome::Holds);
    if all_hold && !criteria.is_empty() {
        Status::Certified
    } else {
        Status::Undetermined
    }
}
