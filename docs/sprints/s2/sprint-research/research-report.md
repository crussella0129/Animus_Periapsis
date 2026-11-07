# Sprint 2 Research Report

## Intents Reviewed
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — **selected**; relevance: the first user-facing Phase 1 behaviour. Every command scans the working copy and snapshots it into the working-copy change (AC-1), and ignored paths are never snapshotted (AC-2). Writing to the working copy (checkout, conflicts, AC-6), multiple workspaces (AC-3), API-only workspaces (AC-4) and the stat cache (AC-5) come later; current state: proposed
- [INT-0006](../../../intents/INT-0006-format-v1.md) — **selected**; relevance: a snapshot needs the reserved `changeset` (`0x02`) and `view` (`0x04`) kinds, specified before code (AC-6). Both start *provisional*, as `operation` did. The `operation` body gains the view hash in place of its object ids (backlog T-012, in part: actor and signature wait for INT-0016); current state: active
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — **selected**; relevance: once an op names a view instead of listing objects, closure (AC-1) must follow references inside objects. The store must refuse an object whose references are missing, so closure holds by induction, and `fsck` must check the same; AC-5 (four writing processes) stays in the backlog (T-011); current state: active
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — **selected**; relevance: paths are stored relative, `/`-separated and UTF-8, with the same bytes on both systems (AC-2), and a name with no UTF-8 form is reported and left alone (AC-7). Scanning is the first code that meets both. AC-3 and AC-4 (case-fold and Windows path hazards) concern writing a working copy, so they wait for checkout; current state: active
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — **selected**; relevance: the new scan syscall must exist in `SimFs` with the same contract as `RealFs`, and the crash matrix gains a snapshot scenario with a negative control; current state: active
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — selected for backlog task T-010 only: record Sprint 1's GitHub Actions run and the owner's sign-offs. INT-0003's guardrails also cover the new crate and its dependencies; current state: proposed, active, active
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — reviewed, not selected; relevance: each snapshot is a new op over the old view, so earlier working-copy states stay in the op log, and nothing is deleted. `undo` and `op restore` need checkout, so they wait; current state: proposed
- [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) — reviewed, not selected; relevance: two concurrent commands can leave two op heads, and their canonical merge is Phase 2. Until then a command that finds several heads stops and names them, and the limit is recorded. Backlog T-011 (writers in separate processes) pairs naturally with INT-0010 and INT-0014 AC-3 in a later sprint; current state: proposed
- [INT-0013](../../../intents/INT-0013-derived-data.md) — reviewed, not selected; relevance: trees are derived. A snapshot computes the parent's tree from its changesets and compares the scan against it; the tree is never stored; current state: proposed
- [INT-0034](../../../intents/INT-0034-any-content.md) — reviewed, not selected; relevance: snapshots keep each file's bytes exactly, with no normalisation, whatever the content; current state: active

## 1. Sprint Goal
Make Periapsis record a working copy for the first time:
- `praps init [DIR]` creates a repository;
- every command snapshots the working copy into the working-copy change, following `.gitignore` rules;
- `praps status` reports what changed against the change's parent.

What this needs:
- **Format v1:** the provisional `changeset` and `view` kinds, and the `operation` body carrying a view hash.
- **Store:** closure that follows references inside objects.
- **`sim`:** a scan syscall that reports entry kinds, the executable bit, and names with no UTF-8 form.
- **Crash matrix:** a snapshot scenario.

Carry forward T-010 (the Sprint 1 CI record and sign-offs).

