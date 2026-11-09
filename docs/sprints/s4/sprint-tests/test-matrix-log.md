# Sprint 4 Test Matrix Log

The output of the two leg scripts at the tested head `9fa59fb`, run on 2026-11-09. The head holds the [code review](code-review.md) fixes and the [round-1 test critique](critique-round1.md) fixes. The legs ran first at `5dfb418` (commit `bd8dc8e` recorded that run); this is the second round, and every matrix count and digest is the same as in the first. The two legs ran at the same time on one 24-thread host, so every duration here is longer than either leg alone. The Windows leg ran in the clean working checkout. The Linux leg ran in a fresh clone in the WSL ext4 home directory, with its own in-tree target directory, removed at the end. Lines of the form `test ... ok` are left out. [unit-tests.md](unit-tests.md), [integration-tests.md](integration-tests.md) and [e2e-tests.md](e2e-tests.md) cite its sections by name.

## Windows (`s4_windows.sh`)

```text
==== host (Windows)
Microsoft Windows 11 Pro 10.0.26300
rustc 1.99.1 (58cf8f9c3 2026-10-13)
binary: rustc
commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67
commit-date: 2026-10-13
cargo-deny 0.20.2
logical processors: 24
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, RUSTFLAGS, RUSTC*, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_* set: none
head 9fa59fb8d0f94170860cff5bbee685625c612233 on dev; uncommitted paths: 0

==== int_ci_windows
PASS fmt
PASS clippy
PASS test
PASS reach
PASS policy
PASS book
ci: all 6 steps passed
cargo ci exit 0 in 2060 s
reach files after cargo ci: four-writers.txt materialise.txt snapshot.txt store.txt two-writers.txt
tracked files changed by the run: 0
tests passed on Windows: 477

==== int_reach_windows
reach: wrote docs/reach.md
docs/reach.md changed by cargo reach: 0

==== int_writers_matrix_windows
writers unsynced-refs        Strict kill false: 10246 cells, 0 killed a process
writers unsynced-refs        Strict kill true: 9766 cells, 2281 killed a process
writers unsynced-refs        Strict: closure 12 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 100
writers unsynced-objects     Strict kill false: 10441 cells, 0 killed a process
writers unsynced-objects     Strict kill true: 9731 cells, 2828 killed a process
writers unsynced-objects     Strict: closure 52481 durability 141608 heads 0 atomicity 0 torn 144176 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers unsynced-init        Strict kill false: 10794 cells, 0 killed a process
writers unsynced-init        Strict kill true: 10038 cells, 2990 killed a process
writers unsynced-init        Strict: closure 5982 durability 207914 heads 0 atomicity 0 torn 0 layout 17753 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers write-in-place       Strict kill false: 10507 cells, 0 killed a process
writers write-in-place       Strict kill true: 9920 cells, 1897 killed a process
writers write-in-place       Strict: closure 0 durability 0 heads 0 atomicity 0 torn 6758 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers pointer-file         Strict kill false: 12370 cells, 0 killed a process
writers pointer-file         Strict kill true: 11516 cells, 3161 killed a process
writers pointer-file         Strict: closure 0 durability 0 heads 16302 atomicity 0 torn 0 layout 0 stray 23064; odd cells with process 0's op undurable at process 2's rename: 0
writers matrix correct: 44836 cells in 95 s
writers correct              Lenient kill false: 11616 cells, 0 killed a process
writers correct              Lenient kill true: 10802 cells, 3276 killed a process
writers correct              Strict kill false: 11616 cells, 0 killed a process
writers correct              Strict kill true: 10802 cells, 3276 killed a process
writers correct              Lenient: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers correct              Strict: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers fsck problems: 0
cells that crashed during the retry: 196
writers matrix digest over 44836 cells: blake3:253b1eabc4054f7aebfb6edd2e907fdaccbc76549158410a47fedc610ba0855c
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 131.43s
writers binary wall 133 s

==== e2e real_four_processes_lose_nothing (NTFS)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.40s

==== int_sim_points_and_processes_windows
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 538.20s

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
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 560.27s

==== int_materialise_matrix_windows
materialise matrix NoRecord Lenient: 19840 cells; violations: closure 0 durability 0 working copy 2800 liveness 0
materialise matrix NoRecord Strict: 19840 cells; violations: closure 0 durability 0 working copy 2820 liveness 0
materialise matrix NoRecord: 0 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct (Windows rules) Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules) Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules): 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix digest over 40960 cells: blake3:6274344b358b15cb24d182a681c592ccad66a82564fd40c5cf45e4fcb666a1ac
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 118306 cells, 0 violations
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 999.20s

==== e2e praps skipped-name reports and Sprint 3 CLI (NTFS)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.20s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.37s

==== int_deny_windows
advisories ok, bans ok, licenses ok, sources ok
cargo deny exit 0

==== probe: a point not registered fails cargo policy
FAIL points: crates/store/src/lib.rs: point `store.probe.unregistered` is not registered in crates/store/src/points.rs
cargo policy exit 1 (1 required)
restored: 0 changed paths

==== done
leg exit 0
```

