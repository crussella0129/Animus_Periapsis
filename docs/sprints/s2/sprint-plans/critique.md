# Plan Critique — Sprint 2

Round 2. Round 1 blocked with 16 concerns ([critique-round1.md](critique-round1.md)). The reviewer checked each round-1 response against the revised plans and intents and against the code (`store`, `sim`, `xtask/src/policy.rs`, and the vendored `ignore` 0.4.34 `gitignore.rs`).

**Round-1 responses:**
- **Landed and sound:** C-001, C-003, C-004, C-005, C-008 (in part), C-009 to C-014, and C-016 (in part).
- **C-002's race is sound.** `UnsyncedReferences` gives closure violations in about half of the raced cells. `UnsyncedObjects` is still caught, because `syncs_object_data()` already gates the write and the re-sync. The correct store stays at 0.

The concerns are condensed here; the full quotes are in the review transcript.

## Concerns

### C-001: The parent-op re-sync is called "cross-directory" and said to have a negative control; it is neither, and the induction argument leaves out the in-flight window
- **Where:** INT-0007 Consequences; `build-plan.md` T-003 EARS 5 and Notes; `test-plan.md` `commit_resyncs_view_and_parents`
- **Quote:** "`commit_op` re-syncs the view and `objects/` (and each parent op and `ops/`) before writing the op; that re-sync has its own negative control"
- **Failure mode:** intent-drift / missing-risk
- **Why it matters:**
  - Parent ops share `ops/` with the new op, and their re-sync is still needed: a crash between the new op's rename and its directory sync can keep the in-flight op and lose an unsynced parent.
  - No scenario probes the parent re-sync.
  - Same-directory object references are safe because of reachability, not because of masking.
- **Suggested response:** fix-in-plan. Restate the argument through reachability, and probe the parent re-sync or record a deferral.

### C-002: `Workspace::init` has to create and sync `.praps/` and the working-copy root, and the plan does not say so
- **Where:** `build-plan.md` T-005 EARS 1; `store` `finish_init` and `sync_layout`
- **Quote:** "it **SHALL** run `Store::init` on `root/.praps/store`"
- **Failure mode:** hidden-dep / missing-risk
- **Why it matters:** `create_dir` needs `.praps/` to exist, and nothing syncs the `.praps` entry in the root. In strict mode a crash could lose the whole store.
- **Suggested response:** fix-in-plan. Have `init` and `open` create `.praps` and sync it and the root.

### C-003: The policy check for "`getrandom` only in `real.rs`" does not exist, and no task builds it
- **Where:** `test-plan.md` randomness row; `build-plan.md` T-004
- **Quote:** "`cargo policy` `fs-boundary` (`getrandom` only in `real.rs`)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `policy.rs` has no such rule, and T-004 does not touch it.
- **Suggested response:** fix-in-plan. Add the rule and a fixture test, or drop the clause.

### C-004: Several EARS clauses and tests do not map to each other
- **Where:** `build-plan.md` T-003, T-005, T-006; `test-plan.md`
- **Failure mode:** plan-test-mismatch
- **Why it matters:**
  - A damaged view has no test.
  - The mode change after `new` has no status row.
  - The CLI several-heads exit has no test.
  - Snapshot does not say its changeset replaces the old head.
  - `fresh_clone_ci` traces to no clause.
- **Suggested response:** fix-in-plan.

### C-005: The Snapshot negative control cannot build a variant store through the planned API
- **Where:** `build-plan.md` T-005 `from_store`; T-007 EARS 3
- **Quote:** "`Workspace::from_store(store, root)` … build a workspace over an already opened `Store`"
- **Failure mode:** hidden-dep
- **Why it matters:** `init` builds its own store, and `from_store` cannot commit the root op. The seed for minting change IDs is unspecified.
- **Suggested response:** fix-in-plan. Add a constructor that commits the root op on a given store, and name the seed source.

### C-006: The `ignore` composition is feasible, but three details are unpinned
- **Where:** `build-plan.md` T-005 EARS 7 and Notes; the C-015 response
- **Failure mode:** missing-risk
- **Why it matters:**
  - The matcher's prefix-stripping convention is not fixed.
  - `gitconfig_excludes_path` reads the real filesystem and is not banned.
  - `git check-ignore --no-index` knows nothing of tracking, so the corpus needs a second expected column.
- **Suggested response:** fix-in-plan.

