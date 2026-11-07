# Sprint 1 checkpoint: GitHub Actions record and sign-offs

This is backlog task T-010, done as Sprint 2's T-001. It records two things that could only exist once `dev` was pushed at the Sprint 1 checkpoint:
- the GitHub-matrix halves of [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-2 to AC-4 and [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3;
- the state of the owner's sign-offs.

Every value below was read on 2026-11-07 from `gh run view <run> --json …` and `gh run view --job <job> --log`.

## Runs

The checkpoint pull request is [crussella0129/Animus_Periapsis#3](https://github.com/crussella0129/Animus_Periapsis/pull/3). Its last head was `9ffa162025e3c70e3f8e1470cf588baf7f9d37e9` (sprint-1 record checkpoint), which changes only Book records after the tested head `9f138f4`. The merge into `main` is `5c3b81bace91d2696187edbd81eef8b5a33aacf1`. Every run concluded `success`.

| Run | Event, head | Job | Conclusion | Runner image | Toolchain | Tests | `cargo ci` steps | Other |
|---|---|---|---|---|---|---|---|---|
| [42329906757](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42329906757) | pull request, `9ffa162` | `cargo ci (ubuntu-latest)` (128273800675) | success | `ubuntu-26.04` 20261101.186.1 | rustc 1.99.1 (58cf8f9c3 2026-10-13) | 242 passed, 0 failed | PASS fmt, clippy, test, policy, book | 6 named matrix, golden and fsck tests ok |
| 42329906757 | pull request, `9ffa162` | `cargo ci (windows-latest)` (128273800909) | success | `windows-2025-vs2026` 20261103.298.1 (Windows Server 2025) | rustc 1.99.1 | 242 passed, 0 failed | PASS fmt, clippy, test, policy, book | the same 6 ok |
| 42329906757 | pull request, `9ffa162` | `cargo deny check` (128273800798) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | — | advisories, bans, licenses, sources ok |
| [42329902655](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42329902655) | push to `dev`, `9ffa162` | `cargo ci (ubuntu-latest)` (128273787717) | success | `ubuntu-26.04` | rustc 1.99.1 | 242 passed, 0 failed | PASS fmt, clippy, test, policy, book | the same 6 ok |
| 42329902655 | push to `dev`, `9ffa162` | `cargo ci (windows-latest)` (128273787467) | success | `windows-2025-vs2026` | rustc 1.99.1 | 242 passed, 0 failed | PASS fmt, clippy, test, policy, book | the same 6 ok |
| 42329902655 | push to `dev`, `9ffa162` | `cargo deny check` (128273787781) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | — | advisories, bans, licenses, sources ok |
| [42375204698](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42375204698) | push to `main`, `5c3b81b` (merge of #3) | `cargo ci (ubuntu-latest)` (128411255331) | success | `ubuntu-26.04` | rustc 1.99.1 | 242 passed, 0 failed | PASS fmt, clippy, test, policy, book | the same 6 ok |
| 42375204698 | push to `main`, `5c3b81b` | `cargo ci (windows-latest)` (128411255359) | success | `windows-2025-vs2026` | rustc 1.99.1 | 242 passed, 0 failed | PASS fmt, clippy, test, policy, book | the same 6 ok |
| 42375204698 | push to `main`, `5c3b81b` | `cargo deny check` (128411255147) | success | `ubuntu-26.04` | cargo-deny-action v2 | — | — | advisories, bans, licenses, sources ok |

How the columns were read:
- **Tests.** The count is the sum of every `test result:` line in each job's log: 29 test binaries, 242 tests. This equals the 242 tests that Sprint 1's test report recorded locally on Windows and in WSL at `9f138f4`.
- **Other.** The six named tests are the following, each `... ok` in every `cargo ci` leg:
  - `crash_matrix_finds_no_violations`;
  - `matrix_runs_are_reproducible`;
  - `pointer_file_variant_is_caught`;
  - `write_in_place_variant_is_caught`;
  - `fsck_is_clean_on_matrix_stores`;
  - `golden_v1_corpus_matches`.

  `cargo ci` runs `cargo test` without `--nocapture`, so the matrix's per-cell `matrix` summary lines (cell counts, race counts, digest) are not in the GitHub logs. The counts that GitHub confirms are pass or fail of those assertions: 0 violations, races > 0, controls caught. The counts themselves are in Sprint 1's [test-matrix-log.md](../../s1/sprint-tests/test-matrix-log.md).
- **Earlier head.** Runs [42329847300](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42329847300) (push) and [42329860775](https://github.com/crussella0129/Animus_Periapsis/actions/runs/42329860775) (pull request) ran at the earlier head `0fa8aae` (sprint-1 close), and both succeeded.

## Owner sign-offs

None of the following has been given in the conversation. Between the Sprint 1 checkpoint and this record, the owner's only messages were "Merged, continue", "There aren't any open PRs you should be able to continue", "Okay, now it has been merged", and "All merged, continue".
- INT-0001 AC-1 (charter decomposition);
- INT-0002 AC-1 (typed-acceptance grammar);
- INT-0003 AC-5 (design credits);
- INT-0006 AC-6 (format v1's Sprint 0 sections);
- INT-0004 AC-5 (the Windows durability strategy).

Merging PR #3 is recorded as acceptance of the checkpoint only, not as any of these sign-offs. They are carried forward again, in a new backlog task that replaces T-010.
