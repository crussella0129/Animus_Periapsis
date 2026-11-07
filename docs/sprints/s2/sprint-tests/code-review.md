# Sprint 2 Code Review

An independent, read-only reviewer (a subagent) reviewed the Sprint 2 diff `29438ce..e243177` (crates and `docs/spec`) against the locked plans, the specs and the intents. It built and ran the suites and confirmed findings with throwaway probes in a separate worktree, which it removed. Fifteen findings: seven medium, eight low. All are fixed or answered below, each with a regression test where code changed.

## Findings and responses

| Id | Severity | Finding | Response |
|----|----------|---------|----------|
| CR-01 | medium | Without an executable bit, a tracked file's mode came only from the working-copy tree, not "or else its parent" as the spec says. A file deleted then restored on Windows lost its executable mode, and `status` showed a spurious `M`. The edit-corpus oracle copied the bug. | Fixed. The mode falls back to the parent tree. `no_executable_bit_restores_the_parents_mode` (delete, restore, no change, mode kept). The oracle now applies the spec rule (working-copy change or else parent). |
| CR-02 | medium | On Windows, a real NTFS name such as `foo.`, `foo ` or `aux.txt` was read through Win32 normalisation, which reads `foo` or a device. Its content was silently replaced. | Fixed in `RealFs`. On Windows every path becomes a verbatim `\\?\` path: absolute, with names untouched and `..` taken lexically. `real_fs_keeps_names_win32_would_rewrite` (`foo`, `foo.`, `foo `, `aux.txt`, `CON`, `nul.c`, each read back as itself, on NTFS and ext4). |
| CR-03 | medium | A UTF-8 byte-order mark at the start of `.gitignore` became part of the first pattern. Git strips it. | Fixed: stripped before parsing. Corpus case `bom`, with Git's verdict recorded. |
| CR-04 | medium | A first snapshot was quadratic in the number of files (`tree.values().any(…)`, and a tracked-path scan for every directory). Every file's content was held at once. | Fixed. Each unknown blob is put directly. Tracked paths below a directory are found by one `BTreeMap::range` probe, only for ignored directories. Content is kept only for blobs not already in the store. Holding a changed large file whole is left to INT-0008 AC-6 (streaming), as before. |
| CR-05 | medium | A file deleted between the listing and its read aborted the snapshot. On Unix the same race existed in `RealFs::scan`'s metadata call. | Fixed. `NotFound` on the read, or on the entry's type or metadata, counts as deleted. The spec states the rule. `a_file_vanishing_mid_scan_is_deleted`. |
| CR-06 | medium | The root's parent was synced only when `init` created the root. A process that died between creating the root and that sync left a root whose entry might never become durable. | Fixed. `init` and `open` sync the root's parent every time. `init_and_open_sync_the_roots_parent`. |
| CR-07 | medium | The `getrandom` confinement missed aliases in `[workspace.dependencies]` and `getrandom::fill_uninit`. Other routes to OS randomness (`std::hash::RandomState`, a transitive `rand`) were not covered. | Fixed for aliases, which are resolved like the store's, and `fill_uninit`, which is banned in every boundary crate. `policy_confines_getrandom_to_sim` adds the alias case. Deferred: `RandomState` and transitive randomness, recorded as technical debt in the test report. No current dependency of a boundary crate draws OS randomness except through `sim::os_seed`. |
| CR-08 | low | A reported path escaped only its last component, so a directory name holding a control character could put a raw control character into `status` output. | Fixed. Every component is shown escaped. `reported_paths_are_escaped_in_full`. |
| CR-09 | low | `praps fsck DIR` fell back to checking `DIR` as a bare store when `DIR/.praps/store` failed to open, hiding the real damage. | Fixed. When `DIR` holds `.praps`, `fsck` checks `.praps/store` and reports its errors. Covered by `praps_fsck_checks_a_working_copy` and the Sprint 1 bare-store tests. |
| CR-10 | low | `Store::ops` skips damaged op files silently. A damaged head let its parent become the head, and a store whose ops were all damaged looked uninitialised, so `init` would start again. | Fixed. The workspace refuses (`Corrupt`, pointing to `praps fsck`) when `ops/` holds an op file that does not hash or decode, in `open`, every command, and `init`. `invalid_op_files_are_refused`. |
| CR-11 | low | Reference kinds were not checked, so a view could name a blob as its head. The store and `fsck` accepted this, and the workspace failed later. | Fixed. `codec::typed_references` gives the kind each reference must be. `put_object` refuses a reference of the wrong kind, and `fsck` reports it as a missing reference. `references_of_the_wrong_kind_are_refused`. INT-0007 records it. |
| CR-12 | low | `SimFs` accepted a NUL in a name, which could collide with raw-name keys, and accepted "raw" names that have a UTF-8 form. | Fixed. Both are refused. `simfs_refuses_nul_names_and_false_raw_names`. |
| CR-13 | low | `.praps` and `.git` at the top were matched case-sensitively, so `.Git` was snapshotted. | Fixed. Matched ignoring ASCII case on every system, and the spec says so. `always_ignored_names_ignore_case`. |
| CR-14 | low (docs) | The closure argument's premise ("no op can name an object before its put is acknowledged") is false for two writers: a second writer's `put_object` can name a first writer's renamed but unacknowledged object. | The property holds, for other reasons. `storage-durability.md` and INT-0007 restate the argument on data-before-names and the shared `objects/` directory sync. The `put_object` reference race is named as covered by that sync rather than by a scenario of its own. |
| CR-15 | low (tests) | The mode oracle mirrored the implementation. Real-filesystem mode preservation was never exercised on Windows. The ignore corpus lacked BOM, trailing-space and directory-whitelist cases. | Oracle fixed with CR-01. Corpus extended to 18 cases and 47 paths (`bom`, `trailing-space`, `whitelist-dir`), all agreeing with Git. Windows real-filesystem mode preservation stays covered by the simulator (`no_executable_bit_*`, half the edit corpus with no bit). The test report records that the real corpus cannot set a bit on NTFS. |

## Plan deviation

T-005 planned each `.gitignore` matcher "rooted at that file's directory's repository path". The `ignore` crate strips its root as a byte prefix, so a root of `a` would also strip from `ab/x`. Each matcher is therefore rooted at `.` and given paths relative to its own directory. Precedence, scope and case sensitivity are as planned, and `docs/spec/working-copy.md` states the convention.

## Verified sound by the reviewer

- **Codec:**
  - canonical decoding of changesets, views, operations and paths: every accepted body re-encodes identically;
  - `references` covers every id.
- **Store:**
  - `put_object` and `commit_op` checks and re-syncs;
  - `fsck` reachability and leftovers;
  - no case in which an op is durable while something it reaches is lost (single writer and both races, strict and lenient).
- **Negative controls:** each is non-vacuous. Both race kinds fire, and each control produces its violations.
- **Display:** `Name::display` is injective for genuine raw names.
- **`SimFs::recover`:** keeps raw, symlink and special entries and executable bits.
- **`RealFs::scan`:** does not follow symlinks.
- **Ignore precedence:** innermost file first, last match wins, case-sensitive, CR stripped.
- **Ignored directories:** tracked paths under them are visited.
- **Exit codes:** 0, 1 and 2.
- **`ignore` crate:** the bans on its filesystem readers are complete.
