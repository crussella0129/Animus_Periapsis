#![forbid(unsafe_code)]
//! `praps`, the Periapsis command.
//!
//! ```text
//! praps init [DIR]     make DIR (default: .) a repository
//! praps new [DIR]      start a new working-copy change
//! praps status [DIR]   snapshot the working copy and list its changes
//! praps fsck [DIR]     check the store of the repository or store in DIR
//! ```
//!
//! `init`, `new` and `status` open the working copy and snapshot it first
//! (INT-0014). `fsck` only reads (INT-0007 AC-4), and `init` on an existing
//! repository writes nothing, so neither snapshots. Exit codes follow the
//! project's convention: 0 success (for `fsck`, clean), 1 a repository
//! error (not a repository, several op heads, corruption), 2 a usage error.
//! The commands take no flags.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use hash::HashAlg;
use sim::{Fs, RealFs, os_seed};
use store::Store;
use workspace::{PRAPS_DIR, STORE_DIR, Workspace};

const USAGE: &str = "usage: praps init|new|status|fsck [DIR]   (no flags)";

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
    let (command, dir) = match args {
        [command] => (command, Path::new(".")),
        [command, dir] => (command, Path::new(dir)),
        _ => return usage(err),
    };
    match command.to_str() {
        Some("init") => init(dir, out, err),
        Some("new") => new(dir, out, err),
        Some("status") => status(dir, out, err),
        Some("fsck") => fsck(dir, out, err),
        _ => usage(err),
    }
}

fn fail(command: &str, dir: &Path, error: impl std::fmt::Display, err: &mut impl Write) -> u8 {
    let _ = writeln!(err, "praps {command}: {}: {error}", dir.display());
    1
}

fn init(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    match Workspace::init(RealFs, dir, os_seed()) {
        Ok((_, true)) => {
            let _ = writeln!(
                out,
                "initialised a Periapsis repository in {}",
                dir.display()
            );
            0
        }
        Ok((_, false)) => {
            let _ = writeln!(out, "{} is already a Periapsis repository", dir.display());
            0
        }
        Err(error) => fail("init", dir, error, err),
    }
}

/// Opens the repository at `dir`, printing each note that resuming an
/// interrupted write left (`note: …`) on `err`.
fn open(dir: &Path, err: &mut impl Write) -> Result<Workspace<RealFs>, workspace::Error> {
    let workspace = Workspace::open(RealFs, dir, os_seed())?;
    for note in workspace.notes() {
        let _ = writeln!(err, "note: {note}");
    }
    Ok(workspace)
}

fn new(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let result = open(dir, err).and_then(|mut ws| ws.new_change());
    match result {
        Ok(change_id) => {
            let hex: String = change_id.iter().map(|b| format!("{b:02x}")).collect();
            let _ = writeln!(out, "new change {hex}");
            0
        }
        Err(error) => fail("new", dir, error, err),
    }
}

fn status(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let result = open(dir, err).and_then(|mut ws| ws.status());
    match result {
        Ok(status) => {
            for (path, change) in &status.changes {
                // Paths are valid UTF-8 repository paths; they are shown in
                // the same escaped form as reported names.
                let shown = sim::Name::Utf8(path.as_str().to_owned()).display();
                let _ = writeln!(out, "{} {shown}", change.letter());
            }
            for reported in &status.reported {
                let _ = writeln!(
                    out,
                    "skipped {} ({})",
                    reported.path,
                    reported.reason.text()
                );
            }
            0
        }
        Err(error) => fail("status", dir, error, err),
    }
}

/// The store `fsck` checks: `DIR/.praps/store` when `DIR` holds a `.praps`
/// entry, whatever state that store is in, so damage there is reported
/// rather than hidden by checking `DIR` instead (code review CR-09);
/// otherwise `DIR` itself, as a bare store.
fn store_dir(dir: &Path) -> PathBuf {
    let has_praps = RealFs
        .list(dir)
        .is_ok_and(|names| names.iter().any(|name| name == PRAPS_DIR));
    if has_praps {
        dir.join(PRAPS_DIR).join(STORE_DIR)
    } else {
        dir.to_path_buf()
    }
}

fn fsck(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let store_dir = store_dir(dir);
    // `inspect`, not `open`: checking only reads, so it works on a read-only
    // store and changes nothing.
    let report = match Store::inspect(RealFs, &store_dir, HashAlg::DEFAULT).and_then(|s| s.fsck()) {
        Ok(report) => report,
        Err(error) => return fail("fsck", dir, error, err),
    };
    for leftover in &report.leftovers {
        let _ = writeln!(out, "{leftover}: left by an interrupted write (harmless)");
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
