//! The command registry: the one description of each `praps` command
//! (INT-0022 AC-11, INT-0037).
//!
//! Dispatch, the command list, each command's help and each usage line are
//! generated from these entries, so a command cannot be listed without being
//! runnable, or run without being described. `praps mcp` will read the same
//! table (INT-0022 AC-3).

/// What a command does, as dispatch names it. Each entry of [`COMMANDS`]
/// carries one, and running a command is one exhaustive `match` over them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Status,
    Describe,
    New,
    Log,
    Ops,
    Undo,
    Restore,
    Init,
    Verify,
    Help,
    Version,
}

/// The purpose a command is listed under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// What a day's work needs (INT-0037 AC-5: at most ten).
    Everyday,
    LookingBack,
    Care,
}

impl Group {
    /// Every group, in the order the list shows them.
    pub const ALL: [Group; 3] = [Group::Everyday, Group::LookingBack, Group::Care];

    /// The heading the list prints.
    pub fn heading(self) -> &'static str {
        match self {
            Group::Everyday => "Every day",
            Group::LookingBack => "Looking back",
            Group::Care => "Starting and care",
        }
    }
}

/// One argument a command takes, in order.
#[derive(Debug)]
pub struct Arg {
    /// Its name in the usage line.
    pub name: &'static str,
    pub required: bool,
    /// What it is, for `praps help`.
    pub about: &'static str,
}

/// One command.
#[derive(Debug)]
pub struct Command {
    pub action: Action,
    /// The word that runs it: one lower-case English word (INT-0037 AC-1).
    pub name: &'static str,
    pub group: Group,
    /// One line, for the list.
    pub summary: &'static str,
    /// What it does, for `praps help`.
    pub about: &'static str,
    /// Its arguments, in the order they are given.
    pub args: &'static [Arg],
    /// Its flags. An idempotent command declares none (INT-0022 AC-8), and
    /// so far no command declares any.
    pub flags: &'static [&'static str],
    /// Whether a repeat changes nothing and prints the same (INT-0022).
    pub idempotent: bool,
    /// Command lines that work, each with what it does.
    pub examples: &'static [(&'static str, &'static str)],
}

/// The working copy's directory, which every command that opens one takes
/// last.
const DIR: Arg = Arg {
    name: "DIR",
    required: false,
    about: "the working copy's directory; the current directory if left out",
};

