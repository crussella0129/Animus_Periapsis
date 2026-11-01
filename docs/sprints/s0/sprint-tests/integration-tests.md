# Sprint 0 Integration Test Results

- **Tested head:** `24d01f55cad930f8430289996efe84eea56dfce0` on `dev`, committed. `git status --porcelain` printed nothing in the Windows working checkout before and after the runs; the WSL runs check their own clone. The head includes the fixes from the Test-phase [code review](code-review.md) and from the test critiques. Later commits change only the Book.
- **Canonical runner:** `cargo ci` (steps fmt, clippy, test, policy, book). `cargo deny check` (cargo-deny 0.20.2) runs beside it.
- **Raw output:** [test-matrix-log.md](test-matrix-log.md). It holds the final run, which produced every result below except the two task-boundary deny runs, and the scripts that ran it.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, installed from `rust-toolchain.toml` on each host. Git 2.56.0.windows.3 on Windows, Git 2.53.0 in WSL.
- **Environment:** before running, each host printed which of `PROPTEST_*`, `PERI_*`, `CLIPPY_CONF_DIR`, `CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR` and `GIT_*` it sets. WSL sets none. Windows sets only `GIT_EDITOR`, which the session sets; the golden rule removes it like any other `GIT_*` variable. Neither host has a Cargo config file above the checkouts. `int_ci_ignores_hostile_environment` and `int_policy_reports_failures` set such variables on purpose, to show that the tools ignore them.
- **Target directories:** every checkout builds into its own `target/`, and each run prints its target directory. No run shares one with another checkout, which [code-review.md](code-review.md) records as a known limit.

| Host | OS release | `rustc -Vv` host | Checkout |
|------|------------|------------------|----------|
| Windows | Microsoft Windows 11 Pro 10.0.26300 | `x86_64-pc-windows-msvc` | the working checkout `C:\Users\charl\Animus_Periapsis` (NTFS); scratch clones for the probes |
| WSL 2 | Ubuntu 26.04.1 LTS, kernel 6.6.114.1-microsoft-standard-WSL2 | `x86_64-unknown-linux-gnu` | a fresh `git clone --branch dev` into `$HOME/periapsis-s0-test` (ext4, not `/mnt/c`); a clone of that for the failure, Book and policy probes |

## Results

### `int_deny_at_t001`, `int_deny_at_t003`
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-3
- **Run:** `cargo deny check` on the Windows host at each task's boundary (`f89c741`, `0fdb12a`).
- **Observed:** "advisories ok, bans ok, licenses ok, sources ok" both times, as recorded in the T-001 and T-003 entries of [completed-tasks.md](../../../work/completed-tasks.md).
- **Advisory database:** both runs used RustSec advisory-db revision `2295466`, according to the database's reflog. That revision was fetched at 17:03:12, before the T-001 commit at 17:03:52, and reused at 17:06:43, before the T-003 commit at 17:06:58.
- **Result:** pass

### `int_ci_windows`
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-4 (local half), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1, [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3 (local half), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) AC-2
- **Run:** `cargo ci` in the Windows working checkout.
- **Observed:** `PASS fmt`, `PASS clippy`, `PASS test`, `PASS policy`, `PASS book`, then `ci: all 5 steps passed`; exit 0. A separate `cargo test --workspace --locked` in the same checkout listed 151 tests passed ([unit-tests.md](unit-tests.md)).
- **Result:** pass

