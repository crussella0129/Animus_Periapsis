# Sprint 3 Build Plan

## Intents
- [INT-0009](../../../intents/INT-0009-reversible-by-default.md) — state: planned; acceptance criteria covered:
  - AC-1, by construction: every command appends ops and none removes one. It is checked by property and by the restore corpus;
  - AC-2: snapshot-before-overwrite, with 0 of 200 pairs collapsed;
  - AC-3: `undo` and `restore` return views and working copy to the exact earlier state, after every op type, for the tracked set.

  AC-4 (signed erasure) and AC-5 (shallow and partial clones) wait for actors and sync.
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — state: active; acceptance criteria covered: AC-1 (0 lost edits), kept while commands also write the working copy, including after an interrupted write. AC-3 to AC-6 wait.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered:
  - AC-1 for this sprint's code;
  - AC-3: case-fold collisions are detected before materialising where the filesystem is case-insensitive, and neither file is overwritten. A case-only rename is refused there as a known limit (backlog T-016);
  - AC-4: names Windows cannot represent, among the paths a command would create or rewrite, produce an error naming the path, never a rename or a partial write.
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered:
  - INT-0007 AC-1 and AC-2, and INT-0005 AC-1 to AC-3, through a Materialise crash scenario with an exact oracle and a negative control;
  - INT-0005 AC-4 and its randomness Consequence (backlog T-014).
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
    - T-005: The hazard check
    - T-006: Materialise, the working-copy record and resume
    - T-007: `log`, `undo` and `restore` in `workspace`
    - T-008: `praps log`, `praps undo` and `praps restore OP`
    - T-009: Reversibility and hazard corpora, the Materialise crash scenario, and the extended edit corpus
  - Documentation
    - T-010: README, credits and backlog

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
  - **WHEN** `cargo policy` runs, **THEN** `deps` **SHALL** reject any boundary crate other than `sim` that reaches `getrandom`, `rand`, `uuid` or `fastrand`, naming the crate and the path. Reaching is taken over:
    - members' normal and target-specific `[dependencies]`, from their manifests, with renamed keys resolved through `package`, including inherited `[workspace.dependencies]` aliases;
    - registry crates' dependencies, from `Cargo.lock`, where an entry may be `"name"` or `"name version"`.

    Dev and build dependencies are not edges. An edge into `sim` is allowed, since `sim` is the crate that owns operating-system randomness.
  - **WHEN** the policy tests run, **THEN** the following **SHALL** fail:
    - a boundary config without the `RandomState` ban;
    - a fixture in which `store` reaches `rand` through a registry crate;
    - the same through a renamed dependency;
    - the same through a `[target.'cfg(windows)'.dependencies]` dependency.

    The following **SHALL** pass: a member with `proptest` as a dev-dependency only, `store` → `sim` → `getrandom`, and the real workspace.

