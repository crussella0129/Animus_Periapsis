# Sprint 3 Research Report

## Intents Reviewed
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — **selected**; current state: proposed. Relevance: Periapsis records history but cannot go back. The intent's user-facing promises need the working copy to be written:
  - `undo` and `op restore` return the views and the working copy to an earlier state (AC-3);
  - every command that overwrites the working copy snapshots it first (AC-2).

  Injectivity (AC-1) holds by construction while ops only append. AC-4 (signed erasure) and AC-5 (partial clones) need actors and sync, so they wait.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — **selected**; current state: active. Relevance: writing the working copy (materialising a tree) is the half of the working-copy model that Sprint 2 left out. AC-1 must keep holding when commands also write files. AC-3 to AC-6 need several workspaces or conflicts, so they wait.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — **selected**; current state: active. Relevance: materialising is where the platform hazards bite:
  - AC-3: case-fold collisions are detected on a case-insensitive filesystem, and neither file is overwritten;
  - AC-4: names Windows cannot represent (reserved names, trailing dots or spaces, forbidden characters) produce an error naming the path, never a silent rename or partial write.
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — **selected**; current state: active. Relevance: a crash during a materialise must leave a state the next command recovers from without losing an acknowledged op or an uncommitted edit. That needs a crash scenario and a negative control. Backlog T-014 (the remaining randomness bans) belongs to INT-0005.
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md) — selected for backlog T-013 only: record Sprint 2's GitHub Actions run and the owner's sign-offs; current state: proposed and active.
- [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md), [INT-0011](../../../intents/INT-0011-change-identity-and-rewriting.md) — reviewed, not selected; current state: proposed. Relevance: merging concurrent heads and rewriting changes are Phase 2. Writers in separate processes (T-011) pair with INT-0010's merge, so they stay in the backlog. `undo` is defined over a single head, as every command is now.

## 1. Sprint Goal
Make history usable going back:
- **`praps log`** lists the op log.
- **`praps undo`** returns the repository and the working copy to the state before the last op.
- **`praps restore OP`** returns them to the state any earlier op left.

Each of these writes the working copy to match the target view, after first snapshotting it. Materialising must:
- be crash-safe, finishing or resuming after a crash;
- detect the INT-0004 hazards before writing anything.

Carry forward T-013 (the Sprint 2 CI record and sign-offs) and T-014 (the randomness bans).

Out of scope:
- checking out another change (`edit`, INT-0011);
- conflicts (INT-0012);
- several workspaces and processes (T-011);
- signed erasure;
- the stat cache;
- compression.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| `docs/intents/INT-0009-reversible-by-default.md` | high | AC-2 (200 pairs, 0 collapsed), AC-3 (exact earlier state for every op type) |
| `docs/intents/INT-0004-windows-and-linux.md` | high | AC-3 case-fold corpus, AC-4 hazard corpus, 100% |
| `docs/spec/working-copy.md` | high | The snapshot sequence; nothing yet writes the working copy |
| `crates/workspace/src/lib.rs` | high | `load`, `snapshot`, `new_change`, `replace_working_copy`; trees derived via `tree_of` |
| `crates/workspace/tests/crash.rs`, `tests/common/mod.rs` | high | The Snapshot crash scenario and the edit-corpus model to extend with undo and restore |
| `crates/sim/src/fs.rs`, `real.rs`, `simfs.rs` | high | `Fs` has create, write, overwrite, rename, remove, scan, but no directory removal and no case-sensitivity capability |
| `crates/store/src/lib.rs` | medium | `ops()`, `heads()`, `get_object`; ops are append-only, with no delete API (INT-0009 AC-4) |
| `crates/praps/src/main.rs` | high | The command dispatch, exit codes and no-flag convention |
| `crates/xtask/src/policy.rs` | medium | `randomness` bans (T-014): `RandomState`, transitive `rand` and `uuid` |
| `docs/work/tasks.md` | high | T-011 to T-014 |

