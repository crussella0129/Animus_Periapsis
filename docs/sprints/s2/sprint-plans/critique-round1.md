# Plan Critique — Sprint 2

Round 1, by a read-only subagent reviewer. The concerns are condensed here. Each keeps its location, failure mode and suggested response; the full quotes are in the review transcript.

## Concerns

### C-001: T-002 breaks the build, and nothing repairs it until T-003 and T-006
- **Where:** `build-plan.md` T-002 Touches and Depends on; T-003 Touches; T-006 Touches
- **Quote:** T-002 "**Depends on:** (none)" while replacing `objects` in the operation body
- **Failure mode:** hidden-dep
- **Why it matters:** `Operation::objects` is used across `store` (lib, fsck, tests, `real_store.rs`, the matrix) and in `praps/tests/cli.rs`. After T-002 the workspace would not compile until T-003, and `cli.rs` would stay broken until T-006.
- **Suggested response:** fix-in-plan. Move the operation change into the task that ports its users, and list `real_store.rs` and `cli.rs`.

### C-002: The `UnsyncedReferences` control cannot fail for references in the same directory
- **Where:** `build-plan.md` T-003 EARS 3 and 6; INT-0007 Consequences
- **Quote:** "**WHEN** the `UnsyncedReferences` variant skips re-syncing present references, **THEN** the TwoWriters scenario **SHALL** report closure violations"
- **Failure mode:** plan-test-mismatch (vacuous control)
- **Why it matters:** Blobs, changesets and views share `objects/`. The new object's own `sync_dir(objects)` commits every pending entry, so a same-directory re-sync is masked. Only the op → view reference crosses directories. The TwoWriters redesign and its race trigger were not specified.
- **Suggested response:** fix-in-plan. Specify TwoWriters with B naming A's pending view, recompute the trigger, and drop the same-directory re-sync or document it.

### C-003: The `UnsyncedObjects` control may go vacuous because of the new reference re-sync
- **Where:** `build-plan.md` T-003; T-007 EARS 2
- **Quote:** "Snapshot scenario runs with the `UnsyncedObjects` variant … **SHALL** report violations"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** If the new re-syncs are not gated by the variant, every object becomes durable before the op, and the control finds nothing.
- **Suggested response:** fix-in-plan. Define each variant against the new code paths, with expected violation kinds.

### C-004: The edit-corpus oracle contradicts the rule that a tracked file stays tracked once ignored
- **Where:** `build-plan.md` T-007 EARS 3 versus T-005 EARS 4
- **Quote:** "**SHALL** equal the working copy's non-ignored regular files"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** A correct implementation would fail the oracle when an ignore rule covers a tracked file. The Windows mode rule is not in the oracle either.
- **Suggested response:** fix-in-plan. Use tracked-before ∪ non-ignored, with per-OS modes.

### C-005: `status` can never report M or D this sprint, so its tests are vacuous
- **Where:** `build-plan.md` T-005 EARS 7, T-006; `test-plan.md` status tests and E2E
- **Quote:** "against its parents' merged tree (empty for a root change)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** With only the root change, every path is `A`. No real code produces a `delete` entry.
- **Suggested response:** fix-in-plan. Test M and D with a child change, or change what status compares against.

### C-006: A crash inside `Workspace::init` can leave a repository that cannot be recovered, and no invariant checks it
- **Where:** `build-plan.md` T-005 EARS 1, T-007 EARS 1
- **Quote:** "run again, it **SHALL** report the existing repository and write nothing"
- **Failure mode:** missing-risk
- **Why it matters:** A crash after the layout exists but before the root op leaves zero heads, and `init` would refuse to finish it. Recency is undefined with zero heads.
- **Suggested response:** fix-in-plan. Make idempotence depend on a root op, define recency for these cells, and add a liveness check.

### C-007: Nested `.gitignore` semantics need a design that `ignore` 0.4.34 does not provide through `add_line`
- **Where:** `build-plan.md` T-005 EARS 4 and Notes; research §3
- **Quote:** "any `.gitignore` in the working copy, applied to its directory and below"
- **Failure mode:** missing-risk
- **Why it matters:**
  - One builder has one root, so nesting and precedence need composing by hand.
  - `matched_path_or_any_parents` panics outside its root.
  - Tracked files under ignored directories need scanning.
  - Git on Windows defaults to `ignorecase=true`.
- **Suggested response:** fix-in-plan. State the composition, add corpus rows, and pin case sensitivity.

### C-008: The Workspace API cannot select a broken store variant under the `deps` policy
- **Where:** `build-plan.md` T-005, T-007; `policy.rs` `deps`
- **Quote:** "Snapshot scenario runs with the `UnsyncedObjects` variant"
- **Failure mode:** hidden-dep
- **Why it matters:** `workspace` cannot forward `store/variants`. The tests need a constructor taking a built `Store`, and the matrix sharing method is unspecified.
- **Suggested response:** fix-in-plan. Add a store-injecting constructor, and say how the harness is shared.

