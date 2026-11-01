# Test Critique — Sprint 0, round 2

## Concerns

### C-001: No run shows `cargo ci` failing on a changed encoding
- **Where:** `INT-0006` AC-3 / `integration-tests.md` `int_ci_windows`, `int_ci_wsl` and `e2e-tests.md` `e2e_fresh_clone_windows` (cited as the AC-3 local-half evidence) / `unit-tests.md` T-007 row "two failing steps, one a missing tool → every step runs, non-zero exit, both named" / `crates/xtask/src/ci.rs` `run`, `crates/xtask/src/main.rs` `cli`, `main`
- **Quote:** "fails on Windows and Linux whenever an encoding changes without a format-version change"
- **Failure mode:** negative-path
- **Why it matters:** Every recorded `cargo ci` run with no arguments exits 0, and no test reaches `ci::run`. The T-007 non-zero-exit row rests on `ci_failures_do_not_stop_later_steps`, which scripts its outcomes, and on `steps_execute_for_real`, which covers `execute_with` only. So `run` passing failed steps to `finish`, the `ci` arm of `cli` and the exit status of `main` have never run on the failure path. If `run` called `finish(&[], out)` or `cli` dropped `run`'s code, every test and record would stay the same, yet the gate and the T-009 GitHub gate built on it could never fail.
- **Suggested response:** add-test — in a scratch clone of 4e25bfc, flip one byte of a golden `.bin` and record a probe like `int_disallowed_types_probe`: `cargo ci` on Windows and in WSL prints `FAIL test: ...`, the other four step lines and `ci: failed: test`, and exits non-zero.

### C-002: Golden rule passes with no root `.git`, against the locked clause
- **Where:** `build-plan.md` T-005 EARS (`golden` rule) and `test-plan.md` `policy_requires_binary_golden_attr` line / `crates/xtask/src/policy.rs` `golden()` and `policy_requires_binary_golden_attr` (golden-archive case) / `unit-tests.md` T-005 golden-rule row / `code-review.md` Plan deviation
- **Quote:** "golden `.bin` files not binary to Git → FAIL `golden`"
- **Failure mode:** EARS-coverage
- **Why it matters:** The locked clause, and the locked test-plan line for this same test, say `cargo policy` SHALL fail whenever `.gitattributes` does not mark `*.bin` as binary. Since 9672c26, though, `golden()` reports no problems when `<root>/.git` is absent, and the golden-archive case asserts a pass with `* text=auto eol=lf`. The unit-tests.md row rewords the trigger and records the clause as pass, and the Plan deviation bullet names only T-006. The skip also covers a copy nested inside a parent repository: a scratch probe showed Git reading a `.bin` as `text: auto` while `cargo policy` printed `PASS golden`. So the record backs a false pass even of its own paraphrase, although every AC run has a root `.git` and takes the `git check-attr` path.
- **Suggested response:** defer-with-rationale — keep the behaviour, but record it in code-review.md and the unit-tests.md row as a second plan deviation against the T-005 `golden` clause. State the exception as "no `.git` entry at the workspace root" and name nested copies as a known limit, or make the skip ask `git rev-parse --is-inside-work-tree` instead.

### C-003: Clippy's `HashSet` trigger never executed
- **Where:** `integration-tests.md` `int_disallowed_types_probe` and its `test-matrix-log.md` run / `unit-tests.md` T-001 row "a member uses `HashMap` or `HashSet`" / `build-plan.md` T-001 EARS (Clippy `disallowed_types`) / `INT-0005` AC-4
- **Quote:** "Append a function that builds a `std::collections::HashMap::<u8, u8>` to `crates/codec/src/lib.rs`."
- **Failure mode:** EARS-coverage
- **Why it matters:** The locked clause fires when a member uses `HashMap` or `HashSet`, and INT-0005 AC-4 (`empirical`) names both. Yet the T-001 completion probe, the code-review check and the Test-phase run all planted only a `HashMap`, while unit-tests.md marks the whole clause pass. If Clippy did not reject `HashSet`, nothing recorded could tell. The `clippy` policy rule, which pins the config, makes that unlikely, but that is inference, not observation.
- **Suggested response:** add-test — plant `std::collections::HashSet::<u8>::new()` in its own scratch-clone run at 4e25bfc, and record Clippy's `HashSet`-specific "use of a disallowed type" error and the non-zero exit in both probe records. Fallback: defer-with-rationale, restating the T-001 row as inferred for `HashSet`.

### C-004: Changed-byte golden test cannot isolate the encoder comparison
- **Where:** `unit-tests.md` T-005 row "create mode off, one changed byte" / `crates/codec/tests/golden.rs` `golden_changed_file_fails`, `golden_create_mode_never_overwrites`, `check_golden` / `test-plan.md` `golden_changed_file_fails` line / `build-plan.md` T-005 EARS 1 and 3 / `INT-0006` AC-3
- **Quote:** "`golden_changed_file_fails` (exactly one failure)"
- **Failure mode:** weak-assertion
- **Why it matters:** The flipped byte turns "periapsis" into "pesiapsis", so the real harness reports two failures ("encoding changed" and "does not decode"). That makes the record's "exactly one failure" false, and the test checks only that failures exist and name the file. Deleting the `file != case.bytes` block, the only place where the encoder's output meets the committed bytes, leaves all five golden tests and both spec tests green, and `golden_create_mode_never_overwrites` has the same blind spot. At 4e25bfc the block is still present, and `golden_v1_corpus_matches` runs it over all 23 files.
- **Suggested response:** tighten-assertion — assert the exact two-message failure list, and correct the record's "(exactly one failure)", noting that it departs from the test plan. Also add a case whose `Case.bytes` differ from a file that still decodes, such as uvarint-0 as `80 00` against the file `00`, and expect exactly one "encoding changed" failure that names uvarint-0.bin.

