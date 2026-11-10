# Sprint 4 checkpoint: GitHub Actions record and sign-offs

This is backlog task T-020, done as Sprint 5's T-001. It records two things that could only exist once `dev` was pushed at the Sprint 4 checkpoint:
- the GitHub-matrix halves of these criteria:
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4;
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1;
  - [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) AC-5's CI run;
- the state of the owner's sign-offs.

Every value below was read on 2026-11-09 from three sources:
- `gh run view <run> --json …`;
- `gh api repos/crussella0129/Animus_Periapsis/actions/runs/<run>/attempts/<n>/jobs`;
- each job's own log, from `gh api repos/crussella0129/Animus_Periapsis/actions/jobs/<job>/logs`.

`gh run view --job <job> --log` is **not** a source here. For a job in a re-run run, it returns the latest attempt's log for that job name, so the failed deny attempts would show the passing re-runs' logs (Sprint 5 test critique, C-008).

## Runs

The checkpoint pull request is [crussella0129/Animus_Periapsis#6](https://github.com/crussella0129/Animus_Periapsis/pull/6).
- **Its two commits:** `bf40f24` (sprint-4: close) and then `ed14283` (sprint-4: record checkpoint), its last head.
- **The merge:** the owner merged it at 2026-11-09T19:16:25Z as `f0910e67288cd7d135a6adceb06add2947e50e51`.
- **The workflow then in force:** `push` on `dev` and `main`, plus `pull_request`, so each commit ran twice.

| Run | Event, head | Attempt | Job | Conclusion | Started → completed (UTC) |
|---|---|---|---|---|---|
| [42580616158](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42580616158) | push to `dev`, `bf40f24` | 1 | `cargo ci (ubuntu-latest)` (129038158737) | success | 07:38:33 → 08:07:02 |
| 42580616158 | push to `dev`, `bf40f24` | 1 | `cargo ci (windows-latest)` (129038159118) | success | 07:38:33 → 08:27:23 |
| 42580616158 | push to `dev`, `bf40f24` | 1 | `cargo deny check` (129038159059) | success | 07:38:33 → 07:39:05 |
| [42580625889](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42580625889) | pull request, `bf40f24` | 1 | `cargo ci (ubuntu-latest)` (129038191893) | success | 07:38:39 → 08:07:00 |
| 42580625889 | pull request, `bf40f24` | 1 | `cargo ci (windows-latest)` (129038191962) | success | 07:38:39 → 08:27:49 |
| 42580625889 | pull request, `bf40f24` | 1 | `cargo deny check` (129038191572) | **failure**, not re-run | 07:38:38 → 07:39:08 |
| [42580692694](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42580692694) | push to `dev`, `ed14283` | 1 | `cargo ci (windows-latest)` (129038413093) | success | 07:39:19 → 08:28:20 |
| 42580692694 | push to `dev`, `ed14283` | 1 | `cargo ci (ubuntu-latest)` (129038413397) | success | 07:39:20 → 08:17:40 |
| 42580692694 | push to `dev`, `ed14283` | 1 | `cargo deny check` (129038413586) | **failure** | 07:39:19 → 07:39:51 |
| 42580692694 | push to `dev`, `ed14283` | 2 | `cargo deny check` (129244467046) | success | 15:09:32 → 15:10:04 |
| [42580699028](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42580699028) | pull request, `ed14283` | 1 | `cargo ci (ubuntu-latest)` (129038433875) | success | 07:39:23 → 08:17:45 |
| 42580699028 | pull request, `ed14283` | 1 | `cargo ci (windows-latest)` (129038433882) | success | 07:39:23 → 08:05:08 |
| 42580699028 | pull request, `ed14283` | 1 | `cargo deny check` (129038433532) | **failure** | 07:39:23 → 07:39:46 |
| 42580699028 | pull request, `ed14283` | 2 | `cargo deny check` (129244396365) | success | 15:09:25 → 15:09:59 |
| [42681766482](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42681766482) | push to `main`, `f0910e6` (merge of #6) | 1 | `cargo ci (windows-latest)` (129377864817) | success | 19:16:33 → 20:05:20 |
| 42681766482 | push to `main`, `f0910e6` | 1 | `cargo ci (ubuntu-latest)` (129377865225) | success | 19:16:33 → 19:54:24 |
| 42681766482 | push to `main`, `f0910e6` | 1 | `cargo deny check` (129377865189) | success | 19:16:33 → 19:17:03 |

Final run conclusions:
- `42580616158`: success.
- `42580625889`: failure. Its deny job was never re-run, because `ed14283` superseded it.
- `42580692694`: success at attempt 2.
- `42580699028`: success at attempt 2.
- `42681766482`: success.

In attempt 2 of the two `ed14283` runs, the API lists the `cargo ci` jobs again under new IDs (129244471894, 129244473363, 129244399433, 129244401519). They carry their attempt-1 times and results, because only the failed job was re-run.

## What each job showed

| Job kind | Runner image | Toolchain | Tests (sum of `test result:` lines) | Summary |
|---|---|---|---|---|
| `cargo ci (windows-latest)`, all four runs | `windows-2025-vs2026` 20261103.629 | 1.99.1-x86_64-pc-windows-msvc | 478 passed, 0 failed | `ci: all 6 steps passed` |
| `cargo ci (ubuntu-latest)`, all four runs | `ubuntu-26.04` 20261028.624 | 1.99.1-x86_64-unknown-linux-gnu | 479 passed, 0 failed | `ci: all 6 steps passed` |
| `cargo deny check`, all seven jobs: four passing (129038159059, 129244467046, 129244396365, 129377865189) and three failed (129038191572, 129038413586, 129038433532) | `ubuntu-26.04` 20261028.624 | cargo-deny 0.20.2, through `EmbarkStudios/cargo-deny-action@v2` | — | passing: `advisories ok, bans ok, licenses ok, sources ok`; failed: `gzip: invalid magic` (6 lines in each failed job's log, 0 in the passing ones) |

- **Test counts.** They equal Sprint 4's local counts at the tested head `9fa59fb` (477 on Windows, 478 in WSL), plus the CR-07 regression test `a_failed_snapshot_reports_nothing_stale`, which was committed after the tested head in `6665913`. The extra Linux test needs a real ext4 symlink, as before.
- **The six steps:** `fmt`, `clippy`, `test`, `reach`, `policy` and `book`. `reach` was new in Sprint 4. In every leg it passed against the committed `docs/reach.md`, so the reach table generated on GitHub's Windows and Linux runners equals the one committed from the local machines.
- **INT-0007 AC-5's CI run.** Both legs of every run passed `writers_matrix_finds_no_violations` and `writers_matrix_is_reproducible`; the latter asserts the four-writer digest `blake3:253b1eab…0855c`. They also passed `real_writers`, four OS processes on the runner's NTFS and ext4.
- **The deny failures.** All three failed jobs (129038191572, 129038413586, 129038433532) failed the same way. The action's Docker build downloaded `cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz` from GitHub releases, and `tar` stopped with `gzip: invalid magic`. That release file exists: the re-runs and the `main` run downloaded it. So these were download failures, not dependency findings. Sprint 5's T-004 replaces the action with a checksum-verified install that retries.
- **Durations.** The `cargo ci` legs took 25 min 45 s to 49 min 1 s, which is what Sprint 5 exists to fix ([research](../sprint-research/research-report.md)).

## Owner sign-offs

None of the following has been given in the conversation:
- INT-0001 AC-1 (charter decomposition);
- INT-0002 AC-1 (typed-acceptance grammar);
- INT-0003 AC-5 (design credits);
- INT-0006 AC-6 (format v1's Sprint 0 sections);
- INT-0004 AC-5 (the Windows durability strategy).

Between the Sprint 4 checkpoint and this record, the owner's messages were:
- a request to reconsider CI time, which became this sprint's goal;
- "Yes, Sprint 5";
- choices on the parallel runner, the triggers, and opening this sprint's pull request during Test.

The owner also merged PR #6. That is recorded as acceptance of the checkpoint only, not as any of these sign-offs. They are carried forward again in a backlog task that replaces T-020.

## INT-0007

All five INT-0007 criteria now hold with local and CI evidence:
- AC-1 to AC-4 come from the crash matrix, the controls and `fsck`, run locally and in every leg above. AC-1 is `formal` with obligation `unstated`, so its proxy evidence counts as `empirical`, per INT-0002.
- AC-5 comes from Sprint 4's local evidence and this record's CI run.

No criterion is `asserted`, so none waits on a sign-off. Under INT-0002's rule, the chapter is therefore Certified, and it moves from `active` to `realized`.
