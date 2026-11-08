//! The `Fs` contract on both implementations, and the simulator's fault model
//! (INT-0005 AC-1, AC-2; INT-0004 AC-1).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use sim::{DirEntry, Fs, Kind, Mode, Name, NameRules, Node, RealFs, Rng, SimFs, is_crash};

/// Seeds tried where an outcome must be seen "across seeds".
const SEEDS: u64 = 200;

// ---------------------------------------------------------------- contract

/// What every `Fs` must do without crashes, under `base`.
fn contract(fs: &impl Fs, base: &Path) {
    let dir = base.join("d");
    fs.create_dir(&dir).unwrap();
    assert_eq!(
        fs.create_dir(&dir).unwrap_err().kind(),
        ErrorKind::AlreadyExists
    );
    assert_eq!(fs.list(&dir).unwrap(), Vec::<String>::new());

    let a = dir.join("a");
    fs.create_new(&a).unwrap();
    assert_eq!(
        fs.create_new(&a).unwrap_err().kind(),
        ErrorKind::AlreadyExists
    );
    fs.write(&a, b"hel").unwrap();
    fs.write(&a, b"lo").unwrap();
    assert_eq!(fs.read(&a).unwrap(), b"hello");
    fs.overwrite(&a, b"bye").unwrap();
    assert_eq!(fs.read(&a).unwrap(), b"bye");
    fs.sync_file(&a).unwrap();
    fs.sync_dir(&dir).unwrap();

    // rename replaces an existing target.
    let b = dir.join("b");
    fs.create_new(&b).unwrap();
    fs.write(&b, b"old").unwrap();
    fs.rename(&a, &b).unwrap();
    assert_eq!(fs.read(&b).unwrap(), b"bye");
    assert_eq!(fs.read(&a).unwrap_err().kind(), ErrorKind::NotFound);
    assert_eq!(fs.list(&dir).unwrap(), vec!["b".to_owned()]);

    // Names are listed sorted by their bytes. (No two differ only in case:
    // NTFS would take them as one name.)
    for name in ["c", "_x", "a"] {
        fs.create_new(&dir.join(name)).unwrap();
    }
    assert_eq!(fs.list(&dir).unwrap(), ["_x", "a", "b", "c"]);

    fs.remove_file(&dir.join("c")).unwrap();
    assert_eq!(
        fs.remove_file(&dir.join("c")).unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        fs.write(&dir.join("missing"), b"x").unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        fs.create_new(&base.join("nodir").join("x"))
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        fs.list(&base.join("nodir")).unwrap_err().kind(),
        ErrorKind::NotFound
    );
    fs.sync_dir(&dir).unwrap();
    scan_contract(fs, base);
    removal_and_mode_contract(fs, base);
}

