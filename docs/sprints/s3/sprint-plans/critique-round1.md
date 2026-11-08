# Plan Critique — Sprint 3

Round 1, by a read-only subagent reviewer. The concerns are condensed here. Each keeps its location, failure mode and suggested response; the full quotes are in the review transcript.

## Concerns

### C-001: `undo`'s target is ambiguous: is it the parent of the head before the snapshot or after it?
- **Where:** `build-plan.md` T-005; INT-0009 Consequences
- **Quote:** "the exact view object of that head op's parent"
- **Failure mode:** EARS-vague
- **Why it matters:** One reading only throws away the pending edits and never toggles. The root case is undefined too.
- **Suggested response:** fix-in-plan. H is the head before the snapshot.

### C-002: AC-3 is planned for undo only through "toggles"; there is no undo-per-op-type case
- **Where:** `test-plan.md` INT-0009 AC-3
- **Quote:** "each op of a corpus … is restored"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `undo` is never checked after each op type, and the case of undoing a restore is missing.
- **Suggested response:** fix-in-plan.

### C-003: The `deps` rule as written fails the real workspace
- **Where:** `build-plan.md` T-002
- **Quote:** "depends, directly or transitively, on `getrandom`, `rand` or `uuid`"
- **Failure mode:** missing-risk
- **Why it matters:** Dev-dependencies (`proptest` → `rand`) and `sim` → `getrandom` would reject every boundary crate.
- **Suggested response:** fix-in-plan. Walk only normal dependency edges, and allow edges into `sim`.

### C-004: Removing the record is not made durable, and resume does not check the target against the head
- **Where:** `build-plan.md` T-005
- **Quote:** "finish materialising F → T, if the target op is in the store"
- **Failure mode:** missing-risk
- **Why it matters:** A stale record could come back after a crash and materialise an old tree.
- **Suggested response:** fix-in-plan. Sync `.praps` after removal, and resume only when the target is the head.

### C-005: The NoRecord control may be unable to fail; "user edits the model knows" is undefined
- **Where:** `build-plan.md` T-007
- **Quote:** "holds only user edits the model knows"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The control could pass, and the matrix would prove nothing.
- **Suggested response:** fix-in-plan. Give the oracle an exact expected tree.

### C-006: The case-fold rules break case-only renames, and SimFs cannot model the per-path rule on NTFS
- **Where:** `build-plan.md` T-005, T-003
- **Quote:** "a T path equal under lowercase to an existing entry with another spelling"
- **Failure mode:** missing-risk
- **Why it matters:** `undo` of a case-only rename would be refused, or would lose both files on NTFS. SimFs lookups are exact-case.
- **Suggested response:** fix-in-plan.

### C-007: Lowercase comparison is not conservative for every NTFS fold
- **Where:** `build-plan.md` T-005
- **Quote:** "two paths equal under lowercase"
- **Failure mode:** missing-risk
- **Why it matters:** `σ`/`ς` and `s`/`ſ` slip through.
- **Suggested response:** fix-in-plan. Compare under both lowercase and uppercase.

### C-008: The Windows-name rule blocks undo in repositories Periapsis itself creates, and the reserved-name list is incomplete
- **Where:** `build-plan.md` T-005; INT-0004
- **Quote:** "a reserved device name with any extension and in any case"
- **Failure mode:** intent-drift
- **Why it matters:** Tracked hazardous names, reachable through verbatim paths, would block every `undo`. `CONIN$`/`CONOUT$` and the base and extension rules are missing.
- **Suggested response:** fix-in-plan.

### C-009: The `set_executable` syscall collides with the existing SimFs test helper
- **Where:** `build-plan.md` T-003; `simfs.rs`; workspace tests
- **Quote:** "An unsynced executable-bit change **SHALL** be kept or reverted"
- **Failure mode:** hidden-dep
- **Why it matters:** Method resolution may switch to the new syscall, changing digests and the edit-corpus setup. Durability of the bit is unspecified.
- **Suggested response:** fix-in-plan. Rename the helper, and add a durable bit.

### C-010: `remove_dir` in SimFs: directories have no identity
- **Where:** `build-plan.md` T-003
- **Quote:** "an unsynced `remove_dir` **SHALL** be kept or reverted"
- **Failure mode:** missing-risk
- **Why it matters:** Recovery could panic, or a later `create_dir` could share stale state.
- **Suggested response:** fix-in-plan, or state the restriction and test it.

### C-011: Conflicts between file and directory types and blocked writes are unspecified
- **Where:** `build-plan.md` T-005
- **Quote:** "any other file **SHALL** be left as it is"
- **Failure mode:** EARS-vague
- **Why it matters:** An untracked entry in the way could leave the repository stuck, or make AC-3 equality false.
- **Suggested response:** fix-in-plan.

### C-012: `.praps-tmp-*` is silently excluded from snapshots, which conflicts with INT-0014 and the spec
- **Where:** `build-plan.md` T-005
- **Quote:** "Snapshots **SHALL** never track a `.praps-tmp-*` name."
- **Failure mode:** intent-drift
- **Why it matters:** A user's file with that prefix would be silently lost.
- **Suggested response:** fix-in-plan.

### C-013: Resume in "any constructor" contradicts "init on an existing repository writes nothing"
- **Where:** `build-plan.md` T-005; INT-0014
- **Quote:** "`init` on an existing repository writes nothing"
- **Failure mode:** intent-drift
- **Why it matters:** The intent would go stale, and a corrupt record is unhandled.
- **Suggested response:** fix-in-plan.

