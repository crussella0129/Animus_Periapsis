# Sprint 6 Test Matrix Log

The result lines of the three working-copy matrices (Snapshot, Materialise, Describe), run optimised with `--nocapture` at `4152526`. The tested head, `bbffce6`, adds tests only and changes no source the matrices run; their three literal tests passed there on both systems.

```
cargo test -p workspace --test crash --test materialise_crash --test describe_crash -- --nocapture
```

Windows 11 (NTFS, the working checkout) and WSL Ubuntu (ext4, in the clone made for this sprint's runs and checked out at that commit) printed the same 39 lines: sorted, the two outputs are equal line for line. They are given once, below, in the order Windows printed them.

The two store matrices were not run with `--nocapture` this sprint. Their tests assert the cell counts and digests as literals, 32,840 cells with `blake3:d36b8c9c…` and 44,836 cells with `blake3:253b1eab…`, and both passed on both systems at every task's commit and at the head. No file under `crates/store/tests` changed but for one test added.

## Cell counts and digests, before and after

A cell is one crash point: one for each syscall of the scenario and one after the last, for each of 40 seeds in each of two directory modes. So fewer cells means the same scenario makes fewer syscalls.

| Matrix | At | Cells | Cells ÷ 80 (the average for one seed in one mode) | Digest |
|---|---|---|---|---|
| Snapshot | `b0675f6` (Sprint 5) | 25,360 | 317 | `blake3:c1e39cb10607ff208e0f2774eee8770659aad590dbc8c71abd76522b69013ea3` |
| Snapshot | after T-003, and at the head | 24,320 | 304 | `blake3:b6fca2372d26d8a0b40000df18c57b226eb976d4b9597d2deb2a95a629e84e1c` |
| Materialise | `b0675f6` (Sprint 5) | 40,960 | 512 | `blake3:6274344b358b15cb24d182a681c592ccad66a82564fd40c5cf45e4fcb666a1ac` |
| Materialise | after T-003 | 35,680 | 446 | `blake3:fa121e34c6e0a1b5f8d652ba7cb6f177955593a4dde7902efdc64d6d66cb599c` |
| Materialise | after T-005, and at the head | 34,400 | 430 | `blake3:5645fdf5829cc41aaf387ef8e05e2a00912417c5272f15548e062a48c8e5a7ab` |
| Describe | new in T-010, and at the head | 28,720 | 359 | `blake3:686dfb8f7cf9c8e10f15223549fa00b237669767c97e3f1c903962dd7f6c0f1a` |

- **T-003** took, on average, 13 crash points out of each Snapshot run and 66 out of each Materialise run: a command reads the op log once.
- **T-005** took 16 more, on average, out of each Materialise run: `undo` and `restore` scan the working copy once.
- **No count rose.** T-004, T-006 to T-009 and T-011 to T-013 moved no cell: the three tests that assert these literals passed unchanged at each of those commits.
- The fourth column is the total divided by 80: an average, not a count taken from one run. The test plan asks for one seed's syscall count; that was not printed. Each total divides exactly, which is what equal runs would give.

## Violations and controls

- **The real workspace on the real store: 0 violations** in every matrix, in both modes.
- **Every control was caught:**

| Matrix | Control | Violations (lenient; strict) |
|---|---|---|
| Snapshot | the store that does not sync objects | closure 96,622; 96,099. recency 9,435; 9,464. liveness 9,054; 8,927 |
| Materialise | the workspace that writes no record | working copy 2,800; 2,820 |
| Describe | the workspace whose snapshot drops the description | recency 1,560; 1,560. liveness 4,664; 4,660. closure and durability 0 |
| Describe | the store that does not sync objects | closure 138,445; 139,145. recency 11,318; 11,404. liveness 11,318; 11,404 |

- The Materialise matrix resumed a record in 7,998 cells, and its second-crash pass ran 103,786 cells with 0 violations.

## The lines

```text
snapshot matrix unsynced-objects Lenient: 11120 cells; violations: closure 96622 durability 0 recency 9435 liveness 9054
snapshot matrix unsynced-objects Strict: 11120 cells; violations: closure 96099 durability 0 recency 9464 liveness 8927
snapshot matrix correct          Lenient: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix digest over 24320 cells: blake3:b6fca2372d26d8a0b40000df18c57b226eb976d4b9597d2deb2a95a629e84e1c
describe matrix unsynced-objects Correct Lenient: 13160 cells; violations: closure 138445 durability 0 recency 11318 liveness 11318
describe matrix unsynced-objects Correct Strict: 13160 cells; violations: closure 139145 durability 0 recency 11404 liveness 11404
describe matrix correct          Correct Lenient: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Strict: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          DropDescription Lenient: 14360 cells; violations: closure 0 durability 0 recency 1560 liveness 4664
describe matrix correct          DropDescription Strict: 14360 cells; violations: closure 0 durability 0 recency 1560 liveness 4660
describe matrix correct          Correct Lenient: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Strict: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Lenient: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Strict: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix digest over 28720 cells: blake3:686dfb8f7cf9c8e10f15223549fa00b237669767c97e3f1c903962dd7f6c0f1a
materialise matrix NoRecord Lenient: 16560 cells; violations: closure 0 durability 0 working copy 2800 liveness 0
materialise matrix NoRecord Strict: 16560 cells; violations: closure 0 durability 0 working copy 2820 liveness 0
materialise matrix NoRecord: 0 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct (Windows rules) Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules) Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules): 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix digest over 34400 cells: blake3:5645fdf5829cc41aaf387ef8e05e2a00912417c5272f15548e062a48c8e5a7ab
materialise matrix Correct Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 103786 cells, 0 violations
```
