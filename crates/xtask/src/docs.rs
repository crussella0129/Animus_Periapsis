//! Documentation checks: design credits (supporting evidence for the owner's
//! INT-0003 AC-5 review) and the README (INT-0003 AC-10).

use std::fs;
use std::path::Path;

fn repo_file(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Systems the Book's intents adopt or evaluate, the owner's projects and the
/// third-party test data. Completeness beyond this list is the owner's review.
const CREDITED: [&str; 36] = [
    "Jujutsu",
    "Mercurial",
    "Sapling",
    "Mononoke",
    "Radicle",
    "Git",
    "JGit",
    "Mergiraf",
    "Spork",
    "TigerBeetle",
    "turmoil",
    "madsim",
    "Zuul",
    "bors",
    "SubmitQueue",
    "Lean 4",
    "comparator",
    "nanoda",
    "lean4export",
    "Aeneas",
    "Pijul",
    "Darcs",
    "Xet",
    "Negentropy",
    "zstd",
    "Agent Trace",
    "in-toto",
    "DSSE",
    "SLSA",
    "Model Context Protocol",
    "Scion",
    "Fossil",
    "Animus_Sprint_Loops",
    "CubiKan",
    "array-test",
    "BLAKE3 test vectors",
];

/// The owner's original README paragraphs, as of commit `620f99c`, kept
/// verbatim (the second really does end with a space).
const OWNER_PARAGRAPHS: [&str; 2] = [
    "Rust based, agent-human hybrid Distributed Version Control System (DCVS), based on the concepts of the Animus_Sprint_Loops recursive development system, CubiKan, array-test (both my repositories), with research on the best parts of Git, Jujitsu, Mercurial, and other DCVSs",
    "Visual Branding intent for the readme (and github pages if i go that route) - like general Radahn's sword, very crude yet mystical looking bent field lines and objects trajectories, emblazoned in stone, associated with 'gravity magick', bc the name of the product is Apsis, which is an orbit like term. Perhaps the Readme hero is like a figure doing a GFOB around a celestial body or something, emblazoned in the stone. ",
];

const NO_FLAGS: &str = "`cargo ci`, `cargo policy` and `cargo book` take no flags.";

/// Whether `text` names `name` as a whole word, with no ASCII letter or digit
/// right before or after it, so that `JGit` does not credit Git.
fn names(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + name.len()..].chars().next();
        !before
            .into_iter()
            .chain(after)
            .any(|c| c.is_ascii_alphanumeric())
    })
}

#[test]
fn prior_art_credits_every_listed_system() {
    let credits = repo_file("docs/prior-art.md");
    let missing: Vec<&str> = CREDITED
        .into_iter()
        .filter(|name| !names(&credits, name))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/prior-art.md does not credit {missing:?}"
    );
}

#[test]
fn credited_names_are_whole_words() {
    assert!(!names("Histogram diff from [JGit](url)", "Git"));
    assert!(!names("[JGit](url)", "Git"));
    assert!(names("[JGit](url) and [Git](url)", "Git"));
    assert!(names("Git", "Git"));
    assert!(names("Git's object model", "Git"));
    assert!(!names("Gitea", "Git"));
    assert!(!names("Lean 40", "Lean 4"));
    assert!(names("[Lean 4](url)", "Lean 4"));
}

/// What a README lacks against the T-008 clause: the owner's paragraphs
/// verbatim, a link to the Book, `cargo ci` and `cargo deny check` as command
/// lines of their own in fenced blocks, what `cargo policy` and `cargo book`
/// do (on a line besides the no-flags sentence), and that sentence exactly.
fn readme_problems(readme: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for paragraph in OWNER_PARAGRAPHS {
        if !readme.lines().any(|line| line == paragraph) {
            problems.push(format!(
                "the owner's paragraph must stay verbatim: {paragraph:?}"
            ));
        }
    }
    if !readme.contains("](docs/SUMMARY.md)") {
        problems.push("README must link the Book".to_owned());
    }
    let fenced = fenced_lines(readme);
    for command in ["cargo ci", "cargo deny check"] {
        if !fenced.contains(&command) {
            problems.push(format!("README must show `{command}` as a command line"));
        }
    }
    let described = readme.lines().any(|line| {
        line != NO_FLAGS && line.contains("`cargo policy`") && line.contains("`cargo book`")
    });
    if !described {
        problems.push("README must say what `cargo policy` and `cargo book` do".to_owned());
    }
    if !readme.lines().any(|line| line == NO_FLAGS) {
        problems.push(format!("README must say exactly: {NO_FLAGS}"));
    }
    problems
}

