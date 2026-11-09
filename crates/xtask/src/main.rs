#![forbid(unsafe_code)]
//! Project tooling behind the `cargo ci` and `cargo policy` aliases.
//!
//! Every command here is idempotent, so none takes flags (INT-0003 AC-9).

mod attributes;
mod ci;
#[cfg(test)]
mod deny;
#[cfg(test)]
mod docs;
mod manifest;
#[cfg(test)]
mod pins;
mod points;
mod policy;
#[cfg(test)]
mod workflow;
mod workspace;

use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: cargo ci | cargo policy   (no flags: both commands are idempotent)";

/// Runs one command and returns its exit code: 0 clean, 1 problems found,
/// 2 for a usage error, including an argument that is not valid Unicode.
/// Output goes to the given writers, so tests need no subprocess.
fn cli(args: &[OsString], root: &Path, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match args {
        [command] if command == "ci" => ci::run(root, out),
        [command] if command == "policy" => {
            let mut clean = true;
            for result in policy::check(root) {
                if result.problems.is_empty() {
                    writeln!(out, "PASS {}", result.rule).ok();
                } else {
                    clean = false;
                    for problem in &result.problems {
                        writeln!(out, "FAIL {}: {problem}", result.rule).ok();
                    }
                }
            }
            u8::from(!clean)
        }
        _ => {
            writeln!(err, "{USAGE}").ok();
            2
        }
    }
}

/// The workspace root, two levels above this crate's manifest.
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(manifest_dir)
        .to_path_buf()
}

fn main() -> ExitCode {
    // `args_os`, because `args` panics on an argument that is not Unicode.
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let code = guarded(
        &args,
        &workspace_root(),
        || (env::current_exe(), located_workspace()),
        &mut io::stdout(),
        &mut io::stderr(),
    );
    ExitCode::from(code)
}

/// `cli`, refused with exit 1 when this binary must not check `root`: see
/// [`foreign`]. Arguments are validated first, so a usage error still exits
/// 2; `paths` gives the running executable and the manifest of the current
/// directory's workspace, and is only asked for a valid command.
fn guarded(
    args: &[OsString],
    root: &Path,
    paths: impl FnOnce() -> (io::Result<PathBuf>, io::Result<PathBuf>),
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    if matches!(args, [command] if command == "ci" || command == "policy") {
        let (exe, located) = paths();
        if let Some(problem) = foreign(root, exe, located) {
            writeln!(err, "{problem}").ok();
            return 1;
        }
    }
    cli(args, root, out, err)
}

/// Why this xtask binary must not check the checkout at `root`, if it must
/// not (Sprint 0 test critiques round 3, C-005; round 5, C-006; round 6,
/// C-008; round 7, C-007):
///
/// - the binary is not inside `<root>/target`: the `cargo ci` alias built it
///   into a target directory outside the checkout (`CARGO_TARGET_DIR` or
///   `build.target-dir`), where another checkout's build could stand in for
///   this one's;
/// - the current directory's workspace is not `root`: the binary was built
///   for another checkout.
///
/// Paths are compared canonicalized, so Windows `\\?\` prefixes and letter
/// case agree. Anything that cannot be resolved is refused.
fn foreign(root: &Path, exe: io::Result<PathBuf>, located: io::Result<PathBuf>) -> Option<String> {
    let canonical = |path: &Path| std::fs::canonicalize(path).ok();
    let Some(root_dir) = canonical(root) else {
        return Some(format!(
            "xtask: cannot resolve its workspace {}",
            root.display()
        ));
    };
    let target = root.join("target");
    let exe = exe.ok().and_then(|exe| canonical(&exe));
    match (&exe, canonical(&target)) {
        (Some(exe), Some(target_dir)) if exe.starts_with(&target_dir) => {}
        _ => {
            return Some(format!(
                "xtask: {} is not inside {}; a target directory outside the checkout \
                 (CARGO_TARGET_DIR or build.target-dir) is refused",
                exe.as_deref()
                    .map_or("the running executable".into(), |exe| exe
                        .display()
                        .to_string()),
                target.display()
            ));
        }
    }
    let here = located
        .ok()
        .and_then(|manifest| manifest.parent().map(Path::to_path_buf))
        .and_then(|dir| canonical(&dir));
    match here {
        Some(here) if here == root_dir => None,
        Some(here) => Some(format!(
            "xtask: built for the checkout at {}, but run in the workspace at {}",
            root_dir.display(),
            here.display()
        )),
        None => Some(format!(
            "xtask: the current directory is not in a Cargo workspace; run it in {}",
            root_dir.display()
        )),
    }
}

