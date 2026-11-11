# Plan Critique — Sprint 6

Round 3. The same read-only subagent reviewer checked the plans at `e562914` against its ten round-2 concerns ([critique-round2.md](critique-round2.md); round 1 is [critique-round1.md](critique-round1.md)) and screened the new T-012 and the rewritten T-008. It found every response carried out and raised one new concern, given as it wrote it with file paths made relative to the repository; the response is at the end.

## Concerns

### C-001: T-012's exception misses a file the target has above the directory
- **Where:** `build-plan.md` T-012, clauses 1 and 2; `test-plan.md` T-012 unit tests; `docs/intents/INT-0014-working-copy-and-workspaces.md`, Consequences (the paragraph added in round 2); `crates/workspace/src/materialise.rs` (`plan`; `apply`, steps 2 and 4; `unchanged`); `crates/workspace/tests/real_hazards.rs`
- **Quote:** "and the target tree has no file at that path"
- **Failure mode:** EARS-vague
- **Why it matters:** `plan.emptied` holds every directory the source had files under and the target has none under, so when `a/b/x` gives way to a file `a` it holds `a/b` and then `a`, and `plan.write` holds `a`. If `a/b` cannot be removed, the target has no file at `a/b`, so clause 1 goes on: `a` is passed over because it is not empty, step 4 finds a directory where it must put the file `a`, takes that for an edit made during the write and keeps it, and the record is dropped. The command exits 0 with no file `a`, and the next snapshot records `a` as deleted: INT-0009 AC-3 is broken, INT-0014's "the working copy still equals the target" is false there, and neither unit test has a directory below the file's path. `apply` is also given `plan.write` and not the target tree, and a test calls it directly, so a rule stated on the target tree needs a change outside T-012's Touches.
- **Suggested response:** fix-in-plan. State the exception on what `apply` holds: the write stops, as today, when a path in `plan.write` is the directory or lies above it. Say the same in INT-0014, and add a test where `d/e` cannot be removed and the target has a file at `d`.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---|---|
| C-001 | **fix-in-plan.** T-012's two clauses now state the exception on what the write holds: it stops, as today, when a file it must put lies at the directory's path or at a path above it, and goes on otherwise. The Notes say why: a directory `a/b` that stays keeps `a` a directory, so a file due at `a` could not be put, and going on would drop the record with that file missing. INT-0014's Consequence says the same. A new test, `a_directory_below_a_file_to_put_stops_the_write`, has `d/e/x` in the source, a file `d` in the target and a refusal to remove `d/e`: the restore must fail with the record kept, and resume must put the file once the refusal is lifted. The rule is read from `Plan::write`, which `apply` is already given, so no signature changes. |