### C-014: The edit-corpus model with undo and restore risks reading the state it checks
- **Where:** `build-plan.md` T-007
- **Quote:** "the independent model, which keeps its own op history"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Op ids come from `log`, and the model's rules live outside the files.
- **Suggested response:** fix-in-plan. Predict the op list and assert the count.

### C-015: The hazard trace test conflicts with snapshotting first
- **Where:** `test-plan.md` T-005
- **Quote:** "trace: no write syscall, no record"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The opening snapshot writes when there are edits.
- **Suggested response:** fix-in-plan.

### C-016: Platform-detected case rules: WSL drvfs and case-sensitive NTFS directories
- **Where:** `build-plan.md` T-003
- **Quote:** "both false on Unix"
- **Failure mode:** missing-risk
- **Why it matters:** Over-reports on case-sensitive NTFS directories and misses collisions on drvfs.
- **Suggested response:** defer-with-rationale. Record it in INT-0004, and add a backlog task.

### C-017: T-005 mixes several concerns
- **Where:** `build-plan.md` T-005
- **Quote:** "Hazards, materialise, the working-copy record, and `log`, `undo` and `restore`"
- **Failure mode:** granularity
- **Why it matters:** One task held four separable outcomes.
- **Suggested response:** fix-in-plan. Split it.

## Confidence
block

## Responses

Every concern is addressed in the revised plans and intents before round 2.

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan. T-007 and INT-0009 now define H as the single head at the command's start, before its snapshot, and the target as the view of H's parent. The new op's parent is the head after the snapshot. With no parent, `undo` fails after its snapshot. So a second `undo` toggles. |
| C-002 | fix-in-plan. `undo_after_every_op_type` (T-009, traced to AC-3) covers init, snapshot, new, undo and restore. |
| C-003 | fix-in-plan. T-002 walks only members' normal `[dependencies]` from their manifests and registry crates' dependencies from `Cargo.lock`, and allows edges into `sim`. Fixtures: dev-only `proptest` passes, `sim` → `getrandom` passes, and the real workspace passes. |
| C-004 | fix-in-plan. Materialise ends by syncing `.praps` after removing the record. Resume finishes only when the target op is the current single head. When the target op is absent, or present but not the head, resume drops the record and touches no file, and the second case leaves a note. Test: `resume_drops_a_record_whose_target_is_not_the_head`. |
| C-005 | fix-in-plan. The oracle is exact: after recovery and resume, and before any edit, the tracked working copy equals T if the in-flight op survived, F if it did not, and otherwise the last acknowledged state's files. `NoRecord` must fail in both modes. |
| C-006 | fix-in-plan.<br>• Case-only renames are not hazards: an existing entry that F tracks and T removes is excluded.<br>• Materialise removes before it writes.<br>• T-003 makes `SimFs` lookups case-insensitive when configured, with folding under both cases.<br>• Tests: `case_only_rename_undoes` on `SimFs`, and `real_case_rename_undoes_on_ntfs`. |
| C-007 | fix-in-plan. Names are equal under `to_lowercase` or `to_uppercase`. The corpus adds `σ`/`ς` and `s`/`ſ`. |
| C-008 | fix-in-plan.<br>• Hazards apply only to the paths a write creates or rewrites, so a tracked, unchanged hazardous name does not block `undo` (`unchanged_hazardous_names_are_not_refused`).<br>• The reserved list adds `CONIN$` and `CONOUT$`. The base is the part before the first `.`, with trailing spaces removed.<br>• INT-0004 records the scope. |
| C-009 | fix-in-plan. The existing helper is renamed `make_executable`, and its test call sites are in T-003's Touches. The inode gains a durable bit that `sync_file` updates. The seed is drawn only when a change is pending, so existing digests stay unchanged (asserted). |
| C-010 | fix-in-plan, by restriction. `SimFs::remove_dir` needs an empty directory whose own entry changes are all synced, so a reverted removal is an empty directory: a removed directory has no state to lose. Materialise always syncs a directory before removing it. Test: `remove_dir_then_create_dir_survives_crashes`. |
| C-011 | fix-in-plan. Blocking entries are checked before the record and refused like hazards: an untracked or ignored entry at a T path that F does not track, an entry where T needs a directory, or a directory where T needs a file. Only directories this materialise emptied are removed. AC-3 equality is over the tracked set. |
| C-012 | fix-in-plan. No name is ignored. Temporary names are deterministic, `.praps-tmp-<n>` (the index among the sorted paths to write), and exist only while a record names them; resume removes exactly those. A user's file with the prefix is tracked (`prefix_names_are_tracked`). INT-0014 records it. |
| C-013 | fix-in-plan. `init` and `init_with_store` never resume, so INT-0014's "a repeated `init` writes nothing" stands, and INT-0014 now says so. The record is written atomically; a record that does not decode is a `Corrupt` error. |
| C-014 | fix-in-plan. The model predicts its own op list: the count, and for each op the files with contents, modes and `.gitignore` rules, and the parent. Its count must equal `log`'s length before ops are mapped by position. |
| C-015 | fix-in-plan. `hazards_are_refused_before_any_write` starts from a clean working copy. Both plans say the opening snapshot is the only write. |
| C-016 | defer-with-rationale. INT-0004's Consequences record that the rules come from the platform, with the over- and under-reporting cases. Backlog T-015 adds probe-based detection. |
| C-017 | fix-in-plan. T-005 is split into T-005 (hazards), T-006 (materialise, record, resume, the `variants` policy) and T-007 (`log`, `undo`, `restore`). The later tasks are renumbered T-008 to T-010. |
