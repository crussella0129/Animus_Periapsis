# Test Critique — Sprint 0, round 8

Two independent critics reviewed the records at `87fde81` (tested head `0a9fd40`): one for trace coverage and evidence, one for assertion strength and the changes since round 7. Their concerns are merged below.

## Concerns

### C-001: A legacy `.cargo/config` file overrides the pinned aliases, and no test reads it
- **Where:** `crates/xtask/src/pins.rs` `cargo_aliases_are_the_reviewed_ones` / `critique-round7.md` Responses, C-001 / `INT-0003` AC-2, AC-9
- **Quote:** "so dropping `--locked` from any alias, or the `book` alias's trailing `--`, fails a test"
- **Failure mode:** weak-assertion
- **Why it matters:** Cargo prefers a sibling `.cargo/config` to `.cargo/config.toml` when both exist. On a scratch clone of `0a9fd40` with one that redefined the three aliases without `--locked` or the trailing `--`, `cargo book --strict` exited 1, `cargo policy` regenerated a deleted lock and passed, and every test, the pin included, still passed. No `.cargo/config` is tracked, so nothing recorded is false.
- **Suggested response:** add-test — require `.cargo/config` to be absent

### C-002: The `credits` rule counts a dependency row that GitHub hides
- **Where:** `crates/xtask/src/policy.rs` `uncredited` / `build-plan.md` T-001 EARS (`credits`) / `mutation-sweep.md` Outcome / `INT-0003` AC-6
- **Quote:** "`.find(|line| line.trim_start().starts_with(&row_start))`"
- **Failure mode:** negative-path
- **Why it matters:** With `blake3`'s row deleted from the table and put back only inside `<!-- ... -->`, `cargo policy` printed `PASS credits` and all 171 tests passed. It is the defect the sweep fixed in the `spec` rule, kept in `credits`. The committed file holds real rows.
- **Suggested response:** tighten-assertion — read the rows with pulldown-cmark and add hidden-row fixtures

### C-003: The sweep record's reader-mutant total does not add up
- **Where:** `mutation-sweep.md` Method, step 5, and its Second pass rows
- **Quote:** "mutated both new readers: 26 mutants, each failing a test."
- **Failure mode:** evidence-drift
- **Why it matters:** The Second pass and `code-review.md` give 15 and 10 mutants for the two readers, 25 in all. No result depends on the number.
- **Suggested response:** tighten-assertion — correct the count or name the 26th mutant

### C-004: Scratch directories are assumed empty but never checked
- **Where:** `crates/codec/tests/golden.rs` `scratch_dir`; `policy.rs` `Fixture::new`; `book.rs` `TempBook::new`; `cli.rs` `temp_dir`
- **Quote:** "An empty directory under the OS temp directory, unique to this process."
- **Failure mode:** flake-risk
- **Why it matters:** A panicking golden test leaves its directory, and the startup removal ignores errors. A later run with the same process id whose removal failed could find correct files already written, so a create mode that writes nothing would pass. It needs a reused process id and a failed removal; no recorded run shows leftover state.
- **Suggested response:** tighten-assertion — assert the directory is empty after creating it, or never reuse one that exists

## Confidence
block

## Responses

C-001 is material, so the round blocked. Each concern is addressed in `c342412`, or in the records committed with this file; mutating each fix makes a test fail.

| Concern | Response |
|---------|----------|
| C-001 | add-test in `c342412`. `cargo_aliases_are_the_reviewed_ones` now also fails when `.cargo/config` exists; planting one that redefines `ci` makes it fail. Together with the pin of `.cargo/config.toml`, no file in the checkout's `.cargo/` can change the aliases unseen, and the matrix already prints the configuration files above each checkout and in `CARGO_HOME`. |
| C-002 | tighten-assertion in `c342412`. `credits` reads `docs/prior-art.md` with pulldown-cmark, as the `spec` rule does, and finds a dependency by a top-level table row whose first cell is its name as code. `policy_flags_uncredited_dependency` adds the row in a comment, a fence, a quote and a list item, and with the name not as code, each failing `credits` with "has no row"; a table after a heading, with CRLF, or after a fence and a comment ended by lone CRs passes. `policy_requires_complete_dependency_rows` now writes each row under the table's header and requires the "needs a licence" problem. Three mutants of the new reader (nested tables read, the name matched without its code span, lone CRs left to the parser) each fail a test. The sweep record's Outcome names this second instance of the defect. |
| C-003 | tighten-assertion. Step 5 now gives 25 reader mutants (15 and 10) and the one golden-corpus mutant run with them, 26 in all. |
| C-004 | defer-with-rationale. It needs a reused process id and a removal that failed in the same place, and every recorded run starts from fresh clones whose temp directories are new. Making each fixture claim a new directory, as `attributes.rs` does, is recorded as a known limit in [code-review.md](code-review.md) for a later head. |
