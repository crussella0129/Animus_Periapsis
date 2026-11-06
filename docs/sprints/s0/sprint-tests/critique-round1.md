# Test Critique — Sprint 0, round 1

## Concerns

### C-001: deps and credits rules skip target-specific and dotted dependency tables
- **Where:** `crates/xtask/src/policy.rs` `deps` and `credits` rules / tests `policy_flags_direct_dependency`, `policy_flags_uncredited_dependency` / `build-plan.md` T-001 EARS (`deps`, `credits`) / `unit-tests.md` T-001 rows / `INT-0003` AC-6
- **Quote:** "`let kind = table.split('.').next().unwrap_or_default();`"
- **Failure mode:** negative-path
- **Why it matters:** A probe of the committed rules showed two gaps. `[target.'cfg(windows)'.dependencies] foo = "1"` passes, along with its dev and build forms, because the header's first segment is `target`. An uncredited `[workspace.dependencies.foo]` table also passes, because `credits` reads only the exact `[workspace.dependencies]` header. So two in-scope EARS clauses are recorded as pass while false for inputs within their own wording, and a Windows-and-Linux project is likely to add platform-specific dependencies.
- **Suggested response:** add-test — add fixtures with target-specific normal, dev and build dependencies (expect FAIL `deps` naming crate and dependency) and an uncredited `[workspace.dependencies.foo]` (expect FAIL `credits`). Then fix both rules to parse quoted header keys, or use a TOML parser, because cfg strings can contain dots.

### C-002: clippy rule passes when the hash paths sit outside `disallowed-types`
- **Where:** `crates/xtask/src/policy.rs` `clippy` rule / test `policy_requires_disallowed_hash_collections` / `build-plan.md` T-001 EARS (`clippy`) / `unit-tests.md` T-001 row / `INT-0005` AC-4
- **Quote:** "`let has_list = text.contains("disallowed-types");`"
- **Failure mode:** negative-path
- **Why it matters:** The rule substring-matches the whole of `clippy.toml`, comments included. Three broken files each report PASS `clippy`: `disallowed-types = []` with both paths under `disallowed-methods`, the key named only in a comment, and both entries commented out inside the array. With any of them, Clippy accepts `HashMap` and exits 0, so `cargo ci` stays green after INT-0005 AC-4's guard is gone. Every fixture contains the substring, so `has_list` is never false in any test.
- **Suggested response:** add-test — add those three fixtures and one with no `disallowed-types` at all, each expecting FAIL `clippy`, and make the rule strip comments and read the entries of the `disallowed-types` array.

### C-003: The real `cargo ci` failure path never runs because the executor is stubbed
- **Where:** `crates/xtask/src/ci.rs` `ci_failures_do_not_stop_later_steps` / `unit-tests.md` header "Stubs: none." and T-007 row 2 / `build-plan.md` T-007 EARS ("including because a tool is not installed")
- **Quote:** "two failing steps, one a missing tool → every step runs, non-zero exit, both named"
- **Failure mode:** stub-leak
- **Why it matters:** The closure given to `run_with` returns ready-made `Outcome::Fail` strings by step name, modeled on `execute()`'s messages. So `Action::Cargo(&["no-such-tool"])` never runs, and a mutant `execute()` that reports a non-zero exit as Pass still passes all 24 xtask tests, while every recorded `cargo ci` run shows all PASS. The PASS evidence for INT-0003 AC-4, INT-0004 AC-1 and the local half of INT-0006 AC-3 therefore has no negative control, and the record's "Stubs: none" is false.
- **Suggested response:** add-test — run the real `execute()` through `run_with` with a missing-tool step (`Cargo(&["no-such-tool"])`) and a second real failure among passing steps. Assert both FAIL lines, that every step ran and that `finish()` returns 1, then correct the Stubs line.

