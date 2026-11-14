# Sprint 7 Test Matrix Log

The result lines of the seven simulation matrices at the tested head, `5b65da8`, run optimised with `--nocapture` on both systems:

```
cargo test --no-fail-fast -p store --test crash --test writers -- --nocapture
cargo test --no-fail-fast -p workspace --test crash --test materialise_crash --test describe_crash --test lane_crash --test concurrency -- --nocapture
```

They ran on Windows 11 (NTFS, the working checkout) and in WSL Ubuntu 26.04.1 (ext4, a fresh clone of `5b65da8`). They had also run at `1d5c94b`, and the lines were the same there but for the timing line below. Each system printed 94 result lines, and every test in the seven binaries passed. Sorted, the two outputs are equal line for line except four:
- **Three digests that cover lane files.** The lane matrix, and the Concurrency scenario's one-store and two-store runs, differ by system. A lane's `.praps/workspace` names the store's path as the system spells it, `repo\.praps\store` on Windows, and that file is part of every cell's state. Each test asserts one digest for Windows and one for other systems.
- **The four-writer matrix's own timing line:** 132 s on Windows, 369 s in WSL (59 s and 88 s at `1d5c94b`, while the machine was quieter).

Every cell count, every violation count and every control's result is the same on both systems.

## Cell counts and digests, before and after

A cell is one crash point. In the five matrices from Sprints 1 to 6, that is one for each syscall of the scenario and one after the last, for each of 40 seeds in each of two directory modes, so a count moves only if a scenario makes more or fewer syscalls.

| Matrix | Cells at `855eb67` | Cells at `5b65da8` | Digest at `855eb67` | Digest at `5b65da8` |
|---|---|---|---|---|
| store | 32,840 | 32,840 | `blake3:d36b8c9c…` | `blake3:08b4e33445e94b1b154c19293d47a66da8b440a6f748d5a429d52ea40c8755d8` |
| four writers | 44,836 | 44,836 | `blake3:253b1eab…` | `blake3:e5819bdf6d5f40714b3e6d8188016cc80ef08c5aadd312307e8b1646d194d28b` |
| Snapshot | 24,320 | 24,320 | `blake3:b6fca237…` | `blake3:dde53a7ba4954356d8bcee71c8d12a78376a3653f4ea2168aa9e67536091da3f` |
| Materialise | 34,400 | 34,400 | `blake3:5645fdf5…` | `blake3:1992301c25ffe9794573fe5c3eacebf0c8dd865333a6e0ae06fced55e6d0319a` |
| Describe | 28,720 | 28,720 | `blake3:686dfb8f…` | `blake3:919cd713956767f84cdad2c512bb34c2247f6c439793a59dd0dd2afcb986dd29` |
| Lane (new in T-006) | — | 5,760 | — | Windows `blake3:e67e05ecad31d03c26475a742d45eb9e9d40a1f9e2c06f278ffee3ff3b501724`; others `blake3:913e4ba049f5951fbddd5804a6965b2a22c26600c7d1fc4b4f300250a77bc103` |
| Concurrency, one store (new in T-009) | — | 160 | — | Windows `blake3:588a08e5c6fc02fb9f71d77961a93916b51be0cad0e1ee389872149aa8efe283`; others `blake3:a6e9c08352e0e4707468e26fa34b412382b9570c9327c3a1d401c911006e1f28` |
| Concurrency, two stores (new in T-009) | — | 160 | — | Windows `blake3:459ab6fd17fe97ed20f0c82520a2366badedfbb601ffec9cf36e5a78e0375473`; others `blake3:ab960f27cd5085c5456ff6d1575e4fbabde6bbfcc090f2cfb88b236c000be8f7` |

- **No count moved.** The five earlier matrices have the same cell counts as at `855eb67`, so no task added or removed a syscall from their scenarios.
- **T-002 moved all five digests.** An operation now names its workspace, and a view holds one entry per workspace, so the bytes of every operation and view change.
- **T-004 moved the Materialise digest again** (`blake3:3806881a…` after T-002), because the record names the changeset its write starts from.
- No other task moved any of the five. Each asserts its digest as a literal, and all five literals passed at every task's commit.
- A Concurrency cell is one run: for each seed and mode, the run with a kill, crashed once after its last syscall and once at a syscall drawn from the seed.

## Violations and controls