### T-003: remove_dir, set_executable and name_rules in sim
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/sim/src/fs.rs, crates/sim/src/real.rs, crates/sim/src/simfs.rs, crates/sim/src/lib.rs, crates/sim/tests/contract.rs, crates/sim/tests/scan.rs, crates/sim/tests/real_scan.rs, crates/store/tests/common/matrix.rs, crates/workspace/tests/edits.rs, crates/workspace/tests/workspace.rs, crates/workspace/tests/real_edits.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0005 AC-1 and AC-2 for the new syscalls; INT-0004 AC-1
- **Success criterion (EARS):**
  - **WHEN** `remove_dir` is called on an empty directory whose own entry changes are all synced, **THEN** it **SHALL** remove it. On a non-empty, missing or not-yet-synced directory, `SimFs` **SHALL** fail and change nothing.
  - **WHEN** `create_dir` is called in `SimFs` for a name whose removal is still pending in the parent, **THEN** it **SHALL** fail. Materialise syncs the parent between removing and creating, so this never stops a correct write, and a reverted removal always recovers an empty directory with no shared state.
  - **WHEN** `set_executable` is called on a file where the filesystem has an executable bit, **THEN** it **SHALL** set or clear it. Where there is none, it **SHALL** succeed and change nothing.
  - **WHEN** `SimFs` crashes, **THEN**:
    - an unsynced `remove_dir` **SHALL** be kept or reverted by the seed, like any entry change;
    - an unsynced executable-bit change **SHALL** be kept or reverted by the seed. The inode keeps a durable bit that `sync_file` updates, and the seed is drawn only when a change is pending.

    Both syscalls **SHALL** take no effect at their crash point and count as one syscall each.
  - **WHEN** `name_rules` is read, **THEN** `RealFs` **SHALL** report `{ case_insensitive: true, windows_names: true }` on Windows and both false on Unix, and `SimFs` **SHALL** report its configured value, the same on every host. It is not a syscall. `SimFs` lookups stay exact-case: since any case-variant write is refused as a hazard, no write depends on case-insensitive lookup.
  - **WHEN** the existing helper is used, **THEN** it **SHALL** be named `make_executable` (not a syscall, durable at once). On `Rc<SimFs>`, method lookup would otherwise reach the new `Fs::set_executable` first.
  - **WHEN** the contract tests run on `SimFs` and on `RealFs` on both systems, **THEN** `remove_dir` and `set_executable` **SHALL** meet the contract.
  - **WHEN** the store and Snapshot crash matrices run, **THEN** their digests **SHALL** be unchanged.

### T-004: Specify writing the working copy, undo, restore and log
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** docs/spec/working-copy.md, docs/intents/INT-0009-reversible-by-default.md, docs/intents/INT-0014-working-copy-and-workspaces.md, docs/intents/INT-0004-windows-and-linux.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:** INT-0009 AC-2 and AC-3; INT-0004 AC-3 and AC-4 (the rules a reviewer checks the code against)
- **Success criterion (EARS):**
  - **WHEN** the spec is read, **THEN** `working-copy.md` **SHALL** state each rule T-005 to T-007 implement:
    - the record, its atomic write, resume, an incomplete resume, and drop;
    - the classification of each path, F-owned entries, and blocking entries;
    - the materialise order;
    - the deterministic temporary names and their collisions;
    - the executable bit;
    - the hazard rules: per component, their scope, and the full reserved list;
    - `undo`, `restore` and `log`;
    - which commands resume, and where notes go.
  - **WHEN** the intents are read, **THEN**:
    - INT-0009's Consequences **SHALL** state the precise `undo` definition;
    - INT-0014's **SHALL** say that every command that opens the working copy resumes, that `init` never does, and that temporary names are not ignored;
    - INT-0004's **SHALL** record that the rules come from the platform, with their limits, and that a case-only rename is refused on a case-insensitive filesystem as a known limit (backlog T-016).

### T-005: The hazard check
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/workspace/src/hazard.rs, crates/workspace/src/lib.rs, crates/workspace/tests/hazards.rs
- **Depends on:** T-003, T-004
- **Acceptance criterion:** INT-0004 AC-3 and AC-4
- **Success criterion (EARS):**
  - **WHEN** the check runs, **THEN** it **SHALL** be a pure function of F, T, the existing directory listings and `name_rules`. It **SHALL** consider every component of each path a write creates or rewrites, and of each directory it creates.
  - **WHEN** the filesystem is case-insensitive, **THEN** a hazard **SHALL** be either of the following. Two names are equal when they match under `to_lowercase` or under `to_uppercase`, a deliberately conservative test:
    - two such components equal ignoring case and spelled differently, among T's paths;
    - such a component equal ignoring case to an existing entry spelled differently in the same directory.

    The second case includes a case-only rename, a known limit (INT-0004 Consequences, T-016).
  - **WHEN** Windows names apply, **THEN** a hazard **SHALL** be a component with any of the following:
    - a character in `<>:"|?*` or 0–31;
    - a trailing `.` or space;
    - a reserved base: the part before the first `.`, with trailing spaces removed, compared ignoring case, against `CON`, `PRN`, `AUX`, `NUL`, `CONIN$`, `CONOUT$`, `COM0`–`COM9`, `LPT0`–`LPT9`, and `COM`/`LPT` with ¹, ² or ³.
  - **WHEN** any hazard exists, **THEN** the command **SHALL** fail with an error naming every offending path. Its only write **SHALL** be its opening snapshot: no record, no op, no file.

