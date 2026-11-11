# Sprint 6 integration test results

Tested head: `bbffce6`. Windows 11 (NTFS) ran in the working checkout. WSL Ubuntu (Linux 6.6.114.1-microsoft-standard-WSL2, ext4) ran in a clone on ext4 with its own target directory. The clone was made fresh for the first run, at `80a3e7e`, and then checked out at `4152526` and at `bbffce6` in turn, so only its first run built from nothing. rustc 1.99.1 and cargo-deny 0.20.2 on both. No two timed runs overlapped.

## The suite on both platforms

| | Windows | WSL |
|---|---|---|
| `cargo ci` | all 6 steps passed | all 6 steps passed |
| Wall time | 288 s | 186 s (the first run in the clone, a cold build at `80a3e7e`, took 209 s) |
| `test:` line | 37 units, 588 passed, 0 failed, 0 ignored | 37 units, 590 passed, 0 failed, 0 ignored |
| Executables | 46 built, 46 run | 46 built, 46 run |
| Tree after the run | clean | clean |
| `cargo book`, first line | `book: docs (typed-acceptance 1, strict)` | the same |
| `cargo book`, last line | `38 chapters, 267 criteria, 0 violations` | the same |
| `cargo deny check` | `advisories ok, bans ok, licenses ok, sources ok` | the same |
| `cargo reach`, then `git status` | wrote `docs/reach.md` with no difference | wrote `docs/reach.md` with no difference |

- **Totals against Sprint 5.** 501 and 502 at `b0675f6`; 588 and 590 now. The difference is the 88 test functions this sprint adds, one of them for Unix only. No test was removed, and the eight renamed tests are listed in [unit-tests.md](unit-tests.md).
- **The store matrices.** `matrix_runs_are_reproducible` and `writers_matrix_is_reproducible` passed with Sprint 5's literals on both systems.
- **The slowest units on Windows:** `materialise_crash` 285 s, `describe_crash` 166 s, `crash` 162 s, `writers` 118 s, `cli` 63 s. The new unit, `describe_crash`, is the second slowest.

**At each task's commit.** `cargo ci` passed on Windows before each task was committed. The times were taken while other work ran on the machine at some commits, so they are not a series to compare.

| Task | Commit | `test:` total | Wall time |
|---|---|---|---|
| T-002 | `c639114` | 36 units, 504 passed | 343 s |
| T-003 | `0ca3cea` | 36 units, 510 passed | 343 s |
| T-004 | `83d0f6d` | 36 units, 511 passed | 341 s |
| T-005 | `91e6a2f` | 36 units, 512 passed | 334 s |
| T-006 | `6264d34` | 36 units, 533 passed | 291 s |
| T-007 | `766b106` | 36 units, 536 passed | 273 s |
| T-008 | `510c3e0` | 36 units, 551 passed | 288 s |
| T-009 | `6eb6d93` | 36 units, 556 passed | 279 s |
| T-010 | `0e2693b` | 37 units, 571 passed | 354 s |
| T-011 | `4adba79` | 37 units, 579 passed | 383 s |
| T-012 | `11bace2` | 37 units, 586 passed | 366 s |
| T-013 | `92f4e22` | 37 units, 587 passed | 304 s |

- T-001 changed only records, and `cargo book` passed at its commit.
- T-005's follow-up `d4137e4` changed one sentence of the spec and one comment. An edit script had failed without a visible error, so the task's own commit lacked them; the completion entry says so.
- Three commits follow T-013, all made during Test: `80a3e7e` adds one case to a test; `4152526` holds the fixes from the code review; `bbffce6` holds the tests the first test critique asked for. `cargo ci` passed on both systems at each: 587 and 589 at the first two, in 327 s and 316 s on Windows and 209 s and 188 s in WSL.

In WSL, at each task from T-008 on, the `praps` tests were also run, and from T-010 the tests of the working copy that the task touched. Each completion entry gives its counts.

## The matrices

`snapshot_matrix_is_reproducible`, `materialise_matrix_is_reproducible` and `describe_matrix_is_reproducible` assert their cell counts and digests as literals, and passed on both systems. [test-matrix-log.md](test-matrix-log.md) holds the counts before and after, the controls, and all 39 result lines, which are the same on both systems.

