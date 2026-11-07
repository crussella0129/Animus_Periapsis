# Sprint 2 Test Matrix Log

The output of the two matrix scripts at the tested head `c5c6965`, run on 2026-11-07. This is the final run of the Test phase, after the [round-1 test critique](critique-round1.md); the run at `d248ccf` before it gave the same matrix, corpus and digest figures with 327 tests. The two legs ran at the same time, so the durations are long. The Windows leg ran in the clean working checkout; the Linux leg ran in a fresh clone in the WSL ext4 home directory, with its own in-tree target directory, removed afterwards. The scratchpad path is shortened to `<scratchpad>`. [unit-tests.md](unit-tests.md), [integration-tests.md](integration-tests.md) and [e2e-tests.md](e2e-tests.md) cite its sections by name.

## Windows (`s2_windows.sh`)

```text

==== host (Windows)
Microsoft Windows 11 Pro 10.0.26300
rustc 1.99.1 (58cf8f9c3 2026-10-13)
binary: rustc
commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67
commit-date: 2026-10-13
cargo-deny 0.20.2
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, RUSTFLAGS, RUSTC*, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_* set: none
head c5c6965249467a27ea8aecd21a30aec21b625458 on dev; uncommitted paths: 0

==== int_ci_windows
PASS fmt
PASS clippy
PASS test
PASS policy
PASS book
ci: all 5 steps passed
cargo ci exit 0 in 1270 s
tests passed on Windows: 332

==== int_crash_matrix_windows
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix digest over 32840 cells: blake3:d36b8c9c77744d485a362ef3d94cebe40dcb67ca7260eb409f5430c2a13ae812
matrix pointer-file     Lenient: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10616 atomicity 0 torn 0 layout 0 stray 16212
matrix pointer-file     Strict: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10718 atomicity 0 torn 0 layout 0 stray 16056
matrix unsynced-init    Lenient: 15620 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-init    Strict: 15620 cells (280 view races, 40 parent races); violations: closure 3065 durability 88332 heads 0 atomicity 0 torn 0 layout 12826 stray 0
matrix unsynced-objects Lenient: 14620 cells (260 view races, 40 parent races); violations: closure 25040 durability 55332 heads 0 atomicity 0 torn 54183 layout 0 stray 0
matrix unsynced-objects Strict: 14620 cells (260 view races, 40 parent races); violations: closure 24988 durability 55330 heads 0 atomicity 0 torn 54193 layout 0 stray 0
matrix unsynced-refs    Lenient: 14740 cells (200 view races, 40 parent races); violations: closure 19 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-refs    Strict: 14740 cells (200 view races, 40 parent races); violations: closure 25 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix write-in-place   Lenient: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
matrix write-in-place   Strict: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 366.27s

==== int_fsck_corpus_windows
bit flips detected: 40 of 40
truncations detected: 207 of 207
fsck on 30408 recovered stores: 0 reports
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 51.76s

==== int_snapshot_matrix_windows
snapshot matrix unsynced-objects Lenient: 11640 cells; violations: closure 101135 durability 0 recency 9909 liveness 9508
snapshot matrix unsynced-objects Strict: 11640 cells; violations: closure 100560 durability 0 recency 9939 liveness 9379
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix digest over 25360 cells: blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 388.94s

==== int_edit_corpora_windows
edit corpus: 500 sequences, 6000 snapshots, 0 lost edits
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.80s
ignore corpus: 47 paths in 18 cases agree with Git
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
real edit corpus: 20 sequences, 240 snapshots, 0 lost edits
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.70s
working-copy changeset: blake3:9ad9ad907eb641bf35322a3db8ed790384b4b1c8bd6f5f4377fe54c2142228c3
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

==== e2e praps (Windows): init, status, new, fsck and a non-UTF-8 name on NTFS
test praps_status_reports_non_utf8_names_and_directories ... ok
test praps_status_reports_a_non_utf8_name ... ok
test praps_end_to_end ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
test os_seed_varies ... ok
test real_scan_keeps_non_utf8_names ... ok
test real_scan_reports_the_executable_bit ... ok
test real_scan_meets_the_contract ... ok
test real_fs_keeps_names_win32_would_rewrite ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

==== e2e praps_fsck_end_to_end (Windows): the praps binary on real NTFS stores
test fsck_without_dir_checks_the_current_directory ... ok
test fsck_on_a_non_store_exits_one ... ok
test praps_fsck_reports_a_broken_repository_store ... ok
test praps_status_outside_a_repository_fails ... ok
test praps_init_is_idempotent ... ok
test fsck_exits_zero_on_a_healthy_store ... ok
test praps_status_with_several_heads_fails ... ok
test praps_fsck_checks_a_working_copy ... ok
test fsck_lists_corrupt_files_and_exits_one ... ok
test praps_init_creates_a_repository ... ok
test praps_rejects_flags_and_extra_arguments ... ok
test praps_new_prints_a_change_id ... ok
test usage_errors_exit_two ... ok
test praps_status_lists_changes ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test directory_flush_needs_write_access ... ok
test write_sequence_succeeds_on_the_real_filesystem ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test real_store_reopens_clean ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

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

==== probe: getrandom outside sim's real half fails Clippy
error: use of a disallowed method `getrandom::u64`
clippy exit 101 (non-zero required)
restored: 0 changed paths

==== probe: ignore's filesystem readers in workspace/src fail Clippy
error: use of a disallowed method `ignore::gitignore::GitignoreBuilder::add`
error: use of a disallowed type `ignore::WalkBuilder`
clippy exit 101 (non-zero required)
restored: 0 changed paths

==== done
```

