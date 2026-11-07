# Sprint 1 End-to-End Test Results

- **Status:** possible, and run, at tested head `9f138f4` ([test-matrix-log.md](test-matrix-log.md)).

### praps_fsck_end_to_end
- **Intents:**
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-4
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1, AC-5
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-8
- **Run:** the built `praps` binary as a subprocess (`crates/praps/tests/cli.rs`), on stores built through `RealFs` on NTFS (Windows) and on ext4 (WSL).
- **Steps:**
  1. `praps fsck repo` on a healthy store prints `clean` and exits 0, and a second run says the same.
  2. One op file truncated by hand and one bit of an object flipped by hand: both files are listed, exit 1.
  3. Both restored: exit 0.
  4. A leftover `tmp-*` file is reported as harmless, exit 0.
  5. On a directory that is not a store: exit 1, with a message on stderr. Usage errors (no arguments, `--help`, `-h`, a flag, an unknown subcommand, an extra argument) exit 2 with the usage line.
- **Result:** 5 of 5 passed on Windows and 5 of 5 on Linux. On both systems, `real_store_reopens_clean`, `write_sequence_succeeds_on_the_real_filesystem` and `directory_flush_needs_write_access` passed with them (sections `e2e praps_fsck_end_to_end`).

### fresh_clone_ci
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
- **Run:** a fresh `git clone --branch dev` into the WSL ext4 home directory. Then `rustup toolchain install` from `rust-toolchain.toml`, then a cold `cargo ci`.
- **Result:**
  - Every step PASS, exit 0 in 358 s, on a busy host.
  - 242 tests passed.
  - The run left no tracked file changed. The clone was removed afterwards.

### Not yet possible
- **Power-loss crashes:** would need hardware fault injection, which isn't planned. The simulator's crash matrix and the documented Windows strategy stand in, and INT-0004 AC-5 is an `asserted` criterion for the reviewer.
- **Nighthawk runs:** unlocked by the owner approving the Tailscale SSH check (INT-0004 AC-6).
- **Writers in separate processes:** unlocked by a later sprint on INT-0007 AC-5. Two writers in one process are covered: writer B runs at one of eight syscall boundaries inside writer A's put of an object, chosen by the seed, and 70 cells per mode run the race the re-sync guards.
- **GitHub Actions on this sprint's head:** runs when `dev` is pushed at the Sprint 1 checkpoint. The next sprint records it, as Sprint 1's T-009 did for Sprint 0 ([ci-sprint0.md](ci-sprint0.md)).
