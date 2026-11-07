//! Repository paths (`docs/spec/format-v1.md`, "path"; INT-0004 AC-2).
//!
//! A repository path is relative, `/`-separated and UTF-8. It has at least
//! one component; no component is empty, `.` or `..`; and it holds no NUL and
//! no `\`. It is encoded as a `string`, so its bytes are the same on every
//! host, and it orders by its UTF-8 bytes like any string.

use std::fmt;

use crate::error::DecodeError;
use crate::wire::{Decode, Decoder, Encode, Encoder};

/// A valid repository path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoPath(String);

/// Why a string is not a repository path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathError {
    /// The path is empty.
    Empty,
    /// A component is empty: a leading, trailing or doubled `/`.
    EmptyComponent,
    /// A component is `.` or `..`.
    DotComponent,
    /// The path holds a NUL or a `\`.
    ForbiddenCharacter(char),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::Empty => write!(f, "the path is empty"),
            PathError::EmptyComponent => write!(f, "the path has an empty component"),
            PathError::DotComponent => write!(f, "the path has a `.` or `..` component"),
            PathError::ForbiddenCharacter(c) => {
                write!(f, "the path holds the forbidden character {c:?}")
            }
        }
    }
}

impl std::error::Error for PathError {}

impl RepoPath {
    /// Checks `path` and wraps it.
    pub fn new(path: impl Into<String>) -> Result<Self, PathError> {
        let path = path.into();
        if path.is_empty() {
            return Err(PathError::Empty);
        }
        if let Some(c) = path.chars().find(|&c| c == '\0' || c == '\\') {
            return Err(PathError::ForbiddenCharacter(c));
        }
        for component in path.split('/') {
            match component {
                "" => return Err(PathError::EmptyComponent),
                "." | ".." => return Err(PathError::DotComponent),
                _ => {}
            }
        }
        Ok(RepoPath(path))
    }

    /// The path as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path's components, in order.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }

    /// The last component.
    pub fn name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// The path of the directory holding this one, or `None` at the top.
    pub fn parent(&self) -> Option<RepoPath> {
        self.0
            .rsplit_once('/')
            .map(|(dir, _)| RepoPath(dir.to_owned()))
    }

    /// `self/name`, if `name` is one valid component.
    pub fn join(&self, name: &str) -> Result<RepoPath, PathError> {
        if name.contains('/') {
            return Err(PathError::ForbiddenCharacter('/'));
        }
        RepoPath::new(format!("{}/{name}", self.0))
    }

    /// Whether `self` is `dir` or lies below it.
    pub fn starts_with(&self, dir: &RepoPath) -> bool {
        self.0 == dir.0
            || (self.0.len() > dir.0.len()
                && self.0.starts_with(&dir.0)
                && self.0.as_bytes()[dir.0.len()] == b'/')
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for RepoPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Encode for RepoPath {
    fn encode(&self, enc: &mut Encoder) {
        enc.str(&self.0);
    }
}

impl Decode for RepoPath {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let at = dec.position();
        let text = dec.str()?;
        RepoPath::new(text).map_err(|_| DecodeError::InvalidPath { at })
    }
}
