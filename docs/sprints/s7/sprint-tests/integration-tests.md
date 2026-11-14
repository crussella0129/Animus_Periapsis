# Sprint 7 integration test results

The tested head is `b951d6d`: T-001 to T-010 and three commits made during Test.
- `1d5c94b` holds the fixes from the [code review](code-review.md), and gives the lane matrix and the Concurrency scenario a digest for each platform.
- `5b65da8` moves T-008's 1,000-process test into a test binary of its own and lets the scheduler tests' deadlock guards run five minutes. See "GitHub's Windows runner and the scheduler tests" below.
- `b951d6d` answers the first round of the [test critique](critique-round1.md). An unreadable `.praps/workspace` is now refused as that file, and the tests the critique found weak are added or tightened.

The unit results are in [unit-tests.md](unit-tests.md), the matrices in [test-matrix-log.md](test-matrix-log.md), and the runs through the binary and on GitHub in [e2e-tests.md](e2e-tests.md).

## The suite on both platforms

| | Windows 11 (NTFS) | WSL Ubuntu 26.04.1 (ext4) |
|---|---|---|
| Where | the working checkout | a fresh clone of `b951d6d`, made for this run |
| `rustc` | 1.99.1 (58cf8f9c3 2026-10-13) | the same |
| `cargo ci` | all 6 steps passed | all 6 steps passed |
| Wall time | 205 s, on a quiet machine, building only what changed | 211 s, a cold build in the fresh clone |
| `test:` lines | 52 test executables built, 52 run; 43 units, 663 passed, 0 failed, 0 ignored | 52 built, 52 run; 43 units, 666 passed, 0 failed, 0 ignored |
| Tree after the run | clean | clean |
| `cargo book`, first line | `book: docs (typed-acceptance 1, strict)` | the same |
| `cargo book`, last line | `38 chapters, 267 criteria, 0 violations` | the same |
| `cargo deny check` | `advisories ok, bans ok, licenses ok, sources ok` | the same |
| `cargo reach`, then `git status` | wrote `docs/reach.md` with no difference | the same |

- **Totals against Sprint 6.** At `855eb67`, 588 on Windows and 590 on Linux; now 663 and 666. The difference is the 76 tests this sprint adds, one of them for Unix only. No test was removed; the three renamed are listed in [unit-tests.md](unit-tests.md).
- **The slowest units.** On Windows: `materialise_crash` 180 s, `describe_crash` 125 s, `crash` 120 s, `writers` 80 s, `lane_crash` 53 s, `concurrency` 53 s. In WSL: `materialise_crash` 128 s, `describe_crash` 94 s, `crash` 89 s, `writers` 83 s, `concurrency` 35 s. Units run several at a time, so a unit's time includes waiting for the machine.
- **Earlier heads.** At `5b65da8` the suite took 868 s on Windows and 507 s in WSL, while the machine ran slow; at `1d5c94b`, 426 s and 304 s. It passed on both systems at both, with 659 tests on Windows and 661 on Linux.

## GitHub's Windows runner and the scheduler tests

The pull request's run at `1d5c94b` ([43628379803](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43628379803)) failed one test on Windows only: `a_kill_comes_before_an_ordering_wait` reported "no result within 60s: the run deadlocked".
- **Slow, not stuck.** Three other tests in the same binary that have no guard ran past 60 s there and passed. The binary took 77 s; it takes under a second here.
- **Already slow before the sprint.** On that runner the binary took 46.6 s in the run at `38a6b96`, and 31.5 s in Sprint 6's merge run, before T-008 added the 1,000-process test. On Linux it takes 2.7 s.
- **No deadlock found here.** The binary was run 25 times on Windows while the Concurrency tests loaded the machine, 14 of them with the 1,000-process test in it, and every run passed.
- **The cause.** Each step of a simulated process hands the baton from one thread to another. On GitHub's Windows runners under the suite's load such a handoff is slow, and the 1,000-process test makes 3,001 of them in the same process as the guarded tests.
- **The response, in `5b65da8`.** The 1,000-process test moved to `crates/sim/tests/thousand.rs`, and the guards allow five minutes. That is still far inside the job's 30-minute limit, so a deadlock still fails rather than hangs.
- **The scheduler's logic is unchanged.** T-008's per-process wake was read again against every path that sets the next runner, and no change was made to it.
- **What GitHub's three samples at `5b65da8` showed,** read from each job's own log:
  - on Windows, `processes` took 22.6, 49.8 and 47.8 s, `thousand` 12.4, 7.8 and 22.9 s, and `concurrency` 113.0, 66.5 and 212.1 s;
  - on Linux, 0.5 to 0.8 s, 2.2 to 2.7 s and 46.8 to 75.6 s.

  The slowest run of the whole binary, 49.8 s, is a sixth of the 300 s that each guarded test may take, and the job's limit is 30 minutes. The times at `b951d6d` are in [e2e-tests.md](e2e-tests.md).
