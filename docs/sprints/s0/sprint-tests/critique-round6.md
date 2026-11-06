# Test Critique — Sprint 0, round 6

## Concerns

### C-001: Dev-dependency licences are never checked, yet the deny doc says nothing is hidden
- **Where:** `crates/xtask/src/deny.rs` module doc (lines 1-4) and `hiding` / `deny.toml` `[licenses]` / `critique-round5.md` Responses, C-007 / `INT-0003` AC-3 / `build-plan.md` T-001 and T-003 Acceptance criterion lines / `unit-tests.md` T-001 deny row / `integration-tests.md` `int_deny_windows`
- **Quote:** "provided the policy hides no crate from it; these tests prove that it hides none"
- **Failure mode:** weak-assertion
- **Why it matters:** cargo-deny 0.20.2 skips dev-dependency licences unless `[licenses] include-dev = true` is set. `deny.toml` does not set it, and `hiding` does not check for it. So no recorded deny run and no CI deny job checks the 13 crates reached only through dev-dependencies (proptest, allocation-counter, rand, rand_core, rand_xorshift, chacha20, getrandom, libc, num-traits, regex-syntax, unarray, autocfg, core_detect) against the allowlist, and the doc's "hides none" is false. On a scratch clone of `94f7363`, an MPL-2.0 dev-dependency gave `licenses ok` and all three deny tests passed. SSPL-1.0 planted on proptest or on allocation-counter also gave `licenses ok`. Yet AC-3 (`necessary`) covers every dependency licence, and T-003 claims AC-3 for proptest. No current licence is off the allowlist: with `include-dev = true` the graph still gives `licenses ok`.
- **Suggested response:** tighten-assertion — set `include-dev = true` under `[licenses]` and make `hiding` name its absence, with a negative case (needs a new tested head); otherwise limit the doc, the round-5 C-007 response and the AC-3 evidence to non-dev dependencies and record why

### C-002: Workflow test accepts push and PR triggers that never fire
- **Where:** `crates/xtask/src/workflow.rs` `problems` (lines 17-27) / `crates/xtask/src/ci.rs` `workflow_matches_ci_contract` / `build-plan.md` T-007 EARS (workflow clause, trigger bullet) / `unit-tests.md` T-007 workflow row
- **Quote:** "`let branches = under(&top, &["on", "push", "branches"])`"
- **Failure mode:** weak-assertion
- **Why it matters:** The test checks the trigger bullet only as a `push.branches` list naming dev and main plus the presence of a `pull_request` key. None of the 19 mutants that the T-007 row cites touches an `on:` filter. On a scratch clone of `94f7363`, `paths-ignore: ['**']` under `push` with `types: [closed]` under `pull_request` passes both `workflow_matches_ci_contract` and `workflow_forms_read_alike`. With that workflow, no push starts a run and a PR runs only after it is closed. Every job-level guard behind INT-0003 AC-2 to AC-4 and INT-0006 AC-3 then stops mattering, and even T-009's post-merge run could look green. Four other filters each pass `workflow_matches_ci_contract`: `branches-ignore`, a `paths` filter, `pull_request` `branches: [nothing]` and `types: [opened]`. `ci.yml` has no filters, so nothing recorded is false.
- **Suggested response:** tighten-assertion — reject `paths`, `paths-ignore`, `branches-ignore` and `types` under `push` and `pull_request`, require any `pull_request` `branches` list to include dev and main, and add those forms as mutants that must fail (needs a new tested head)

