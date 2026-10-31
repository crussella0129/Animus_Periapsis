#![forbid(unsafe_code)]
//! Reads the acceptance criteria of a Sprint Loops Book and checks them
//! against the typed-acceptance grammar (INT-0002).
//!
//! A Book that declares `typed-acceptance: 1` in `docs/.sprint-loop-book` is
//! checked strictly: every criterion must be typed. A Book without the
//! declaration is read with the port rule, which lifts each untyped criterion
//! to `necessary` `asserted`. Any other declared value, a second declaration,
//! or a directory with no marker, is refused.
//!
//! Chapters are read as CommonMark, the way GitHub renders the Book: the
//! criteria are the top-level list items of the `## Acceptance criteria`
//! section, and code blocks, HTML blocks and thematic breaks never hold one.

use std::fmt;
use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, HeadingLevel, OffsetIter, Parser, Tag, TagEnd};

const MARKER_FILE: &str = ".sprint-loop-book";
const DECLARATION_KEY: &str = "typed-acceptance:";
const CRITERIA_TITLE: &str = "Acceptance criteria";
const TYPED_PREFIX: &str = "**AC-";
const CLAIM_SEPARATOR: &str = " — ";

/// The list depth of a criterion's details. Lists nested deeper hold nothing
/// the grammar reads, so the reader skips them without recursing, and a
/// deeply nested chapter cannot exhaust the stack.
const DETAIL_DEPTH: usize = 2;

/// Block quotes nested deeper than this inside an item are skipped likewise.
const MAX_QUOTES: usize = 8;

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
    /// The marker declares typed-acceptance more than once, so it names no
    /// single contract.
    DuplicateDeclaration,
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
            CheckError::DuplicateDeclaration => write!(
                f,
                "the {MARKER_FILE} marker declares typed-acceptance more than once"
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

/// Reads the rule a Book marker declares. A marker declares
/// `typed-acceptance` at most once, and only with the value `1`.
pub fn rule_from_marker(marker: &str) -> Result<Rule, CheckError> {
    let mut values = marker
        .lines()
        .filter_map(|line| line.trim().strip_prefix(DECLARATION_KEY));
    let Some(value) = values.next() else {
        return Ok(Rule::Port);
    };
    if values.next().is_some() {
        return Err(CheckError::DuplicateDeclaration);
    }
    match value.trim() {
        "1" => Ok(Rule::Strict),
        other => Err(CheckError::UnsupportedDeclaration {
            value: other.to_owned(),
        }),
    }
}

/// Parses the acceptance criteria of one chapter.
pub fn parse_chapter(path: &Path, text: &str, rule: Rule) -> (Chapter, Vec<Violation>) {
    // CommonMark ends a line at a lone CR too, but pulldown-cmark does not when
    // it scans fences, HTML blocks and indented code. Each lone CR becomes an
    // LF first; that swaps one byte for one, so lines keep their numbers.
    let normalized;
    let text = if text.contains('\r') {
        normalized = lone_cr_to_lf(text);
        normalized.as_str()
    } else {
        text
    };
    let section = Reader::new(text).section();
    let mut checker = Checker {
        path,
        rule,
        criteria: Vec::new(),
        violations: Vec::new(),
    };
    for block in &section.blocks {
        checker.block(block);
    }

    // `missing` is for a section that is absent or holds nothing that could
    // be a criterion; a malformed item or stray prose has its own violation.
    if rule == Rule::Strict && section.blocks.is_empty() {
        checker.violation(
            section.heading.unwrap_or(1),
            Kind::Missing,
            "chapter has no acceptance criteria".to_owned(),
        );
    }

    let chapter = Chapter {
        path: path.to_path_buf(),
        criteria: checker.criteria,
    };
    (chapter, checker.violations)
}

/// `text` with each CR that is not followed by an LF replaced by an LF.
fn lone_cr_to_lf(text: &str) -> String {
    let bytes = text.as_bytes();
    text.char_indices()
        .map(|(at, c)| match c {
            '\r' if bytes.get(at + 1) != Some(&b'\n') => '\n',
            c => c,
        })
        .collect()
}

/// The blocks of a chapter's criteria section, as CommonMark reads them.
struct Section {
    /// 1-based line of the first `## Acceptance criteria` heading.
    heading: Option<usize>,
    blocks: Vec<Block>,
}

/// A block directly in the criteria section.
enum Block {
    /// A top-level list item: a criterion, typed or not.
    Item(Item),
    /// A paragraph or block quote, at its first line: prose where the grammar
    /// allows only list items.
    Prose(usize),
}

/// A list item: its own text and the items of the lists nested directly in it.
struct Item {
    /// 1-based line of the item's first line.
    line: usize,
    /// The item's paragraphs and headings, outside nested lists, code and
    /// HTML: each source line trimmed and the lines joined by single spaces.
    text: String,
    children: Vec<Item>,
}

/// Walks a chapter's CommonMark events, keeping what the grammar needs.
struct Reader<'t> {
    text: &'t str,
    events: OffsetIter<'t>,
    /// Byte offset of the start of each line.
    line_starts: Vec<usize>,
}