/// `remove_dir` and `set_executable` (Sprint 3 T-003).
fn removal_and_mode_contract(fs: &impl Fs, base: &Path) {
    let dir = base.join("r");
    fs.create_dir(&dir).unwrap();
    let (empty, file) = (dir.join("e"), dir.join("f"));
    fs.create_dir(&empty).unwrap();
    fs.create_new(&file).unwrap();
    fs.sync_file(&file).unwrap();
    fs.sync_dir(&dir).unwrap();
    fs.sync_dir(&empty).unwrap();

    fs.remove_dir(&empty).unwrap();
    assert_eq!(fs.list(&dir).unwrap(), ["f"]);
    assert_eq!(
        fs.remove_dir(&empty).unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        fs.remove_dir(&dir).unwrap_err().kind(),
        ErrorKind::DirectoryNotEmpty
    );
    assert!(fs.remove_dir(&file).is_err(), "a file is not a directory");
    fs.sync_dir(&dir).unwrap();

    let bit = fs.has_executable_bit();
    let kind_of_f = || {
        fs.scan(&dir)
            .unwrap()
            .into_iter()
            .find(|entry| entry.name == Name::Utf8("f".to_owned()))
            .map(|entry| entry.kind)
    };
    fs.set_executable(&file, true).unwrap();
    assert_eq!(kind_of_f(), Some(Kind::File { executable: bit }));
    fs.set_executable(&file, false).unwrap();
    assert_eq!(kind_of_f(), Some(FILE));
    fs.sync_file(&file).unwrap();
    assert_eq!(
        fs.set_executable(&dir.join("missing"), true)
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    assert!(
        fs.set_executable(&dir, true).is_err(),
        "a directory is not a file"
    );
}

fn utf8(name: &str, kind: Kind) -> DirEntry {
    DirEntry {
        name: Name::Utf8(name.to_owned()),
        kind,
    }
}

const FILE: Kind = Kind::File { executable: false };

/// `scan` on plain files and directories: kinds, and order by display form
/// (Sprint 2 T-004).
fn scan_contract(fs: &impl Fs, base: &Path) {
    let dir = base.join("s");
    fs.create_dir(&dir).unwrap();
    assert_eq!(fs.scan(&dir).unwrap(), []);
    for name in ["b.txt", "A", "é", "_"] {
        fs.create_new(&dir.join(name)).unwrap();
    }
    fs.create_dir(&dir.join("sub")).unwrap();
    assert_eq!(
        fs.scan(&dir).unwrap(),
        [
            utf8("A", FILE),
            utf8("_", FILE),
            utf8("b.txt", FILE),
            utf8("sub", Kind::Dir),
            utf8("é", FILE),
        ]
    );
    assert_eq!(fs.scan(&dir.join("sub")).unwrap(), []);
    assert_eq!(
        fs.scan(&base.join("nodir")).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

/// A directory under the OS temp directory that did not exist before: a name
/// already taken is never reused.
fn claim_temp_dir(label: &str) -> PathBuf {
    let base = std::env::temp_dir();
    let mut rng = Rng::new(u64::from(std::process::id()) ^ label.len() as u64);
    loop {
        let dir = base.join(format!("sim-{label}-{:016x}", rng.next_u64()));
        match RealFs.create_dir(&dir) {
            Ok(()) => return dir,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("{}: {error}", dir.display()),
        }
    }
}

#[allow(clippy::disallowed_methods)]
fn remove_temp_dir(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn sim_fs_meets_the_contract() {
    for mode in [Mode::Lenient, Mode::Strict] {
        contract(&SimFs::new(7, mode), Path::new(""));
    }
}

#[test]
fn name_rules_report_the_platform() {
    let windows = cfg!(windows);
    assert_eq!(
        RealFs.name_rules(),
        NameRules {
            case_insensitive: windows,
            windows_names: windows,
        }
    );
    let fs = SimFs::new(0, Mode::Strict);
    assert_eq!(fs.name_rules(), NameRules::default());
    let rules = NameRules {
        case_insensitive: true,
        windows_names: true,
    };
    fs.set_name_rules(rules);
    assert_eq!(fs.name_rules(), rules);
    assert_eq!(fs.syscalls(), 0, "not a syscall");
}

#[test]
fn real_fs_meets_the_contract() {
    let dir = claim_temp_dir("contract");
    contract(&RealFs, &dir);
    RealFs.sync_dir(&dir).unwrap();
    remove_temp_dir(&dir);
}

#[test]
fn rename_replaces_and_create_new_refuses() {
    fn check(fs: &impl Fs, base: &Path) {
        let (x, y) = (base.join("x"), base.join("y"));
        fs.create_new(&x).unwrap();
        fs.write(&x, b"1").unwrap();
        fs.create_new(&y).unwrap();
        fs.write(&y, b"2").unwrap();
        assert_eq!(
            fs.create_new(&y).unwrap_err().kind(),
            ErrorKind::AlreadyExists
        );
        assert_eq!(
            fs.read(&y).unwrap(),
            b"2",
            "a refused create_new leaves the file"
        );
        fs.rename(&x, &y).unwrap();
        assert_eq!(fs.read(&y).unwrap(), b"1");
    }
    check(&SimFs::new(1, Mode::Strict), Path::new(""));
    let dir = claim_temp_dir("rename");
    check(&RealFs, &dir);
    remove_temp_dir(&dir);
}

// ------------------------------------------------------------ fault model

#[test]
fn splitmix64_matches_reference() {
    // Vigna's splitmix64.c; the 1234567 sequence is the one its test vectors
    // and the java.util.SplittableRandom tests use.
    let mut rng = Rng::new(1_234_567);
    let outputs: Vec<u64> = (0..5).map(|_| rng.next_u64()).collect();
    assert_eq!(
        outputs,
        [
            6_457_827_717_110_365_317,
            3_203_168_211_198_807_973,
            9_817_491_932_198_370_423,
            4_593_380_528_125_082_431,
            16_408_922_859_458_223_821,
        ]
    );
    assert_eq!(Rng::new(0).next_u64(), 0xe220_a839_7b1d_cdaf);
}

/// One scripted run: a directory, two files written and synced in different
/// ways, a rename. Returns the filesystem after the run (crashed or not).
fn script(seed: u64, mode: Mode, crash_at: Option<u64>) -> SimFs {
    let fs = SimFs::new(seed, mode);
    fs.set_crash_at(crash_at);
    let d = Path::new("d");
    let _ = (|| -> std::io::Result<()> {
        fs.create_dir(d)?;
        fs.sync_dir(Path::new(""))?;
        fs.create_new(&d.join("a"))?;
        fs.write(&d.join("a"), b"synced")?;
        fs.sync_file(&d.join("a"))?;
        fs.sync_dir(d)?;
        fs.create_new(&d.join("t"))?;
        fs.write(&d.join("t"), b"pending")?;
        fs.rename(&d.join("t"), &d.join("b"))?;
        fs.write(&d.join("a"), b"+more")?;
        Ok(())
    })();
    fs
}

const SCRIPT_SYSCALLS: u64 = 10;

#[test]
fn crash_point_boundary_is_exact() {
    assert_eq!(script(0, Mode::Strict, None).syscalls(), SCRIPT_SYSCALLS);
    for k in 1..=SCRIPT_SYSCALLS {
        let fs = script(0, Mode::Strict, Some(k));
        assert!(fs.crashed());
        assert_eq!(fs.syscalls(), k, "syscall {k} is the last one counted");
        let trace = fs.trace();
        assert_eq!(trace.len() as u64, k);
        assert!(trace[k as usize - 1].ends_with("(crash)"));
        // Whether syscall k took effect is checked against the state from just
        // before it in `every_syscall_has_no_effect_at_its_crash_point`; here,
        // the crash at k + 1 shows syscall k's effect.
        let before = script(0, Mode::Strict, Some(k)).snapshot();
        let after_k = script(0, Mode::Strict, Some(k + 1)).snapshot();
        if k == 1 {
            assert!(before.is_empty(), "create_dir at 1 never happened");
            assert_eq!(after_k.get("d"), Some(&Node::Dir));
        }
        // Later syscalls fail with the crash error and change nothing.
        let error = fs.create_new(Path::new("late")).unwrap_err();
        assert!(is_crash(&error));
        assert_eq!(fs.snapshot(), before);
        assert_eq!(fs.syscalls(), k);
    }
    // Spot-check the step that differs: write at 4 is absent at 4, present at 5.
    let at4 = script(0, Mode::Strict, Some(4)).snapshot();
    let at5 = script(0, Mode::Strict, Some(5)).snapshot();
    assert_eq!(at4.get("d/a"), Some(&Node::File(Vec::new())));
    assert_eq!(at5.get("d/a"), Some(&Node::File(b"synced".to_vec())));
}

#[test]
fn simfs_runs_are_deterministic() {
    for mode in [Mode::Lenient, Mode::Strict] {
        for seed in 0..50 {
            for k in 1..=SCRIPT_SYSCALLS + 1 {
                let (a, b) = (script(seed, mode, Some(k)), script(seed, mode, Some(k)));
                assert_eq!(a.trace(), b.trace());
                assert_eq!(a.recover().snapshot(), b.recover().snapshot());
            }
        }
    }
}

#[test]
fn synced_files_survive_any_crash() {
    // `d/a` is synced with its directory at syscall 6; the script later
    // appends to it, so its synced prefix must survive, intact.
    for mode in [Mode::Lenient, Mode::Strict] {
        for seed in 0..SEEDS {
            for k in 7..=SCRIPT_SYSCALLS + 1 {
                let recovered = script(seed, mode, Some(k)).recover().snapshot();
                match recovered.get("d/a") {
                    Some(Node::File(content)) => assert!(
                        content.starts_with(b"synced"),
                        "seed {seed} crash {k}: {content:?}"
                    ),
                    other => panic!("seed {seed} crash {k}: d/a is {other:?}"),
                }
            }
        }
    }
}

/// The outcomes, across seeds, of `name`'s content after a crash.
fn outcomes(
    run: impl Fn(u64) -> SimFs,
    name: &str,
    classify: impl Fn(Option<&Node>) -> &'static str,
) -> BTreeSet<&'static str> {
    (0..SEEDS)
        .map(|seed| classify(run(seed).recover().snapshot().get(name)))
        .collect()
}

#[test]
fn unsynced_writes_are_kept_dropped_or_torn() {
    let run = |seed| {
        let fs = SimFs::new(seed, Mode::Strict);
        fs.create_new(Path::new("f")).unwrap();
        fs.write(Path::new("f"), b"base").unwrap();
        fs.sync_file(Path::new("f")).unwrap();
        fs.sync_dir(Path::new("")).unwrap();
        fs.write(Path::new("f"), b"-appended").unwrap();
        fs
    };
    let seen = outcomes(run, "f", |node| match node {
        Some(Node::File(c)) if c == b"base-appended" => "kept",
        Some(Node::File(c)) if c == b"base" => "dropped",
        Some(Node::File(c)) if b"base-appended".starts_with(c) && c.starts_with(b"base") => "torn",
        other => panic!("bytes that were never written, or synced bytes lost: {other:?}"),
    });
    assert_eq!(seen, BTreeSet::from(["kept", "dropped", "torn"]));
}

#[test]
fn strict_mode_loses_unsynced_entries() {
    let run = |mode| {
        move |seed| {
            let fs = SimFs::new(seed, mode);
            fs.create_new(Path::new("f")).unwrap();
            fs.write(Path::new("f"), b"x").unwrap();
            fs.sync_file(Path::new("f")).unwrap();
            fs
        }
    };
    let present = |node: Option<&Node>| if node.is_some() { "present" } else { "absent" };
    assert_eq!(
        outcomes(run(Mode::Strict), "f", present),
        BTreeSet::from(["present", "absent"])
    );
    assert_eq!(
        outcomes(run(Mode::Lenient), "f", present),
        BTreeSet::from(["present"])
    );
}

#[test]
fn strict_mode_reverts_unsynced_renames() {
    let run = |seed| {
        let fs = SimFs::new(seed, Mode::Strict);
        fs.create_new(Path::new("t")).unwrap();
        fs.write(Path::new("t"), b"x").unwrap();
        fs.sync_file(Path::new("t")).unwrap();
        fs.sync_dir(Path::new("")).unwrap();
        fs.rename(Path::new("t"), Path::new("f")).unwrap();
        fs
    };
    let names: BTreeSet<Vec<String>> = (0..SEEDS)
        .map(|seed| run(seed).recover().snapshot().into_keys().collect())
        .collect();
    // The rename is atomic: one name or the other, never both or neither.
    assert_eq!(
        names,
        BTreeSet::from([vec!["f".to_owned()], vec!["t".to_owned()]])
    );
}

#[test]
fn strict_mode_loses_unsynced_directories() {
    let run = |mode| {
        move |seed| {
            let fs = SimFs::new(seed, mode);
            fs.create_dir(Path::new("d")).unwrap();
            fs.create_new(Path::new("d/f")).unwrap();
            fs.write(Path::new("d/f"), b"x").unwrap();
            fs.sync_file(Path::new("d/f")).unwrap();
            fs.sync_dir(Path::new("d")).unwrap();
            fs
        }
    };
    let shape = |seed, mode| {
        let snap = run(mode)(seed).recover().snapshot();
        match (snap.get("d"), snap.get("d/f")) {
            (Some(Node::Dir), Some(Node::File(c))) if c == b"x" => "whole",
            (None, None) => "gone with its contents",
            other => panic!("{other:?}"),
        }
    };
    let strict: BTreeSet<_> = (0..SEEDS).map(|s| shape(s, Mode::Strict)).collect();
    assert_eq!(strict, BTreeSet::from(["whole", "gone with its contents"]));
    let lenient: BTreeSet<_> = (0..SEEDS).map(|s| shape(s, Mode::Lenient)).collect();
    assert_eq!(lenient, BTreeSet::from(["whole"]));
}

#[test]
fn lenient_mode_keeps_synced_entries() {
    for seed in 0..SEEDS {
        let fs = SimFs::new(seed, Mode::Lenient);
        fs.create_dir(Path::new("d")).unwrap();
        fs.create_new(Path::new("d/t")).unwrap();
        fs.write(Path::new("d/t"), b"x").unwrap();
        fs.rename(Path::new("d/t"), Path::new("d/f")).unwrap();
        fs.sync_file(Path::new("d/f")).unwrap();
        let snap = fs.recover().snapshot();
        assert_eq!(
            snap.get("d/f"),
            Some(&Node::File(b"x".to_vec())),
            "seed {seed}"
        );
        assert_eq!(snap.get("d/t"), None, "seed {seed}");
    }
}

#[test]
fn unsynced_data_stays_at_risk_across_rename() {
    let run = |seed| {
        let fs = SimFs::new(seed, Mode::Strict);
        fs.create_new(Path::new("t")).unwrap();
        fs.write(Path::new("t"), b"content").unwrap();
        fs.rename(Path::new("t"), Path::new("f")).unwrap();
        fs.sync_dir(Path::new("")).unwrap();
        fs
    };
    let seen = outcomes(run, "f", |node| match node {
        Some(Node::File(c)) if c == b"content" => "kept",
        Some(Node::File(c)) if c.is_empty() => "dropped or torn to nothing",
        Some(Node::File(c)) if b"content".starts_with(c) => "torn",
        other => panic!("the name was synced, so it must exist: {other:?}"),
    });
    assert_eq!(
        seen,
        BTreeSet::from(["kept", "dropped or torn to nothing", "torn"])
    );
}

#[test]
fn unsynced_overwrite_recovers_old_empty_or_torn() {
    let run = |seed| {
        let fs = SimFs::new(seed, Mode::Strict);
        fs.create_new(Path::new("f")).unwrap();
        fs.write(Path::new("f"), b"old").unwrap();
        fs.sync_file(Path::new("f")).unwrap();
        fs.sync_dir(Path::new("")).unwrap();
        fs.overwrite(Path::new("f"), b"new!").unwrap();
        fs
    };
    let seen = outcomes(run, "f", |node| match node {
        Some(Node::File(c)) if c == b"old" => "old",
        Some(Node::File(c)) if c == b"new!" => "new",
        Some(Node::File(c)) if c.is_empty() => "empty",
        Some(Node::File(c)) if b"new!".starts_with(c) => "torn",
        other => panic!("{other:?}"),
    });
    assert_eq!(seen, BTreeSet::from(["old", "new", "empty", "torn"]));
}

#[test]
fn parent_components_are_lexical() {
    let fs = SimFs::new(0, Mode::Strict);
    fs.create_dir(Path::new("d")).unwrap();
    fs.create_new(Path::new("d/../f")).unwrap();
    assert_eq!(fs.list(Path::new("d/..")).unwrap(), ["d", "f"]);
    fs.sync_dir(Path::new("d/..")).unwrap();
    assert_eq!(
        fs.create_new(Path::new("../x")).unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
}

/// A pending removal acts only on the file it removed: after the name is
/// reused for a new file and that file is synced in lenient mode, no crash
/// loses it (code review, Sprint 1).
#[test]
fn stale_removals_spare_a_reused_name() {
    for seed in 0..SEEDS {
        let fs = SimFs::new(seed, Mode::Lenient);
        let x = Path::new("x");
        fs.create_new(x).unwrap();
        fs.write(x, b"old").unwrap();
        fs.remove_file(x).unwrap();
        fs.create_new(x).unwrap();
        fs.write(x, b"new").unwrap();
        fs.sync_file(x).unwrap();
        assert_eq!(
            fs.recover().snapshot().get("x"),
            Some(&Node::File(b"new".to_vec())),
            "seed {seed}"
        );
    }
}

/// Every kind of syscall, with every effect visible in a snapshot: the 13
/// kinds create_dir, create_new, write, overwrite, set_executable, sync_file,
/// rename, sync_dir, read, list, scan, remove_file and remove_dir (with a
/// second create_dir for the directory to remove).
fn every_kind(fs: &SimFs) -> std::io::Result<()> {
    fs.create_dir(Path::new("e"))?;
    fs.create_dir(Path::new("d"))?;
    fs.create_new(Path::new("d/a"))?;
    fs.write(Path::new("d/a"), b"one")?;
    fs.overwrite(Path::new("d/a"), b"two")?;
    fs.set_executable(Path::new("d/a"), true)?;
    fs.sync_file(Path::new("d/a"))?;
    fs.rename(Path::new("d/a"), Path::new("d/b"))?;
    fs.sync_dir(Path::new("d"))?;
    fs.read(Path::new("d/b"))?;
    fs.list(Path::new("d"))?;
    fs.scan(Path::new("d"))?;
    fs.remove_file(Path::new("d/b"))?;
    fs.remove_dir(Path::new("e"))?;
    Ok(())
}

const EVERY_KIND_SYSCALLS: u64 = 14;

#[test]
fn every_kind_runs_are_deterministic() {
    for mode in [Mode::Lenient, Mode::Strict] {
        for seed in 0..50 {
            for k in 1..=EVERY_KIND_SYSCALLS + 1 {
                let run = || {
                    let fs = SimFs::new(seed, mode);
                    fs.set_crash_at(Some(k));
                    let _ = every_kind(&fs);
                    fs
                };
                let (a, b) = (run(), run());
                assert_eq!(a.trace(), b.trace());
                assert_eq!(a.recover().snapshot(), b.recover().snapshot());
            }
        }
    }
}

/// An unsynced `remove_dir` is kept or reverted by the seed; a reverted one
/// leaves the directory empty. Once its parent is synced it is kept.
#[test]
fn unsynced_remove_dir_may_revert() {
    let run = |sync_parent: bool| {
        move |seed| {
            let fs = SimFs::new(seed, Mode::Strict);
            fs.create_dir(Path::new("e")).unwrap();
            fs.sync_dir(Path::new("")).unwrap();
            fs.remove_dir(Path::new("e")).unwrap();
            if sync_parent {
                fs.sync_dir(Path::new("")).unwrap();
            }
            fs
        }
    };
    let shape = |node: Option<&Node>| match node {
        Some(Node::Dir) => "reverted",
        None => "kept",
        other => panic!("{other:?}"),
    };
    assert_eq!(
        outcomes(run(false), "e", shape),
        BTreeSet::from(["reverted", "kept"])
    );
    assert_eq!(outcomes(run(true), "e", shape), BTreeSet::from(["kept"]));
    // A reverted removal holds nothing.
    for seed in 0..SEEDS {
        let snap = run(false)(seed).recover().snapshot();
        assert!(snap.keys().all(|path| path == "e"), "{snap:?}");
    }
}

/// `remove_dir` needs the directory empty with its own entry changes synced.
#[test]
fn remove_dir_refuses_an_unsynced_directory() {
    let fs = SimFs::new(0, Mode::Strict);
    fs.create_dir(Path::new("e")).unwrap();
    fs.create_new(Path::new("e/f")).unwrap();
    fs.remove_file(Path::new("e/f")).unwrap();
    assert_eq!(
        fs.remove_dir(Path::new("e")).unwrap_err().kind(),
        ErrorKind::ResourceBusy
    );
    assert!(fs.snapshot().contains_key("e"), "nothing changed");
    fs.sync_dir(Path::new("e")).unwrap();
    fs.remove_dir(Path::new("e")).unwrap();
}

#[test]
fn create_dir_is_refused_while_its_removal_is_pending() {
    for mode in [Mode::Lenient, Mode::Strict] {
        let fs = SimFs::new(0, mode);
        fs.create_dir(Path::new("e")).unwrap();
        fs.sync_dir(Path::new("")).unwrap();
        fs.remove_dir(Path::new("e")).unwrap();
        assert_eq!(
            fs.create_dir(Path::new("e")).unwrap_err().kind(),
            ErrorKind::ResourceBusy
        );
        fs.sync_dir(Path::new("")).unwrap();
        fs.create_dir(Path::new("e")).unwrap();
    }
}

/// An unsynced executable-bit change is kept or reverted by the seed, and
/// kept once the file is synced.
#[test]
fn unsynced_mode_change_may_revert() {
    let run = |sync: bool| {
        move |seed| {
            let fs = SimFs::new(seed, Mode::Strict);
            let f = Path::new("f");
            fs.create_new(f).unwrap();
            fs.write(f, b"x").unwrap();
            fs.sync_file(f).unwrap();
            fs.sync_dir(Path::new("")).unwrap();
            fs.set_executable(f, true).unwrap();
            if sync {
                fs.sync_file(f).unwrap();
            }
            fs
        }
    };
    let shape = |node: Option<&Node>| match node {
        Some(Node::Executable(c)) if c == b"x" => "kept",
        Some(Node::File(c)) if c == b"x" => "reverted",
        other => panic!("{other:?}"),
    };
    assert_eq!(
        outcomes(run(false), "f", shape),
        BTreeSet::from(["kept", "reverted"])
    );
    assert_eq!(outcomes(run(true), "f", shape), BTreeSet::from(["kept"]));
}

/// Without an executable bit, `set_executable` succeeds and changes nothing.
#[test]
fn set_executable_without_the_bit_changes_nothing() {
    let fs = SimFs::new(0, Mode::Strict);
    fs.set_has_executable_bit(false);
    fs.create_new(Path::new("f")).unwrap();
    fs.set_executable(Path::new("f"), true).unwrap();
    assert_eq!(fs.snapshot().get("f"), Some(&Node::File(Vec::new())));
}

/// The helper is not a syscall, and is durable at once.
#[test]
fn make_executable_is_not_a_syscall() {
    for seed in 0..SEEDS {
        let fs = SimFs::new(seed, Mode::Strict);
        fs.create_new(Path::new("f")).unwrap();
        fs.sync_dir(Path::new("")).unwrap();
        let before = fs.syscalls();
        fs.make_executable(Path::new("f"), true).unwrap();
        assert_eq!(fs.syscalls(), before);
        assert_eq!(
            fs.recover().snapshot().get("f"),
            Some(&Node::Executable(Vec::new()))
        );
    }
}

/// What a snapshot holds: every visible path and its node.
type Snapshot = BTreeMap<String, Node>;

/// A crash at syscall k leaves exactly the state from just before syscall k
/// (captured by the hook at that boundary), visible and durable alike, and k
/// itself returns the crash error (test critique round 1, C-002).
#[test]
fn every_syscall_has_no_effect_at_its_crash_point() {
    let fs = SimFs::new(0, Mode::Strict);
    every_kind(&fs).unwrap();
    assert_eq!(fs.syscalls(), EVERY_KIND_SYSCALLS);
    for mode in [Mode::Lenient, Mode::Strict] {
        for k in 1..=EVERY_KIND_SYSCALLS {
            for seed in 0..20 {
                let fs = Rc::new(SimFs::new(seed, mode));
                fs.set_crash_at(Some(k));
                let before: Rc<RefCell<Option<(Snapshot, Snapshot)>>> = Rc::default();
                let (hook_fs, hook_before) = (Rc::clone(&fs), Rc::clone(&before));
                fs.set_hook(Some(Box::new(move |next| {
                    if next == k {
                        *hook_before.borrow_mut() =
                            Some((hook_fs.snapshot(), hook_fs.recover().snapshot()));
                    }
                })));
                let error = every_kind(&fs).unwrap_err();
                fs.set_hook(None);
                assert!(is_crash(&error), "k {k}: {error}");
                assert_eq!(fs.trace().len() as u64, k);
                let (visible, durable) = before.borrow_mut().take().expect("hook ran at k");
                assert_eq!(
                    fs.snapshot(),
                    visible,
                    "{mode:?} k {k}: syscall k changed what is visible"
                );
                assert_eq!(
                    fs.recover().snapshot(),
                    durable,
                    "{mode:?} k {k} seed {seed}: syscall k changed what survives"
                );
            }
        }
    }
}
