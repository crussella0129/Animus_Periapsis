# Sprint 7 end-to-end test results

At the tested head `b951d6d`, through the built `praps` binary on the real filesystem: Windows 11 (NTFS) and WSL Ubuntu 26.04.1 (ext4). Both systems ran these tests inside `cargo ci`. Each of the two tests of commands at once was then run three more times on each system with `--nocapture`, for the counts it prints.

## Through the binary

- **`lanes_at_once_end_to_end` (T-010; INT-0014 AC-3): passes on both systems.**
  - Setup: 8 lanes made with `praps lane`, one at a time. Then 8 threads, released together by a barrier, each run in their own lane, as real processes, `status` after an edit, `describe` with their own text, and `new`.
  - Every command exits 0.
  - `praps lanes` prints the same 9 lines in every lane and in the repository, apart from the `*` on each one's own line.
  - Each lane's change before the current one carries its own text.
  - `praps verify` prints `clean`.
  - **The commands overlapped** (test critique, C-006). The test prints, from the op log, how many ops it holds, how many op heads are left, and how many ops name several parents, each one made by a command that merged heads. A run with no overlap would leave one head and no such op. In three more runs on each system, the op log held 35 ops. Of them, 16 in each WSL run and 16, 15 and 15 on Windows were made over several heads, and 8, 3 and 8 op heads were left in WSL, 7 in each Windows run.
- **`a_lane_session_end_to_end` (T-010; INT-0037 AC-2 and AC-3; INT-0009): passes on both systems.** It uses only words `praps` printed:
  - `praps` lists `lane` and `lanes`, and `praps help lane`'s example, `praps lane ../agent-17`, is run as printed;
  - in lane A it runs `status`, `describe`, `new`, `log` and `ops`; in lane B, an edit and `status`;
  - `undo` in A leaves B's files and B's line of `praps lanes` as they were;
  - `restore` in A of the first op `ops` printed brings A's log back, and `verify` prints `clean`.

  Every command exits 0 and prints the line its step names.
- **`eight_new_at_once_all_succeed` (T-004): passes on both systems.** It is research's measurement, made a test. At `855eb67`, 8 `praps new` at once in one working copy left 3 succeeded and 5 failed, and every later command refused. Now all 8 exit 0, the next `status` exits 0, and `verify` prints `clean`.
  - Since the test critique (C-006), it prints the op heads the eight left, the ops over several heads, and how many changes the working copy's pointer holds in their merge. Whenever the pointer holds more than one, `status`'s note must name each of them.
  - In three more runs on each system, the eight left 4, 2 and 3 op heads in WSL, with 2, 4 and 3 ops over several heads, and 8, 7 and 6 on Windows, with 0, 0 and 1. Each time the pointer held one change for each head, and `status` named every one. With 8 heads and no op over several, every process read the op log before any of them committed; in WSL they overlapped less, and some merged the heads others had left.
- **`commands_act_the_same_from_below_in_a_lane` (T-007; INT-0037 AC-3; test critique, C-007): passes on both systems.** `status` with an edit and with none, `log`, `ops`, `describe`, `undo`, `restore`, `verify`, `lanes` and a refusal are each run at a lane's top, from two levels down in it, and from outside with a directory below the top given. Each run is on its own copy of one repository and lane. Exit code, stdout, stderr and every file left are equal byte for byte, once the copy's path is masked. `new` is not in the corpus: each run draws a new change id, so its output differs by design.
- **The earlier end-to-end tests** (`praps_end_to_end`, `praps_history_end_to_end`, `praps_first_session_end_to_end` and the rest) pass unchanged, but for `ops` lines that now name their workspace.

## GitHub Actions

`dev` was pushed and the sprint's pull request, [crussella0129/Animus_Periapsis#10](https://github.com/crussella0129/Animus_Periapsis/pull/10), was opened during Test, as the owner approved. Nothing was merged. Values were read with `gh api …/actions/runs/<run>/attempts/<n>`, `…/attempts/<n>/jobs`, and each job's own log from `gh api …/actions/jobs/<job>/logs`.

**The tested head's run: [43642111524](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43642111524), event `pull_request`, head `b951d6d`, and two re-runs of all its jobs (INT-0003 AC-11's three samples).** Every job of every attempt concluded `success`.

| Attempt | `cargo ci (windows-latest)` | `cargo ci (ubuntu-latest)` | `lint (windows-latest)` | `lint (ubuntu-latest)` | `cargo deny check` |
|---|---|---|---|---|---|
| 1 | 21:18:38 → 21:31:20, **12 min 42 s** | 21:18:38 → 21:28:43, 10 min 5 s | 44 s | 16 s | 17 s |
| 2 | 21:31:33 → 21:48:00, **16 min 27 s** | 21:31:35 → 21:37:05, 5 min 30 s | 39 s | 20 s | 15 s |
| 3 | 21:48:17 → 22:01:00, **12 min 43 s** | 21:48:16 → 21:56:55, 8 min 39 s | 37 s | 18 s | 16 s |