impl<'t> Reader<'t> {
    /// A reader of `text`, whose lines end at LF or CRLF.
    fn new(text: &'t str) -> Self {
        let line_starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        Reader {
            text,
            events: Parser::new(text).into_offset_iter(),
            line_starts,
        }
    }

    /// The 1-based line that holds byte `offset`.
    fn line(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= offset)
    }

    /// The offset of the first character at or after `offset` that is not
    /// whitespace: a nested item's range can begin at the line break before
    /// its marker.
    fn visible(&self, offset: usize) -> usize {
        self.text[offset..]
            .find(|c: char| !c.is_whitespace())
            .map_or(offset, |at| offset + at)
    }

    /// Reads the blocks of every `## Acceptance criteria` section. A section
    /// ends at the next top-level heading of level 1 or 2; a heading inside a
    /// list item or block quote belongs to that container.
    fn section(mut self) -> Section {
        let mut section = Section {
            heading: None,
            blocks: Vec::new(),
        };
        let mut inside = false;
        while let Some((event, range)) = self.events.next() {
            // Besides start tags, only thematic breaks appear at this level.
            let Event::Start(tag) = event else {
                continue;
            };
            let line = self.line(range.start);
            match tag {
                Tag::Heading { level, .. } => {
                    let title = self.heading_text();
                    if level <= HeadingLevel::H2 {
                        inside = level == HeadingLevel::H2 && title == CRITERIA_TITLE;
                        if inside {
                            section.heading.get_or_insert(line);
                        }
                    }
                }
                Tag::List(_) if inside => {
                    let items = self.list(1);
                    section.blocks.extend(items.into_iter().map(Block::Item));
                }
                Tag::Paragraph | Tag::BlockQuote(_) if inside => {
                    self.skip();
                    section.blocks.push(Block::Prose(line));
                }
                _ => self.skip(),
            }
        }
        section
    }

    /// Reads a heading's text, up to its end tag.
    fn heading_text(&mut self) -> String {
        let mut run = None;
        for (event, range) in self.events.by_ref() {
            if !is_inline(&event) {
                break;
            }
            run = Some(cover(run, range));
        }
        run.map(|run| self.join(run, 0)).unwrap_or_default()
    }

    /// Reads a list's items, up to its end tag; `depth` is the list's depth
    /// in the section, 1 for criteria.
    fn list(&mut self, depth: usize) -> Vec<Item> {
        let mut items = Vec::new();
        while let Some((event, range)) = self.events.next() {
            match event {
                Event::Start(Tag::Item) => items.push(self.item(range.start, depth)),
                Event::End(_) => break,
                _ => {}
            }
        }
        items
    }

    /// Reads a list item at `depth`, up to its end tag.
    fn item(&mut self, start: usize, depth: usize) -> Item {
        let mut parts = Vec::new();
        let mut children = Vec::new();
        self.container(0, depth, &mut parts, &mut children);
        Item {
            line: self.line(self.visible(start)),
            text: parts.join(" "),
            children,
        }
    }

    /// Reads a list item or a block quote, up to its end tag: the text of each
    /// paragraph and heading goes to `parts`, and the items of each list
    /// nested directly in the list item go to `children`. `quotes` counts the
    /// block quotes open inside the item, whose lists are not its own; `depth`
    /// is the item's list depth.
    fn container(
        &mut self,
        quotes: usize,
        depth: usize,
        parts: &mut Vec<String>,
        children: &mut Vec<Item>,
    ) {
        // A tight list item's text comes without paragraph tags, so text is
        // gathered in runs of inline events that end at any block event.
        let mut run = None;
        while let Some((event, range)) = self.events.next() {
            if is_inline(&event) {
                run = Some(cover(run, range));
                continue;
            }
            if let Some(run) = run.take() {
                parts.push(self.join(run, quotes));
            }
            match event {
                Event::Start(Tag::Paragraph | Tag::Heading { .. })
                | Event::End(TagEnd::Paragraph | TagEnd::Heading(_))
                | Event::Rule => {}
                Event::Start(Tag::BlockQuote(_)) if quotes < MAX_QUOTES => {
                    self.container(quotes + 1, depth, parts, children);
                }
                Event::Start(Tag::List(_)) if quotes == 0 && depth < DETAIL_DEPTH => {
                    children.extend(self.list(depth + 1));
                }
                Event::Start(_) => self.skip(),
                Event::End(_) => break,
                _ => {}
            }
        }
    }

    /// Consumes the rest of the block whose start tag was just read.
    fn skip(&mut self) {
        let mut depth = 1;
        for (event, _) in self.events.by_ref() {
            match event {
                Event::Start(_) => depth += 1,
                Event::End(_) => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                break;
            }
        }
    }

    /// The source in `range` with each line trimmed, continuation lines
    /// stripped of up to `quotes` block-quote markers, and the lines joined by
    /// single spaces.
    fn join(&self, range: Range<usize>, quotes: usize) -> String {
        // Splitting at both CR and LF leaves an empty piece inside each CRLF.
        let mut lines = self.text[range].split(['\r', '\n']).map(str::trim);
        let first = lines.next().unwrap_or_default().to_owned();
        lines
            .map(|line| strip_quotes(line, quotes))
            .filter(|line| !line.is_empty())
            .fold(first, |mut text, line| {
                text.push(' ');
                text.push_str(line);
                text
            })
    }
}

