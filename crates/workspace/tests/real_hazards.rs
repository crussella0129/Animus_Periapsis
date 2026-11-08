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

/// A directory replaced by a symbolic link after a crash is never followed:
/// nothing outside the working copy is removed (code review CR-05).
#[cfg(unix)]
#[test]
fn real_symlinked_directories_are_not_followed() {
    use workspace::materialise;
    use workspace::tree::{File, Tree};
    let base = claim();
    let root = base.join("repo");
    RealFs.create_dir(&root).unwrap();
    put(&base, "elsewhere/x", b"F's x");
    std::os::unix::fs::symlink(base.join("elsewhere"), root.join("d")).unwrap();
    let from: Tree = [(
        codec::RepoPath::new("d/x").unwrap(),
        File {
            blob: codec::object_id(hash::HashAlg::DEFAULT, &codec::blob(b"F's x")),
            mode: codec::Mode::Regular,
        },
    )]
    .into();
    let plan = materialise::plan(
        &RealFs,
        &root,
        &from,
        &Tree::new(),
        RealFs.name_rules(),
        hash::HashAlg::DEFAULT,
        true,
    )
    .unwrap();
    assert!(plan.remove.is_empty(), "{:?}", plan.remove);
    assert!(plan.emptied.is_empty(), "{:?}", plan.emptied);
    let kept = materialise::apply(
        &RealFs,
        &root,
        &from,
        hash::HashAlg::DEFAULT,
        &plan,
        |_| unreachable!(),
    )
    .unwrap();
    assert!(kept.is_empty());
    assert_eq!(std::fs::read(base.join("elsewhere/x")).unwrap(), b"F's x");
    remove_all(&base);
    let _ = std::fs::remove_dir_all(&base);
}

/// A name with no UTF-8 form, in a directory `undo` would empty: it is
/// never removed, and the directory that holds it is kept (INT-0004 AC-7;
/// test critique C-007). Byte 0xff on ext4, an unpaired surrogate on NTFS.
#[test]
fn real_names_without_utf8_survive_undo() {
    #[cfg(unix)]
    let odd: std::ffi::OsString = {
        use std::os::unix::ffi::OsStringExt;
        std::ffi::OsString::from_vec(b"r\xffw".to_vec())
    };
    #[cfg(windows)]
    let odd: std::ffi::OsString = {
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(&[u16::from(b'r'), 0xd800, u16::from(b'w')])
    };
    let base = claim();
    let root = base.join("repo");
    let mut ws = Workspace::init(RealFs, &root, 1).unwrap().0;
    ws.new_change().unwrap();
    put(&root, "sub/x", b"tracked");
    ws.snapshot().unwrap();
    std::fs::write(root.join("sub").join(&odd), b"not UTF-8").unwrap();
    ws.undo().unwrap();
    assert!(!root.join("sub/x").exists(), "the tracked file is removed");
    assert_eq!(
        std::fs::read(root.join("sub").join(&odd)).unwrap(),
        b"not UTF-8",
        "the entry with no UTF-8 name is kept"
    );
    assert!(root.join("sub").is_dir(), "its directory is kept");
    assert!(!root.join(".praps/working-copy").exists());
    std::fs::remove_file(root.join("sub").join(&odd)).unwrap();
    remove_all(&base);
    let _ = std::fs::remove_dir_all(&base);
}

/// CR-05 through resume: a crash left a record, and a directory of F was
/// then replaced by a symbolic link. The resuming `open` removes nothing
/// through the link (test critique C-008). Skipped, and said so, where the
/// host cannot make a directory symlink.
#[test]
fn real_resume_does_not_follow_a_symlinked_directory() {
    use workspace::record::{self, Record};
    let base = claim();
    let root = base.join("repo");
    let mut ws = Workspace::init(RealFs, &root, 1).unwrap().0;
    put(&root, "d/x", b"F's x");
    ws.snapshot().unwrap();
    let from = *ws.store().heads().unwrap().first().unwrap();
    RealFs.remove_file(&root.join("d").join("x")).unwrap();
    RealFs.remove_dir(&root.join("d")).unwrap();
    ws.snapshot().unwrap();
    let target = *ws.store().heads().unwrap().first().unwrap();
    drop(ws);
    // The crash: F's files are back on disk, but `d` is now a link to a
    // directory outside the working copy holding the same `x`.
    put(&base, "elsewhere/x", b"F's x");
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(base.join("elsewhere"), root.join("d"));
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_dir(base.join("elsewhere"), root.join("d"));
    if let Err(error) = linked {
        println!("SKIPPED real_resume_does_not_follow_a_symlinked_directory: {error}");
        let _ = std::fs::remove_dir_all(&base);
        return;
    }
    record::write(&RealFs, &root.join(".praps"), &Record { from, target }).unwrap();
    let ws = Workspace::open(RealFs, &root, 1).unwrap();
    assert_eq!(std::fs::read(base.join("elsewhere/x")).unwrap(), b"F's x");
    assert!(!root.join(".praps/working-copy").exists());
    drop(ws);
    let _ = std::fs::remove_dir_all(&base);
}
