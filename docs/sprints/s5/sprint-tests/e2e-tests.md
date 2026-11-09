# Sprint 5 End-to-End Test Results

**Setup.** At the owner's request (2026-11-09), Test pushed `dev` and opened the sprint's pull request into `main` early: [crussella0129/Animus_Periapsis#7](https://github.com/crussella0129/Animus_Periapsis/pull/7). Nothing was merged.

**Sources.** Every value below was read from `gh api repos/crussella0129/Animus_Periapsis/actions/runs/<run>/attempts/<n>/jobs`, the jobs' logs (`…/actions/jobs/<job>/logs`) and `gh run list`.

**The runners:**
- `ubuntu-26.04` 20261028.624;
- `windows-2025-vs2026` 20261103.629;
- toolchain 1.99.1-x86_64-unknown-linux-gnu and -pc-windows-msvc.

## `sampled_runs_meet_the_budget`

**Tested head `87df514`, run [42707577882](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42707577882).** Sample 1 is the pull request's first completed run of that commit. Samples 2 and 3 are "re-run all jobs" attempts (`gh run rerun 42707577882`). Each job's time runs from its own `started_at` to `completed_at`.

| Job | Sample 1 | Sample 2 | Sample 3 |
|---|---|---|---|
| `cargo ci (ubuntu-latest)` | 9:04 | 8:58 | 9:07 |
| `cargo ci (windows-latest)` | 9:41 | 15:50 | **16:02** |
| `lint (ubuntu-latest)` | 0:21 | 0:19 | 0:19 |
| `lint (windows-latest)` | 0:37 | 0:45 | 0:43 |
| `cargo deny check` | 0:14 | 0:14 | 0:15 |
| **Slowest job** | **9:41** | **15:50** | **16:02** |

- **Conclusions.** Every job of every sample succeeded.
- **Queue time.** `created_at` to `started_at` was 2 to 4 seconds for every job.
- **Cache state, as each log reports it.** Every `lint` and `cargo ci` job of all three samples printed `Cache hit for: v0-rust-{ci,lint}-{Linux,Windows_NT}-x64-…`.
  - Sample 1 was **warm** too. rust-cache's key comes from the toolchain and `Cargo.lock`, not the sources, so it found the entries saved by the earlier runs of this pull request.
  - The cold case was measured in the first round below.

**Result:** AC-11's threshold is met, 3 of 3. No job of any sample took more than 20 minutes; the slowest took 16:02. The aim of 15 minutes was met in sample 1 and missed in samples 2 and 3, by the Windows suite alone.

**What sets the Windows time.** It varies from runner to runner. In both runs over 15 minutes, `materialise_crash` alone took 715 s and 729 s; in sample 1 it took 397 s. Ubuntu's `materialise_crash` took 435 to 443 s in all three samples. The Windows suite is that one unit plus a few seconds, so it is the next thing to shorten (backlog T-027).

### The first round, at `3289008`, before the code review fixes

Run [42698665267](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42698665267), the pull request's first run (cold cache), then two "re-run all jobs" attempts:

| Job | Sample 1 (cold: "No cache found") | Sample 2 (warm) | Sample 3 (warm) |
|---|---|---|---|
| `cargo ci (ubuntu-latest)` | 8:53 | 6:51 | 9:09 |
| `cargo ci (windows-latest)` | 12:46 | **16:12** | 11:42 |
| `lint (ubuntu-latest)` | 0:29 | 0:17 | 0:19 |
| `lint (windows-latest)` | 0:50 | 0:33 | 1:05 |
| `cargo deny check` | 0:15 | 0:14 | 0:14 |

Every job succeeded, and the slowest job was 16:12. With a cold cache, both lint jobs and both suites still finished within 13 minutes.

**For comparison:** Sprint 4's checkpoint ran `cargo ci` in 25:45 to 49:01 per leg ([ci-sprint4.md](ci-sprint4.md)).

## `checks_appear_separately`

`gh pr checks 7` lists five checks:
- `cargo ci (ubuntu-latest)`;
- `cargo ci (windows-latest)`;
- `cargo deny check`;
- `lint (ubuntu-latest)`;
- `lint (windows-latest)`.

Clippy and fmt now show by name on both platforms. Each lint job's log shows `Run cargo fmt --all --check`, then `Run cargo clippy --workspace --all-targets --locked -- -D warnings`.

## `every_executable_runs_in_ci`

In every `cargo ci` leg of the three samples at `87df514`, the test step printed `test: 45 test executables built, 45 run`. The totals were:
- Linux: `test: 36 units, 501 passed, 0 failed, 0 ignored`;
- Windows: `test: 36 units, 500 passed, 0 failed, 0 ignored`.

Both equal the local counts for their platform ([unit-tests.md](unit-tests.md)). At `3289008`, before the executable count was printed, the totals were 497 and 496, again equal to the local ones.

## `push_to_dev_starts_no_run`

`dev` was pushed three times during Test, at `3289008`, `1ebb35e` and `87df514`. None started a `push` run. `gh run list` shows only `pull_request` runs for them, and the last `push` run is `42681766482`, the merge of PR #6 into `main`.

## `superseded_run_is_cancelled`

`1ebb35e` was pushed, and its run [42707506720](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42707506720) started. 25 seconds later `87df514` was pushed:
- `42707506720` ended **`cancelled`**;
- run `42707577882` for `87df514` ran to `success`.

The cancelled run was never used as a sample.

## `cargo_ci_is_idempotent`

In a fresh WSL clone at `87df514`, `cargo ci` ran twice, in 253 s and then 207 s. `git status --porcelain` was empty after each run (INT-0003 AC-9). The same held at `3289008`, in 241 s and 202 s ([integration-tests.md](integration-tests.md)).

## The deny job's install

The `cargo deny check` job's log shows each step:
- `tool: cargo-deny@0.20.2` and `checksum: true`;
- the download of `cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz`;
- `verifying sha256 checksum`;
- `advisories ok, bans ok, licenses ok, sources ok`.

No `cargo deny check` job failed in any of this sprint's runs.

## Not yet possible
- **`dispatch_runs_on_dev`.** `workflow_dispatch` works only once the workflow declaring it is on `main`, so it unlocks with this sprint's merge. Sprint 6 records it through backlog T-025.
- **Nighthawk (INT-0004 AC-6).** It unlocks when the owner approves Tailscale.