/// The trimmed lines inside fenced code blocks.
fn fenced_lines(text: &str) -> Vec<&str> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            inside = !inside;
        } else if inside {
            lines.push(line.trim());
        }
    }
    lines
}

/// The README passes, and each change to what it must keep fails with that
/// problem alone: the Book named but not linked, either owner paragraph
/// extended or trimmed (the second loses only its trailing space), and the
/// no-flags sentence reworded.
#[test]
fn readme_keeps_statement_and_links_book() {
    assert_eq!(
        readme_problems(&repo_file("README.md")),
        Vec::<String>::new()
    );

    let readme = repo_file("README.md").replace("\r\n", "\n");
    let replace = |from: &str, to: &str| {
        assert!(readme.contains(from), "{from:?} is not in README.md");
        readme.replace(from, to)
    };
    let unlinked = replace("[`docs/SUMMARY.md`](docs/SUMMARY.md)", "`docs/SUMMARY.md`");
    assert_eq!(readme_problems(&unlinked), ["README must link the Book"]);

    let [first, second] = OWNER_PARAGRAPHS;
    let extended = replace(&format!("{first}\n"), &format!("{first} (edited)\n"));
    assert_eq!(
        readme_problems(&extended),
        [format!(
            "the owner's paragraph must stay verbatim: {first:?}"
        )]
    );
    let trimmed = replace(&format!("{second}\n"), &format!("{}\n", second.trim_end()));
    assert_eq!(
        readme_problems(&trimmed),
        [format!(
            "the owner's paragraph must stay verbatim: {second:?}"
        )]
    );

    let reworded = replace(
        &format!("{NO_FLAGS}\n"),
        "`cargo ci` and `cargo policy` take no flags.\n",
    );
    assert_eq!(
        readme_problems(&reworded),
        [format!("README must say exactly: {NO_FLAGS}")]
    );
}

/// READMEs that only mention the commands fail: the build section cut to
/// the no-flags sentence, a narrowed, changed or unfenced command line, and
/// no line on what `cargo policy` and `cargo book` do.
#[test]
fn readme_must_document_each_command() {
    let readme = repo_file("README.md").replace("\r\n", "\n");
    let replace = |from: &str, to: &str| {
        assert!(readme.contains(from), "{from:?} is not in README.md");
        readme.replace(from, to)
    };
    let ci = "README must show `cargo ci` as a command line";
    let deny = "README must show `cargo deny check` as a command line";
    let described = "README must say what `cargo policy` and `cargo book` do";

    let (head, rest) = readme.split_once("Requires rustup").unwrap();
    let (_, tail) = rest.split_once("Designs borrowed").unwrap();
    let cut = format!("{head}{NO_FLAGS}\nNever run `cargo deny check`.\n\nDesigns borrowed{tail}");
    assert_eq!(readme_problems(&cut), [ci, deny, described]);

    let narrowed = replace(
        "```bash\ncargo deny check\n```",
        "```bash\ncargo deny check bans\n```",
    );
    assert_eq!(readme_problems(&narrowed), [deny]);
    let changed = replace("```bash\ncargo ci\n```", "```bash\ncargo ci --fast\n```");
    assert_eq!(readme_problems(&changed), [ci]);
    let unfenced = replace("```bash\ncargo ci\n```", "cargo ci");
    assert_eq!(readme_problems(&unfenced), [ci]);
    let undescribed = replace(
        "`cargo policy` and `cargo book` run those last two checks on their own.\n",
        "",
    );
    assert_eq!(readme_problems(&undescribed), [described]);
}
