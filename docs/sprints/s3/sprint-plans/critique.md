# Plan Critique — Sprint 3

Round 3, by a read-only subagent reviewer. Rounds 1 and 2 blocked ([critique-round1.md](critique-round1.md), [critique-round2.md](critique-round2.md)). Every earlier response landed; round 3 found six narrower gaps, condensed here.

## Concerns

### C-001: Removals in step 1 are not made durable before directories are created in step 3
- **Where:** `build-plan.md` T-006 steps 1–3
- **Failure mode:** missing-risk
- **Why it matters:** An unsynced file removal can revert after a crash while a directory of the same name was created, and `SimFs` refuses `create_dir` while a removal is pending.
- **Suggested response:** fix-in-plan. Step 1 syncs each parent it touched; step 2 is "sync the directory, remove it, sync its parent".

### C-002: No class for paths already at their target, and F-owned misses emptied directories
- **Where:** `build-plan.md` T-006 classification
- **Failure mode:** EARS-vague
- **Why it matters:** After a partial run, paths already equal to T would be classed as user edits and noted, and an emptied directory left by step 1 would block a directory→file swap on resume.
- **Suggested response:** fix-in-plan. Add a "done" class, extend F-owned, and test a crash between steps 1 and 2.

### C-003: Resume has no rule for a path that became a hazard after the crash
- **Where:** `build-plan.md` T-006 resume
- **Failure mode:** missing-risk
- **Why it matters:** A case variant created after the crash would make every command fail.
- **Suggested response:** fix-in-plan. Treat it like a blocking path.

### C-004: Temporary names are left behind when the target is not the head
- **Where:** `build-plan.md` T-006 resume; INT-0014
- **Failure mode:** missing-risk
- **Why it matters:** Leftover `.praps-tmp-*` files would then be snapshotted as user files. Ownership of those names while a record exists is unstated.
- **Suggested response:** fix-in-plan for the removal; defer-with-rationale for ownership, recorded in INT-0014.

### C-005: The crash matrix never crashes during recovery itself
- **Where:** `build-plan.md` T-009
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Resume is idempotence-critical, and a second crash inside it is untested.
- **Suggested response:** fix-in-plan, bounded by seeds.

### C-006: Named fixtures and tests are missing
- **Where:** `test-plan.md` T-002, T-006, T-007; `build-plan.md` T-009
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Round 2's promised fixtures (inherited aliases, `fastrand`, `"name version"` entries), `log` snapshotting first, and a filesystem without an executable bit had no named test; AC-3 compared modes unconditionally.
- **Suggested response:** fix-in-plan.

## Confidence
proceed-with-caveats

## Responses

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan. Step 1 removes paths, then syncs each touched parent; step 2 syncs each directory, removes it, then syncs its parent. |
| C-002 | fix-in-plan. A **done** class (already equal to T, or absent where T has none) is never written or noted. F-owned includes an empty directory where F had files and T has none. Test: `resume_after_removals_finishes_a_directory_to_file_swap`. |
| C-003 | fix-in-plan. A path that blocks or is now a hazard is left as it is and noted; resume finishes the rest and drops the record. `resume_finishes_what_it_can_and_notes_the_rest` adds a case-variant hazard. |
| C-004 | fix-in-plan: when the target is not the head, resume removes the diff's existing temporary names before dropping the record (`resume_with_a_target_that_is_not_the_head_removes_temporaries`). defer-with-rationale: INT-0014 records that those names belong to `praps` while a record exists. |
| C-005 | fix-in-plan. `resume_survives_a_second_crash`: a crash at every syscall of each recovering `open`, 10 seeds per mode, then the same oracle. |
| C-006 | fix-in-plan. T-002 fixtures add an inherited `[workspace.dependencies]` alias, `fastrand` and a `"name version"` lock entry. `log_lists_ops_newest_first` checks the snapshot first. `materialise_without_an_executable_bit_compares_bytes_only` is added, and AC-3 compares modes only where the filesystem has the bit. |
