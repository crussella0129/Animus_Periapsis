//! Directory entries as [`crate::Fs::scan`] reports them: names that may have
//! no UTF-8 form, and entry kinds (INT-0004 AC-7; Sprint 2 T-004).

use std::fmt::Write as _;

/// A name with no UTF-8 form, kept exactly as the filesystem gave it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RawName {
    /// Bytes that are not UTF-8 (Unix filesystems such as ext4).
    Bytes(Vec<u8>),
    /// UTF-16 code units holding an unpaired surrogate (NTFS).
    Wide(Vec<u16>),
}

/// A directory entry's name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Name {
    /// A name with a UTF-8 form.
    Utf8(String),
    /// A name with none; it can be reported but never tracked.
    Raw(RawName),
}

/// What a directory entry is. Symbolic links are not followed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// A regular file, and whether its owner-execute bit is set (always
    /// `false` where the filesystem has no such bit).
    File {
        /// The owner-execute bit.
        executable: bool,
    },
    /// A directory.
    Dir,
    /// A symbolic link (or, on Windows, a junction or other reparse point
    /// that is a link).
    Symlink,
    /// Anything else: a FIFO, socket or device.
    Other,
}

/// One entry of a directory.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirEntry {
    /// The entry's name.
    pub name: Name,
    /// What it is.
    pub kind: Kind,
}

/// Escapes one character for display: `\` becomes `\\`, a control
/// character `\u{NN}`, anything else itself.
fn push_char(out: &mut String, c: char) {
    if c == '\\' {
        out.push_str("\\\\");
    } else if c.is_control() {
        let _ = write!(out, "\\u{{{:x}}}", c as u32);
    } else {
        out.push(c);
    }
}

impl Name {
    /// The name's UTF-8 form, if it has one.
    pub fn as_utf8(&self) -> Option<&str> {
        match self {
            Name::Utf8(name) => Some(name),
            Name::Raw(_) => None,
        }
    }

    /// The name for display. Every name is escaped the same way: `\` as `\\`
    /// and control characters as `\u{NN}`. A raw name also shows each byte
    /// that is not part of valid UTF-8 as `\xNN`, and each unpaired surrogate
    /// as `\u{DNNN}`. Such a sequence never appears unescaped in a UTF-8
    /// name's display, so no raw name displays like a UTF-8 one.
    pub fn display(&self) -> String {
        let mut out = String::new();
        match self {
            Name::Utf8(name) => name.chars().for_each(|c| push_char(&mut out, c)),
            Name::Raw(RawName::Bytes(bytes)) => {
                for chunk in bytes.utf8_chunks() {
                    chunk.valid().chars().for_each(|c| push_char(&mut out, c));
                    for byte in chunk.invalid() {
                        let _ = write!(out, "\\x{byte:02x}");
                    }
                }
            }
            Name::Raw(RawName::Wide(units)) => {
                for unit in char::decode_utf16(units.iter().copied()) {
                    match unit {
                        Ok(c) => push_char(&mut out, c),
                        Err(error) => {
                            let _ = write!(out, "\\u{{{:x}}}", error.unpaired_surrogate());
                        }
                    }
                }
            }
        }
        out
    }
}