Out of scope:
- writing to the working copy (checkout, `new`, `undo`, conflicts);
- multiple workspaces and multi-process writers (T-011);
- merging concurrent op heads (INT-0010);
- the stat cache;
- compression (INT-0008);
- signatures and actors;
- symbolic links, which are reported and left alone, like a name with no UTF-8 form.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| `docs/spec/format-v1.md` | high | `changeset` `0x02` and `view` `0x04` reserved; `operation` provisional with `objects` standing in for the view hash; Change control allows provisional kinds |
| `docs/intents/INT-0006-format-v1.md` | high | The object set names each kind's fields: Changeset (ordered parents, change-ID cache, path map Add/Modify(HashRef, mode) \| Delete \| Rename(from), author, intent refs, description, conflict terms); View (heads, named pointers, per-workspace working-copy change) |
| `crates/codec/src/operation.rs`, `envelope.rs`, `wire.rs` | high | `Operation` encode/decode; `ObjectKind::from_id` rejects reserved kinds; `wire` has uvarint, bytes, str and hashref primitives, plus sets in canonical order |
| `crates/codec/tests/` (golden, properties) | high | Golden corpus and spec-table tests to extend for the new kinds |
| `crates/sim/src/fs.rs` | high | 10 syscalls; `list` returns names only, with no entry kind, mode or non-UTF-8 handling |
| `crates/sim/src/real.rs` | high | `list` lossily converts non-UTF-8 names, which would track a file under a changed name (INT-0004 AC-7 forbids it) |
| `crates/sim/src/simfs.rs` | high | In-memory tree: files and directories only, with no executable bit or symlinks |
| `crates/store/src/lib.rs` | high | `put_object`, `commit_op` (refuses missing parents and objects), `heads`, `fsck`; closure uses `Operation::objects` |
| `crates/store/tests/common/matrix.rs` | high | Scenarios, invariants and variants to extend with a snapshot scenario |
| `crates/praps/src/main.rs` | high | `fsck` only; usage and exit-code conventions (0, 1, 2) |
| `crates/xtask/src/policy.rs` | medium | `fs-boundary` and `names` rules must cover the new crate; `deps` and `credits` cover new dependencies |
| `docs/intents/INT-0014-working-copy-and-workspaces.md` | high | AC-1 (0 lost edits over a randomized edit-and-command corpus, both systems), AC-2 (ignore rules documented and tested) |
| `docs/intents/INT-0004-windows-and-linux.md` | high | AC-2 path encoding, AC-7 non-UTF-8 names |
| `docs/work/tasks.md` | high | Backlog T-010, T-011, T-012 |
| `Cargo.toml`, `deny.toml` | medium | Workspace dependencies; licence allow list (MIT, Unlicense via `OR MIT`, Apache-2.0) |