### C-003: `unsafe` rule passes a crate root whose attribute is commented out or nested
- **Where:** `crates/xtask/src/policy.rs` `forbid_unsafe` (line 147) and `policy_flags_missing_forbid_unsafe` (lines 732-744) / `build-plan.md` T-001 EARS (`unsafe`) / `unit-tests.md` T-001 `unsafe` row
- **Quote:** "`if path.is_file() && !read(&path).lines().any(|line| line.trim() == FORBID_UNSAFE) {`"
- **Failure mode:** negative-path
- **Why it matters:** The rule accepts the attribute as an exact line anywhere in the file, and the negative cases only delete it outright. On a scratch clone of `94f7363`, `cargo policy` printed `PASS unsafe` (all 11 rules PASS) and exited 0 for two crate roots that lack the attribute in effect: hash, with its line 1 inside a multi-line `/* */`, and codec, with its attribute only inside `mod inner { }`. So the T-001 row records the clause as pass for inputs its own wording covers. The severity is low: INT-0003 AC-1 still holds through the inherited workspace `unsafe_code = "forbid"` lint, which the `lints` rule checks.
- **Suggested response:** add-test — add fixtures with the attribute only inside a multi-line block comment and only inside a nested module, each expecting `fails_only("unsafe", <file>)`, and accept the attribute only among the file's leading lines (needs a new tested head)

### C-004: Deny-job rule still accepts narrowed deny steps
- **Where:** `crates/xtask/src/workflow.rs` `runs_deny_on_ubuntu` (line 110) and `keeps_every_check` (line 133) / `critique-round5.md` Responses, C-003 / `unit-tests.md` T-007 workflow row / `test-plan.md` Intent Traceability, INT-0003 AC-3 row
- **Quote:** "The deny job must now run every check on this workspace"
- **Failure mode:** weak-assertion
- **Why it matters:** On a scratch clone of `94f7363`, `workflow::problems` reported no problem for nine narrowed deny steps:
  - a `log-level` of `warn --exclude winnow` or `warn --exclude-dev`, which cargo-deny-action v2 word-splits into cargo-deny arguments; with winnow's licence planted as SSPL-1.0, the injected `--exclude winnow` turned `licenses FAILED` into `licenses ok`;
  - a flow-mapping `with:`, or a `with:` on the step's first line, neither of which the rule reads;
  - `run: cargo deny check` with a `working-directory` set on the step, the job or the workflow (28 crates checked instead of 42), or with `shell: cat {0}`.

  So the round-5 response and the T-007 row claim more than the rule checks. Nothing is false at `94f7363`, whose deny step is the bare action, and every form that gets through needs a deliberate, non-standard edit.
- **Suggested response:** tighten-assertion — reword the round-5 C-003 response and the T-007 row to say what the rule actually reads (block-form `with:` inputs and plain `run: cargo deny check` lines), and name what it does not check: an inline `with:` or one on the step's first line, `log-level` values, `working-directory`, `defaults` and `shell`; optionally reject those forms in a later tested head

### C-005: Cargo `[env]` above the checkout re-injects what `cargo ci` strips
- **Where:** `code-review.md` Issues raised while verifying the fixes: the known-limit row on `PROPTEST_*` in a cargo `[env]` table and the fixed row on an inherited `CLIPPY_CONF_DIR` / `crates/xtask/src/ci.rs` `cargo_command`, `scrubbed` / `integration-tests.md` `int_ci_ignores_hostile_environment` / `INT-0005` AC-4
- **Quote:** "`PROPTEST_*` variables set in a cargo `[env]` table reach the tests."
- **Failure mode:** flake-risk
- **Why it matters:** The same `[env]` route carries `CLIPPY_CONF_DIR` past the scrub, and no guard catches it. A scratch clone of `94f7363` sat under a parent `.cargo/config.toml` whose `[env]` pointed `CLIPPY_CONF_DIR` at an empty directory. With a planted `HashMap`, it printed `ci: all 5 steps passed` instead of `FAIL clippy`. Yet the records call that scrub fixed and describe the `[env]` limit as affecting only `PROPTEST_*` and failing safe. Setting the variables to inert values would not close the route: a forced `[env]` entry, or a parent `[build] rustflags` with `--cap-lints allow`, lifts the ban too. By this route, `PERI_GOLDEN_CREATE` can only recreate a deleted file with identical bytes, which the spec test backstops. No recorded run is affected, given the no-config precondition that C-006 examines.
- **Suggested response:** defer-with-rationale — widen the `PROPTEST_*` known-limit row to cover any Cargo config above the checkout (forced or unforced `[env]` entries, or keys such as `rustflags`), say that only `PROPTEST_CASES` is guarded, qualify the `CLIPPY_CONF_DIR` row's "fixed", and rely on the recorded no-config precondition rather than offering inert values as a fix