/// Whether an event is inline content: text, a code span, inline HTML, a line
/// break, or the start or end of emphasis, a link or an image.
fn is_inline(event: &Event<'_>) -> bool {
    match event {
        Event::Start(tag) => is_inline_tag(tag.to_end()),
        Event::End(tag) => is_inline_tag(*tag),
        Event::Rule => false,
        _ => true,
    }
}

fn is_inline_tag(tag: TagEnd) -> bool {
    matches!(
        tag,
        TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript
            | TagEnd::Link
            | TagEnd::Image
    )
}

/// Extends a run of inline events to cover `range`.
fn cover(run: Option<Range<usize>>, range: Range<usize>) -> Range<usize> {
    match run {
        Some(run) => run.start..run.end.max(range.end),
        None => range,
    }
}

/// Strips up to `quotes` block-quote markers from a trimmed line. A lazy
/// continuation line has fewer markers, or none.
fn strip_quotes(mut line: &str, quotes: usize) -> &str {
    for _ in 0..quotes {
        let Some(rest) = line.strip_prefix('>') else {
            break;
        };
        line = rest.trim_start();
    }
    line
}

/// Checks a section's blocks against the grammar under one rule.
struct Checker<'p> {
    path: &'p Path,
    rule: Rule,
    criteria: Vec<Criterion>,
    violations: Vec<Violation>,
}

