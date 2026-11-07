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
  - AC-3: case-fold collisions are detected before materialising where the filesystem is case-insensitive, and neither file is overwritten. A case-only rename is not a collision;
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
    - T-003: `remove_dir`, `set_executable`, `name_rules` and case-insensitive lookups in `sim`
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
  - **WHEN** `cargo policy` runs, **THEN** `deps` **SHALL** reject any boundary crate other than `sim` that reaches `getrandom`, `rand` or `uuid`, naming the crate and the path. Reaching is taken over:
    - members' `[dependencies]` (and target-specific `[dependencies]`) from their manifests, without dev or build dependencies;
    - registry crates' dependencies from `Cargo.lock`.

    An edge into `sim` is allowed, since `sim` is the crate that owns operating-system randomness.
  - **WHEN** the policy tests run, **THEN** the following **SHALL** fail:
    - a boundary config without the `RandomState` ban;
    - a fixture in which `store` normally depends on a registry crate that depends on `rand`.

    The following **SHALL** pass: a member with `proptest` as a dev-dependency only, `store` → `sim` → `getrandom`, and the real workspace.

### T-003: remove_dir, set_executable, name_rules and case-insensitive lookups in sim
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/sim/src/fs.rs, crates/sim/src/real.rs, crates/sim/src/simfs.rs, crates/sim/src/lib.rs, crates/sim/tests/contract.rs, crates/sim/tests/scan.rs, crates/sim/tests/real_scan.rs, crates/store/tests/common/matrix.rs, crates/workspace/tests/edits.rs, crates/workspace/tests/workspace.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0005 AC-1 and AC-2 for the new syscalls; INT-0004 AC-1 and AC-3 (a faithful case-insensitive model)
- **Success criterion (EARS):**
  - **WHEN** `remove_dir` is called on an empty directory whose own entry changes are all synced, **THEN** it **SHALL** remove it. On a non-empty, missing or not-yet-synced directory, `SimFs` **SHALL** fail and change nothing. (A real filesystem may accept the not-yet-synced case; materialise always syncs a directory before removing it.)
  - **WHEN** `set_executable` is called on a file where the filesystem has an executable bit, **THEN** it **SHALL** set or clear it. Where there is none, it **SHALL** succeed and change nothing.
  - **WHEN** `SimFs` crashes, **THEN**:
    - an unsynced `remove_dir` **SHALL** be kept or reverted by the seed, like any entry change; a reverted one leaves an empty directory;
    - an unsynced executable-bit change **SHALL** be kept or reverted by the seed. The inode keeps a durable bit that `sync_file` updates, and the seed is drawn only when a change is pending, so recovery of existing scenarios is unchanged.

    Both syscalls **SHALL** take no effect at their crash point and count as one syscall each.
  - **WHEN** `name_rules` is read, **THEN** `RealFs` **SHALL** report `{ case_insensitive: true, windows_names: true }` on Windows and both false on Unix, and `SimFs` **SHALL** report its configured value, the same on every host. It is not a syscall.
  - **WHEN** `SimFs` is case-insensitive, **THEN** every lookup **SHALL** match a name ignoring case under both lowercase and uppercase folding, keeping the stored spelling:
    - `create_new` and `create_dir` **SHALL** fail with `AlreadyExists` on a case variant;
    - `rename` onto a case variant of itself **SHALL** change the spelling.
  - **WHEN** the existing helper is used, **THEN** it **SHALL** be named `make_executable` (not a syscall, durable at once), so that it never resolves to the new `Fs::set_executable`.
  - **WHEN** the contract tests run on `SimFs` and on `RealFs` on both systems, **THEN** `remove_dir` and `set_executable` **SHALL** meet the contract.
  - **WHEN** the store and Snapshot crash matrices run, **THEN** their digests **SHALL** be unchanged.

