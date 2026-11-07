# Sprint 3 Build Plan

## Intents
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — state: planned; acceptance criteria covered:
  - AC-1, by construction: every command appends ops and none removes one. It is checked by property and by the restore corpus;
  - AC-2: snapshot-before-overwrite, with 0 of 200 pairs collapsed;
  - AC-3: `undo` and `restore` return views and working copy to the exact earlier state, for every op type in the corpus.

  AC-4 (signed erasure) and AC-5 (shallow and partial clones) wait for actors and sync.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — state: active; acceptance criteria covered: AC-1 (0 lost edits), kept while commands also write the working copy, including after an interrupted write. AC-3 to AC-6 wait.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered:
  - AC-1 for this sprint's code;
  - AC-3: case-fold collisions are detected before materialising where the filesystem is case-insensitive, and neither file is overwritten;
  - AC-4: names Windows cannot represent produce an error naming the path, never a rename or a partial write.
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered:
  - INT-0007 AC-1 and AC-2, and INT-0005 AC-1 to AC-3, through a Materialise crash scenario with a negative control;
  - INT-0005 AC-4 and its Consequence on randomness (backlog T-014).
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — through T-001 (backlog T-013) and T-002 only.

## Schema Tree
- Sprint Goal: Periapsis can go back: `praps log`, `praps undo` and `praps restore OP`, which write the working copy safely
  - Carried forward
    - T-001: Record Sprint 2's GitHub Actions runs and owner sign-offs (backlog T-013)
    - T-002: Ban the remaining routes to operating-system randomness (backlog T-014)
  - Simulation
    - T-003: `remove_dir`, `set_executable` and `name_rules` in `sim`
  - Specification
    - T-004: Specify writing the working copy, `undo`, `restore` and `log`
  - Working copy
    - T-005: Hazards, materialise, the working-copy record, and `log`, `undo` and `restore` in `workspace`
    - T-006: `praps log`, `praps undo` and `praps restore OP`
    - T-007: Reversibility and hazard corpora, the Materialise crash scenario, and the extended edit corpus
  - Documentation
    - T-008: README, credits and backlog

## Execution Sequence

### T-001: Record Sprint 2's GitHub Actions runs and owner sign-offs
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- **Touches:** docs/sprints/s3/sprint-tests/ci-sprint2.md, docs/intents/INT-0001-periapsis-charter.md, docs/intents/INT-0002-typed-acceptance-criteria.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0004-windows-and-linux.md, docs/intents/INT-0006-format-v1.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:**
  - the GitHub-matrix halves of Sprint 2's checkpoint: INT-0003 AC-2 to AC-4, INT-0004 AC-1, INT-0006 AC-3;
  - owner sign-offs: INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6, INT-0004 AC-5.
- **Success criterion (EARS):**
  - **WHEN** the record is written, **THEN** `ci-sprint2.md` **SHALL** name each of the following, all taken from `gh run view` output:
    - for runs 42403413644, 42403409376 and 42404724911: head SHA, event, conclusion;
    - every job, with its conclusion and runner OS;
    - the `rustc` version and the test count of each `cargo ci` leg.
  - **WHEN** an owner sign-off has not been given in the conversation, **THEN** the record **SHALL** say it is pending, and backlog T-013 **SHALL** be replaced by a carried-forward task. No sign-off **SHALL** be recorded without the owner's words.

### T-002: Ban the remaining routes to operating-system randomness
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** crates/xtask/src/policy.rs, crates/sim/clippy.toml, crates/store/clippy.toml, crates/praps/clippy.toml, crates/workspace/clippy.toml
- **Depends on:** (none)
- **Acceptance criterion:** INT-0005 AC-4 and its Consequence that randomness reaches a boundary crate only through `sim` (Sprint 2 code review CR-07)
- **Success criterion (EARS):**
  - **WHEN** `cargo policy` runs, **THEN** `fs-boundary` **SHALL** require every boundary crate's `clippy.toml` to list `std::hash::RandomState` under `disallowed-types`.
  - **WHEN** `cargo policy` runs, **THEN** `deps` **SHALL** read `Cargo.lock` and reject any boundary crate other than `sim` that depends, directly or transitively, on `getrandom`, `rand` or `uuid`, naming the crate and the path to the dependency.
  - **WHEN** the policy tests run, **THEN** fixtures with a missing `RandomState` ban, and with a transitive `rand` under a boundary crate, **SHALL** fail, and the real workspace **SHALL** pass.

