//! What a refusal says to do next (INT-0037 AC-4).
//!
//! Every refusal `praps` prints says what was wrong and then ends with
//! exactly one line that starts `hint:`. That line gives one of three
//! things: a command to run next; what to put right outside `praps`; or
//! that this build has no remedy yet and that nothing was lost. [`Hint`] is
//! those three.
//!
//! Every hint `praps` can print is made in this module, so that one test
//! reads them all. [`for_error`] gives the hint for each error of the
//! `workspace` crate by a `match` with no catch-all arm, so an error added
//! there does not compile until it has one.

use std::ffi::OsStr;
use std::fmt;

use crate::registry::Command;

/// The one line a refusal ends with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hint {
    /// A command line to run next, and to what end.
    Run(String, &'static str),
    /// What to put right outside `praps`.
    Fix(&'static str),
    /// What this build cannot do yet. Nothing was lost.
    NoRemedy(&'static str),
}

impl fmt::Display for Hint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hint::Run(line, to) => write!(f, "hint: run `{line}` {to}"),
            Hint::Fix(what) => write!(f, "hint: {what}"),
            Hint::NoRemedy(what) => {
                write!(f, "hint: this build cannot {what} yet; nothing was lost")
            }
        }
    }
}

fn run(line: &str, to: &'static str) -> Hint {
    Hint::Run(line.to_owned(), to)
}

/// What `praps` says was wrong, for an error of the `workspace` crate, after
/// it has named the working copy: the error's own text, but for the one
/// error whose text names the directory itself. That would name it twice,
/// the second time as the workspace was given it, which from the top is
/// `.`.
pub fn said(error: &workspace::Error) -> String {
    match error {
        workspace::Error::NotInitialised(_) => "not an initialised Periapsis repository".to_owned(),
        other => other.to_string(),
    }
}

/// The hint for an error of the `workspace` crate.
pub fn for_error(error: &workspace::Error) -> Hint {
    use workspace::Error;
    match error {
        Error::Io(_) => run(
            "praps verify",
            "to check what is stored; if it is clean, the cause is outside the repository",
        ),
        Error::NotInitialised(_) => run(
            "praps init",
            "in that directory to finish making it a repository",
        ),
        Error::Corrupt(_) => run(
            "praps verify",
            "to list what is damaged in the store; this build cannot repair what it finds, or \
             what the message names outside the store",
        ),
        Error::Merge(_) => Hint::NoRemedy("read a change that has several parents"),
        Error::Hazards(_) => Hint::NoRemedy("write those paths on this filesystem"),
        Error::Blocked(_) => {
            Hint::Fix("move those entries out of the way, then run the command again")
        }
        Error::NothingToUndo => run("praps ops", "to list the operations there are"),
        Error::UnknownOp(_) => operation_ids(),
        Error::AmbiguousOp(..) => {
            Hint::Fix("give more digits of the id, enough to name one of those")
        }
        Error::LaneExists(_) => {
            Hint::Fix("give the lane a directory whose last name no workspace has")
        }
        Error::NotFree(_) => Hint::Fix("give a directory that does not exist yet, or an empty one"),
        Error::UnfinishedLane(_, dir) => Hint::Run(
            format!("praps lane {}", dir.display()),
            "from the working copy it was made from, to finish the lane",
        ),
        Error::StoreGone(..) => Hint::Fix(
            "put the store back where this lane names it; a lane holds no store of its own",
        ),
        Error::BadWorkspaceFile(..) => {
            Hint::Fix("put that file right: it holds two lines, `name NAME` and `store PATH`")
        }
        Error::NotInWorkspace(..) => run(
            "praps ops",
            "to list the operations, and give one made since this workspace was",
        ),
    }
}

/// Arguments that fit no command line of `command`.
pub fn usage(command: &Command) -> Hint {
    Hint::Run(format!("praps help {}", command.name), "for what it takes")
}

/// A command line that names no command, and a word that is not one.
pub fn list() -> Hint {
    run("praps help", "to list the commands")
}

/// An argument that cannot be an operation's id.
pub fn operation_ids() -> Hint {
    run("praps ops", "to list the operations and their ids")
}