### C-004: The `cargo ci` step test does not pin the step commands
- **Where:** `crates/xtask/src/ci.rs` `ci_steps_are_ordered_and_locked` / `build-plan.md` T-007 EARS 1 / `unit-tests.md` T-007 row 1
- **Quote:** "`assert_eq!(names, ["fmt", "clippy", "test", "policy", "book"]);`"
- **Failure mode:** weak-assertion
- **Why it matters:** The EARS fixes each step's exact command, but the test checks only the names, `--locked` and `--workspace` (via `contains()`), and clippy's `-- -D warnings` tail. Each of these changes would leave this test and every recorded all-PASS run green:
  - dropping fmt's `--check`;
  - dropping clippy's `--all-targets`;
  - adding `--lib` to the test step, which skips every `tests/*.rs`, golden included;
  - changing a subcommand;
  - pointing book at `Action::Policy`.

  Any of them would empty out INT-0003 AC-2 and AC-4, INT-0002 AC-2 and INT-0004 AC-1.
- **Suggested response:** tighten-assertion — `assert_eq!` the whole `STEPS` array against the EARS commands and actions (`Step` and `Action` already derive `PartialEq`).

### C-005: The root `unsafe_code = "forbid"` lint behind INT-0003 AC-1 is never checked
- **Where:** `test-plan.md` Intent Traceability, `INT-0003` AC-1 row / `unit-tests.md` T-001 / `crates/xtask/src/policy.rs` `lints` and `forbid_unsafe` rules, `Fixture::new`
- **Quote:** "T-001 / `lints` and `unsafe` rules; every rule passes on the workspace and on the clean fixture"
- **Failure mode:** intent-coverage
- **Why it matters:** AC-1's proxy is the workspace lint, but no check reads it:
  - `lints` checks only that members say `[lints] workspace = true`;
  - `unsafe` reads only `src/lib.rs` and `src/main.rs`;
  - the clean fixture has no `[workspace.lints.rust]` table, yet it passes every rule.

  A scratch probe showed that with the root value set to `"allow"`, clippy and the tests stay green with `unsafe` in a `tests/*.rs` target. None of the seven such targets has `#![forbid(unsafe_code)]`.
- **Suggested response:** add-test — check that the root has `[workspace.lints.rust] unsafe_code = "forbid"` and list the check under INT-0003 AC-1. Either add it to the `lints` rule (with the table in the clean fixture and an `"allow"` negative fixture) or write an xtask test against the real manifest.

### C-006: Records name a clean head that the working tree no longer matches
- **Where:** `unit-tests.md` header "Tested head" / head lines of `integration-tests.md` and `e2e-tests.md` / working tree `crates/hash/tests/vectors.rs`, `Cargo.toml`, `Cargo.lock`, three crate manifests, `docs/prior-art.md`
- **Quote:** "on `dev`, committed, with a clean working tree."
- **Failure mode:** evidence-drift
- **Why it matters:** After the records were written against 40f6afb, `text_rejects_malformed` was rewritten to assert exact errors. Later edits added `pulldown-cmark`, `toml_edit` and `allocation-counter` to the manifests, the lockfile and the prior-art list. The rewrite hides no failure on Windows, but no recorded run covers the tree that will be committed, so a report written from these records would vouch for an untested head.
- **Suggested response:** tighten-assertion — commit or revert the pending changes, re-run `cargo ci` on all three checkouts and `cargo deny check`, and update the tested head, pass counts and clean-tree lines in all three records before writing `test-report.md`.

### C-007: `PROPTEST_CASES` can silently override every configured case count
- **Where:** `crates/xtask/src/ci.rs` `cargo_command` (T-007 test step) / `build-plan.md` T-004 and T-006 property EARS / `unit-tests.md` Randomness line / `INT-0006` AC-1 and AC-5 proxies
- **Quote:** "`.env_remove(GOLDEN_CREATE);`"
- **Failure mode:** weak-assertion
- **Why it matters:** proptest 1.12 passes even explicit `with_cases(10_000)` and `with_cases(100_000)` configs through `contextualize_config`. So `PROPTEST_CASES=0` makes every property pass without running a single case, printing the same "ok" lines as a full run. The runner removes only `PERI_GOLDEN_CREATE`, the listing run bypassed `cargo ci`, and no record attests the environment. The "at least 10,000 / 100,000 cases" proxies could therefore rest on a false pass, though neither host had the variable set.
- **Suggested response:** add-test — add guard tests in `properties.rs` and `book.rs` that fail when the effective case count is below the configured N, remove `PROPTEST_CASES` in the test step, and state in the records that no `PROPTEST_*` variable was set.