### T-003: remove_dir, set_executable and name_rules in sim
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/sim/src/fs.rs, crates/sim/src/real.rs, crates/sim/src/simfs.rs, crates/sim/src/lib.rs, crates/sim/tests/contract.rs, crates/sim/tests/real_scan.rs, crates/store/tests/common/matrix.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0005 AC-1 and AC-2 for the new syscalls; INT-0004 AC-1
- **Success criterion (EARS):**
  - **WHEN** `remove_dir` is called on an empty directory, **THEN** it **SHALL** remove it. On a non-empty or missing directory it **SHALL** fail and change nothing.
  - **WHEN** `set_executable` is called on a file where the filesystem has an executable bit, **THEN** it **SHALL** set or clear it. Where there is none, it **SHALL** succeed and change nothing.
  - **WHEN** `SimFs` crashes, **THEN** an unsynced `remove_dir` **SHALL** be kept or reverted by the seed, like any entry change. An unsynced executable-bit change **SHALL** be kept or reverted by the seed, like file data. Both **SHALL** take no effect at their crash point and count as one syscall each.
  - **WHEN** `name_rules` is read, **THEN** `RealFs` **SHALL** report `{ case_insensitive: true, windows_names: true }` on Windows and both false on Unix, and `SimFs` **SHALL** report its configured value, the same on every host. It is not a syscall.
  - **WHEN** the contract tests run on `SimFs` and on `RealFs` on both systems, **THEN** `remove_dir` and `set_executable` **SHALL** meet the contract.
  - **WHEN** the store crash matrix runs, **THEN** its digest over existing scenarios **SHALL** be unchanged.

### T-004: Specify writing the working copy, undo, restore and log
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** docs/spec/working-copy.md, docs/intents/INT-0009-reversible-by-default.md
- **Depends on:** (none)
- **Acceptance criterion:** INT-0009 AC-2 and AC-3; INT-0004 AC-3 and AC-4 (the rules a reviewer checks the code against)
- **Success criterion (EARS):**
  - **WHEN** the spec is read, **THEN** `working-copy.md` **SHALL** state:
    - the working-copy record (location, content, write order relative to the op, resume and drop rules);
    - the materialise rules (the per-path rule for user edits, temporary names, the executable bit, directory creation and removal, the sync order);
    - the hazard rules (case-fold collisions, including with existing untracked entries, and the Windows name rules);
    - the semantics of `undo` (the head op's parent's view, after the snapshot; toggling; an error on the root op), `restore OP` (an op id or a unique prefix) and `log`.
  - **WHEN** INT-0009 is read, **THEN** its Consequences **SHALL** record the record-based recovery and the `undo` toggle, with the edits reachable by `restore`.

### T-005: Hazards, materialise, the working-copy record, and log, undo and restore in workspace
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/workspace/Cargo.toml, crates/workspace/src/lib.rs, crates/workspace/src/hazard.rs, crates/workspace/src/materialise.rs, crates/workspace/src/variant.rs, crates/workspace/tests/workspace.rs, crates/workspace/tests/history.rs, crates/xtask/src/policy.rs
- **Depends on:** T-003, T-004
- **Acceptance criterion:** INT-0009 AC-1 to AC-3; INT-0004 AC-3 and AC-4; INT-0014 AC-1
- **Success criterion (EARS):**
  - **WHEN** `log` runs, **THEN** it **SHALL** return every op, newest first, with its id, parents and working-copy change. It writes nothing besides the usual snapshot.
  - **WHEN** `undo` runs on a head op with a parent, **THEN** it **SHALL**:
    - snapshot;
    - write the working-copy record;
    - commit an op whose single parent is the head (after the snapshot) and whose view is the exact view object of that head op's parent;
    - materialise;
    - remove the record.

    **WHEN** the head op has no parent, **THEN** `undo` **SHALL** fail with an error and write nothing.
  - **WHEN** `restore OP` runs with an op id or a unique hex prefix, **THEN** it **SHALL** do the same with OP's view. **WHEN** the prefix matches no op or several, **THEN** it **SHALL** fail, listing the candidates, and write nothing.
  - **WHEN** materialising from tree F to tree T, **THEN**, for each path in either tree:
    - a file whose bytes and mode equal F's version (or which is absent where F has none) **SHALL** be made T's version: written through a `.praps-tmp-*` name, synced and renamed, or removed;
    - any other file **SHALL** be left as it is.

    Missing directories **SHALL** be created. Directories left empty that T does not need **SHALL** be removed. Every touched file and directory **SHALL** be synced before the record is removed.
  - **WHEN** a hazard exists in T for the filesystem's `name_rules`, **THEN** the command **SHALL** fail with an error naming every offending path, before writing the record, the op or any file. The hazards are:
    - two paths equal under lowercase, or a T path equal under lowercase to an existing entry with another spelling, where the filesystem is case-insensitive;
    - a component with `<>:"|?*`, a control character, a trailing `.` or space, or a reserved device name with any extension and in any case, where Windows names apply.
  - **WHEN** any constructor or command starts and a working-copy record exists, **THEN** it **SHALL** first:
    - finish materialising F → T, if the target op is in the store;
    - otherwise drop the record and touch no file;
    - then remove leftover `.praps-tmp-*` files in the record's directories.

    Snapshots **SHALL** never track a `.praps-tmp-*` name.
  - **WHEN** the `variants` feature selects `NoRecord`, **THEN** materialise **SHALL** run without the record, for the negative control. The `deps` rule **SHALL** allow that feature only through `[dev-dependencies]`, as for the store.
