//! What each command does. [`run`] is the one place a registry entry
//! becomes an action, by an exhaustive `match`, so a command the registry
//! holds cannot be left without a body.

use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};

use hash::HashAlg;
use sim::{Fs, RealFs, os_seed};
use store::Store;
use workspace::{PRAPS_DIR, STORE_DIR, Workspace};

use crate::help;
use crate::registry::{Action, Command};

/// How many hex digits of an op id or change ID `praps` prints.
const SHOWN: usize = 12;

/// Runs `command` with `args`, which the caller has counted against the
/// registry entry. The working copy's directory is the last argument, and
/// the current directory when it is left out.
pub fn run(command: &Command, args: &[&OsStr], out: &mut impl Write, err: &mut impl Write) -> u8 {
    let dir = |at: usize| args.get(at).map_or(Path::new("."), |arg| Path::new(*arg));
    match command.action {
        Action::Status => status(dir(0), out, err),
        Action::New => new(dir(0), out, err),
        Action::Log => log(dir(0), out, err),
        Action::Undo => undo(dir(0), out, err),
        Action::Restore => match args.first().and_then(|op| op.to_str()) {
            Some(op) => restore(op, dir(1), out, err),
            None => {
                let _ = writeln!(err, "{}", command.usage());
                let _ = writeln!(
                    err,
                    "hint: an operation's id is hex digits, as `praps log` prints them"
                );
                2
            }
        },
        Action::Init => init(dir(0), out, err),
        Action::Fsck => fsck(dir(0), out, err),
        // The parser answers these two itself; they are here so that every
        // action has a body.
        Action::Help => {
            let _ = out.write_all(help::list().as_bytes());
            0
        }
        Action::Version => {
            let _ = writeln!(out, "praps {}", env!("CARGO_PKG_VERSION"));
            0
        }
    }
}

/// The first [`SHOWN`] hex digits of `bytes`.
fn short(bytes: &[u8]) -> String {
    let hex = hash::hex(bytes);
    hex[..SHOWN.min(hex.len())].to_owned()
}

fn log(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let (result, reported) = with_reports(dir, err, |ws| ws.log());
    let code = match result {
        Ok(entries) => {
            for entry in entries {
                let files = if entry.files == 1 { "file" } else { "files" };
                let _ = writeln!(
                    out,
                    "{} change {} {} {files}",
                    short(entry.id.digest()),
                    short(&entry.change_id),
                    entry.files
                );
            }
            0
        }
        Err(error) => fail("log", dir, error, err),
    };
    print_reported(&reported, out);
    code
}

fn undo(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let (result, reported) = with_reports(dir, err, |ws| ws.undo());
    let code = match result {
        Ok((restored, _)) => {
            let _ = writeln!(
                out,
                "undid the last op: the working copy is back to op {}",
                short(restored.digest())
            );
            0
        }
        Err(error) => fail("undo", dir, error, err),
    };
    print_reported(&reported, out);
    code
}

fn restore(op: &str, dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let (result, reported) = with_reports(dir, err, |ws| ws.restore(op));
    let code = match result {
        Ok((restored, _)) => {
            let _ = writeln!(
                out,
                "restored the working copy to op {}",
                short(restored.digest())
            );
            0
        }
        Err(error) => fail("restore", dir, error, err),
    };
    print_reported(&reported, out);
    code
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

/// Opens the repository at `dir` and runs `command` on it, then prints on
/// `err` each note the workspace left (`note: …`): what resuming an
/// interrupted write, or the write itself, left as it was.
fn with_workspace<T>(
    dir: &Path,
    err: &mut impl Write,
    command: impl FnOnce(&mut Workspace<RealFs>) -> Result<T, workspace::Error>,
) -> Result<T, workspace::Error> {
    let mut workspace = Workspace::open(RealFs, dir, os_seed())?;
    let result = command(&mut workspace);
    for note in workspace.notes() {
        let _ = writeln!(err, "note: {note}");
    }
    result
}

/// `with_workspace`, also returning what the command's snapshot reported
/// and left alone, whether the command succeeded or failed (INT-0004
/// AC-7). Nothing is reported when the repository did not open.
fn with_reports<T>(
    dir: &Path,
    err: &mut impl Write,
    command: impl FnOnce(&mut Workspace<RealFs>) -> Result<T, workspace::Error>,
) -> (Result<T, workspace::Error>, Vec<workspace::Reported>) {
    let mut reported = Vec::new();
    let result = with_workspace(dir, err, |ws| {
        let result = command(ws);
        reported = ws.reported().to_vec();
        result
    });
    (result, reported)
}

/// Prints each entry a snapshot reported and left alone, in `status`'s
/// format: `skipped PATH (REASON)`.
fn print_reported(reported: &[workspace::Reported], out: &mut impl Write) {
    for reported in reported {
        let _ = writeln!(
            out,
            "skipped {} ({})",
            reported.path,
            reported.reason.text()
        );
    }
}

fn new(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let (result, reported) = with_reports(dir, err, |ws| ws.new_change());
    let code = match result {
        Ok(change_id) => {
            let _ = writeln!(out, "new change {}", hash::hex(&change_id));
            0
        }
        Err(error) => fail("new", dir, error, err),
    };
    print_reported(&reported, out);
    code
}

fn status(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let result = with_workspace(dir, err, |ws| ws.status());
    match result {
        Ok(status) => {
            for (path, change) in &status.changes {
                // Paths are valid UTF-8 repository paths; they are shown in
                // the same escaped form as reported names.
                let shown = sim::Name::Utf8(path.as_str().to_owned()).display();
                let _ = writeln!(out, "{} {shown}", change.letter());
            }
            print_reported(&status.reported, out);
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
