# Test Critique — Sprint 0, round 7

## Concerns

### C-001: Cargo aliases behind INT-0003 AC-2 and AC-9 are untested
- **Where:** `.cargo/config.toml` `[alias]` / `test-plan.md` Intent Traceability, INT-0003 AC-2 and AC-9 rows / `build-plan.md` T-001 and T-007 Acceptance criterion lines / `unit-tests.md` T-001 and T-007 argument rows / `INT-0003` AC-2, AC-9
- **Quote:** "T-001 / `toolchain`, `lockfile` rules; T-007 / clippy and test run `--locked`"
- **Failure mode:** intent-coverage
- **Why it matters:** Every `cargo ci`, `cargo policy` and `cargo book` run goes through these aliases, including T-009's run. Yet nothing under `crates/` reads `.cargo/config.toml`, and the AC-9 tests call `cli()` or the `book` binary directly. On a scratch clone of `df52b9d` with a stale `Cargo.lock`, the committed `ci` alias exits 101. With `--locked` removed from that alias, `cargo run` rewrote the lock and `cargo ci` printed `ci: all 5 steps passed` (exit 0). With `--locked` removed from `policy`, a deleted lock was regenerated and `lockfile` passed. With `book`'s trailing `--` removed, `cargo book --strict` exited 1 instead of 2. So the "locked" half of `necessary` AC-2 rests on no test, no run and no recorded inspection at the tested head, and the tests, the matrix and T-009 would all look the same without it.
- **Suggested response:** add-test — add an xtask test or `policy` rule that parses `[alias]` and requires the three strings exactly, with mutants that drop `--locked` or the trailing `--` failing (needs a new tested head); optionally add the flag runs (exit 2) and a stale-lock run (exit 101) to the matrix

### C-002: YAML continuation lines let narrowed steps pass the workflow rule
- **Where:** `crates/xtask/src/workflow.rs` `runs_deny_on_ubuntu` and `keeps_every_check` (lines 143-184), `item` (lines 222-226) / `unit-tests.md` T-007 workflow row / `critique-round6.md` Responses, C-004 / `INT-0003` AC-3
- **Quote:** "`// Inputs are read only as a block under the step.`"
- **Failure mode:** weak-assertion
- **Why it matters:** The rule treats a more-indented line as a child and reads each value from its own line only. YAML instead folds a plain continuation line into the value above it.
  - On a scratch clone of `df52b9d`, `workflow::problems` accepted five wrapped forms. PyYAML 6.0.3 and Go yaml.v3 read them as `run: cargo deny check sources`, `command: check bans`, `log-level: warn --exclude-dev`, `arguments: --all-features --exclude winnow` and `run: cargo ci ; exit 0`.
  - The rule rejects each of these when it is written on one line.
  - The last form exits 0 under an emulation of the windows-latest pwsh wrapper.
  - A folded `log-level: warn --exclude winnow` also passes. That is round 6's own `licenses ok` attack.

  This rule is the sprint's only check of what the deny job runs for `necessary` AC-3. So the T-007 row's "its narrowed forms must not count" and the round-6 `log-level` claim are false for valid YAML, and a green deny job or Windows leg in T-009 could be a false pass. `ci.yml` at `df52b9d` uses none of these forms.
- **Suggested response:** tighten-assertion — reject any child line under a step entry or a `with:` input, and add the five wrapped forms and the folded `--exclude winnow` as mutants that must fail (needs a new tested head)

### C-003: Declaration and create-mode checks never try empty or `1`-prefixed values
- **Where:** `build-plan.md` T-006 EARS (marker clause) and T-005 EARS (`create_mode`) / `crates/book/src/lib.rs` `rule_from_marker` / `crates/book/tests/book.rs` `unsupported_declaration_is_an_error`, `repeated_declaration_is_an_error` / `crates/codec/tests/golden.rs` `create_mode_is_exactly_one` / `unit-tests.md` T-006 marker row and T-005 create-mode row / `INT-0002` AC-2
- **Quote:** "the marker declares `typed-acceptance:` with any value other than `1`"
- **Failure mode:** negative-path
- **Why it matters:** The error path is tried only with `2` and `01`. On a scratch clone of `df52b9d`, four `rule_from_marker` mutants pass all 37 book tests:
  - two that read an empty value as the port rule, directly or through a "blank means unset" filter, which the clause forbids by name;
  - a `1`-prefixed match and a semver-style match, which read `10` or `1.1` as version 1.

  `create_mode` has the same gap: `starts_with('1')`, `trim_end` and `parse::<u8>() == 1` all pass the 8 golden tests, although there a looser match cannot produce a false pass of AC-3. The code at `df52b9d` is correct, so nothing recorded is false.