- **`three_samples_of_every_job` (INT-0003 AC-11): pass.** Every job of all three samples took under 20 minutes. The Windows suite took 12 min 42 s, 16 min 27 s and 12 min 43 s. The first and third are under the aim of 15 minutes, and the second is over it by 1 min 27 s.
- **The wait for a runner,** which AC-11 asks to be reported apart from the job's time: every job of every attempt started 2 to 4 s after it was queued.
- **The scheduler tests on each runner** (test critique, C-011), from each job's own log:

  | Attempt | Windows: `processes`, `thousand`, `concurrency` | Linux: the same |
  |---|---|---|
  | 1 | 27.5 s, 16.3 s, 146.6 s | 0.9 s, 2.8 s, 96.3 s |
  | 2 | 50.2 s, 15.8 s, 240.2 s | 0.4 s, 2.5 s, 52.7 s |
  | 3 | 20.8 s, 18.4 s, 103.2 s | 0.5 s, 2.3 s, 48.3 s |

  The slowest run of the `processes` binary took 50.2 s, under a fifth of the 300 s that each of its guarded tests may take, and the job's limit is 30 minutes.
- **`sprint_pull_request_checks_pass`: pass.** All five checks succeeded in every attempt. Windows printed `test: 43 units, 663 passed, 0 failed, 0 ignored` and Linux `test: 43 units, 666 passed, 0 failed, 0 ignored` each time, both ending `ci: all 6 steps passed`, and both equal the local totals at the tested head.
- **Runners.** Windows `windows-2025-vs2026` 20261103.298.1 in every attempt; Linux `ubuntu-26.04` 20261111.196.1 in every attempt. Every `cargo ci` job restored its build cache.

**Earlier runs of this pull request**, each at a head that a later push replaced:
- **Run [43632738184](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43632738184), head `5b65da8`, attempts 1 to 3, every job `success` each time.** It was the tested head until the test critique's first round led to `b951d6d`. Windows passed 659 tests and Linux 661 in every attempt.

  | Attempt | `cargo ci (windows-latest)` | `cargo ci (ubuntu-latest)` | Windows `processes`, `thousand`, `concurrency` | Linux, the same | Linux runner version |
  |---|---|---|---|---|---|
  | 1 | 12 min 7 s | 5 min 18 s | 22.6 s, 12.4 s, 113.0 s | 0.5 s, 2.7 s, 46.8 s | 20261111.196.1 |
  | 2 | 12 min 24 s | 9 min 16 s | 49.8 s, 7.8 s, 66.5 s | 0.7 s, 2.6 s, 75.6 s | 20261111.196.1 |
  | 3 | 17 min 23 s | 6 min 27 s | 47.8 s, 22.9 s, 212.1 s | 0.8 s, 2.2 s, 49.6 s | 20261111.82.6 |

  Every job of every attempt waited 2 to 4 s for a runner. Windows ran `windows-2025-vs2026` 20261103.298.1 in each attempt.
- **Run [43624973832](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43624973832), head `38a6b96` (T-010's record), conclusion `failure`.** The Linux suite failed `lane_matrix_is_reproducible` and `concurrency_is_reproducible` at their digest literals, with the digests WSL had printed. This was the lane file's path spelling, fixed in `1d5c94b` ([integration-tests.md](integration-tests.md)). The Windows suite passed in 12 min 26 s, with 656 tests. The other three jobs succeeded.
- **Run [43628379803](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43628379803), head `1d5c94b`, conclusion `failure`.** The Windows suite failed `a_kill_comes_before_an_ordering_wait`, whose 60-second deadlock guard ran out while the 1,000-process test shared its binary. This was handled in `5b65da8`; see [integration-tests.md](integration-tests.md). The Linux suite passed with 661 tests. The other three jobs succeeded.

**A correction to the first version of this record.** It read each attempt's log with `gh run view --job <job> --log`, which returns the log of the run's latest attempt whatever job is named. So it gave attempt 3's Linux runner version, 20261111.82.6, for all three attempts at `5b65da8`; attempts 1 and 2 ran 20261111.196.1. Every value above is from each job's own log. The record of Sprint 6's runs, [ci-sprint6.md](ci-sprint6.md), is not affected, since each of those runs had one attempt.

The authoritative conclusion is **success**: run 43642111524, attempts 1 to 3, the last finished at 2026-11-14T22:01:00Z. Commits after `b951d6d` change records only.

## Not yet possible

| What | Why | What unlocks it |
|---|---|---|
| INT-0010 AC-8 (1,000 writers on 100 replicas) and AC-9 (merge cost), and INT-0035 AC-1 to AC-5 | They need packed storage, derived merged views and the simulator at scale | Backlog T-030, T-032, T-039 and T-040 |
| Ownership against forged operations (INT-0016 AC-7; INT-0035 AC-6) | An operation names its workspace but is not signed | Signed operations, in INT-0016's sprint; backlog T-037 |
| Real replicas over a network (INT-0010 AC-5 in full; INT-0035 AC-9) | The Concurrency scenario's two stores share one simulated filesystem and exchange twice | [INT-0024](../../../intents/INT-0024-staged-sync.md) |
| INT-0037 AC-6: agents that learn `praps` from its help alone | It needs fresh agents and a task set kept out of every prompt | Backlog T-044, under [INT-0037](../../../intents/INT-0037-command-set.md) |
| Nighthawk (INT-0004 AC-6) | The owner must approve Tailscale access | The owner |
