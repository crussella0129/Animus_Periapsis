# Sprint 1 Code Review

Test-phase review of the Sprint 1 build at `3479e31` (T-001 to T-009), run
before the test records were written.

- **Method:**
  1. One independent, read-only reviewer read the diff from `7e84b43` through `git show`. It probed each suspicion in its own scratch clone, with its own target directory, by changing the code and running the real tests.
  2. Each confirmed finding was fixed in `133127a`, with a regression test or probe that fails on the reviewed code.
  3. The two durability-layer findings were re-probed after the fix with `probe_layers.py`: each layer, removed on its own, now makes the correct-store crash matrix fail.
- **Outcome:**
  - 11 findings raised. 10 were fixed; 1 (the process-crash model, finding 4) was answered by design, with its remaining gap recorded as a known limit.
  - Findings 1, 2, 3, 5 and 6 were reproduced by the reviewer before the fix.
- **Fix commit:** `133127a`.
- **Plan deviation:** the build plan said lenient mode makes a synced file's own entry durable. Finding 11 showed that promoting that one entry alone could leave an older change to the same name pending, which a crash could reapply. Lenient mode now commits every entry change of the directory up to that one, in order, as a journaling filesystem does. Strict mode, which the store must pass and the negative controls are judged in, is unchanged.

## Findings

| ID | Severity | Where | Finding | Disposition | Regression test or probe |
|----|----------|-------|---------|-------------|--------------------------|
| 1 | medium-high | `crates/store/tests/common/matrix.rs` (TwoWriters) | Writer B put X itself before committing. When X was present, `put_object`'s own sync covered it, so deleting all of `commit_op`'s re-syncs still left 0 violations. The INT-0007 "another writer renamed without syncing" protection had no test, and the scenario comment named the wrong code. | fixed `133127a`: B, opened before the hook is armed (`open` syncs the layout), commits an op naming X without putting it. | Probe P1, `commit_op`'s object re-sync removed: 9 closure violations (5 lenient, 4 strict). |
| 2 | medium | `matrix.rs` `check` | Durability was checked for acknowledged ops only. `put_object`'s acknowledgement was never checked, and `commit_op`'s re-sync masked any gap. | fixed `133127a`: `Log` records each acknowledged object, and a missing one is a durability violation. | Probe P2, `put_object`'s `sync_dir(objects/)` removed: 1,953 durability violations (777 lenient, 1,176 strict). |
| 3 | medium | `crates/xtask/src/policy.rs` `FS_OPT_OUTS` | `#![allow(warnings)]` in `store/src`, or a Cargo `[workspace.lints]`/`[lints]` entry for the lints or their groups, switched the boundary off while `cargo policy` passed. | fixed `133127a`: `warnings` joins the opt-outs; any `[workspace.lints]` or boundary-crate `[lints]` setting of the two lints, their old names, `style`, `all` or `rust.warnings` fails `fs-boundary`. | `policy_enforces_the_fs_boundary`: the `#![allow(warnings)]` fixture and four Cargo-lints fixtures. |
| 4 | medium | `crates/store/src/lib.rs` `open` | A process that died inside `init` after creating the layout but before syncing leaves directories that exist but whose entries are not durable. `open` accepted them, and acknowledged ops could then vanish with them at a later power loss. The matrix models only whole-system crashes, so it cannot show this. | fixed by design `133127a`: `open` syncs `objects/`, `ops/`, the root and its parent, as `init` does. The gap in the simulated model is recorded as a known limit in `docs/spec/storage-durability.md`. | `open_syncs_the_layout`: a layout created unsynced, opened, an op acknowledged, then recovered across 50 strict seeds — the op is always present. |
| 5 | medium | `lib.rs` `init` | For a relative root (`repo` or `.`), `parent()` is the empty path, so `RealFs::sync_dir("")` failed with NotFound. The simulator, rooted at `""`, accepted it. | fixed `133127a`: the parent of a bare relative root is `root/..`; the simulator resolves `..` lexically. | `relative_roots_work_on_the_real_filesystem` (`repo` and `.`); `parent_components_are_lexical`. |
| 6 | medium-low | `policy.rs` `deps` | `[features] default = ["store/variants"]` in another member, `default = ["variants"]` in the store, or a renamed workspace key with `package = "store"` turned the variants on in production while `deps` passed. | fixed `133127a`: `deps` checks every member's features for `store/variants` and `store?/variants`, the store's own features, and every workspace key that names the store. | `policy_rejects_variants_outside_dev_dependencies`: three feature fixtures and a renamed-key fixture. |
| 7 | low | `policy.rs` `fs_boundary` | Boundary crates were found by their directory, so moving one dropped it from the rule. | fixed `133127a`: matched by package name. | `policy_enforces_the_fs_boundary`, the `crates/core/store` fixture. |
| 8 | low | `policy.rs` `rust_files` | Every directory named `target` was skipped, so `src/target/mod.rs` was never scanned. | fixed `133127a`: only the crate's own `target/` is skipped. | `policy_enforces_the_fs_boundary`, the `src/target/mod.rs` fixture. |
| 9 | low | `crates/sim/src/real.rs` `list` | A non-UTF-8 file name failed the whole listing, making `fsck` and `heads` unusable instead of reporting a bad name. | fixed `133127a`: names are listed lossily, so such a name parses as no id and `fsck` reports it as a bad name. | Not testable on NTFS, which cannot hold such a name; reviewed. |
| 10 | low | `matrix.rs` `run_cell` | `check`'s final `init` ran on the recovered state before it was hashed and handed to `fsck_is_clean_on_matrix_stores`, so both saw a repaired state. | fixed `133127a`: the checks run on a second recovery, which is equal by construction; the state hashed and handed on is the one the crash left. | `fsck_is_clean_on_matrix_stores` now runs on raw crash states (16,256 that open). |
| 11 | low | `crates/sim/src/simfs.rs` | Directory ops matched by name, not inode. A pending `Remove` or an earlier `Insert` of a reused name could reapply after a later file under that name became durable: an impossible, pessimistic state. | fixed `133127a`: `Remove` and `Rename` act only on their own inode, and lenient promotion commits the directory's changes in order (see Plan deviation). | `stale_removals_spare_a_reused_name`: fails on the reviewed model, which recovered the old file. |

## Known limits

| Severity | Where | Limit | Why it stands |
|----------|-------|-------|---------------|
| medium | `sim` crash model | A process crash with the system still running (unsynced changes visible to the next process) is not simulated. | The store answers it by design: `open` syncs the layout, and every write re-syncs what it depends on. A matrix mode that continues on un-recovered state is future work for INT-0007 AC-5 (multi-process writers). |
| low | INT-0007 AC-5 | Writers in separate processes are not tested. | Deferred by the plan; two writers in one process are tested, interleaved at every syscall. |
| low | `RealFs` on Windows | `FlushFileBuffers` on a directory handle is not documented by Microsoft. | Recorded in `docs/spec/storage-durability.md`; INT-0004 AC-5 is an `asserted` criterion for the reviewer. |
