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
mod policy;
#[cfg(test)]
mod workflow;
mod workspace;

use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

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
    let code = cli(
        &args,
        &workspace_root(),
        &mut io::stdout(),
        &mut io::stderr(),
    );
    ExitCode::from(code)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{cli, workspace_root};

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

    fn assert_usage_error(args: &[OsString]) {
        let (code, out, err) = run(args);
        assert_eq!(code, 2, "{args:?}");
        assert!(!err.trim().is_empty(), "{args:?} must explain its usage");
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
            &["--ci"],
        ] {
            assert_usage_error(&os_args(args));
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
}
