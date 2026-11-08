# Sprint 3 Test Matrix Log

The output of the two matrix scripts at the tested head `70a2a4d`, run on 2026-11-07, after the [code review](code-review.md) fixes. The two legs ran at the same time, so the durations are long. The Windows leg ran in the clean working checkout. The Linux leg ran in a fresh clone in the WSL ext4 home directory, with its own in-tree target directory, removed afterwards. The scratchpad path is shortened to `<scratchpad>`, and lines of the form `test … ok` are left out. [unit-tests.md](unit-tests.md), [integration-tests.md](integration-tests.md) and [e2e-tests.md](e2e-tests.md) cite its sections by name.

In the Windows log, the section `int_snapshot_matrix_windows` was overwritten with NUL bytes by a leg stopped earlier, which still held the file open. That test was run again on its own at the same head; its output replaces the section below. The same overwrite cut the start of the first line of `int_materialise_matrix_windows` (the `NoRecord` Lenient line); what is left of it matches the Linux line.

## Windows (`s3_windows.sh`)

```text
==== host (Windows)
Microsoft Windows 11 Pro 10.0.26300
rustc 1.99.1 (58cf8f9c3 2026-10-13)
binary: rustc
commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67
commit-date: 2026-10-13
cargo-deny 0.20.2
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, RUSTFLAGS, RUSTC*, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_* set: none
head 70a2a4d70065babca41483f78f5c68a258f38c06 on dev; uncommitted paths: 0

==== int_ci_windows
PASS fmt
PASS clippy
PASS test
PASS policy
PASS book
ci: all 5 steps passed
cargo ci exit 0 in 1790 s
tests passed on Windows: 403

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
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 521.51s

==== int_fsck_corpus_windows
bit flips detected: 40 of 40
truncations detected: 207 of 207
fsck on 30408 recovered stores: 0 reports
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 79.54s

==== int_snapshot_matrix_windows (rerun at 70a2a4d; the original section was overwritten)
snapshot matrix unsynced-objects Lenient: 11640 cells; violations: closure 101135 durability 0 recency 9909 liveness 9508
snapshot matrix unsynced-objects Strict: 11640 cells; violations: closure 100560 durability 0 recency 9939 liveness 9379
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix digest over 25360 cells: blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 549.04s
exit 0

==== int_materialise_matrix_windows
ations: closure 0 durability 0 working copy 2800 liveness 0
materialise matrix NoRecord Strict: 19840 cells; violations: closure 0 durability 0 working copy 2820 liveness 0
materialise matrix NoRecord: 0 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix digest over 40960 cells: blake3:6274344b358b15cb24d182a681c592ccad66a82564fd40c5cf45e4fcb666a1ac
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 118306 cells, 0 violations
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2418.45s

==== int_reversibility_and_hazards_windows
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
200 pairs: 800 runs, 0 collapsed
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.76s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

==== int_edit_corpora_windows
edit corpus: 500 sequences, 6000 snapshots, 470 undos, 496 restores, 16 refusals (0 blocked), 6416 ops checked, 0 lost edits
long edit corpus: 200 sequences, 8000 snapshots, 662 undos, 656 restores, 31 refusals (10 blocked), 7389 ops checked, 0 lost edits
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 56.73s
ignore corpus: 47 paths in 18 cases agree with Git
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
real edit corpus: 20 sequences, 240 snapshots, 25 undos, 15 restores, 3 refusals (0 blocked), 246 ops checked, 0 lost edits
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.36s
working-copy changeset: blake3:9ad9ad907eb641bf35322a3db8ed790384b4b1c8bd6f5f4377fe54c2142228c3
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

==== e2e praps (Windows): init, status, new, log, undo, restore, fsck and a non-UTF-8 name on NTFS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.82s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

==== e2e praps_fsck_end_to_end (Windows): the praps binary on real NTFS stores
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.38s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

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

==== probe: RandomState in store/src fails Clippy
error: use of a disallowed type `std::hash::RandomState`
error: use of a disallowed type `std::hash::RandomState`
clippy exit 101 (non-zero required)
restored: 0 changed paths

==== probe: ignore's filesystem readers in workspace/src fail Clippy
error: use of a disallowed method `ignore::gitignore::GitignoreBuilder::add`
error: use of a disallowed type `ignore::WalkBuilder`
clippy exit 101 (non-zero required)
restored: 0 changed paths

==== done
```

## Linux, WSL (`s3_wsl.sh`)

```text
==== host (WSL)
Linux 6.6.114.1-microsoft-standard-WSL2 x86_64
PRETTY_NAME="Ubuntu 26.04.1 LTS"
PROPTEST_*, PERI_*, CLIPPY_CONF_DIR, RUSTFLAGS, RUSTC*, RUSTUP_TOOLCHAIN, CARGO_BUILD_*, CARGO_TARGET_*, CARGO_PROFILE_* set: none

==== e2e fresh_clone_ci
head 70a2a4d70065babca41483f78f5c68a258f38c06; checkout on ext4
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
cold cargo ci exit 0 in 993 s
tests passed on Linux: 404
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
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 324.55s

==== int_fsck_corpus_linux
bit flips detected: 40 of 40
truncations detected: 207 of 207
fsck on 30408 recovered stores: 0 reports
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.44s

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
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 322.75s

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
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix digest over 40960 cells: blake3:6274344b358b15cb24d182a681c592ccad66a82564fd40c5cf45e4fcb666a1ac
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 118306 cells, 0 violations
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1381.81s

==== int_reversibility_and_hazards_linux
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
200 pairs: 800 runs, 0 collapsed
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.72s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==== int_edit_corpora_linux
edit corpus: 500 sequences, 6000 snapshots, 470 undos, 496 restores, 16 refusals (0 blocked), 6416 ops checked, 0 lost edits
long edit corpus: 200 sequences, 8000 snapshots, 662 undos, 656 restores, 31 refusals (10 blocked), 7389 ops checked, 0 lost edits
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.48s
ignore corpus: 47 paths in 18 cases agree with Git
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
real edit corpus: 20 sequences, 240 snapshots, 26 undos, 12 restores, 3 refusals (0 blocked), 240 ops checked, 0 lost edits
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
working-copy changeset: blake3:9ad9ad907eb641bf35322a3db8ed790384b4b1c8bd6f5f4377fe54c2142228c3
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==== e2e praps (Linux): init, status, new, log, undo, restore, fsck and a non-UTF-8 name on ext4
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==== e2e praps_fsck_end_to_end (Linux): the praps binary on real ext4 stores
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==== probe: CARGO_TARGET_DIR outside the checkout
cargo ci exit 1 (1 required)
xtask: /home/charles/periapsis-outside-target/debug/xtask is not inside /home/charles/periapsis-s3-test/target; a target directory outside the checkout (CARGO_TARGET_DIR or build.target-dir) is refused
steps run: 0

==== done
```
