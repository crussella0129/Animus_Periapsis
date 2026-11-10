# Sprint 5 Research Report

## Intents Reviewed
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — **selected and to be revised**; current state: active.
  - **The owner's request (2026-11-09).** "Adequate coverage but not outrageous in terms of waiting." Clippy should be visible in CI, not hidden inside one check.
  - **What holds now:**
    - AC-2 and AC-4 are claims about CI, and `ci.yml` is pinned word for word by `xtask/src/pins.rs`.
    - The T-007 clause in `xtask/src/workflow.rs` requires `push` on `dev` and `main`, a `pull_request` trigger, and a `cargo ci` job on both platforms.
  - **The revision.** Name how CI triggers and splits its checks, and record the time budget as a Consequence or criterion.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — **selected**; current state: active. Every check that runs on Linux must keep running on Windows. This sprint changes how they run, not where.
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — **selected**; current state: active.
  - The crash matrices are about 35 of a CI run's 38 minutes.
  - Any speed-up must keep every cell, every digest and the reach table unchanged. Losing coverage to save time is not the goal.
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — **selected for backlog T-020 only**: record the Sprint 4 checkpoint's GitHub Actions runs and the pending sign-offs. Current states: proposed and active.

## 1. Sprint Goal
Make CI fast enough to wait for, without dropping a single test, and make each kind of check visible on its own:
- **Speed.** Build the tests with optimisation, and run test files in parallel, so a PR's slowest leg finishes in about 15 minutes or less, instead of 38–49.
- **Visibility.** fmt and Clippy, the tests, cargo-deny, and the policy/book/reach checks each appear as their own named GitHub check.
- **Less waste:**
  - one run per commit instead of a `push` run plus a `pull_request` run;
  - a newer push cancels the runs it replaces;
  - the cargo-deny install retries a bad download and checks its checksum, so a corrupt file never reaches the check.
- **Carry forward:** T-020, the Sprint 4 CI record and sign-offs.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| `.github/workflows/ci.yml` | high | Two jobs: `cargo ci` on an ubuntu/windows matrix and `cargo deny check` through `EmbarkStudios/cargo-deny-action@v2`. Triggers: `push` on `dev` and `main`, plus `pull_request`. No cache and no `concurrency`. |
| `crates/xtask/src/ci.rs` | high | `cargo ci` runs six steps in order: fmt, clippy, test, reach, policy, book. The test step is one `cargo test --workspace --locked`, so Cargo runs test files one after another. It clears hostile variables (`PERI_GOLDEN_CREATE`, `PERI_WRITER_CHILD`, `PROPTEST_*`, `CLIPPY_CONF_DIR`) and the recorded reach. |
| `crates/xtask/src/pins.rs` | high | `ci_workflow_is_the_reviewed_one` pins `ci.yml` byte for byte, as `CARGO_CONFIG` and `DENY_POLICY` pin theirs. |
| `crates/xtask/src/workflow.rs` | high | The structural T-007 check. It needs to change wherever the triggers or the job shape change. |
| `Cargo.toml` | high | No `[profile]` section, so tests build unoptimised. |
| `crates/sim/src/reach.rs` | medium | Each scenario's reach is written by one designated test, through a temporary file and a rename. That is safe whether test files run in parallel or one at a time. |
| `crates/xtask/src/main.rs` | medium | `cargo ci` refuses a target directory outside the checkout (Sprint 1 T-008). That rules out a shared, out-of-tree cache path. |
| `docs/work/tasks.md` | medium | T-020 (carry forward). T-021, test selection from reach, is a different lever and stays in the backlog. |

## Measurements
**GitHub, PR #6, Ubuntu leg** (job `129038433875`, 38 min 20 s):

