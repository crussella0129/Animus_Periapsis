# Plan Critique — Sprint 3

Round 2, by a read-only subagent reviewer. Round 1 blocked with 17 concerns ([critique-round1.md](critique-round1.md)). Most responses landed. C-005, C-006 and C-010 were only partly sound against the code, and the revision introduced new materialise problems. The concerns are condensed here; the full quotes are in the review transcript.

## Concerns

### C-001: Materialise order writes files before it creates directories, and cannot change a path between file and directory
- **Where:** `build-plan.md` T-006 steps 1, 3, 5, 6
- **Failure mode:** missing-risk
- **Why it matters:**
  - Writes into new directories fail.
  - A file↔directory change is always refused.
  - A directory case-only rename lands under the old spelling.
- **Suggested response:** fix-in-plan. Reorder the steps and exempt F-owned entries.

### C-002: The executable bit is set after the rename, so a crash leaves T's bytes with F's mode, and resume never fixes it
- **Where:** `build-plan.md` T-006 step 3
- **Failure mode:** missing-risk
- **Why it matters:** T's bytes can be left with F's mode. The rule is also undefined on a filesystem with no executable bit.
- **Suggested response:** fix-in-plan. Set the bit on the temporary file before the sync and rename.

### C-003: Temporary names depend on state and can collide with the user's files
- **Where:** `build-plan.md` T-006
- **Failure mode:** missing-risk
- **Why it matters:** Indices shift on resume, and a tracked user file named `.praps-tmp-0` makes the repository stuck or is deleted.
- **Suggested response:** fix-in-plan. Name them from the F→T diff, and make collisions blocking.

### C-004: Resume has no rule for when it cannot finish
- **Where:** `build-plan.md` T-006 resume
- **Failure mode:** EARS-vague
- **Why it matters:** An entry created after the crash, or a leftover directory, makes every command fail.
- **Suggested response:** fix-in-plan.

### C-005: The exact crash oracle is unsound, because the scenario's user edits are never made durable
- **Where:** `build-plan.md` T-009; `crates/workspace/tests/crash.rs` `put`
- **Failure mode:** plan-test-mismatch
- **Why it matters:** An unsynced user edit can revert at any later crash, and a crash inside an edit step has no expected tree.
- **Suggested response:** fix-in-plan.

### C-006: Case-insensitive `SimFs` is not a faithful NTFS model
- **Where:** `build-plan.md` T-003; `simfs.rs` `recover`
- **Failure mode:** missing-risk
- **Why it matters:**
  - Independent recovery of entry changes can leave both `Foo` and `foo`.
  - The two-way fold is not transitive.
  - Full `to_uppercase` folds `ß` to `ss`, unlike NTFS.
- **Suggested response:** fix-in-plan.

### C-007: The blocking check lacks the case-only-rename exemption, and the hazard rules are not stated per component
- **Where:** `build-plan.md` T-006 step 1, T-005
- **Failure mode:** EARS-vague
- **Why it matters:** "F tracks that entry" is undefined for directories, and a collision such as `a` against `A/` is not covered.
- **Suggested response:** fix-in-plan.

### C-008: Hidden dependency: a third `Fs` implementation is missing from T-003's Touches
- **Where:** T-003 Touches; `crates/workspace/tests/real_edits.rs` `impl Fs for Under`
- **Failure mode:** hidden-dep
- **Why it matters:** `real_edits.rs` would stop compiling in T-003.
- **Suggested response:** fix-in-plan.

### C-009: "undo right after each op type" includes `init`, which T-007 says must fail
- **Where:** T-009, T-007
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The two clauses contradict each other.
- **Suggested response:** fix-in-plan.

### C-010: The T-002 walk does not cover renamed dependencies, its fixtures are incomplete, and other randomness routes remain
- **Where:** T-002
- **Failure mode:** plan-test-mismatch / missing-risk
- **Why it matters:** Renamed and target-specific dependencies have no fixtures, `"name version"` lock entries are not handled, and `fastrand` is not covered.
- **Suggested response:** fix-in-plan.

### C-011: `SimFs` shares directory state between `remove_dir` and a later `create_dir`
- **Where:** T-003; `simfs.rs` `dirs`
- **Failure mode:** missing-risk
- **Why it matters:** A pending removal and a pending re-creation share state under one path key.
- **Suggested response:** fix-in-plan.