- **Suggested response:** add-test — add an empty declaration, a whitespace-only one, `10` and `1.1` to `unsupported_declaration_is_an_error`, and add `10`, `1 ` and `01` to `create_mode_is_exactly_one`, so that every mutant above fails (needs a new tested head)

### C-004: `unsafe` rule passes a forbid line inside a comment or string
- **Where:** `crates/xtask/src/policy.rs` `opens_with_forbid_unsafe` (lines 155-169) / `unit-tests.md` T-001 `unsafe` row / `build-plan.md` T-001 EARS (`unsafe`) / `critique-round6.md` Responses, C-003 / `INT-0003` AC-1
- **Quote:** "inside a block comment or a nested module it fails `unsafe`"
- **Failure mode:** negative-path
- **Why it matters:** The rule skips any leading line that starts with `#![` without checking where the attribute ends. On a scratch clone of `df52b9d`, three hash crate roots passed even though none has the attribute in effect:
  - `#![allow(dead_code)] /*`, then the forbid line, then `*/`, which passed fmt, `cargo policy` (`PASS unsafe`) and the full `cargo ci`;
  - `#![doc = "` with the forbid line inside the string, which printed `PASS unsafe`;
  - `#![doc = "]` with the forbid line inside the string, which also printed `PASS unsafe`.

  So the T-001 row and the round-6 "one-line inner attributes" response credit a negative path that the code does not enforce. Severity is low, because AC-1 still holds through the inherited workspace `unsafe_code = "forbid"`.
- **Suggested response:** add-test — fail `unsafe` when a leading `#![` line contains `"`, `/*` or `{`, or require the attribute as the first non-blank line, and add all three layouts as `fails_only("unsafe", ...)` fixtures (needs a new tested head); a check that the attribute closes on its own line is not enough, because it misses `#![doc = "]`