### C-008: The publish rule's negative case for the inherited form is untested
- **Where:** `build-plan.md` T-001 EARS (`publish`) / `crates/xtask/src/policy.rs` `publish()`, `policy_requires_unpublished_members` / `unit-tests.md` T-001 row 3 / `INT-0003` AC-8
- **Quote:** "neither `publish = false` nor `publish.workspace = true` with the workspace value `false`"
- **Failure mode:** negative-path
- **Why it matters:** `Fixture::new` always writes `[workspace.package] publish = false`. As a result, a mutant that ignores the workspace value (dropping the `workspace_unpublished &&` guard) still passes all 24 xtask tests. All four real members use `publish.workspace = true`, so that guard is the only policy check that would catch a root `publish = true`, which Cargo accepts. AC-8's "every member unpublished" half is unproved, though its row reads pass.
- **Suggested response:** add-test — add fixtures in which the member keeps `publish.workspace = true` and the root `[workspace.package]` either sets `publish = true` or omits `publish`, each expecting `fails_only("publish", "crates/hash")`.

### C-009: Workflow test does not scope its checks to jobs
- **Where:** `crates/xtask/src/ci.rs` `workflow_matches_ci_contract` / `build-plan.md` T-007 EARS (workflow) / `test-plan.md` `INT-0003` AC-3 row / `unit-tests.md` T-007 workflow row
- **Quote:** "`let deny_job = &lines[deny..];`"
- **Failure mode:** weak-assertion
- **Why it matters:** The test treats everything from `deny:` to the end of the file as the deny job, and searches every trimmed line for the matrix and `- run: cargo ci`. Three mutants that break the per-job EARS all pass:
  - `cargo ci` moved into the deny job;
  - a Windows deny job followed by an ubuntu audit job;
  - a matrix job that never runs `cargo ci`.

  Until T-009 records a live run, this test is Sprint 0's only evidence for the clause and for AC-3's deny job. Adding any job after `deny` would make the deny checks meaningless.
- **Suggested response:** tighten-assertion — split the untrimmed lines into job blocks under `jobs:`. In the ci block, assert the `os:` matrix, `runs-on: ${{ matrix.os }}` and `- run: cargo ci`. In the deny block, assert `runs-on: ubuntu-latest` and the cargo-deny step.

### C-010: The spec rule's Hash algorithms table, status columns and most sections are untested
- **Where:** `crates/xtask/src/policy.rs` `policy_requires_format_spec`, `SPEC_SECTIONS`, `SPEC_TABLES` / `build-plan.md` T-002 EARS 1 and 2 / `unit-tests.md` T-002 row / `INT-0006` AC-6
- **Quote:** "`fixture.fails_only("spec", "Object kinds");`"
- **Failure mode:** negative-path
- **Why it matters:** The tests remove only the Object kinds row, the Worked examples row and one heading. These mutants all pass:
  - Hash algorithms dropped from `SPEC_TABLES`;
  - either registry's minimum cell count lowered, so the status column is no longer required;
  - "Change control" dropped from `SPEC_SECTIONS`.

  No test guards seven of the required sections, yet the T-002 row records the clause as pass.
- **Suggested response:** add-test — remove the Hash algorithms row (expect a FAIL naming it) and use status-less registry rows (expect FAIL `spec`). Then loop over the 11 section names, hard-coded in the test, removing each heading and expecting a FAIL naming it.