### T-004: Specify writing the working copy, undo, restore and log
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** docs/spec/working-copy.md, docs/intents/INT-0009-reversible-by-default.md, docs/intents/INT-0014-working-copy-and-workspaces.md, docs/intents/INT-0004-windows-and-linux.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:** INT-0009 AC-2 and AC-3; INT-0004 AC-3 and AC-4 (the rules a reviewer checks the code against)
- **Success criterion (EARS):**
  - **WHEN** the spec is read, **THEN** `working-copy.md` **SHALL** state each rule T-005 to T-007 implement:
    - the record, its atomic write, resume and drop;
    - the per-path materialise rule and blocking entries;
    - the deterministic temporary names;
    - directory creation and removal, and the sync order;
    - the hazard rules: their scope and the full reserved list;
    - `undo`, `restore` and `log`;
    - which commands resume.
  - **WHEN** the intents are read, **THEN**:
    - INT-0009's Consequences **SHALL** state the precise `undo` definition;
    - INT-0014's **SHALL** state that `init` never resumes and that temporary names are not ignored;
    - INT-0004's **SHALL** record that the case rules come from the platform, not from a probe of each directory, with its limits (WSL drvfs, case-sensitive NTFS directories);
    - backlog **SHALL** gain probe-based detection.

### T-005: The hazard check
- **Intent:** [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/workspace/src/hazard.rs, crates/workspace/src/lib.rs, crates/workspace/tests/hazards.rs
- **Depends on:** T-003, T-004
- **Acceptance criterion:** INT-0004 AC-3 and AC-4
- **Success criterion (EARS):**
  - **WHEN** the check runs, **THEN** it **SHALL** consider only the paths materialise would create or rewrite (and the directories it would create), against the filesystem's `name_rules`. It is a pure function of F, T, the existing directory listings, and the rules.
  - **WHEN** the filesystem is case-insensitive, **THEN** a hazard **SHALL** be either of the following, two names being equal when they match under `to_lowercase` or under `to_uppercase`:
    - two T paths equal ignoring case;
    - a written path equal ignoring case to an existing entry with another spelling, unless F tracks that entry and T removes it (a case-only rename).
  - **WHEN** Windows names apply, **THEN** a hazard **SHALL** be a component with any of the following:
    - a character in `<>:"|?*` or 0–31;
    - a trailing `.` or space;
    - a reserved base: the part before the first `.`, with trailing spaces removed, compared ignoring case, against `CON`, `PRN`, `AUX`, `NUL`, `CONIN$`, `CONOUT$`, `COM0`–`COM9`, `LPT0`–`LPT9`, and `COM`/`LPT` with ¹, ² or ³.
  - **WHEN** any hazard exists, **THEN** the command **SHALL** fail with an error naming every offending path. Its only write **SHALL** be its opening snapshot: no record, no op, no file.

### T-006: Materialise, the working-copy record and resume
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** crates/workspace/Cargo.toml, crates/workspace/src/materialise.rs, crates/workspace/src/record.rs, crates/workspace/src/variant.rs, crates/workspace/src/lib.rs, crates/workspace/tests/materialise.rs, crates/xtask/src/policy.rs
- **Depends on:** T-005
- **Acceptance criterion:** INT-0014 AC-1; INT-0009 AC-3 (the working-copy half)
- **Success criterion (EARS):**
  - **WHEN** materialising from tree F to tree T, **THEN** materialise **SHALL**, in this order:
    1. check for blocking entries before anything is written. A blocking entry is an untracked or ignored entry at a T path that F does not track, an entry where T needs a directory, or a directory where T needs a file. Any blocking entry **SHALL** fail the command, like a hazard;
    2. remove every path that F tracks and T does not, if its bytes and mode equal F's version; otherwise leave it;
    3. write every T path whose current bytes and mode equal F's version (or which is absent where F has none). Each is written through the temporary name `.praps-tmp-<n>` in its directory, where `n` is its index among the sorted paths to write. The file is written, synced and renamed, and the bit set with `set_executable`;
    4. leave any other path as it is;
    5. create missing directories;
    6. remove only the directories this materialise emptied;
    7. sync every touched file and directory;
    8. remove the record;
    9. sync `.praps`.
  - **WHEN** a record must be written, **THEN** `.praps/working-copy` **SHALL** be written through a temporary name, synced and renamed, and `.praps` synced, before the target op is committed. The record holds the from op and the target op. The target op's id **SHALL** be computed from its envelope first.
  - **WHEN** `open`, or any command but `init`, starts and a record exists, **THEN** it **SHALL**:
    - if the target op is the current single head: remove the record's temporary names, finish materialising F → T, then remove the record;
    - if the target op is absent: drop the record, touching no file;
    - if the target op is present but not the head: drop the record, touching no file, and leave a note in the command's report.

    A record that does not decode **SHALL** be a `Corrupt` error. `init` and `init_with_store` **SHALL** never resume.
  - **WHEN** the `variants` feature selects `NoRecord`, **THEN** materialise **SHALL** run without writing the record. The `deps` rule **SHALL** treat (crate, feature) pairs, `store/variants` and `workspace/variants`, as test-only, allowed only through `[dev-dependencies]`.
- **Notes:** No name is ever silently ignored. Temporary names exist only while a record names them, and resume removes exactly those.

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
  - **WHEN** `restore OP` runs with an op id or a unique hex prefix of one, **THEN** it **SHALL** do the same with OP's view as the target. **WHEN** the prefix matches no op or several, **THEN** it **SHALL** fail listing the candidates, having written only the snapshot.
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
  - **WHEN** each op of a corpus covering every op type (init, snapshot, new, undo, restore) is restored, and separately when `undo` runs right after each op type, **THEN** the new head's view **SHALL** be the expected view object, and the tracked working copy **SHALL** equal that view's working-copy tree byte for byte, with modes (AC-3).
  - **WHEN** the ops are listed after any command sequence, **THEN** every earlier op **SHALL** still be present (AC-1).
  - **WHEN** a case corpus runs on a case-insensitive `SimFs`, **THEN** every hazard **SHALL** be refused with 0 files changed, and every allowed case **SHALL** materialise. The corpus has at least 12 cases: files, directories, non-ASCII, `σ`/`ς`, `s`/`ſ`, an untracked variant on disk, and a case-only rename by `undo`, which is allowed.
  - **WHEN** a Windows-name corpus (at least 34 names) runs with Windows rules, **THEN** every hazard **SHALL** be refused, 100%. The names cover every reserved base with extensions, case variants and trailing spaces before the extension; every forbidden character; a trailing dot and a trailing space; and a tracked, unchanged hazardous name, which is not refused. Without those rules, the same trees **SHALL** materialise.
  - **WHEN** these run on real filesystems, **THEN**:
    - `restore` of a hazardous tree on real NTFS **SHALL** fail with nothing written;
    - an `undo` of a case-only rename on NTFS **SHALL** succeed;
    - the hazardous tree **SHALL** materialise on ext4.
  - **WHEN** the Materialise scenario (init, edits, snapshot, new, edits, `undo`, `restore`) runs with a crash at every syscall × 40 seeds × both modes, **THEN**, after recovery and before any further edit:
    - `open` (which resumes) **SHALL** succeed;
    - the tracked working copy **SHALL** equal exactly the oracle's expected tree. That is T if the in-flight `undo` or `restore` op survived, F (the files when that command started) if it did not, and, for a crash in any other command, the files of the last acknowledged state;
    - closure and durability of acknowledged ops **SHALL** hold;
    - a following snapshot **SHALL** make the change equal the working copy.

    Every cell **SHALL** reproduce, with the same outcome.
  - **WHEN** the scenario runs with `NoRecord`, **THEN** it **SHALL** report working-copy violations in both modes.
  - **WHEN** the edit corpus (≥ 500 sequences on `SimFs`, ≥ 20 on real NTFS and ext4) also runs `undo` and `restore` of a random earlier op, **THEN** the following **SHALL** hold, with 0 lost edits:
    - the independent model predicts its own op list (count, and for each op the files with contents, modes and `.gitignore` rules, and the parent);
    - the model's count **SHALL** equal `log`'s length before ops are mapped by position;
    - after every step, the change and the tracked working copy **SHALL** match the model.

### T-010: README, credits and backlog
- **Intent:** [INT-0009](../../../intents/INT-0009-reversible-by-default.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md)
- **Touches:** README.md, docs/prior-art.md, docs/work/tasks.md
- **Depends on:** T-008
- **Acceptance criterion:** INT-0003 AC-5 (credits)
- **Success criterion (EARS):**
  - **WHEN** the README is read, **THEN** it **SHALL** list `praps log`, `praps undo` and `praps restore OP`.
  - **WHEN** `cargo policy` runs, **THEN** `credits` **SHALL** pass, with `docs/prior-art.md` crediting jj's operation log and stale-working-copy recovery, and Microsoft's naming rules.
  - **WHEN** the backlog is updated, **THEN** work found and not done **SHALL** be a `(backlog)` task with its intents.