- **The mechanism is now tested** (test critique, C-011). `sim::Processes` counts the wakes in which a process woke while another was chosen to run, outside an abort. `a_thousand_processes_run` asserts at most one per 100 syscalls; it saw 1 on Windows and 0 in WSL over 3,002 syscalls. With every process woken at each boundary, the count is 1,819,674 and the test fails.

**The first WSL run found the one cross-platform difference.** At `38a6b96`, the head before the Test fixes, WSL ran 42 units, 656 passed and 2 failed: `lane_matrix_is_reproducible` and `concurrency_is_reproducible`. Both failed only at their digest literal, which had been taken on Windows. Their cells were equal between the two runs on Linux, and both matrices found no violation there.

The cause is a lane's `.praps/workspace`. It names the store's path as the system spells it, `repo\.praps\store` on Windows and `repo/.praps/store` elsewhere, and that file is part of every cell's state. GitHub's Linux job failed the same way and printed the same two digests as WSL. Each test now asserts one digest for Windows and one for other systems.

**Since the test critique, one digest is the same everywhere** (C-010). The lane matrix and both Concurrency runs also compute a portable digest, which reads every `\` in a lane file as `/`, and each test asserts it as one literal on every system. It equals the Linux digest, and it passed on Windows, in WSL and in both of GitHub's suites. So the separator in the store's path is the only difference between the two systems' states, in every cell.

**At each task's commit.** `cargo ci` passed on Windows before each task was committed. Wall times are not given, because other work ran on the machine during most of these runs.

| Task | Commit | `test:` total on Windows |
|---|---|---|
| T-002 | `609a8df` | 37 units, 593 passed |
| T-003 | `828ddea` | 38 units, 605 passed |
| T-004 | `ec33381` | 38 units, 614 passed |
| T-006 | `1a08bbc` | 40 units, 626 passed |
| T-005 | `2cf73b4` | 41 units, 635 passed |
| T-007 | `3aa5784` | 41 units, 643 passed |
| T-008 | `893c1a1` | 41 units, 644 passed |
| T-009 | `89f4f15` | 42 units, 654 passed |
| T-010 | `3eadc03` | 42 units, 656 passed |
| Test fixes | `1d5c94b` | 42 units, 659 passed |
| Test: the 1,000-process test apart | `5b65da8` | 43 units, 659 passed |
| Test: answers to the test critique, round 1 | `b951d6d` | 43 units, 663 passed |

- T-001 changed records only, and `cargo book` passed at its commit.
- T-006 was committed before T-005, which builds on its lanes.
- While T-009 was built, its first `cargo ci` failed twice:
  - `units_never_exceed_the_slots`, a test of `cargo ci`'s own runner, saw 2 units at once where it asserts 3, under the load the Concurrency tests add. The test now holds each unit until three are inside; T-009's evidence gives this.
  - A `clippy` lint in the new test.
- The Concurrency unit took 374 s beside the other units while its negative controls ran all 320 cells each. With each control stopping at its first caught cell, it took 63 s at T-009's commit.

## The matrices

Seven matrices ran with `--nocapture` on both systems at the tested head. Their result lines are in [test-matrix-log.md](test-matrix-log.md).
- **The five from earlier sprints keep their cell counts:** 32,840, 44,836, 24,320, 34,400 and 28,720, as at `855eb67`. T-002 moved all five digests and T-004 the Materialise digest again, as those tasks say; each digest is equal on the two systems.
- **`b951d6d` moved no count and no digest.** Its new checks found no violation, and every cell's state and trace are as at `5b65da8`.
- **The correct variants find 0 violations** in all seven, and every negative control is caught.

## The Concurrency scenario

The details are in the matrix log. Since the test critique, each cell also checks:
- that each lane loads, as its own entry, the entry of its last acknowledged op, where that op is still the lane's writer (C-003);
- after the next command, that each lane's `.praps` holds its lane file and no store, and that no object lies outside the stores (C-009);
- in the two-store run, after the next command, that a second exchange brings both stores and every lane to one view, so ops made on top of received ops are checked too (C-009).

On both systems:
- **One store** (INT-0014 AC-3): 160 cells, 0 violations. 112 processes killed, 590 commands stopped by the crash, and 66 operations of a command in flight that landed.
- **Two stores that then exchange** (INT-0010 AC-5): 160 cells, 0 violations; 116 killed, 594 stopped, 182 landed.
- **Reproducibility:** each run is repeated, and every cell's state, trace, violations, coverage and reach are equal.
- **Readers** (INT-0010 AC-7): 2, 8 and 1,000 processes over three op heads leave the op files as they were, and all compute the merge of the heads.
- **The controls:** `FirstHeadOnly`, `MergeOnRead` and `UnsyncedObjects` are each caught in the first cell, by the oracle each breaks. A fourth, `UnsyncedObjects` again, stops only where an acknowledged op is lost (C-003). Its one-store run breaks every cell but loses no acknowledged op; in the two-store run, the first cell loses 29.
- **Every control over all its cells.** With `PRAPS_FULL_CONTROLS=1` each control runs all 320 cells. At the tested head: `FirstHeadOnly` is caught in 308 of 320 cells, `MergeOnRead` in 180 and `UnsyncedObjects` in all 320, and the lost-op check fires 3,632 times in the two-store run. Both systems printed the same lines; they are in the matrix log.

## Cost before and after

A record, not a test: it traces to no clause and cannot fail the sprint. It was measured at `5b65da8`. `b951d6d` changes only how an unreadable lane file is reported, on no path the probe takes.

Sprint 6's probe, run with the release binary at `855eb67` and at the tested head on both systems, with one more column the test plan asks for. It times `status` with nothing changed, and the two lists, as the op log grows to 800 operations, on a working copy of 100 files: each figure is the median of 5. It then runs `status` on repositories whose op log has 2, 8 and 32 heads.
- **The stores with several heads** were made at each commit by a throwaway test, not in the repository. It opens that many handles over one head and has each describe the change, so each commits over the same head.
- **The script and the helper:** `probe7.py`, Sprint 6's probe with the heads column added, and `zz_heads.rs`. Both are given at the end.

**WSL (ext4), on a quiet machine:**

| Ops | `status` at `855eb67` | `status` now | op list at `855eb67` | op list now | change list at `855eb67` | change list now |
|---:|---:|---:|---:|---:|---:|---:|
| 10 | 4.6 ms | 4.9 ms | 4.7 ms | 4.3 ms | 4.5 ms | 3.7 ms |
| 50 | 8.1 | 8.4 | 7.3 | 7.3 | 6.2 | 6.1 |
| 100 | 8.5 | 7.7 | 11.0 | 9.5 | 8.8 | 7.9 |
| 200 | 9.7 | 10.4 | 15.2 | 14.7 | 11.4 | 9.8 |
| 400 | 14.8 | 17.7 | 27.8 | 26.5 | 17.9 | 17.7 |
| 800 | 27.6 | 32.4 | 50.3 | 59.7 | 31.7 | 39.4 |

| Op heads | `status` at `855eb67` | `status` now |
|---:|---|---|
| 2 | fails, exit 1: "the op log has 2 heads, which cannot be merged yet" | 4.5 ms, exit 0, writes nothing |
| 8 | fails, exit 1, the same | 4.7 ms, exit 0, writes nothing |
| 32 | fails, exit 1, the same | 5.6 ms, exit 0, writes nothing |

**Windows (NTFS):** on the same machine with nothing else running, after the matrices:

| Ops | `status` at `855eb67` | `status` now | op list at `855eb67` | op list now | change list at `855eb67` | change list now |
|---:|---:|---:|---:|---:|---:|---:|
| 10 | 95.9 ms | 76.5 ms | 99.2 ms | 77.9 ms | 99.9 ms | 74.1 ms |
| 50 | 115.1 | 99.6 | 131.2 | 123.9 | 100.5 | 102.2 |
| 100 | 127.0 | 118.8 | 218.5 | 176.0 | 145.5 | 129.5 |
| 200 | 314.3 | 153.6 | 305.8 | 259.3 | 223.9 | 173.1 |
| 400 | 242.7 | 221.5 | 503.9 | 477.8 | 344.9 | 316.9 |
| 800 | 553.3 | 411.3 | 994.1 | 569.5 | 569.7 | 390.0 |

| Op heads | `status` at `855eb67` | `status` now |
|---:|---|---|
| 2 | fails, exit 1: "the op log has 2 heads, which cannot be merged yet" | 64.2 ms, exit 0, writes nothing |
| 8 | fails, exit 1, the same | 68.0 ms, exit 0, writes nothing |
| 32 | fails, exit 1, the same | 78.8 ms, exit 0, writes nothing |

A first Windows run, made while WSL ran its matrices, gave the same picture, with every figure higher.

- **Several heads no longer stop a command.** At `855eb67` every `status` on a repository with several op heads failed; now it works from their merge, writes no op file, and costs about what a single head costs.
- **Up to 200 operations the two binaries are level in WSL.** At 800, the new one is slower there in all three columns: `status` 32.4 against 27.6 ms, the op list 59.7 against 50.3, the change list 39.4 against 31.7, between 17 and 24 percent.
- **On Windows the new binary is no slower at any size.** There a command's time is mostly starting the process, and the times vary by tens of percent from run to run.
- **The likely cause is size.** An operation now names its workspace, and a view holds an entry for each workspace with its pointer and heads, so each object read is larger. Every load also checks that no op names a missing parent.
- **A view whose size follows the change is backlog T-038,** and the read that grows with the history is T-030. Each figure is one machine's median and a record, not a test.

## What was not run

- **Nighthawk.** The Debian machine still needs the owner to approve Tailscale access (INT-0004 AC-6).
- **Real replicas over a network.** The two stores of the Concurrency scenario share one simulated filesystem, and exchange their files twice: after recovery, and after the next command. A store that syncs with another again and again belongs to INT-0024.

## The probe's script and helper

`probe7.py`, run as `python probe7.py PRAPS_BINARY WORK_DIR HEADS_DIR` (on Linux with `python3`):

```python
"""Sprint 7 cost probe: Sprint 6's probe (`status` on an unchanged working
copy of 100 files, and the listings, against the length of the op log),
then `status` with nothing changed on repositories with 2, 8 and 32 op heads.

Usage: python probe7.py PRAPS_BINARY WORK_DIR HEADS_DIR

HEADS_DIR holds heads-2, heads-8 and heads-32, made for this binary's format
by the throwaway test zz_heads.rs. Everything else is written under WORK_DIR.
"""
import os
import random
import shutil
import statistics
import subprocess
import sys
import time