| Part | Time |
|---|---|
| fmt, clippy and the test build | about 1 min |
| `workspace/tests/materialise_crash.rs` | 1,156 s |
| `store/tests/writers.rs` | 493 s |
| `workspace/tests/crash.rs` | 269 s |
| `store/tests/crash.rs` | 172 s |
| `fsck` | 63 s |
| `edits` | 42 s |
| `properties` | 28 s |
| `vectors` | 15 s |
| everything else | under 5 s |

The Windows legs took 25 min 45 s and 49 min 1 s for the same commit. Their split was the same; `materialise_crash` alone took 1,385 s.

**Locally, on a 24-thread Windows machine,** with `opt-level = 1` for workspace crates and `3` for dependencies, applied through `--config` with nothing in the repo changed:

| Test file | Unoptimised (Sprint 4 log) | Optimised |
|---|---|---|
| `store` crash | 538 s | 19 s |
| `workspace` crash | 560 s | 48 s |
| `materialise_crash` | 999 s | 204 s |
| `writers` | 131 s | 66 s |

- The whole optimised suite, run one file after another, took 587 s, after a 61 s build. The unoptimised `cargo ci` took 2,060 s.
- The six slowest files, run all at once, took 236 s. `materialise_crash` sets that time.
- Every matrix passed when optimised. The digests were not compared in this experiment; that is for the Test phase.
- Two `xtask` tests failed, as they should: they refuse a target directory outside the checkout, and this experiment used one.

**cargo-deny.** On PR #6, `cargo-deny-action@v2` downloaded cargo-deny 0.20.2 and got a non-gzip body ("gzip: invalid magic"). That release asset exists. Re-running the failed jobs passed: jobs `129244467046` and `129244396365`.