### C-011: Book violations are never asserted to name the file
- **Where:** `crates/book/tests/book.rs` `kinds()`, `typed_line_violations_are_reported`, `declared_book_rejects_untyped_items` / `crates/book/tests/cli.rs` `cli_exit_codes` / `build-plan.md` T-006 EARS 2 / `unit-tests.md` T-006 row
- **Quote:** "`violations.iter().map(|v| (v.kind, v.line)).collect()`"
- **Failure mode:** weak-assertion
- **Why it matters:** The EARS promises a violation naming the file, the 1-based line and the rule, but `kinds()` drops the path and the CLI test checks only `contains(": untyped: ")`. A regression in how violations are built, or in the printed `{path}:{line}:` prefix, would pass while `unit-tests.md` records "file, line and rule" as pass.
- **Suggested response:** tighten-assertion — make `kinds()` return the path and assert it, and assert that the CLI's stdout contains `"INT-0001-x.md:4: untyped: "`.

### C-012: The deps failure is not asserted to name the crate
- **Where:** `crates/xtask/src/policy.rs` `policy_flags_direct_dependency` / `build-plan.md` T-001 EARS (`deps`) / `test-plan.md` T-001 "naming crate and dependency" / `unit-tests.md` T-001 `deps` row
- **Quote:** "`` fixture.fails_only("deps", "`foo`"); ``"
- **Failure mode:** weak-assertion
- **Why it matters:** The EARS and the test plan require the failure to name both the crate and the dependency, but the needle checks only the dependency. A message that lost the crate path would still pass all four cases, while the record marks the clause pass.
- **Suggested response:** tighten-assertion — use the needle `` "crates/hash declares `foo`" `` in all four cases.

### C-013: The hash properties run proptest's 256-case default, not the recorded 10,000
- **Where:** `unit-tests.md` Randomness line and T-003 property rows / `crates/hash/tests/vectors.rs` `proptest!` block
- **Quote:** "fixed case counts: 10,000 per property, and 100,000 for decoder totality"
- **Failure mode:** evidence-drift
- **Why it matters:** `binary_round_trip`, `text_round_trip` and `incremental_matches_one_shot` have no `proptest_config`, so they run 256 cases, about 39 times fewer than recorded. T-003 sets no case count, so no AC is unproved, but the record overstates the T-003 sample. "Fixed" is also untrue whenever `PROPTEST_CASES` is set (C-007).
- **Suggested response:** tighten-assertion — add `#![proptest_config(ProptestConfig::with_cases(10_000))]` to the hash block, or correct the Randomness line for these three properties.

### C-014: In the prior-art check, "JGit" always satisfies "Git"
- **Where:** `crates/xtask/src/docs.rs` `prior_art_credits_every_listed_system` / `build-plan.md` T-008 EARS 1 / `unit-tests.md` T-008 row "(36 names)"
- **Quote:** "`.filter(|name| !credits.contains(name))`"
- **Failure mode:** weak-assertion
- **Why it matters:** The check matches substrings, so the required name `Git` can never fail on its own: `JGit`, also required, contains it. Deleting Git's own row therefore passes, and only 35 of the 36 recorded names are checked independently. The impact is low because INT-0003 AC-5 rests on the owner's review.
- **Suggested response:** tighten-assertion — match each name as a whole word bounded by non-alphanumeric characters.

### C-015: The unit listing run names no host or checkout
- **Where:** `unit-tests.md` Runner and Toolchain lines / `test-plan.md` recording rule
- **Quote:** "The listing below comes from running that command directly."
- **Failure mode:** evidence-drift
- **Why it matters:** The test plan requires every recorded run to state its host OS release, `rustc -Vv`, head SHA and checkout. The 94-test listing names no host or checkout and lists both target triples. Because it was a direct `cargo test`, it also skipped the environment cleanup that `cargo ci` applies. The recorded `cargo ci` runs back every pass on their own, so nothing is unproved.
- **Suggested response:** tighten-assertion — name the listing run's host, OS release and checkout and state that `PERI_GOLDEN_CREATE` and `PROPTEST_*` were unset, or take the listing from the `int_ci_windows` output.