PRAPS, BASE, HEADS = sys.argv[1], sys.argv[2], sys.argv[3]


def run(cmd, cwd, check=True):
    start = time.perf_counter()
    done = subprocess.run(cmd, cwd=cwd, capture_output=True)
    took = time.perf_counter() - start
    if check and done.returncode != 0:
        raise SystemExit(f"{cmd} failed: {done.stderr.decode(errors='replace')[:300]}")
    return took, done


def median_of(n, cmd, cwd):
    return statistics.median(run(cmd, cwd)[0] for _ in range(n))


def populate(root, files, size=2048, seed=1):
    rng = random.Random(seed)
    words = [f"word{n}" for n in range(500)]
    for n in range(files):
        text, length = [], 0
        while length < size:
            line = " ".join(rng.choice(words) for _ in range(8))
            text.append(line)
            length += len(line) + 1
        with open(os.path.join(root, f"f{n:06d}.txt"), "w", newline="\n") as out:
            out.write("\n".join(text) + "\n")


p = os.path.join(BASE, "history")
if os.path.exists(p):
    shutil.rmtree(p)
os.makedirs(p)
populate(p, 100)
run([PRAPS, "init"], p)
run([PRAPS, "status"], p)
made = 2
print("ops   status(unchanged) ms   op list ms   change list ms")
for target in (10, 50, 100, 200, 400, 800):
    while made < target:
        with open(os.path.join(p, f"f{made % 100:06d}.txt"), "a", newline="\n") as out:
            out.write(f"edit {made}\n")
        run([PRAPS, "new"], p)
        made += 2
    status = median_of(5, [PRAPS, "status"], p)
    oplist = median_of(5, [PRAPS, "ops"], p)
    changes = median_of(5, [PRAPS, "log"], p)
    print(f"{made:4d}   {status * 1000:10.1f}            {oplist * 1000:9.1f}        {changes * 1000:9.1f}")