/// The root manifest of the current directory's workspace, as
/// `cargo locate-project --workspace` reports it with the inherited
/// environment.
fn located_workspace() -> io::Result<PathBuf> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args(["locate-project", "--workspace", "--message-format", "plain"])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::OsString;
    use std::path::Path;
    use std::process::{Command, ExitCode};

    use super::{cli, foreign, guarded, located_workspace, workspace_root};

    const USAGE_LINE: &str =
        "usage: cargo ci | cargo policy   (no flags: both commands are idempotent)\n";

    fn run(args: &[OsString]) -> (u8, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = cli(args, &workspace_root(), &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn os_args(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    /// `cli` rejects `args` as a usage error. It runs on a root where nothing
    /// can run, so a command that wrongly takes `args` fails at once rather
    /// than running the real suite.
    fn assert_usage_error(args: &[OsString]) {
        let nowhere = Path::new(env!("CARGO_MANIFEST_DIR")).join("no-such-workspace");
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = cli(args, &nowhere, &mut out, &mut err);
        assert_eq!(code, 2, "{args:?}");
        assert_eq!(
            String::from_utf8(err).unwrap(),
            USAGE_LINE,
            "{args:?} must explain its usage"
        );
        assert!(out.is_empty(), "{args:?} must not run the command");
    }

    /// `prefix` followed by a code unit that is not valid Unicode on this
    /// platform: a lone surrogate on Windows, a stray byte elsewhere.
    #[cfg(windows)]
    fn non_unicode(prefix: &str) -> OsString {
        use std::os::windows::ffi::OsStringExt;
        let mut wide: Vec<u16> = prefix.encode_utf16().collect();
        wide.push(0xD800);
        OsString::from_wide(&wide)
    }

    #[cfg(unix)]
    fn non_unicode(prefix: &str) -> OsString {
        use std::os::unix::ffi::OsStringExt;
        let mut bytes = prefix.as_bytes().to_vec();
        bytes.push(0xff);
        OsString::from_vec(bytes)
    }

    #[test]
    fn policy_rejects_extra_arguments() {
        for args in [
            &["policy", "--fix"][..],
            &["policy", "-v"],
            &["policy", "x"],
            &["policy", "--"],
            &["policy", ""],
            &["--policy"],
            &[],
        ] {
            assert_usage_error(&os_args(args));
        }
    }

    #[test]
    fn ci_rejects_extra_arguments() {
        for args in [
            &["ci", "--fast"][..],
            &["ci", "-q"],
            &["ci", "x"],
            &["ci", "--"],
            &["ci", ""],
            &["--ci"],
        ] {
            assert_usage_error(&os_args(args));
        }
    }

    /// `main` drops no argument after the command name. A child copy of this
    /// test binary starts as `policy --exact <the child half>`: libtest reads
    /// `policy` as a filter that names no test, while `main` must reject
    /// `policy` with two more arguments instead of running the rules. That
    /// `main` skips exactly the program name, and exits 0 for a valid command,
    /// is shown where `cargo policy` and `cargo ci` run for real: the
    /// integration probes in docs/sprints/s0/sprint-tests.
    #[test]
    fn main_passes_every_argument_to_cli() {
        let child = Command::new(env::current_exe().unwrap())
            .args(["policy", "--exact", "tests::main_rejects_extra_arguments"])
            .env("XTASK_MAIN_CHILD", "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&child.stdout);
        let stderr = String::from_utf8_lossy(&child.stderr);
        assert!(
            child.status.success()
                && stdout.contains("test tests::main_rejects_extra_arguments ... ok"),
            "{stdout}{stderr}"
        );
        assert_eq!(stderr, USAGE_LINE);
    }

    /// The child half of `main_passes_every_argument_to_cli`: `main` itself,
    /// with this process's arguments. Run on its own, without
    /// `XTASK_MAIN_CHILD`, it does nothing.
    #[test]
    fn main_rejects_extra_arguments() {
        if env::var_os("XTASK_MAIN_CHILD").is_some() {
            assert_eq!(super::main(), ExitCode::from(2));
        }
    }

    #[test]
    fn non_unicode_arguments_are_usage_errors() {
        for prefix in ["", "-"] {
            let arg = non_unicode(prefix);
            assert!(arg.to_str().is_none(), "{arg:?} must not be Unicode");
            for args in [
                vec![arg.clone()],
                vec!["policy".into(), arg.clone()],
                vec!["ci".into(), arg.clone()],
                vec![arg.clone(), "policy".into()],
            ] {
                assert_usage_error(&args);
            }
        }
    }

    #[test]
    fn policy_command_reports_every_rule() {
        let (code, out, err) = run(&os_args(&["policy"]));
        assert_eq!(code, 0, "{out}{err}");
        for (rule, _) in super::policy::RULES {
            assert!(
                out.contains(&format!("PASS {rule}\n")),
                "{rule} missing from {out}"
            );
        }
    }

    #[test]
    fn xtask_outside_its_target_dir_is_refused() {
        let root = workspace_root();
        let outside = env::temp_dir().join(format!("xtask-outside-{}", std::process::id()));
        std::fs::write(&outside, b"stand-in executable").unwrap();
        let located = Ok(root.join("Cargo.toml"));
        let problem = foreign(&root, Ok(outside.clone()), located).unwrap();
        let shown = std::fs::canonicalize(&outside).unwrap();
        assert!(problem.contains(&shown.display().to_string()), "{problem}");
        assert!(
            problem.contains(&root.join("target").display().to_string()),
            "{problem}"
        );
        assert!(problem.contains("CARGO_TARGET_DIR"), "{problem}");
        // An executable that cannot be resolved is refused too.
        assert!(
            foreign(
                &root,
                Err(std::io::Error::other("gone")),
                Ok(root.join("Cargo.toml"))
            )
            .is_some()
        );
        std::fs::remove_file(&outside).unwrap();
    }

    #[test]
    fn xtask_for_another_checkout_is_refused() {
        let root = workspace_root();
        let exe = env::current_exe();
        // Another workspace: here, a member directory stands in for it.
        let other = root.join("crates/hash/Cargo.toml");
        let problem = foreign(&root, exe, Ok(other)).unwrap();
        assert!(problem.contains("built for the checkout"), "{problem}");
        let problem = foreign(
            &root,
            env::current_exe(),
            Err(std::io::Error::other("none")),
        )
        .unwrap();
        assert!(problem.contains("not in a Cargo workspace"), "{problem}");
    }

    /// The real paths: this test binary lives in `<root>/target` (as `cargo ci`
    /// builds it, with every CARGO_TARGET_* variable scrubbed), and cargo
    /// locates this workspace. On Windows the canonical paths carry `\\?\`.
    #[test]
    fn xtask_in_its_own_target_dir_runs() {
        let root = workspace_root();
        let located = located_workspace();
        assert!(located.is_ok(), "{located:?}");
        assert_eq!(foreign(&root, env::current_exe(), located), None);
    }

    #[test]
    fn usage_errors_come_before_the_checkout_checks() {
        let root = workspace_root();
        let outside = || {
            (
                Ok(env::temp_dir().join("no-such-xtask")),
                Ok(workspace_root().join("Cargo.toml")),
            )
        };
        let run = |args: &[&str]| {
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let code = guarded(&os_args(args), &root, outside, &mut out, &mut err);
            (code, out, String::from_utf8(err).unwrap())
        };
        let (code, out, err) = run(&["ci", "--fast"]);
        assert_eq!((code, err.as_str()), (2, USAGE_LINE));
        assert!(out.is_empty());
        // A valid command is refused before anything runs.
        for command in ["ci", "policy"] {
            let (code, out, err) = run(&[command]);
            assert_eq!(code, 1, "{err}");
            assert!(out.is_empty(), "{command} must not run");
            assert!(err.contains("is not inside"), "{err}");
        }
    }
}
