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

use crate::hint::{self, Hint};
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
        Action::Describe => match args.first().map(|text| text_bytes(text)) {
            Some(Some(text)) => describe(&text, args, out, err),
            // Only Windows can pass text that has no UTF-8 form.
            Some(None) => usage_error(command, &hint::unicode_text(), err),
            // The parser has counted the arguments; this keeps the match
            // whole.
            None => usage_error(command, &hint::usage(command), err),
        },
        Action::New => new(dir(0), out, err),
        Action::Log => log(dir(0), out, err),
        Action::Ops => ops(dir(0), out, err),
        Action::Undo => undo(dir(0), out, err),
        Action::Restore => match args.first().and_then(|op| op.to_str()) {
            Some(op) => restore(op, dir(1), out, err),
            None => usage_error(command, &hint::operation_ids(), err),
        },
        Action::Init => init(dir(0), out, err),
        Action::Lane => match dir(0) {
            Some(dir) => lane(dir, out, err),
            // The parser has counted the arguments; this keeps the match
            // whole.
            None => usage_error(command, &hint::usage(command), err),
        },
        Action::Lanes => lanes(dir(0), out, err),
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

/// Prints `command`'s usage line and then the hint, and returns exit code 2.
/// Every usage error that names a command is printed here, so each ends
/// with exactly one hint line (INT-0037 AC-4).
pub fn usage_error(command: &Command, hint: &Hint, err: &mut impl Write) -> u8 {
    let _ = writeln!(err, "{}", command.usage());
    let _ = writeln!(err, "{hint}");
    2
}

/// The bytes of a description given on the command line. On Unix they are
/// the argument's bytes exactly as given, whatever they are. Elsewhere they
/// are its UTF-8 form, and text that has none (on Windows, text holding an
/// unpaired surrogate) gives `None`: the bytes the standard library holds
/// for such text are its own, and nothing to store.
#[cfg(unix)]
fn text_bytes(text: &OsStr) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    Some(text.as_bytes().to_vec())
}

#[cfg(not(unix))]
fn text_bytes(text: &OsStr) -> Option<Vec<u8>> {
    text.to_str().map(|text| text.as_bytes().to_vec())
}

/// The first [`SHOWN`] hex digits of `bytes`.
fn short(bytes: &[u8]) -> String {
    let hex = hash::hex(bytes);
    hex[..SHOWN.min(hex.len())].to_owned()
}

/// Prints why `command` could not do what was asked and then the hint, and
/// returns exit code 1. Every refusal with that exit code is printed here,
/// so each ends with exactly one hint line (INT-0037 AC-4).
fn refuse(command: &str, why: impl Display, hint: &Hint, err: &mut impl Write) -> u8 {
    let _ = writeln!(err, "praps {command}: {why}");
    let _ = writeln!(err, "{hint}");
    1
}

/// Prints an error from the working copy at `place`, then the hint for it,
/// and returns exit code 1.
fn fail(command: &str, place: &str, error: &workspace::Error, err: &mut impl Write) -> u8 {
    // A command opens the working copy as `.` from its top, so an unfinished
    // lane would be named `.`; it is named by the place instead, so that the
    // hint's `praps lane DIR` works from the working copy it was made from
    // (Sprint 7 code review, finding 5).
    let named;
    let error = match error {
        workspace::Error::UnfinishedLane(name, _) => {
            named = workspace::Error::UnfinishedLane(name.clone(), PathBuf::from(place));
            &named
        }
        other => other,
    };
    refuse(
        command,
        format_args!("{place}: {}", hint::said(error)),
        &hint::for_error(error),
        err,
    )
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
            &hint::current_directory(),
            err,
        )),
    }
}

