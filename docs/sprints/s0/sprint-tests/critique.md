# Test Critique — Sprint 0, round 9

One critic reviewed the records at `08c4787` (tested head `c342412`): the responses to round 8 and the changes since, and a full pass over every EARS clause and the matrix log. In a scratch clone with its own target directory, `cargo test --workspace --locked` passed 171 tests, with per-binary counts matching the log. Each round-8 response held: a planted legacy `.cargo/config` fails `cargo_aliases_are_the_reviewed_ones`; the three named mutants of the `credits` reader each fail `policy_flags_uncredited_dependency`; the real `docs/prior-art.md` passes `credits`, and hiding the `sha2` licence or the `blake3` features in a comment fails it; the sweep record's count adds up. Every EARS clause of T-001 to T-008 has a named test or run, and the records agree with `test-matrix-log.md` (head, 171 tests on both systems, advisory-db `26fd2d4`, a 59 s cold build, 132 tracked files, 31 chapters, the probe exits), with `code-review.md` and with `mutation-sweep.md`.

## Concerns

### C-001: A name written with escaped backticks still counts as the name as code
- **Where:** `unit-tests.md` T-001, `credits` row / `crates/xtask/src/policy.rs` `table_cells`, `uncredited` / `critique-round8.md` Responses, C-002
- **Quote:** "or with the name not as code, fails `credits` with \"has no row\""
- **Failure mode:** weak-assertion
- **Why it matters:** `table_cells` joins text events, so an escaped `` \`blake3\` `` first cell reads as `` `blake3` `` although GitHub shows it as plain text; with that row, `cargo policy` printed `PASS credits`, contrary to the record. The row is still visible, and the clause asks only for a row, so no AC-6 promise is unproved and nothing hidden is counted.
- **Suggested response:** defer-with-rationale — reword the record to say the name's text must read `` `name` ``, or match the cell on a code span and add the escaped case as a fixture

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---------|----------|
| C-001 | defer-with-rationale (records only). The `credits` row in [unit-tests.md](unit-tests.md) now says the first cell is matched on its text: the name as plain text fails, and a name written with escaped backticks reads as the name, a row GitHub still shows. The rule's purpose, that every dependency has a visible, complete row, holds either way; matching on the code span itself can follow in a later head. |