### C-009: T-004 changes to SimFs ripple into the store matrix and the cross-host digests
- **Where:** `build-plan.md` T-004; `simfs.rs` `Node`; `matrix.rs` `digest`
- **Quote:** "`SimFs` … **SHALL** model executable bits, symlinks and raw names"
- **Failure mode:** hidden-dep
- **Why it matters:**
  - New node kinds break `matrix.rs`'s exhaustive match.
  - Raw names cannot be `&Path` on every host.
  - A `cfg!(windows)` executable rule would make the digests differ by host.
- **Suggested response:** fix-in-plan. Touch `matrix.rs`, specify a host-independent raw-name model, and add an executable-bit capability.

### C-010: The `Name::Raw` escape guarantee is false as stated
- **Where:** `build-plan.md` T-004 EARS 1 and Notes
- **Quote:** "an escaped form that is never a valid UTF-8 name of another file"
- **Failure mode:** EARS-vague
- **Why it matters:** The UTF-8 name `a\xffb` displays the same as the raw bytes `a 0xff b`.
- **Suggested response:** fix-in-plan. Escape `\` in every displayed name, and test the collision.

### C-011: Windows feasibility of the real-filesystem tests for symlinks and other entries
- **Where:** `test-plan.md` `real_scan_meets_the_contract`, `snapshot_reports_and_leaves_raw_names`
- **Quote:** "the same contract on RealFs in a temporary directory, on Windows and Linux"
- **Failure mode:** missing-risk
- **Why it matters:** Windows symlinks need a privilege, and FIFOs do not exist on NTFS. Unpaired surrogates are feasible.
- **Suggested response:** fix-in-plan. Make the symlink case conditional and reported as skipped, and run `Other` on Unix only.

### C-012: The INT-0004 AC-2 proxy does not test host independence
- **Where:** `test-plan.md` traceability, INT-0004 AC-2
- **Quote:** "`path_golden_files_match` (run on Windows and Linux)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** A constant path encodes the same everywhere by construction. The risk is turning OS paths into repository paths.
- **Suggested response:** fix-in-plan. Snapshot a fixed tree on NTFS and ext4 and compare changeset ids.

### C-013: The Windows executable-bit rule refers to a parent that does not exist this sprint
- **Where:** `build-plan.md` T-005 EARS 6; INT-0014 Consequences
- **Quote:** "keep the mode its parent tree recorded"
- **Failure mode:** intent-drift
- **Why it matters:** Read literally, the rule makes every Windows file regular.
- **Suggested response:** fix-in-plan. Use "the working-copy change, or else its parent".

### C-014: AC-9 is cited for `praps`, which it does not cover
- **Where:** `build-plan.md` INT-0003 entry and T-006
- **Quote:** "INT-0003 AC-9 (no flags on idempotent commands)"
- **Failure mode:** intent-drift
- **Why it matters:** INT-0003 AC-9 names only `cargo ci`, `cargo policy` and `cargo book`, and `status` writes snapshots.
- **Suggested response:** fix-in-plan. Drop the citation, or revise an intent.

### C-015: The new filesystem boundary can be bypassed through `ignore`
- **Where:** `build-plan.md` T-005; `policy.rs` `FS_BOUNDARY_CRATES`
- **Quote:** "`add_line` is fed lines read through `Fs`"
- **Failure mode:** missing-risk
- **Why it matters:** `GitignoreBuilder::add`, `Gitignore::new` and `global`, and `WalkBuilder` read the real filesystem, and nothing bans them.
- **Suggested response:** fix-in-plan. Ban them in `clippy.toml`, and extend the policy fixture.

### C-016: Some research risks and EARS clauses have no planned test, or a test has no clause
- **Where:** research §4; `test-plan.md` traceability
- **Quote:** "the gap is recorded in the sprint's technical debt"
- **Failure mode:** missing-risk / plan-test-mismatch
- **Why it matters:**
  - Scanning cost and racy edits vanished, and the reference check re-reads blobs.
  - INT-0010 does not note the several-heads gap.
  - Four tests are missing from the table.
  - "Distinct error" conflicts with `UnsortedOrDuplicateKey`.
- **Suggested response:** fix-in-plan. Add deferrals, the INT-0010 note, the table rows, and an error clarification.

## Confidence
block

## Responses

Every concern is fixed in the plans and intents before round 2.

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan. T-002 now specifies only `changeset`, `view` and `path`, and leaves `operation` unchanged, so the workspace stays green. T-003 changes the `operation` body in the same commit as its spec, golden files and every user. Its Touches list `store` (lib, fsck, variant, all tests, `real_store.rs`, the matrix) and `praps/tests/cli.rs`. |
| C-002 | fix-in-plan. `put_object` checks references but does not re-sync them, because they share `objects/` and the object's own directory sync covers them. INT-0007's Consequences now say so. `commit_op` re-syncs the view, `objects/` and parents across directories. `UnsyncedReferences` skips only that re-sync. TwoWriters is redesigned: B commits an op naming A's view after A's rename and before A's directory sync, without putting the view. The trigger window is recomputed, and races > 0 is still asserted. |
| C-003 | fix-in-plan. `UnsyncedObjects` suppresses every data sync of an object file, its own and `commit_op`'s re-sync, and is expected to give durability and torn violations. `UnsyncedReferences` suppresses only the cross-directory re-sync and is expected to give closure violations. |
| C-004 | fix-in-plan. The oracle (T-007, both corpora) is the regular files that are not ignored or were tracked before, with modes under the filesystem's executable-bit rule. |
| C-005 | fix-in-plan. `praps new` is added (T-005, T-006). It starts a child change without writing files, so `status` compares against a real parent, `M` and `D` occur, and `delete` entries are produced. Tests: `new_starts_a_child_change`, `status_lists_added_modified_deleted_after_new`, `praps_new_prints_a_change_id`, and an E2E step after `new`. INT-0014's Consequences record `new`. |
| C-006 | fix-in-plan. `init` writes the root op if the store has no op, and reports an existing repository only if one exists, so it completes an interrupted init. `open` with no op fails clearly. Snapshot recency allows no op only while init is unacknowledged. A liveness invariant requires `init` then `snapshot` to succeed on every recovered state. Tests: `init_completes_an_interrupted_init`, `open_without_ops_fails`. |
| C-007 | fix-in-plan. T-005 specifies the matching rules:<br>• one matcher per `.gitignore`, rooted at its directory;<br>• the deepest match decides, and the last match within a file;<br>• matching is case-sensitive on every system;<br>• an ignored directory is entered only for tracked paths, and nothing under it can be re-included.<br>The corpus records Git's verdicts with `core.ignorecase=false` and `--no-index`, and adds rows for a tracked file under an ignored directory, `!` under an excluded parent, and a nested override. INT-0014's Consequences record the rules. |
| C-008 | fix-in-plan. `Workspace::from_store` takes an opened `Store`. The workspace tests enable `store/variants` through a dev-dependency, and the crash harness is written in `workspace/tests/` against `sim`'s public crash API, without a `#[path]` include. |
| C-009 | fix-in-plan. T-004 touches `matrix.rs` (the digest covers the new node kinds) and depends on T-003. Raw names are stored in `SimFs` as bytes or code units, the same on every host. `Fs::has_executable_bit()` replaces any `cfg!` in the workspace, and `SimFs` defaults to `true` on every host, so digests stay host-independent. |
| C-010 | fix-in-plan. `Name::display` escapes `\` and control characters in every name, plus invalid bytes or surrogates in raw names, so a raw display never equals a UTF-8 display. `raw_display_never_equals_a_utf8_display` covers the `a\xffb` case. |
| C-011 | fix-in-plan. On Windows the symlink case reports skipped when the privilege is missing, and `Other` entries are tested on Unix only. |
| C-012 | fix-in-plan. INT-0004 AC-2 is now verified by `real_snapshot_id_is_host_independent`: the same nested tree, snapshotted with a fixed seed on NTFS and ext4, must give one recorded changeset id. |
| C-013 | fix-in-plan. INT-0014 and T-005 now say "the mode the working-copy change, or else its parent, last recorded". |
| C-014 | fix-in-plan. The AC-9 citation is dropped. T-006's Notes record that `praps`'s no-flag convention is not an AC-9 command and that its intent home is INT-0022, which is not advanced this sprint. |
| C-015 | fix-in-plan. `fs-boundary` requires a boundary crate that depends on `ignore` to ban `GitignoreBuilder::add`, `Gitignore::new`, `Gitignore::global`, `GitignoreBuilder::build_global`, `WalkBuilder` and `Walk` in its `clippy.toml`. This is tested by the policy fixture and a Clippy probe. |
| C-016 | fix-in-plan. INT-0014's Consequences defer the scanning cost, the blob re-reads of the reference check and the mtime race to the stat cache. INT-0010's Consequences record the several-heads gap. The traceability table now lists `os_seed_varies`, `scan_lists_kinds_sorted`, `real_scan_reports_the_executable_bit` and `status_lists_changes_sorted`. T-002 names the errors: `UnsortedOrDuplicateKey` for order and duplicates, and new named errors for tags, modes and paths. |