### T-006: Materialise, the working-copy record and resume
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** crates/workspace/Cargo.toml, crates/workspace/src/materialise.rs, crates/workspace/src/record.rs, crates/workspace/src/variant.rs, crates/workspace/src/lib.rs, crates/workspace/tests/materialise.rs, crates/praps/src/main.rs, crates/xtask/src/policy.rs
- **Depends on:** T-005
- **Acceptance criterion:** INT-0014 AC-1; INT-0009 AC-3 (the working-copy half)
- **Success criterion (EARS):**
  - **WHEN** materialising from tree F to tree T, **THEN** each path **SHALL** first be classified. "Matches F" means bytes, and also mode where the filesystem has an executable bit, equal F's version:
    - **to remove:** tracked by F and not by T, and still matching F;
    - **to write:** in T, with T's entry different from F's, and either matching F or absent where F has none;
    - **done:** already equal to T's entry, or absent where T has none (as after a partial run); nothing to do, and never noted;
    - **kept:** every other path, a user edit, left as it is.

    The F→T diff is the sorted list of paths whose entry differs between F and T. The temporary name for the `n`th path of that diff is `.praps-tmp-<n>` in its directory, a pure function of F and T.
  - **WHEN** the classification is done, **THEN** materialise **SHALL** refuse, like a hazard and before anything is written, any blocking entry. An F-owned entry is a path to remove, a directory holding only paths to remove, or an empty directory where F had files and T has none. A blocking entry is any of the following that is not F-owned:
    - an entry at a path to write;
    - an entry where a path to write needs a directory;
    - a directory where a path to write needs a file;
    - an entry, or any F or T path, equal (after case folding where the filesystem is case-insensitive) to a temporary name this run needs.
  - **WHEN** there is nothing blocking, **THEN** materialise **SHALL**, in this order:
    1. remove the paths to remove, then sync each parent directory touched;
    2. remove the emptied directories (an empty directory holding no T path, where F had files under it), deepest first: sync the directory, remove it, sync its parent;
    3. create missing directories, shallowest first, syncing each parent;
    4. write each path to write through its temporary name: write, set the executable bit, `sync_file`, rename;
    5. sync every touched directory;
    6. remove the record;
    7. sync `.praps`.
  - **WHEN** a record must be written, **THEN** `.praps/working-copy` (holding the from op and the target op) **SHALL** be written through a temporary name, synced and renamed, and `.praps` synced, before the target op is committed. The target op's id **SHALL** be computed from its envelope first.
  - **WHEN** `open` (and so every command but `init`) runs and a record exists, **THEN**:
    - if the target op is the current single head: resume **SHALL** remove the diff's temporary names that exist, then materialise F → T again with the same rules;
    - if a path still blocks, or is now a hazard (for example, a case variant created after the crash): resume **SHALL** finish every other path, drop the record, and add a note naming the paths left as they are, rather than fail;
    - if the target op is absent: resume **SHALL** drop the record and touch no file;
    - if the target op is present but not the head: resume **SHALL** remove the diff's temporary names that exist, drop the record, touch no other file, and add a note.

    Notes are returned by `Workspace::notes()`, and `praps` prints them on stderr as `note: …`. A record that does not decode **SHALL** be a `Corrupt` error. `init` and `init_with_store` **SHALL** never resume.
  - **WHEN** the `variants` feature selects `NoRecord`, **THEN** materialise **SHALL** run without writing the record. The `deps` rule **SHALL** treat `store/variants` and `workspace/variants` as test-only, allowed only through `[dev-dependencies]`.

