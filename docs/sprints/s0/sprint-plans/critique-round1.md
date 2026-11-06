# Plan Critique — Sprint 0, round 1 (superseded)

Provenance only. This was the first critic pass over the Sprint 0 plans. Its
verdict was `block`. Every concern was addressed in the revised plans, and the
final verdict is in [`critique.md`](critique.md). The resolution of each
concern is listed at the end.

## Concerns (summary of the critic's report)

- **C-001** intent-drift — The format spec (old T-004) depended on the code it specifies (T-002, T-003), breaking INT-0006 AC-6 "spec before code".
- **C-002** plan-test-mismatch — The README/prior-art task (old T-007) had EARS clauses with no named tests; design credits were unmeasurable; the licence column was unchecked.
- **C-003** plan-test-mismatch — No rule caught a missing or empty criteria section; `status([])` was Certified; there were no clauses for wrapped typed claims, an unknown declaration value or `AC-0`.
- **C-004** plan-test-mismatch — The object envelope was missing from the round-trip, mutation and totality property sets.
- **C-005** plan-test-mismatch — No test read the CI workflow; "in CI" criteria depended on a run after the Test Phase; `int_deny_windows` traced to no clause; cargo-deny's place outside `cargo ci` lived only in task notes.
- **C-006** e2e-drift — The Windows line-ending risk was never tested on a fresh Windows checkout; the WSL run's location and toolchain records were unspecified.
- **C-007** hidden-dep — `* text=auto eol=lf` would normalise a committed CRLF fixture, so the CRLF test would compare LF with LF in every clone.
- **C-008** hidden-dep — Dependencies were added after `deny.toml` was written, with no task updating it; sha2 0.11.1 was unverified.
- **C-009** intent-drift — Blob identity (hash of the envelope) contradicted INT-0008 AC-2's wording, and freezing v1 golden files clashed with INT-0008 AC-5.
- **C-010** plan-test-mismatch — The no-allocation test used a ~4 GiB length that a 64-bit host can allocate, so it could not fail; count prefixes were untested.
- **C-011** EARS-vague — Reserved object kinds had no named error; "equal exactly" ignored the spec's reserved rows; the `u32` range had no clause.
- **C-012** intent-drift — INT-0003 AC-5 (design credits, signed off) was relabelled as a mechanical dependency check, and dependencies declared directly in member manifests escaped the rule.
- **C-013** hidden-dep — Edition 2024 makes `std::env::set_var` unsafe, so no test could flip `PERI_GOLDEN_CREATE`; the missing-file branch with create mode off was untested.
- **C-014** EARS-vague — Hard-coded chapter counts (31, 13) would break valid future sprints.
- **C-015** intent-drift — INT-0005 AC-4's `disallowed-types` guard was not applied to `codec`, the first serialised code path.

## Confidence
block

## Resolution in the revised plans

| Concern | Resolution |
|---|---|
| C-001 | The spec is now T-002 and comes before `hash` (T-003) and `codec` (T-004); the golden corpus and spec-consistency checks moved to T-005. A `spec` policy rule verifies the spec at its own commit. |
| C-002 | Prior-art credits are written in T-001 and extended by each task that adds a dependency or fixture. T-008 adds `prior_art_credits_every_listed_system` (with an explicit list) and `readme_keeps_statement_and_links_book`. The `credits` rule checks each dependency row's licence and C-build cells. |
| C-003 | New `missing` rule; `number` rejects `AC-0`; clauses and tests for wrapped typed claims and unknown declarations (exit 2); `status([])` is Undetermined, recorded in INT-0002. |
| C-004 | The envelope is in the round-trip, mutation and totality properties. |
| C-005 | New `workflow_matches_ci_contract` test; T-001 owns `cargo deny check`, and every dependency-adding task re-runs it; INT-0003 records deny's place and the CI-dependence of AC-2 to AC-4. |
| C-006 | New `e2e_fresh_clone_windows` (with `core.autocrlf=true`); the WSL run is an ext4 clone; every run records its OS release and `rustc -Vv`. |
| C-007 | The CRLF variant is generated in memory, and the test asserts it contains `\r\n`. |
| C-008 | `deny.toml` is in the touches of T-003 and T-004, with a `cargo deny check` clause; sha2 0.11.1 was verified with `cargo info` (MIT OR Apache-2.0, MSRV 1.88, no asm feature). |
| C-009 | INT-0006 records object identity and staged freezing; INT-0008 AC-2 and AC-5 were reworded, with history. |
| C-010 | Length and count prefixes of `u64::MAX` for bytes, `Vec`, `BTreeSet` and `BTreeMap`: any pre-allocation would overflow and panic. |
| C-011 | `ReservedObjectKind` versus `UnknownObjectKind`; the spec test compares full (id, name, status) rows including reserved ones; `u32` range clause with vectors. |
| C-012 | AC-5 is mapped to the owner's review plus the design-list test; the `credits` rule maps to AC-6. A new `deps` rule requires `workspace = true` for every dependency in a member manifest. |
| C-013 | The golden harness takes create mode as a parameter; the variable is read once at test entry; a missing file with create mode off fails and names the file. |
| C-014 | Expected chapter counts are derived from the `INT-*.md` files; the E2E records the Animus_Sprint_Loops commit and host. |
| C-015 | `clippy.toml` disallows `std::collections::HashMap` and `HashSet` workspace-wide in T-001, and INT-0005 moves to `planned`. A planted probe shows Clippy rejects them. |