## 3. External Sources
- [Jujutsu documentation, "Working copy"](https://jj-vcs.github.io/jj/latest/working-copy/) — the working copy is a commit, snapshotted at the start of every command, with `.gitignore` honoured. This is the prior art INT-0014 cites.
- [Git documentation, `gitignore`](https://git-scm.com/docs/gitignore) — pattern semantics: anchoring, `**`, negation and directory-only patterns, and the rule that a file cannot be re-included once its parent directory is excluded.
- [`ignore` 0.4.34, module `gitignore`](https://docs.rs/ignore/0.4.34/ignore/gitignore/) — `GitignoreBuilder::add_line` builds a matcher from lines the caller reads, so no file is read behind the `Fs` boundary, and `matched_path_or_any_parents` applies parent-directory exclusion. Licence: Unlicense OR MIT.
- [Mononoke Bonsai changesets](https://github.com/facebook/sapling/blob/main/eden/mononoke/mononoke_types/src/bonsai_changeset.rs) — the canonical changeset form INT-0006 adopts: ordered parents and a sorted path → change map, with trees derived.
- [`getrandom` 0.4.3](https://docs.rs/getrandom/0.4.3) — operating-system randomness for 128-bit change IDs on stable Rust. `std::random` is still unstable on rustc 1.99.1 (probed here: error E0658). Licence: MIT OR Apache-2.0.

## 4. Risks, Unknowns, Dependencies
- **Format churn.** `changeset`, `view` and the new `operation` body are provisional. They will change again when actors, signatures, rename and conflict terms land, so stores from this sprint are development stores, as INT-0006 says. The golden files make each change visible.
- **Closure through references.** The store must decode each object enough to find its references. Refusing an object whose direct references are absent keeps closure inductive without a transitive walk. `fsck` must check the same thing, and the crash matrix must show that a crash between a changeset and its blobs never leaves an acknowledged op with a missing blob.
- **Non-UTF-8 names.** `RealFs::list`'s lossy conversion would track a file under a changed name. The scan must carry the raw name and report it escaped. On Linux, a byte name that is not UTF-8 can be created in tests. On NTFS an unpaired surrogate needs `OsString::from_wide`, which is Windows-only safe `std` and can be tested on Windows.
- **The executable bit.** Linux has it and Windows does not. On Windows, a snapshot keeps the mode the parent recorded, so a checkout made on Linux does not lose its bits. This is specified and tested on both systems.
- **Several op heads.** Without INT-0010's merge, a command finding two heads must stop and name them, not guess. That contradicts INT-0010's "never fails because of another writer", which is a Phase 2 property, so the gap is recorded in the sprint's technical debt.
- **Scanning cost.** Without the stat cache, every command reads and hashes every tracked file. This is acceptable at test scale. AC-5 and its time budget come with the cache.
- **Racy edits.** With no cache, a file edited during a scan is read either before or after the edit; either result is a correct snapshot of some moment. The race that matters (mtime granularity) arrives with the cache.
- **Dependencies:** `ignore` (and its tree: `globset`, `regex-automata`, `memchr`, `bstr`, `log`, `walkdir`, `same-file`, `crossbeam-deque`) and `getrandom`. `cargo deny` and the `credits` rule must pass. `ignore` would be used only through `add_line` and path matching, never its directory walker, which bypasses `Fs`.

## 5. Recommended Approach
Build in dependency order, each task with EARS acceptance and tests:
1. **T-010:** record Sprint 1's GitHub Actions runs (42329906757 and 42329902655 on `dev`, 42375204698 on `main` after merging) and any owner sign-offs given. Missing ones are carried forward again.
2. **Format v1:** specify `changeset` and `view` provisionally, then implement them, with golden files.
   - `changeset` = `parents: list<hashref>` ‖ `change_id: 16 bytes` ‖ `changes: map<path, entry>` ‖ `description: bytes`, where an entry is `put(blob hashref, mode)` or `delete`. The mode is regular or executable, with symlink reserved.
   - `view` = `heads: set<hashref>` ‖ `workspaces: map<name, changeset hashref>`.
   - `operation` = `parents: set<hashref>` ‖ `view: hashref`.
   - The path encoding is relative, `/`-separated UTF-8. `.`, `..`, empty components, `/`, NUL and backslash are rejected.
   - Golden files cover each kind.
   - Author, intent references, renames and conflict terms are not yet included (T-012, revised).
3. **`codec::references`:** for each kind, the ids it names. The store refuses an object or op whose references are absent, and `fsck` reports missing references through it.
4. **`sim` scan syscall:** a directory's entries with kind (file, directory, symlink, other), executable bit and name (UTF-8, or raw bytes shown escaped).
   - `RealFs` implements it on both systems; `SimFs` models executable bits, symlinks and raw names.
   - Contract tests run on both.
   - `RealFs` also gains OS randomness through `getrandom`.
5. **`workspace` crate:** `init` and `snapshot`, both through `Fs`. The parent tree is derived from changesets.
   - The scan is recursive and sorted. `.gitignore` files are read through `Fs` into an `ignore::gitignore` matcher, and `.praps/` and `.git/` are always ignored.
   - Unrepresentable names and symlinks are reported, never tracked.
   - If the scan differs from the working-copy change, the result is a new changeset with the same change ID, then a new view, then an op.
   - If nothing changed, nothing is written.
   - Several op heads cause an error naming them.
6. **`praps init [DIR]` and `praps status [DIR]`.** `status` snapshots first, then lists added, modified, deleted and reported paths. Exit codes stay 0, 1, 2.
7. **Tests:**
   - The crash matrix gains a snapshot scenario, and a negative control (a changeset put before its blobs) must be caught.
   - A randomized edit-and-command corpus on `SimFs`, and on `RealFs` on both systems, gives INT-0014 AC-1 (0 lost edits).
   - An ignore-rule corpus gives AC-2, and fixture corpora give INT-0004 AC-2 and AC-7.

## Artifacts
- This report is the only artifact. The Sprint 0 master report it builds on is at [master-report-v6.md](../../s0/sprint-research/master-report-v6.md).