### C-006: No-config check skips the scratch clones' ancestors
- **Where:** `integration-tests.md` header, Environment bullet / `test-matrix-log.md` config-file check (`test_matrix.sh` line 415, `wsl_ci.sh` line 628; log lines 19 and 275) / `e2e-tests.md` `e2e_fresh_clone_windows` / the Windows probes in `integration-tests.md`
- **Quote:** "Neither host has a Cargo config file above the checkouts."
- **Failure mode:** evidence-drift
- **Why it matters:** The logged check covers only the ancestors of the main working checkouts and CARGO_HOME. But `e2e_fresh_clone_windows` and all five Windows probes built under the scratchpad, and the check never looks in its seven ancestors (AppData through scratchpad). On WSL it also skips the extension-less `config` in `/home/.cargo` and `/.cargo`. Nothing recorded is false: none of those directories holds a Cargo config today. The probes' expected fmt, clippy, test, policy and book failures under the same ancestors also rule out a parent `[env]` indirectly. Still, the bullet claims more than the log shows.
- **Suggested response:** tighten-assertion — reword the Environment bullet to name the directories actually checked, or have `test_matrix.sh` and `wsl_ci.sh` check `config` and `config.toml` in every ancestor of each checkout they build in

### C-007: README test cannot tell documented commands from the no-flags sentence
- **Where:** `crates/xtask/src/docs.rs` `readme_keeps_statement_and_links_book` (lines 114-120) and `NO_FLAGS` (line 62) / `build-plan.md` T-008 EARS (README) / `INT-0003` AC-10 / `unit-tests.md` T-008 README row
- **Quote:** "``assert!(readme.contains(command), "README must document `{command}`");``"
- **Failure mode:** weak-assertion
- **Why it matters:** The same test requires `NO_FLAGS` as an exact line, and that line already contains `cargo ci`, `cargo policy` and `cargo book`. The per-command loop therefore adds nothing for those three, and for `cargo deny check` it is a bare substring test. On a scratch clone of `94f7363`, the test still passed for three READMEs:
  - the 'Build and test' section cut to the no-flags sentence plus one mention of `cargo deny check`;
  - the fenced block narrowed to `cargo deny check bans`;
  - a minimal README whose only deny mention is "Never run `cargo deny check`."

  AC-10 is `supporting`, and the tested README does document all four commands. So only regression protection and the T-008 row's credit for "documents the four commands" are weakened.
- **Suggested response:** tighten-assertion — require exact `cargo ci` and `cargo deny check` lines in the fenced blocks and a line other than `NO_FLAGS` that names `cargo policy` and `cargo book`, with those three passing READMEs as cases that must fail (needs a new tested head)

