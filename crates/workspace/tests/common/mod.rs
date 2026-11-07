//! Helpers shared by the workspace tests.
#![allow(dead_code)]

use std::io::ErrorKind;
use std::path::Path;

use sim::Fs;

pub const ROOT: &str = "repo";

/// Writes `content` to the file at `path` under the repository, creating
/// its directories and the file as needed.
pub fn put(fs: &impl Fs, path: &str, content: &[u8]) {
    let mut dir = Path::new(ROOT).to_path_buf();
    let mut parts: Vec<&str> = path.split('/').collect();
    let name = parts.pop().unwrap();
    for part in parts {
        dir = dir.join(part);
        match fs.create_dir(&dir) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
    let file = dir.join(name);
    match fs.create_new(&file) {
        Ok(()) => fs.write(&file, content).unwrap(),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            fs.overwrite(&file, content).unwrap();
        }
        Err(error) => panic!("{}: {error}", file.display()),
    }
}

/// Removes the file at `path` under the repository.
pub fn delete(fs: &impl Fs, path: &str) {
    let file = path
        .split('/')
        .fold(Path::new(ROOT).to_path_buf(), |p, c| p.join(c));
    fs.remove_file(&file).unwrap();
}

/// The syscalls that write, in a trace.
pub fn writes(trace: &[String]) -> Vec<&String> {
    trace
        .iter()
        .filter(|line| {
            [
                "create_dir ",
                "create_new ",
                "write ",
                "overwrite ",
                "rename ",
                "remove_file ",
                "sync_file ",
                "sync_dir ",
            ]
            .iter()
            .any(|op| line.starts_with(op))
        })
        .collect()
}
