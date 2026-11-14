//! Lanes: working copies over one store (INT-0014; Sprint 7 T-006).
//!
//! A lane is a workspace whose working copy has a `.praps` of its own that
//! holds no store, only `.praps/workspace`: two lines naming the workspace
//! and the store it shares, `name NAME` and `store PATH`. So a lane is no
//! clone: every op and object it makes goes into the one store.
//! `docs/spec/working-copy.md` ("Lanes") is the specification.

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use sim::Fs;

/// The file in `.praps` that makes a directory a lane's top.
pub const WORKSPACE_FILE: &str = "workspace";
/// The name it is written under before the rename.
pub const WORKSPACE_FILE_NEW: &str = "workspace.new";

/// What `.praps/workspace` says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneFile {
    /// The workspace the lane is.
    pub name: String,
    /// The directory of the store it shares.
    pub store: PathBuf,
}

impl LaneFile {
    /// The file's bytes: `name NAME` and `store PATH`, one per line.
    pub fn encode(&self) -> Vec<u8> {
        format!("name {}\nstore {}\n", self.name, self.store.display()).into_bytes()
    }

    /// Reads a file back; the error says what is wrong with it.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "it is not UTF-8".to_owned())?;
        let mut lines = text.split_terminator('\n');
        let mut field = |name: &str| -> Result<String, String> {
            let line = lines
                .next()
                .ok_or_else(|| format!("it has no `{name}` line"))?;
            let value = line
                .strip_prefix(name)
                .and_then(|rest| rest.strip_prefix(' '))
                .ok_or_else(|| format!("`{line}` is not a `{name}` line"))?;
            if value.is_empty() {
                return Err(format!("its `{name}` is empty"));
            }
            Ok(value.to_owned())
        };
        let file = LaneFile {
            name: field("name")?,
            store: PathBuf::from(field("store")?),
        };
        match lines.next() {
            None if text.ends_with('\n') => Ok(file),
            None => Err("its last line is not ended".to_owned()),
            Some(line) => Err(format!("`{line}` follows the store")),
        }
    }
}

/// Writes `file` durably into `praps`: under [`WORKSPACE_FILE_NEW`]
/// (replacing any left over), synced, renamed to [`WORKSPACE_FILE`], then
/// `praps` synced. With `in_place` (the negative control
/// `Variant::LaneFileInPlace`), it writes the file under its own name, with
/// no temporary name and no sync.
pub fn write(fs: &impl Fs, praps: &Path, file: &LaneFile, in_place: bool) -> io::Result<()> {
    if in_place {
        let path = praps.join(WORKSPACE_FILE);
        fs.create_new(&path)?;
        return fs.write(&path, &file.encode());
    }
    let new = praps.join(WORKSPACE_FILE_NEW);
    match fs.remove_file(&new) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    fs.create_new(&new)?;
    fs.write(&new, &file.encode())?;
    fs.sync_file(&new)?;
    fs.rename(&new, &praps.join(WORKSPACE_FILE))?;
    fs.sync_dir(praps)
}

/// The lane file in `praps`, if there is one: `Ok(None)` when there is none,
/// and the inner error when it does not decode.
pub fn read(fs: &impl Fs, praps: &Path) -> io::Result<Option<Result<LaneFile, String>>> {
    match fs.read(&praps.join(WORKSPACE_FILE)) {
        Ok(bytes) => Ok(Some(LaneFile::decode(&bytes))),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_files_round_trip() {
        let file = LaneFile {
            name: "agent-17".to_owned(),
            store: PathBuf::from("repo/.praps/store"),
        };
        assert_eq!(
            String::from_utf8(file.encode()).unwrap(),
            "name agent-17\nstore repo/.praps/store\n"
        );
        assert_eq!(LaneFile::decode(&file.encode()), Ok(file));
        for bad in [
            &b""[..],
            b"name a\n",
            b"name \nstore s\n",
            b"store s\nname a\n",
            b"name a\nstore s",
            b"name a\nstore s\nmore\n",
            b"\xff",
        ] {
            assert!(LaneFile::decode(bad).is_err(), "{bad:?}");
        }
    }
}