### C-005: Caller's RUSTFLAGS lifts the HashMap ban, and the WSL log cannot rule it out
- **Where:** `crates/xtask/src/ci.rs` `scrubbed` / `code-review.md` the `CLIPPY_CONF_DIR` row and the Cargo-configuration known-limit row / `integration-tests.md` header Environment bullet and `int_ci_wsl` / `test-matrix-log.md` environment lines of `test_matrix.sh` and `wsl_ci.sh` / `INT-0003` AC-4 (local Linux half), `INT-0005` AC-4
- **Quote:** "fixed `9672c26` for the caller's environment: `cargo ci` strips it, with `PERI_GOLDEN_CREATE` and `PROPTEST_*`"
- **Failure mode:** flake-risk
- **Why it matters:** `scrubbed` removes only `PERI_GOLDEN_CREATE`, `PROPTEST_*` and `CLIPPY_CONF_DIR`, so `RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS` reach every step. On a scratch clone of `df52b9d` with the matrix's planted `HashMap`, Clippy fails without these variables. With `RUSTFLAGS='-A clippy::disallowed_types'` or `--cap-lints=allow`, `cargo ci` printed all five PASS lines and exited 0. Neither matrix script prints these variables, no known-limit row names this route, and the HashMap probes ran only on Windows. So the log cannot tell the WSL `PASS clippy` from a lifted ban. The WSL login shell sets none of these variables today, but that was checked outside the log.
- **Suggested response:** tighten-assertion — print `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, `RUSTC_WRAPPER` and `RUSTC_WORKSPACE_WRAPPER` in both hosts' environment lines and repeat the HashMap probe on WSL (a new matrix run, no new tested head), and either name this route beside the configuration known limit or strip it for the clippy step

### C-006: Workflow rule accepts a ci job whose legs never run the suite on the commit
- **Where:** `crates/xtask/src/workflow.rs` `runs_ci_on_both_platforms` (lines 118-135) / `crates/xtask/src/ci.rs` `workflow_matches_ci_contract` / `build-plan.md` T-007 EARS (workflow clause, matrix bullet) / `unit-tests.md` T-007 workflow row / `critique-round6.md` Responses, C-004
- **Quote:** "`entries.contains(&("run", "cargo ci")) && !entries.iter().any(|&(key, _)| key == "if")`"
- **Failure mode:** weak-assertion
- **Why it matters:** The round-6 C-004 fix rejects `shell`, `working-directory` and job-level `defaults` only for the deny job. On a scratch clone of `df52b9d`, `workflow::problems` accepted all of these:
  - `shell: cat {0}` or `working-directory` on the `cargo ci` step;
  - `defaults` or a `container` on the ci job;
  - a checkout `with: ref:` in either job;
  - an earlier step that checks out another commit or writes `$GITHUB_ENV`;
  - `env` at any level, including `CARGO_ALIAS_CI: version` (`cargo ci` becomes `cargo version` and exits 0) and `RUSTFLAGS: --cap-lints=allow` (Clippy then passes a planted `HashMap`).

  `ci.yml` at `df52b9d` uses none of these, and the in-scope ACs rest on the local runs. But until T-009 this rule is Sprint 0's only check of the matrix job, and adding forms one at a time cannot close the gap.
- **Suggested response:** defer-with-rationale — record what the ci-job rule does not read (`shell`, `working-directory`, job `defaults`, `env`, `container`, checkout inputs, earlier steps), and have T-009 confirm that `ci.yml` is unchanged from `df52b9d` and that every leg printed all five PASS lines; an exact-steps rule for both jobs can follow in a later head

### C-007: Redefined T-010 cannot stop reuse through this checkout's own target
- **Where:** `critique-round6.md` Responses, C-008 / `e2e-tests.md` Carried forward, T-010 bullet / `code-review.md` known-limit row on xtask's `workspace_root()` / `critique-round3.md` Responses, C-005 / `critique-round5.md` Responses, C-006 / `integration-tests.md` Target directories bullet
- **Quote:** "which is the only way another checkout can share one"
- **Failure mode:** flake-risk
- **Why it matters:** Both planned T-010 checks guard only the checkout that runs `cargo ci`. Another checkout's plain cargo command can still build into this checkout's in-tree `target/`. On two fresh clones of `df52b9d`:
  - clone X had a broken bool encoder;
  - clone Y ran `cargo test` with `CARGO_TARGET_DIR` set to X's `target/`;
  - X's `cargo ci` then printed `ci: all 5 steps passed`, because its test step reused Y's codec library and golden binary;
  - with a fresh target, X fails 4 of the 8 golden tests;
  - X's target sits inside its root and its xtask is its own, so neither planned T-010 check would fire.

  So T-010 cannot deliver "no binary or library built for another checkout can be reused". The round-3 C-005 and round-5 C-006 responses also still describe the test-side check that was dropped. No recorded result depends on this, because the cold-clone and WSL runs used fresh target directories.
- **Suggested response:** defer-with-rationale — narrow the T-010 purpose clause, the code-review row and the round-6 C-008 response to what the checks cover (a checkout's own `cargo ci` refusing a target directory outside its root), name the reverse case as an open limit covered only by the one-target-per-checkout rule, and align the round-3 and round-5 responses and the `integration-tests.md` Target directories bullet (records only)

### C-008: Branch lists with negative patterns pass the trigger check
- **Where:** `crates/xtask/src/workflow.rs` `problems` (lines 17-24 and 49-56) / `build-plan.md` T-007 EARS (trigger bullet) / `unit-tests.md` T-007 workflow row / `critique-round6.md` Responses, C-002
- **Quote:** "No trigger may be filtered (`paths`, `paths-ignore`, `branches-ignore`, `types`, an inline `pull_request`, a `pull_request` branch list without dev and main)"
- **Failure mode:** weak-assertion
- **Why it matters:** The branch checks only ask whether `dev` and `main` appear in the list. GitHub applies branch patterns in order, so a later `!` pattern excludes a ref that an earlier one matched. On a scratch copy of the `df52b9d` rule, `push: branches: [dev, main, '!dev', '!main']` and `pull_request: branches: [dev, main, '!**']` both passed. A workflow that never runs for dev, main or any pull request therefore meets the trigger bullet. `ci.yml` has no `!` pattern, so nothing recorded is false.
- **Suggested response:** tighten-assertion — reject any `!`-prefixed entry in both branch lists and add the two forms as mutants that must fail (needs a new tested head); otherwise reword the T-007 row to say that `!` patterns are not evaluated

### C-009: Deny tests check three named hiding settings, not every one
- **Where:** `crates/xtask/src/deny.rs` module doc (lines 1-5) and `hiding` (lines 78-98) / `deny.toml` `[graph]`, `[licenses]`, `[licenses.private]` / `critique-round6.md` Responses, C-001 / `unit-tests.md` T-001 deny row / `INT-0003` AC-3
- **Quote:** "these tests prove that it hides none, dev-dependencies included"
- **Failure mode:** weak-assertion
- **Why it matters:** On a scratch clone of `df52b9d` with cargo-deny 0.20.2, each of three settings turned a planted SSPL-1.0 licence into `licenses ok` while all three deny tests still passed:
  - `[graph] exclude-dev = true`, which undoes the round-6 C-001 fix;
  - `[licenses] include-build = false`;
  - `[licenses.private] ignore-sources` naming the crates.io index.

  So `hiding` checks named settings; it does not prove that nothing is hidden. The delta critic's other two points (unpublished path crates and `[graph] targets`) leave out no crate that is built on Windows or Linux.
- **Suggested response:** reject (the critique is wrong because `deny.toml` at `df52b9d` hides no built dependency's licence, and no AC or EARS clause asks the deny tests to guard against a future `deny.toml` edit, as round 5 settled for C-007) — optionally reword the module doc to say that `hiding` checks the settings it names

## Confidence
block

## Responses

Round 7 reviewed the records at `c09af33`, which describe the tested head `df52b9d`. Each concern is addressed in the head that the final critique (`critique.md`) reviews. The fixes are in `d7b0484`; mutating each fix makes a new or tightened test fail. Before the next round, a sweep of the whole suite found 108 more mutants that no test caught, fixed in `0a9fd40` ([mutation-sweep.md](mutation-sweep.md)).

| Concern | Response |
|---------|----------|
| C-001 | add-test in `d7b0484`. `cargo_aliases_are_the_reviewed_ones` holds the reviewed `.cargo/config.toml`, line endings aside, so dropping `--locked` from any alias, or the `book` alias's trailing `--`, fails a test. The matrix adds `int_aliases_refuse_flags_and_stale_locks` on Windows and WSL: `cargo ci --fast`, `cargo policy --fix` and `cargo book --strict` each exit 2, and with a stale `Cargo.lock`, `cargo ci` exits 101 and leaves the lock as it was. |
| C-002 | add-test in `d7b0484`, beyond the tightening asked for. `ci_workflow_is_the_reviewed_one` holds the reviewed `ci.yml`, line endings aside, so any edit to it fails a test until the edit is reviewed and the pin updated with it: the five wrapped forms, a folded `--exclude`, and anything else. The workflow rule stays as the readable statement of the contract; the pin is what fails closed. |
| C-003 | add-test in `d7b0484`. `unsupported_declaration_is_an_error` adds an empty value, whitespace only, `10`, `1.1` and `01` to `2`, and `create_mode_is_exactly_one` adds `1 `, `01` and `10` (and `2` in `0a9fd40`). Each of the critique's mutants fails a test. |
| C-004 | add-test in `d7b0484`. The `unsafe` rule now requires the crate root's first line that is neither blank nor a `//` comment to be exactly `#![forbid(unsafe_code)]`. `policy_flags_missing_forbid_unsafe` adds the critique's three layouts, each failing `unsafe`. |
| C-005 | tighten-assertion in `d7b0484`. Besides `PERI_GOLDEN_CREATE`, `PROPTEST_*` and `CLIPPY_CONF_DIR`, `cargo ci` now removes from every step `RUSTFLAGS`, `RUSTDOCFLAGS`, `RUSTC`, `RUSTDOC`, `RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER` and `RUSTUP_TOOLCHAIN`, and every variable whose name starts, in any letter case, with `CARGO_BUILD_`, `CARGO_TARGET_`, `CARGO_PROFILE_`, `CARGO_ENCODED_` or `CARGO_ALIAS_`. `test_step_removes_golden_create` and `proptest_variables_are_never_inherited` check the lists; `hostile_variables_never_reach_a_step` runs a real `cargo ci` with such variables set. Both matrix scripts print these variables, and the HashMap and HashSet probe now runs on WSL too. A Cargo configuration file above a checkout remains the known limit in [code-review.md](code-review.md). |
| C-006 | add-test in `d7b0484`, in place of the deferral suggested. The `ci.yml` pin (C-002) fails on each of the critique's forms: `shell`, `working-directory`, `defaults`, `container`, checkout inputs, an earlier step, and `env` at any level. |
| C-007 | defer-with-rationale (records only). T-010 is narrowed to what its checks cover: a checkout's own `cargo ci` refusing a target directory outside its workspace root, and xtask refusing to run for a checkout it was not built for. The reverse case, another checkout's plain cargo command building into this checkout's in-tree `target/`, is an open limit that only the rule of one target directory per checkout covers, and every recorded run follows it (the matrix prints each one). The code-review row, the Carried forward bullet, the Target directories bullet and the round-3 C-005, round-5 C-006 and round-6 C-008 responses now say the same. |
| C-008 | tighten-assertion in `d7b0484`. The workflow rule rejects any `!` entry in either branch list, and `workflow_matches_ci_contract` adds the critique's two forms as mutants that must fail. The `ci.yml` pin fails on them too. |
| C-009 | add-test in `d7b0484`, in place of the rejection suggested. `deny_policy_is_the_reviewed_one` holds the reviewed `deny.toml`, line endings aside, so the critique's three settings, and any other edit, fail a test. In `0a9fd40`, the module doc of `deny.rs` says that its tests check the settings they name, and that the pin is what keeps any other setting from being added unseen. |
