# Sprint 3 checkpoint: GitHub Actions record and sign-offs

This is backlog task T-017, done as Sprint 4's T-001. It records two things that could only exist once `dev` was pushed at the Sprint 3 checkpoint:
- the GitHub-matrix halves of [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4 and [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1, including the `cargo deny` run on Linux that Sprint 3 could not run in WSL;
- the state of the owner's sign-offs.

Every value below was read on 2026-11-09 from `gh run view <run> --json …` and `gh run view --job <job> --log`.

## Runs

The checkpoint pull request is [crussella0129/Animus_Periapsis#5](https://github.com/crussella0129/Animus_Periapsis/pull/5). Its last head was `4f05dd1d3bdfb2826a741aacdda2fddbfce00961` (sprint-3 record checkpoint), which changes only Book records after the tested head `ad22319`. The merge into `main` is `7b2fd59bbf2e8304e6389d708f03c3c314bd654c`, which `dev` was then fast-forwarded to and pushed. Every run concluded `success`.

| Run | Event, head | Job | Conclusion | Runner image | Toolchain | Tests | `cargo ci` steps |
|---|---|---|---|---|---|---|---|
| [42443548089](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42443548089) | pull request, `4f05dd1` | `cargo ci (windows-latest)` (128617625616) | success | `windows-2025-vs2026` 20261103.629 | 1.99.1-x86_64-pc-windows-msvc | 407 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42443548089 | pull request, `4f05dd1` | `cargo ci (ubuntu-latest)` (128617625895) | success | `ubuntu-26.04` 20261028.624 | 1.99.1-x86_64-unknown-linux-gnu | 408 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42443548089 | pull request, `4f05dd1` | `cargo deny check` (128617625710) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | advisories, bans, licenses, sources ok |
| [42443543930](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42443543930) | push to `dev`, `4f05dd1` | `cargo ci (windows-latest)` (128617613683) | success | `windows-2025-vs2026` | 1.99.1-x86_64-pc-windows-msvc | 407 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42443543930 | push to `dev`, `4f05dd1` | `cargo ci (ubuntu-latest)` (128617613713) | success | `ubuntu-26.04` | 1.99.1-x86_64-unknown-linux-gnu | 408 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42443543930 | push to `dev`, `4f05dd1` | `cargo deny check` (128617613819) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | advisories, bans, licenses, sources ok |
| [42536778981](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42536778981) | push to `main`, `7b2fd59` (merge of #5) | `cargo ci (windows-latest)` (128902705244) | success | `windows-2025-vs2026` | 1.99.1-x86_64-pc-windows-msvc | 407 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42536778981 | push to `main`, `7b2fd59` | `cargo ci (ubuntu-latest)` (128902704883) | success | `ubuntu-26.04` | 1.99.1-x86_64-unknown-linux-gnu | 408 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42536778981 | push to `main`, `7b2fd59` | `cargo deny check` (128902705289) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | advisories, bans, licenses, sources ok |
| [42537492546](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42537492546) | push to `dev`, `7b2fd59` | `cargo ci (windows-latest)` (128904917856) | success | `windows-2025-vs2026` | 1.99.1-x86_64-pc-windows-msvc | 407 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42537492546 | push to `dev`, `7b2fd59` | `cargo ci (ubuntu-latest)` (128904917901) | success | `ubuntu-26.04` | 1.99.1-x86_64-unknown-linux-gnu | 408 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42537492546 | push to `dev`, `7b2fd59` | `cargo deny check` (128904917663) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | advisories, bans, licenses, sources ok |

How the columns were read:
- **Toolchain.** The `active toolchain` line that rustup printed. `rust-toolchain.toml` sets it.
- **Tests.** The sum of every `test result:` line in each job's log, over 46 test binaries:
  - 407 tests on Windows and 408 on Linux. The extra Linux test needs a real ext4 symlink.
  - These equal the counts in Sprint 3's [test report](../../s3/sprint-tests/test-report.md), recorded locally on Windows and in WSL.
  - Each `cargo ci` leg also printed `ci: all 5 steps passed`.
- **`cargo deny` on Linux.** Every run's Linux `cargo deny check` job passed. That is the half Sprint 3 deferred, because cargo-deny was not installed in WSL.
- **Named tests.** In the `main` run, both `cargo ci` legs printed `... ok` for these:
  - `crash_matrix_finds_no_violations`;
  - `snapshot_matrix_finds_no_violations`;
  - `materialise_matrix_finds_no_violations`.

  The Windows leg also printed it for `real_hazards_are_refused_on_ntfs`, and the Linux leg for `real_hazard_trees_materialise_on_ext4`. As in earlier sprints, `cargo test` runs without `--nocapture`, so the matrices' cell counts and digests are in Sprint 3's [test-matrix-log.md](../../s3/sprint-tests/test-matrix-log.md), not in these logs.

## Owner sign-offs

None of the following has been given in the conversation:
- INT-0001 AC-1 (charter decomposition);
- INT-0002 AC-1 (typed-acceptance grammar);
- INT-0003 AC-5 (design credits);
- INT-0006 AC-6 (format v1's Sprint 0 sections);
- INT-0004 AC-5 (the Windows durability strategy).

Between the Sprint 3 checkpoint and this record, the owner's messages were:
- a proposal for coded breakpoints, which became this sprint's goal;
- a diagram of runs × units;
- "No, I merged PR number five so you are good to take what we talked about, extract the value, and continue".

Merging PR #5 is recorded as acceptance of the checkpoint only, not as any of these sign-offs. They are carried forward again in backlog task T-020, which replaces T-017.
