//! What `praps` says about itself: the command list, one command's help,
//! and the answer to a word that is not a command (INT-0037 AC-2). All of
//! it is rendered from the registry.

use std::fmt::Write;

use crate::registry::{COMMANDS, Command, Group};

/// The line every refusal about a word or its arguments ends with.
pub const LIST_HINT: &str = "hint: run `praps help` to list the commands";

/// The usage line when no command was named.
pub const USAGE: &str = "usage: praps COMMAND [ARGUMENT]...";

/// The command list: every command by purpose, the everyday group first,
/// each with its one-line summary.
pub fn list() -> String {
    let width = COMMANDS
        .iter()
        .map(|command| command.name.len())
        .max()
        .unwrap_or(0);
    let mut out =
        String::from("praps: version control for people and agents working at the same time\n");
    for group in Group::ALL {
        let _ = write!(out, "\n{}\n", group.heading());
        for command in COMMANDS.iter().filter(|command| command.group == group) {
            let _ = writeln!(out, "  {:<width$}  {}", command.name, command.summary);
        }
    }
    out.push_str(
        "\nRun `praps help WORD` for one command. A command that takes DIR acts on the\n\
         working copy there, and on the current directory if DIR is left out.\n",
    );
    out
}

/// One command's help: its usage line, summary, what it does, its
/// arguments, its examples, and whether it is idempotent.
pub fn explain(command: &Command) -> String {
    let mut out = format!(
        "{}\n\n{}\n\n{}\n",
        command.usage(),
        command.summary,
        wrap(command.about)
    );
    if !command.args.is_empty() {
        out.push_str("\nArguments:\n");
        let width = command
            .args
            .iter()
            .map(|arg| arg.name.len())
            .max()
            .unwrap_or(0);
        for arg in command.args {
            let _ = writeln!(out, "  {:<width$}  {}", arg.name, arg.about);
        }
    }
    out.push_str("\nExamples:\n");
    for (line, what) in command.examples {
        let _ = writeln!(out, "  {line}\n      {what}");
    }
    out.push('\n');
    out.push_str(if command.idempotent {
        "Idempotent: run again, it changes nothing and prints the same.\n"
    } else {
        "Not idempotent: each run may change the repository, or say something else.\n"
    });
    out
}

/// `text` broken into lines of at most 78 columns, at spaces.
fn wrap(text: &str) -> String {
    let mut out = String::new();
    let mut column = 0;
    for word in text.split_whitespace() {
        let len = word.chars().count();
        if column > 0 && column + 1 + len > 78 {
            out.push('\n');
            column = 0;
        }
        if column > 0 {
            out.push(' ');
            column += 1;
        }
        out.push_str(word);
        column += len;
    }
    out
}

/// Words of other version control systems, and what to use in `praps`. An
/// answer names only commands the registry holds.
pub const OTHER_WORDS: &[(&[&str], &str)] = &[
    (
        &["commit", "ci", "add", "stage", "save", "snapshot"],
        "there is nothing to add or commit: every command records the working copy first, \
         and `praps new` starts the next change",
    ),
    (
        &["checkout", "switch", "reset", "revert"],
        "`praps undo` takes back the last operation, and `praps restore` returns to an \
         earlier one",
    ),
    (
        &["reflog", "oplog", "history"],
        "`praps log` lists every operation",
    ),
    (
        &["stash"],
        "there is nothing to stash: the working copy is already a change, and `praps new` \
         starts another",
    ),
    (
        &["check"],
        "`praps fsck` checks that everything stored is intact",
    ),
    (
        &[
            "branch", "merge", "rebase", "pull", "push", "fetch", "clone", "sync", "diff", "show",
            "tag", "blame",
        ],
        "this build has no command for that yet",
    ),
];

/// What `praps` says to use for `word`, if it is another system's word.
pub fn other_word(word: &str) -> Option<&'static str> {
    OTHER_WORDS
        .iter()
        .find(|(words, _)| words.contains(&word))
        .map(|(_, answer)| *answer)
}

/// How many single-character insertions, deletions and substitutions turn
/// `a` into `b`.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, from) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, to) in b.iter().enumerate() {
            let substituted = diagonal + usize::from(from != to);
            diagonal = row[j + 1];
            row[j + 1] = substituted.min(row[j] + 1).min(diagonal + 1);
        }
    }
    row[b.len()]
}

/// Every command within two edits of `word`, nearest first and then by
/// name.
pub fn nearest(word: &str) -> Vec<&'static str> {
    let mut near: Vec<(usize, &'static str)> = COMMANDS
        .iter()
        .map(|command| (edit_distance(word, command.name), command.name))
        .filter(|(distance, _)| *distance <= 2)
        .collect();
    near.sort_unstable();
    near.into_iter().map(|(_, name)| name).collect()
}