### C-008: Widened T-010 still lets a test check another checkout's code
- **Where:** `e2e-tests.md` Carried forward, T-010 bullet / `critique-round5.md` Responses, C-006 / `code-review.md` known-limit row on xtask's `workspace_root()` and `CARGO_MANIFEST_DIR` / `critique-round3.md` Responses, C-005
- **Quote:** "cannot make `cargo ci`, `cargo policy` or a test check the wrong one"
- **Failure mode:** flake-risk
- **Why it matters:** Two scratch clones of `94f7363` shared one target directory, and clone B changed the bool encoding. Cargo rebuilt only B's `golden` test crate and linked clone A's codec library. All 8 golden tests then passed on B's golden files with A's encoder; with its own target directory, B fails 4 of 8. That binary's compiled-in `CARGO_MANIFEST_DIR` is B's, so T-010's planned check would let it run. xtask's in-process `book::check` has the same gap. No recorded run is affected, since each used its own target directory, but T-010 as scoped cannot deliver its stated purpose.
- **Suggested response:** defer-with-rationale — make T-010 refuse a target directory shared with another checkout (for example, require `cargo metadata`'s `target_directory` under the workspace root), or narrow its purpose clause and name stale linked libraries as an open limit; then align the code-review row and the round-3 C-005 and round-5 C-006 responses

## Confidence
block

## Responses

Round 6 reviewed the records at `48e7bab`, which describe the tested head `94f7363`. Each concern is addressed in the head that the final critique (`critique.md`) reviews. The code and configuration fixes are in `df52b9d`; mutating each fix makes a new or tightened test fail.

| Concern | Response |
|---------|----------|
| C-001 | tighten-assertion in `df52b9d`. `deny.toml` now sets `[licenses] include-dev = true`, so `cargo deny check` reads the licences of the 13 crates reached only through dev-dependencies, and still reports `licenses ok`. `deny_allows_only_gpl3_compatible_licences` fails unless `include-dev` is `true`, and `settings_that_hide_licences_are_named` covers it missing, `false`, and the string `"true"`. The module doc's "hides none" now holds. |
| C-002 | tighten-assertion in `df52b9d`. The workflow check rejects `paths`, `paths-ignore` and `branches-ignore` under `push`; those and `types` under `pull_request`; an inline `pull_request`; and a `pull_request` branch list that leaves out dev or main. `workflow_matches_ci_contract` adds the critique's forms and more as mutants that must fail, and a `pull_request` limited to dev and main as a form that must pass. |
| C-003 | add-test in `df52b9d`. The `unsafe` rule now counts `#![forbid(unsafe_code)]` only among the crate root's leading lines, where blank lines, line comments and one-line inner attributes may come first. `policy_flags_missing_forbid_unsafe` adds the attribute inside a block comment and inside a nested module, each failing `unsafe`, and after a doc comment and another attribute, passing. |
| C-004 | tighten-assertion in `df52b9d`, beyond the rewording asked for. The rule now rejects an inline `with:` or one on the step's first line, a `log-level` that is not one of cargo-deny's five levels, `working-directory` or `shell` on the step, and `defaults` on the deny job or the workflow. Each of the critique's narrowed forms is a mutant that must fail. |
| C-005 | defer-with-rationale. The known-limit row now covers any Cargo configuration above a checkout: an `[env]` entry, forced or not, can set `PROPTEST_*`, `CLIPPY_CONF_DIR` or `PERI_GOLDEN_CREATE`, and keys such as `build.rustflags` can lift the HashMap ban; only `PROPTEST_CASES` fails closed. The `CLIPPY_CONF_DIR` row now says it is fixed for the caller's environment only. No recorded run sits under such a configuration, which the matrix now checks for every checkout (C-006). |
| C-006 | tighten-assertion. Both matrix scripts now print, for every checkout they build in, the `.cargo/config` and `.cargo/config.toml` files in every directory above it and in `CARGO_HOME`. Every checkout prints none, and the Environment bullet says what was checked. |
| C-007 | tighten-assertion in `df52b9d`. The README check now needs `cargo ci` and `cargo deny check` as command lines in fenced blocks, and a line besides the no-flags sentence that says what `cargo policy` and `cargo book` do. `readme_must_document_each_command` shows that the critique's three READMEs, and one with `cargo ci --fast`, fail. |
| C-008 | defer-with-rationale, with T-010 redefined so that it can deliver its purpose. `cargo ci` will refuse a target directory outside the workspace root, as `cargo metadata` reports it, and xtask will refuse to run for a checkout it was not built for. (Narrowed after the round-7 critique, C-007: neither check stops another checkout's cargo command from building into this checkout's in-tree `target/`, which only one target directory per checkout prevents.) Until then, the known limit names stale libraries linked from another checkout. The code-review row, the Carried forward section and the round-3 and round-5 responses say the same. |
