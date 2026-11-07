//! The `Fs` contract on both implementations, and the simulator's fault model
//! (INT-0005 AC-1, AC-2; INT-0004 AC-1).

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use sim::{Fs, Mode, Node, RealFs, Rng, SimFs, is_crash};

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
        // Syscall k took no effect: the run without it equals the one with a
        // crash at k, while the crash at k + 1 shows its effect.
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