/// Every command, in the order the list shows them within each group.
pub const COMMANDS: &[Command] = &[
    Command {
        action: Action::Status,
        name: "status",
        group: Group::Everyday,
        summary: "List what the working copy changes",
        about: "Records the working copy, then lists each path the current change adds (A), \
                modifies (M) or deletes (D), and each entry it left alone and why.",
        args: &[DIR],
        flags: &[],
        idempotent: true,
        examples: &[(
            "praps status",
            "what has changed since the last `praps new`",
        )],
    },
    Command {
        action: Action::Describe,
        name: "describe",
        group: Group::Everyday,
        summary: "Say what the current change is for",
        about: "Records the working copy, then sets the description of the current change \
                to TEXT, in place of any it had. TEXT is one argument, so text of more than \
                one word goes in quotes. `praps log` shows its first line.",
        args: &[
            Arg {
                name: "TEXT",
                required: true,
                about: "the description; in quotes if it is more than one word",
            },
            DIR,
        ],
        flags: &[],
        idempotent: false,
        examples: &[(
            "praps describe \"Add the parser\"",
            "say what the current change is for",
        )],
    },
    Command {
        action: Action::New,
        name: "new",
        group: Group::Everyday,
        summary: "Start the next change",
        about: "Records the working copy into the current change, then starts a new, empty \
                change on top of it. No file is written.",
        args: &[DIR],
        flags: &[],
        idempotent: false,
        examples: &[("praps new", "finish this change and start the next")],
    },
    Command {
        action: Action::Log,
        name: "log",
        group: Group::Everyday,
        summary: "List the changes that lead to the working copy, newest first",
        about: "Records the working copy, then lists the current change and each change \
                before it: the first digits of its change ID, how many paths it changes, \
                and the first line of its description.",
        args: &[DIR],
        flags: &[],
        idempotent: true,
        examples: &[("praps log", "the changes so far, the current one first")],
    },
    Command {
        action: Action::Undo,
        name: "undo",
        group: Group::Everyday,
        summary: "Take back the last operation",
        about: "Records the working copy, then returns it to the state before the last \
                operation. Nothing is lost: a second undo brings that operation back.",
        args: &[DIR],
        flags: &[],
        idempotent: false,
        examples: &[("praps undo", "take back what the last command did")],
    },
    Command {
        action: Action::Ops,
        name: "ops",
        group: Group::LookingBack,
        summary: "List every operation, newest first",
        about: "Records the working copy, then lists each operation: the first digits of \
                its id, the change it left as the working copy, and how many paths that \
                change changes. An operation is one thing a command did, and `praps \
                restore` takes these ids.",
        args: &[DIR],
        flags: &[],
        idempotent: true,
        examples: &[(
            "praps ops",
            "every operation, with the ids `praps restore` takes",
        )],
    },
    Command {
        action: Action::Restore,
        name: "restore",
        group: Group::LookingBack,
        summary: "Return to the state an operation left",
        about: "Records the working copy, then returns it to the state the operation OP \
                left. The operations since stay in the list, so this too can be taken back.",
        args: &[
            Arg {
                name: "OP",
                required: true,
                about: "an operation's id, or enough of its first digits to name one",
            },
            DIR,
        ],
        flags: &[],
        idempotent: false,
        examples: &[(
            "praps restore 3f9a2c",
            "go back to the state operation 3f9a2c… left",
        )],
    },
    Command {
        action: Action::Init,
        name: "init",
        group: Group::Care,
        summary: "Make a directory a repository",
        about: "Makes DIR a repository with an empty first change, creating the directory \
                if it is not there. On a directory that already is one it writes nothing.",
        args: &[Arg {
            name: "DIR",
            required: false,
            about: "the directory to make a repository; the current directory if left out",
        }],
        flags: &[],
        idempotent: false,
        examples: &[
            ("praps init", "make the current directory a repository"),
            ("praps init project", "make `project` a repository"),
        ],
    },
    Command {
        action: Action::Verify,
        name: "verify",
        group: Group::Care,
        summary: "Check that everything stored is intact",
        about: "Reads every stored file and checks it against its name and what it refers \
                to. It prints each problem it finds and changes nothing.",
        args: &[Arg {
            name: "DIR",
            required: false,
            about: "a working copy, or a store's own directory; the current directory if \
                    left out",
        }],
        flags: &[],
        idempotent: true,
        examples: &[("praps verify", "check this repository's store")],
    },
    Command {
        action: Action::Help,
        name: "help",
        group: Group::Care,
        summary: "Explain the commands, or one command",
        about: "With no word, lists every command. With a command's name, explains that \
                command: its arguments, what it does, and an example.",
        args: &[Arg {
            name: "WORD",
            required: false,
            about: "the name of a command",
        }],
        flags: &[],
        idempotent: true,
        examples: &[("praps help status", "what `status` does and takes")],
    },
    Command {
        action: Action::Version,
        name: "version",
        group: Group::Care,
        summary: "Name this build",
        about: "Prints `praps` and the version of this build.",
        args: &[],
        flags: &[],
        idempotent: true,
        examples: &[("praps version", "which build this is")],
    },
];

/// The command `word` names, if the registry holds it.
pub fn find(word: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|command| command.name == word)
}

impl Command {
    /// The usage line: `usage: praps NAME [FLAG] ARG [OPTIONAL]`.
    pub fn usage(&self) -> String {
        let mut line = format!("usage: praps {}", self.name);
        for flag in self.flags {
            line.push_str(&format!(" [{flag}]"));
        }
        for arg in self.args {
            if arg.required {
                line.push_str(&format!(" {}", arg.name));
            } else {
                line.push_str(&format!(" [{}]", arg.name));
            }
        }
        line
    }

