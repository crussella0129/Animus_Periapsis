#![forbid(unsafe_code)]
//! `praps`, the Periapsis command. This sprint it has one subcommand:
//!
//! ```text
//! praps fsck [DIR]
//! ```
//!
//! checks the store in `DIR` (default: the current directory) for checksum
//! and reference faults (INT-0007 AC-4). Exit codes follow the project's
//! convention: 0 clean, 1 corrupt or not a usable store, 2 a usage error.
//! Checking is idempotent, so there are no flags.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use hash::HashAlg;
use sim::RealFs;
use store::Store;

const USAGE: &str = "usage: praps fsck [DIR]   (no flags: checking is idempotent)";

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let (stdout, stderr) = (io::stdout(), io::stderr());
    ExitCode::from(cli(&args, &mut stdout.lock(), &mut stderr.lock()))
}

/// Runs the command line `args` (without the program name), writing the
/// report to `out` and errors to `err`; returns the exit code.
fn cli(args: &[OsString], out: &mut impl Write, err: &mut impl Write) -> u8 {
    let usage = |err: &mut dyn Write| {
        let _ = writeln!(err, "{USAGE}");
        2
    };
    if args
        .iter()
        .any(|arg| arg.to_string_lossy().starts_with('-'))
    {
        return usage(err);
    }
    let dir = match args {
        [command] if command == "fsck" => Path::new("."),
        [command, dir] if command == "fsck" => Path::new(dir),
        _ => return usage(err),
    };
    fsck(dir, out, err)
}

fn fsck(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    // `inspect`, not `open`: checking only reads, so it works on a read-only
    // store and changes nothing.
    let report = match Store::inspect(RealFs, dir, HashAlg::DEFAULT).and_then(|s| s.fsck()) {
        Ok(report) => report,
        Err(error) => {
            let _ = writeln!(err, "praps fsck: {}: {error}", dir.display());
            return 1;
        }
    };
    for leftover in &report.leftovers {
        let _ = writeln!(out, "{leftover}: temporary file left by a crash (harmless)");
    }
    for problem in &report.problems {
        let _ = writeln!(out, "{problem}");
    }
    if report.is_clean() {
        let _ = writeln!(out, "clean");
        0
    } else {
        let _ = writeln!(out, "{} problems", report.problems.len());
        1
    }
}