### C-012: Minor gaps in the EARS clauses and the intents
- **Where:** T-006; INT-0014
- **Failure mode:** EARS-vague / intent-drift
- **Why it matters:**
  - Notes have no channel.
  - The intent says only commands that write the working copy resume.
  - Nothing traces that the record is durable before the op.
- **Suggested response:** fix-in-plan.

## Confidence
block

## Responses

Every concern is addressed in the plans and intents before round 3. The main decision: rather than model case-insensitive lookups faithfully (C-006), Sprint 3 refuses any case-variant write as a hazard. A case-only rename on a case-insensitive filesystem is therefore a recorded known limit (INT-0004, backlog T-016). That removes the need for case-insensitive `SimFs` lookups, and with it the directory-case and transitivity problems.

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan.<br>• T-006 classifies each path first (to remove, to write, kept). F-owned entries (paths to remove, and directories holding only those) never block.<br>• The order is: remove files; remove emptied directories, deepest first, syncing each and its parent; create directories, shallowest first; write files; sync; remove the record; sync `.praps`.<br>• Directory case-only renames are refused (C-006).<br>• Tests: `materialise_swaps_file_and_directory` (both ways), and file↔directory swaps in the AC-3 and crash corpora. |
| C-002 | fix-in-plan. Each file is written through its temporary name in the order write, `set_executable`, `sync_file`, rename. "Matches F" compares the mode only where the filesystem has an executable bit. Test: `executable_bit_is_set_before_the_rename`. The crash scenario adds an executable file whose content changes. |
| C-003 | fix-in-plan. Temporary names come from the sorted F→T diff, a pure function of F and T, so they are the same on resume. Any entry, or any F or T path, equal (after case folding where relevant) to a needed temporary name is a blocking entry, refused before anything is written. Test: `temporary_name_collisions_are_refused`. |
| C-004 | fix-in-plan. "Emptied" is defined as an empty directory that holds no T path, where F had files under it. When something still blocks, resume finishes every other path, drops the record, and adds a note naming the paths left. Nothing is lost: the op and the snapshot hold both trees. Test: `resume_finishes_what_it_can_and_notes_the_rest`. |
| C-005 | fix-in-plan. The scenario's edits are durable: each syncs its file and every directory it changes. The exact oracle applies to crashes inside commands. For crashes inside an edit step, only liveness is checked. |
| C-006 | fix-in-plan, by narrowing.<br>• `SimFs` lookups stay exact-case.<br>• The hazard check refuses any case-variant write: a different spelling among T's paths, or against an existing entry. So no write depends on case-insensitive lookup, on `SimFs` or on NTFS.<br>• Case-only renames are refused as a known limit (INT-0004 Consequences, T-016, which owns the faithful model).<br>• The conservative two-way fold stays only in the check. |
| C-007 | fix-in-plan. Hazards are checked per component, for every written path and created directory, including file `a` against directory `A/` (corpus case). F-owned is defined for files and directories. The case exemption is gone (C-006). |
| C-008 | fix-in-plan. `crates/workspace/tests/real_edits.rs` is in T-003's Touches. |
| C-009 | fix-in-plan. `undo_after_every_op_type` covers snapshot, new, undo and restore; `undo` right after `init` is the expected failure, tested by `undo_on_the_root_op_fails`. |
| C-010 | fix-in-plan. The walk resolves `package` renames, including inherited workspace aliases, and handles `"name version"` lock entries. It now covers `fastrand` too. Fixtures add a renamed dependency and a target-specific one. |
| C-011 | fix-in-plan. `SimFs::create_dir` fails for a name whose removal is still pending in the parent. Materialise syncs the parent between removing and creating, so a reverted removal recovers an empty directory with no shared state. Test: `create_dir_is_refused_while_its_removal_is_pending`. |
| C-012 | fix-in-plan.<br>• Notes go to `Workspace::notes()`, and `praps` prints them on stderr (`praps_prints_notes`).<br>• INT-0014 now says every command that opens the working copy resumes.<br>• `record_is_durable_before_the_op` asserts the record rename, then the `.praps` sync, then the op's `create_new`. |