- **The correct variants find 0 violations in all seven matrices**, in both directory modes.
- **Every control is caught.** The store matrices' four broken stores, the Snapshot and Describe matrices' store that does not sync objects, the Materialise matrix's workspace that writes no record, and the Describe matrix's workspace that drops the description are caught as before.
- **The lane matrix's control,** `LaneFileInPlace`, is caught in 1,394 of its 5,440 cells: there the next command at the lane does not do what the cell allows. Making the lane again fails in 796 of them.
- **The Concurrency scenario's three controls** are each caught in the first cell, by the oracle each breaks:
  - `FirstHeadOnly`: two writers for every lane;
  - `MergeOnRead`: a command that changes nothing commits;
  - the store's `UnsyncedObjects`: the store does not check clean.
- **The Concurrency scenario's coverage:**
  - one store: 112 processes killed, 590 commands stopped by the crash, 66 operations in flight that landed;
  - two stores: 116, 594 and 182.

## The result lines

As printed on Windows and sorted. The four lines that differ in WSL are given above.

```
LaneFileInPlace: 1394 of 5440 cells caught
Variants { store: Correct, workspace: FirstHeadOnly }, one store: 1 cells, 1 with a violation; violations: Violations { failed: 5, readers: 1, lost: 0, store: 0, writer: 8, kept: 0, view: 8, next: 0 }; seen: Seen { landed: 0, crashed: 0, killed: 1 }
Variants { store: Correct, workspace: MergeOnRead }, one store: 1 cells, 1 with a violation; violations: Violations { failed: 0, readers: 7, lost: 0, store: 0, writer: 7, kept: 0, view: 0, next: 0 }; seen: Seen { landed: 1, crashed: 0, killed: 1 }
Variants { store: UnsyncedObjects, workspace: Correct }, one store: 1 cells, 1 with a violation; violations: Violations { failed: 0, readers: 0, lost: 0, store: 1, writer: 0, kept: 8, view: 9, next: 8 }; seen: Seen { landed: 0, crashed: 0, killed: 1 }
cells that crashed during the retry: 196
describe matrix correct          Correct Lenient: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Lenient: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Lenient: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Strict: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Strict: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          Correct Strict: 14360 cells; violations: closure 0 durability 0 recency 0 liveness 0
describe matrix correct          DropDescription Lenient: 14360 cells; violations: closure 0 durability 0 recency 1560 liveness 4664
describe matrix correct          DropDescription Strict: 14360 cells; violations: closure 0 durability 0 recency 1560 liveness 4660
describe matrix digest over 28720 cells: blake3:919cd713956767f84cdad2c512bb34c2247f6c439793a59dd0dd2afcb986dd29
describe matrix unsynced-objects Correct Lenient: 13160 cells; violations: closure 138289 durability 0 recency 11312 liveness 11312
describe matrix unsynced-objects Correct Strict: 13160 cells; violations: closure 138879 durability 0 recency 11408 liveness 11408
lane matrix Correct: 5760 cells; violations: Violations { outcome: 0, files: 0, again: 0, store: 0, durability: 0 }
lane matrix Correct: 5760 cells; violations: Violations { outcome: 0, files: 0, again: 0, store: 0, durability: 0 }
lane matrix Correct: 5760 cells; violations: Violations { outcome: 0, files: 0, again: 0, store: 0, durability: 0 }
lane matrix LaneFileInPlace: 5440 cells; violations: Violations { outcome: 1394, files: 0, again: 796, store: 0, durability: 0 }
lane matrix digest over 5760 cells: blake3:e67e05ecad31d03c26475a742d45eb9e9d40a1f9e2c06f278ffee3ff3b501724
materialise matrix Correct (Windows rules) Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules) Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct (Windows rules): 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Lenient: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct Strict: 17200 cells; violations: closure 0 durability 0 working copy 0 liveness 0
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct: 7998 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix Correct: 7998 cells resumed a record; second pass 102266 cells, 0 violations
materialise matrix NoRecord Lenient: 16560 cells; violations: closure 0 durability 0 working copy 2800 liveness 0
materialise matrix NoRecord Strict: 16560 cells; violations: closure 0 durability 0 working copy 2820 liveness 0
materialise matrix NoRecord: 0 cells resumed a record; second pass 0 cells, 0 violations
materialise matrix digest over 34400 cells: blake3:1992301c25ffe9794573fe5c3eacebf0c8dd865333a6e0ae06fced55e6d0319a
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Lenient: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix correct          Strict: 16420 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix digest over 32840 cells: blake3:08b4e33445e94b1b154c19293d47a66da8b440a6f748d5a429d52ea40c8755d8
matrix pointer-file     Lenient: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10616 atomicity 0 torn 0 layout 0 stray 16212
matrix pointer-file     Strict: 17700 cells (320 view races, 80 parent races); violations: closure 0 durability 0 heads 10718 atomicity 0 torn 0 layout 0 stray 16056
matrix unsynced-init    Lenient: 15620 cells (280 view races, 40 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-init    Strict: 15620 cells (280 view races, 40 parent races); violations: closure 3065 durability 88332 heads 0 atomicity 0 torn 0 layout 12826 stray 0
matrix unsynced-objects Lenient: 14620 cells (260 view races, 40 parent races); violations: closure 25341 durability 55346 heads 0 atomicity 0 torn 54183 layout 0 stray 0
matrix unsynced-objects Strict: 14620 cells (260 view races, 40 parent races); violations: closure 25307 durability 55356 heads 0 atomicity 0 torn 54193 layout 0 stray 0
matrix unsynced-refs    Lenient: 14740 cells (200 view races, 40 parent races); violations: closure 19 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix unsynced-refs    Strict: 14740 cells (200 view races, 40 parent races); violations: closure 25 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0
matrix write-in-place   Lenient: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
matrix write-in-place   Strict: 14320 cells (0 view races, 0 parent races); violations: closure 0 durability 0 heads 0 atomicity 0 torn 1134 layout 0 stray 0
one store: 160 cells, 0 with a violation; violations: Violations { failed: 0, readers: 0, lost: 0, store: 0, writer: 0, kept: 0, view: 0, next: 0 }; seen: Seen { landed: 66, crashed: 590, killed: 112 }
one store: digest over 160 cells: blake3:588a08e5c6fc02fb9f71d77961a93916b51be0cad0e1ee389872149aa8efe283
snapshot matrix correct          Lenient: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Lenient: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix correct          Strict: 12160 cells; violations: closure 0 durability 0 recency 0 liveness 0
snapshot matrix digest over 24320 cells: blake3:dde53a7ba4954356d8bcee71c8d12a78376a3653f4ea2168aa9e67536091da3f
snapshot matrix unsynced-objects Lenient: 11120 cells; violations: closure 96140 durability 0 recency 9407 liveness 9090
snapshot matrix unsynced-objects Strict: 11120 cells; violations: closure 96163 durability 0 recency 9514 liveness 9045
two stores: 160 cells, 0 with a violation; violations: Violations { failed: 0, readers: 0, lost: 0, store: 0, writer: 0, kept: 0, view: 0, next: 0 }; seen: Seen { landed: 182, crashed: 594, killed: 116 }
two stores: digest over 160 cells: blake3:459ab6fd17fe97ed20f0c82520a2366badedfbb601ffec9cf36e5a78e0375473
writers correct              Lenient kill false: 11616 cells, 0 killed a process
writers correct              Lenient kill true: 10802 cells, 3276 killed a process
writers correct              Lenient: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers correct              Strict kill false: 11616 cells, 0 killed a process
writers correct              Strict kill true: 10802 cells, 3276 killed a process
writers correct              Strict: closure 0 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers fsck problems: 0
writers matrix correct: 44836 cells in 132 s
writers matrix digest over 44836 cells: blake3:e5819bdf6d5f40714b3e6d8188016cc80ef08c5aadd312307e8b1646d194d28b
writers pointer-file         Strict kill false: 12370 cells, 0 killed a process
writers pointer-file         Strict kill true: 11516 cells, 3161 killed a process
writers pointer-file         Strict: closure 0 durability 0 heads 16302 atomicity 0 torn 0 layout 0 stray 23064; odd cells with process 0's op undurable at process 2's rename: 0
writers unsynced-init        Strict kill false: 10794 cells, 0 killed a process
writers unsynced-init        Strict kill true: 10038 cells, 2990 killed a process
writers unsynced-init        Strict: closure 5982 durability 207914 heads 0 atomicity 0 torn 0 layout 17753 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers unsynced-objects     Strict kill false: 10441 cells, 0 killed a process
writers unsynced-objects     Strict kill true: 9731 cells, 2828 killed a process
writers unsynced-objects     Strict: closure 52367 durability 141644 heads 0 atomicity 0 torn 144176 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
writers unsynced-refs        Strict kill false: 10246 cells, 0 killed a process
writers unsynced-refs        Strict kill true: 9766 cells, 2281 killed a process
writers unsynced-refs        Strict: closure 12 durability 0 heads 0 atomicity 0 torn 0 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 100
writers write-in-place       Strict kill false: 10513 cells, 0 killed a process
writers write-in-place       Strict kill true: 9922 cells, 1897 killed a process
writers write-in-place       Strict: closure 0 durability 0 heads 0 atomicity 0 torn 6752 layout 0 stray 0; odd cells with process 0's op undurable at process 2's rename: 0
```
