//! Hazards on real filesystems (INT-0004 AC-3, AC-4; Sprint 3 T-009): a
//! tree that NTFS cannot hold is refused there with nothing written, and the
//! same tree materialises on ext4. `RealFs` passes verbatim paths on
//! Windows, so the tree itself can be made there.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use sim::{Fs, Kind, RealFs, Rng};
use workspace::Workspace;

fn claim() -> PathBuf {
    let mut rng = Rng::new(u64::from(std::process::id()) ^ 0x4a7a);
    loop {
        let dir = std::env::temp_dir().join(format!("praps-hazards-{:016x}", rng.next_u64()));
        match RealFs.create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

/// Removes a test directory, through `RealFs` so that names Win32 cannot
/// open (`CON`, `dot.`) are removed too.
fn remove_all(dir: &Path) {
    for entry in RealFs.scan(dir).unwrap_or_default() {
        let path = dir.join(entry.name.as_utf8().unwrap());
        match entry.kind {
            Kind::Dir => {
                remove_all(&path);
                let _ = RealFs.remove_dir(&path);
            }
            _ => {
                let _ = RealFs.remove_file(&path);
            }
        }
    }
}

fn put(root: &Path, path: &str, content: &[u8]) {
    let mut dir = root.to_path_buf();
    let mut parts: Vec<&str> = path.split('/').collect();
    let name = parts.pop().unwrap();
    for part in parts {
        dir = dir.join(part);
        match RealFs.create_dir(&dir) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
    let file = dir.join(name);
    RealFs.create_new(&file).unwrap();
    RealFs.write(&file, content).unwrap();
}

/// Every file under `root` but `.praps`, with its content.
fn files(root: &Path, prefix: &str) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for entry in RealFs.scan(root).unwrap() {
        let name = entry.name.as_utf8().unwrap().to_owned();
        if prefix.is_empty() && name == ".praps" {
            continue;
        }
        let shown = format!("{prefix}{name}");
        match entry.kind {
            Kind::Dir => out.extend(files(&root.join(&name), &format!("{shown}/"))),
            _ => {
                out.insert(shown, RealFs.read(&root.join(&name)).unwrap());
            }
        }
    }
    out
}

/// Makes a repository whose earlier view holds `target`, with `from` on
/// disk now, and restores the earlier view.
fn restore_over(
    base: &Path,
    target: &[(&str, &str)],
    from: &[(&str, &str)],
) -> (PathBuf, Result<(), workspace::Error>) {
    let root = base.join("repo");
    let mut ws = Workspace::init(RealFs, &root, 1).unwrap().0;
    for (path, content) in target {
        put(&root, path, content.as_bytes());
    }
    ws.snapshot().unwrap();
    let ops = ws.store().ops().unwrap();
    let t_view = ops[ws.store().heads().unwrap().first().unwrap()].view;
    for (path, _) in target {
        let mut on_disk = root.clone();
        for part in path.split('/') {
            on_disk = on_disk.join(part);
        }
        RealFs.remove_file(&on_disk).unwrap();
    }
    for (path, content) in from {
        put(&root, path, content.as_bytes());
    }
    ws.snapshot().unwrap();
    let result = ws.restore_view(&t_view).map(|_| ());
    (root, result)
}

/// On NTFS: a reserved name, a trailing dot, and a case-only rename are
/// refused, and nothing is written.
#[cfg(windows)]
#[test]
fn real_hazards_are_refused_on_ntfs() {
    let base = claim();
    let target = [("README", "upper"), ("CON", "device"), ("dot.", "dot")];
    let (root, result) = restore_over(&base, &target, &[("readme", "lower")]);
    match result {
        Err(workspace::Error::Hazards(hazards)) => {
            let shown: Vec<String> = hazards.iter().map(ToString::to_string).collect();
            for name in ["README", "CON", "dot."] {
                assert!(
                    shown.iter().any(|h| h.starts_with(name)),
                    "{name}: {shown:?}"
                );
            }
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        files(&root, ""),
        BTreeMap::from([("readme".to_owned(), b"lower".to_vec())]),
        "nothing written"
    );
    assert!(!root.join(".praps").join("working-copy").exists());
    remove_all(&base);
    let _ = std::fs::remove_dir_all(&base);
}

/// On ext4 the same names, and the case collisions NTFS could not even
/// hold, materialise exactly.
#[cfg(unix)]
#[test]
fn real_hazard_trees_materialise_on_ext4() {
    let base = claim();
    let target = [
        ("README", "upper"),
        ("readme", "lower"),
        ("CON", "device"),
        ("dot.", "dot"),
        ("a:b", "colon"),
        ("a", "file"),
        ("A/x", "dir"),
    ];
    let (root, result) = restore_over(&base, &target, &[("other", "o")]);
    result.unwrap();
    let expected: BTreeMap<String, Vec<u8>> = target
        .iter()
        .map(|(p, c)| ((*p).to_owned(), c.as_bytes().to_vec()))
        .collect();
    assert_eq!(files(&root, ""), expected);
    remove_all(&base);
    let _ = std::fs::remove_dir_all(&base);
}