## Linux, WSL (`s2_wsl.sh`)

```text

==== host (WSL)
Linux 6.6.114.1-microsoft-standard-WSL2 x86_64
PRETTY_NAME="Ubuntu 26.04.1 LTS"
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, RUSTFLAGS, RUSTC*, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_* set: none

==== e2e fresh_clone_ci
head c5c6965249467a27ea8aecd21a30aec21b625458; checkout on ext4
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
cold cargo ci exit 0 in 1181 s
tests passed on Linux: 332
tracked files changed by the run: 0

==== int_crash_matrix_linux
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix digest over 32840 cells: blake3:d36b8c9c77744d485a362ef3d94cebe40dcb67ca7260eb409f5430c2a13ae812
matrix pointer-file     Lenient: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10616 atomicity 0 torn 0 layout 0 stray 16212
matrix pointer-file     Strict: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10718 atomicity 0 torn 0 layout 0 stray 16056
matrix unsynced-init    Lenient: 15620 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-init    Strict: 15620 cells (280 view races, 40 parent races); violations: closure 3065 durability 88332 heads 0 atomicity 0 torn 0 layout 12826 stray 0
matrix unsynced-objects Lenient: 14620 cells (260 view races, 40 parent races); violations: closure 25040 durability 55332 heads 0 atomicity 0 torn 54183 layout 0 stray 0
matrix unsynced-objects Strict: 14620 cells (260 view races, 40 parent races); violations: closure 24988 durability 55330 heads 0 atomicity 0 torn 54193 layout 0 stray 0
matrix unsynced-refs    Lenient: 14740 cells (200 view races, 40 parent races); violations: closure 19 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-refs    Strict: 14740 cells (200 view races, 40 parent races); violations: closure 25 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix write-in-place   Lenient: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
matrix write-in-place   Strict: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 314.03s

==== int_fsck_corpus_linux
bit flips detected: 40 of 40
truncations detected: 207 of 207
fsck on 30408 recovered stores: 0 reports
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.56s

==== int_snapshot_matrix_linux
snapshot matrix unsynced-objects Lenient: 11640 cells; violations: closure 101135 durability 0 recency 9909 liveness 9508
snapshot matrix unsynced-objects Strict: 11640 cells; violations: closure 100560 durability 0 recency 9939 liveness 9379
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix digest over 25360 cells: blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 323.37s

==== int_edit_corpora_linux
edit corpus: 500 sequences, 6000 snapshots, 0 lost edits
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.21s
ignore corpus: 47 paths in 18 cases agree with Git
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
real edit corpus: 20 sequences, 240 snapshots, 0 lost edits
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
working-copy changeset: blake3:9ad9ad907eb641bf35322a3db8ed790384b4b1c8bd6f5f4377fe54c2142228c3
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==== e2e praps (Linux): init, status, new, fsck and a non-UTF-8 name on ext4
test praps_status_reports_non_utf8_names_and_directories ... ok
test praps_status_reports_a_non_utf8_name ... ok
test praps_end_to_end ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test os_seed_varies ... ok
test real_scan_reports_the_executable_bit ... ok
test real_scan_keeps_non_utf8_names ... ok
test real_fs_keeps_names_win32_would_rewrite ... ok
test real_scan_meets_the_contract ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==== e2e praps_fsck_end_to_end (Linux): the praps binary on real ext4 stores
test fsck_without_dir_checks_the_current_directory ... ok
test fsck_exits_zero_on_a_healthy_store ... ok
test praps_fsck_reports_a_broken_repository_store ... ok
test praps_init_creates_a_repository ... ok
test praps_init_is_idempotent ... ok
test fsck_on_a_non_store_exits_one ... ok
test fsck_lists_corrupt_files_and_exits_one ... ok
test praps_status_outside_a_repository_fails ... ok
test praps_status_with_several_heads_fails ... ok
test praps_fsck_checks_a_working_copy ... ok
test praps_new_prints_a_change_id ... ok
test praps_rejects_flags_and_extra_arguments ... ok
test usage_errors_exit_two ... ok
test praps_status_lists_changes ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test directory_flush_needs_write_access ... ok
test write_sequence_succeeds_on_the_real_filesystem ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test real_store_reopens_clean ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==== probe: CARGO_TARGET_DIR outside the checkout
cargo ci exit 1 (1 required)
xtask: /home/charles/periapsis-outside-target/debug/xtask is not inside /home/charles/periapsis-s2-test/target; a target directory outside the checkout (CARGO_TARGET_DIR or build.target-dir) is refused
steps run: 0

==== done
```
