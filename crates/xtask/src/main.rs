#![forbid(unsafe_code)]
//! Project tooling behind the `cargo policy` alias.
//!
//! Every command here is idempotent, so none takes flags (INT-0003 AC-9).

mod policy;

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: cargo policy   (no flags: the command is idempotent)";

/// Runs one command and returns its exit code: 0 clean, 1 problems found,
/// 2 for a usage error. Output goes to the given writers, so tests need no
/// subprocess.
fn cli(args: &[String], root: &Path, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match args {
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
    let args: Vec<String> = env::args().skip(1).collect();
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
    use super::{cli, workspace_root};

    fn run(args: &[&str]) -> (u8, String, String) {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = cli(&args, &workspace_root(), &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
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
            let (code, out, err) = run(args);
            assert_eq!(code, 2, "{args:?}");
            assert!(!err.trim().is_empty(), "{args:?} must explain its usage");
            assert!(out.is_empty(), "{args:?} must not run the command");
        }
    }

    #[test]
    fn policy_command_reports_every_rule() {
        let (code, out, err) = run(&["policy"]);
        assert_eq!(code, 0, "{out}{err}");
        for (rule, _) in super::policy::RULES {
            assert!(
                out.contains(&format!("PASS {rule}\n")),
                "{rule} missing from {out}"
            );
        }
    }
}