### T-007: log, undo and restore in workspace
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- **Touches:** crates/workspace/src/lib.rs, crates/workspace/tests/history.rs
- **Depends on:** T-006
- **Acceptance criterion:** INT-0009 AC-1 to AC-3
- **Success criterion (EARS):**
  - **WHEN** `log` runs, **THEN** it **SHALL** snapshot, then return every op, newest first, with its id, parents and working-copy change.
  - **WHEN** `undo` runs, **THEN** it **SHALL**:
    - take H, the single head when the command starts (before its snapshot);
    - snapshot;
    - if H has no parent, fail ("nothing to undo"), having written only the snapshot;
    - otherwise make the target view the view object of H's parent;
    - check hazards and blocking entries;
    - write the record;
    - commit an op whose single parent is the head after the snapshot and whose view is the target view;
    - materialise from the post-snapshot tree to the target tree.

    A second `undo` therefore restores the view before the first.
  - **WHEN** `restore OP` runs with an op id or a unique hex prefix of one, **THEN** it **SHALL** do the same with OP's view as the target. **WHEN** the prefix matches no op or several, **THEN** it **SHALL** fail, listing the candidates, having written only the snapshot.
  - **WHEN** any command runs, **THEN** it **SHALL** remove no op (INT-0009 AC-1).

### T-008: praps log, praps undo and praps restore OP
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md)
- **Touches:** crates/praps/src/main.rs, crates/praps/tests/cli.rs, crates/praps/tests/e2e.rs
- **Depends on:** T-007
- **Acceptance criterion:** INT-0009 AC-3 through the CLI
- **Success criterion (EARS):**
  - **WHEN** `praps log [DIR]` runs, **THEN** it **SHALL** print one line per op, newest first: a 12-hex op id prefix, the working-copy change's change-ID prefix, and the file count. It **SHALL** exit 0.
  - **WHEN** `praps undo [DIR]` runs, **THEN** it **SHALL** undo, print the op restored to, and exit 0. With nothing to undo, it **SHALL** exit 1.
  - **WHEN** `praps restore OP [DIR]` runs, **THEN** it **SHALL** restore and exit 0. With an unknown or ambiguous OP, a hazard or a blocking entry, it **SHALL** exit 1 with the error on stderr.
  - **WHEN** a flag or an extra argument is given, **THEN** `praps` **SHALL** exit 2. `restore` without OP **SHALL** exit 2.
  - **WHEN** a command's workspace has notes, **THEN** `praps` **SHALL** print each on stderr.