### C-016: The HashMap probe does not name its host
- **Where:** `integration-tests.md` `int_disallowed_types_probe` / `INT-0005` AC-4
- **Quote:** "clone the tested head into a scratch directory, append a function that builds a"
- **Failure mode:** evidence-drift
- **Why it matters:** Every other integration result names its host or checkout. The probe names neither its host and OS release nor where the scratch clone lived, which breaks the test plan's rule for recorded runs. The pinned toolchain fixes Clippy's version, so the result itself stands.
- **Suggested response:** tighten-assertion — add the probe's host, OS release, `rustc -Vv` host triple and scratch clone path, and re-run on a named host if these cannot be stated truthfully.

### C-017: Deny evidence omits the advisory-DB revision
- **Where:** `integration-tests.md` `int_deny_at_t001`, `int_deny_at_t003`, `int_deny_windows` / `INT-0003` AC-3 local half
- **Quote:** "advisories ok, bans ok, licenses ok, sources ok"
- **Failure mode:** flake-risk
- **Why it matters:** `cargo deny check` fetches the RustSec database over the network, and no record names the database commit. The critic argued that the local AC-3 evidence therefore cannot be replayed or compared with T-009's floating `cargo-deny-action@v2`.
- **Suggested response:** reject (the critique is wrong because every Sprint 0 deny run used one advisory-db revision, `2295466c`, recoverable from the cache reflog, and no AC or EARS clause asks for it) — optionally note that revision beside `int_deny_windows` and have T-009 log the action's cargo-deny version.

## Confidence
block

## Responses

Round 1 ran on the draft records at `40f6afb`, while the code review's fixes were in progress. Every concern is addressed in the tested head that the final critique (`critique.md`) reviews.

| Concern | Response |
|---------|----------|
| C-001 | add-test. `policy` now reads manifests with `toml_edit`: target-specific tables and `[workspace.dependencies.<name>]` tables are checked (review findings xtask-1, xtask-2). Tests: `policy_flags_direct_dependency` (target tables), `policy_flags_uncredited_dependency` (table form), `policy_requires_complete_dependency_rows`. |
| C-002 | add-test. `clippy` parses `clippy.toml` and reads the `disallowed-types` entries; shadowing configs fail (xtask-3). Tests: `policy_requires_disallowed_hash_collections`. |
| C-003 | add-test. The executor's spawn-error and non-zero-exit paths run for real (xtask-11). The records no longer say "Stubs: none" for the runner test. Tests: `steps_execute_for_real`. |
| C-004 | tighten-assertion. `ci_steps_are_ordered_and_locked` asserts the whole step list (tests-1). |
| C-005 | add-test. `lints` requires `[workspace.lints.rust] unsafe_code = "forbid"` at the root (xtask-6). Tests: `policy_flags_missing_workspace_lints`. |
| C-006 | Re-ran every integration and E2E test at the fixed head and rewrote the records against it. |
| C-007 | add-test. The `cargo ci` test step removes `PROPTEST_*` variables. Guard tests in `hash`, `codec` and `book` fail when the effective case count differs from the configured one. Tests: `test_step_removes_golden_create`, `proptest_variables_are_never_inherited`, and the guards `case_count_is_not_overridden` (hash, book) and `case_counts_are_not_overridden` (codec). |
| C-008 | add-test. Cases with a workspace `publish = true` and with a missing workspace `publish` (tests-8). Tests: `policy_requires_unpublished_members`. |
| C-009 | tighten-assertion. The workflow test checks each job's own block (xtask-9). |
| C-010 | add-test. The spec rule's tests now cover the hash table, the status columns and all 11 sections. Tests: `policy_requires_format_spec`. |
| C-011 | tighten-assertion. Book violation tests assert the path (tests-12). |
| C-012 | tighten-assertion. The `deps` tests assert that the message names the crate. |
| C-013 | tighten-assertion. The hash properties run 10,000 cases. |
| C-014 | tighten-assertion. Credited names match as whole words (xtask-10). |
| C-015 | The unit listing now comes from the matrix run on the named host and checkout. |
| C-016 | The probe's record names its host, OS and toolchain. |
| C-017 | Rejected as the critic suggested. `int_deny_windows` records the advisory-db revision; round 2 (C-006) added it to the two earlier deny runs, from the database's reflog. |
