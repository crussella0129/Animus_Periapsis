//! The `test` step of `cargo ci` (Sprint 5 T-003): build every test
//! executable once, then run the test targets as units, several at a time.
//!
//! Cargo runs a workspace's test executables one after another, so one long
//! simulation matrix holds up every file after it. Here each unit is its own
//! `cargo test --workspace --locked --no-fail-fast <selection>`, and up to
//! `available_parallelism()` of them run at once. Every unit keeps
//! `--workspace`, so its features resolve as the pre-build's did and nothing
//! rebuilds.
//!
//! Nothing is dropped silently. Cargo's own report of what it built is the
//! oracle: every executable the pre-build reports on an `Executable` line must
//! be reported on a `Running` line by some unit, every unit but `--doc` must
//! run something, and no unit but `--doc` may compile anything.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::workspace;

/// The pre-build: every test executable, built once.
pub const PREBUILD: [&str; 4] = ["test", "--workspace", "--locked", "--no-run"];

/// How each unit's command starts. `--no-fail-fast` keeps a failing
/// executable from hiding the ones after it in the same unit.
pub const UNIT: [&str; 4] = ["test", "--workspace", "--locked", "--no-fail-fast"];

/// Settings that fix the form of Cargo's status lines, which the step reads.
/// Set on the command, they override the environment and `term.*` in any
/// Cargo configuration.
pub const TERMINAL: [(&str, &str); 4] = [
    ("CARGO_TERM_QUIET", "false"),
    ("CARGO_TERM_VERBOSE", "false"),
    ("CARGO_TERM_COLOR", "never"),
    ("CARGO_TERM_PROGRESS_WHEN", "never"),
];

/// One `cargo test` selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub name: String,
    pub selection: Vec<String>,
}

impl Unit {
    fn new(name: &str, selection: &[&str]) -> Unit {
        Unit {
            name: name.to_owned(),
            selection: selection.iter().map(|arg| (*arg).to_owned()).collect(),
        }
    }

    fn is_doc(&self) -> bool {
        self.selection == ["--doc"]
    }

    /// The unit's full cargo arguments.
    fn args(&self) -> Vec<String> {
        UNIT.iter()
            .map(|arg| (*arg).to_owned())
            .chain(self.selection.iter().cloned())
            .collect()
    }
}

/// The units of the workspace at `root`: `--lib --bins`, then one `--test`
/// unit per distinct test-target name, sorted, then `--doc`. A member's
/// test targets are its `tests/*.rs` files and its `tests/<dir>/main.rs`
/// directories, as Cargo discovers them.
pub fn discover(root: &Path) -> Vec<Unit> {
    let mut names = BTreeSet::new();
    for member in workspace::members(root) {
        let Ok(entries) = fs::read_dir(member.join("tests")) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = if path.is_dir() {
                path.join("main.rs")
                    .is_file()
                    .then(|| entry.file_name().to_string_lossy().into_owned())
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
            } else {
                None
            };
            names.extend(name);
        }
    }
    let mut units = vec![Unit::new("lib and bins", &["--lib", "--bins"])];
    units.extend(
        names
            .iter()
            .map(|name| Unit::new(name, &["--test", name.as_str()])),
    );
    units.push(Unit::new("doc", &["--doc"]));
    units
}

/// What one command did: whether it succeeded, and its standard output and
/// error, interleaved as the command wrote them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ran {
    pub ok: bool,
    pub output: String,
}

