# Sprint 5 Test Matrix Log

The matrix result lines of the four matrix files, run optimised with `--nocapture` at `3289008`. That was the first tested head; the code review changed only `crates/xtask/src/{ci,units,workflow}.rs`, which these files do not use. Windows 11 (NTFS) ran in the working checkout, and WSL Ubuntu (ext4) in a fresh clone. Kept: every line that reports a matrix result (cells, races, violation counts, controls, kills, `fsck`, digests). Removed: durations, libtest's progress dots, `test ... ok` lines and "has been running for over 60 seconds" notices. [integration-tests.md](integration-tests.md) reports the comparison with Sprint 4's [test-matrix-log.md](../../s4/sprint-tests/test-matrix-log.md), filtered the same way.

## Windows

Comparison with Sprint 4, as multisets: Sprint 4 69 lines, Sprint 5 69 lines, missing 0, extra 0.

```text
matrix unsynced-objects Lenient: 14620 cells (260 view races, 40 parent races); violations: closure 25040 durability 55332 heads 0 atomicity 0 torn 54183 layout 0 stray 0
matrix unsynced-objects Strict: 14620 cells (260 view races, 40 parent races); violations: closure 24988 durability 55330 heads 0 atomicity 0 torn 54193 layout 0 stray 0
matrix unsynced-refs    Lenient: 14740 cells (200 view races, 40 parent races); violations: closure 19 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-refs    Strict: 14740 cells (200 view races, 40 parent races); violations: closure 25 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix write-in-place   Lenient: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
matrix write-in-place   Strict: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
matrix unsynced-init    Lenient: 15620 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-init    Strict: 15620 cells (280 view races, 40 parent races); violations: closure 3065 durability 88332 heads 0 atomicity 0 torn 0 layout 12826 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix pointer-file     Lenient: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10616 atomicity 0 torn 0 layout 0 stray 16212
matrix pointer-file     Strict: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10718 atomicity 0 torn 0 layout 0 stray 16056
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix digest over 32840 cells: blake3:d36b8c9c77744d485a362ef3d94cebe40dcb67ca7260eb409f5430c2a13ae812
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
writers matrix correct: 44836 cells
writers correct              Lenient kill false: 11616 cells, 0 killed a process
writers correct              Lenient kill true: 10802 cells, 3276 killed a process
writers correct              Strict kill false: 11616 cells, 0 killed a process
writers correct              Strict kill true: 10802 cells, 3276 killed a process
writers correct              Lenient: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers correct              Strict: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers fsck problems: 0
writers matrix digest over 44836 cells: blake3:253b1eabc4054f7aebfb6edd2e907fdaccbc76549158410a47fedc610ba0855c
snapshot matrix unsynced-objects Lenient: 11640 cells; violations: closure 101135 durability 0 recency 9909 liveness 9508
snapshot matrix unsynced-objects Strict: 11640 cells; violations: closure 100560 durability 0 recency 9939 liveness 9379
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix digest over 25360 cells: blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3
materialise matrix NoRecord Lenient: 19840 cells; violations: closure 0 durability 0 working copy 2800 liveness 0
materialise matrix NoRecord Strict: 19840 cells; violations: closure 0 durability 0 working copy 2820 liveness 0
materialise matrix NoRecord: 0 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct (Windows rules) Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules) Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules): 7998 cells resumed a record; second pass 0 cells, 0 violations
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
```

## Linux (WSL)

Comparison with Sprint 4, as multisets: Sprint 4 69 lines, Sprint 5 69 lines, missing 0, extra 0.

```text
matrix unsynced-objects Lenient: 14620 cells (260 view races, 40 parent races); violations: closure 25040 durability 55332 heads 0 atomicity 0 torn 54183 layout 0 stray 0
matrix unsynced-objects Strict: 14620 cells (260 view races, 40 parent races); violations: closure 24988 durability 55330 heads 0 atomicity 0 torn 54193 layout 0 stray 0
matrix write-in-place   Lenient: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
matrix write-in-place   Strict: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
matrix unsynced-init    Lenient: 15620 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-init    Strict: 15620 cells (280 view races, 40 parent races); violations: closure 3065 durability 88332 heads 0 atomicity 0 torn 0 layout 12826 stray 0
matrix unsynced-refs    Lenient: 14740 cells (200 view races, 40 parent races); violations: closure 19 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-refs    Strict: 14740 cells (200 view races, 40 parent races); violations: closure 25 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix pointer-file     Lenient: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10616 atomicity 0 torn 0 layout 0 stray 16212
matrix pointer-file     Strict: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10718 atomicity 0 torn 0 layout 0 stray 16056
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix digest over 32840 cells: blake3:d36b8c9c77744d485a362ef3d94cebe40dcb67ca7260eb409f5430c2a13ae812
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
writers matrix correct: 44836 cells
writers correct              Lenient kill false: 11616 cells, 0 killed a process
writers correct              Lenient kill true: 10802 cells, 3276 killed a process
writers correct              Strict kill false: 11616 cells, 0 killed a process
writers correct              Strict kill true: 10802 cells, 3276 killed a process
writers correct              Lenient: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers correct              Strict: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers fsck problems: 0
writers matrix digest over 44836 cells: blake3:253b1eabc4054f7aebfb6edd2e907fdaccbc76549158410a47fedc610ba0855c
snapshot matrix unsynced-objects Lenient: 11640 cells; violations: closure 101135 durability 0 recency 9909 liveness 9508
snapshot matrix unsynced-objects Strict: 11640 cells; violations: closure 100560 durability 0 recency 9939 liveness 9379
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12680 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix digest over 25360 cells: blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3
materialise matrix NoRecord Lenient: 19840 cells; violations: closure 0 durability 0 working copy 2800 liveness 0
materialise matrix NoRecord Strict: 19840 cells; violations: closure 0 durability 0 working copy 2820 liveness 0
materialise matrix NoRecord: 0 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct (Windows rules) Lenient: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules) Strict: 20480 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules): 7998 cells resumed a record; second pass 0 cells, 0 violations
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
```
