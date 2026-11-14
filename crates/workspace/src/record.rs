//! The working-copy record: `.praps/working-copy`, which exists only while
//! a write of the working copy is in flight (INT-0009, INT-0014).
//!
//! It names what was on disk when the write began (`from`): the working-copy
//! changeset, or `empty` for the empty tree; and the op whose tree is being
//! written (`target`). A merged state has no single op, so `from` names a
//! changeset (Sprint 7). It is written durably before the target op is
//! committed, so a crash at any point leaves either no target op, or a
//! record that the next command uses to finish the write.
//! `docs/spec/working-copy.md` ("The record") is the specification.

use std::io::{self, ErrorKind};
use std::path::Path;

use hash::HashRef;
use sim::Fs;

/// The record's name in `.praps`.
pub const RECORD: &str = "working-copy";
/// The name it is written under before the rename.
pub const RECORD_NEW: &str = "working-copy.new";
/// What `from` says for the empty tree.
const EMPTY: &str = "empty";

/// An in-flight write of the working copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    /// The working-copy changeset whose tree was on disk, or `None` for the
    /// empty tree.
    pub from: Option<HashRef>,
    /// The op whose working-copy tree is being written.
    pub target: HashRef,
}

impl Record {
    /// The record's bytes: `from <changeset>` or `from empty`, then
    /// `target <op>`, one per line.
    pub fn encode(&self) -> Vec<u8> {
        let from = match &self.from {
            Some(changeset) => changeset.to_string(),
            None => EMPTY.to_owned(),
        };
        format!("from {from}\ntarget {}\n", self.target).into_bytes()
    }

    /// Reads a record back; the error says what is wrong with it.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "it is not UTF-8".to_owned())?;
        let mut lines = text.split_terminator('\n');
        let mut field = |name: &str| -> Result<&str, String> {
            let line = lines
                .next()
                .ok_or_else(|| format!("it has no `{name}` line"))?;
            line.strip_prefix(name)
                .and_then(|rest| rest.strip_prefix(' '))
                .ok_or_else(|| format!("`{line}` is not a `{name}` line"))
        };
        let id = |value: &str| -> Result<HashRef, String> {
            value
                .parse()
                .map_err(|error| format!("`{value}` is not an id: {error:?}"))
        };
        let from = match field("from")? {
            EMPTY => None,
            value => Some(id(value)?),
        };
        let record = Record {
            from,
            target: id(field("target")?)?,
        };
        match lines.next() {
            None if text.ends_with('\n') => Ok(record),
            None => Err("its last line is not ended".to_owned()),
            Some(line) => Err(format!("`{line}` follows the record")),
        }
    }
}

/// Writes `record` durably into `praps`: under [`RECORD_NEW`] (replacing any
/// left over), synced, renamed to [`RECORD`], then `praps` synced.
pub fn write(fs: &impl Fs, praps: &Path, record: &Record) -> io::Result<()> {
    let new = praps.join(RECORD_NEW);
    match fs.remove_file(&new) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    fs.create_new(&new)?;
    fs.write(&new, &record.encode())?;
    fs.sync_file(&new)?;
    fs.rename(&new, &praps.join(RECORD))?;
    fs.sync_dir(praps)?;
    fs.point("workspace.record.written");
    Ok(())
}

/// The record in `praps`, if there is one: `Ok(None)` when there is none,
/// and the inner error when it does not decode.
pub fn read(fs: &impl Fs, praps: &Path) -> io::Result<Option<Result<Record, String>>> {
    match fs.read(&praps.join(RECORD)) {
        Ok(bytes) => Ok(Some(Record::decode(&bytes))),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Drops the record: removes it, then syncs `praps`.
pub fn drop_record(fs: &impl Fs, praps: &Path) -> io::Result<()> {
    match fs.remove_file(&praps.join(RECORD)) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    fs.sync_dir(praps)?;
    fs.point("workspace.record.dropped");
    Ok(())
}
