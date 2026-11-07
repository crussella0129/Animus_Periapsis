# Sprint 1 Integration Test Results

At tested head `499fb8c`. Every run is in [test-matrix-log.md](test-matrix-log.md), under the section named here.

### Canonical runner on the Windows host and in WSL
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-1 to AC-4
  - [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-1 to AC-4

| Run | System | Result |
|---|---|---|
| `int_ci_windows` | Windows 11 Pro 10.0.26300, NTFS working checkout, rustc 1.99.1 | `cargo ci`: PASS fmt, clippy, test, policy, book; exit 0 in 87 s; 240 tests passed |
| `e2e fresh_clone_ci` | Ubuntu 26.04.1 (WSL2), ext4 fresh clone, rustc 1.99.1 | cold `cargo ci`: every step PASS, exit 0 in 64 s; 240 tests passed; 0 tracked files changed |
| `int_crash_matrix_windows`, `int_crash_matrix_linux` | both | The correct store had 0 violations of any kind in 9,344 cells per mode, 70 of which ran the two-writer race. Negative controls, strict mode: pointer-file had 5,985 heads violations in 10,576 cells; unsynced-init had 20,979 durability and 1,425 closure in 8,544; unsynced-objects had 6,643 closure, 9,412 durability and 9,535 torn in 8,547; write-in-place had 563 torn in 8,600. Lenient mode catches every control except unsynced-init (0: lenient makes `create_dir` durable at once). The counts are identical on both systems |
| Cross-system determinism | both | The digest over all 18,688 cells is `blake3:4e4d9e8bbdfa2396fa5b39085f62854fb3640280205ed2f4b9d6396648d99c59` on Windows and on Linux: the simulator never touches the real filesystem, so every recovered state and trace agrees |
| `int_fsck_corpus_windows`, `int_fsck_corpus_linux` | both | 243 of 243 truncations and 40 of 40 bit flips detected; 0 reports on 16,256 recovered stores |
| `int_deny_windows` | Windows, cargo-deny 0.20.2 | advisories ok, bans ok, licenses ok, sources ok; no new dependency this sprint |
| `cargo policy` (inside `cargo ci`) | both | every rule PASS, including the new `fs-boundary` and the extended `deps` |

### Probes
- **Intents:**
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
  - [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)

| Probe | System | Result |
|---|---|---|
| `CARGO_TARGET_DIR` set outside the checkout, then `cargo ci` (T-008) | Windows | exit 1, `xtask: …\outside-target\debug\xtask.exe is not inside C:\Users\charl\Animus_Periapsis\target; …`, 0 steps run |
| the same | Linux | exit 1, `xtask: /home/charles/periapsis-outside-target/debug/xtask is not inside /home/charles/periapsis-s1-test/target; …`, 0 steps run |
| `std::fs::read` and `Path::exists`, reached through `use std::{fs, io}`, planted in `store/src` (T-003) | Windows | Clippy rejects both as disallowed methods, exit 101; file restored |
| Mutation probes P1 and P2 of the store's two durability layers ([code-review.md](code-review.md)) | Windows | each makes `crash_matrix_finds_no_violations` fail |

The environment variables that could change a run (`PROPTEST_*`, `PERI_*`, `CLIPPY_CONF_DIR`, `RUSTFLAGS`, `RUSTC*`, `RUSTUP_TOOLCHAIN`, `CARGO_BUILD_*`, `CARGO_TARGET_*`, `CARGO_PROFILE_*`) were unset on both systems.
