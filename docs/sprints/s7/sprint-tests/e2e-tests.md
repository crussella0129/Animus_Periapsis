# Sprint 7 end-to-end test results

At the tested head `5b65da8`, through the built `praps` binary on the real filesystem: Windows 11 (NTFS) and WSL Ubuntu 26.04.1 (ext4). Both ran these tests inside `cargo ci`.

## Through the binary

- **`lanes_at_once_end_to_end` (T-010; INT-0014 AC-3): passes on both systems.**
  - Setup: 8 lanes made with `praps lane`, one at a time. Then 8 threads, released together by a barrier, each run in their own lane, as real processes, `status` after an edit, `describe` with their own text, and `new`.
  - Every command exits 0.
  - `praps lanes` prints the same 9 lines in every lane and in the repository, apart from the `*` on each one's own line.
  - Each lane's change before the current one carries its own text.
  - `praps verify` prints `clean`.
  - It passed 11 times in a row on Windows before T-010 was committed, and in every `cargo ci` since, on both systems and on GitHub. The interleaving is the system's; what the test asserts holds under any.
- **`a_lane_session_end_to_end` (T-010; INT-0037 AC-2 and AC-3; INT-0009): passes on both systems.** It uses only words `praps` printed:
  - `praps` lists `lane` and `lanes`, and `praps help lane`'s example, `praps lane ../agent-17`, is run as printed;
  - in lane A it runs `status`, `describe`, `new`, `log` and `ops`; in lane B, an edit and `status`;
  - `undo` in A leaves B's files and B's line of `praps lanes` as they were;
  - `restore` in A of the first op `ops` printed brings A's log back, and `verify` prints `clean`.

  Every command exits 0 and prints the line its step names.
- **`eight_new_at_once_all_succeed` (T-004): passes on both systems.** It is research's measurement, made a test. At `855eb67`, 8 `praps new` at once in one working copy left 3 succeeded and 5 failed, and every later command refused. Now all 8 exit 0, and the next `status` exits 0.
- **The earlier end-to-end tests** (`praps_end_to_end`, `praps_history_end_to_end`, `praps_first_session_end_to_end` and the rest) pass unchanged, but for `ops` lines that now name their workspace.

## GitHub Actions

`dev` was pushed and the sprint's pull request, [crussella0129/Animus_Periapsis#10](https://github.com/crussella0129/Animus_Periapsis/pull/10), was opened during Test, as the owner approved. Nothing was merged. Values were read with `gh api …/actions/runs/<run>`, `…/jobs` and each job's log.

**The tested head's run: [43632738184](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43632738184), event `pull_request`, head `5b65da8`, and two re-runs of all its jobs (INT-0003 AC-11's three samples).** Every job of every attempt concluded `success`.

| Attempt | `cargo ci (windows-latest)` | `cargo ci (ubuntu-latest)` | `lint (windows-latest)` | `lint (ubuntu-latest)` | `cargo deny check` |
|---|---|---|---|---|---|
| 1 | 19:33:30 → 19:45:37, **12 min 7 s** | 19:33:30 → 19:38:48, 5 min 18 s | 43 s | 23 s | 14 s |
| 2 | 19:49:27 → 20:01:51, **12 min 24 s** | 19:49:27 → 19:58:43, 9 min 16 s | 43 s | 16 s | 16 s |
| 3 | 20:02:27 → 20:19:50, **17 min 23 s** | 20:02:27 → 20:08:54, 6 min 27 s | 44 s | 19 s | 17 s |

- **INT-0003 AC-11: pass.** Every job of all three samples took under 20 minutes. The Windows suite was under the aim of 15 minutes in two samples and over it in the third, by 2 min 23 s.
- **Totals in every attempt:** Windows `test: 43 units, 659 passed, 0 failed, 0 ignored`; Linux `test: 43 units, 661 passed, 0 failed, 0 ignored`. Both end `ci: all 6 steps passed`, and both equal the local totals at the tested head.
- **Runners.** Windows `windows-2025-vs2026` 20261103.298.1 in every attempt. Linux `ubuntu-26.04` 20261111.82.6, which is newer than Sprint 6's 20261101.186.1. Every `cargo ci` job restored its build cache.

**Earlier runs of this pull request**, each at a head that a later push replaced:
- **Run [43624973832](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43624973832), head `38a6b96` (T-010's record), conclusion `failure`.** The Linux suite failed `lane_matrix_is_reproducible` and `concurrency_is_reproducible` at their digest literals, with the digests WSL had printed. This was the lane file's path spelling, fixed in `1d5c94b` ([integration-tests.md](integration-tests.md)). The Windows suite passed in 12 min 26 s, with 656 tests. The other three jobs succeeded.
- **Run [43628379803](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43628379803), head `1d5c94b`, conclusion `failure`.** The Windows suite failed `a_kill_comes_before_an_ordering_wait`, whose 60-second deadlock guard ran out while the 1,000-process test shared its binary. This was handled in `5b65da8`; see [integration-tests.md](integration-tests.md). The Linux suite passed with 661 tests. The other three jobs succeeded.

The authoritative conclusion is **success**: run 43632738184, attempts 1 to 3, the last finished at 2026-11-14T20:19:50Z. Commits after `5b65da8` change records only.

## Not yet possible

| What | Why | What unlocks it |
|---|---|---|
| INT-0010 AC-8 (1,000 writers on 100 replicas) and AC-9 (merge cost), and INT-0035 AC-1 to AC-5 | They need packed storage, derived merged views and the simulator at scale | Backlog T-030, T-032, T-039 and T-040 |
| Ownership against forged operations (INT-0016 AC-7; INT-0035 AC-6) | An operation names its workspace but is not signed | Signed operations, in INT-0016's sprint; backlog T-037 |
| Real replicas over a network (INT-0010 AC-5 in full; INT-0035 AC-9) | The Concurrency scenario's two stores share one simulated filesystem | [INT-0024](../../../intents/INT-0024-staged-sync.md) |
| INT-0037 AC-6: agents that learn `praps` from its help alone | It needs fresh agents and a task set kept out of every prompt | A later sprint under [INT-0037](../../../intents/INT-0037-command-set.md) |
| Nighthawk (INT-0004 AC-6) | The owner must approve Tailscale access | The owner |