/// Runs `command`, capturing both streams through one pipe, so each
/// `Running` line stays next to the results it heads.
pub fn run_command(mut command: Command) -> Ran {
    let failed = |error: io::Error| Ran {
        ok: false,
        output: format!("could not run cargo: {error}\n"),
    };
    let (mut reader, writer) = match io::pipe() {
        Ok(pipe) => pipe,
        Err(error) => return failed(error),
    };
    let errors = match writer.try_clone() {
        Ok(errors) => errors,
        Err(error) => return failed(error),
    };
    command.stdout(writer).stderr(errors);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => return failed(error),
    };
    // The command still holds this process's ends of the pipe; dropping it
    // closes them, so the read below ends when the child exits.
    drop(command);
    let mut output = Vec::new();
    let read = reader.read_to_end(&mut output);
    let status = child.wait();
    match (read, status) {
        (Ok(_), Ok(status)) => Ran {
            ok: status.success(),
            output: String::from_utf8_lossy(&output).into_owned(),
        },
        (Err(error), _) | (_, Err(error)) => failed(error),
    }
}

/// What a worker tells the main thread, which alone writes to `out`.
enum Event {
    Started(usize),
    Finished(usize, Ran, Duration),
}

/// How many units run at once.
pub fn slots() -> usize {
    thread::available_parallelism().map_or(1, usize::from)
}

/// Runs the test step: the pre-build, then `units` through `slots` workers,
/// then the `--doc` unit alone. `cargo` builds a command from cargo
/// arguments; the step adds `TERMINAL` to it and hands it to `run`. Each
/// unit's output is written whole to `out` as it finishes. Returns the
/// reasons the step failed, empty when it passed.
pub fn execute(
    units: &[Unit],
    slots: usize,
    out: &mut dyn Write,
    cargo: &(dyn Fn(&[&str]) -> Command + Sync),
    run: &(dyn Fn(Command) -> Ran + Sync),
) -> Vec<String> {
    let command = |args: &[&str]| {
        let mut command = cargo(args);
        command.envs(TERMINAL);
        command
    };
    let build = run(command(&PREBUILD));
    if !build.ok {
        out.write_all(build.output.as_bytes()).ok();
        return vec!["the test build failed".to_owned()];
    }
    let built = executables(&build.output, "Executable");

    let (parallel, doc): (Vec<&Unit>, Vec<&Unit>) = units.iter().partition(|unit| !unit.is_doc());
    let mut finished: Vec<(&Unit, Ran)> = Vec::new();
    let report = |out: &mut dyn Write, unit: &Unit, ran: &Ran, took: Duration| {
        let result = if ran.ok { "PASS" } else { "FAIL" };
        writeln!(
            out,
            "---- unit {}: {result} in {:.1} s",
            unit.name,
            took.as_secs_f64()
        )
        .ok();
        out.write_all(ran.output.as_bytes()).ok();
        if !ran.output.is_empty() && !ran.output.ends_with('\n') {
            writeln!(out).ok();
        }
        out.flush().ok();
    };
    let next = AtomicUsize::new(0);
    let (send, receive) = mpsc::channel();
    thread::scope(|scope| {
        for _ in 0..slots.max(1).min(parallel.len().max(1)) {
            let (send, next, parallel, command) = (send.clone(), &next, &parallel, &command);
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::SeqCst);
                    let Some(unit) = parallel.get(index) else {
                        break;
                    };
                    let args = unit.args();
                    let args: Vec<&str> = args.iter().map(String::as_str).collect();
                    if send.send(Event::Started(index)).is_err() {
                        break;
                    }
                    let started = Instant::now();
                    let ran = run(command(&args));
                    if send
                        .send(Event::Finished(index, ran, started.elapsed()))
                        .is_err()
                    {
                        break;
                    }
                }
            });
        }
        drop(send);
        for event in receive {
            match event {
                Event::Started(index) => {
                    writeln!(out, "---- unit {}: started", parallel[index].name).ok();
                    out.flush().ok();
                }
                Event::Finished(index, ran, took) => {
                    report(out, parallel[index], &ran, took);
                    finished.push((parallel[index], ran));
                }
            }
        }
    });
    for unit in doc {
        writeln!(out, "---- unit {}: started", unit.name).ok();
        let args = unit.args();
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let started = Instant::now();
        let ran = run(command(&args));
        report(out, unit, &ran, started.elapsed());
        finished.push((unit, ran));
    }

    let mut problems = Vec::new();
    let failed: Vec<&str> = finished
        .iter()
        .filter(|(_, ran)| !ran.ok)
        .map(|(unit, _)| unit.name.as_str())
        .collect();
    if !failed.is_empty() {
        problems.push(format!("units failed: {}", failed.join(", ")));
    }
    let mut run_anywhere = BTreeSet::new();
    let mut idle = Vec::new();
    let mut rebuilt = Vec::new();
    for (unit, ran) in &finished {
        let running = executables(&ran.output, "Running");
        if !unit.is_doc() {
            if running.is_empty() {
                idle.push(unit.name.as_str());
            }
            if compiled(&ran.output) {
                rebuilt.push(unit.name.as_str());
            }
        }
        run_anywhere.extend(running);
    }
    if !idle.is_empty() {
        problems.push(format!("units ran no test executable: {}", idle.join(", ")));
    }
    if !rebuilt.is_empty() {
        problems.push(format!(
            "units rebuilt after the test build: {}",
            rebuilt.join(", ")
        ));
    }
    // The oracle must not pass for want of input: with test units to run, the
    // build must have reported executables, and every executable a unit ran
    // must be one the build reported (a rename of Cargo's status lines, or a
    // rebuild, shows up here).
    if built.is_empty() && !parallel.is_empty() {
        problems.push("the test build reported no test executable".to_owned());
    }
    let unbuilt: Vec<&str> = run_anywhere
        .difference(&built)
        .map(String::as_str)
        .collect();
    if !built.is_empty() && !unbuilt.is_empty() {
        problems.push(format!(
            "units ran executables the test build did not report: {}",
            unbuilt.join(", ")
        ));
    }
    // A failing unit can stop before some of its executables, so unrun
    // executables are reported only when every unit passed.
    if failed.is_empty() {
        let unrun: Vec<&str> = built
            .difference(&run_anywhere)
            .map(String::as_str)
            .collect();
        if !unrun.is_empty() {
            problems.push(format!(
                "test executables built but never run: {}",
                unrun.join(", ")
            ));
        }
    }

    let (mut passed, mut failed_tests, mut ignored) = (0, 0, 0);
    for (_, ran) in &finished {
        for line in ran.output.lines() {
            if let Some(result) = plain(line).trim().strip_prefix("test result: ") {
                passed += count(result, "passed");
                failed_tests += count(result, "failed");
                ignored += count(result, "ignored");
            }
        }
    }
    writeln!(
        out,
        "test: {} test executables built, {} run",
        built.len(),
        run_anywhere.len()
    )
    .ok();
    writeln!(
        out,
        "test: {} units, {passed} passed, {failed_tests} failed, {ignored} ignored",
        finished.len()
    )
    .ok();
    problems
}