### T-009: Reversibility and hazard corpora, the Materialise crash scenario, and the extended edit corpus
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/workspace/tests/history.rs, crates/workspace/tests/hazards.rs, crates/workspace/tests/materialise_crash.rs, crates/workspace/tests/common/mod.rs, crates/workspace/tests/edits.rs, crates/workspace/tests/real_edits.rs, crates/workspace/tests/real_hazards.rs
- **Depends on:** T-007
- **Acceptance criterion:** INT-0009 AC-1 to AC-3; INT-0004 AC-3 and AC-4; INT-0005 AC-1 to AC-3; INT-0007 AC-1 and AC-2; INT-0014 AC-1
- **Success criterion (EARS):**
  - **WHEN** 200 pairs of working copies that differ only in uncommitted content each run `undo`, and separately `restore` of an earlier op, **THEN**:
    - their snapshot ops' trees **SHALL** differ;
    - each pre-state **SHALL** be recovered exactly by `restore` of its snapshot op.

    That is 0 collapsed (AC-2).
  - **WHEN** each op of a corpus covering every op type (init, snapshot, new, undo, restore) is restored, and separately when `undo` runs right after each op type other than `init` (undo right after `init` is the expected failure of `undo_on_the_root_op_fails`), **THEN** the new head's view **SHALL** be the expected view object. The tracked working copy **SHALL** equal that view's working-copy tree byte for byte, with modes where the filesystem has an executable bit (AC-3). The corpus includes file↔directory swaps and executable files whose content changes.
  - **WHEN** the ops are listed after any command sequence, **THEN** every earlier op **SHALL** still be present (AC-1).
  - **WHEN** a case corpus runs on a `SimFs` with case-insensitive rules, **THEN** every hazard **SHALL** be refused with 0 files changed, and every allowed tree **SHALL** materialise. The corpus has at least 12 cases: files, directories (file `a` against directory `A/`), non-ASCII, `σ`/`ς`, `s`/`ſ`, an untracked variant on disk, and a case-only rename (refused, the known limit). With case-sensitive rules, the same trees **SHALL** materialise.
  - **WHEN** a Windows-name corpus (at least 34 names) runs with Windows rules, **THEN** every hazard **SHALL** be refused, 100%. The names cover every reserved base with extensions, case variants and trailing spaces before the extension; every forbidden character; a trailing dot and a trailing space; and a tracked, unchanged hazardous name, which is not refused. Without those rules, the same trees **SHALL** materialise.
  - **WHEN** these run on real filesystems, **THEN**:
    - `restore` of a hazardous tree on real NTFS **SHALL** fail with nothing written;
    - the same tree **SHALL** materialise on ext4.
  - **WHEN** the Materialise scenario runs with a crash at every syscall × 40 seeds × both modes, **THEN**, after recovery and before any further edit:
    - `open` (which resumes) **SHALL** succeed;
    - for a crash inside a command, the tracked working copy **SHALL** equal exactly T if the in-flight `undo` or `restore` op survived, F if it did not, and the last acknowledged state's files for any other command;
    - for a crash inside an edit step, only liveness is checked: the following snapshot **SHALL** make the change equal the working copy;
    - closure and durability **SHALL** hold.

    The scenario runs init, durable edits, snapshot, new, durable edits (including an executable file whose content changes and a file↔directory swap), `undo` and `restore`. Each edit syncs its file and every directory it changes. Every cell **SHALL** reproduce, with the same outcome.
  - **WHEN** a cell's recovery resumes a record, **THEN** a second pass **SHALL** crash at every syscall of that recovering `open` (for 10 seeds per mode), recover, `open` again, and apply the same oracle with 0 violations.
  - **WHEN** the scenario runs with `NoRecord`, **THEN** it **SHALL** report working-copy violations in both modes.
  - **WHEN** the edit corpus (≥ 500 sequences on `SimFs`, ≥ 20 on real NTFS and ext4) also runs `undo` and `restore` of a random earlier op, **THEN** the following **SHALL** hold, with 0 lost edits:
    - the independent model predicts its own op list (count, and for each op the files with contents, modes and `.gitignore` rules, and the parent);
    - the model's count **SHALL** equal `log`'s length before ops are mapped by position;
    - after every step, the change and the tracked working copy **SHALL** match the model.

    Sequences whose step would be refused (a hazard or a blocking entry) **SHALL** assert the refusal and the unchanged state instead.

### T-010: README, credits and backlog
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** README.md, docs/prior-art.md, docs/work/tasks.md
- **Depends on:** T-008
- **Acceptance criterion:** INT-0003 AC-5 (credits)
- **Success criterion (EARS):**
  - **WHEN** the README is read, **THEN** it **SHALL** list `praps log`, `praps undo` and `praps restore OP`.
  - **WHEN** `cargo policy` runs, **THEN** `credits` **SHALL** pass, with `docs/prior-art.md` crediting jj's operation log and stale-working-copy recovery, and Microsoft's naming rules.
  - **WHEN** the backlog is updated, **THEN** work found and not done **SHALL** be a `(backlog)` task with its intents.
