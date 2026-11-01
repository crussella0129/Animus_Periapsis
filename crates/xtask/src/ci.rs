//! `cargo ci`: the canonical suite runner. The same steps run on every
//! developer machine and in GitHub Actions.

use std::collections::BTreeSet;
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::policy;

/// What a step does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Runs `cargo` with these arguments at the workspace root.
    Cargo(&'static [&'static str]),
    /// Runs the policy rules in-process.
    Policy,
    /// Checks the Book in-process.
    Book,
}

/// One named step of the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub name: &'static str,
    pub action: Action,
}

/// The suite, in order.
pub const STEPS: [Step; 5] = [
    Step {
        name: "fmt",
        action: Action::Cargo(&["fmt", "--all", "--check"]),
    },
    Step {
        name: "clippy",
        action: Action::Cargo(&[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ]),
    },
    Step {
        name: "test",
        action: Action::Cargo(&["test", "--workspace", "--locked"]),
    },
    Step {
        name: "policy",
        action: Action::Policy,
    },
    Step {
        name: "book",
        action: Action::Book,
    },
];

/// Golden-file create mode (see `crates/codec/tests/golden.rs`). No step
/// needs it, and the test step must never see it, so a variable left set in
/// the shell cannot create golden files.
const GOLDEN_CREATE: &str = "PERI_GOLDEN_CREATE";

/// proptest reads `PROPTEST_*` variables, and `PROPTEST_CASES` replaces even an
/// explicit case count, so no step inherits them: the property tests run the
/// counts their sources and the test records state.
const PROPTEST_PREFIX: &str = "PROPTEST_";

/// Clippy reads its configuration from this directory instead of the
/// workspace's `clippy.toml`, which would lift the ban on randomly seeded hash
/// collections (INT-0005 AC-4), so no step inherits it.
const CLIPPY_CONF_DIR: &str = "CLIPPY_CONF_DIR";

/// How a step ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail(String),
}

/// Runs every step, even after a failure, printing one line per step, and
/// returns the names of the failed steps.
pub fn run_with(
    steps: &[Step],
    out: &mut dyn Write,
    mut execute: impl FnMut(&Step) -> Outcome,
) -> Vec<&'static str> {
    let mut failed = Vec::new();
    for step in steps {
        // Child processes write to the same terminal; keep the lines in order.
        out.flush().ok();
        match execute(step) {
            Outcome::Pass => {
                writeln!(out, "PASS {}", step.name).ok();
            }
            Outcome::Fail(reason) => {
                writeln!(out, "FAIL {}: {reason}", step.name).ok();
                failed.push(step.name);
            }
        }
    }
    failed
}

/// Prints the summary line and returns the exit code: 0 when every step
/// passed, 1 otherwise.
pub fn finish(failed: &[&str], out: &mut dyn Write) -> u8 {
    if failed.is_empty() {
        writeln!(out, "ci: all {} steps passed", STEPS.len()).ok();
        0
    } else {
        writeln!(out, "ci: failed: {}", failed.join(", ")).ok();
        1
    }
}

/// Runs the suite at `root` and returns the exit code.
pub fn run(root: &Path, out: &mut dyn Write) -> u8 {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let failed = run_with(&STEPS, out, |step| {
        execute_with(root, step, |args| cargo_command(&cargo, root, args))
    });
    finish(&failed, out)
}

/// The child process a cargo step runs, with `program` as cargo.
pub fn cargo_command(program: &OsStr, root: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.args(args).current_dir(root);
    for name in scrubbed(env::vars_os().map(|(name, _)| name)) {
        command.env_remove(name);
    }
    command
}

