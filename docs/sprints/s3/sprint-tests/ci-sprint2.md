# Sprint 2 checkpoint: GitHub Actions record and sign-offs

This is backlog task T-013, done as Sprint 3's T-001. It records two things that could only exist once `dev` was pushed at the Sprint 2 checkpoint:
- the GitHub-matrix halves of [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4, [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1 and [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3;
- the state of the owner's sign-offs.

Every value below was read on 2026-11-07 from `gh run view <run> --json …` and `gh run view --job <job> --log`.

## Runs

The checkpoint pull request is [crussella0129/Animus_Periapsis#4](https://github.com/crussella0129/Animus_Periapsis/pull/4). Its last head was `bda21b13022994a2639acf1a846e90943a0bf590` (sprint-2 record checkpoint), which changes only Book records after the tested head. The merge into `main` is `6e972bb24ebcd715a90f7e057884b93068473c51`. Every run concluded `success`.

| Run | Event, head | Job | Conclusion | Runner image | Toolchain | Tests | `cargo ci` steps |
|---|---|---|---|---|---|---|---|
| [42403413644](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42403413644) | pull request, `bda21b1` | `cargo ci (ubuntu-latest)` (128495876453) | success | `ubuntu-26.04` 20261101.186.1 | 1.99.1-x86_64-unknown-linux-gnu | 332 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42403413644 | pull request, `bda21b1` | `cargo ci (windows-latest)` (128495876364) | success | `windows-2025-vs2026` 20261103.298.1 | 1.99.1-x86_64-pc-windows-msvc | 332 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42403413644 | pull request, `bda21b1` | `cargo deny check` (128495876200) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | advisories, bans, licenses, sources ok |
| [42403409376](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42403409376) | push to `dev`, `bda21b1` | `cargo ci (ubuntu-latest)` (128495863558) | success | `ubuntu-26.04` | 1.99.1-x86_64-unknown-linux-gnu | 332 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42403409376 | push to `dev`, `bda21b1` | `cargo ci (windows-latest)` (128495863782) | success | `windows-2025-vs2026` | 1.99.1-x86_64-pc-windows-msvc | 332 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42403409376 | push to `dev`, `bda21b1` | `cargo deny check` (128495863831) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | advisories, bans, licenses, sources ok |
| [42404724911](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42404724911) | push to `main`, `6e972bb` (merge of #4) | `cargo ci (ubuntu-latest)` (128499845866) | success | `ubuntu-26.04` | 1.99.1-x86_64-unknown-linux-gnu | 332 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42404724911 | push to `main`, `6e972bb` | `cargo ci (windows-latest)` (128499845815) | success | `windows-2025-vs2026` | 1.99.1-x86_64-pc-windows-msvc | 332 passed, 0 failed | PASS fmt, clippy, test, policy, book |
| 42404724911 | push to `main`, `6e972bb` | `cargo deny check` (128499845671) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | advisories, bans, licenses, sources ok |

How the columns were read:
- **Toolchain.** The `active toolchain` line rustup printed, set by `rust-toolchain.toml`.
- **Tests.** The sum of every `test result:` line in each job's log: 41 test binaries, 332 tests. This equals the 332 tests Sprint 2's [test report](../../s2/sprint-tests/test-report.md) recorded locally on Windows and in WSL. Each `cargo ci` leg also printed `ci: all 5 steps passed`.
- **Named tests.** In every `cargo ci` leg, these were each `... ok`: `crash_matrix_finds_no_violations`, `matrix_runs_are_reproducible`, `pointer_file_variant_is_caught`, `write_in_place_variant_is_caught`, `fsck_is_clean_on_matrix_stores`, `golden_v1_corpus_matches`, `snapshot_matrix_finds_no_violations`, `snapshot_matrix_is_reproducible` and `edit_corpus_loses_nothing`. As in Sprint 1, `cargo test` runs without `--nocapture`, so the matrices' per-cell counts are in Sprint 2's [test-matrix-log.md](../../s2/sprint-tests/test-matrix-log.md), not in these logs.

## Owner sign-offs

None of the following has been given in the conversation. Between the Sprint 2 checkpoint and this record, the owner's only message was "Merged, continue".
- INT-0001 AC-1 (charter decomposition);
- INT-0002 AC-1 (typed-acceptance grammar);
- INT-0003 AC-5 (design credits);
- INT-0006 AC-6 (format v1's Sprint 0 sections);
- INT-0004 AC-5 (the Windows durability strategy).

Merging PR #4 is recorded as acceptance of the checkpoint only, not as any of these sign-offs. They are carried forward again, in backlog task T-017, which replaces T-013.