**The reach table.** `docs/reach.md` gained the row `Describe (S6)` and the column `S6 workspace::describe` with 31,840 hits. Every earlier row kept its counts. `reach_table_shows_the_triangle` checks that Describe passes back through `workspace::snapshot` and the store's units, and that no earlier scenario reaches the new unit.

## Cost before and after

This is a record, not a test. It traces to no clause: [INT-0036](../../../intents/INT-0036-measured-speed.md) is not planned and its suite does not exist.

The probe and the simulator count were run at `4152526`. `bbffce6` changes tests only.

**The probe.** A working copy of 100 files of 2 KiB. Each step edits one file and runs `praps new`, which adds two operations and one change. At each length the probe takes the median of five runs of `status` with nothing changed, of the operation list (`log` in the Sprint 5 binary, `ops` now) and, at the head, of the change list (`log`). Both binaries are release builds, run one after the other on the same machine in the same session. The script is at the end of this file. Its command line is `python probe6.py BINARY WORK_DIR OPLIST_COMMAND [CHANGES_COMMAND]`: `log` for the Sprint 5 binary, and `ops log` for the head's.

Milliseconds, Windows (NTFS):

| Operations | `status`, Sprint 5 | `status`, head | Operation list, Sprint 5 | Operation list, head | Change list, head |
|---|---|---|---|---|---|
| 10 | 86.5 | 34.5 | 58.4 | 36.2 | 35.2 |
| 50 | 72.7 | 41.6 | 185.9 | 52.7 | 43.0 |
| 100 | 85.7 | 49.7 | 371.5 | 75.0 | 53.1 |
| 200 | 149.7 | 63.4 | 1,293.7 | 102.1 | 72.5 |
| 400 | 251.6 | 99.9 | 6,012.9 | 173.7 | 111.8 |
| 800 | 401.4 | 158.8 | 21,597.8 | 372.8 | 197.2 |

Milliseconds, WSL (ext4):

| Operations | `status`, Sprint 5 | `status`, head | Operation list, Sprint 5 | Operation list, head | Change list, head |
|---|---|---|---|---|---|
| 10 | 3.2 | 2.4 | 3.5 | 2.5 | 2.4 |
| 50 | 5.3 | 3.5 | 13.8 | 3.9 | 3.2 |
| 100 | 7.7 | 4.0 | 39.2 | 5.7 | 4.3 |
| 200 | 11.7 | 5.6 | 136.5 | 9.0 | 6.2 |
| 400 | 76.8 | 9.0 | 570.6 | 16.0 | 10.5 |
| 800 | 63.8 | 16.2 | 1,825.0 | 29.8 | 18.5 |

What the figures say, and what they do not:
- **The operation list no longer grows with the square of the history.** At 800 operations it took 21.6 s on Windows and now takes 0.37 s; on Linux 1.8 s and now 0.03 s.
- **`status` on an unchanged working copy is about 2.5 times faster at 800 operations on Windows, and about 4 times on Linux.** It still grows with the history: it reads every op file once, and every change of the chain. That is the storage form's work ([INT-0008](../../../intents/INT-0008-compact-history-storage.md), backlog T-030).
- **Nothing here compares Periapsis with another system.** The research report's comparison with Git stands: recording many new files is still slower than Git, because each file is still flushed twice. This sprint did not touch that.
- **These are single sessions on one machine.** The first run of the head's probe on Windows was disturbed at 200 and 400 operations (about 1 s for every command there, then 152 ms at 800), while other commands ran on the machine. It was run again with nothing else running, and the table gives that second run. The Sprint 5 column on Linux shows the same kind of noise at 400 operations (76.8 ms, then 63.8 at 800).

**On the simulator.** `open` and one `status` on a working copy with one small file, by the length of the op log. A throwaway test counted the trace, at `b0675f6` and at the head, in WSL.

| Operations | Syscalls, Sprint 5 | Syscalls, head | `ops/` listed, Sprint 5 | head | Reads of op files, Sprint 5 | head | Reads of objects, Sprint 5 | head |
|---|---|---|---|---|---|---|---|---|
| 5 | 42 | 23 | 7 | 2 | 15 | 5 | 6 | 2 |
| 50 | 177 | 68 | 7 | 2 | 150 | 50 | 6 | 2 |
| 500 | 1,527 | 518 | 7 | 2 | 1,500 | 500 | 6 | 2 |

