# Test Critique — Sprint 5

Round 2. The same read-only subagent reviewer re-reviewed the result files at `fad21f7` against its round-1 concerns ([critique-round1.md](critique-round1.md)). Every round-1 response landed as claimed.

**Re-verified for this round:**
- the C-002 fix, in a scratch clone: fmt, Clippy and the 111 `xtask` tests;
- the 69-line matrix comparison;
- all 15 job IDs and the time ranges;
- every unit in the per-unit table.

## Concerns

### C-001: `ci-sprint4.md` says "six" deny attempts; there are seven
- **Where:** `ci-sprint4.md`, "What each job showed", the `cargo deny check` row
- **Quote:** "`cargo deny check`, all six attempts, failed and passing"
- **Failure mode:** evidence-drift
- **Why it matters:** The Runs table lists seven deny jobs: four passing and three failed. All seven ran on `ubuntu-26.04` 20261028.624. Each failed log has 6 `invalid magic` lines and each passing log has 0. The finding is right; the count is wrong.
- **Suggested response:** tighten-assertion.

### C-002: `664b156` changes code after the tested head, and CI has not run it
- **Where:** `unit-tests.md`, "The code review fixes"; tested head `87df514`
- **Quote:** "At `664b156`, `xtask`'s binary passed 111 of 111 tests on Windows and in a fresh WSL clone. `664b156` changes nothing else."
- **Failure mode:** evidence-drift
- **Why it matters:**
  - No `cargo ci` or GitHub run covers this test-only commit.
  - It adds `stream_order_child`, so the next pull-request run will report 501 and 502 tests, not 500 and 501.
- **Suggested response:** defer-with-rationale. State it in the report, and have T-024 confirm the totals.

### C-003: Round-1 deferrals the test report must still carry
- **Where:** `critique-round1.md` Responses; `test-report.md`
- **Quote:** "The test report marks AC-2's `main` half as evidenced structurally only (`workflow.rs`'s exact-group rule) until then."
- **Failure mode:** evidence-drift
- **Why it matters:** The report must state three things the round-1 responses promised: AC-2's `main` half, INT-0007's pending owner decision, and AC-11's cold sample from `3289008`. It must also be linked from the verified intents.
- **Suggested response:** defer-with-rationale (accepted).

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---|---|
| C-001 | **tighten-assertion.** The row now reads "all seven jobs" and names them: passing 129038159059, 129244467046, 129244396365 and 129377865189; failed 129038191572, 129038413586 and 129038433532. |
| C-002 | **defer-with-rationale.** [test-report.md](test-report.md) gives the tested head as `87df514` plus the test-only `664b156`, and the totals the next pull-request run should print: 501 on Windows and 502 on Linux. Backlog T-024 now confirms those totals on the checkpoint and merge runs. Until the checkpoint push, `664b156` is covered by its own binary's 111 passing tests on both systems, and by the reviewer's scratch-clone fmt, Clippy and test run. |
| C-003 | **accepted.** [test-report.md](test-report.md) states each point: AC-2's `main` half is evidenced structurally until the merge (T-024); INT-0007's move to `realized` is put to the owner; AC-11's only cold sample is at `3289008`. The report is added to the Test evidence of INT-0003, INT-0004, INT-0005 and INT-0007. |
