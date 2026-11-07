# INT-0014 — The working copy is a change; workspaces are cheap lanes

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0014
- **State:** planned
- **Work evidence:** [Sprint 2 T-005 build plan](../sprints/s2/sprint-plans/build-plan.md#t-005-the-workspace-crate-init-snapshot-new-and-status), [Sprint 2 T-006 build plan](../sprints/s2/sprint-plans/build-plan.md#t-006-praps-init-praps-new-and-praps-status), [Sprint 2 T-007 build plan](../sprints/s2/sprint-plans/build-plan.md#t-007-snapshot-crash-scenario-and-edit-corpora), [Sprint 2 T-008 build plan](../sprints/s2/sprint-plans/build-plan.md#t-008-working-copy-spec-readme-credits-and-backlog)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
The working copy is itself a change. At the start of every command Periapsis
scans it (using an mtime/inode cache and ignore rules) and snapshots it into
the working-copy change, so no command discards uncommitted work. Snapshotting
stores each file's bytes exactly as found, unless the project has opted into a
normalisation rule for that path ([INT-0034](INT-0034-any-content.md)). Only a
line layer may write conflict markers into a conflicted file and read them
back as a conflict. Any other conflicted file is written as one of its sides,
and its conflict stays in the working-copy change until a side is chosen or
the file's bytes change, which makes those bytes the resolution. Any number of
workspaces share one op log and one object store, each with its own
working-copy change; agent workspaces can be sparse or API-only (writes without
a checkout). A workspace is the "lane" of
[INT-0021](INT-0021-lanes-and-review-requests.md), replacing per-lane clones and
git worktrees.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Auto-snapshot: every file modification made between two commands is captured in the working-copy change by the second command.
  - Threshold: 0 lost edits over a randomized edit-and-command corpus, on Windows and Linux.
- **AC-2** `necessary` `empirical` — Ignored paths are never snapshotted, and the matching rules are documented and covered by tests.
- **AC-3** `necessary` `empirical` — At least 8 workspaces over one repository operate concurrently without clones, without duplicated objects and without lost ops.
  - Threshold: 0 violations in simulation.
- **AC-4** `supporting` `empirical` — An agent can create and describe a change touching N files through an API-only workspace without materialising a checkout.
- **AC-5** `supporting` `empirical` — `status` on an unchanged working copy of 100,000 files completes within a time budget that is fixed before the first measurement and recorded in this chapter.
- **AC-6** `necessary` `empirical` — A conflicted file that no line layer reads is written as one of its sides and never with conflict markers; it stays conflicted while its bytes are unchanged, and once they change, the bytes found are recorded as its resolution.
  - Threshold: 100% over a corpus of conflicts in binaries, random bytes and files in a synthetic language.

## Rationale
- Snapshot-before-overwrite is what makes the history complete (R1-E2,
  INT-0009); scanning into the working-copy change is how Jujutsu achieves it
  (II exec summary #2).
- Lanes as workspaces over one op log remove the per-lane clone and
  `.git/worktrees` bookkeeping, and snapshots are automatic (R5 §C.1).
- Agent workspaces can be sparse or API-only (II §3.3).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §3.3, II §4 Phase 1, R5 §C.1.

## Alternatives
- **An explicit staging area (Git index).** Adds a state where work can be lost
  or forgotten, and agents gain nothing from it.
- **One clone or worktree per agent.** The bookkeeping and duplication this
  intent exists to remove.

## Consequences
- A filesystem watcher is not planned yet, so very large working copies rely on
  the stat cache (see AC-5).
- Case-fold and path-hazard rules (INT-0004) apply whenever a workspace is
  materialised.
- Ignore rules come from the `.gitignore` files inside the working copy, with
  Git's pattern semantics: each file applies to its own directory and below,
  the deepest file with a match decides, and nothing under an ignored
  directory can be re-included. Matching is case-sensitive on every system.
  `.praps/` and `.git/` at the root are always ignored. Git's global excludes
  file and `.git/info/exclude` are not read, so a snapshot does not depend on
  the machine it runs on. A tracked file that becomes ignored, or lies under a
  directory that becomes ignored, stays tracked until it is deleted, as in
  Git.
- A symbolic link, a special file, or a name with no UTF-8 form
  ([INT-0004](INT-0004-windows-and-linux.md) AC-7) is reported by every
  snapshot and otherwise left alone, until symlinks are specified.
- Windows has no executable bit, so a snapshot taken there keeps the mode the
  working-copy change, or else its parent, last recorded for a tracked file,
  and records a new file as regular.
- Every command that opens the working copy snapshots it first. `fsck`
  checks the store read-only and `init` on an existing repository writes
  nothing, so neither snapshots; an edit made before them is captured by the
  next command that does. AC-1's "second command" means the next command
  that opens the working copy.
- `new` starts a fresh working-copy change, with a newly minted change ID,
  on top of the current one; it writes no working-copy file. Its rewriting
  semantics belong to [INT-0011](INT-0011-change-identity-and-rewriting.md).
- Until the stat cache exists (AC-5), every command reads and hashes every
  tracked file, and the store re-reads each blob a changed changeset names to
  check it. A file edited during a scan is captured as it was before or after
  the edit, either of which is a correct snapshot; the mtime-granularity race
  arrives with the cache and must be handled there.
- Until [INT-0010](INT-0010-lock-free-concurrency.md)'s canonical merge exists,
  a command that finds several op heads stops and names them instead of
  guessing which to build on.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §4 Phase 1 and R5 §C.1).
- 2026-11-01: revised while `proposed`, on owner direction: the working copy must hold conflicts in any content. The Intent says snapshots keep each file's bytes exactly unless a normalisation rule is opted in, only a line layer writes and reads conflict markers, and any other conflicted file is written as one of its sides and resolved by choosing a side or changing its bytes; AC-6 checks this.
- 2026-11-07: revised while `proposed` (Sprint 2 Plan): Consequences record the ignore-rule sources, that symlinks, special files and names with no UTF-8 form are reported and left alone, the Windows executable-bit rule, and the several-heads stop until INT-0010.
- 2026-11-07: `proposed` → `planned` (Sprint 2 build plan T-005 to T-008: init, snapshot and status, the CLI, the crash and edit corpora; AC-1 and AC-2. AC-3 to AC-6 come in later sprints).
- 2026-11-07: revised while `planned` (Sprint 2 plan critique, round 1): Consequences state the ignore matcher composition and case sensitivity, that tracked files under ignored directories stay tracked, that the Windows mode comes from the working-copy change or else its parent, that `new` starts a change without writing files, and the scanning costs and racy edits deferred to the stat cache.
- 2026-11-07: revised while `planned` (Sprint 2 plan critique, round 2): Consequences say which commands snapshot, and that `fsck` and a repeated `init` do not.