- **Notes:**
  - The view of each new op is the existing view object, so the view id equals the target op's.
  - The target op's id is computed from its envelope before the record is written.

### T-006: praps log, praps undo and praps restore OP
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- **Touches:** crates/praps/src/main.rs, crates/praps/tests/cli.rs, crates/praps/tests/e2e.rs
- **Depends on:** T-005
- **Acceptance criterion:** INT-0009 AC-3 through the CLI
- **Success criterion (EARS):**
  - **WHEN** `praps log [DIR]` runs, **THEN** it **SHALL** print one line per op, newest first: a 12-hex op id prefix, the working-copy change's change-ID prefix, and the file count. It **SHALL** exit 0.
  - **WHEN** `praps undo [DIR]` runs, **THEN** it **SHALL** undo as T-005 states, print the op it restored to, and exit 0. With nothing to undo it **SHALL** exit 1.
  - **WHEN** `praps restore OP [DIR]` runs, **THEN** it **SHALL** restore as T-005 states and exit 0. With an unknown or ambiguous OP, or a hazard, it **SHALL** exit 1 with the error on stderr.
  - **WHEN** a flag or an extra argument is given, **THEN** `praps` **SHALL** exit 2. `restore` without OP **SHALL** exit 2.

### T-007: Reversibility and hazard corpora, the Materialise crash scenario, and the extended edit corpus
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/workspace/tests/history.rs, crates/workspace/tests/hazards.rs, crates/workspace/tests/materialise_crash.rs, crates/workspace/tests/common/mod.rs, crates/workspace/tests/edits.rs, crates/workspace/tests/real_edits.rs, crates/workspace/tests/real_hazards.rs
- **Depends on:** T-005
- **Acceptance criterion:** INT-0009 AC-1 to AC-3; INT-0004 AC-3 and AC-4; INT-0005 AC-1 to AC-3; INT-0007 AC-1 and AC-2; INT-0014 AC-1
- **Success criterion (EARS):**
  - **WHEN** 200 pairs of working copies that differ only in uncommitted content each run `undo` (and, separately, `restore` of an earlier op), **THEN**:
    - the pairs' op logs **SHALL** stay distinguishable: their snapshot ops' trees differ;
    - each pre-state **SHALL** be recovered exactly by `restore` of its snapshot op.

    That is 0 collapsed (INT-0009 AC-2).
  - **WHEN** each op of a corpus covering every op type (init, snapshot, new, undo, restore) is restored, **THEN** the new head's view **SHALL** be that op's view object, and the working copy **SHALL** equal that view's working-copy tree byte for byte, with modes (INT-0009 AC-3).
  - **WHEN** the ops are listed after any command sequence, **THEN** every earlier op **SHALL** still be present (INT-0009 AC-1, append-only).
  - **WHEN** each case of a case-fold corpus (at least 10 cases: files, directories, non-ASCII, an existing untracked variant) and of a Windows-name corpus (at least 30 names: every reserved base, with extensions and case variants, every forbidden character, trailing dot and space) is restored on a filesystem with those rules, **THEN** the command **SHALL** fail naming the paths, with 0 files changed: 100%. On a filesystem without those rules, the same trees **SHALL** materialise.
  - **WHEN** `restore` of a hazardous tree runs on real NTFS, **THEN** it **SHALL** fail with nothing written. The same tree **SHALL** materialise on ext4.
  - **WHEN** the Materialise scenario (init, edits, snapshot, new, edits, `undo`, `restore`) runs with a crash at every syscall × 40 seeds × both modes, **THEN** after recovery and resume there **SHALL** be 0 violations of:
    - closure and durability;
    - recency: the head's working-copy tree is the last acknowledged command's or the in-flight one's;
    - liveness and no lost edit: open, then snapshot, succeeds, and the change then equals the working copy, which equals the head's target tree or holds only user edits the model knows.

    Every cell **SHALL** reproduce, with the same outcome.
  - **WHEN** the scenario runs with the `NoRecord` variant, **THEN** it **SHALL** report recency or liveness violations in strict mode.
  - **WHEN** the edit corpus (≥ 500 sequences on `SimFs`, ≥ 20 on real NTFS and ext4) also runs `undo` and `restore` of a random earlier op, **THEN** after every step the change's tree and the working copy **SHALL** match the independent model, which keeps its own op history: 0 lost edits.

### T-008: README, credits and backlog
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** README.md, docs/prior-art.md, docs/work/tasks.md
- **Depends on:** T-006
- **Acceptance criterion:** INT-0003 AC-5 (credits)
- **Success criterion (EARS):**
  - **WHEN** the README is read, **THEN** it **SHALL** list `praps log`, `praps undo` and `praps restore OP`.
  - **WHEN** `cargo policy` runs, **THEN** the `credits` rule **SHALL** pass, and `docs/prior-art.md` **SHALL** credit jj's operation log and stale-working-copy recovery and Microsoft's naming rules.
  - **WHEN** the backlog is updated, **THEN** any work found and not done **SHALL** be a `(backlog)` task with its intents.
