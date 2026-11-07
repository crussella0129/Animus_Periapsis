# Sprint 0 checkpoint: GitHub Actions record and sign-offs

Backlog task T-009, done as Sprint 1's T-009. It records the GitHub-matrix
halves of [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2
to AC-4 and [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3, which
could only run once `dev` was pushed at the Sprint 0 checkpoint, and the state
of the owner's sign-offs. Every value below was read from
`gh run view <run> --json …` and `gh run view --job <job> --log` on
2026-11-06.

## Runs at the Sprint 0 checkpoint head `5062157`

Head `5062157547786200c81d9f559a28fdafc081f88f` (sprint-0 close), on the
checkpoint pull request
[crussella0129/Animus_Periapsis#1](https://github.com/crussella0129/Animus_Periapsis/pull/1).
Both runs concluded `success`.

| Run | Event | Job | Conclusion | Runner image | Toolchain | Tests | `cargo ci` steps | Golden tests |
|---|---|---|---|---|---|---|---|---|
| [42227550987](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42227550987) | push to `dev` | `cargo ci (ubuntu-latest)` (127924781182) | success | `ubuntu-26.04` 20261101.186.1 | rustc 1.99.1 (58cf8f9c3 2026-10-13), x86_64-unknown-linux-gnu | 171 passed, 0 failed | PASS fmt, clippy, test, policy, book | `golden_v1_corpus_matches`, `spec_examples_match_golden_files`, `every_golden_file_is_a_corpus_case` ok |
| 42227550987 | push to `dev` | `cargo ci (windows-latest)` (127924781142) | success | `windows-2025-vs2026` 20261103.298.1 | rustc 1.99.1 (58cf8f9c3 2026-10-13), x86_64-pc-windows-msvc | 171 passed, 0 failed | PASS fmt, clippy, test, policy, book | the same three ok |
| 42227550987 | push to `dev` | `cargo deny check` (127924781464) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | — | advisories ok, bans ok, licenses ok, sources ok |
| [42227551468](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42227551468) | pull request | `cargo ci (ubuntu-latest)` (127924782264) | success | `ubuntu-26.04` 20261101.186.1 | rustc 1.99.1 (58cf8f9c3 2026-10-13) | 171 passed, 0 failed | PASS fmt, clippy, test, policy, book | the same three ok |
| 42227551468 | pull request | `cargo ci (windows-latest)` (127924782556) | success | `windows-2025-vs2026` 20261103.298.1 | rustc 1.99.1 (58cf8f9c3 2026-10-13) | 171 passed, 0 failed | PASS fmt, clippy, test, policy, book | the same three ok |
| 42227551468 | pull request | `cargo deny check` (127924782741) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | — | advisories ok, bans ok, licenses ok, sources ok |

The test counts are the sum of every `test result:` line in each job's log.
They equal the 171 tests the Sprint 0 test report recorded locally on Windows
and in WSL at `c342412`; `5062157` changed only Book records after that head.

## The pinned workflow after the checkout bump

Dependabot's
[crussella0129/Animus_Periapsis#2](https://github.com/crussella0129/Animus_Periapsis/pull/2)
bumped `actions/checkout` from v4 to v7 in `.github/workflows/ci.yml`.

- Its first run,
  [42237753919](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42237753919)
  at `d17b135`, failed both `cargo ci` legs at one test,
  `pins::ci_workflow_is_the_reviewed_one`, because the reviewed copy of the
  workflow in `crates/xtask/src/pins.rs` still said `@v4`. That failure is the
  pin working as INT-0003 intends: the workflow changes only together with
  its reviewed copy.
- Commit `2c7e74a` updated the reviewed copy on the PR's branch. Run
  [42303659961](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42303659961)
  at `2c7e74a` concluded `success`: `cargo ci (ubuntu-latest)` (job
  128187485235, `ubuntu-26.04`) and `cargo ci (windows-latest)` (job
  128187484947, `windows-2025-vs2026`) each passed 171 tests with every step
  PASS and the golden tests ok; `cargo deny check` (job 128187484664) passed
  all four checks.
- The owner merged PR #2 into `dev` (`7e84b43`) before Sprint 1's first
  commit.

## Owner sign-offs

The owner merged checkpoint PR #1 into `main` (`cff829f`, 2026-11-06). That is
acceptance of the Sprint 0 checkpoint. It is not recorded as any of the four
named sign-offs, which the owner has not yet given in the conversation or on a
pull request:

| Criterion | What it signs off | State |
|---|---|---|
| [INT-0001](../../../intents/INT-0001-periapsis-charter.md) AC-1 | The charter's decomposition into child intents (now including INT-0032 to INT-0034) | not given; carried forward |
| [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) AC-1 | The typed-acceptance grammar | not given; carried forward |
| [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-5 | The design credits in `docs/prior-art.md`, and that no code was copied | not given; carried forward |
| [INT-0006](../../../intents/INT-0006-format-v1.md) AC-6 | Format v1's Sprint 0 sections (primitives, `hashref`, envelope header, blob body); the provisional `operation` body is excluded | not given; carried forward |

The Sprint 1 Loop carries each one forward again; none is recorded on the
owner's behalf.