## 3. External Sources
- [Jujutsu, "Operation log"](https://jj-vcs.github.io/jj/latest/operation-log/): `jj undo` is itself a new operation, which restores the view of the undone op's parent, so undo can be undone. `jj op restore` restores any earlier view. Periapsis follows this: the op log stays append-only (INT-0009 AC-1).
- [Jujutsu, "Working copy" (stale working copy)](https://jj-vcs.github.io/jj/latest/working-copy/): jj records which operation the files on disk reflect. When a command finds the working copy behind the current op (an interrupted update, or a concurrent op), it updates the files before going on. This is the recovery model for an interrupted materialise.
- [Microsoft, "Naming files, paths, and namespaces"](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file): the forbidden characters `<>:"/\|?*` and 0–31; reserved names `CON`, `PRN`, `AUX`, `NUL`, `COM0`–`COM9`, `LPT0`–`LPT9` and superscript-digit forms, alone or with any extension; names may not end in a space or a period. This is the AC-4 hazard set.
- [Git, `core.protectNTFS` and checkout collision reporting](https://git-scm.com/docs/git-config#Documentation/git-config.txt-coreprotectNTFS): Git refuses paths that NTFS would alias. On a case-insensitive filesystem it warns "the following paths have collided" after the checkout. Periapsis must detect collisions *before* writing, so neither file is silently overwritten (AC-3).
- [Unicode case folding (`CaseFolding.txt`)](https://www.unicode.org/Public/UCD/latest/ucd/CaseFolding.txt): NTFS compares names through its own upcase table, close to simple case folding. Rust's `to_lowercase` is close to that. Exact NTFS equivalence is out of reach without the volume's `$UpCase` table, so the check is conservative: it reports any pair equal under full lowercase.

## 4. Risks, Unknowns, Dependencies
- **An interrupted materialise.** Files are written one by one, so a crash leaves a mix of the old and new trees.
  - The plan is a working-copy record in `.praps/`, written durably before any file changes. It is local working-copy state, not a pointer in the store, which INT-0007 forbids.
  - The record holds the op whose tree the files reflect, and the target if an update is in flight.
  - The next command finishes an in-flight update before snapshotting, so the mix is never recorded as user edits.
  - This needs its own crash scenario and a negative control (a materialise without the record).
- **Snapshot before overwrite** (INT-0009 AC-2): every command that writes files snapshots first. `undo` and `restore` therefore always add the snapshot op before the restore op. Undoing a restore needs an undo of the snapshot too, so `undo`'s target is defined explicitly in the spec.
- **What `undo` undoes.** jj restores the view of the last op's parent. A snapshot op created by `undo` itself then becomes "the last op", so the spec must name which op `undo` steps back from. Proposal: `undo` restores the view of the parent of the newest op that is not a snapshot made by this `undo`; a second `undo` steps back once more. This needs a precise definition and tests.
- **Case-insensitivity detection.** NTFS directories can be made case-sensitive per directory, so assuming insensitivity on Windows is conservative: it may report a collision that would not happen there. A capability on `Fs` (`is_case_insensitive`) keeps the rule testable on `SimFs`.
- **Removing files and directories.** `Fs` has no directory removal. Materialising a tree that drops a directory needs `remove_dir` (empty directories only), with its crash semantics in `SimFs`.
- **Ops from Sprint 1 and 2 stores** are development stores (provisional formats). No migration is planned.

## 5. Recommended Approach
1. **T-013 and T-014** first: the Sprint 2 CI record, and the randomness bans.
2. **`sim`:**
   - `Fs::remove_dir` (an empty directory; a syscall, with crash semantics);
   - a case-insensitivity capability (`is_case_insensitive`: Windows true, Unix false, `SimFs` configurable, including case-insensitive lookups when set).
3. **Hazard checks** (a pure function over a tree, run before any write):
   - case-fold collisions where the filesystem is case-insensitive;
   - Windows-unrepresentable names where the filesystem is Windows, or `SimFs` set to behave as Windows.

   A hazard is an error naming every offending path, and nothing is written.
4. **Materialise** in `workspace`:
   - a durable working-copy record `.praps/working-copy` holding the reflected op and an optional target;
   - files written through temporary names and renames, removed paths deleted, empty directories removed;
   - the record cleared last;
   - every command resumes an in-flight update before snapshotting.
5. **Commands:**
   - `praps log` lists ops newest first, with parents and the working-copy change;
   - `praps undo` and `praps restore OP` snapshot, append an op whose view is the target view, then materialise.

   OP is an op id or a unique hex prefix.
6. **Tests:**
   - INT-0009 AC-2: 200 pre-state pairs that differ only in uncommitted content, run through `undo` and `restore`: 0 collapsed.
   - INT-0009 AC-3: every op type in the corpus (init, snapshot, new, undo, restore) is restored exactly, in views and in working copy.
   - INT-0004 AC-3 and AC-4: hazard corpora at 100%.
   - A Materialise crash scenario with a negative control.
   - The edit corpus extended with `undo` and `restore`.

## Artifacts
- This report is the only artifact.