### C-007: Only `status` and `new` are said to snapshot, while INT-0014 AC-1 covers every command
- **Where:** `build-plan.md` T-006; `sprint-meta.md` Summary
- **Quote:** "a snapshot at the start of every command"
- **Failure mode:** intent-drift
- **Why it matters:** `fsck` and a repeated `init` do not snapshot, and the exemption is not recorded in the intent.
- **Suggested response:** fix-in-plan. Record the exemptions in INT-0014.

### C-008: Provenance and links are stale after the revisions
- **Where:** the new Work-evidence anchors in six intents; `research-report.md` §1; `sprint-meta.md`
- **Failure mode:** intent-drift
- **Why it matters:**
  - About 15 anchors point at task headings that no longer exist.
  - The research report still puts `new` out of scope.
  - The meta summary omits `new`.
- **Suggested response:** fix-in-plan.

### C-009: The TwoWriters trigger is described two ways
- **Where:** `build-plan.md` T-003 Notes
- **Quote:** "At a seeded point after A's `rename` of the view and before its `sync_dir(objects)`"
- **Failure mode:** EARS-vague
- **Why it matters:** One boundary lies between the two syscalls, so "a seeded point" is fixed. Drawing the trigger over the whole put would race only about 1 seed in 9.
- **Suggested response:** fix-in-plan. Pick one design and assert a race count.

## Confidence
proceed-with-caveats

## Responses

Every concern is fixed in the plans and intents before finalization.

| Concern | Response |
|---------|----------|
| C-001 | fix-in-plan. INT-0007's Consequences and T-003's Notes restate closure through reachability from ops: no op names an object before its put is acknowledged, and unreachable objects with missing references are `fsck` leftovers (`fsck_lists_unreachable_objects_as_leftovers`). The re-sync Consequence names the view re-sync and the same-directory parent re-sync, with the in-flight reason. `UnsyncedReferences` skips both. TwoWriters runs a view race in even seeds and a parent-op race in odd seeds, and `unsynced_references_variant_is_caught` requires closure violations in each kind. |
| C-002 | fix-in-plan. T-005: `init` creates `root` if missing and syncs its parent, then creates `.praps` and syncs `.praps` and `root`, before `Store::init`. `open` makes them durable the same way. Test: `init_makes_praps_dir_durable` (strict mode, every recovery seed). |
| C-003 | fix-in-plan. T-004 now touches `policy.rs` and `sim/clippy.toml`. The `deps` rule rejects `getrandom` outside `sim`. `sim/clippy.toml` bans `getrandom::fill`, `u32` and `u64`, which only `real.rs` may switch off. Tests: `policy_confines_getrandom_to_sim` and a Clippy probe. |
| C-004 | fix-in-plan. Added:<br>• `commit_with_damaged_view_writes_nothing`;<br>• a mode-change row in `status_lists_added_modified_deleted_after_new`;<br>• `praps_status_with_several_heads_fails`;<br>• a T-005 clause that a snapshot's changeset replaces its head, tested by `snapshot_replaces_its_head`.<br>`fresh_clone_ci` traces to INT-0004 AC-1 and INT-0003 AC-2 to AC-4. |
| C-005 | fix-in-plan. `Workspace::init_with_store(store, root, seed)` commits the root op on a given store; the control passes `Store::init_variant`'s store. Every constructor takes a `seed`, and change IDs come from an `Rng` seeded by it: `praps` passes `os_seed()`, tests pass fixed seeds (`change_ids_follow_the_seed`). |
| C-006 | fix-in-plan. Each matcher is rooted at its directory's repository path and always given full repository paths. `gitconfig_excludes_path` joins the banned list. The corpus has two expected columns, Git's matcher verdict and the snapshot outcome; tracking and re-inclusion rows are cross-checked with `git status --ignored`. |
| C-007 | fix-in-plan. INT-0014's Consequences say every command that opens the working copy snapshots first, that `fsck` and a repeated `init` do not, and that AC-1's "second command" means the next command that opens the working copy. T-006's Notes say the same. |
| C-008 | fix-in-plan. Every Sprint 2 anchor in the intents is regenerated from the current headings. The research report gains a revision note: `new` is in scope, and the plans are authoritative. The meta summary names `praps new`. |
| C-009 | fix-in-plan. B fires at a fixed boundary in every seed: after A's view rename (even seeds) or after A's op rename (odd seeds), before A's directory sync. The matrix asserts that both race kinds run in every mode. |
