# INT-0014 — The working copy is a change; workspaces are cheap lanes

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0014
- **State:** active
- **Work evidence:** [Sprint 2 T-005 build plan](../sprints/s2/sprint-plans/build-plan.md#t-005-the-workspace-crate-init-snapshot-new-and-status), [Sprint 2 T-006 build plan](../sprints/s2/sprint-plans/build-plan.md#t-006-praps-init-praps-new-and-praps-status), [Sprint 2 T-007 build plan](../sprints/s2/sprint-plans/build-plan.md#t-007-snapshot-crash-scenario-and-edit-corpora), [Sprint 2 T-008 build plan](../sprints/s2/sprint-plans/build-plan.md#t-008-working-copy-spec-readme-credits-and-backlog), [Sprint 3 T-004 build plan](../sprints/s3/sprint-plans/build-plan.md#t-004-specify-writing-the-working-copy-undo-restore-and-log), [Sprint 3 T-006 build plan](../sprints/s3/sprint-plans/build-plan.md#t-006-materialise-the-working-copy-record-and-resume), [Sprint 3 T-009 build plan](../sprints/s3/sprint-plans/build-plan.md#t-009-reversibility-and-hazard-corpora-the-materialise-crash-scenario-and-the-extended-edit-corpus), [Sprint 4 T-006 build plan](../sprints/s4/sprint-plans/build-plan.md#t-006-plant-and-register-points-in-store-and-workspace-and-move-the-two-writer-race-to-a-named-point), [Sprint 4 T-009 build plan](../sprints/s4/sprint-plans/build-plan.md#t-009-report-skipped-names-from-every-command-that-snapshots-backlog-t-019), [Sprint 6 T-003 build plan](../sprints/s6/sprint-plans/build-plan.md#t-003-a-command-reads-the-op-log-once), [Sprint 6 T-004 build plan](../sprints/s6/sprint-plans/build-plan.md#t-004-open-checks-the-op-log-before-it-resumes-an-interrupted-write), [Sprint 6 T-005 build plan](../sprints/s6/sprint-plans/build-plan.md#t-005-undo-and-restore-scan-the-working-copy-once), [Sprint 6 T-008 build plan](../sprints/s6/sprint-plans/build-plan.md#t-008-commands-find-their-working-copy-from-any-directory-inside-it), [Sprint 6 T-010 build plan](../sprints/s6/sprint-plans/build-plan.md#t-010-describe-sets-the-description-of-the-current-change), [Sprint 6 T-012 build plan](../sprints/s6/sprint-plans/build-plan.md#t-012-a-write-leaves-a-directory-it-cannot-remove), [Sprint 6 T-013 build plan](../sprints/s6/sprint-plans/build-plan.md#t-013-readme-specs-and-credits)
- **Completion evidence:** [Sprint 2 T-005 completion](../work/completed-tasks.md#t-005-sprint-2), [Sprint 2 T-006 completion](../work/completed-tasks.md#t-006-sprint-2), [Sprint 2 T-007 completion](../work/completed-tasks.md#t-007-sprint-2), [Sprint 2 T-008 completion](../work/completed-tasks.md#t-008-sprint-2)
- **Code evidence:** [crates/workspace](../../crates/workspace/src/lib.rs), [praps](../../crates/praps/src/main.rs)
- **Test evidence:** [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md), [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md), [Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md)
- **Documentation evidence:** [The working copy](../spec/working-copy.md)

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

Three rules come from [INT-0035](INT-0035-scale-and-asynchrony.md) and
[INT-0037](INT-0037-command-set.md):
- **A workspace is cheap.** Making one writes one view entry and no
  working-copy file, so every agent task can have its own. The API-only form
  is the usual one for an agent.
- **Nothing unchanged is read.** A command reads no file whose metadata is
  unchanged since the last snapshot.
- **Anywhere inside.** A command run in any directory of a working copy acts
  on that working copy.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Auto-snapshot: every file modification made between two commands is captured in the working-copy change by the second command.
  - Threshold: 0 lost edits over a randomized edit-and-command corpus, on Windows and Linux.
- **AC-2** `necessary` `empirical` — Ignored paths are never snapshotted, and the matching rules are documented and covered by tests.
- **AC-3** `necessary` `empirical` — At least 8 workspaces over one repository operate concurrently without clones, without duplicated objects and without lost ops.
  - Threshold: 0 violations in simulation.
- **AC-4** `supporting` `empirical` — An agent can create and describe a change touching N files through an API-only workspace without materialising a checkout.
- **AC-5** `supporting` `empirical` — `status` on an unchanged working copy of 100,000 files completes within a time budget that is fixed before the first measurement and recorded in this chapter.
  - Threshold: the budget, fixed on 2026-11-10, is workload W2 of [INT-0036](INT-0036-measured-speed.md): below the faster of Git and Jujutsu on the same tree and machine at equal durability (AC-2 there), with 100 ms as the aim (AC-5 there).
- **AC-6** `necessary` `empirical` — A conflicted file that no line layer reads is written as one of its sides and never with conflict markers; it stays conflicted while its bytes are unchanged, and once they change, the bytes found are recorded as its resolution.
  - Threshold: 100% over a corpus of conflicts in binaries, random bytes and files in a synthetic language.
- **AC-7** `necessary` `empirical` — A thousand workspaces: 1,000 workspaces over one repository operate at once in simulation without clones, duplicated objects or lost ops, and making a workspace writes the same number of bytes, within a factor of 2, whether 10 or 10,000 already exist.
  - Threshold: 0 violations ([INT-0035](INT-0035-scale-and-asynchrony.md) AC-3 and AC-4).
- **AC-8** `necessary` `empirical` — Nothing unchanged is read: a command on a working copy in which no file's metadata has changed since the last snapshot reads the content of no file.
  - Sample: working copies of 10 and 100,000 files in the simulator.
  - Threshold: 0 reads of file content.

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
- Every command that opens the working copy snapshots it first. `verify`
  (named `fsck` before Sprint 6) checks the store read-only and `init` on an
  existing repository writes nothing, so neither snapshots; an edit made
  before them is captured by the next command that does. AC-1's "second command" means the next command
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

- A first snapshot must stay linear in the working copy: Sprint 2's code
  review found a quadratic step and a whole-copy content buffer (CR-04). Until
  the stat cache, content is kept only for blobs the store does not have yet.

- Every command that opens the working copy resumes an interrupted write
  before it does anything of its own, through the record in `.praps/`; `init`
  never resumes, so a repeated `init` still writes nothing. From Sprint 6 it
  first checks that every op file is whole, and stops if one is not, leaving
  the record in place (T-004). Before, resume looked for the record's target
  among the valid ops only. A target whose file was damaged then looked like
  a write that had never begun, and the record was dropped, though files
  might already have been written. A write uses deterministic temporary
  names, `.praps-tmp-<n>`, which exist only while the record names them. No
  name is silently ignored: a user's file with that prefix is tracked like any
  other (Sprint 3 plan critique, C-012, C-013). While a record exists, its
  diff's temporary names belong to `praps`: a user file created at one of them
  during the interrupted write may be replaced or removed by resume. Every
  way resume drops a record also removes those names (round 3, C-004).
- A write re-reads each path just before removing or replacing it, and keeps
  a path the user changed, with a note (Sprint 3 code review, CR-03). AC-1
  still has a small gap: an edit saved between that read and the next
  syscall can be lost, and the mode is not re-read. Closing the gap would
  need an atomic compare-and-replace. The spec states the window.
- What the missing stat cache costs was measured in Sprint 6
  ([research](../sprints/s6/sprint-research/research-report.md)): `status`
  with nothing changed on 10,000 files took 148 ms on Linux and 1,188 ms on
  Windows, about fifteen times Git's time on each. AC-8 is the rule that
  removes that cost, and AC-5 the budget it must meet.
- From Sprint 6 a command reads the op log once and keeps what it derives
  while it runs (T-003). Before, it read the whole log three to six times.
  A command starts from the log as it is on disk when it begins, so it sees
  what another process wrote before then. There is one exception: the first
  command on a workspace just opened uses the reading that opening made, so
  that a `praps` process, which opens and runs one command, reads the log
  once. An operation another process writes between the opening and that
  command is treated as one written while the command ran: both stand, the
  log has two heads, and the next command says so.
- From Sprint 6 `praps` finds the working copy from any directory inside it,
  and `init` refuses to make a repository inside one (T-008). Without that,
  `praps status` in a subdirectory said the directory was not a repository
  and suggested `praps init`, which would have nested one.
- The top of a working copy is the directory that holds `.praps/store`. A
  directory named `.praps` below the top that holds no store is tracked like
  any other directory. One that holds a store is the top of a repository of
  its own: commands run inside it act on it, and the working copy around it
  still tracks its files as content. What should happen there is not decided
  (backlog T-031).
- From Sprint 6 `describe` sets the working-copy change's description
  (T-010). A snapshot carries the description forward, as it carries the
  change ID.
- A write removes each directory it emptied, if it can. Running commands
  from inside the working copy makes one case common: the directory a shell
  stands in is one the write empties. From Sprint 6 a removal that fails
  does not stop the write (T-012). The directory is left, with a note.
  Directories are not tracked, so the working copy still equals the target.
  The one exception is a target that has a file at that path: there the
  write stops, as before, and the next command finishes it once the
  directory can be removed. T-012 tests a directory that is another
  process's current directory on Windows and on Linux, and records what
  each system does. Where a system removes it, the shell that stood there
  has no current directory, and a command run from it says so (T-008).
  Whether a write should spare the directory a command was started in is
  not decided (backlog T-031).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §4 Phase 1 and R5 §C.1).
- 2026-11-01: revised while `proposed`, on owner direction: the working copy must hold conflicts in any content. The Intent says snapshots keep each file's bytes exactly unless a normalisation rule is opted in, only a line layer writes and reads conflict markers, and any other conflicted file is written as one of its sides and resolved by choosing a side or changing its bytes; AC-6 checks this.
- 2026-11-07: revised while `proposed` (Sprint 2 Plan): Consequences record the ignore-rule sources, that symlinks, special files and names with no UTF-8 form are reported and left alone, the Windows executable-bit rule, and the several-heads stop until INT-0010.
- 2026-11-07: `proposed` → `planned` (Sprint 2 build plan T-005 to T-008: init, snapshot and status, the CLI, the crash and edit corpora; AC-1 and AC-2. AC-3 to AC-6 come in later sprints).
- 2026-11-07: revised while `planned` (Sprint 2 plan critique, round 1): Consequences state the ignore matcher composition and case sensitivity, that tracked files under ignored directories stay tracked, that the Windows mode comes from the working-copy change or else its parent, that `new` starts a change without writing files, and the scanning costs and racy edits deferred to the stat cache.
- 2026-11-07: revised while `planned` (Sprint 2 plan critique, round 2): Consequences say which commands snapshot, and that `fsck` and a repeated `init` do not.
- 2026-11-07: `planned` → `active` (Sprint 2 build, T-005 starts: the `workspace` crate).
- 2026-11-07: revised while `active` (Sprint 2 Loop): Consequences record the linear-scan requirement (CR-04). AC-1 and AC-2 are met by the Sprint 2 test report; AC-3 to AC-6 remain, so the chapter stays `active`.
- 2026-11-07: revised while `active` (Sprint 3 plan critique, round 1): Consequences record that every command except `init` resumes an interrupted write, and that temporary names are deterministic and never ignored.
- 2026-11-07: revised while `active` (Sprint 3 plan critique, round 2): every command that opens the working copy resumes, not only those that write it.
- 2026-11-07: revised while `active` (Sprint 3 plan critique, round 3): temporary names belong to `praps` while a record exists, and every way resume drops a record removes them.
- 2026-11-10: revised while `active` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents, speed and a more intuitive command set): the Intent adds three rules (a workspace is cheap, nothing unchanged is read, a command works anywhere inside a working copy). AC-5's budget is fixed by reference to [INT-0036](INT-0036-measured-speed.md), and AC-7 and AC-8 are added. Consequences record Sprint 6's measurements and its tasks in this chapter's code.
- 2026-11-10: revised while `active` (Sprint 6 plan critique, round 1, C-001 and C-008; `semantic`): Consequences now say that opening checks the op log before it resumes, and why; that the first command after opening uses the reading opening made, and what follows if another process writes in between; and what marks the top of a working copy, with a `.praps` directory below the top left as it is today.
- 2026-11-10: revised while `active` (Sprint 6 plan critique, round 2, C-005; `semantic`): Consequences record that a write leaves a directory it emptied and cannot remove, with a note, unless the target has a file at that path; and that a command run from a directory that no longer exists says so.