/// Text for `describe` that has no UTF-8 form.
pub fn unicode_text() -> Hint {
    Hint::Fix("give TEXT as Unicode text, which can be stored as UTF-8")
}

/// Text for `describe` given as several words: the command line with the
/// words as one quoted text, followed by `to`. `None` when a word holds a
/// character that a shell may change inside quotes, or that could not be
/// shown as it was typed; the caller then gives a hint that shows no text.
pub fn quoted(words: &[&OsStr], to: &'static str) -> Option<Hint> {
    let plain = |word: &str| {
        !word.is_empty()
            && word
                .chars()
                .all(|c| !c.is_control() && !matches!(c, '"' | '`' | '\\' | '$' | '%' | '!'))
    };
    let words: Vec<&str> = words
        .iter()
        .map(|word| word.to_str().filter(|word| plain(word)))
        .collect::<Option<_>>()?;
    Some(Hint::Run(
        format!("praps describe \"{}\"", words.join(" ")),
        to,
    ))
}

/// Text for `describe` given as several words that cannot be shown.
pub fn quote_the_text() -> Hint {
    run(
        "praps help describe",
        "for how to give text of more than one word",
    )
}

/// A directory given that is not there, or is not a directory.
pub fn missing_directory() -> Hint {
    Hint::Fix("give a directory that exists, or none for the current directory")
}

/// A directory that could not be listed, for a reason other than absence.
pub fn unreadable_directory() -> Hint {
    Hint::Fix("make that directory readable, then run the command again")
}

/// A file a command must read to know what it acts on, which it could not.
pub fn unreadable_file() -> Hint {
    Hint::Fix("make that file readable, then run the command again")
}

/// A directory above the starting one whose `.praps/store` could not be
/// listed. The message names only where the search started.
pub fn unreadable_above() -> Hint {
    Hint::Fix(
        "make the store of each repository above that directory readable, then run the \
         command again",
    )
}

/// The top of the working copy could not be made the current directory.
pub fn cannot_enter() -> Hint {
    Hint::Fix("make that directory one you may enter, then run the command again")
}

/// No repository at the directory asked about or above it.
pub fn no_repository() -> Hint {
    run(
        "praps init",
        "in the directory that is to be the top of the working copy",
    )
}

/// A current directory that cannot be read.
pub fn current_directory() -> Hint {
    Hint::Fix("change to a directory that exists, or give the working copy's directory")
}

/// `init` in a directory that lies inside a working copy.
pub fn inside_a_working_copy() -> Hint {
    run(
        "praps status",
        "to work in that working copy, or give `praps init` a directory outside it",
    )
}

/// A lane's directory whose last name is no plain name.
pub fn lane_name() -> Hint {
    Hint::Fix("give a directory whose last name is plain text, which names the lane")
}

/// A lane's directory inside a working copy.
pub fn lane_outside() -> Hint {
    Hint::Fix("give a directory outside every working copy, beside this one for example")
}

/// `init` stopped by the filesystem.
pub fn init_again() -> Hint {
    run(
        "praps init",
        "again once the cause is put right: it finishes what it began",
    )
}

/// `verify` given a directory that is not there, or is not a directory.
pub fn store_or_working_copy() -> Hint {
    Hint::Fix("give the directory of a working copy or of a store")
}

/// `verify` on a directory that is neither a store nor in a working copy.
pub fn not_a_store() -> Hint {
    run(
        "praps init",
        "to make a repository, or give `praps verify` the directory of one",
    )
}

/// `verify` stopped by the filesystem while it read the store.
pub fn unreadable_store() -> Hint {
    Hint::Fix("make the store's directories readable, then run the command again")
}

/// `verify` on a working copy whose store lacks `objects` or `ops`.
pub fn unfinished_store() -> Hint {
    run(
        "praps init",
        "there if its `init` was interrupted; if the store was whole before, put it back from \
         a copy of the repository",
    )
}

