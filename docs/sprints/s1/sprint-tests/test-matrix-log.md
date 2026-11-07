# Sprint 1 Test Matrix Log

The output of the two matrix scripts at the tested head `9f138f4`, run on 2026-11-06, the final run of the Test phase. The Windows leg ran in the clean working checkout; the Linux leg ran in a fresh clone in the WSL ext4 home directory, with its own in-tree target directory, removed afterwards. The host was busier than in earlier runs, so the durations are longer; no count changed. The scratchpad path is shortened to `<scratchpad>`. The scripts strip `cargo test -q` progress dots before filtering, so every `matrix` line is kept (test critique round 1, C-004). [unit-tests.md](unit-tests.md), [integration-tests.md](integration-tests.md) and [e2e-tests.md](e2e-tests.md) cite its sections by name.

## Windows (`s1_windows.sh`)

```text

==== host (Windows)
Microsoft Windows 11 Pro 10.0.26300
rustc 1.99.1 (58cf8f9c3 2026-10-13)
binary: rustc
commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67
commit-date: 2026-10-13
cargo-deny 0.20.2
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, RUSTFLAGS, RUSTC*, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_* set: none
head 9f138f4df0f2dff4451ab22b1c81511de5d32811 on dev; uncommitted paths: 0

==== int_ci_windows
PASS fmt
PASS clippy
PASS test
PASS policy
PASS book
ci: all 5 steps passed
cargo ci exit 0 in 157 s
tests passed on Windows: 242

==== int_crash_matrix_windows
matrix correct          Lenient: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix digest over 18688 cells: blake3:4e4d9e8bbdfa2396fa5b39085f62854fb3640280205ed2f4b9d6396648d99c59
matrix pointer-file     Lenient: 10576 cells (80 two-writer races); violations: closure 0 durability 0 heads 5966 atomicity 0 torn 0 layout 0 stray 9088
matrix pointer-file     Strict: 10576 cells (80 two-writer races); violations: closure 0 durability 0 heads 5985 atomicity 0 torn 0 layout 0 stray 8932
matrix unsynced-init    Lenient: 8544 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-init    Strict: 8544 cells (70 two-writer races); violations: closure 1425 durability 20979 heads 0 atomicity 0 torn 0 layout 6636 stray 0
matrix unsynced-objects Lenient: 8547 cells (65 two-writer races); violations: closure 6633 durability 9402 heads 0 atomicity 0 torn 9527 layout 0 stray 0
matrix unsynced-objects Strict: 8547 cells (65 two-writer races); violations: closure 6643 durability 9412 heads 0 atomicity 0 torn 9535 layout 0 stray 0
matrix write-in-place   Lenient: 8600 cells (56 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 563 layout 0 stray 0
matrix write-in-place   Strict: 8600 cells (121 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 563 layout 0 stray 0
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 188.50s

==== int_fsck_corpus_windows
bit flips detected: 40 of 40
truncations detected: 243 of 243
fsck on 16256 recovered stores: 0 reports
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.56s

==== e2e praps_fsck_end_to_end (Windows): the praps binary on real NTFS stores
test fsck_without_dir_checks_the_current_directory ... ok
test fsck_exits_zero_on_a_healthy_store ... ok
test fsck_lists_corrupt_files_and_exits_one ... ok
test fsck_on_a_non_store_exits_one ... ok
test usage_errors_exit_two ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.13s
test directory_flush_needs_write_access ... ok
test write_sequence_succeeds_on_the_real_filesystem ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test real_store_reopens_clean ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

==== int_deny_windows
advisories ok, bans ok, licenses ok, sources ok
cargo deny exit 0

==== probe: CARGO_TARGET_DIR outside the checkout
cargo ci exit 1 (1 required)
xtask: \\?\<scratchpad>\outside-target\debug\xtask.exe is not inside C:\Users\charl\Animus_Periapsis\target; a target directory outside the checkout (CARGO_TARGET_DIR or build.target-dir) is refused
steps run: 0

==== probe: real filesystem calls in store/src fail Clippy
error: use of a disallowed method `std::fs::read`
error: use of a disallowed method `std::path::Path::exists`
clippy exit 101 (non-zero required)
restored: 0 changed paths

==== done
```

## Linux, WSL (`s1_wsl.sh`)

```text

==== host (WSL)
Linux 6.6.114.1-microsoft-standard-WSL2 x86_64
PRETTY_NAME="Ubuntu 26.04.1 LTS"
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, RUSTFLAGS, RUSTC*, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_* set: none

==== e2e fresh_clone_ci
head 9f138f4df0f2dff4451ab22b1c81511de5d32811; checkout on ext4
rustc 1.99.1 (58cf8f9c3 2026-10-13)
binary: rustc
commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67
commit-date: 2026-10-13
PASS fmt
PASS clippy
PASS test
PASS policy
PASS book
ci: all 5 steps passed
cold cargo ci exit 0 in 358 s
tests passed on Linux: 242
tracked files changed by the run: 0

==== int_crash_matrix_linux
matrix correct          Lenient: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 9344 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix digest over 18688 cells: blake3:4e4d9e8bbdfa2396fa5b39085f62854fb3640280205ed2f4b9d6396648d99c59
matrix pointer-file     Lenient: 10576 cells (80 two-writer races); violations: closure 0 durability 0 heads 5966 atomicity 0 torn 0 layout 0 stray 9088
matrix pointer-file     Strict: 10576 cells (80 two-writer races); violations: closure 0 durability 0 heads 5985 atomicity 0 torn 0 layout 0 stray 8932
matrix unsynced-init    Lenient: 8544 cells (70 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-init    Strict: 8544 cells (70 two-writer races); violations: closure 1425 durability 20979 heads 0 atomicity 0 torn 0 layout 6636 stray 0
matrix unsynced-objects Lenient: 8547 cells (65 two-writer races); violations: closure 6633 durability 9402 heads 0 atomicity 0 torn 9527 layout 0 stray 0
matrix unsynced-objects Strict: 8547 cells (65 two-writer races); violations: closure 6643 durability 9412 heads 0 atomicity 0 torn 9535 layout 0 stray 0
matrix write-in-place   Lenient: 8600 cells (56 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 563 layout 0 stray 0
matrix write-in-place   Strict: 8600 cells (121 two-writer races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 563 layout 0 stray 0
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 290.62s

==== int_fsck_corpus_linux
bit flips detected: 40 of 40
truncations detected: 243 of 243
fsck on 16256 recovered stores: 0 reports
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 54.78s

==== e2e praps_fsck_end_to_end (Linux): the praps binary on real ext4 stores
test fsck_without_dir_checks_the_current_directory ... ok
test fsck_exits_zero_on_a_healthy_store ... ok
test fsck_on_a_non_store_exits_one ... ok
test fsck_lists_corrupt_files_and_exits_one ... ok
test usage_errors_exit_two ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test directory_flush_needs_write_access ... ok
test write_sequence_succeeds_on_the_real_filesystem ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test real_store_reopens_clean ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==== probe: CARGO_TARGET_DIR outside the checkout
cargo ci exit 1 (1 required)
xtask: /home/charles/periapsis-outside-target/debug/xtask is not inside /home/charles/periapsis-s1-test/target; a target directory outside the checkout (CARGO_TARGET_DIR or build.target-dir) is refused
steps run: 0

==== done
```