## 3. External Sources
- [nextest: how it works](https://nexte.st/book/how-it-works.html), with its [partitioning](https://nexte.st/book/partitioning) page.
  - `cargo test` runs whole test binaries one after another. nextest lists every test first, then runs each in its own process, all in parallel, and can split a suite across CI jobs.
  - This confirms why our serial binaries waste a 4-vCPU runner while one long matrix runs.
  - Cost: nextest is another tool every machine needs before it can run `cargo ci`. INT-0003 already keeps cargo-deny out of `cargo ci` for that very reason.
- [GitHub: supported runners and hardware resources](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Standard Linux and Windows runners for public repositories have 4 vCPU and 16 GB, and are free. So the budget is wall-clock time on 4 cores, not minutes billed.
- [GitHub: control the concurrency of workflows and jobs](https://docs.github.com/en/enterprise-cloud@latest/actions/how-tos/writing-workflows/choosing-what-your-workflow-does/control-the-concurrency-of-workflows-and-jobs).
  - A `concurrency` group with `cancel-in-progress: true` cancels older runs in the same group.
  - Runs from different events (`push` and `pull_request`) do not dedupe through one group in any documented way. Removing the duplicate is a trigger choice, not a concurrency setting.
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache). It caches `~/.cargo` and `target/` dependency artefacts, keyed by the toolchain and `Cargo.lock`. It works on Linux and Windows. It keeps `target` inside the checkout, which the Sprint 1 T-008 rule requires.
- [taiki-e/install-action](https://github.com/taiki-e/install-action).
  - Installs pinned prebuilt tools, cargo-deny among them, and verifies their checksums by default (its `checksum` input defaults to true). A corrupt download therefore fails with a clear message instead of a gzip error inside a Docker build.
  - Whether the official action retries downloads is unconfirmed: retry code was seen only in a fork.
  - The workflow step would then run a plain `cargo deny check`, which `workflow.rs` already accepts.

## 4. Risks, Unknowns, Dependencies
- **Do optimised builds change any result?**
  - The simulator is deterministic and uses no floating point in what it digests. Optimisation keeps `debug_assertions` and overflow checks on, because those are separate profile settings.
  - Every matrix digest, every cell count and the reach table must still match Sprint 4's. That comparison is the Test phase's first check.
- **`profile.dev` or `profile.test`?**
  - `cargo test` uses the `test` profile, which inherits from `dev`; Clippy and `cargo build` use `dev`.
  - Setting the profile on `dev` keeps one build cache, and optimises `praps` debug builds too.
  - The plan fixes which, and records it.
- **Running test files in parallel without a new tool.**
  - `cargo test --no-run --message-format=json` names every test executable. `cargo ci` could run them several at a time, buffer each one's output and print it in a fixed order. Doc tests still run as `cargo test --doc`.
  - That is new code in `xtask`, with its own tests: ordering, failures, and the cleared variables reaching each child.
  - The alternative is nextest, CI-only or required everywhere. The plan has to choose, and the owner may prefer the tool.
- **One run per commit.**
  - Dropping `push` on `dev` and keeping `pull_request` means `dev` is built only through its PR, which tests the merge of `dev` into `main`.
  - Under Sprint Loops `dev` is pushed only at a checkpoint, which always opens a PR. Still, AC-2's "every CI build of the work branch" needs rewording, and `workflow.rs` needs to change.
  - A `workflow_dispatch` trigger allows a manual run of any branch.
- **Separate checks.**
  - The lint job (`cargo fmt --all --check`, `cargo clippy …`) can run the cargo commands directly on both platforms. It fails in about 2 minutes, while `cargo ci` still runs every step, so the two overlap.
  - The other way is to give each `cargo ci` step its own job. That needs a per-step command, and AC-9 forbids flags.
- **Caching.** The test build is short now (28 s on Ubuntu, unoptimised), and optimisation makes it longer, so a cache matters more than before. Windows caches are slower to restore. The Test phase measures both legs.
- **The Windows leg** was 1.3× slower than Ubuntu on the slowest file, and two Windows runs of one commit took 26 and 49 minutes. So the target has to allow for runner variance, and Test records several runs.
- **Dependencies:** no new crate. Possibly new GitHub Actions (`Swatinem/rust-cache`, `taiki-e/install-action`), each pinned by tag like `actions/checkout@v7`, and possibly nextest.

## 5. Recommended Approach
1. **T-020:** record the Sprint 4 checkpoint's runs, including the deny failure and its passing re-runs, and carry forward the sign-offs.
2. **Optimised test builds:** a `[profile.dev]` in `Cargo.toml` with `opt-level = 1` for workspace crates and `3` for dependencies, keeping debug assertions and overflow checks on. Then show every Sprint 4 digest, count and the reach table unchanged.
3. **Parallel test files inside `cargo ci`:** the test step builds once, then runs the test executables several at a time and prints each one's output whole, in a fixed order. No new tool. nextest is the alternative, for the owner to choose.
4. **The workflow:**
   - jobs `lint (os)` (fmt and Clippy), `cargo ci (os)` and `cargo deny check`;
   - `Swatinem/rust-cache` on the Rust jobs;
   - cargo-deny installed with `taiki-e/install-action` at 0.20.2, with its checksum verified, then `cargo deny check`;
   - triggers `push: [main]`, `pull_request` and `workflow_dispatch`, with a `concurrency` group that cancels superseded runs;
   - `pins.rs` and `workflow.rs` updated to the new reviewed workflow.
5. **INT-0003 revised:** AC-2's CI wording, the separate checks under AC-4, and a Consequence recording the time budget and that no test was dropped to meet it. README's CI description updated to match.
6. **Measure in GitHub:** this sprint's own PR runs are its evidence. Several runs per leg, against a budget of about 15 minutes for the slowest leg.
7. **If the budget is missed:**
   - split `materialise_crash`'s longest test into shards over seed ranges, keeping its digest over all cells;
   - otherwise defer to T-021, test selection from reach.

## Artifacts
- This report is the only artifact. The local timing scripts were throwaway (`--config` overrides, an out-of-tree target directory) and changed nothing in the repository. The numbers they produced are recorded above.
