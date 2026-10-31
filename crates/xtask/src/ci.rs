//! `cargo ci`: the canonical suite runner. The same steps run on every
//! developer machine and in GitHub Actions.

use std::env;
use std::ffi::OsString;
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
    let failed = run_with(&STEPS, out, |step| execute(root, step));
    finish(&failed, out)
}

/// The child process a cargo step runs.
pub fn cargo_command(root: &Path, args: &[&str]) -> Command {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut command = Command::new(cargo);
    command
        .args(args)
        .current_dir(root)
        .env_remove(GOLDEN_CREATE);
    command
}

fn execute(root: &Path, step: &Step) -> Outcome {
    match step.action {
        Action::Cargo(args) => match cargo_command(root, args).status() {
            Ok(status) if status.success() => Outcome::Pass,
            Ok(status) => Outcome::Fail(format!("cargo {} exited with {status}", args[0])),
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
    use std::ffi::OsStr;
    use std::fs;
    use std::path::Path;

    use super::{Action, Outcome, STEPS, Step, cargo_command, finish, run_with};

    fn repo_file(path: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path);
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    #[test]
    fn ci_steps_are_ordered_and_locked() {
        let names: Vec<&str> = STEPS.iter().map(|step| step.name).collect();
        assert_eq!(names, ["fmt", "clippy", "test", "policy", "book"]);
        for name in ["clippy", "test"] {
            let step = STEPS.iter().find(|step| step.name == name).unwrap();
            let Action::Cargo(args) = step.action else {
                panic!("{name} runs cargo");
            };
            assert!(args.contains(&"--locked"), "{name} must be --locked");
            assert!(
                args.contains(&"--workspace"),
                "{name} must cover the workspace"
            );
        }
        let Action::Cargo(clippy) = STEPS[1].action else {
            panic!("clippy runs cargo");
        };
        assert!(
            clippy.ends_with(&["--", "-D", "warnings"]),
            "clippy denies warnings"
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
        let command = cargo_command(Path::new("."), args);
        let removed = command
            .get_envs()
            .any(|(key, value)| key == OsStr::new("PERI_GOLDEN_CREATE") && value.is_none());
        assert!(removed, "the test step must not inherit PERI_GOLDEN_CREATE");
    }

    #[test]
    fn workflow_matches_ci_contract() {
        let workflow = repo_file(".github/workflows/ci.yml");
        let lines: Vec<&str> = workflow.lines().map(str::trim).collect();

        let push = lines
            .iter()
            .position(|line| *line == "push:")
            .expect("a push trigger");
        let branches = lines[push + 1];
        assert!(
            branches.starts_with("branches:")
                && branches.contains("dev")
                && branches.contains("main"),
            "push must trigger on dev and main: {branches}"
        );
        assert!(
            lines.iter().any(|line| line.starts_with("pull_request")),
            "a pull_request trigger"
        );
        assert!(
            lines.iter().any(|line| line.starts_with("os:")
                && line.contains("ubuntu-latest")
                && line.contains("windows-latest")),
            "the matrix must hold ubuntu-latest and windows-latest"
        );
        assert!(
            lines.contains(&"- run: cargo ci"),
            "a step must run cargo ci"
        );

        let deny = lines
            .iter()
            .position(|line| *line == "deny:")
            .expect("a deny job");
        let deny_job = &lines[deny..];
        assert!(
            deny_job.contains(&"runs-on: ubuntu-latest"),
            "deny runs on ubuntu-latest"
        );
        assert!(
            deny_job.iter().any(|line| line.contains("cargo-deny")),
            "the deny job runs cargo-deny"
        );

        assert!(
            !workflow.contains("continue-on-error"),
            "no step may swallow failure"
        );
        assert!(!workflow.contains("|| true"), "no step may swallow failure");
    }
}