impl Checker<'_> {
    fn block(&mut self, block: &Block) {
        match block {
            Block::Item(item) => self.item(item),
            // The grammar defines criteria as list items, so the port rule
            // reads other prose as commentary.
            Block::Prose(line) if self.rule == Rule::Strict => self.violation(
                *line,
                Kind::Untyped,
                "criteria must be typed list items".to_owned(),
            ),
            Block::Prose(_) => {}
        }
    }

    /// Checks one top-level item. The strict rule types every item; the port
    /// rule types only an item that carries type tags and lifts the rest.
    fn item(&mut self, item: &Item) {
        let typed = match self.rule {
            Rule::Strict => item.text.starts_with(TYPED_PREFIX),
            Rule::Port => is_tagged(&item.text),
        };
        if !typed {
            self.untyped(item);
        } else if let Some(mut criterion) = self.typed(&item.text, item.line) {
            for detail in &item.children {
                self.detail(&mut criterion, detail);
            }
            self.criteria.push(criterion);
        }
    }

    fn typed(&mut self, text: &str, line: usize) -> Option<Criterion> {
        let rest = &text[TYPED_PREFIX.len()..];
        let Some((digits, rest)) = rest.split_once("**") else {
            self.violation(line, Kind::Number, "unterminated `**AC-n**`".to_owned());
            return None;
        };
        let Some(ac) = parse_number(digits) else {
            let message = format!("{digits:?} is not a positive number");
            self.violation(line, Kind::Number, message);
            return None;
        };
        if self.criteria.iter().any(|c| c.number == Some(ac)) {
            self.violation(line, Kind::Duplicate, format!("AC-{ac} appears twice"));
            return None;
        }

        let Some((polarity_word, rest)) = code_span(rest) else {
            self.violation(line, Kind::Polarity, "missing `polarity`".to_owned());
            return None;
        };
        let Some(polarity) = Polarity::parse(polarity_word) else {
            let message = format!("{polarity_word:?} is not necessary or supporting");
            self.violation(line, Kind::Polarity, message);
            return None;
        };

        let Some((modality_word, rest)) = code_span(rest) else {
            self.violation(line, Kind::Modality, "missing `modality`".to_owned());
            return None;
        };
        let Some(modality) = Modality::parse(modality_word) else {
            let message = format!("{modality_word:?} is not a modality");
            self.violation(line, Kind::Modality, message);
            return None;
        };

        let claim = rest.strip_prefix(CLAIM_SEPARATOR).map(str::trim);
        let Some(claim) = claim.filter(|claim| !claim.is_empty()) else {
            let message = format!("expected `{}` followed by a claim", CLAIM_SEPARATOR.trim());
            self.violation(line, Kind::Claim, message);
            return None;
        };

        Some(Criterion {
            number: Some(ac),
            polarity,
            modality,
            claim: claim.to_owned(),
            details: Vec::new(),
            line,
            typed: true,
        })
    }

    /// Adds a detail item to a typed criterion, or reports why it cannot be one.
    fn detail(&mut self, criterion: &mut Criterion, detail: &Item) {
        let modality = criterion.modality;
        match detail.text.split_once(':') {
            Some((key, value)) if modality.allows_detail(key.trim()) => criterion
                .details
                .push((key.trim().to_owned(), value.trim().to_owned())),
            Some((key, _)) => {
                let message = format!(
                    "detail key {:?} is not allowed for {} criteria",
                    key.trim(),
                    modality.name()
                );
                self.violation(detail.line, Kind::Detail, message);
            }
            None => self.violation(
                detail.line,
                Kind::Detail,
                "detail item must read `Key: value`".to_owned(),
            ),
        }
    }

    fn untyped(&mut self, item: &Item) {
        match self.rule {
            Rule::Strict => self.violation(
                item.line,
                Kind::Untyped,
                "criterion lacks `**AC-n**` `polarity` `modality`".to_owned(),
            ),
            Rule::Port => self.criteria.push(Criterion {
                number: None,
                polarity: Polarity::Necessary,
                modality: Modality::Asserted,
                claim: item.text.clone(),
                details: Vec::new(),
                line: item.line,
                typed: false,
            }),
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

/// Whether an item's text carries type tags: `**AC-<label>**` followed by a
/// space and a code span.
fn is_tagged(text: &str) -> bool {
    text.strip_prefix(TYPED_PREFIX)
        .and_then(|rest| rest.split_once("**"))
        .is_some_and(|(_, rest)| code_span(rest).is_some())
}

/// Reads the `<n>` of `AC-<n>`: ASCII digits without a leading zero that fit
/// in a `u32`, so at least 1.
fn parse_number(digits: &str) -> Option<u32> {
    if digits.starts_with('0') || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
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