/// The variables a cargo step does not inherit, given the names set where
/// `cargo ci` runs: golden-file create mode, `PROPTEST_CASES` and
/// `CLIPPY_CONF_DIR` always, and every other proptest variable that is set.
fn scrubbed(set: impl IntoIterator<Item = OsString>) -> BTreeSet<OsString> {
    let mut names =
        BTreeSet::from([GOLDEN_CREATE, "PROPTEST_CASES", CLIPPY_CONF_DIR].map(OsString::from));
    names.extend(set.into_iter().filter(|name| {
        name.to_str()
            .is_some_and(|name| name.to_ascii_uppercase().starts_with(PROPTEST_PREFIX))
    }));
    names
}

/// Runs one step at `root`. A cargo step runs the command `cargo` builds from
/// the step's arguments: `cargo_command` with the cargo that started
/// `cargo ci` (or `cargo` from the path), or whatever a test builds.
fn execute_with(root: &Path, step: &Step, cargo: impl Fn(&[&str]) -> Command) -> Outcome {
    match step.action {
        Action::Cargo(args) => match cargo(args).status() {
            Ok(status) if status.success() => Outcome::Pass,
            Ok(status) => Outcome::Fail(format!("cargo {} exited with {status}", args.join(" "))),
            Err(error) => Outcome::Fail(format!("could not run cargo: {error}")),
        },
        Action::Policy => {
            let problems: usize = policy::check(root)
                .iter()
                .map(|result| result.problems.len())
                .sum();
            match problems {
                0 => Outcome::Pass,
                n => Outcome::Fail(format!("{n} policy problems; run `cargo policy`")),
            }
        }
        Action::Book => match book::check(&root.join("docs")) {
            Ok(report) if report.violations.is_empty() => Outcome::Pass,
            Ok(report) => Outcome::Fail(format!(
                "{} Book violations; run `cargo book`",
                report.violations.len()
            )),
            Err(error) => Outcome::Fail(error.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::{
        Action, Outcome, STEPS, Step, cargo_command, execute_with, finish, run_with, scrubbed,
    };
    use crate::workflow;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn repo_file(path: &str) -> String {
        let path = repo_root().join(path);
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// `text` with `from` replaced by `to`; `from` must occur, so a mutant
    /// cannot silently equal the original.
    fn replace(text: &str, from: &str, to: &str) -> String {
        assert!(text.contains(from), "{from:?} is not in the workflow");
        text.replace(from, to)
    }

    /// The exact commands of the T-007 clause, in order.
    #[test]
    fn ci_steps_are_ordered_and_locked() {
        let expected = [
            Step {
                name: "fmt",
                action: Action::Cargo(&["fmt", "--all", "--check"]),
            },
            Step {
                name: "clippy",
                action: Action::Cargo(&[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--locked",
                    "--",
                    "-D",
                    "warnings",
                ]),
            },
            Step {
                name: "test",
                action: Action::Cargo(&["test", "--workspace", "--locked"]),
            },
            Step {
                name: "policy",
                action: Action::Policy,
            },
            Step {
                name: "book",
                action: Action::Book,
            },
        ];
        assert_eq!(STEPS, expected);
    }

    /// Each kind of step really runs: cargo as a child process, policy and
    /// the Book in-process.
    #[test]
    fn steps_execute_for_real() {
        let root = repo_root();
        let cargo = env!("CARGO");
        // The real command, with its output kept out of the test log.
        let quiet = |program: &'static str| {
            let root = root.clone();
            move |args: &[&str]| -> Command {
                let mut command = cargo_command(OsStr::new(program), &root, args);
                command.stdout(Stdio::null()).stderr(Stdio::null());
                command
            }
        };
        let step = |action| Step {
            name: "probe",
            action,
        };

        let version = step(Action::Cargo(&["--version"]));
        assert_eq!(execute_with(&root, &version, quiet(cargo)), Outcome::Pass);

        const UNKNOWN: &[&str] = &["no-such-subcommand", "--locked"];
        let status = cargo_command(OsStr::new(cargo), &root, UNKNOWN)
            .output()
            .expect("cargo runs")
            .status;
        assert!(!status.success(), "cargo must reject {UNKNOWN:?}");
        assert_eq!(
            execute_with(&root, &step(Action::Cargo(UNKNOWN)), quiet(cargo)),
            Outcome::Fail(format!(
                "cargo no-such-subcommand --locked exited with {status}"
            ))
        );

        let missing = execute_with(&root, &version, quiet("no-such-cargo-program"));
        let Outcome::Fail(reason) = missing else {
            panic!("a program that does not exist must fail the step");
        };
        assert!(reason.starts_with("could not run cargo: "), "{reason}");

        for action in [Action::Policy, Action::Book] {
            assert_eq!(
                execute_with(&root, &step(action), quiet(cargo)),
                Outcome::Pass
            );
            let nowhere = root.join("no-such-workspace");
            assert!(
                matches!(
                    execute_with(&nowhere, &step(action), quiet(cargo)),
                    Outcome::Fail(_)
                ),
                "{action:?} must fail without a workspace"
            );
        }
    }

    /// A Book with violations fails the `book` step with their count. In a
    /// declared Book each untyped criterion is one `untyped` violation
    /// (T-006).
    #[test]
    fn book_step_fails_on_a_violation() {
        let root = std::env::temp_dir().join(format!("xtask-ci-{}-book", std::process::id()));
        let chapter = root.join("docs/intents/INT-0001-x.md");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/intents")).unwrap();
        fs::write(
            root.join("docs/.sprint-loop-book"),
            "schema-version: 2\ntyped-acceptance: 1\n",
        )
        .unwrap();
        let book = Step {
            name: "book",
            action: Action::Book,
        };
        let mut outcomes = Vec::new();
        // The second chapter text has two violations in one chapter, so the
        // count is of violations, not chapters.
        for criteria in ["- Untyped claim.\n", "- Untyped claim.\n- Another.\n"] {
            fs::write(
                &chapter,
                format!("# INT-0001 — X\n\n## Acceptance criteria\n{criteria}"),
            )
            .unwrap();
            outcomes.push(execute_with(&root, &book, |_| {
                panic!("the Book step runs no cargo")
            }));
        }
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            outcomes,
            [
                Outcome::Fail("1 Book violations; run `cargo book`".to_owned()),
                Outcome::Fail("2 Book violations; run `cargo book`".to_owned()),
            ]
        );
    }

    #[test]
    fn ci_failures_do_not_stop_later_steps() {
        let steps = [
            Step {
                name: "one",
                action: Action::Policy,
            },
            Step {
                name: "two",
                action: Action::Cargo(&["no-such-tool"]),
            },
            Step {
                name: "three",
                action: Action::Book,
            },
            Step {
                name: "four",
                action: Action::Cargo(&["fmt"]),
            },
        ];
        let mut ran = Vec::new();
        let mut out = Vec::new();
        let failed = run_with(&steps, &mut out, |step| {
            ran.push(step.name);
            match step.name {
                "two" => Outcome::Fail("could not run cargo-no-such-tool".to_owned()),
                "four" => Outcome::Fail("cargo fmt exited with 1".to_owned()),
                _ => Outcome::Pass,
            }
        });
        assert_eq!(ran, ["one", "two", "three", "four"], "every step runs");
        assert_eq!(failed, ["two", "four"]);
        assert_eq!(finish(&failed, &mut out), 1, "a failure exits non-zero");
        let printed = String::from_utf8(out).unwrap();
        assert!(printed.contains("PASS one\n") && printed.contains("PASS three\n"));
        assert!(printed.contains("FAIL two: ") && printed.contains("FAIL four: "));
        assert!(printed.ends_with("ci: failed: two, four\n"), "{printed}");
        assert_eq!(finish(&[], &mut Vec::new()), 0);
    }

    #[test]
    fn test_step_removes_golden_create() {
        let Action::Cargo(args) = STEPS[2].action else {
            panic!("test runs cargo");
        };
        let command = cargo_command(OsStr::new("cargo"), Path::new("."), args);
        for name in ["PERI_GOLDEN_CREATE", "PROPTEST_CASES", "CLIPPY_CONF_DIR"] {
            let removed = command
                .get_envs()
                .any(|(key, value)| key == OsStr::new(name) && value.is_none());
            assert!(removed, "the test step must not inherit {name}");
        }
    }

    #[test]
    fn proptest_variables_are_never_inherited() {
        let set = [
            "PATH",
            "PROPTEST_MAX_SHRINK_ITERS",
            "proptest_rng_seed",
            "CARGO",
        ];
        let names: BTreeSet<OsString> = [
            "CLIPPY_CONF_DIR",
            "PERI_GOLDEN_CREATE",
            "PROPTEST_CASES",
            "PROPTEST_MAX_SHRINK_ITERS",
            "proptest_rng_seed",
        ]
        .map(OsString::from)
        .into();
        assert_eq!(scrubbed(set.map(OsString::from)), names);
    }

    #[test]
    fn workflow_matches_ci_contract() {
        let workflow = repo_file(".github/workflows/ci.yml").replace("\r\n", "\n");
        assert_eq!(workflow::problems(&workflow), Vec::<String>::new());

        // Each mutant breaks one point of the clause and must be caught.
        let without_ci = replace(&workflow, "      - run: cargo ci\n", "");
        let mutants = [
            replace(
                &workflow,
                "runs-on: ${{ matrix.os }}",
                "runs-on: ubuntu-latest",
            ),
            replace(
                &workflow,
                "branches: [dev, main]",
                "branches: [develop, main]",
            ),
            replace(&workflow, "pull_request:", "pull_request_target:"),
            format!("{}\n      - run: cargo ci\n", without_ci.trim_end()),
            format!(
                "{}\n  other:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo --version\n",
                replace(
                    &workflow,
                    "runs-on: ubuntu-latest",
                    "runs-on: windows-latest"
                )
                .trim_end()
            ),
            format!("{workflow}      - run: cargo deny check || true\n"),
            format!("{workflow}      - run: cargo deny check ||true\n"),
            format!("{workflow}      - run: cargo deny check || :\n"),
            format!("{workflow}      - run: cargo deny check || exit 0\n"),
            format!("{workflow}      - run: cargo deny check ||\n          true\n"),
            format!("{workflow}      - run: set +e\n"),
            replace(
                &workflow,
                "uses: EmbarkStudios/cargo-deny-action@v2",
                "uses: taiki-e/install-action@cargo-deny",
            ),
            replace(
                &workflow,
                "      - uses: EmbarkStudios/cargo-deny-action@v2\n",
                "      - uses: EmbarkStudios/cargo-deny-action@v2\n        with:\n          command: fetch\n",
            ),
            replace(
                &workflow,
                "      - uses: EmbarkStudios/cargo-deny-action@v2\n",
                "      - run: cargo deny fetch\n",
            ),
            replace(
                &workflow,
                "      - uses: EmbarkStudios/cargo-deny-action@v2\n",
                "      - run: cargo deny fetch && echo check\n",
            ),
            replace(&workflow, "  deny:\n", "  deny:\n    if: false\n"),
            replace(
                &workflow,
                "      - run: cargo ci\n",
                "      - run: cargo ci\n        if: false\n",
            ),
            replace(
                &workflow,
                "        os: [ubuntu-latest, windows-latest]\n",
                "        os: [ubuntu-latest, windows-latest]\n        exclude:\n          - os: windows-latest\n",
            ),
            replace(
                &workflow,
                "    runs-on: ${{ matrix.os }}\n",
                "    runs-on: ${{ matrix.os }}\n    continue-on-error: true\n",
            ),
        ];
        for mutant in mutants {
            assert!(
                !workflow::problems(&mutant).is_empty(),
                "this workflow must fail:\n{mutant}"
            );
        }
    }
}
