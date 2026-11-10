#![forbid(unsafe_code)]
//! `praps`, the Periapsis command.
//!
//! `praps` alone lists the commands, and `praps help WORD` explains one.
//! Both are printed from the registry ([`registry`]), which is also the only
//! table dispatch reads (INT-0022 AC-11, INT-0037).
//!
//! Every command but `init`, `verify`, `help` and `version` opens the working
//! copy: it finishes an interrupted write (printing a `note:` on stderr for
//! anything it left), then snapshots it (INT-0014, INT-0009). `verify` only
//! reads (INT-0007 AC-4), and `init` on an existing repository writes
//! nothing. A command that cannot do what was asked says what was wrong and
//! ends with one `hint:` line that says what to do next ([`hint`]; INT-0037
//! AC-4). Exit codes follow the project's convention: 0 success (for
//! `verify`, clean), 1 a repository error (not a repository, several op heads,
//! corruption, nothing to undo, an unknown or ambiguous op, a hazard or an
//! entry in the way), 2 a usage error or a word that is not a command.
//!
//! No command takes a flag. `--help` and `-h` are spellings of `help`, and
//! `--version` and `-V` of `version`; any other argument that begins with
//! `-` is a usage error.

mod commands;
mod help;
mod hint;
mod place;
mod registry;

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::process::ExitCode;

use registry::{Action, Command};

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let (stdout, stderr) = (io::stdout(), io::stderr());
    ExitCode::from(cli(&args, &mut stdout.lock(), &mut stderr.lock()))
}

/// What a command line asks for. Parsing touches no file and runs nothing.
#[derive(Debug)]
enum Invocation<'a> {
    /// The command list.
    List,
    /// One command's help.
    Explain(&'static Command),
    /// The build's name.
    Version,
    /// A command, with as many arguments as it takes.
    Run(&'static Command, Vec<&'a OsStr>),
    /// Arguments that fit no command line: a flag, or the wrong number of
    /// arguments for the command named, if one was.
    Usage(Option<&'static Command>),
    /// More words than `describe` takes: most likely one text, given
    /// without its quotes.
    Unquoted(&'static Command, Vec<&'a OsStr>),
    /// A first word that is not a command.
    Unknown(String),
}

/// Reads the command line `args` (without the program name). A word is a
/// command exactly when the registry holds it: there is no other table.
fn parse(args: &[OsString]) -> Invocation<'_> {
    let dashed = |arg: &OsString| arg.to_string_lossy().starts_with('-');
    let words: Vec<&OsStr> = args
        .iter()
        .filter(|arg| !dashed(arg))
        .map(OsString::as_os_str)
        .collect();
    // The first word, as text; a word that is not text is no command.
    let first = words
        .first()
        .map(|word| word.to_string_lossy().into_owned());
    let command = first.as_deref().and_then(registry::find);

    let flags: Vec<&OsString> = args.iter().filter(|arg| dashed(arg)).collect();
    let spelled =
        |long: &str, short: &str| flags.iter().any(|flag| *flag == long || *flag == short);
    let (asks_help, asks_version) = (spelled("--help", "-h"), spelled("--version", "-V"));
    let other_flag = flags.iter().any(|flag| {
        !["--help", "-h", "--version", "-V"]
            .iter()
            .any(|known| *flag == known)
    });
    if other_flag {
        return Invocation::Usage(command);
    }
    if asks_help {
        return match (first, command) {
            (None, _) => Invocation::List,
            (Some(_), Some(command)) => Invocation::Explain(command),
            (Some(word), None) => Invocation::Unknown(word),
        };
    }
    if asks_version {
        return Invocation::Version;
    }

    let Some(first) = first else {
        return Invocation::List;
    };
    let Some(command) = command else {
        return Invocation::Unknown(first);
    };
    let rest = &words[1..];
    let (least, most) = command.arity();
    if rest.len() > most && command.action == Action::Describe {
        return Invocation::Unquoted(command, rest.to_vec());
    }
    if rest.len() < least || rest.len() > most {
        return Invocation::Usage(Some(command));
    }
    match command.action {
        Action::Help => match rest.first() {
            None => Invocation::List,
            Some(word) => {
                let word = word.to_string_lossy();
                match registry::find(&word) {
                    Some(command) => Invocation::Explain(command),
                    None => Invocation::Unknown(word.into_owned()),
                }
            }
        },
        Action::Version => Invocation::Version,
        _ => Invocation::Run(command, rest.to_vec()),
    }
}