## Linux, WSL (`s4_wsl.sh`)

```text
==== host (WSL)
Linux 6.6.114.1-microsoft-standard-WSL2 x86_64
PRETTY_NAME="Ubuntu 26.04.1 LTS"
logical processors: 24
cargo-deny 0.20.2
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, RUSTFLAGS, RUSTC*, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_* set: none

==== e2e fresh_clone_ci
head 9fa59fb8d0f94170860cff5bbee685625c612233; checkout on ext4
rustc 1.99.1 (58cf8f9c3 2026-10-13)
binary: rustc
commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67
commit-date: 2026-10-13
PASS fmt
PASS clippy
PASS test
PASS reach
PASS policy
PASS book
ci: all 6 steps passed
cold cargo ci exit 0 in 1342 s
reach files after cargo ci: four-writers.txt materialise.txt snapshot.txt store.txt two-writers.txt
tracked files changed by the run: 0
tests passed on Linux: 478

==== int_reach_linux
reach: wrote docs/reach.md
docs/reach.md changed by cargo reach: 0

==== int_writers_matrix_linux
writers unsynced-refs        Strict kill false: 10246 cells, 0 killed a process
writers unsynced-refs        Strict kill true: 9766 cells, 2281 killed a process
writers unsynced-refs        Strict: closure 12 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 100
writers unsynced-objects     Strict kill false: 10441 cells, 0 killed a process
writers unsynced-objects     Strict kill true: 9731 cells, 2828 killed a process
writers unsynced-objects     Strict: closure 52481 durability 141608 heads 0 atomicity 0 torn 144176 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers write-in-place       Strict kill false: 10507 cells, 0 killed a process
writers write-in-place       Strict kill true: 9920 cells, 1897 killed a process
writers write-in-place       Strict: closure 0 durability 0 heads 0 atomicity 0 torn 6758 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers unsynced-init        Strict kill false: 10794 cells, 0 killed a process
writers unsynced-init        Strict kill true: 10038 cells, 2990 killed a process
writers unsynced-init        Strict: closure 5982 durability 207914 heads 0 atomicity 0 torn 0 layout 17753 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers pointer-file         Strict kill false: 12370 cells, 0 killed a process
writers pointer-file         Strict kill true: 11516 cells, 3161 killed a process
writers pointer-file         Strict: closure 0 durability 0 heads 16302 atomicity 0 torn 0 layout 0 stray 23064; odd cells with process 0's op undurable at process 2's rename: 0
writers matrix correct: 44836 cells in 140 s
writers correct              Lenient kill false: 11616 cells, 0 killed a process
writers correct              Lenient kill true: 10802 cells, 3276 killed a process
writers correct              Strict kill false: 11616 cells, 0 killed a process
writers correct              Strict kill true: 10802 cells, 3276 killed a process
writers correct              Lenient: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers correct              Strict: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers fsck problems: 0
cells that crashed during the retry: 196
writers matrix digest over 44836 cells: blake3:253b1eabc4054f7aebfb6edd2e907fdaccbc76549158410a47fedc610ba0855c
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 194.95s
writers binary wall 197 s

==== e2e real_four_processes_lose_nothing (ext4)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

==== int_sim_points_and_processes_linux
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 346.63s

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
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 348.74s

==== int_materialise_matrix_linux
materialise matrix NoRecord Lenient: 19840 cells; violations: closure 0 durability 0 working copy 2800 liveness 0
materialise matrix NoRecord Strict: 19840 cells; violations: closure 0 durability 0 working copy 2820 liveness 0
materialise matrix NoRecord: 0 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct (Windows rules) Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules) Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules): 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix digest over 40960 cells: blake3:6274344b358b15cb24d182a681c592ccad66a82564fd40c5cf45e4fcb666a1ac
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 118306 cells, 0 violations
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 680.52s

==== e2e praps skipped-name reports and Sprint 3 CLI (ext4)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

==== int_deny_linux
advisories ok, bans ok, licenses ok, sources ok
cargo deny exit 0

==== probe: CARGO_TARGET_DIR outside the checkout
cargo reach exit 1 (1 required)
xtask: /home/charles/periapsis-outside-target/debug/xtask is not inside /home/charles/periapsis-s4-test/target; a target directory outside the checkout (CARGO_TARGET_DIR or build.target-dir) is refused

==== done
leg exit 0
```