/// `line` without ANSI escape sequences.
fn plain(line: &str) -> String {
    let mut text = String::new();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip to the end of a CSI sequence: a final byte in `@`..=`~`.
            if chars.next() == Some('[') {
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
        } else {
            text.push(c);
        }
    }
    text
}

/// The file names of the executables Cargo reports on `<verb> … (<path>)`
/// lines, whichever separator the path uses.
pub fn executables(output: &str, verb: &str) -> BTreeSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let line = plain(line);
            let rest = line.trim().strip_prefix(verb)?.strip_prefix(' ')?;
            let path = rest.strip_suffix(')')?;
            let path = &path[path.rfind('(')? + 1..];
            let name = path.rsplit(['/', '\\']).next()?;
            (!name.is_empty()).then(|| name.to_owned())
        })
        .collect()
}

/// Whether Cargo compiled anything: a `Compiling` status line.
fn compiled(output: &str) -> bool {
    output
        .lines()
        .any(|line| plain(line).trim_start().starts_with("Compiling "))
}

/// The number before ` <what>` in a `test result:` summary, or 0.
fn count(result: &str, what: &str) -> u64 {
    result
        .split([';', '.'])
        .filter_map(|part| part.trim().strip_suffix(what))
        .filter_map(|number| number.trim().parse::<u64>().ok())
        .sum()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use super::{PREBUILD, Ran, TERMINAL, Unit, discover, executables, execute};

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-units-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// A workspace with members `one` and `two`.
    fn workspace(root: &Path) {
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"one\", \"two\"]\n",
        );
        for member in ["one", "two"] {
            write(
                &root.join(member).join("Cargo.toml"),
                &format!("[package]\nname = \"{member}\"\nversion = \"0.0.0\"\n"),
            );
        }
    }

    fn selections(units: &[Unit]) -> Vec<String> {
        units.iter().map(|unit| unit.selection.join(" ")).collect()
    }

    #[test]
    fn units_cover_every_test_target() {
        let root = temp("cover");
        workspace(&root);
        write(&root.join("one/tests/a.rs"), "");
        write(&root.join("one/tests/b.rs"), "");
        write(&root.join("two/tests/c/main.rs"), "");
        write(&root.join("two/tests/common/mod.rs"), "");
        write(&root.join("two/tests/fixtures/data.txt"), "");
        let units = discover(&root);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            selections(&units),
            ["--lib --bins", "--test a", "--test b", "--test c", "--doc"]
        );
    }

    #[test]
    fn units_merge_shared_names() {
        let root = temp("merge");
        workspace(&root);
        write(&root.join("one/tests/crash.rs"), "");
        write(&root.join("two/tests/crash.rs"), "");
        let units = discover(&root);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            selections(&units),
            ["--lib --bins", "--test crash", "--doc"]
        );
    }

    /// A fake cargo: each command is recorded by its arguments. The
    /// pre-build reports `built`; a unit reports running the executable
    /// named after its last argument, unless `outputs` says otherwise.
    struct Fake {
        built: Vec<&'static str>,
        outputs: Vec<(&'static str, Ran)>,
        commands: Mutex<Vec<Vec<String>>>,
    }

    impl Fake {
        fn new(built: &[&'static str]) -> Fake {
            Fake {
                built: built.to_vec(),
                outputs: Vec::new(),
                commands: Mutex::new(Vec::new()),
            }
        }

        fn with(mut self, last: &'static str, ok: bool, output: &str) -> Fake {
            self.outputs.push((
                last,
                Ran {
                    ok,
                    output: output.to_owned(),
                },
            ));
            self
        }

        fn run(&self, command: Command) -> Ran {
            let args: Vec<String> = command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            self.commands.lock().unwrap().push(args.clone());
            if args.iter().any(|arg| arg == "--no-run") {
                let output = self
                    .built
                    .iter()
                    .map(|name| {
                        format!("  Executable tests/{name}.rs (target/debug/deps/{name})\n")
                    })
                    .collect();
                return Ran { ok: true, output };
            }
            let last = args.last().unwrap().as_str();
            if let Some((_, ran)) = self.outputs.iter().find(|(name, _)| *name == last) {
                return ran.clone();
            }
            if last == "--doc" {
                // Doc tests have no executable of their own, as with Cargo.
                return Ran {
                    ok: true,
                    output: "   Doc-tests fake\n".to_owned(),
                };
            }
            Ran {
                ok: true,
                output: format!(
                    "     Running tests/{last}.rs (target/debug/deps/{last})\n\
                     test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured\n"
                ),
            }
        }
    }

    /// Pre-build output reporting executables `u0` to `u{n-1}`.
    fn built(n: usize) -> String {
        (0..n)
            .map(|i| format!("  Executable tests/u{i}.rs (target/debug/deps/u{i})\n"))
            .collect()
    }

    fn units(names: &[&str]) -> Vec<Unit> {
        names
            .iter()
            .map(|name| match *name {
                "doc" => Unit::new("doc", &["--doc"]),
                name => Unit::new(name, &["--test", name]),
            })
            .collect()
    }

    fn cargo(args: &[&str]) -> Command {
        let mut command = Command::new("cargo");
        command.args(args);
        command
    }

    fn step(fake: &Fake, names: &[&str], slots: usize) -> (Vec<String>, String) {
        let mut out = Vec::new();
        let problems = execute(&units(names), slots, &mut out, &cargo, &|command| {
            fake.run(command)
        });
        (problems, String::from_utf8(out).unwrap())
    }

    #[test]
    fn test_step_builds_once_then_runs_each_unit() {
        let fake = Fake::new(&["a", "b"]);
        let (problems, _) = step(&fake, &["a", "b", "doc"], 2);
        assert_eq!(problems, Vec::<String>::new());
        let commands = fake.commands.into_inner().unwrap();
        assert_eq!(commands[0], PREBUILD);
        let mut units: Vec<String> = commands[1..].iter().map(|args| args.join(" ")).collect();
        units.sort();
        assert_eq!(
            units,
            [
                "test --workspace --locked --no-fail-fast --doc",
                "test --workspace --locked --no-fail-fast --test a",
                "test --workspace --locked --no-fail-fast --test b",
            ]
        );
    }

    #[test]
    fn a_failed_build_runs_no_unit() {
        let commands = Mutex::new(0);
        let mut out = Vec::new();
        let problems = execute(&units(&["a", "doc"]), 2, &mut out, &cargo, &|_| {
            *commands.lock().unwrap() += 1;
            Ran {
                ok: false,
                output: "error[E0425]: cannot find value `x`\n".to_owned(),
            }
        });
        assert_eq!(problems, ["the test build failed"]);
        assert_eq!(*commands.lock().unwrap(), 1, "only the pre-build ran");
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "error[E0425]: cannot find value `x`\n"
        );
    }

    #[test]
    fn the_step_sets_cargo_terminal_settings() {
        let seen = Mutex::new(Vec::new());
        let fake = Fake::new(&["a"]);
        let mut out = Vec::new();
        execute(&units(&["a", "doc"]), 2, &mut out, &cargo, &|command| {
            let envs: Vec<(String, String)> = command
                .get_envs()
                .filter_map(|(key, value)| {
                    Some((
                        key.to_string_lossy().into_owned(),
                        value?.to_string_lossy().into_owned(),
                    ))
                })
                .collect();
            seen.lock().unwrap().push(envs);
            fake.run(command)
        });
        let seen = seen.into_inner().unwrap();
        assert_eq!(seen.len(), 3, "the pre-build and two units");
        for envs in seen {
            for (key, value) in TERMINAL {
                assert!(
                    envs.contains(&(key.to_owned(), value.to_owned())),
                    "{key}={value} missing from {envs:?}"
                );
            }
        }
    }

    #[test]
    fn units_never_exceed_the_slots() {
        let names: Vec<String> = (0..12).map(|i| format!("u{i}")).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let (now, most, ran) = (
            AtomicUsize::new(0),
            AtomicUsize::new(0),
            AtomicUsize::new(0),
        );
        let mut out = Vec::new();
        let problems = execute(&units(&names), 3, &mut out, &cargo, &|command| {
            let last = command
                .get_args()
                .last()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if last == "--no-run" {
                return Ran {
                    ok: true,
                    output: built(12),
                };
            }
            let inside = now.fetch_add(1, Ordering::SeqCst) + 1;
            most.fetch_max(inside, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(20));
            now.fetch_sub(1, Ordering::SeqCst);
            ran.fetch_add(1, Ordering::SeqCst);
            Ran {
                ok: true,
                output: format!("     Running tests/{last}.rs (target/debug/deps/{last})\n"),
            }
        });
        assert_eq!(problems, Vec::<String>::new());
        assert_eq!(ran.load(Ordering::SeqCst), 12);
        assert_eq!(most.load(Ordering::SeqCst), 3);
    }

    /// Only the main thread writes to the step's writer, which is not `Send`,
    /// so units cannot interleave by construction; this test pins that down
    /// against a change that would let workers write.
    #[test]
    fn unit_output_is_never_interleaved() {
        let names: Vec<String> = (0..8).map(|i| format!("u{i}")).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut out = Vec::new();
        execute(&units(&names), 4, &mut out, &cargo, &|command| {
            let last = command
                .get_args()
                .last()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let mut output = format!("     Running tests/{last}.rs (target/debug/deps/{last})\n");
            for _ in 0..50 {
                output.push_str(&format!("marker {last}\n"));
                thread::yield_now();
            }
            Ran { ok: true, output }
        });
        let out = String::from_utf8(out).unwrap();
        // Each unit's markers form one run of lines, directly under its header.
        for name in &names {
            let lines: Vec<&str> = out.lines().collect();
            let header = lines
                .iter()
                .position(|line| line.starts_with(&format!("---- unit {name}: PASS")))
                .unwrap();
            let marker = format!("marker {name}");
            let first = lines.iter().position(|line| *line == marker).unwrap();
            assert_eq!(first, header + 2, "{name}");
            assert!(
                lines[first..first + 50].iter().all(|line| *line == marker),
                "{name}"
            );
            assert_eq!(lines.iter().filter(|line| **line == marker).count(), 50);
        }
    }

    #[test]
    fn a_failed_unit_fails_the_step_after_all_run() {
        let fake = Fake::new(&["u0", "u1", "u2", "u3", "u4", "u5"]).with(
            "u2",
            false,
            "     Running tests/u2.rs (target/debug/deps/u2)\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n",
        );
        let (problems, _) = step(&fake, &["u0", "u1", "u2", "u3", "u4", "u5"], 2);
        assert_eq!(problems, ["units failed: u2"]);
        assert_eq!(fake.commands.into_inner().unwrap().len(), 7);
    }

    #[test]
    fn a_failing_unit_is_not_reported_as_unrun() {
        // `x` fails before running `y-2`, which it would have run next.
        let fake = Fake::new(&["x", "y-2"]).with(
            "x",
            false,
            "     Running tests/x.rs (target/debug/deps/x)\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n",
        );
        let (problems, _) = step(&fake, &["x"], 2);
        assert_eq!(problems, ["units failed: x"]);
    }

    #[test]
    fn an_unrun_executable_fails_the_step() {
        let fake = Fake::new(&["x-1", "y-2"]);
        let (problems, _) = step(&fake, &["x-1"], 2);
        assert_eq!(problems, ["test executables built but never run: y-2"]);
    }

    #[test]
    fn a_unit_that_runs_nothing_fails_the_step() {
        let fake = Fake::new(&["a"])
            .with(
                "a",
                true,
                "test result: ok. 0 passed; 0 failed; 0 ignored\n",
            )
            .with("--doc", true, "   Doc-tests codec\n");
        let (problems, _) = step(&fake, &["a", "doc"], 2);
        assert_eq!(
            problems,
            [
                "units ran no test executable: a",
                "test executables built but never run: a"
            ]
        );
    }

    #[test]
    fn a_unit_that_rebuilds_fails_the_step() {
        for compiling in [
            "   Compiling codec v0.1.0 (C:\\repo\\crates\\codec)",
            "\u{1b}[1m\u{1b}[32m   Compiling\u{1b}[0m codec v0.1.0",
        ] {
            let fake = Fake::new(&["a"])
                .with(
                    "a",
                    true,
                    &format!("{compiling}\n     Running tests/a.rs (target/debug/deps/a)\n"),
                )
                .with("--doc", true, &format!("{compiling}\n   Doc-tests codec\n"));
            let (problems, _) = step(&fake, &["a", "doc"], 2);
            assert_eq!(
                problems,
                ["units rebuilt after the test build: a"],
                "{compiling:?}"
            );
        }
    }

    #[test]
    fn the_doc_unit_runs_alone_and_last() {
        let (now, alone) = (AtomicUsize::new(0), Mutex::new(None));
        let order = Mutex::new(Vec::new());
        let mut out = Vec::new();
        execute(
            &units(&["a", "b", "c", "doc"]),
            3,
            &mut out,
            &cargo,
            &|command| {
                let last = command
                    .get_args()
                    .last()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                if last == "--no-run" {
                    return Ran {
                        ok: true,
                        output: String::new(),
                    };
                }
                let inside = now.fetch_add(1, Ordering::SeqCst) + 1;
                if last == "--doc" {
                    *alone.lock().unwrap() = Some(inside == 1);
                }
                thread::sleep(Duration::from_millis(20));
                now.fetch_sub(1, Ordering::SeqCst);
                order.lock().unwrap().push(last.clone());
                Ran {
                    ok: true,
                    output: format!("     Running tests/{last}.rs (target/debug/deps/{last})\n"),
                }
            },
        );
        assert_eq!(*alone.lock().unwrap(), Some(true));
        assert_eq!(
            order.into_inner().unwrap().last().map(String::as_str),
            Some("--doc")
        );
    }

    #[test]
    fn the_step_totals_every_test_result() {
        let fake = Fake::new(&["a", "b"])
            .with(
                "a",
                true,
                "     Running tests/a.rs (target/debug/deps/a)\ntest result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s\n",
            )
            .with(
                "b",
                false,
                "     Running tests/b.rs (target/debug/deps/b)\ntest result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s\n",
            );
        let (_, out) = step(&fake, &["a", "b"], 2);
        assert_eq!(
            out.lines().rev().take(2).collect::<Vec<_>>(),
            [
                "test: 2 units, 4 passed, 2 failed, 1 ignored",
                "test: 2 test executables built, 2 run"
            ]
        );
    }

    #[test]
    fn a_build_that_reports_nothing_fails_the_step() {
        // Cargo's `Executable` lines did not parse, or there were none: the
        // unrun check would otherwise pass without checking anything.
        let fake = Fake::new(&[]);
        let (problems, _) = step(&fake, &["a"], 2);
        assert_eq!(problems, ["the test build reported no test executable"]);
    }

    #[test]
    fn an_executable_the_build_did_not_report_fails_the_step() {
        let fake = Fake::new(&["a"]).with(
            "b",
            true,
            "     Running tests/b.rs (target/debug/deps/b-9)\n",
        );
        let (problems, _) = step(&fake, &["a", "b"], 2);
        assert_eq!(
            problems,
            ["units ran executables the test build did not report: b-9"]
        );
    }

    #[test]
    fn each_unit_is_announced_when_it_starts() {
        let fake = Fake::new(&["a", "b"]);
        let (_, out) = step(&fake, &["a", "b", "doc"], 1);
        for name in ["a", "b", "doc"] {
            let started = out
                .lines()
                .position(|line| line == format!("---- unit {name}: started"))
                .unwrap_or_else(|| panic!("{name} was not announced"));
            let finished = out
                .lines()
                .position(|line| line.starts_with(&format!("---- unit {name}: PASS")))
                .unwrap();
            assert!(started < finished, "{name}");
        }
    }

    /// The child half of `run_command_keeps_the_streams_in_order`: with
    /// `XTASK_STREAM_ORDER` set, it writes to stdout, then stderr, then stdout,
    /// flushing each. Run on its own, it does nothing.
    #[test]
    fn stream_order_child() {
        use std::io::Write;
        if std::env::var_os("XTASK_STREAM_ORDER").is_none() {
            return;
        }
        let (mut out, mut err) = (std::io::stdout(), std::io::stderr());
        out.write_all(b"order-first\n").unwrap();
        out.flush().unwrap();
        err.write_all(b"order-second\n").unwrap();
        err.flush().unwrap();
        out.write_all(b"order-third\n").unwrap();
        out.flush().unwrap();
    }

    #[test]
    fn run_command_keeps_the_streams_in_order() {
        // A child that writes to stdout, then stderr, then stdout again: the
        // captured output keeps that order. Capturing the two streams apart
        // and joining them would put `order-second` last (test critique
        // C-002).
        let mut child = Command::new(std::env::current_exe().unwrap());
        child
            .args(["--exact", "units::tests::stream_order_child", "--nocapture"])
            .env("XTASK_STREAM_ORDER", "1");
        let ran = super::run_command(child);
        let at = |marker: &str| {
            ran.output
                .find(marker)
                .unwrap_or_else(|| panic!("{marker} missing from {ran:?}"))
        };
        assert!(ran.ok, "{ran:?}");
        assert!(
            at("order-first") < at("order-second") && at("order-second") < at("order-third"),
            "{ran:?}"
        );

        // The exit status and a program that cannot start.
        let mut command = Command::new(env!("CARGO"));
        command.args(["--version"]);
        let ran = super::run_command(command);
        assert!(ran.ok && ran.output.starts_with("cargo "), "{ran:?}");
        let mut command = Command::new(env!("CARGO"));
        command.arg("--no-such-flag");
        let ran = super::run_command(command);
        assert!(!ran.ok && ran.output.contains("--no-such-flag"), "{ran:?}");
        let ran = super::run_command(Command::new("no-such-program-anywhere"));
        assert!(
            !ran.ok && ran.output.starts_with("could not run cargo: "),
            "{ran:?}"
        );
    }

    #[test]
    fn cargo_status_lines_parse_on_both_systems() {
        let output = "\
  Executable unittests src\\lib.rs (target\\debug\\deps\\codec-4a546ad5229c1036.exe)
     Running tests/crash.rs (target/debug/deps/crash-32b2523c1477e52b)
     Running unittests src/main.rs (target/debug/deps/praps-a08907f81f5aba35)
\u{1b}[1m\u{1b}[32m     Running\u{1b}[0m tests/e2e.rs (target/debug/deps/e2e-70e726f17e5bb99e)
test crash_matrix_finds_no_violations ... ok
";
        assert_eq!(
            executables(output, "Executable"),
            BTreeSet::from(["codec-4a546ad5229c1036.exe".to_owned()])
        );
        assert_eq!(
            executables(output, "Running"),
            BTreeSet::from([
                "crash-32b2523c1477e52b".to_owned(),
                "praps-a08907f81f5aba35".to_owned(),
                "e2e-70e726f17e5bb99e".to_owned(),
            ])
        );
    }

    /// This repository yields one unit per distinct test-file stem, between
    /// the two fixed units.
    #[test]
    fn repository_units_match_its_test_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut stems = BTreeSet::new();
        for member in fs::read_dir(root.join("crates")).unwrap().flatten() {
            if let Ok(tests) = fs::read_dir(member.path().join("tests")) {
                for file in tests.flatten() {
                    let path = file.path();
                    if path.extension() == Some(OsStr::new("rs")) {
                        stems.insert(path.file_stem().unwrap().to_string_lossy().into_owned());
                    }
                }
            }
        }
        let units = discover(&root);
        assert_eq!(
            units.first().map(|unit| unit.name.as_str()),
            Some("lib and bins")
        );
        assert_eq!(units.last().map(|unit| unit.name.as_str()), Some("doc"));
        let named: BTreeSet<String> = units[1..units.len() - 1]
            .iter()
            .map(|unit| unit.name.clone())
            .collect();
        assert_eq!(named, stems);
    }
}
