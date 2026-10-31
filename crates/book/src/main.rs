#![forbid(unsafe_code)]
//! `book [BOOK_DIR]` checks a Book's acceptance criteria (default `docs`).
//!
//! Checking is idempotent, so the command takes no flags; the Book's own
//! marker says whether the strict or the port rule applies.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use book::{Rule, check};

const USAGE: &str = "usage: book [BOOK_DIR]   (default: docs; no flags: checking is idempotent)";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let dir = match args.as_slice() {
        [] => PathBuf::from("docs"),
        [dir] if !dir.starts_with('-') => PathBuf::from(dir),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    let report = match check(&dir) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("book: {error}");
            return ExitCode::from(2);
        }
    };

    let rule = match report.rule {
        Rule::Strict => "typed-acceptance 1, strict",
        Rule::Port => "undeclared, port rule",
    };
    println!("book: {} ({rule})", dir.display());
    let mut criteria = 0;
    for chapter in &report.chapters {
        criteria += chapter.criteria.len();
        let name = chapter
            .path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default();
        println!("  {name}: {} criteria", chapter.criteria.len());
    }
    for violation in &report.violations {
        println!("{violation}");
    }
    println!(
        "{} chapters, {criteria} criteria, {} violations",
        report.chapters.len(),
        report.violations.len()
    );

    if report.violations.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