/// Runs the command line `args` (without the program name), writing the
/// report to `out` and errors to `err`; returns the exit code.
fn cli(args: &[OsString], out: &mut impl Write, err: &mut impl Write) -> u8 {
    match parse(args) {
        Invocation::List => {
            let _ = out.write_all(help::list().as_bytes());
            0
        }
        Invocation::Explain(command) => {
            let _ = out.write_all(help::explain(command).as_bytes());
            0
        }
        Invocation::Version => {
            let _ = writeln!(out, "praps {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Invocation::Run(command, args) => commands::run(command, &args, out, err),
        Invocation::Usage(Some(command)) => {
            commands::usage_error(command, &hint::usage(command), err)
        }
        Invocation::Unquoted(command, words) => {
            let hint = hint::quoted(&words, "to give those words as one text")
                .unwrap_or_else(hint::quote_the_text);
            commands::usage_error(command, &hint, err)
        }
        Invocation::Usage(None) => {
            let _ = writeln!(err, "{}", help::USAGE);
            let _ = writeln!(err, "{}", hint::list());
            2
        }
        Invocation::Unknown(word) => {
            let _ = err.write_all(help::unknown(&word).as_bytes());
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::COMMANDS;

    fn args(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    /// Each command given one argument too many is a usage error that
    /// carries that command's own usage line (INT-0022 AC-11). Nothing is
    /// run: parsing touches no file.
    #[test]
    fn usage_lines_come_from_the_registry() {
        for command in COMMANDS {
            let (_, most) = command.arity();
            let mut words = vec![command.name];
            words.extend(std::iter::repeat_n("x", most + 1));
            let args = args(&words);
            match parse(&args) {
                Invocation::Usage(Some(found)) => {
                    assert_eq!(found.name, command.name);
                    assert_eq!(found.usage(), command.usage());
                }
                // Too many words for `describe` are taken as text without
                // its quotes, and answered with the same usage line.
                Invocation::Unquoted(found, words) => {
                    assert_eq!((found.name, command.name), ("describe", "describe"));
                    assert_eq!(words.len(), most + 1);
                }
                other => panic!("{}: {other:?}", command.name),
            }
            // Through `cli`, the first line on stderr is that usage line.
            let (mut out, mut err) = (Vec::new(), Vec::new());
            assert_eq!(cli(&args, &mut out, &mut err), 2);
            assert!(out.is_empty());
            let err = String::from_utf8(err).unwrap();
            assert_eq!(err.lines().next(), Some(command.usage().as_str()));
            assert!(err.lines().last().unwrap().starts_with("hint: "));
            // So is a flag the command does not take.
            assert!(matches!(
                parse(&self::args(&[command.name, "--force"])),
                Invocation::Usage(Some(found)) if found.name == command.name
            ));
        }
        assert!(matches!(
            parse(&args(&["--force"])),
            Invocation::Usage(None)
        ));
        // Too few arguments, too.
        assert!(matches!(
            parse(&args(&["restore"])),
            Invocation::Usage(Some(found)) if found.name == "restore"
        ));
    }

    /// A word is taken as a command exactly when the registry holds it
    /// (INT-0022 AC-11): over the registry's names, names this project has
    /// retired or refuses, and other systems' words.
    #[test]
    fn only_registry_words_dispatch() {
        let mut words: Vec<&str> = COMMANDS.iter().map(|command| command.name).collect();
        words.extend(RETIRED);
        for (others, _) in help::OTHER_WORDS {
            words.extend(*others);
        }
        words.extend(["", "Status", "STATUS", "status ", "st", "praps"]);
        for word in words {
            let known = registry::find(word).is_some();
            let alone = args(&[word]);
            let parsed = parse(&alone);
            assert_eq!(
                !matches!(parsed, Invocation::Unknown(_)),
                known,
                "`{word}`: {parsed:?}"
            );
            // With arguments that fit nothing, a command still answers as
            // itself and any other word as unknown.
            let crowded = args(&[word, "a", "b", "c", "d"]);
            let parsed = parse(&crowded);
            let answered = match parsed {
                Invocation::Usage(Some(found)) | Invocation::Unquoted(found, _) => {
                    found.name == word
                }
                _ => false,
            };
            assert_eq!(answered, known, "`{word}` with too many arguments");
        }
    }

    /// Words this project refuses or has retired as command names
    /// (INT-0037): none is in the registry, so none dispatches.
    const RETIRED: [&str; 6] = [
        "fsck",
        "rebase",
        "checkout",
        "stash",
        "reflog",
        "cherry-pick",
    ];

    #[test]
    fn help_and_version_have_two_spellings_each() {
        for line in [&["--help"][..], &["-h"], &["help"], &[]] {
            assert!(matches!(parse(&args(line)), Invocation::List), "{line:?}");
        }
        for line in [
            &["status", "--help"][..],
            &["--help", "status"],
            &["-h", "status"],
            &["help", "status"],
        ] {
            assert!(
                matches!(parse(&args(line)), Invocation::Explain(found) if found.name == "status"),
                "{line:?}"
            );
        }
        for line in [&["--version"][..], &["-V"], &["version"]] {
            assert!(
                matches!(parse(&args(line)), Invocation::Version),
                "{line:?}"
            );
        }
        for line in [&["help", "statsu"][..], &["statsu", "--help"], &["statsu"]] {
            assert!(
                matches!(parse(&args(line)), Invocation::Unknown(word) if word == "statsu"),
                "{line:?}"
            );
        }
        assert!(matches!(
            parse(&args(&["version", "x"])),
            Invocation::Usage(Some(found)) if found.name == "version"
        ));
        // A command line that fits runs, with its arguments in order.
        match parse(&args(&["restore", "3f9a", "dir"])) {
            Invocation::Run(command, rest) => {
                assert_eq!(command.name, "restore");
                assert_eq!(rest, [OsStr::new("3f9a"), OsStr::new("dir")]);
            }
            other => panic!("{other:?}"),
        }
    }
}
