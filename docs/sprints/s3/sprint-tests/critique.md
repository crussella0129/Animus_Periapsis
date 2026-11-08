# Test Critique — Sprint 3

Round 2. A read-only subagent reviewer wrote it from the bundle's `prompts/test-critic.md`. It checked the responses to [critique-round1.md](critique-round1.md) against tested head `ad22319`, the test commit `0678139`, and the raw logs of both legs. It ran no builds or tests. The concerns are condensed here; the responses are at the end.

**Round-1 responses.** All ten were checked. C-001, C-005, C-006, C-009 and C-010 landed as stated:
- The printed `undo` id is compared with H's parent. That check would catch an `undo` that undid its own snapshot.
- `ops_are_never_removed` fails on any `undo` error but `NothingToUndo`, and checks `log` against the store.
- The CLI blocking case asserts the exit code, stdout, stderr, the files, the record and `log`.

The C-003 deferral is stated. `test-matrix-log.md` matches both raw logs line for line, apart from the shortened path and the dropped `test … ok` lines. Neither log holds a NUL byte. The counts add up: 403/404 + 4 = 407/408.

## Concerns

### C-001: The Windows "none SKIPPED" claim for the C-008 resume test is not in the log
- **Where:** `unit-tests.md`, Round 1 additions; `s3_windows.sh`, section `int_reversibility_and_hazards_windows`
- **Quote:** "passed, with a directory symlink (none `SKIPPED`)"
- **Failure mode:** evidence-drift
- **Why it matters:** The test prints `SKIPPED` and returns when it cannot make the symlink, which counts as a pass. The script runs `real_hazards` once with `--nocapture` but filtered by a `grep`, and once without `--nocapture`. Neither run can show a `SKIPPED` line. Developer Mode is on, so the symlink was likely made, but the log does not show it.
- **Suggested response:** tighten-assertion. Re-run with `--nocapture` and look for `SKIPPED`, or soften the claim.

### C-002: The Windows-rules matrix says "names fold case", but `SimFs` never folds, and the scenario has no name the rules affect
- **Where:** `integration-tests.md`, Materialise crash matrix row "Correct, Windows rules"; the `materialise_crash.rs` `Host::Windows` doc comment; `build-plan.md` T-003 ("`SimFs` lookups stay exact-case")
- **Quote:** "names fold case and Windows' naming rules apply"
- **Failure mode:** evidence-drift
- **Why it matters:**
  - `SimFs` only reports the name rules it is given.
  - None of the scenario's names is a case variant, a reserved name, or a fold of a temporary name. The fold and hazard checks run but never trigger.
  - In effect the variant adds only the bytes-only "matches F" rule.
  - No reproducibility check covers its cells.
- **Suggested response:** tighten-assertion. Reword the claim, and add a digest or say that reproducibility is shown for the Unix variant only.

### C-003: The C-007 test shows only that the file survives, and survival does not depend on CR-02
- **Where:** `real_hazards.rs` `real_names_without_utf8_survive_undo`; INT-0004 AC-7; `workspace/src/lib.rs` `undo` and `restore_view`
- **Quote:** AC-7: "never tracked under a changed name, overwritten, deleted or skipped without a report."
- **Failure mode:** weak-assertion
- **Why it matters:**
  - **Survival is not CR-02.** `apply` keeps a directory if any entry is left, so the test passes whatever the listing keys are.
  - **No report is shown.** `undo` and `restore_view` drop `Snapshot::reported`, and `praps undo` prints only the op line. So `undo` over such a name says nothing about it.
- **Suggested response:** defer-with-rationale, stating what the test shows and filing a backlog task. Or tighten-assertion.

### C-004: `e2e-tests.md` cites the wrong log section and an imprecise intent for the new CLI refusal test
- **Where:** `e2e-tests.md` `praps_restore_refusals_exit_one`
- **Quote:** "Passed on Windows and on Linux (section `e2e praps`)"; "[INT-0004] AC-3"
- **Failure mode:** evidence-drift
- **Why it matters:** `cli.rs` runs under `e2e praps_fsck_end_to_end` in both logs. The Windows case is a case-only rename, the known limit in backlog T-016, not AC-3's two tracked case-colliding paths.
- **Suggested response:** tighten-assertion.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---------|----------|
| C-001 | tighten-assertion. `cargo test -p workspace --test real_hazards -- --nocapture` was re-run on Windows at `19ec3d6`, whose crates are unchanged from `ad22319`. 3 passed, and no `SKIPPED` line was printed, so the directory symlink was made. [unit-tests.md](unit-tests.md) records this, and says that the leg's log cannot show a skip. |
| C-002 | tighten-assertion. The row in [integration-tests.md](integration-tests.md) now says the variant adds the no-executable-bit rule under crashes. It also says the name rules are only reported, and no scenario name triggers them, so case folding and reserved names are covered by the hazard corpora without crashes. The determinism line now says that the digest covers the Unix-rules variant only. The test's doc comment still says names fold case. It is a comment in a test at the tested head; the text is corrected in the docs and is not changed after testing. The same correction applies to the round-1 response for C-004. |
| C-003 | defer-with-rationale. [unit-tests.md](unit-tests.md) now says the test shows only "kept, not deleted", which does not depend on CR-02's key. CR-02 itself is shown by `control_characters_in_names_are_seen`. It also says that `undo` does not report the skipped name. Backlog T-019 makes `undo`, `restore` and `new` return and print their snapshot's report, with a CLI test for each. Until then, INT-0004 AC-7's "without a report" holds for `status` only. The test report lists this. |
| C-004 | tighten-assertion. [e2e-tests.md](e2e-tests.md) now cites section `e2e praps_fsck_end_to_end`. It describes the Windows case as the current refusal of a case-only rename (backlog T-016), not as INT-0004 AC-3 evidence. |