/// `verify` found problems.
pub fn damaged_store() -> Hint {
    Hint::Fix(
        "put back the files named above from a copy of the repository; this build cannot \
         repair a store",
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::io;
    use std::path::PathBuf;

    use hash::HashAlg;
    use workspace::Error;

    use super::*;
    use crate::{help, registry};

    /// The variant an error is, by name. It has no catch-all arm, so a
    /// variant added to the crate must be named here, and
    /// `every_error_kind_has_a_hint` then asks for a sample of it.
    fn variant(error: &Error) -> &'static str {
        match error {
            Error::Io(_) => "Io",
            Error::NotInitialised(_) => "NotInitialised",
            Error::Corrupt(_) => "Corrupt",
            Error::Merge(_) => "Merge",
            Error::Hazards(_) => "Hazards",
            Error::Blocked(_) => "Blocked",
            Error::NothingToUndo => "NothingToUndo",
            Error::UnknownOp(_) => "UnknownOp",
            Error::AmbiguousOp(..) => "AmbiguousOp",
            Error::LaneExists(_) => "LaneExists",
            Error::NotFree(_) => "NotFree",
            Error::UnfinishedLane(..) => "UnfinishedLane",
            Error::StoreGone(..) => "StoreGone",
            Error::BadWorkspaceFile(..) => "BadWorkspaceFile",
            Error::NotInWorkspace(..) => "NotInWorkspace",
        }
    }

    const VARIANTS: [&str; 15] = [
        "Io",
        "NotInitialised",
        "Corrupt",
        "Merge",
        "Hazards",
        "Blocked",
        "NothingToUndo",
        "UnknownOp",
        "AmbiguousOp",
        "LaneExists",
        "NotFree",
        "UnfinishedLane",
        "StoreGone",
        "BadWorkspaceFile",
        "NotInWorkspace",
    ];

    /// One error of each variant, as the crate would make it.
    fn samples() -> Vec<Error> {
        let id = |text: &[u8]| HashAlg::DEFAULT.hash(text);
        let path = codec::RepoPath::new("CON".to_owned()).unwrap();
        vec![
            Error::Io(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Access is denied. (os error 5)",
            )),
            Error::NotInitialised(PathBuf::from("project")),
            Error::Corrupt("2 of 9 op files do not hash to their names or do not decode".into()),
            Error::Merge(id(b"merge")),
            Error::Hazards(vec![workspace::hazard::Hazard {
                path,
                component: "CON".to_owned(),
                problem: workspace::hazard::Problem::Reserved("CON".to_owned()),
            }]),
            Error::Blocked(vec![
                "b: an untracked entry is where a file must be written".into(),
            ]),
            Error::NothingToUndo,
            Error::UnknownOp("3f9a".into()),
            Error::AmbiguousOp("3".into(), vec![id(b"one"), id(b"two")]),
            Error::LaneExists("agent-17".into()),
            Error::NotFree(PathBuf::from("../agent-17")),
            Error::UnfinishedLane("agent-17".into(), PathBuf::from("../agent-17")),
            Error::StoreGone(
                PathBuf::from("/work/project/.praps/store"),
                "The system cannot find the path specified. (os error 3)".into(),
            ),
            Error::BadWorkspaceFile(
                PathBuf::from("../agent-17/.praps/workspace"),
                "it has no `store` line".into(),
            ),
            Error::NotInWorkspace(id(b"old"), "agent-17".into()),
        ]
    }

    /// Every kind of refusal with an example of its message and its hint:
    /// the errors of the `workspace` crate, then the refusals `praps` makes
    /// itself.
    fn table() -> Vec<(String, String, Hint)> {
        let mut rows: Vec<(String, String, Hint)> = samples()
            .iter()
            .map(|error| {
                (
                    format!("workspace::Error::{}", variant(error)),
                    format!("praps undo: project: {}", said(error)),
                    for_error(error),
                )
            })
            .collect();
        let status = registry::find("status").unwrap();
        let describe = registry::find("describe").unwrap();
        let restore = registry::find("restore").unwrap();
        let words = ["Add", "the", "parser"].map(OsStr::new);
        let own: Vec<(&str, String, Hint)> = vec![
            ("usage error", status.usage(), usage(status)),
            (
                "usage error, no command named",
                help::USAGE.to_owned(),
                list(),
            ),
            (
                "a word that is not a command",
                "praps: `statsu` is not a command".to_owned(),
                list(),
            ),
            (
                "an argument that cannot be an operation's id",
                restore.usage(),
                operation_ids(),
            ),
            ("text with no UTF-8 form", describe.usage(), unicode_text()),
            (
                "text given as several words",
                describe.usage(),
                quoted(&words, "to give those words as one text").unwrap(),
            ),
            (
                "text given as several words that cannot be shown",
                describe.usage(),
                quote_the_text(),
            ),
            (
                "text given as two words, the second no directory",
                "praps describe: parser: no such directory".to_owned(),
                quoted(&[words[0], words[2]], "if both words are the text").unwrap(),
            ),
            (
                "a directory that is not there",
                "praps status: nowhere: no such directory".to_owned(),
                missing_directory(),
            ),
            (
                "a file given as the directory",
                "praps status: notes.txt: not a directory".to_owned(),
                missing_directory(),
            ),
            (
                "no repository there or above",
                "praps status: /home/me/work: no Periapsis repository here or in any directory \
                 above"
                    .to_owned(),
                no_repository(),
            ),
            (
                "a `.praps/store` that cannot be listed",
                "praps status: work/.praps/store: Permission denied (os error 13)".to_owned(),
                unreadable_directory(),
            ),
            (
                "a directory above that cannot be searched",
                "praps status: /home/me/work: a directory above it could not be searched for a \
                 repository: Permission denied (os error 13)"
                    .to_owned(),
                unreadable_above(),
            ),
            (
                "a top that cannot be entered",
                "praps status: /home/me/work: Permission denied (os error 13)".to_owned(),
                cannot_enter(),
            ),
            (
                "a current directory that cannot be read",
                "praps status: the current directory cannot be read: No such file or directory \
                 (os error 2)"
                    .to_owned(),
                current_directory(),
            ),
            (
                "`init` inside a working copy",
                "praps init: sub: it lies inside the working copy at /home/me/work, and a \
                 repository is not made inside another"
                    .to_owned(),
                inside_a_working_copy(),
            ),
            (
                "`init` stopped by the filesystem",
                "praps init: project: Permission denied (os error 13)".to_owned(),
                init_again(),
            ),
            (
                "`verify` given a directory that is not there",
                "praps verify: nowhere: no such directory".to_owned(),
                store_or_working_copy(),
            ),
            (
                "`verify` on a directory that is no store",
                "praps verify: plain: plain is not a Periapsis store (it needs objects/ and ops/)"
                    .to_owned(),
                not_a_store(),
            ),
            (
                "`verify` on a working copy's store that is no store",
                "praps verify: work: work/.praps/store is not a Periapsis store (it needs \
                 objects/ and ops/)"
                    .to_owned(),
                unfinished_store(),
            ),
            (
                "`verify` stopped by the filesystem",
                "praps verify: work: Permission denied (os error 13)".to_owned(),
                unreadable_store(),
            ),
            (
                "`verify` finds problems",
                "3 problems".to_owned(),
                damaged_store(),
            ),
            (
                "lane: a directory whose last name is no plain text",
                "praps lane: /: a lane is named after its directory's last name".to_owned(),
                lane_name(),
            ),
            (
                "lane: a directory inside a working copy",
                "praps lane: ../repo/sub: it lies inside the working copy at ../repo".to_owned(),
                lane_outside(),
            ),
        ];
        rows.extend(
            own.into_iter()
                .map(|(kind, message, hint)| (kind.to_owned(), message, hint)),
        );
        rows
    }

    /// Each variant of `workspace::Error` has a hint (INT-0037 AC-4). The
    /// samples cover every variant: `variant` names each without a
    /// catch-all, and its names are counted here.
    #[test]
    fn every_error_kind_has_a_hint() {
        let samples = samples();
        let seen: BTreeSet<&str> = samples.iter().map(variant).collect();
        assert_eq!(seen, VARIANTS.into_iter().collect());
        assert_eq!(samples.len(), VARIANTS.len(), "one sample of each");
        for error in &samples {
            let hint = for_error(error).to_string();
            assert!(hint.starts_with("hint: "), "{}: {hint}", variant(error));
        }
    }

    /// Every hint is one line, and one of the three things a refusal may
    /// end with: a command to run, by a name the registry holds; what to
    /// put right outside `praps`, naming no command; or that this build has
    /// no remedy yet and nothing was lost. Run with `--nocapture`, this
    /// prints the table the review reads: every kind of refusal with an
    /// example of its message and its hint.
    #[test]
    fn hints_take_one_of_three_forms() {
        let table = table();
        let mut forms = [0; 3];
        println!("| Kind | Message | Hint |\n|---|---|---|");
        for (kind, message, hint) in &table {
            let line = hint.to_string();
            println!(
                "| {kind} | `{}` | `{}` |",
                message.replace('\n', " / ").replace('`', "'"),
                line.replace('`', "'")
            );
            assert!(line.starts_with("hint: "), "{kind}: {line}");
            assert!(!line.contains('\n'), "{kind}: {line}");
            assert!(!line.ends_with('.'), "{kind}: {line}");
            match hint {
                Hint::Run(command, to) => {
                    forms[0] += 1;
                    let mut words = command.split(' ');
                    assert_eq!(words.next(), Some("praps"), "{kind}: {command}");
                    let name = words.next().unwrap_or_default();
                    assert!(registry::find(name).is_some(), "{kind}: `{name}`");
                    assert!(!to.is_empty(), "{kind}");
                    assert!(line.starts_with(&format!("hint: run `{command}` ")));
                }
                Hint::Fix(what) => {
                    forms[1] += 1;
                    assert!(!what.contains("`praps "), "{kind}: {what}");
                }
                Hint::NoRemedy(_) => {
                    forms[2] += 1;
                    assert!(line.starts_with("hint: this build cannot "), "{kind}");
                    assert!(line.ends_with(" yet; nothing was lost"), "{kind}");
                }
            }
        }
        println!(
            "{} kinds: {} name a command, {} say what to put right, {} say there is no remedy yet",
            table.len(),
            forms[0],
            forms[1],
            forms[2]
        );
        assert!(forms.iter().all(|count| *count > 0), "{forms:?}");
        assert_eq!(table.len(), VARIANTS.len() + 24);
    }

    /// Every command a hint names, wherever in the line, is one the
    /// registry holds.
    #[test]
    fn hints_name_registry_commands() {
        let mut named = 0;
        for (kind, _, hint) in table() {
            let line = hint.to_string();
            let mut rest = line.as_str();
            while let Some(at) = rest.find("`praps ") {
                let after = &rest[at + "`praps ".len()..];
                let name: String = after.chars().take_while(char::is_ascii_lowercase).collect();
                assert!(
                    registry::find(&name).is_some(),
                    "{kind}: `{name}` in {line:?} is not a command"
                );
                named += 1;
                rest = after;
            }
        }
        assert!(named > 10, "{named}");
        assert_eq!(list().to_string(), help::LIST_HINT);
    }

    /// Words are shown as one quoted text only when each can be shown as it
    /// was typed and no shell would change it inside quotes.
    #[test]
    fn quoted_text_is_shown_only_when_it_is_plain() {
        fn words(words: &[&'static str]) -> Vec<&'static OsStr> {
            words.iter().map(|word| OsStr::new(*word)).collect()
        }
        assert_eq!(
            quoted(
                &words(&["Add", "the", "parser"]),
                "to give those words as one text"
            )
            .unwrap()
            .to_string(),
            "hint: run `praps describe \"Add the parser\"` to give those words as one text"
        );
        assert_eq!(
            quoted(&words(&["caf\u{e9}", "au", "lait"]), "x")
                .unwrap()
                .to_string(),
            "hint: run `praps describe \"caf\u{e9} au lait\"` x"
        );
        for odd in [
            "say \"hi\"",
            "back`tick",
            "back\\slash",
            "$HOME",
            "100%",
            "wow!",
            "tab\there",
            "",
        ] {
            assert_eq!(quoted(&words(&["Add", odd]), "x"), None, "{odd:?}");
        }
    }
}
