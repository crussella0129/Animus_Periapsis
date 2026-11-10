//! What each command does. [`run`] is the one place a registry entry
//! becomes an action, by an exhaustive `match`, so a command the registry
//! holds cannot be left without a body.

use std::ffi::OsStr;
use std::fmt::Display;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use hash::HashAlg;
use sim::{Fs, RealFs, os_seed};
use store::Store;
use workspace::{PRAPS_DIR, STORE_DIR, Workspace};

use crate::registry::{Action, Command};
use crate::{help, place};

/// How many hex digits of an op id or change ID `praps` prints.
const SHOWN: usize = 12;

/// Runs `command` with `args`, which the caller has counted against the
/// registry entry. The working copy's directory is the last argument, and
/// the current directory when it is left out.
pub fn run(command: &Command, args: &[&OsStr], out: &mut impl Write, err: &mut impl Write) -> u8 {
    let dir = |at: usize| args.get(at).map(|arg| Path::new(*arg));
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
        Action::Verify => verify(dir(0), out, err),
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

/// Prints why `command` could not do what was asked, and returns exit code
/// 1.
fn refuse(command: &str, why: impl Display, err: &mut impl Write) -> u8 {
    let _ = writeln!(err, "praps {command}: {why}");
    1
}

/// Prints an error from the working copy or store at `place`, and returns
/// exit code 1.
fn fail(command: &str, place: &str, error: impl Display, err: &mut impl Write) -> u8 {
    refuse(command, format_args!("{place}: {error}"), err)
}

// ------------------------------------------------ finding the working copy

/// Where a command that opens a working copy runs (INT-0037 AC-3).
struct Place {
    /// The top of the working copy, as the workspace is given it.
    root: PathBuf,
    /// How a refusal names the working copy: the directory as given, when
    /// one was given and it is the top; the top's full path otherwise.
    shown: String,
}

/// `dir` as a full path, taken from the current directory as written. The
/// current directory is read only when `dir` is not already a full path;
/// a current directory that cannot be read is a refusal.
fn full_path(command: &str, dir: &Path, err: &mut impl Write) -> Result<PathBuf, u8> {
    if dir.is_absolute() {
        return Ok(place::full_path(Path::new(""), dir));
    }
    match std::env::current_dir() {
        Ok(cwd) => Ok(place::full_path(&cwd, dir)),
        Err(error) => Err(refuse(
            command,
            format_args!("the current directory cannot be read: {error}"),
            err,
        )),
    }
}

/// Refuses a `dir` that is not there, or is not a directory.
fn must_exist(command: &str, dir: &Path, err: &mut impl Write) -> Result<(), u8> {
    match RealFs.list(dir) {
        Ok(_) => Ok(()),
        Err(error) => {
            let why = match error.kind() {
                ErrorKind::NotFound => "no such directory".to_owned(),
                ErrorKind::NotADirectory => "not a directory".to_owned(),
                _ => error.to_string(),
            };
            Err(refuse(
                command,
                format_args!("{}: {why}", dir.display()),
                err,
            ))
        }
    }
}

/// The nearest top above `full`, if there is one. A `.praps/store` that
/// cannot be listed for a reason other than its absence is a refusal: it is
/// never passed over for a repository further up.
fn top_above(command: &str, full: &Path, err: &mut impl Write) -> Result<Option<PathBuf>, u8> {
    let above = place::above(full);
    match place::nearest_top(&RealFs, &above) {
        Ok(found) => Ok(found.map(|at| above[at].clone())),
        Err(error) => Err(refuse(
            command,
            format_args!(
                "{}: a directory above it could not be searched for a repository: {error}",
                full.display()
            ),
            err,
        )),
    }
}

/// Finds the working copy a command acts on: the nearest top among `given`
/// (the current directory when none is given) and the directories above it.
///
/// When the top is above the starting directory, `praps` makes the top its
/// current directory and runs as if started there, so what is written and
/// synced is exactly what a run from the top writes and syncs.
fn locate(command: &str, given: Option<&Path>, err: &mut impl Write) -> Result<Place, u8> {
    let dir = given.unwrap_or(Path::new("."));
    let full = full_path(command, dir, err)?;
    // How the directory asked about is named: as given, or by its full path.
    let named = given.map_or_else(
        || full.display().to_string(),
        |dir| dir.display().to_string(),
    );
    match place::is_top(&RealFs, dir) {
        Ok(true) => Ok(Place {
            root: dir.to_path_buf(),
            shown: named,
        }),
        Ok(false) => {
            must_exist(command, dir, err)?;
            let Some(top) = top_above(command, &full, err)? else {
                return Err(refuse(
                    command,
                    format_args!("{named}: no Periapsis repository here or in any directory above"),
                    err,
                ));
            };
            if let Err(error) = std::env::set_current_dir(&top) {
                return Err(refuse(
                    command,
                    format_args!("{}: {error}", top.display()),
                    err,
                ));
            }
            Ok(Place {
                root: PathBuf::from("."),
                shown: top.display().to_string(),
            })
        }
        Err(error) => Err(refuse(
            command,
            format_args!("{}: {error}", dir.join(PRAPS_DIR).join(STORE_DIR).display()),
            err,
        )),
    }
}

/// Opens the working copy at `place` and runs `command` on it, then prints
/// on `err` each note the workspace left (`note: …`): what resuming an
/// interrupted write, or the write itself, left as it was.
fn with_workspace<T>(
    place: &Place,
    err: &mut impl Write,
    command: impl FnOnce(&mut Workspace<RealFs>) -> Result<T, workspace::Error>,
) -> Result<T, workspace::Error> {
    let mut workspace = Workspace::open(RealFs, &place.root, os_seed())?;
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
    place: &Place,
    err: &mut impl Write,
    command: impl FnOnce(&mut Workspace<RealFs>) -> Result<T, workspace::Error>,
) -> (Result<T, workspace::Error>, Vec<workspace::Reported>) {
    let mut reported = Vec::new();
    let result = with_workspace(place, err, |ws| {
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

// ------------------------------------------------------------ the commands

fn log(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let place = match locate("log", given, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.log());
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
        Err(error) => fail("log", &place.shown, error, err),
    };
    print_reported(&reported, out);
    code
}

fn undo(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let place = match locate("undo", given, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.undo());
    let code = match result {
        Ok((restored, _)) => {
            let _ = writeln!(
                out,
                "undid the last op: the working copy is back to op {}",
                short(restored.digest())
            );
            0
        }
        Err(error) => fail("undo", &place.shown, error, err),
    };
    print_reported(&reported, out);
    code
}

fn restore(op: &str, given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let place = match locate("restore", given, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.restore(op));
    let code = match result {
        Ok((restored, _)) => {
            let _ = writeln!(
                out,
                "restored the working copy to op {}",
                short(restored.digest())
            );
            0
        }
        Err(error) => fail("restore", &place.shown, error, err),
    };
    print_reported(&reported, out);
    code
}

fn new(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let place = match locate("new", given, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.new_change());
    let code = match result {
        Ok(change_id) => {
            let _ = writeln!(out, "new change {}", hash::hex(&change_id));
            0
        }
        Err(error) => fail("new", &place.shown, error, err),
    };
    print_reported(&reported, out);
    code
}

fn status(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let place = match locate("status", given, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let result = with_workspace(&place, err, |ws| ws.status());
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
        Err(error) => fail("status", &place.shown, error, err),
    }
}

/// `init` makes `given` (the current directory when none is given) a
/// repository, or completes one whose `init` was interrupted, or says that
/// it already is one. Inside a working copy it does none of these: with a
/// top above the directory it writes nothing, whatever the directory holds
/// (INT-0037 AC-3).
fn init(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let dir = given.unwrap_or(Path::new("."));
    let full = match full_path("init", dir, err) {
        Ok(full) => full,
        Err(code) => return code,
    };
    match top_above("init", &full, err) {
        Ok(None) => {}
        Ok(Some(top)) => {
            return refuse(
                "init",
                format_args!(
                    "{}: it lies inside the working copy at {}, and a repository is not made inside another",
                    dir.display(),
                    top.display()
                ),
                err,
            );
        }
        Err(code) => return code,
    }
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
        Err(error) => fail("init", &dir.display().to_string(), error, err),
    }
}

/// The store `verify` checks for `dir`, and how a refusal names where it
/// looked. The first of:
/// 1. `DIR/.praps/store`, if DIR is a top, whatever state that store is in,
///    so damage there is reported rather than hidden (code review CR-09);
/// 2. DIR itself, if it holds `objects` and `ops`, whatever lies above it;
/// 3. the store of the nearest top above DIR;
/// 4. DIR itself, which then fails as not a store.
fn store_for(given: Option<&Path>, err: &mut impl Write) -> Result<(PathBuf, String), u8> {
    let dir = given.unwrap_or(Path::new("."));
    let full = full_path("verify", dir, err)?;
    let named = given.map_or_else(
        || full.display().to_string(),
        |dir| dir.display().to_string(),
    );
    match place::is_top(&RealFs, dir) {
        Ok(true) => return Ok((dir.join(PRAPS_DIR).join(STORE_DIR), named)),
        Ok(false) => {}
        Err(error) => {
            return Err(refuse(
                "verify",
                format_args!("{}: {error}", dir.join(PRAPS_DIR).join(STORE_DIR).display()),
                err,
            ));
        }
    }
    if Store::inspect(RealFs, dir, HashAlg::DEFAULT).is_ok() {
        return Ok((dir.to_path_buf(), named));
    }
    must_exist("verify", dir, err)?;
    match top_above("verify", &full, err)? {
        Some(top) => Ok((
            top.join(PRAPS_DIR).join(STORE_DIR),
            top.display().to_string(),
        )),
        None => Ok((dir.to_path_buf(), named)),
    }
}

fn verify(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let (store_dir, shown) = match store_for(given, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    // `inspect`, not `open`: checking only reads, so it works on a read-only
    // store and changes nothing.
    let report = match Store::inspect(RealFs, &store_dir, HashAlg::DEFAULT).and_then(|s| s.fsck()) {
        Ok(report) => report,
        Err(error) => return fail("verify", &shown, error, err),
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