### C-005: Env-stripping run cites an unidentified pre-commit snapshot
- **Where:** `integration-tests.md` header Environment bullet, `int_cc_build_dependency_audit` credits bullet / `e2e-tests.md` Oracle line / `code-review.md` Method step 4 / `build-plan.md` T-007 EARS (`test` step removes `PERI_GOLDEN_CREATE`)
- **Quote:** "a code-review verifier ran `PROPTEST_CASES=1 PERI_GOLDEN_CREATE=1 cargo ci` on a committed copy of the tree"
- **Failure mode:** evidence-drift
- **Why it matters:** That run used scratch snapshots of 4a2c07a plus fixes that were not yet committed: a9bd764, 90f50ad and an uncommitted tmpfs copy. Their `ci.rs` did not yet strip `CLIPPY_CONF_DIR`. The record gives no SHA or checkout location, although test-plan.md requires both, and "a committed copy" reads as if it meant 4e25bfc. Neither matrix script sets or prints these variables, so "no ... variable was set" is not in the raw log. At 4e25bfc, `test_step_removes_golden_create` and `proptest_variables_are_never_inherited` still cover the clause, so only the record misleads.
- **Suggested response:** tighten-assertion — describe the run as a pre-commit snapshot of 4a2c07a plus uncommitted fixes, not 4e25bfc, or drop it. Either mark the unset-environment claim as unlogged or have `test_matrix.sh` print the environment, and fix the credits bullet and the Oracle line the same way.

### C-006: C-017 response overstates where the advisory-db revision is recorded
- **Where:** `critique-round1.md` Responses, C-017 / `integration-tests.md` `int_deny_at_t001`, `int_deny_at_t003` / `completed-tasks.md` T-001, T-003
- **Quote:** "The advisory-db revision is still recorded beside each deny run."
- **Failure mode:** evidence-drift
- **Why it matters:** Only `int_deny_windows` records a revision (advisory-db 2295466c). The T-001 and T-003 deny records and their completion entries give only the four "ok" results, so the response overstates what the records hold. No AC or EARS clause requires the revision, and the cache reflog shows that all three runs used 2295466c, so this only blocks replaying those runs from the written records.
- **Suggested response:** tighten-assertion — add "advisory-db 2295466c (cache reflog, 17:03:12 and 17:06:43)" to the `int_deny_at_t001`/`int_deny_at_t003` entry, or reword the C-017 response to name only `int_deny_windows`.

## Confidence
block
## Responses

Round 2 reviewed the records at `4e25bfc`. Each concern is addressed in the head that the final critique (`critique.md`) reviews.

| Concern | Response |
|---------|----------|
| C-001 | add-test. `int_ci_reports_failures` runs `cargo ci` on Windows and in WSL, in a scratch clone with one golden byte changed and one formatting violation. It must print `FAIL fmt` and `FAIL test`, with the other three steps passing, then `ci: failed: fmt, test`, and exit 1. That run goes through `run`, `finish`, the `ci` arm of `cli` and `main`'s exit status. `int_ci_ignores_hostile_environment` adds a second failing run. |
| C-002 | fixed in `0b8c498`, and in `7f9a63c` after a verifier found that Git also needed to run at the workspace root; no deviation needs recording. The golden rule now evaluates the workspace's own `.gitattributes` files through a private, empty repository whose work tree is the workspace, with the caller's Git settings and the machine's global and system attributes shut out. It fails whenever the files do not make `*.bin` binary, inside a checkout or out of one, which is the locked clause as written. Since `7f9a63c` it checks every committed golden file as well as the probe, and every attribute that rewrites bytes. Tests: `policy_requires_binary_golden_attr`, including a broken `.git` that changes nothing, and the `attributes` unit tests. |
| C-003 | add-test. `int_disallowed_types_probe` now plants `HashMap` and `HashSet` in two separate clones, and each must get its own "use of a disallowed type" error and a non-zero exit. |
| C-004 | tighten-assertion and add-test in `76ff8a4`, extended in `ac0e010` (create mode, undecodable files, unequal lengths). `golden_changed_file_fails` asserts its two failures exactly. `golden_encoder_change_fails` changes only the encoder's bytes, so the byte comparison alone must catch it. The unit record now gives two failures, not "exactly one". |
| C-005 | The environment claim now rests on the tested head. `int_ci_ignores_hostile_environment` runs `cargo ci` with `PROPTEST_CASES=1`, `PERI_GOLDEN_CREATE=1` and an empty `CLIPPY_CONF_DIR` inside the final matrix. The matrix prints which of these variables the host sets. The earlier verifier snapshot is no longer cited as evidence. |
| C-006 | tighten-assertion. `int_deny_at_t001` and `int_deny_at_t003` now give the advisory-db revision, `2295466`, from the database's reflog: it was fetched at 17:03:12, before the T-001 commit, and reused at 17:06:43, before the T-003 commit. Round 1's C-017 response now names only the runs that recorded the revision. |
