//! The working-copy record: `.praps/working-copy`, which exists only while
//! a write of the working copy is in flight (INT-0009, INT-0014).
//!
//! It names the op whose working-copy tree was on disk (`from`) and the op
//! whose tree is being written (`target`). It is written durably before the
//! target op is committed, so a crash at any point leaves either no target
//! op, or a record that the next command uses to finish the write.
//! `docs/spec/working-copy.md` ("The record") is the specification.

use std::io::{self, ErrorKind};
use std::path::Path;

use hash::HashRef;
use sim::Fs;

/// The record's name in `.praps`.
pub const RECORD: &str = "working-copy";
/// The name it is written under before the rename.
pub const RECORD_NEW: &str = "working-copy.new";

/// An in-flight write of the working copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    /// The op whose working-copy tree was on disk.
    pub from: HashRef,
    /// The op whose working-copy tree is being written.
    pub target: HashRef,
}

impl Record {
    /// The record's bytes: `from <op>` and `target <op>`, one per line.
    pub fn encode(&self) -> Vec<u8> {
        format!("from {}\ntarget {}\n", self.from, self.target).into_bytes()
    }

    /// Reads a record back; the error says what is wrong with it.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "it is not UTF-8".to_owned())?;
        let mut lines = text.split_terminator('\n');
        let mut field = |name: &str| -> Result<HashRef, String> {
            let line = lines
                .next()
                .ok_or_else(|| format!("it has no `{name}` line"))?;
            let value = line
                .strip_prefix(name)
                .and_then(|rest| rest.strip_prefix(' '))
                .ok_or_else(|| format!("`{line}` is not a `{name}` line"))?;
            value
                .parse()
                .map_err(|error| format!("`{value}` is not an op id: {error:?}"))
        };
        let record = Record {
            from: field("from")?,
            target: field("target")?,
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
