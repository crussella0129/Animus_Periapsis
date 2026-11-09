# Sprint 4 End-to-End Test Results

- **Status:** possible, and run at tested head `5dfb418` ([test-matrix-log.md](test-matrix-log.md)).

### real_four_processes_lose_nothing
- **Intents:**
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-5
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
- **Run:** `crates/store/tests/real_writers.rs`, on the real filesystem.
  1. The parent `init`s a store in a temporary directory and commits a base op.
  2. It starts its own test binary four times at once, as children, and only then waits for each.
  3. Each child opens the store with `RealFs` and commits 25 ops of its own content, each on its previous op (the first on the base). It prints an `ACK` line for each acknowledged op, with that op's parent.
  4. A child writes only when its exact test name was passed and `PERI_WRITER_CHILD` is set. `cargo ci` scrubs that variable.
- **Passes when:**
  - every child exits 0 and reports 25 ops: 100 in all;
  - every reported op is present, with exactly the parent it reported, and hashes to its name;
  - the store holds 101 ops: the 100, plus the base;
  - `fsck` is clean;
  - `store.heads()` equals the four heads computed from the reports.
- **Result:**

  | System | Filesystem | Section | Result | Time |
  |---|---|---|---|---:|
  | Windows | NTFS | `e2e real_four_processes_lose_nothing (NTFS)` | passed | 2.30 s |
  | Linux (WSL) | ext4 | `e2e real_four_processes_lose_nothing (ext4)` | passed | 0.05 s |

  It also passed in every `cargo ci` run on both systems, and three times on NTFS during the build. The ext4 time is short, but it is not a skip: the test fails unless 100 acknowledged ops come back from four child processes.
- **What it does not show:**
  - The kernel chooses the interleaving, so no run reproduces.
  - The children never name each other's ops, so the parent race is not exercised here. The four-writer matrix forces that race in every odd seed ([integration-tests.md](integration-tests.md)).
  - No crash is injected. Crashes are the simulator's job; power loss is out of scope by INT-0007's Consequences.

### praps_commands_report_skipped_names
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-7, through T-009 (backlog T-019).
- **Run:** the built `praps` binary as a subprocess (`crates/praps/tests/e2e.rs`). A file is named with no UTF-8 form:
  - on NTFS, an unpaired surrogate, `q\u{d800}`;
  - on ext4, byte `0xff`, `q\xff`.
- **Steps:**
  1. `praps undo` with nothing to undo exits 1, and its stdout is only `skipped <name> (name is not UTF-8)`.
  2. `new`, `log`, `undo` and `restore <prefix>` each exit 0 and print their own lines first, then the report line, once.
  3. The file is byte for byte untouched.
- **Result:** passed on Windows (section `e2e praps skipped-name reports and Sprint 3 CLI (NTFS)`) and on Linux (in `cargo ci`'s test step, section `e2e fresh_clone_ci`).

  Sprint 3's CLI tests ran unchanged and passed: 22 in `cli.rs` and the 4 other `e2e.rs` tests, `praps_end_to_end` and `praps_history_end_to_end` among them. So a command whose snapshot skips nothing prints what it printed before.

### fresh_clone_ci
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
- **Run:** a fresh clone of `dev` at `5dfb418` into the WSL ext4 home directory. Then a cold `cargo ci`, with every override variable unset.
- **Result:**
  - all six steps PASS, exit 0 in 1,351 s;
  - 474 tests;
  - five reach files written;
  - 0 tracked files changed;
  - `cargo reach` afterwards changed nothing.

  The clone was removed afterwards.

### Not yet possible
- **Power loss:** out of scope by INT-0007's Consequences. The simulator's crash model stands in.
- **Nighthawk:** waiting for Tailscale approval (INT-0004 AC-6).
- **Several workspaces:** INT-0014 AC-3, which comes after INT-0010's merge.
- **A process killed on a real filesystem:** the kill model runs only in the simulator (`sim::Processes`). Killing a real child mid-write would test the operating system's page cache, which INT-0007 already relies on through `fsync`.
