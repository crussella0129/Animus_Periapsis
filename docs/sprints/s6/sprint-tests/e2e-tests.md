# Sprint 6 end-to-end test results

Tested head: `bbffce6`. The tests in `crates/praps/tests/e2e.rs` run the built `praps` binary on the real filesystem: NTFS on Windows 11, and ext4 in WSL Ubuntu. All 10 passed on both, inside `cargo ci` and again by themselves.

## Through the binary

| Test | Task | What it shows | Windows | WSL |
|---|---|---|---|---|
| `praps_first_session_end_to_end` | T-010 | A session that uses only words `praps` printed. `praps` alone gives the list, and every word used after it is checked to be in that list. `praps help WORD` gives the usage line of each of nine commands. `praps help init` gives the example `praps init project`, which is run as printed. Then: two files and `status`; `describe`; `new`; two edits and `status`; a second `describe` of several lines; `log` (two changes, each with its first line); `ops` (six operations); `undo` (the second description is gone, no file changes); `restore` of the describing operation (it is back); `restore` of the first description's operation (the files of that time); `undo` (the later files return); `verify` (`clean`). Every command exits 0, and each output is compared with what the step names. At the end every operation is still listed, with the four the session added. | pass | pass |
| `praps_first_session_from_below` | T-010 | The same session with every command after `init` run from `src/app`, two levels below the top. The same assertions hold. | pass | pass |
| `praps_end_to_end_from_below` | T-008 | The history session from two levels down ends with the same files and the same number of operations as from the top. | pass | pass |
| `undo_from_a_directory_another_process_stands_in` | T-012 | See below. | pass | pass |
| `praps_commands_report_skipped_names` | T-009, T-010 | `ops`, `log`, `describe` (when it sets a description and when it has nothing to change), `undo`, `restore` and `new` each report a name with no UTF-8 form as `status` does, after their own output. | pass | pass |
| `praps_end_to_end`, `praps_history_end_to_end`, `praps_status_reports_a_non_utf8_name`, `praps_status_reports_non_utf8_names_and_directories` | earlier sprints | Unchanged but for `verify` and `ops` where they used `fsck` and `log`. | pass | pass |
| `stand_in_the_current_directory` | T-012 | Not a test of its own: it is the process the test below starts. Run as a test it does nothing. | pass | pass |

**Where these tests differ from the test plan.**
- The plan's outline of the first session has `undo` take back the `new`. The session as written undoes and restores a description and a write of files instead. It meets the plan's clause: only words `praps` printed, every command exits 0, and each output holds what its step names.
- The plan's `undo_from_a_directory_another_process_stands_in` has a file at `d1/d2/f`, with the helper process and `undo` in `d1/d2`, two levels down. The test has `work/file` and `work/deep/other`, with both in `work`, one level down, so that the write must remove a directory below the one stood in as well.

## `undo` from a directory another process stands in

A change adds `work/file` and `work/deep/other`. A second process is started with `work` as its current directory and waits. `praps undo` is run from `work`; its write removes both files and then the two directories they leave empty.

| | Windows (NTFS) | WSL (ext4) |
|---|---|---|
| Exit code of `undo` | 0 | 0 |
| The files | equal the target's: `keep` only | the same |
| `work` | **left**, empty | **removed** |
| stderr | `note: work: the directory could not be removed, and was left (The process cannot access the file because it is being used by another process. (os error 32))` | empty |
| `.praps/working-copy` | gone | gone |
| The next `status`, from the top | exit 0, nothing on stderr, `A keep` | the same |
| `verify` | `clean` | `clean` |
| A second `undo` | brings both files back | the same |

Before T-012, `praps` built at T-011's commit failed this `undo` on Windows with exit 1, and every later command failed the same way until the other process left ([unit-tests.md](unit-tests.md), Negative controls).

## GitHub Actions

`dev` was pushed and the sprint's pull request, [crussella0129/Animus_Periapsis#8](https://github.com/crussella0129/Animus_Periapsis/pull/8), was opened during Test, as the owner approved. Nothing was merged, and no job was re-run. Read on 2026-11-10 with `gh api …/actions/runs/<run>`, `…/jobs`, and `gh run view --job <job> --log`.

**`sprint_pull_request_checks_pass`: yes.** Run [42858372557](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42858372557), event `pull_request`, head `4152526`, attempt 1, concluded `success`.

| Job | Conclusion | Started → completed (UTC) | Job time |
|---|---|---|---|
| `cargo ci (windows-latest)` (129977556677) | success | 16:48:58 → 17:05:15 | 16 min 17 s |
| `cargo ci (ubuntu-latest)` (129977557370) | success | 16:48:58 → 16:55:45 | 6 min 47 s |
| `lint (windows-latest)` (129977557161) | success | 16:48:58 → 16:49:39 | 41 s |
| `lint (ubuntu-latest)` (129977557304) | success | 16:48:58 → 16:49:20 | 22 s |
| `cargo deny check` (129977557361) | success | 16:48:58 → 16:49:14 | 16 s |

- **Totals.** Windows: `test: 37 units, 587 passed, 0 failed, 0 ignored`. Linux: `test: 37 units, 589 passed, 0 failed, 0 ignored`. Both end `ci: all 6 steps passed`, and both equal the local totals. `cargo deny check` printed `advisories ok, bans ok, licenses ok, sources ok`.
- **Runners.** `windows-2025-vs2026` 20261103.298.1 and `ubuntu-26.04` 20261101.186.1, the images Sprint 5's runs used. Both `cargo ci` jobs restored their build cache.
- **Time** (one sample each, not INT-0003 AC-11's three). The Windows suite took 16 min 17 s: under the criterion's 20 minutes, over its aim of 15. Sprint 5's last pull request run took 16 min 19 s there, with 501 tests. The Linux suite took 6 min 47 s.
- **An earlier run.** Run 42856495316 started at the first push, head `80a3e7e`. The push of `4152526` replaced it, as the workflow intends for a pull request: its Windows suite was cancelled after ten minutes, and its other four jobs had already succeeded.

**`dispatch_runs_on_dev` (T-001): yes.** Run 42825642596, event `workflow_dispatch`, `dev` at `b0675f6`, five jobs, all `success`. Its record is [ci-sprint5.md](ci-sprint5.md).

## Not yet possible

| What | Why | What unlocks it |
|---|---|---|
| INT-0003 AC-11's three samples of one commit | They need two re-runs of every job, which the owner has not approved. | Backlog T-035 |
| The runs of the commits after `4152526`, and of the merge | They exist only after the sprint's last push and the owner's merge. | Backlog T-035 |
| INT-0037 AC-6: agents that learn `praps` from its help alone | It needs fresh agents and a task set kept out of every prompt. | A later sprint under [INT-0037](../../../intents/INT-0037-command-set.md) |
| INT-0037 AC-7, and INT-0022 AC-4 to AC-7 | A renderer that takes the terminal's width, and panels of people. | The sprint that designs the `log` layout ([INT-0022](../../../intents/INT-0022-agent-and-human-interfaces.md)) |
| Two agents on one repository | Two commands that overlap still leave two op heads. | The canonical merge, [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), backlog T-029 |
| Every criterion of [INT-0035](../../../intents/INT-0035-scale-and-asynchrony.md) and [INT-0036](../../../intents/INT-0036-measured-speed.md) | Neither suite exists yet. | Backlog T-029 to T-032 |
| Nighthawk (INT-0004 AC-6) | The owner must approve Tailscale access. | The owner |
