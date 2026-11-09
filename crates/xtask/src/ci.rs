//! `cargo ci`: the canonical suite runner. The same steps run on every
//! developer machine and in GitHub Actions.

use std::collections::BTreeSet;
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::{policy, reach, units};

/// What a step does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Runs `cargo` with these arguments at the workspace root.
    Cargo(&'static [&'static str]),
    /// Builds every test executable once, then runs the test targets as
    /// units, several at a time (`units`, Sprint 5 T-003).
    Test,
    /// Runs the policy rules in-process.
    Policy,
    /// Checks the Book in-process.
    Book,
    /// Checks in-process that the committed reach table is the one the
    /// test step's run records (INT-0005 AC-6). Writes nothing.
    Reach,
}

/// One named step of the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub name: &'static str,
    pub action: Action,
}

/// The suite, in order.
pub const STEPS: [Step; 6] = [
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
        action: Action::Test,
    },
    Step {
        name: "reach",
        action: Action::Reach,
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

/// The child mode of the four-process store test
/// (`crates/store/tests/real_writers.rs`). The test step must never see it,
/// so a variable left set in the shell cannot turn the no-op child test into
/// a writer (Sprint 4 T-007).
const WRITER_CHILD: &str = "PERI_WRITER_CHILD";

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
/// returns the names of the failed steps. A step may write its own output to
/// `out` before its line.
pub fn run_with(
    steps: &[Step],
    out: &mut dyn Write,
    mut execute: impl FnMut(&Step, &mut dyn Write) -> Outcome,
) -> Vec<&'static str> {
    let mut failed = Vec::new();
    for step in steps {
        // Child processes write to the same terminal; keep the lines in order.
        out.flush().ok();
        match execute(step, out) {
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
    let failed = run_with(&STEPS, out, |step, out| {
        prepare(root, step);
        execute_with(root, step, out, |args| cargo_command(&cargo, root, args))
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

/// Variables a cargo step never inherits, set or not: golden-file create
/// mode, the proptest case count, and those that change what Clippy denies,
/// how code is compiled or which toolchain compiles it.
const ALWAYS_SCRUBBED: [&str; 11] = [
    GOLDEN_CREATE,
    WRITER_CHILD,
    "PROPTEST_CASES",
    CLIPPY_CONF_DIR,
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "RUSTC",
    "RUSTDOC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTUP_TOOLCHAIN",
];

/// Prefixes of variables a cargo step does not inherit when they are set:
/// proptest settings, and the cargo settings for builds, targets (their
/// runners and linkers too), profiles, encoded flags and command aliases.
const SCRUBBED_PREFIXES: [&str; 6] = [
    PROPTEST_PREFIX,
    "CARGO_BUILD_",
    "CARGO_TARGET_",
    "CARGO_PROFILE_",
    "CARGO_ENCODED_",
    "CARGO_ALIAS_",
];

/// The variables a cargo step does not inherit, given the names set where
/// `cargo ci` runs: `ALWAYS_SCRUBBED`, and every set name, in any letter
/// case, that starts with one of `SCRUBBED_PREFIXES`.
fn scrubbed(set: impl IntoIterator<Item = OsString>) -> BTreeSet<OsString> {
    let mut names = BTreeSet::from(ALWAYS_SCRUBBED.map(OsString::from));
    names.extend(set.into_iter().filter(|name| {
        name.to_str().is_some_and(|name| {
            let name = name.to_ascii_uppercase();
            SCRUBBED_PREFIXES
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
    }));
    names
}

/// What `cargo ci` does before a step: before `test`, it clears the recorded
/// reach, so a scenario that no longer records cannot leave a file behind
/// for the `reach` step (INT-0005 AC-6). Kept apart from `execute_with`, so
/// tests that run a step's command at the real root clear nothing.
fn prepare(root: &Path, step: &Step) {
    if step.action == Action::Test {
        reach::clear(root);
    }
}

/// Runs one step at `root`. A cargo step runs the command `cargo` builds from
/// the step's arguments: `cargo_command` with the cargo that started
/// `cargo ci` (or `cargo` from the path), or whatever a test builds. The test
/// step builds all its commands with `cargo` too, and writes their captured
/// output to `out`; other cargo steps print straight to the terminal.
fn execute_with(
    root: &Path,
    step: &Step,
    out: &mut dyn Write,
    cargo: impl Fn(&[&str]) -> Command + Sync,
) -> Outcome {
    match step.action {
        Action::Test => {
            let problems = units::execute(
                &units::discover(root),
                units::slots(),
                out,
                &cargo,
                &units::run_command,
            );
            if problems.is_empty() {
                Outcome::Pass
            } else {
                Outcome::Fail(problems.join("; "))
            }
        }
        Action::Cargo(args) => match cargo(args).status() {
            Ok(status) if status.success() => Outcome::Pass,
            Ok(status) => Outcome::Fail(format!("cargo {} exited with {status}", args.join(" "))),
            Err(error) => Outcome::Fail(format!("could not run cargo: {error}")),
        },
        Action::Reach => match reach::check(root) {
            Ok(()) => Outcome::Pass,
            Err(why) => Outcome::Fail(why),
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
    use std::sync::Mutex;

    use super::{
        Action, Outcome, STEPS, Step, cargo_command, execute_with, finish, prepare, run, run_with,
        scrubbed,
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

    /// The clean workspace's one crate. Its one test fails if a variable that
    /// no step may inherit reaches the `test` step.
    const PROBE: &str = r#"#![forbid(unsafe_code)]

#[test]
fn inherits_no_stripped_variable() {
    for name in [
        "PERI_GOLDEN_CREATE",
        "PERI_WRITER_CHILD",
        "PROPTEST_CASES",
        "PROPTEST_RNG_SEED",
        "CLIPPY_CONF_DIR",
    ] {
        assert_eq!(std::env::var_os(name), None, "{name}");
    }
}
"#;

    /// A workspace at `root` that every step passes: one crate, `PROBE`, with
    /// no dependencies; this repository's toolchain pin, Clippy configuration,
    /// attributes, credits and format spec; and a Book with no chapters.
    fn clean_workspace(root: &Path) {
        let write = |path: &str, text: &str| {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        for path in [
            "rust-toolchain.toml",
            "clippy.toml",
            ".gitattributes",
            "docs/prior-art.md",
            "docs/spec/format-v1.md",
        ] {
            write(path, &repo_file(path));
        }
        write(
            "Cargo.toml",
            "[package]\nname = \"probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n\n\
             [lints]\nworkspace = true\n\n[workspace]\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n",
        );
        write(
            "Cargo.lock",
            "version = 4\n\n[[package]]\nname = \"probe\"\nversion = \"0.0.0\"\n",
        );
        write("src/lib.rs", PROBE);
        // An integration test too, so a `--test` unit also runs under the
        // scrubbed environment (Sprint 5 T-003).
        write(
            "tests/probe.rs",
            &PROBE.replace(
                "fn inherits_no_stripped_variable",
                "fn probe_inherits_no_stripped_variable",
            ),
        );
        write(
            "docs/.sprint-loop-book",
            "schema-version: 2\ntyped-acceptance: 1\n",
        );
        fs::create_dir_all(root.join("docs/intents")).unwrap();
    }

    /// Removes `dir` and everything in it. On Windows the emptied directory
    /// can stay in use for a moment after Git has run in it (the `golden`
    /// rule runs Git), so removal backs off for up to about 9.5 s. The first
    /// try normally succeeds.
    fn remove_all(dir: &Path) {
        for attempt in 0..100 {
            if fs::remove_dir_all(dir).is_ok() || !dir.exists() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10 * attempt.min(10)));
        }
    }

    /// The exact commands of the T-007 clause, in order. Each cargo step runs
    /// its command as written: `execute_with` hands on every argument, and
    /// `cargo_command` runs them all with the given cargo at the root.
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
                action: Action::Test,
            },
            Step {
                name: "reach",
                action: Action::Reach,
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

        let root = repo_root();
        let owned = |args: &[&str]| args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
        for step in expected {
            let Action::Cargo(args) = step.action else {
                continue;
            };
            let command = cargo_command(OsStr::new("probe-cargo"), &root, args);
            assert_eq!(
                (
                    command.get_program(),
                    command.get_args().collect::<Vec<_>>(),
                    command.get_current_dir()
                ),
                (
                    OsStr::new("probe-cargo"),
                    args.iter().map(OsStr::new).collect(),
                    Some(root.as_path())
                ),
                "{}",
                step.name
            );
            let handed = Mutex::new(Vec::new());
            let outcome = execute_with(&root, &step, &mut std::io::sink(), |passed| {
                handed.lock().unwrap().push(owned(passed));
                // A command that passes, so only the arguments are at stake.
                let mut version = Command::new(env!("CARGO"));
                version.arg("--version").stdout(Stdio::null());
                version
            });
            assert_eq!(
                (outcome, handed.into_inner().unwrap()),
                (Outcome::Pass, vec![owned(args)]),
                "{}",
                step.name
            );
        }
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
        assert_eq!(
            execute_with(&root, &version, &mut std::io::sink(), quiet(cargo)),
            Outcome::Pass
        );

        const UNKNOWN: &[&str] = &["no-such-subcommand", "--locked"];
        let status = cargo_command(OsStr::new(cargo), &root, UNKNOWN)
            .output()
            .expect("cargo runs")
            .status;
        assert!(!status.success(), "cargo must reject {UNKNOWN:?}");
        assert_eq!(
            execute_with(
                &root,
                &step(Action::Cargo(UNKNOWN)),
                &mut std::io::sink(),
                quiet(cargo)
            ),
            Outcome::Fail(format!(
                "cargo no-such-subcommand --locked exited with {status}"
            ))
        );

        // Exit status 1, as `cargo fmt --check` gives for unformatted code.
        const BAD_FLAG: &[&str] = &["--no-such-flag"];
        let status = cargo_command(OsStr::new(cargo), &root, BAD_FLAG)
            .output()
            .expect("cargo runs")
            .status;
        assert_eq!(status.code(), Some(1), "cargo must exit 1 on {BAD_FLAG:?}");
        assert_eq!(
            execute_with(
                &root,
                &step(Action::Cargo(BAD_FLAG)),
                &mut std::io::sink(),
                quiet(cargo)
            ),
            Outcome::Fail(format!("cargo --no-such-flag exited with {status}"))
        );

        let missing = execute_with(
            &root,
            &version,
            &mut std::io::sink(),
            quiet("no-such-cargo-program"),
        );
        let Outcome::Fail(reason) = missing else {
            panic!("a program that does not exist must fail the step");
        };
        assert!(reason.starts_with("could not run cargo: "), "{reason}");

        for action in [Action::Policy, Action::Book] {
            assert_eq!(
                execute_with(&root, &step(action), &mut std::io::sink(), quiet(cargo)),
                Outcome::Pass
            );
            let nowhere = root.join("no-such-workspace");
            assert!(
                matches!(
                    execute_with(&nowhere, &step(action), &mut std::io::sink(), quiet(cargo)),
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
            outcomes.push(execute_with(&root, &book, &mut std::io::sink(), |_| {
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

    /// A workspace with policy problems fails the `policy` step with their
    /// count, whichever rules find them: `.gitattributes` without
    /// `*.bin binary` gives one `golden` problem (T-005), and a missing
    /// `Cargo.lock` one `lockfile` problem more (T-001).
    #[test]
    fn policy_step_fails_on_a_problem() {
        let root = std::env::temp_dir().join(format!("xtask-ci-{}-policy", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        clean_workspace(&root);
        let policy = Step {
            name: "policy",
            action: Action::Policy,
        };
        let execute = || {
            execute_with(&root, &policy, &mut std::io::sink(), |_| {
                panic!("the policy step runs no cargo")
            })
        };
        let mut outcomes = vec![execute()];
        fs::write(root.join(".gitattributes"), "* text=auto eol=lf\n").unwrap();
        outcomes.push(execute());
        fs::remove_file(root.join("Cargo.lock")).unwrap();
        outcomes.push(execute());
        remove_all(&root);
        assert_eq!(
            outcomes,
            [
                Outcome::Pass,
                Outcome::Fail("1 policy problems; run `cargo policy`".to_owned()),
                Outcome::Fail("2 policy problems; run `cargo policy`".to_owned()),
            ]
        );
    }

    /// Two failures, one a tool that is not installed, among passing steps
    /// and before a last one: every step runs and prints its own line, in
    /// order, and the summary names both failures.
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
            Step {
                name: "five",
                action: Action::Policy,
            },
        ];
        let mut ran = Vec::new();
        let mut out = Vec::new();
        let failed = run_with(&steps, &mut out, |step, _| {
            ran.push(step.name);
            match step.name {
                "two" => Outcome::Fail("could not run cargo-no-such-tool".to_owned()),
                "four" => Outcome::Fail("cargo fmt exited with 1".to_owned()),
                _ => Outcome::Pass,
            }
        });
        assert_eq!(
            ran,
            ["one", "two", "three", "four", "five"],
            "every step runs"
        );
        assert_eq!(failed, ["two", "four"]);
        assert_eq!(finish(&failed, &mut out), 1, "a failure exits non-zero");
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "PASS one\nFAIL two: could not run cargo-no-such-tool\nPASS three\n\
             FAIL four: cargo fmt exited with 1\nPASS five\nci: failed: two, four\n"
        );
        let mut out = Vec::new();
        assert_eq!(finish(&[], &mut out), 0);
        assert_eq!(String::from_utf8(out).unwrap(), "ci: all 6 steps passed\n");
    }

    /// The test step clears the recorded reach before it runs, so a
    /// scenario that stopped recording cannot leave a stale file
    /// (Sprint 4 T-008).
    #[test]
    fn test_step_clears_the_recorded_reach() {
        let root = std::env::temp_dir().join(format!("xtask-reach-clear-{}", std::process::id()));
        let dir = crate::reach::reach_dir(&root);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("store.txt"),
            "scenario store
sprint 1
",
        )
        .unwrap();
        for step in STEPS.iter().filter(|step| step.name != "test") {
            prepare(&root, step);
            assert!(dir.exists(), "{} cleared the reach", step.name);
        }
        let test = STEPS.iter().find(|step| step.name == "test").unwrap();
        prepare(&root, test);
        assert!(!dir.exists());
        let _ = fs::remove_dir_all(&root);
    }

    /// No cargo step inherits a variable that creates golden files, cuts
    /// proptest's case counts, lifts Clippy's bans, or changes how code is
    /// compiled or which toolchain compiles it.
    #[test]
    fn test_step_removes_golden_create() {
        // Every command a step runs: each cargo step's own, and every command
        // the test step builds, its pre-build and each unit.
        let mut commands: Vec<(&str, Command)> = Vec::new();
        for step in STEPS {
            if let Action::Cargo(args) = step.action {
                commands.push((
                    step.name,
                    cargo_command(OsStr::new("cargo"), Path::new("."), args),
                ));
            }
        }
        let built = Mutex::new(Vec::new());
        let units = crate::units::discover(&repo_root());
        crate::units::execute(
            &units,
            4,
            &mut std::io::sink(),
            &|args| cargo_command(OsStr::new("cargo"), Path::new("."), args),
            &|command| {
                built.lock().unwrap().push(command);
                crate::units::Ran {
                    ok: true,
                    output: String::new(),
                }
            },
        );
        let built = built.into_inner().unwrap();
        assert_eq!(built.len(), units.len() + 1, "the pre-build and every unit");
        commands.extend(built.into_iter().map(|command| ("test", command)));
        for (step, command) in commands {
            for name in [
                "PERI_GOLDEN_CREATE",
                "PERI_WRITER_CHILD",
                "PROPTEST_CASES",
                "CLIPPY_CONF_DIR",
                "RUSTFLAGS",
                "RUSTDOCFLAGS",
                "RUSTC",
                "RUSTDOC",
                "RUSTC_WRAPPER",
                "RUSTC_WORKSPACE_WRAPPER",
                "RUSTUP_TOOLCHAIN",
            ] {
                let removed = command
                    .get_envs()
                    .any(|(key, value)| key == OsStr::new(name) && value.is_none());
                assert!(removed, "the {step} step must not inherit {name}");
            }
        }
    }

    /// `cargo ci` itself, run by a child copy of this test binary whose
    /// environment sets variables no step may inherit: golden create mode;
    /// proptest settings, one of which no list names, so only its prefix
    /// removes it; and a Clippy configuration that bans what `PROBE` calls.
    /// Any of them reaching a step fails it. The child runs outside the
    /// workspace, so only the root it is given can make the steps pass.
    #[test]
    fn hostile_variables_never_reach_a_step() {
        let base = std::env::temp_dir().join(format!("xtask-ci-{}-hostile", std::process::id()));
        let (root, config) = (base.join("workspace"), base.join("clippy"));
        let _ = fs::remove_dir_all(&base);
        clean_workspace(&root);
        fs::create_dir_all(&config).unwrap();
        fs::write(
            config.join("clippy.toml"),
            "disallowed-methods = [\"std::env::var_os\"]\n",
        )
        .unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "ci::tests::run_passes_a_clean_workspace",
                "--nocapture",
            ])
            .env("XTASK_CI_WORKSPACE", &root)
            .env("PERI_GOLDEN_CREATE", "1")
            .env("PROPTEST_CASES", "1")
            .env("PROPTEST_RNG_SEED", "7")
            .env("CLIPPY_CONF_DIR", &config)
            // Quiet output, from the caller's environment or a cargo config,
            // would print `PROBE`'s result as a dot.
            .env("CARGO_TERM_QUIET", "false")
            .current_dir(&base)
            .output()
            .unwrap();
        remove_all(&base);
        // `PROBE`'s own result shows that the child really ran `cargo ci`.
        let stdout = String::from_utf8_lossy(&child.stdout);
        assert!(
            child.status.success()
                && stdout.contains("test ci::tests::run_passes_a_clean_workspace ... ok")
                && stdout.contains("test inherits_no_stripped_variable ... ok")
                && stdout.contains("test probe_inherits_no_stripped_variable ... ok"),
            "{stdout}{}",
            String::from_utf8_lossy(&child.stderr)
        );
    }

    /// The child half of `hostile_variables_never_reach_a_step`: `cargo ci`
    /// on the clean workspace that `XTASK_CI_WORKSPACE` names runs every step,
    /// in order, and each passes. Run on its own, with no workspace named, it
    /// does nothing.
    #[test]
    fn run_passes_a_clean_workspace() {
        let Some(root) = std::env::var_os("XTASK_CI_WORKSPACE") else {
            return;
        };
        let mut out = Vec::new();
        let code = run(Path::new(&root), &mut out);
        let out = String::from_utf8(out).unwrap();
        // The test step's unit output, which now sits among the step lines,
        // reaches the parent through this test's own output.
        println!("{out}");
        let steps: Vec<&str> = out
            .lines()
            .filter(|line| line.starts_with("PASS ") || line.starts_with("FAIL "))
            .collect();
        assert_eq!(
            (code, steps, out.lines().last()),
            (
                0,
                vec![
                    "PASS fmt",
                    "PASS clippy",
                    "PASS test",
                    "PASS reach",
                    "PASS policy",
                    "PASS book"
                ],
                Some("ci: all 6 steps passed")
            )
        );
        for line in [
            "test inherits_no_stripped_variable ... ok",
            "test probe_inherits_no_stripped_variable ... ok",
            // The pre-build's own report found both executables, the probe
            // library's and `tests/probe.rs`'s, so the unrun check checked
            // something real (code review CR-04).
            "test: 2 test executables built, 2 run",
        ] {
            assert!(out.contains(line), "{line}");
        }
    }

    #[test]
    fn proptest_variables_are_never_inherited() {
        // Set where `cargo ci` runs: the ones a step must not inherit, in
        // any letter case, and the ones it needs.
        let set = [
            "PATH",
            "PROPTEST_MAX_SHRINK_ITERS",
            "proptest_rng_seed",
            "CARGO",
            "CARGO_HOME",
            "CARGO_NET_OFFLINE",
            "CARGO_BUILD_RUSTFLAGS",
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
            "CARGO_TARGET_DIR",
            "cargo_profile_test_debug_assertions",
            "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_ALIAS_CLIPPY",
        ];
        let names: BTreeSet<OsString> = [
            "CLIPPY_CONF_DIR",
            "PERI_GOLDEN_CREATE",
            "PERI_WRITER_CHILD",
            "PROPTEST_CASES",
            "RUSTC",
            "RUSTC_WORKSPACE_WRAPPER",
            "RUSTC_WRAPPER",
            "RUSTDOC",
            "RUSTDOCFLAGS",
            "RUSTFLAGS",
            "RUSTUP_TOOLCHAIN",
            "PROPTEST_MAX_SHRINK_ITERS",
            "proptest_rng_seed",
            "CARGO_BUILD_RUSTFLAGS",
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
            "CARGO_TARGET_DIR",
            "cargo_profile_test_debug_assertions",
            "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_ALIAS_CLIPPY",
        ]
        .map(OsString::from)
        .into();
        assert_eq!(scrubbed(set.map(OsString::from)), names);
    }

    /// `workflow` with `from` replaced by `to` inside job `job` only.
    fn in_job(workflow: &str, job: &str, from: &str, to: &str) -> String {
        let start = workflow
            .find(&format!("\n  {job}:\n"))
            .unwrap_or_else(|| panic!("no job {job}"));
        // The job ends at the next line at job depth: two spaces, then a
        // letter.
        let body = &workflow[start + 1..];
        let end = body
            .match_indices("\n  ")
            .find(|&(at, _)| {
                body[at + 3..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic())
            })
            .map_or(workflow.len(), |(at, _)| start + 1 + at);
        let section = &workflow[start..end];
        assert!(section.contains(from), "{from:?} is not in job {job}");
        format!(
            "{}{}{}",
            &workflow[..start],
            section.replace(from, to),
            &workflow[end..]
        )
    }

    #[test]
    fn workflow_matches_ci_contract() {
        let workflow = repo_file(".github/workflows/ci.yml").replace("\r\n", "\n");
        assert_eq!(workflow::problems(&workflow), Vec::<String>::new());

        const PUSH: &str = "    branches: [main]\n";
        const PR: &str = "  pull_request:\n";
        const INSTALL: &str = "      - uses: taiki-e/install-action@v2\n        with:\n          tool: cargo-deny@0.20.2\n";
        const DENY: &str = "      - run: cargo deny check\n";
        const MATRIX: &str = "        os: [ubuntu-latest, windows-latest]\n";
        const CLIPPY: &str =
            "      - run: cargo clippy --workspace --all-targets --locked -- -D warnings\n";

        // Each mutant breaks one point of the clause and must be caught.
        let without_ci = replace(&workflow, "      - run: cargo ci\n", "");
        let without_install = replace(&workflow, INSTALL, "");
        let mutants = [
            // Triggers (INT-0003 AC-2).
            replace(&workflow, PUSH, "    branches: [dev, main]\n"),
            replace(&workflow, PUSH, "    branches: ['**']\n"),
            replace(&workflow, PUSH, "    branches: [develop]\n"),
            replace(&workflow, PUSH, ""),
            replace(&workflow, &format!("  push:\n{PUSH}"), ""),
            replace(&workflow, "pull_request:", "pull_request_target:"),
            replace(&workflow, "  workflow_dispatch:\n", ""),
            // Concurrency.
            replace(
                &workflow,
                "concurrency:\n  group: ci-${{ github.ref }}-${{ github.event_name == 'pull_request' || github.sha }}\n  \
                 cancel-in-progress: ${{ github.event_name == 'pull_request' }}\n",
                "",
            ),
            replace(
                &workflow,
                "  group: ci-${{ github.ref }}-${{ github.event_name == 'pull_request' || github.sha }}\n",
                "  group: ci\n",
            ),
            replace(
                &workflow,
                "  group: ci-${{ github.ref }}-${{ github.event_name == 'pull_request' || github.sha }}\n",
                "  group: ci-${{ github.ref }}\n",
            ),
            replace(
                &workflow,
                "  cancel-in-progress: ${{ github.event_name == 'pull_request' }}\n",
                "  cancel-in-progress: true\n",
            ),
            // The suite on both platforms.
            in_job(
                &workflow,
                "ci",
                "runs-on: ${{ matrix.os }}",
                "runs-on: ubuntu-latest",
            ),
            format!("{}\n      - run: cargo ci\n", without_ci.trim_end()),
            replace(
                &workflow,
                "      - run: cargo ci\n",
                "      - run: cargo ci\n        if: false\n",
            ),
            in_job(
                &workflow,
                "ci",
                MATRIX,
                &format!("{MATRIX}        exclude:\n          - os: windows-latest\n"),
            ),
            in_job(
                &workflow,
                "ci",
                "    timeout-minutes: 30\n",
                "    timeout-minutes: 30\n    continue-on-error: true\n",
            ),
            // The lint checks on both platforms (INT-0003 AC-4).
            in_job(&workflow, "lint", MATRIX, "        os: [ubuntu-latest]\n"),
            in_job(
                &workflow,
                "lint",
                "runs-on: ${{ matrix.os }}",
                "runs-on: ubuntu-latest",
            ),
            in_job(
                &workflow,
                "lint",
                "    timeout-minutes: 30\n",
                "    timeout-minutes: 30\n    if: false\n",
            ),
            replace(&workflow, CLIPPY, ""),
            replace(&workflow, "      - run: cargo fmt --all --check\n", ""),
            replace(&workflow, " -- -D warnings\n", "\n"),
            in_job(
                &workflow,
                "lint",
                MATRIX,
                &format!("{MATRIX}        include:\n          - os: macos-latest\n"),
            ),
            replace(&workflow, CLIPPY, &format!("{}        if: false\n", CLIPPY)),
            // The deny job (INT-0003 AC-3).
            without_install.clone(),
            in_job(
                &without_install,
                "lint",
                CLIPPY,
                &format!("{CLIPPY}{INSTALL}"),
            ),
            replace(&without_install, DENY, &format!("{DENY}{INSTALL}")),
            replace(
                &without_install,
                DENY,
                "      - uses: EmbarkStudios/cargo-deny-action@v2\n",
            ),
            replace(&workflow, "tool: cargo-deny@0.20.2", "tool: cargo-nextest"),
            replace(&workflow, INSTALL, &format!("{INSTALL}        if: false\n")),
            replace(&workflow, DENY, &format!("{DENY}        if: false\n")),
            replace(&workflow, "  deny:\n", "  deny:\n    if: false\n"),
            format!(
                "{}\n  other:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo --version\n",
                in_job(
                    &workflow,
                    "deny",
                    "runs-on: ubuntu-latest",
                    "runs-on: windows-latest"
                )
                .trim_end()
            ),
            // Code review fixes (CR-01, CR-02, CR-03, CR-07). First, no lint
            // job of its own: fmt and Clippy moved into the suite's job.
            format!(
                "{}{}",
                &workflow[..workflow.find("\n  lint:\n").unwrap()],
                workflow[workflow.find("\n  ci:\n").unwrap()..].replace(
                    "      - run: cargo ci\n",
                    &format!(
                        "      - run: cargo fmt --all --check\n{CLIPPY}      - run: cargo ci\n"
                    )
                )
            ),
            replace(
                &workflow,
                "  group: ci-${{ github.ref }}-${{ github.event_name == 'pull_request' || github.sha }}\n",
                "  group: ci-${{ github.ref }}-${{ github.event_name == 'push' || github.sha }}\n",
            ),
            in_job(
                &workflow,
                "ci",
                "    timeout-minutes: 30\n",
                "    timeout-minutes: 30\n    concurrency:\n      group: x-${{ github.ref }}\n      cancel-in-progress: true\n",
            ),
            replace(
                &workflow,
                "      - run: cargo ci\n",
                "      - run: cargo ci\n        shell: cat {0}\n",
            ),
            in_job(
                &workflow,
                "ci",
                "    timeout-minutes: 30\n",
                "    timeout-minutes: 30\n    defaults:\n      run:\n        shell: cat {0}\n",
            ),
            replace(
                &workflow,
                CLIPPY,
                &format!("{CLIPPY}        working-directory: crates/hash\n"),
            ),
            replace(
                &workflow,
                "          tool: cargo-deny@0.20.2\n",
                "          tool: cargo-deny@0.20.2\n          checksum: false\n",
            ),
            replace(
                &workflow,
                "tool: cargo-deny@0.20.2",
                "tool: cargo-deny-unrelated",
            ),
            // Commands that outlive their own failure.
            format!("{workflow}      - run: cargo deny check || true\n"),
            format!("{workflow}      - run: cargo deny check ||true\n"),
            format!("{workflow}      - run: cargo deny check || :\n"),
            format!("{workflow}      - run: cargo deny check || exit 0\n"),
            format!("{workflow}      - run: cargo deny check ||\n          true\n"),
            format!("{workflow}      - run: set +e\n"),
        ];
        for mutant in mutants {
            assert!(
                !workflow::problems(&mutant).is_empty(),
                "this workflow must fail:\n{mutant}"
            );
        }

        // A deny check that narrows the checks, the workspace or the
        // configuration does not count as the deny job's check.
        for narrowed in [
            "cargo deny check sources",
            "cargo deny check advisories bans",
            "cargo deny --manifest-path other/Cargo.toml check",
            "cargo deny check --config other.toml",
            "cargo deny --offline check",
            "cargo deny check check",
            "cargo deny fetch",
            "cargo deny fetch && echo check",
        ] {
            let mutant = replace(&workflow, DENY, &format!("      - run: {narrowed}\n"));
            assert!(
                !workflow::problems(&mutant).is_empty(),
                "this deny step must not count:\n{narrowed}"
            );
        }
        // Settings that change where or how the check runs.
        for (from, to) in [
            (
                DENY,
                "      - run: cargo deny check\n        working-directory: crates/hash\n",
            ),
            (
                DENY,
                "      - run: cargo deny check\n        shell: cat {0}\n",
            ),
            (
                "    timeout-minutes: 10\n",
                "    timeout-minutes: 10\n    defaults:\n      run:\n        working-directory: crates/hash\n",
            ),
            (
                "jobs:\n",
                "defaults:\n  run:\n    working-directory: crates/hash\njobs:\n",
            ),
        ] {
            let mutant = replace(&workflow, from, to);
            assert!(
                !workflow::problems(&mutant).is_empty(),
                "this deny job must not count:\n{to}"
            );
        }
        // A trigger filter can keep CI from ever running.
        for (from, filter) in [
            (PUSH, "    paths-ignore: ['**']\n"),
            (PUSH, "    paths: ['nothing/**']\n"),
            (PUSH, "    branches-ignore: [main]\n"),
            (PR, "    types: [closed]\n"),
            (PR, "    paths-ignore: ['**']\n"),
            (PR, "    branches: [nothing]\n"),
            (PR, "    branches: [dev]\n"),
            (PR, "    branches-ignore: [main]\n"),
        ] {
            let mutant = replace(&workflow, from, &format!("{from}{filter}"));
            assert!(
                !workflow::problems(&mutant).is_empty(),
                "this trigger must fail:\n{from}{filter}"
            );
        }
        // A later `!` pattern excludes a branch the list names.
        for (from, to) in [
            (PUSH, "    branches: [main, '!main']\n".to_owned()),
            (PR, format!("{PR}    branches: [dev, main, '!**']\n")),
        ] {
            let mutant = replace(&workflow, from, &to);
            assert!(
                !workflow::problems(&mutant).is_empty(),
                "this trigger must fail:\n{to}"
            );
        }
        let inline = replace(&workflow, PR, "  pull_request: {types: [closed]}\n");
        assert!(!workflow::problems(&inline).is_empty(), "{inline}");
        let both = replace(&workflow, PR, &format!("{PR}    branches: [dev, main]\n"));
        assert_eq!(workflow::problems(&both), Vec::<String>::new());

        // The forms of a full check: plain commands.
        for full in [
            "      - run: cargo deny check\n",
            "      - run: cargo deny --all-features --locked check\n",
        ] {
            let accepted = replace(&workflow, DENY, full);
            assert_eq!(
                workflow::problems(&accepted),
                Vec::<String>::new(),
                "{full}"
            );
        }
    }
}
