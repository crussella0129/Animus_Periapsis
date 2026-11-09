# Sprint 5 Integration Test Results

**Tested head:** `87df514`, which carries the [code review](code-review.md) fixes. The suite ran first at `3289008`, the build's last commit, and every result held at both heads. Both rounds are recorded here.

Windows 11 (NTFS) ran in the working checkout. WSL Ubuntu (Linux 6.6.114.1-microsoft-standard-WSL2, ext4) ran in a fresh clone at each head, in its own in-tree target directory. rustc 1.99.1 on both. No two runs overlapped.

## The suite, optimised and parallel

| Run | `cargo ci` | Wall time at `87df514` | At `3289008` | Sprint 4 at `9fa59fb` | Tree after |
|---|---|---|---|---|---|
| Windows | all 6 steps passed | **269 s** | 291 s | 2,060 s | clean |
| WSL, run 1 (cold build) | all 6 steps passed | **253 s** | 241 s | 1,342 s | clean |
| WSL, run 2 | all 6 steps passed | **207 s** | 202 s | — | clean |

**How the gain splits on Windows:**
- **T-002 alone:** optimised, with the files still one after another: 667 s. That run was measured during the build, in a scratch worktree, with other work running alongside.
- **T-003 added:** 344 s at the T-003 state, and 291 s at the tested head.

**Slowest units at `3289008`** (seconds). At `87df514` they were close to these: `materialise_crash` 254, 207 and 206; `crash` 165, 131 and 133; `writers` 108, 145 and 149; Windows `lib and bins` 179.

| Unit | Windows | WSL run 1 | WSL run 2 |
|---|---|---|---|
| `materialise_crash` | 263.8 | 195.6 | 201.4 |
| `lib and bins` | 267.3 | 10.7 | — |
| `crash` (store and workspace) | 154.0 | 136.9 | 126.2 |
| `writers` | 100.9 | 136.5 | 140.2 |
| `cli` (book and praps) | 70.9 | 1.1 | — |

Every other unit took under 40 s on Windows and under 18 s in WSL.

- **Why `lib and bins` is long on Windows.** It holds `xtask`'s own tests. `hostile_variables_never_reach_a_step` runs a whole nested `cargo ci` on a small workspace, and the policy tests spawn Git and cargo. Process start-up is far slower on Windows than on Linux.
- **The shape of a run.** On both systems, `materialise_crash` sets the time: the run ends a few seconds after it does.

**Totals and counts.**
- `test:` totals: 500 on Windows and 501 in each WSL run at `87df514`, against 496 and 497 at `3289008`.
- These equal Sprint 4's GitHub counts (478 and 479) plus this sprint's 22 new tests, or 18 before the code review ([unit-tests.md](unit-tests.md)).
- At `87df514` every run printed `test: 45 test executables built, 45 run`. That is the oracle's own count, which the code review made visible (CR-04).
- No unit failed its unrun-executable, ran-nothing or rebuild check. So every test executable the pre-build reported was run, and no unit compiled anything.

**Lock waits.** Cargo printed "Blocking waiting for file lock" 53 times on Windows, and 150 and 144 times in WSL, at `3289008`. At `87df514` the counts were 78, 146 and 148.
- The waits are for the build directory or the package cache. Each unit's cargo takes the lock briefly to check freshness before it runs its executables.
- They cost the short units a few seconds each on Windows, where a 0.1 s test file reports 6 to 9 s, but nothing on the critical path. They also caused no rebuild.

**Timing guards.** No test failed or was retried in any run. The tests that printed "has been running for over 60 seconds" all passed:
- the matrices;
- `writers_retry_reaches_present`;
- several `policy` tests;
- `steps_execute_for_real`.

`sim/tests/processes.rs`'s 60-second watchdogs did not fire.

**Memory.** The peak working set of cargo, rustc and the test executables was **465 MB** at `3289008` and 374 MB at `87df514`, sampled every second during the Windows runs, out of 64 GB. A 16 GB GitHub runner has ample room. GitHub gives no memory probe; a runner running out of memory would show as a failed job, and none failed ([e2e-tests.md](e2e-tests.md)).

**cargo-deny.** `cargo deny check` passed on Windows, and in WSL at both heads, with `advisories ok, bans ok, licenses ok, sources ok`, using cargo-deny 0.20.2.

**The reach table.** `cargo reach`, run on Windows after the suite, rewrote `docs/reach.md` with no change. The `reach` step of every `cargo ci` run also passed against the committed table.

## The matrices printed in full

At `3289008`, on each system, the four matrix files were run optimised with `--nocapture`. The code review changed no test or library code these files use; at `87df514` their digest tests passed again in every `cargo ci`. The files:
- `store/tests/crash.rs`;
- `store/tests/writers.rs`;
- `workspace/tests/crash.rs`;
- `workspace/tests/materialise_crash.rs`.

Every line reporting a matrix result was compared, as a multiset, with the same lines in Sprint 4's [test-matrix-log.md](../../s4/sprint-tests/test-matrix-log.md). That covers cell counts, races, violation counts per invariant, controls, kills, `fsck` problems and the four digests. Only durations and libtest's progress dots were removed first.

| System | Sprint 4 lines | Sprint 5 lines | Missing | Extra |
|---|---|---|---|---|
| Windows | 69 | 69 | 0 | 0 |
| Linux (WSL) | 69 | 69 | 0 | 0 |

So optimised builds and parallel running changed no cell, count, violation or digest. That is INT-0005 AC-1 to AC-3 unchanged. Among the identical lines:
- the digests `d36b8c9c…`, `253b1eab…`, `c1e39cb1…` and `6274344b…`;
- the four-writer matrix's 44,836 cells;
- every negative control's violation counts.