shutil.rmtree(p)

print()
print("op heads   status(unchanged)")
for n in (2, 8, 32):
    root = os.path.join(HEADS, f"heads-{n}")
    ops = os.path.join(root, ".praps", "store", "ops")
    before = sorted(os.listdir(ops))
    took, done = run([PRAPS, "status"], root, check=False)
    if done.returncode != 0:
        why = done.stderr.decode(errors="replace").strip().splitlines()[0]
        print(f"{n:8d}   fails, exit {done.returncode}: {why}")
        continue
    times = [took] + [run([PRAPS, "status"], root)[0] for _ in range(4)]
    after = sorted(os.listdir(ops))
    wrote = "wrote nothing" if after == before else f"op files {len(before)} -> {len(after)}"
    print(f"{n:8d}   {statistics.median(times) * 1000:.1f} ms, exit 0, {wrote}")
```

`zz_heads.rs`, copied into `crates/workspace/tests/` at each commit, run with `PROBE_DIR` set, and removed again:

```rust
//! Throwaway, for the Sprint 7 cost probe; not part of the repository.
//! With `PROBE_DIR` set, makes `heads-2`, `heads-8` and `heads-32` there:
//! each a repository of 100 files whose op log has that many heads, made by
//! that many handles that each opened over one head and then described the
//! change. The same file builds at `855eb67` and at Sprint 7's head.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::PathBuf;

use sim::RealFs;
use workspace::Workspace;

#[test]
fn make_repositories_with_heads() {
    let Some(base) = std::env::var_os("PROBE_DIR") else {
        return;
    };
    let base = PathBuf::from(base);
    for n in [2_usize, 8, 32] {
        let root = base.join(format!("heads-{n}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        for i in 0..100 {
            let text = format!("file {i} line\n").repeat(150);
            std::fs::write(root.join(format!("f{i:06}.txt")), text).unwrap();
        }
        let (mut ws, _) = Workspace::init(RealFs, &root, 1).unwrap();
        ws.new_change().unwrap();
        drop(ws);
        let mut handles: Vec<_> = (0..n)
            .map(|i| Workspace::open(RealFs, &root, 10 + i as u64).unwrap())
            .collect();
        for (i, ws) in handles.iter_mut().enumerate() {
            ws.describe(format!("head {i}").as_bytes()).unwrap();
        }
        drop(handles);
        let store = store::Store::open(
            RealFs,
            &root.join(".praps").join("store"),
            hash::HashAlg::DEFAULT,
            0,
        )
        .unwrap();
        let heads = store.heads().unwrap().len();
        println!("heads-{n}: {heads} op heads");
        assert_eq!(heads, n);
    }
}
```