### `int_ci_reports_failures` (added in the Test phase)
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-4, [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3: the suite fails when an encoding changes. This is the end-to-end form of the T-007 clause "two steps fail → every step runs, non-zero exit, both named".
- **Run:**
  1. Clone the tested head into a scratch directory: on Windows from the working checkout, in WSL from the ext4 clone.
  2. Overwrite one byte of `crates/codec/tests/golden/v1/string-ascii.bin`.
  3. Append the unformatted line `pub fn  badly_formatted( ) {}` to `crates/codec/src/lib.rs`.
  4. Run `cargo ci`, then delete the clone.
- **Observed on both systems:** `FAIL fmt: cargo fmt --all --check exited with …1`, `PASS clippy`, `FAIL test: cargo test --workspace --locked exited with …101`, `PASS policy`, `PASS book`, then `ci: failed: fmt, test`; exit 1. The test step's output shows `test golden_v1_corpus_matches ... FAILED` and `string-ascii.bin: encoding changed (file 10 bytes, encoder 10 bytes)`.
- **Result:** pass. Every step ran, the run exited non-zero, the summary named both failures, and the golden test itself reported the changed file. This exercises `ci::run`, `finish`, the `ci` arm of `cli` and `main`'s exit status.

### `int_ci_reports_book_violation` (added after the round-4 test critique)
- **Intent:** [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) AC-2: a Book violation fails `cargo ci`, in its `book` step and in its test step. It has its own run because a failing Book test stops `cargo test` before the golden test would run.
- **Run:** on Windows and in WSL:
  1. Clone the tested head into a scratch directory. In WSL the failure probe's clone is reused, restored first with `git checkout -- .`.
  2. Insert the untyped criterion `- An untyped probe criterion.` after the criteria heading of `docs/intents/INT-0031-visual-identity.md`.
  3. Run `cargo ci`, then delete the clone.
- **Observed on both systems:** `PASS fmt`, `PASS clippy`, `FAIL test: cargo test --workspace --locked exited with …101`, `PASS policy`, `` FAIL book: 1 Book violations; run `cargo book` ``, then `ci: failed: test, book`; exit 1. The test step's output shows `test book_intents_pass_strict ... FAILED` and the violation, ending `docs/intents/INT-0031-visual-identity.md:25: untyped: criterion lacks …`.
- **Result:** pass. The Book step's violation arm runs end to end, and the test step reports the same violation at its line.

### `int_ci_ignores_hostile_environment` (added in the Test phase)
- **Intents:** [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3, [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-4; T-007 clause "the `test` step removes `PERI_GOLDEN_CREATE`"
- **Run:** on Windows:
  1. Clone the tested head into a scratch directory.
  2. Append a function using `std::collections::HashMap` to `crates/codec/src/lib.rs` and delete `crates/codec/tests/golden/v1/uvarint-0.bin`.
  3. Run `PROPTEST_CASES=1 PERI_GOLDEN_CREATE=1 CLIPPY_CONF_DIR=<an empty directory> cargo ci`.
  4. Delete the clone.
- **Observed:**
  - `PASS fmt`, `FAIL clippy` ("use of a disallowed type `std::collections::HashMap`"), `FAIL test`, `PASS policy`, `PASS book`, then `ci: failed: clippy, test`; exit 1.
  - In the test output, `case_count_is_not_overridden ... ok` (the book crate's guard, the first to run) and `golden_v1_corpus_matches ... FAILED`. `cargo test` stops at the first failing test binary, so the later guards did not run.
  - `uvarint-0.bin` was not recreated.
- **Result:** pass. Had any of the three variables reached a step, it would have shown:
  - `CLIPPY_CONF_DIR` would have let the `HashMap` through Clippy;
  - `PERI_GOLDEN_CREATE` would have recreated the file and passed the test step;
  - `PROPTEST_CASES` would have failed the guard.

### `int_policy_reports_failures` (added after the round-3 test critique)
- **Intents:** [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3, [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1; the T-005 clause "`cargo policy` **SHALL** fail with the new rule `golden`"
- **Run:** on Windows and in WSL:
  1. Clone the tested head into a scratch directory. In WSL the failure probe's clone is reused, restored first with `git checkout -- .`.
  2. Empty `.gitattributes`, so nothing marks `*.bin` binary.
  3. Run `cargo policy`.
  4. Write a file holding `*.bin binary`. Set `GIT_CONFIG_PARAMETERS`, and `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_0` and `GIT_CONFIG_VALUE_0`, so that both forms point `core.attributesFile` at it. As a control, run a plain `git check-attr text` on a golden file without and with these variables. Then run `cargo policy` again with them.
  5. Delete the clone.
- **Observed on both systems:**
  - Both `cargo policy` runs printed `PASS` for the ten other rules and `` FAIL golden: 24 *.bin paths are not binary: the .gitattributes files give crates/codec/tests/golden/v1/blob-envelope.bin `text: unspecified` ``, and exited 1.
  - The control printed `text: unspecified` without the variables and `text: unset` with them.
- **Result:** pass. `cargo policy` fails naming `golden` and exits 1. Git variables that change a plain `git`'s answer do not reach the rule.

### `int_deny_windows`
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-3 (local half)
- **Run:** `cargo deny check` on the Windows host at the tested head.
- **Observed:** "advisories ok, bans ok, licenses ok, sources ok"; exit 0. The licences `deny.toml` allows are themselves checked against a GPL-3.0-compatible list by `deny_allows_only_gpl3_compatible_licences` ([unit-tests.md](unit-tests.md), T-001). The RustSec advisory database was `advisory-db-3157b0e258782691` at `2295466c4e9df26647fd67d8fa1c705ac2fc7f81` (2026-10-30).
- **Result:** pass

### `int_ci_wsl`
- **Intents:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-4 (local half), [INT-0004](../../../intents/INT-0004-windows-and-linux.md) AC-1, [INT-0006](../../../intents/INT-0006-format-v1.md) AC-3 (local half)
- **Run:** clone the committed `dev` head into the WSL ext4 home directory, install the pinned toolchain, run `cargo ci`, then list the tests with `cargo test --workspace --locked`.
- **Observed:**
  - The clone's head is `24d01f5` and `df` reports ext4.
  - `cargo ci` printed `PASS` for all five steps, then `ci: all 5 steps passed`; exit 0.
  - 151 tests passed on Linux, the same count as on Windows.
  - The golden test ran inside the `test` step, so the 23 `.bin` files matched the Linux encoder byte for byte.
  - `PASS policy` includes the golden rule: Git, run against the rule's private repository in this fresh clone, gave all 23 `.bin` files and the `probe.bin` path `text: unset` and no filter, `ident` or working-tree encoding.
- **Result:** pass

### `int_tooling_is_idempotent`
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-9
- **Run:** in the same WSL clone, `cargo ci`, `cargo policy` and `cargo book` twice each, with no arguments; `git status --porcelain` after every run.

| Run | `cargo ci` | `cargo policy` | `cargo book` | Uncommitted paths after each |
|-----|------------|----------------|--------------|------------------------------|
| 1 | exit 0 | exit 0 | exit 0, 31 counted chapter lines | 0, 0, 0 |
| 2 | exit 0 | exit 0 | exit 0, 31 counted chapter lines | 0, 0, 0 |

- **Observed:** `docs/intents` holds 31 `INT-*.md` chapters, equal to the counted lines.
- **Result:** pass

### `int_cc_build_dependency_audit`
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md) AC-6
- **Run:** on the Windows host, `cargo tree --workspace --target all -e normal,build -i cc --depth 1` and `cargo tree --workspace -e features -i blake3`.
- **Observed:**
  - `cc v1.10.0` has exactly one direct dependent, `blake3 v1.8.7`, as a build dependency.
  - The dependency table in [prior-art.md](../../../prior-art.md) has exactly one C-build `yes` row, `blake3`. The rows added in the Test phase, `pulldown-cmark`, `toml_edit` and `allocation-counter`, are all `no`.
  - The feature tree shows `blake3 feature "pure"` and `"std"`.
  - The tree does not change when `pure` is dropped, because `cc` stays a build dependency of `blake3`. That case is caught by `cargo policy`'s `credits` rule (`policy_requires_complete_dependency_rows`).
- **Result:** pass

### `int_disallowed_types_probe`
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) AC-4
- **Run:** on the Windows host, twice, once per type:
  1. Clone the tested head into a scratch directory.
  2. Append a function to `crates/codec/src/lib.rs` that builds a `std::collections::HashMap::<u8, u8>` in the first clone, and a `std::collections::HashSet::<u8>` in the second.
  3. Run `cargo clippy --workspace --all-targets -- -D warnings`, building into the clone's own target directory.
  4. Delete the clone.
- **Observed:**
  - HashMap clone (at `24d01f5`, 1 file changed): "error: use of a disallowed type `std::collections::HashMap`", exit 101.
  - HashSet clone (at `24d01f5`, 1 file changed): "error: use of a disallowed type `std::collections::HashSet`", exit 101.
  - Both clones were removed.
- **Result:** pass