/// The answer to a first word that is not a command: what it is not, the
/// commands near it, what to use if another system has the word, and where
/// the list is.
pub fn unknown(word: &str) -> String {
    let mut out = format!("praps: `{word}` is not a command\n");
    let near = nearest(word);
    if !near.is_empty() {
        let names: Vec<String> = near.iter().map(|name| format!("`{name}`")).collect();
        let _ = writeln!(out, "nearest: {}", names.join(", "));
    }
    if let Some(answer) = other_word(word) {
        let _ = writeln!(out, "`{word}`: {answer}");
    }
    out.push_str(LIST_HINT);
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    /// The list holds each command's name and summary once, under its
    /// group's heading, with the everyday group first.
    #[test]
    fn the_list_is_rendered_from_the_registry() {
        let list = list();
        let mut at = 0;
        for group in Group::ALL {
            let heading = format!("\n{}\n", group.heading());
            let found = list[at..].find(&heading).map(|i| at + i);
            at = found.unwrap_or_else(|| panic!("no heading {heading:?} after byte {at}"));
        }
        assert!(
            list.find("\nEvery day\n") < list.find("\nLooking back\n"),
            "{list}"
        );
        for command in COMMANDS {
            let lines: Vec<&str> = list
                .lines()
                .filter(|line| line.split_whitespace().next() == Some(command.name))
                .filter(|line| line.starts_with("  "))
                .collect();
            assert_eq!(lines.len(), 1, "{}: {lines:?}", command.name);
            assert!(lines[0].ends_with(command.summary), "{}", lines[0]);
            // Its line lies under its own group's heading.
            let line_at = list.find(lines[0]).unwrap();
            let heading_at = list[..line_at]
                .rfind("\n\n")
                .map(|i| &list[i + 2..line_at])
                .and_then(|block| block.lines().next())
                .unwrap();
            assert_eq!(heading_at, command.group.heading(), "{}", command.name);
        }
        // Nothing is listed that the registry does not hold.
        let listed = list.lines().filter(|line| line.starts_with("  ")).count();
        assert_eq!(listed, COMMANDS.len());
    }

    #[test]
    fn help_holds_every_part() {
        for command in COMMANDS {
            let help = explain(command);
            assert!(help.starts_with(&command.usage()), "{help}");
            assert!(help.contains(command.summary));
            for (line, what) in command.examples {
                assert!(help.contains(line) && help.contains(what));
            }
            assert_eq!(help.contains("\nArguments:\n"), !command.args.is_empty());
            for arg in command.args {
                assert!(help.contains(arg.about), "{}", arg.name);
            }
            assert_eq!(
                help.contains("\nIdempotent: "),
                command.idempotent,
                "{}",
                command.name
            );
            assert_eq!(help.contains("\nNot idempotent: "), !command.idempotent);
        }
    }

    #[test]
    fn edit_distance_matches_known_cases() {
        for (a, b, distance) in [
            ("status", "status", 0),
            ("", "", 0),
            ("", "new", 3),
            ("statu", "status", 1),
            ("statuss", "status", 1),
            ("stutus", "status", 1),
            ("statsu", "status", 2),
            ("sttaus", "status", 2),
            ("lg", "log", 1),
            ("kitten", "sitting", 3),
            ("commit", "undo", 6),
            ("abc", "xyz", 3),
        ] {
            assert_eq!(edit_distance(a, b), distance, "{a} -> {b}");
            assert_eq!(edit_distance(b, a), distance, "{b} -> {a}");
        }
    }

    /// Nearest first, then by name; nothing further than two edits.
    #[test]
    fn suggestions_are_nearest_first_then_by_name() {
        // One edit from `log`, two from `new`.
        assert_eq!(nearest("nog"), ["log", "new"]);
        assert_eq!(nearest("statsu"), ["status"]);
        assert_eq!(nearest("status"), ["status"]);
        assert!(nearest("frobnicate").is_empty());
        for word in ["nog", "lg", "inot", "udno", "x"] {
            let near = nearest(word);
            let distances: Vec<usize> = near.iter().map(|name| edit_distance(word, name)).collect();
            assert!(distances.iter().all(|d| *d <= 2), "{word}: {near:?}");
            let mut sorted: Vec<(usize, &str)> = distances
                .iter()
                .copied()
                .zip(near.iter().copied())
                .collect();
            sorted.sort_unstable();
            assert_eq!(
                sorted.iter().map(|(_, name)| *name).collect::<Vec<_>>(),
                near,
                "{word}"
            );
            // Every command within two edits is there.
            let all = COMMANDS
                .iter()
                .filter(|command| edit_distance(word, command.name) <= 2)
                .count();
            assert_eq!(near.len(), all, "{word}");
        }
    }

    /// Each answer names only commands the registry holds, and no word in
    /// the table is itself a command.
    #[test]
    fn other_systems_words_name_registry_commands() {
        for (words, answer) in OTHER_WORDS {
            for word in *words {
                assert!(registry::find(word).is_none(), "`{word}` is a command");
                assert_eq!(other_word(word), Some(*answer));
            }
            let mut rest = *answer;
            while let Some(at) = rest.find("`praps ") {
                let named = &rest[at + "`praps ".len()..];
                let end = named.find('`').expect("a closing backtick");
                let name = named[..end].split(' ').next().unwrap();
                assert!(
                    registry::find(name).is_some(),
                    "`{name}` in {answer:?} is not a command"
                );
                rest = &named[end..];
            }
        }
        assert_eq!(other_word("status"), None);
    }

    #[test]
    fn unknown_words_end_with_the_way_to_the_list() {
        let answer = unknown("statsu");
        assert!(answer.starts_with("praps: `statsu` is not a command\n"));
        assert!(answer.contains("nearest: `status`\n"));
        assert!(answer.ends_with(&format!("{LIST_HINT}\n")));
        let answer = unknown("commit");
        assert!(answer.contains("`commit`: there is nothing to add or commit"));
        assert!(!answer.contains("nearest:"));
        let answer = unknown("frobnicate");
        assert_eq!(
            answer,
            format!("praps: `frobnicate` is not a command\n{LIST_HINT}\n")
        );
    }
}