    /// How few and how many arguments it takes.
    pub fn arity(&self) -> (usize, usize) {
        let required = self.args.iter().filter(|arg| arg.required).count();
        (required, self.args.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// INT-0037 AC-1: every name is one plain lower-case word, and no two
    /// commands share one.
    #[test]
    fn registry_names_are_plain_words() {
        for command in COMMANDS {
            assert!(
                !command.name.is_empty() && command.name.bytes().all(|b| b.is_ascii_lowercase()),
                "{}",
                command.name
            );
            assert_eq!(
                COMMANDS.iter().filter(|c| c.name == command.name).count(),
                1,
                "{} is listed twice",
                command.name
            );
            assert_eq!(
                COMMANDS
                    .iter()
                    .filter(|c| c.action == command.action)
                    .count(),
                1,
                "{:?} has two names",
                command.action
            );
        }
    }

    /// INT-0037 AC-5.
    #[test]
    fn everyday_group_is_at_most_ten() {
        let everyday = COMMANDS
            .iter()
            .filter(|command| command.group == Group::Everyday)
            .count();
        assert!((1..=10).contains(&everyday), "{everyday}");
    }

    /// INT-0022 AC-8: no idempotent command declares a flag.
    #[test]
    fn idempotent_commands_declare_no_flag() {
        let idempotent: Vec<&Command> = COMMANDS.iter().filter(|c| c.idempotent).collect();
        assert!(!idempotent.is_empty(), "no command carries the mark");
        for command in idempotent {
            assert!(command.flags.is_empty(), "{}", command.name);
        }
    }

    /// Every command has what the list and its help print.
    #[test]
    fn every_command_is_documented() {
        for command in COMMANDS {
            let name = command.name;
            assert!(!command.summary.is_empty(), "{name}");
            assert!(!command.summary.contains('\n'), "{name}: one line");
            assert!(!command.summary.ends_with('.'), "{name}: no full stop");
            assert!(command.about.ends_with('.'), "{name}");
            assert!(
                command.usage().starts_with(&format!("usage: praps {name}")),
                "{name}"
            );
            assert!(!command.examples.is_empty(), "{name}");
            for (line, what) in command.examples {
                assert!(
                    *line == format!("praps {name}") || line.starts_with(&format!("praps {name} ")),
                    "{name}: {line}"
                );
                assert!(!what.is_empty(), "{name}");
            }
            for arg in command.args {
                assert!(
                    arg.name.bytes().all(|b| b.is_ascii_uppercase()),
                    "{name}: {}",
                    arg.name
                );
                assert!(!arg.about.is_empty(), "{name}: {}", arg.name);
            }
            // A required argument never follows an optional one.
            let (required, _) = command.arity();
            assert!(
                command.args.iter().take(required).all(|arg| arg.required),
                "{name}"
            );
        }
    }

    /// INT-0037 AC-1: no command is named by a word the chapter refuses.
    #[test]
    fn registry_refuses_other_systems_words() {
        for refused in [
            "fsck",
            "rebase",
            "checkout",
            "stash",
            "reflog",
            "cherry-pick",
        ] {
            assert!(find(refused).is_none(), "`{refused}` is a command");
        }
        assert!(find("verify").is_some());
    }

    #[test]
    fn usage_lines_name_each_argument() {
        assert_eq!(find("status").unwrap().usage(), "usage: praps status [DIR]");
        assert_eq!(
            find("restore").unwrap().usage(),
            "usage: praps restore OP [DIR]"
        );
        assert_eq!(find("version").unwrap().usage(), "usage: praps version");
        assert_eq!(
            find("describe").unwrap().usage(),
            "usage: praps describe TEXT [DIR]"
        );
        assert_eq!(find("describe").unwrap().arity(), (1, 2));
        assert_eq!(find("restore").unwrap().arity(), (1, 2));
        assert!(find("statsu").is_none());
    }
}
