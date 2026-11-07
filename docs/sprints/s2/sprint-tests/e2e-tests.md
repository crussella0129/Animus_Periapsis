# Sprint 2 End-to-End Test Results

- **Status:** possible, and run, at tested head `c5c6965` ([test-matrix-log.md](test-matrix-log.md)).

### praps_end_to_end
- **Intents:**
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1, AC-2
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-4
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
- **Run:** the built `praps` binary as a subprocess (`crates/praps/tests/e2e.rs`), in a temporary directory on NTFS (Windows) and on ext4 (WSL).
- **Steps:**
  1. `praps init project`.
  2. Write `README.md`, `src/main.rs`, a `.gitignore` of `target/` and `*.log`, `target/debug/app` and `run.log`.
  3. `praps status` lists exactly `A .gitignore`, `A README.md` and `A src/main.rs`.
  4. A second `praps status` gives the same answer, and the op log does not grow.
  5. `praps new`, then `praps status` is empty.
  6. Edit `README.md`, delete `src/main.rs`, add `src/lib.rs` and `notes.log`. `praps status` lists `M README.md`, `A src/lib.rs` and `D src/main.rs`.
  7. `praps fsck` (in the repository) and `praps fsck project` (from outside) both print `clean`.
- **Result:** passed on Windows and on Linux (sections `e2e praps`).

### praps_status_reports_a_non_utf8_name
- **Intents:**
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-7
  - [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-1
- **Run:** a file whose name has no UTF-8 form, made with the platform's own API: `r\xffw` on ext4 (a byte name), and `r` + U+D800 + `w` on NTFS (an unpaired surrogate). Beside it is a plain file.
- **Result:**
  - `praps status` prints `A plain` and `skipped r\xffw (name is not UTF-8)` on ext4, or `skipped r\u{d800}w (name is not UTF-8)` on NTFS.
  - The file keeps its content and is never tracked, through a second `status`.
  - `fsck` is clean.
  - Passed on both systems.

### praps_status_reports_non_utf8_names_and_directories
- **Intents:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-7
- **Run:** three more names with no UTF-8 form per platform, made with the platform's own API:
  - on ext4, a leading, a trailing and a truncated invalid byte;
  - on NTFS, a leading surrogate, two surrogates and a trailing surrogate.

  The last is a directory holding a file. There is also a plain sibling (test critique C-005).
- **Result:**
  - `praps status` prints `A sibling` and one `skipped … (name is not UTF-8)` line per name, in escaped form.
  - The directory's file is never tracked, and every file keeps its content.
  - `fsck` is clean.
  - Passed on both systems.

### The CLI and real-filesystem suites
Sections `e2e praps_fsck_end_to_end`, on both systems:
- **`praps` (`tests/cli.rs`):** 14 of 14 passed. They are the 9 new init, new, status, fsck and usage tests (including `praps_fsck_reports_a_broken_repository_store`) and the 5 Sprint 1 fsck tests.
- **Sprint 1 real-filesystem tests:** `real_store_reopens_clean`, `write_sequence_succeeds_on_the_real_filesystem` and `directory_flush_needs_write_access` passed again.
- **`sim` real-filesystem scan tests:** 5 of 5 passed, including `real_fs_keeps_names_win32_would_rewrite`. On NTFS it reads `foo.`, `foo `, `aux.txt`, `CON` and `nul.c` as themselves, through verbatim paths (code review CR-02).

### fresh_clone_ci
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
- **Run:** a fresh `git clone --branch dev` into the WSL ext4 home directory, then `rustup toolchain install`, then a cold `cargo ci`.
- **Result:** every step PASS, exit 0 in 1,181 s (the Windows leg ran at the same time), with 332 tests passed. The run changed no tracked file, and the clone was removed afterwards.

### Not yet possible
- **Power-loss crashes:** they would need hardware fault injection. The simulator's crash matrices stand in, under a fault model at least as harsh as the filesystems they model ([INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)).
- **Nighthawk:** waiting for the owner's Tailscale approval ([INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-6).
- **Several processes and several workspaces:** unlocked by backlog T-011, together with [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) and [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) AC-3.
- **`status` on 100,000 files within a fixed time budget:** unlocked by the stat cache (INT-0014 AC-5).
- **This head's GitHub Actions run:** it happens when `dev` is pushed at the checkpoint, and the next sprint records it (backlog T-013).
