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

#[test]
fn prior_art_credits_every_listed_system() {
    let credits = repo_file("docs/prior-art.md");
    let missing: Vec<&str> = CREDITED
        .into_iter()
        .filter(|name| !credits.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/prior-art.md does not credit {missing:?}"
    );
}

#[test]
fn readme_keeps_statement_and_links_book() {
    let readme = repo_file("README.md");
    for paragraph in OWNER_PARAGRAPHS {
        assert!(
            readme.lines().any(|line| line == paragraph),
            "the owner's paragraph must stay verbatim: {paragraph:?}"
        );
    }
    assert!(
        readme.contains("](docs/SUMMARY.md)"),
        "README must link the Book"
    );
    for command in ["cargo ci", "cargo policy", "cargo book", "cargo deny check"] {
        assert!(readme.contains(command), "README must document `{command}`");
    }
    assert!(
        readme.lines().any(|line| line == NO_FLAGS),
        "README must say exactly: {NO_FLAGS}"
    );
}