Each op file was read three times and is now read once. Of the two listings left, one is the store checking that `ops/` exists. In this history every snapshot replaces the same root change, so there is no chain of changes to read; the probe above, which starts a new change at every step, shows that cost.

## What was not run

- `cargo ci` was not run a second time in WSL from a cold build at the head, and no run was repeated for a spread of times.
- Nothing ran on Nighthawk, the Debian machine: its Tailscale access still waits for the owner (INT-0004 AC-6).
- GitHub's runs are in [e2e-tests.md](e2e-tests.md).

## The probe's script

```python
"""Sprint 6 cost probe: `status` on an unchanged working copy of 100 files,
and the listings, against the length of the op log.

Usage: python probe6.py PRAPS_BINARY WORK_DIR OPLIST_COMMAND [CHANGES_COMMAND]

OPLIST_COMMAND is the command that lists operations: `log` for the Sprint 5
binary, `ops` from Sprint 6. CHANGES_COMMAND, if given, is `log`, the list
of changes, which only Sprint 6 has. Everything is written under WORK_DIR.
"""
import os
import random
import shutil
import statistics
import subprocess
import sys
import time

PRAPS, BASE, OPLIST = sys.argv[1], sys.argv[2], sys.argv[3]
CHANGES = sys.argv[4] if len(sys.argv) > 4 else None


def run(cmd, cwd):
    start = time.perf_counter()
    done = subprocess.run(cmd, cwd=cwd, capture_output=True)
    took = time.perf_counter() - start
    if done.returncode != 0:
        raise SystemExit(f"{cmd} failed: {done.stderr.decode(errors='replace')[:300]}")
    return took


def median_of(n, cmd, cwd):
    return statistics.median(run(cmd, cwd) for _ in range(n))


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
head = "ops   status(unchanged) ms   op list ms"
print(head + ("   change list ms" if CHANGES else ""))
for target in (10, 50, 100, 200, 400, 800):
    while made < target:
        # An edit that `new` records (one op), then the op `new` commits.
        with open(os.path.join(p, f"f{made % 100:06d}.txt"), "a", newline="\n") as out:
            out.write(f"edit {made}\n")
        run([PRAPS, "new"], p)
        made += 2
    status = median_of(5, [PRAPS, "status"], p)
    oplist = median_of(5, [PRAPS, OPLIST], p)
    line = f"{made:4d}   {status * 1000:10.1f}            {oplist * 1000:9.1f}"
    if CHANGES:
        line += f"        {median_of(5, [PRAPS, CHANGES], p) * 1000:9.1f}"
    print(line)
shutil.rmtree(p)
```

The simulator count is this test, put in `crates/workspace/tests/` for the run and removed after it:

```rust
//! Throwaway count for the Sprint 6 test record: what one `status` costs on
//! the simulator, by the length of the op log. Not part of the repository.

use std::path::Path;
use std::rc::Rc;

use sim::{Fs, Mode, SimFs};
use workspace::Workspace;

#[test]
fn status_cost_by_history() {
    for ops in [5usize, 50, 500] {
        let fs = Rc::new(SimFs::new(1, Mode::Strict));
        let root = Path::new("repo");
        let mut ws = Workspace::init(Rc::clone(&fs), root, 1).unwrap().0;
        let file = root.join("f");
        fs.create_new(&file).unwrap();
        // The root op is one; each round adds one.
        for round in 1..ops {
            fs.overwrite(&file, format!("{round}").as_bytes()).unwrap();
            ws.snapshot().unwrap();
        }
        drop(ws);
        let before = fs.trace().len();
        let mut ws = Workspace::open(Rc::clone(&fs), root, 2).unwrap();
        ws.status().unwrap();
        let trace = fs.trace();
        let part = &trace[before..];
        let lists = part
            .iter()
            .filter(|line| *line == "list repo/.praps/store/ops")
            .count();
        let reads = part
            .iter()
            .filter(|line| line.starts_with("read repo/.praps/store/ops/"))
            .count();
        let objects = part
            .iter()
            .filter(|line| line.starts_with("read repo/.praps/store/objects/"))
            .count();
        println!(
            "open + status at {ops} ops: {} syscalls; ops/ listed {lists} times; {reads} reads of op files; {objects} reads of objects",
            part.len()
        );
    }
}
```
