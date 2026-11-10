# Sprint 5 checkpoint: GitHub Actions record, one run on demand, and sign-offs

This is backlog tasks T-024 and T-025, done as Sprint 6's T-001. It records what could only exist once Sprint 5's pull request was pushed for the last time and merged:
- the GitHub halves of these criteria:
  - [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4;
  - [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1;
- one run started on demand, the half of INT-0003 AC-2 that could not run before the workflow reached `main`;
- the state of every sign-off that waits on the project owner.

Every value below was read on 2026-11-10 from three sources:
- `gh api repos/crussella0129/Animus_Periapsis/actions/runs/<run>`;
- `gh api repos/crussella0129/Animus_Periapsis/actions/runs/<run>/jobs`;
- each job's own log, from `gh api repos/crussella0129/Animus_Periapsis/actions/jobs/<job>/logs --allow-escape-sequences`.

No job below was re-run, so every run has one attempt.

## Runs

Sprint 5's pull request is [crussella0129/Animus_Periapsis#7](https://github.com/crussella0129/Animus_Periapsis/pull/7).
- **Its tested head** was `87df514`. Sprint 5's test report covers that commit, with three samples.
- **Seven commits followed it**, all Book records but one: `81d130d`, `664b156`, `fad21f7`, `f3b5a26`, `0aa9623`, `e0a97cb` (sprint-5: close) and `b0675f6` (sprint-5: record checkpoint), its last head. `664b156` changed one test.
- **The merge:** the owner merged it at 2026-11-10T00:09:59Z as `55bdc075c128e186c21e780994653e4efaa2a9c8`.
- **The workflow in force:** `pull_request`, `push` on `main`, and `workflow_dispatch`.

| Run | Event, head | Attempt | Job | Conclusion | Started → completed (UTC) | Job time |
|---|---|---|---|---|---|---|
| [42721345619](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42721345619) | pull request, `b0675f6` | 1 | `cargo deny check` (129515793191) | success | 2026-11-09 23:33:03 → 23:33:18 | 15 s |
| 42721345619 | pull request, `b0675f6` | 1 | `lint (windows-latest)` (129515793590) | success | 23:33:03 → 23:33:36 | 33 s |
| 42721345619 | pull request, `b0675f6` | 1 | `cargo ci (windows-latest)` (129515793611) | success | 23:33:04 → 23:49:23 | 16 min 19 s |
| 42721345619 | pull request, `b0675f6` | 1 | `lint (ubuntu-latest)` (129515793649) | success | 23:33:04 → 23:33:24 | 20 s |
| 42721345619 | pull request, `b0675f6` | 1 | `cargo ci (ubuntu-latest)` (129515793652) | success | 23:33:04 → 23:42:09 | 9 min 5 s |
| [42726187298](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42726187298) | push to `main`, `55bdc07` (merge of #7) | 1 | `cargo ci (windows-latest)` (129532295959) | success | 2026-11-10 00:10:05 → 00:27:37 | 17 min 32 s |
| 42726187298 | push to `main`, `55bdc07` | 1 | `lint (ubuntu-latest)` (129532296351) | success | 00:10:06 → 00:10:36 | 30 s |
| 42726187298 | push to `main`, `55bdc07` | 1 | `cargo ci (ubuntu-latest)` (129532296386) | success | 00:10:05 → 00:19:50 | 9 min 45 s |
| 42726187298 | push to `main`, `55bdc07` | 1 | `lint (windows-latest)` (129532296418) | success | 00:10:06 → 00:11:24 | 1 min 18 s |
| 42726187298 | push to `main`, `55bdc07` | 1 | `cargo deny check` (129532296442) | success | 00:10:06 → 00:10:20 | 14 s |
| [42825642596](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42825642596) | on demand (`workflow_dispatch`), `dev` at `b0675f6` | 1 | `cargo ci (windows-latest)` (129862120428) | success | 2026-11-10 13:33:50 → 13:45:29 | 11 min 39 s |
| 42825642596 | on demand, `b0675f6` | 1 | `cargo ci (ubuntu-latest)` (129862120672) | success | 13:33:49 → 13:41:51 | 8 min 2 s |
| 42825642596 | on demand, `b0675f6` | 1 | `lint (ubuntu-latest)` (129862120769) | success | 13:33:49 → 13:34:07 | 18 s |
| 42825642596 | on demand, `b0675f6` | 1 | `cargo deny check` (129862120810) | success | 13:33:49 → 13:34:04 | 15 s |
| 42825642596 | on demand, `b0675f6` | 1 | `lint (windows-latest)` (129862121024) | success | 13:33:49 → 13:34:29 | 40 s |

All three runs concluded `success`. Every job waited 2 to 4 seconds for a runner, from `created_at` to `started_at`.

## What each job showed

| Job kind | Runner image, version | Toolchain | Build cache | Tests (`test:` line) | Summary |
|---|---|---|---|---|---|
| `cargo ci (windows-latest)`, pull request | `windows-2025-vs2026` 20261103.298.1 | 1.99.1-x86_64-pc-windows-msvc | restored | 36 units, 501 passed, 0 failed, 0 ignored | `ci: all 6 steps passed` |
| `cargo ci (ubuntu-latest)`, pull request | `ubuntu-26.04` 20261101.186.1 | 1.99.1-x86_64-unknown-linux-gnu | restored | 36 units, 502 passed, 0 failed, 0 ignored | `ci: all 6 steps passed` |
| `cargo ci (windows-latest)`, `main` | `windows-2025-vs2026` 20261103.298.1 | 1.99.1-x86_64-pc-windows-msvc | none found | 36 units, 501 passed, 0 failed, 0 ignored | `ci: all 6 steps passed` |
| `cargo ci (ubuntu-latest)`, `main` | `ubuntu-26.04` 20261101.186.1 | 1.99.1-x86_64-unknown-linux-gnu | none found | 36 units, 502 passed, 0 failed, 0 ignored | `ci: all 6 steps passed` |
| `lint`, both systems, pull request | as the `cargo ci` leg of its system | as above | restored | — | `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings` both passed |
| `lint`, both systems, `main` | as above | as above | none found | — | both passed |
| `cargo deny check`, both runs | `ubuntu-26.04` 20261101.186.1 | cargo-deny 0.20.2 | — | — | `advisories ok, bans ok, licenses ok, sources ok` |
| `cargo ci (windows-latest)`, on demand | `windows-2025-vs2026` 20261103.298.1 | 1.99.1-x86_64-pc-windows-msvc | restored | 36 units, 501 passed, 0 failed, 0 ignored | `ci: all 6 steps passed` |
| `cargo ci (ubuntu-latest)`, on demand | `ubuntu-26.04` 20261101.186.1 | 1.99.1-x86_64-unknown-linux-gnu | restored | 36 units, 502 passed, 0 failed, 0 ignored | `ci: all 6 steps passed` |
| `lint`, both systems, on demand | as above | as above | restored | — | both passed |
| `cargo deny check`, on demand | `ubuntu-26.04` 20261101.186.1 | cargo-deny 0.20.2 | — | — | `advisories ok, bans ok, licenses ok, sources ok` |

- **Test counts.** Sprint 5's tested head `87df514` ran 500 tests on Windows and 501 on Linux. `664b156` added one test, `stream_order_child`, so 501 and 502 are the counts expected, and they are the counts found in all six legs. The extra Linux test needs a real ext4 symbolic link, as before.
- **The `main` run was neither cancelled nor replaced** (INT-0003 AC-2). It is attempt 1, it concluded `success`, and no push to `main` has followed it: it is the latest run on `main` in `gh run list`.
- **Windows `cargo ci` time** (INT-0003 AC-11). 16 min 19 s on the pull request, with the build cache restored, and 17 min 32 s on `main`, with none. Both are under the criterion's 20 minutes and over its aim of 15. Each is one run of its commit. Neither is one of the three samples AC-11 asks for, which Sprint 5 took at `87df514`. The run on demand took 11 min 39 s on Windows, with the cache restored.
- **Runner image versions.** Each log has two version lines: one under `Runner Image Provisioner` (20261103.629 on Windows, 20261028.624 on Linux) and one under `Runner Image` (20261103.298.1 and 20261101.186.1). The table above gives the second. Sprint 5's record of Sprint 4's runs ([ci-sprint4.md](../../s5/sprint-tests/ci-sprint4.md)) gave the first under "Runner image"; the log of one of those runs (job 129377864817) shows the same two lines, so that record named the provisioner's version. It is left as it is, since Sprint 5 is closed.

## The run on demand

Sprint 5 could not run CI on demand: `workflow_dispatch` works only once the workflow that declares it is on the default branch, which the merge of #7 did.

On 2026-11-10, with the owner's approval of this sprint's plan, `gh workflow run ci --ref dev` started run [42825642596](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42825642596):
- **Event:** `workflow_dispatch`.
- **Head:** `dev` at `b0675f6e1a1df3de13b6275635d40a5e81a41485`, which was `dev`'s head on GitHub then. Sprint 6's own commits had not been pushed.
- **Jobs:** the five the workflow declares, each in the table above, each `success`. The run concluded `success` at attempt 1.
- **What it ran:** the same commit as the pull-request run, with the same 501 and 502 tests, both `cargo ci` legs ending `ci: all 6 steps passed`.

So CI runs on a pull request, on a push to `main`, and on demand, which is the whole of INT-0003 AC-2's list of triggers.

## Owner sign-offs

The list comes from this search, run on 2026-11-10, which finds every acceptance criterion whose `Signers` detail names the project owner:

```bash
grep -nE "^\s+- Signers:.*project owner" docs/intents/*.md
```

It finds 17. INT-0004 AC-5, whose signer is a human reviewer, is listed with them because every sprint since Sprint 1 has carried it with the owner's.

| Criterion | What the owner signs | Chapter's state | Signed |
|---|---|---|---|
| INT-0001 AC-1 | the charter's decomposition into child intents | proposed | no; carried from earlier sprints |
| INT-0002 AC-1 | the typed-acceptance grammar | active | no; carried from earlier sprints |
| INT-0003 AC-5 | the design credits, and that no code was copied | active | no; carried from earlier sprints |
| INT-0004 AC-5 | the Windows durability strategy and its crash evidence | active | no; carried from earlier sprints |
| INT-0006 AC-6 | each change to the format specification | active | no; carried from earlier sprints |
| INT-0003 AC-7 | package names reserved before any crate is published | active | no; not due until publication |
| INT-0035 AC-12 | the scale model and the cost rules | planned | no |
| INT-0037 AC-8 | the vocabulary table and the refused words | planned | no; two names chosen, see below |
| INT-0011 AC-5 | the divergence policy rules | proposed | no |
| INT-0016 AC-8 | the classes of owner and what each may change | proposed | no |
| INT-0020 AC-6 | panel and signer authority | proposed | no |
| INT-0025 AC-3 | the Git bridge released before any public release | proposed | no |
| INT-0031 AC-1 | the hero artwork | proposed | no |
| INT-0032 AC-7 | the semantic and editorial split, and the kinds of link | proposed | no |
| INT-0033 AC-5 | the topic list | proposed | no |
| INT-0034 AC-8 | the line between the core and the layers | proposed | no |
| INT-0036 AC-7 | the speed workloads and thresholds | proposed | no |
| INT-0038 AC-7 | what the live view shows, and whether a graphical client is built | proposed | no |

None of these has been given in the conversation. Since the Sprint 5 checkpoint the owner has:
- merged pull request #7. That is acceptance of the checkpoint, not a sign-off;
- given the direction for this sprint: thousands of agents, speed, and a more intuitive command set and UI;
- approved this sprint's plan outline on 2026-11-10 with three choices: `verify` as the name of the integrity check, `log` for changes with `ops` for operations, and the GitHub actions this sprint takes.

The second and third choices settle two names of INT-0037's vocabulary table. AC-8 asks for the whole table and the list of refused words, which were not put to the owner as a table to sign, so AC-8 stays unsigned.

Backlog T-035 carries the eight that are due or already carried: the first eight rows above. The other ten fall due when their chapters are planned.
