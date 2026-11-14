# Sprint 6's last runs, and this boundary's dependency update

This is Sprint 7's T-001 and part of backlog T-035. It records two things:
- the GitHub Actions runs that could only exist once Sprint 6's pull request had been pushed for the last time and merged;
- the dependency update the owner merged between the sprints.

Every value below was read on 2026-11-14 from three sources:
- `gh api repos/crussella0129/Animus_Periapsis/actions/runs/<run>`;
- `gh api repos/crussella0129/Animus_Periapsis/actions/runs/<run>/jobs`;
- each `cargo ci` job's own log, from `gh run view --job <job> --log`.

No job below was re-run, so every run has one attempt.

## Runs after Sprint 6's tested head

Sprint 6's pull request is [crussella0129/Animus_Periapsis#8](https://github.com/crussella0129/Animus_Periapsis/pull/8). Its tested head was `bbffce6`, recorded in [Sprint 6's end-to-end tests](../../s6/sprint-tests/e2e-tests.md). Seven commits followed it, `8726dd7` to `ca4bbd9`. Each is a record or a sign-off, and they change files under `docs/` only. They went up in two pushes, the first six commits and then the seventh, and each push started a run:

| Run | Event, head | Job | Conclusion | Started → completed (UTC) | Job time |
|---|---|---|---|---|---|
| [42872605202](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42872605202) | pull request, `8d3331c` (sprint-6: record checkpoint) | `cargo ci (windows-latest)` (130027475659) | success | 2026-11-10 18:11:19 → 18:27:56 | 16 min 37 s |
| 42872605202 | | `cargo ci (ubuntu-latest)` (130027475596) | success | 18:11:19 → 18:20:26 | 9 min 7 s |
| 42872605202 | | `lint (windows-latest)` (130027475195) | success | 18:11:18 → 18:11:56 | 38 s |
| 42872605202 | | `lint (ubuntu-latest)` (130027475708) | success | 18:11:19 → 18:11:38 | 19 s |
| 42872605202 | | `cargo deny check` (130027475749) | success | 18:11:18 → 18:11:32 | 14 s |
| [42927097585](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42927097585) | pull request, `ca4bbd9` (the owner's sign-offs) | `cargo ci (windows-latest)` (130216214385) | success | 2026-11-10 23:54:10 → 2026-11-11 00:09:55 | 15 min 45 s |
| 42927097585 | | `cargo ci (ubuntu-latest)` (130216214189) | success | 23:54:10 → 00:01:26 | 7 min 16 s |
| 42927097585 | | `lint (windows-latest)` (130216214356) | success | 23:54:10 → 23:54:59 | 49 s |
| 42927097585 | | `lint (ubuntu-latest)` (130216214220) | success | 23:54:10 → 23:54:26 | 16 s |
| 42927097585 | | `cargo deny check` (130216214231) | success | 23:54:10 → 23:54:23 | 13 s |

## The merge into `main`

The owner merged the pull request at 2026-11-11T03:27:58Z, as `aab90fd`.

| Run | Event, head | Job | Conclusion | Started → completed (UTC) | Job time |
|---|---|---|---|---|---|
| [42952942473](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42952942473) | push to `main`, `aab90fd` (merge of #8) | `cargo ci (windows-latest)` (130301269070) | success | 2026-11-11 03:28:04 → 03:40:10 | 12 min 6 s |
| 42952942473 | | `cargo ci (ubuntu-latest)` (130301269221) | success | 03:28:04 → 03:35:15 | 7 min 11 s |
| 42952942473 | | `lint (windows-latest)` (130301268710) | success | 03:28:04 → 03:28:44 | 40 s |
| 42952942473 | | `lint (ubuntu-latest)` (130301269020) | success | 03:28:04 → 03:28:25 | 21 s |
| 42952942473 | | `cargo deny check` (130301268998) | success | 03:28:03 → 03:28:17 | 14 s |

**The `main` run was neither cancelled nor replaced** (INT-0003 AC-2). It is attempt 1 and concluded `success`. The next push to `main` came three days later; see the last section.

## What each suite showed

Every `cargo ci` job of the three runs printed the same:

| Leg | Runner image, version | Build cache | `test:` line | Summary |
|---|---|---|---|---|
| Windows | `windows-2025-vs2026` 20261103.298.1 | restored | 37 units, 588 passed, 0 failed, 0 ignored | `ci: all 6 steps passed` |
| Linux | `ubuntu-26.04` 20261101.186.1 | restored | 37 units, 590 passed, 0 failed, 0 ignored | `ci: all 6 steps passed` |

- **Totals.** These are the totals Sprint 6 recorded at its tested head, 588 and 590: the commits after it changed no test.
- **Time** (single runs, not INT-0003 AC-11's three samples). The Windows suite took 16 min 37 s, 15 min 45 s and 12 min 6 s; every one is under the criterion's 20 minutes, and only the last is under its aim of 15.

## The dependency update

Dependabot opened [crussella0129/Animus_Periapsis#9](https://github.com/crussella0129/Animus_Periapsis/pull/9) against `dev` on 2026-11-13. It bumps `toml_edit` from 0.25.17 to 0.25.19 and `toml_datetime` from 1.1.2 to 1.1.3, in `Cargo.lock` only. Its update ran as two Dependabot jobs, runs 43347092748 (`cargo`) and 43347092737 (`github_actions`), both `success`.

| Check, at its head `421c6a2` | Conclusion | Started → completed (UTC) |
|---|---|---|
| `cargo ci (windows-latest)` | success | 2026-11-13 01:45:20 → 02:01:57 |
| `cargo ci (ubuntu-latest)` | success | 01:45:20 → 01:51:25 |
| `lint (windows-latest)` | success | 01:45:20 → 01:46:02 |
| `lint (ubuntu-latest)` | success | 01:45:20 → 01:45:42 |
| `cargo deny check` | success | 01:45:20 → 01:45:35 |

- **The owner's approval.** On 2026-11-14, before Sprint 7 began, the owner was asked whether to merge it. The question said that it changes `Cargo.lock` only and that all five checks had passed. The owner answered "Merge it". The sprint skill allows updater intake only between sprints.
- **The merge.** `gh pr merge 9 --merge` merged it at 2026-11-14T13:05:54Z as `855eb67`. `main` was then brought into `dev` as merge `2cedb42`, with no change to any file, and Sprint 7 began on that commit.
- **No credit changes.** `docs/prior-art.md` lists `toml_edit` at 0.25.17, the version `Cargo.toml` requires. The lock's patch release meets that requirement, and `cargo policy` passes.

## Since Sprint 7 began

A commit was pushed straight to `main` on 2026-11-14 at 14:49 UTC, after this sprint began. It is `334fc08`, "sprint-0: test records - show a renamed project under its current name", and changes two Sprint 0 record files only. Its run on `main`, [43606847894](https://github.com/crussella0129/Animus_Periapsis/actions/runs/43606847894), is attempt 1 and concluded `success` at 15:01:15 UTC. Its `cargo ci (windows-latest)` job ran from 14:49:18 to 15:01:15, and `cargo ci (ubuntu-latest)` from 14:49:19 to 14:56:12. This sprint does not touch those two files. Its pull request is opened against `main` as it stands, so `334fc08` is already in the base it is compared with.