/// Refuses a `dir` that is not there, or is not a directory, with the hint
/// `missing`.
fn must_exist(command: &str, dir: &Path, missing: &Hint, err: &mut impl Write) -> Result<(), u8> {
    match RealFs.list(dir) {
        Ok(_) => Ok(()),
        Err(error) => {
            let (why, hint) = match error.kind() {
                ErrorKind::NotFound => ("no such directory".to_owned(), missing.clone()),
                ErrorKind::NotADirectory => ("not a directory".to_owned(), missing.clone()),
                _ => (error.to_string(), hint::unreadable_directory()),
            };
            Err(refuse(
                command,
                format_args!("{}: {why}", dir.display()),
                &hint,
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
            &hint::unreadable_above(),
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
    locate_or(command, given, &hint::missing_directory(), err)
}

/// [`locate`], with the hint to give when the directory given is not there
/// or is not a directory.
fn locate_or(
    command: &str,
    given: Option<&Path>,
    missing: &Hint,
    err: &mut impl Write,
) -> Result<Place, u8> {
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
            must_exist(command, dir, missing, err)?;
            let Some(top) = top_above(command, &full, err)? else {
                return Err(refuse(
                    command,
                    format_args!("{named}: no Periapsis repository here or in any directory above"),
                    &hint::no_repository(),
                    err,
                ));
            };
            if let Err(error) = std::env::set_current_dir(&top) {
                return Err(refuse(
                    command,
                    format_args!("{}: {error}", top.display()),
                    &hint::cannot_enter(),
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
            &hint::unreadable_directory(),
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

/// `count` paths, as a listing prints it: `1 path`, `2 paths`.
fn paths(count: usize) -> String {
    format!("{count} {}", if count == 1 { "path" } else { "paths" })
}

/// The first line of a change's description as a listing shows it: escaped
/// as names are, so that it stays one line whatever bytes it holds, or
/// `(no description)` for a change that has none.
fn first_line(description: &[u8]) -> String {
    if description.is_empty() {
        return "(no description)".to_owned();
    }
    let line = description
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default();
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    sim::Name::Raw(sim::RawName::Bytes(line.to_vec())).display()
}

/// `log`: the working-copy change and the changes before it, newest first,
/// one line each: `CHANGE N paths FIRST LINE`.
fn log(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let place = match locate("log", given, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.changes());
    let code = match result {
        Ok(entries) => {
            for entry in entries {
                let _ = writeln!(
                    out,
                    "{} {} {}",
                    short(&entry.change_id),
                    paths(entry.paths),
                    first_line(&entry.description)
                );
            }
            0
        }
        Err(error) => fail("log", &place.shown, &error, err),
    };
    print_reported(&reported, out);
    code
}

/// `ops`: every operation, newest first, one line each:
/// `OP change CHANGE N paths`.
fn ops(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let place = match locate("ops", given, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.ops());
    let code = match result {
        Ok(entries) => {
            for entry in entries {
                let _ = writeln!(
                    out,
                    "{} {} change {} {}",
                    short(entry.id.digest()),
                    entry.workspace,
                    short(&entry.change_id),
                    paths(entry.paths)
                );
            }
            0
        }
        Err(error) => fail("ops", &place.shown, &error, err),
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
            // One op, as before; several, every one in id order.
            let ops: Vec<String> = restored.iter().map(|op| short(op.digest())).collect();
            let _ = match ops.as_slice() {
                [one] => writeln!(
                    out,
                    "undid the last op: the working copy is back to op {one}"
                ),
                _ => writeln!(
                    out,
                    "undid the last op: the working copy is back to ops {}",
                    ops.join(", ")
                ),
            };
            0
        }
        Err(error) => fail("undo", &place.shown, &error, err),
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
        Err(error) => fail("restore", &place.shown, &error, err),
    };
    print_reported(&reported, out);
    code
}

/// `describe`: sets the description of the working-copy change to `text`,
/// byte for byte, and says so in one line: `described change CHANGE: FIRST
/// LINE`. With the description the change already has, it says that nothing
/// changed.
///
/// `words` are the arguments as given: the text, then the directory if one
/// was given. Two words of which the second is no directory are most likely
/// one text without its quotes, so that refusal's hint shows them as one.
fn describe(text: &[u8], words: &[&OsStr], out: &mut impl Write, err: &mut impl Write) -> u8 {
    let given = words.get(1).map(|dir| Path::new(*dir));
    let missing = given
        .and_then(|_| hint::quoted(words, "if both words are the text"))
        .unwrap_or_else(hint::missing_directory);
    let place = match locate_or("describe", given, &missing, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.describe(text));
    let code = match result {
        Ok(described) => {
            let change = short(&described.change_id);
            let _ = match described.op {
                Some(_) => writeln!(out, "described change {change}: {}", first_line(text)),
                None => writeln!(
                    out,
                    "nothing changed: change {change} already has that description"
                ),
            };
            0
        }
        Err(error) => fail("describe", &place.shown, &error, err),
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
        Err(error) => fail("new", &place.shown, &error, err),
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
        Err(error) => fail("status", &place.shown, &error, err),
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
    // A lane's own top: a repository is not made in a lane either. A lane
    // file that cannot be read is no proof there is none (Sprint 7 code
    // review, finding 6).
    let lane_file = dir.join(PRAPS_DIR).join(workspace::lane::WORKSPACE_FILE);
    match RealFs.read(&lane_file) {
        Ok(_) => {
            return refuse(
                "init",
                format_args!(
                    "{}: it is a lane's working copy, and a repository is not made inside another",
                    dir.display()
                ),
                &hint::inside_a_working_copy(),
                err,
            );
        }
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {}
        Err(error) => {
            return refuse(
                "init",
                format_args!("{}: {error}", lane_file.display()),
                &hint::unreadable_file(),
                err,
            );
        }
    }
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
                &hint::inside_a_working_copy(),
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
        Err(error) => {
            // A failure of the filesystem leaves an `init` that can be run
            // again; any other error is one of the repository's.
            let hint = match &error {
                workspace::Error::Io(_) => hint::init_again(),
                other => hint::for_error(other),
            };
            refuse(
                "init",
                format_args!("{}: {}", dir.display(), hint::said(&error)),
                &hint,
                err,
            )
        }
    }
}

/// The store `verify` checks for `dir`, and how a refusal names where it
/// looked. The first of:
/// 1. `DIR/.praps/store`, if DIR is a top, whatever state that store is in,
///    so damage there is reported rather than hidden (code review CR-09);
/// 2. DIR itself, if it holds `objects` and `ops`, whatever lies above it;
/// 3. the store of the nearest top above DIR;
/// 4. DIR itself, which then fails as not a store.
///
/// The last value says whether the store is a working copy's (rules 1 and
/// 3) or a directory taken as a store itself (rules 2 and 4).
fn store_for(given: Option<&Path>, err: &mut impl Write) -> Result<(PathBuf, String, bool), u8> {
    let dir = given.unwrap_or(Path::new("."));
    let full = full_path("verify", dir, err)?;
    let named = given.map_or_else(
        || full.display().to_string(),
        |dir| dir.display().to_string(),
    );
    match place::is_top(&RealFs, dir) {
        Ok(true) => return Ok((store_of(dir), named, true)),
        Ok(false) => {}
        Err(error) => {
            return Err(refuse(
                "verify",
                format_args!("{}: {error}", dir.join(PRAPS_DIR).join(STORE_DIR).display()),
                &hint::unreadable_directory(),
                err,
            ));
        }
    }
    match Store::inspect(RealFs, dir, HashAlg::DEFAULT) {
        Ok(_) => return Ok((dir.to_path_buf(), named, false)),
        // Not a store: the search goes on.
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {}
        // Whether it is one could not be read. Going on would check, and
        // perhaps call clean, a store that was not asked about (code
        // review, finding 1).
        Err(error) => {
            return Err(refuse(
                "verify",
                format_args!("{named}: {error}"),
                &hint::unreadable_store(),
                err,
            ));
        }
    }
    must_exist("verify", dir, &hint::store_or_working_copy(), err)?;
    match top_above("verify", &full, err)? {
        Some(top) => Ok((store_of(&top), top.display().to_string(), true)),
        None => Ok((dir.to_path_buf(), named, false)),
    }
}

/// The store a working copy at `top` uses: its own `.praps/store`, or, for
/// a lane, the one its `.praps/workspace` names.
fn store_of(top: &Path) -> PathBuf {
    let own = top.join(PRAPS_DIR).join(STORE_DIR);
    if RealFs.list(&own).is_ok() {
        return own;
    }
    match workspace::lane::read(&RealFs, &top.join(PRAPS_DIR)) {
        Ok(Some(Ok(file))) => file.store,
        _ => own,
    }
}

fn lane(dir: &Path, out: &mut impl Write, err: &mut impl Write) -> u8 {
    // DIR is read from where `praps` was started, before `locate` moves to
    // the working copy's top.
    let full = match full_path("lane", dir, err) {
        Ok(full) => full,
        Err(code) => return code,
    };
    let Some(name) = full
        .file_name()
        .and_then(|name| name.to_str())
        // A control character, such as a line break, is not plain text, and
        // `.praps/workspace` holds the name on a line (Sprint 7 code review,
        // finding 4).
        .filter(|name| {
            !name.is_empty() && *name != "." && *name != ".." && !name.chars().any(char::is_control)
        })
        .map(str::to_owned)
    else {
        return refuse(
            "lane",
            format_args!(
                "{}: a lane is named after its directory's last name, and this one has none \
                 that is plain text",
                dir.display()
            ),
            &hint::lane_name(),
            err,
        );
    };
    // From DIR's parent up: DIR itself only has to be free, which the
    // workspace checks, and an interrupted lane there is made again.
    match top_above("lane", &full, err) {
        Ok(None) => {}
        Ok(Some(top)) => {
            return refuse(
                "lane",
                format_args!(
                    "{}: it lies inside the working copy at {}, which would record the lane's \
                     files as its own",
                    dir.display(),
                    top.display()
                ),
                &hint::lane_outside(),
                err,
            );
        }
        Err(code) => return code,
    }
    let place = match locate("lane", None, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    // The lane names the store by its full path, so the working copy is
    // opened by its full path too.
    let root = match full_path("lane", &place.root, err) {
        Ok(root) => root,
        Err(code) => return code,
    };
    let place = Place {
        root,
        shown: place.shown,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.make_lane(&name, &full));
    let code = match result {
        Ok((_, change_id)) => {
            let _ = writeln!(
                out,
                "made lane {name} at {}, on change {}",
                dir.display(),
                short(&change_id)
            );
            0
        }
        Err(error) => fail("lane", &place.shown, &error, err),
    };
    print_reported(&reported, out);
    code
}

fn lanes(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let place = match locate("lanes", given, err) {
        Ok(place) => place,
        Err(code) => return code,
    };
    let (result, reported) = with_reports(&place, err, |ws| ws.lanes());
    let code = match result {
        Ok(lanes) => {
            for lane in lanes {
                let others: Vec<String> = lane.others.iter().map(|id| short(id)).collect();
                let _ = writeln!(
                    out,
                    "{} {} change {} {}{}",
                    if lane.this { '*' } else { ' ' },
                    lane.name,
                    short(&lane.change_id),
                    paths(lane.paths),
                    if others.is_empty() {
                        String::new()
                    } else {
                        format!(", also changes {}", others.join(", "))
                    }
                );
            }
            0
        }
        Err(error) => fail("lanes", &place.shown, &error, err),
    };
    print_reported(&reported, out);
    code
}

fn verify(given: Option<&Path>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let (store_dir, shown, of_working_copy) = match store_for(given, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    // `inspect`, not `open`: checking only reads, so it works on a read-only
    // store and changes nothing.
    let report = match Store::inspect(RealFs, &store_dir, HashAlg::DEFAULT).and_then(|s| s.fsck()) {
        Ok(report) => report,
        Err(error) => {
            // A directory that is no store is one to make or to name
            // rightly; a working copy's store that is no store was left by
            // an interrupted `init`, or has lost its directories.
            let hint = match (error.kind() == ErrorKind::NotFound, of_working_copy) {
                (true, false) => hint::not_a_store(),
                (true, true) => hint::unfinished_store(),
                (false, _) => hint::unreadable_store(),
            };
            return refuse("verify", format_args!("{shown}: {error}"), &hint, err);
        }
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
        let _ = writeln!(err, "{}", hint::damaged_store());
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A description's first line is shown alone, escaped as names are, and
    /// a change with none says so (Sprint 6 T-009).
    #[test]
    fn first_line_is_escaped() {
        assert_eq!(first_line(b""), "(no description)");
        assert_eq!(first_line(b"Add the parser"), "Add the parser");
        assert_eq!(
            first_line(b"Add the parser\n\nIt reads one line at a time.\n"),
            "Add the parser"
        );
        assert_eq!(first_line(b"one\r\ntwo"), "one");
        // A blank first line is shown as it is: the change has a description.
        assert_eq!(first_line(b"\nbody"), "");
        // Control characters and bytes that are not UTF-8 cannot break the
        // line or the terminal.
        assert_eq!(first_line(b"tab\there\x1b[31m"), "tab\\u{9}here\\u{1b}[31m");
        assert_eq!(first_line(b"caf\xc3\xa9 \xff\xfe"), "caf\u{e9} \\xff\\xfe");
        assert_eq!(first_line(b"back\\slash"), "back\\\\slash");
        assert_eq!(paths(0), "0 paths");
        assert_eq!(paths(1), "1 path");
        assert_eq!(paths(2), "2 paths");
    }
}
