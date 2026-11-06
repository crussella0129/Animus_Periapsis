# INT-0014 — The working copy is a change; workspaces are cheap lanes

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0014
- **State:** proposed
- **Work evidence:** none
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

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §4 Phase 1 and R5 §C.1).
- 2026-11-01: revised while `proposed`, on owner direction: the working copy must hold conflicts in any content. The Intent says snapshots keep each file's bytes exactly unless a normalisation rule is opted in, only a line layer writes and reads conflict markers, and any other conflicted file is written as one of its sides and resolved by choosing a side or changing its bytes; AC-6 checks this.
