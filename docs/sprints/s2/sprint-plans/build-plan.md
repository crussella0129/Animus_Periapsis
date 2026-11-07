# Sprint 2 Build Plan

## Intents
- [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md) — state: planned; acceptance criteria covered: AC-1 (auto-snapshot: every modification between two commands is captured by the second, 0 lost edits over a randomized edit-and-command corpus on Windows and Linux) and AC-2 (ignored paths are never snapshotted; the matching rules are documented and tested). `praps new` starts a fresh working-copy change on top of the current one without touching files, so `status` has a real parent to compare against. AC-3 (eight workspaces), AC-4 (API-only workspaces), AC-5 (the stat cache's time budget) and AC-6 (writing conflicted files) need writing to the working copy or several workspaces, and come in later sprints.
- [INT-0006](../../../intents/INT-0006-format-v1.md) — state: active; acceptance criteria covered: AC-1, AC-3 and AC-5 for the provisional `changeset` and `view` bodies and the revised `operation` body (round trip, golden files, total decoding); AC-6's first half (each body specified in `docs/spec/format-v1.md` before or with its code). All three are *provisional*, so they are excluded from the owner's AC-6 sign-off until the Phase 1 exit gate. The `operation` body gains the view hash (backlog T-012, in part).
- [INT-0007](../../../intents/INT-0007-crash-safe-storage.md) — state: active; acceptance criteria covered: AC-1 and AC-2 re-established when closure follows references inside objects (op → view → changesets → blobs), and AC-4 (`fsck` reports missing references of every kind). AC-5 (four writing processes) stays in backlog T-011.
- [INT-0004](../../../intents/INT-0004-windows-and-linux.md) — state: active; acceptance criteria covered: AC-1 for this sprint's code; AC-2 (a working copy snapshotted on NTFS and on ext4 gives the same changeset id); AC-7 (a working-copy name with no UTF-8 form is reported by an escaped form and left alone, on ext4 and NTFS). AC-3 and AC-4 concern writing a working copy and wait for checkout.
- [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md) — state: active; acceptance criteria covered: AC-1 to AC-3 for the new scan syscall, the redesigned two-writer race and the new Snapshot scenario, with a new negative control for the op's view re-sync; AC-4 for the new crate's Clippy config.
- [INT-0003](../../../intents/INT-0003-engineering-baseline.md) — state: active; acceptance criteria covered: AC-1 and AC-8 for the new `workspace` crate (no `unsafe`, one-word name); AC-5's credits for jj's working copy, Mononoke's Bonsai changesets, `ignore` and `getrandom`.
- [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md) — state: proposed and active; acceptance criteria covered: only through T-001 (backlog T-010), which records Sprint 1's GitHub Actions runs and whichever owner sign-offs have actually been given.
- Context, not advanced: [INT-0009](../../../intents/INT-0009-reversible-by-default.md) (every snapshot is a new op over the previous view, so earlier working-copy states stay in the op log); [INT-0010](../../../intents/INT-0010-lock-free-concurrency.md) (until its canonical merge exists, a command that finds several op heads stops and names them; its Consequences record the gap); [INT-0011](../../../intents/INT-0011-change-identity-and-rewriting.md) (`new` mints a change ID; rewriting semantics stay Phase 2); [INT-0013](../../../intents/INT-0013-derived-data.md) (trees are derived from changesets and never stored); [INT-0034](../../../intents/INT-0034-any-content.md) (snapshots store bytes exactly, with no normalisation).

## Schema Tree
- Sprint Goal: Periapsis records a working copy: `praps init`, `praps new`, a snapshot at the start of every command, and `praps status`
  - Carried forward
    - T-001: Record Sprint 1's GitHub Actions runs and owner sign-offs (backlog T-010)
  - Format
    - T-002: Specify and implement the provisional `changeset` and `view` kinds and repository paths
  - Storage
    - T-003: The operation names its view; closure through references inside objects
  - Simulation
    - T-004: A scan syscall, an executable-bit capability and operating-system randomness in `sim`
  - Working copy
    - T-005: The `workspace` crate: init, snapshot, new and status
    - T-006: `praps init`, `praps new` and `praps status`
    - T-007: Snapshot crash scenario and edit corpora
  - Documentation
    - T-008: Working-copy spec, README, credits and backlog

## Execution Sequence

### T-001: Record Sprint 1's GitHub Actions runs and owner sign-offs
- **Intent:** [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0001](../../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../../intents/INT-0002-typed-acceptance-criteria.md)
- **Touches:** docs/sprints/s2/sprint-tests/ci-sprint1.md, docs/intents/INT-0001-periapsis-charter.md, docs/intents/INT-0002-typed-acceptance-criteria.md, docs/intents/INT-0003-engineering-baseline.md, docs/intents/INT-0004-windows-and-linux.md, docs/intents/INT-0006-format-v1.md, docs/work/tasks.md
- **Depends on:** (none)
- **Acceptance criterion:** INT-0003 AC-2 to AC-4 and INT-0006 AC-3 (the GitHub matrix halves of Sprint 1's checkpoint); INT-0001 AC-1, INT-0002 AC-1, INT-0003 AC-5, INT-0006 AC-6 and INT-0004 AC-5 (owner sign-offs)
- **Success criterion (EARS):**
  - **WHEN** the record is written, **THEN** `ci-sprint1.md` **SHALL** name, for each of runs 42329906757, 42329902655 and 42375204698, its head SHA, event, conclusion, every job with its conclusion and runner OS, the `rustc` version, and the crash-matrix and test counts printed in each test leg, all taken from `gh run view` output.
  - **WHEN** an owner sign-off has not been given in the conversation, **THEN** the record **SHALL** say it is still pending and backlog T-010 **SHALL** be replaced by a carried-forward task naming the pending sign-offs; no sign-off **SHALL** be recorded as given without the owner's words.
- **Notes:** The Sprint 0 record `docs/sprints/s1/sprint-tests/ci-sprint0.md` is the model.

### T-002: Specify and implement the provisional changeset and view kinds and repository paths
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** docs/spec/format-v1.md, crates/codec/src/lib.rs, crates/codec/src/envelope.rs, crates/codec/src/error.rs, crates/codec/src/changeset.rs, crates/codec/src/view.rs, crates/codec/src/path.rs, crates/codec/tests/golden.rs, crates/codec/tests/golden/v1/*.bin, crates/codec/tests/spec.rs, crates/codec/tests/properties.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 AC-6 (specified before code; provisional), AC-1 (canonical round trip), AC-3 (golden files), AC-5 (total decoding)
- **Success criterion (EARS):**
  - **WHEN** the spec commit is made, before or with the code, **THEN** `docs/spec/format-v1.md` **SHALL**:
    - give `0x02 changeset` and `0x04 view` the status `defined` and mark both provisional;
    - specify `changeset` = `parents: list<hashref>` ‖ `change_id: 16 bytes` ‖ `changes: map<path, entry>` ‖ `description: bytes`, where an entry is `0x00 delete` or `0x01 put` ‖ `blob: hashref` ‖ `mode: u8`, with mode `0x00` regular, `0x01` executable, and `0x02` symlink reserved and rejected;
    - specify `view` = `heads: set<hashref>` ‖ `workspaces: map<string, hashref>`;
    - specify `path` as a non-empty UTF-8 string of `/`-separated components, none empty, `.` or `..`, with no NUL and no `\`;
    - include worked examples for a root changeset, a changeset with a put and a delete, and a view.
  - **WHEN** any `Changeset` or `View` value is encoded and decoded, **THEN** the codec **SHALL** return the same value, and every byte string it accepts **SHALL** re-encode to the same bytes (proptest, 10,000 cases each).
  - **WHEN** a body has unsorted or duplicate set elements or map keys, **THEN** decoding **SHALL** fail with the existing `UnsortedOrDuplicateKey` error.
  - **WHEN** a body has an unknown entry tag, an unknown mode, the reserved symlink mode, an invalid path or trailing bytes, **THEN** decoding **SHALL** fail with a new named error for each: `UnknownEntryTag`, `UnknownMode`, `ReservedMode`, `InvalidPath`, or the existing trailing-bytes error.
  - **WHEN** `ObjectKind::from_id` reads `0x02` or `0x04`, **THEN** it **SHALL** return that kind; `0x05` **SHALL** still fail with `ReservedObjectKind`.
  - **WHEN** `RepoPath::new` is given each invalid form in a fixture list, **THEN** it **SHALL** reject it, and **SHALL** accept the valid forms.
  - **WHEN** random byte strings are decoded as `changeset` or `view` bodies, **THEN** decoding **SHALL** never panic (100,000 cases each).
  - **WHEN** the golden and spec suites run, **THEN** the new golden files **SHALL** match their corpus values byte for byte, the spec's worked examples **SHALL** match them, and the registry table **SHALL** match the code.
- **Notes:**
  - Reuse `wire.rs`'s set, map, string and hashref encodings and the canonical-order checks.
  - The `change_id` field is the change-ID cache INT-0006 names; INT-0011 (Phase 2) owns its rewriting semantics.
  - `operation` is unchanged in this task, so every existing user of `Operation::objects` keeps compiling and the workspace stays green; T-003 changes it.

### T-003: The operation names its view; closure through references inside objects
- **Intent:** [INT-0006](../../../intents/INT-0006-format-v1.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md)
- **Touches:** docs/spec/format-v1.md, docs/spec/storage-durability.md, crates/codec/src/lib.rs, crates/codec/src/operation.rs, crates/codec/src/references.rs, crates/codec/tests/golden.rs, crates/codec/tests/golden/v1/operation-root.bin, crates/codec/tests/golden/v1/operation-merge.bin, crates/codec/tests/spec.rs, crates/codec/tests/properties.rs, crates/store/src/lib.rs, crates/store/src/fsck.rs, crates/store/src/variant.rs, crates/store/tests/store.rs, crates/store/tests/fsck.rs, crates/store/tests/crash.rs, crates/store/tests/real_store.rs, crates/store/tests/common/matrix.rs, crates/praps/tests/cli.rs
- **Depends on:** T-002
- **Acceptance criterion:** INT-0006 AC-6, AC-1, AC-3, AC-5 for the revised `operation` body; INT-0007 AC-1 (recovered ops closed under ancestry, the objects they reach present), AC-2 (acknowledged ops survive), AC-4 (`fsck`, 100% detection, 0 false reports); INT-0005 AC-2 and AC-3
- **Success criterion (EARS):**
  - **WHEN** this task's commit is made, **THEN** `format-v1.md` **SHALL** specify `operation` = `parents: set<hashref>` ‖ `view: hashref`, replacing Sprint 1's provisional `objects`, with revised worked examples, and `operation-root.bin` and `operation-merge.bin` **SHALL** change in the same commit as the encoder, as INT-0006 allows for provisional kinds.
  - **WHEN** any `Operation` is encoded and decoded, or random bytes are decoded as one, **THEN** the round-trip, re-encoding (10,000 cases) and never-panic (100,000 cases) properties **SHALL** hold.
  - **WHEN** `codec::references` reads a changeset, view, operation or blob envelope, **THEN** it **SHALL** return exactly the ids the body names: changeset parents and put blobs; view heads and workspace changesets; operation parents and view; nothing for a blob.
  - **WHEN** `put_object` is given an object one of whose references is absent or does not hash to its id, **THEN** it **SHALL** fail with `NotFound` and write nothing.
  - **WHEN** `commit_op` is given an op whose view is absent or damaged, **THEN** it **SHALL** fail with `NotFound` and write nothing; **WHEN** the view is present, **THEN** it **SHALL** sync the view file and `objects/`, and each parent op and `ops/`, before writing the op.
  - **WHEN** `fsck` runs on a store with a changeset whose blob is missing, a view whose changeset is missing, or an op whose view is missing, **THEN** it **SHALL** report each one as a missing reference.
  - **WHEN** the crash matrix runs with scenarios that build blobs, changesets and views, **THEN** the correct store **SHALL** have 0 violations, with closure and durability following references, and the race **SHALL** run in at least one cell per mode.
  - **WHEN** the `UnsyncedObjects` variant suppresses every data sync of an object file (its own and `commit_op`'s re-sync), **THEN** the matrix **SHALL** report durability and torn violations in strict mode; the `PointerFile`, `UnsyncedInit` and `WriteInPlace` controls **SHALL** still be caught.
  - **WHEN** the new `UnsyncedReferences` variant skips `commit_op`'s re-sync of the view with `objects/` and of the parent ops with `ops/`, **THEN** the redesigned TwoWriters scenario **SHALL** report closure violations in strict mode, both in cells where B's op names A's pending view and in cells where it names A's pending op as a parent.
- **Notes:**
  - **Closure through reachability.** Closure (INT-0007 AC-1) concerns the ops that survive and what they reach. `put_object` checks that each direct reference is present and valid, but does not re-sync it. No op can name an object before that object's put is acknowledged, and an acknowledged put is durable with everything it reaches, by induction. An unacknowledged object that survives a crash with a lost reference is unreachable from every op. `fsck` reports a missing reference only for objects reachable from an op, and lists unreachable objects with missing references as leftovers, like `tmp-` files.
  - **What `commit_op` re-syncs.** A writer may name another writer's object or op that is present but not yet durable: a view in `objects/`, or a parent op in `ops/`. `commit_op` re-syncs the view with `objects/`, and each parent with `ops/`, before writing. The parent re-sync is needed even though it is in the same directory: a crash between the new op's `rename` and its own `sync_dir(ops)` can recover the in-flight op while losing an unsynced parent. INT-0007's Consequences and `storage-durability.md` state this argument.
  - **Redesigned TwoWriters.** Writer A puts blobs, a changeset and a view, then commits an op. B fires at a fixed boundary in every seed:
    - in even seeds, after A's `rename` of the view and before its `sync_dir(objects)`, B commits an op naming A's view, which it finds present but not durable;
    - in odd seeds, after A's `rename` of its op and before its `sync_dir(ops)`, B commits an op whose parent is A's op.

    B never writes the view or A's op itself. The matrix asserts that both race kinds run in every mode.

### T-004: A scan syscall, an executable-bit capability and operating-system randomness in sim
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md)
- **Touches:** crates/sim/Cargo.toml, crates/sim/clippy.toml, crates/sim/src/lib.rs, crates/sim/src/fs.rs, crates/sim/src/real.rs, crates/sim/src/simfs.rs, crates/sim/tests/contract.rs, crates/sim/tests/real_scan.rs, crates/store/tests/common/matrix.rs, crates/xtask/src/policy.rs, Cargo.toml, Cargo.lock
- **Depends on:** T-003 (both touch `matrix.rs`)
- **Acceptance criterion:** INT-0005 AC-1 and AC-2 for the new syscall; INT-0004 AC-7 (names with no UTF-8 form) and AC-1
- **Success criterion (EARS):**
  - **WHEN** `Fs::scan(dir)` is called, **THEN** it **SHALL** return the directory's entries sorted by their name's display form, each with a `Name` (`Utf8(String)` or `Raw(RawName)`) and a kind (`File { executable }`, `Dir`, `Symlink` or `Other`), without following symlinks.
  - **WHEN** a name has no UTF-8 form (non-UTF-8 bytes on ext4, an unpaired surrogate on NTFS), **THEN** `RealFs::scan` **SHALL** return it as `Name::Raw` and never as a lossy `Utf8` name.
  - **WHEN** any name is displayed, **THEN** `Name::display` **SHALL** escape `\` as `\\` and control characters as `\u{NN}`. A `Raw` name **SHALL** also escape each invalid byte as `\xNN` (ext4) or each unpaired surrogate as `\u{DNNN}` (NTFS), so a raw name's display never equals any UTF-8 name's display.
  - **WHEN** `Fs::has_executable_bit()` is called, **THEN** `RealFs` **SHALL** return `true` on Unix and `false` on Windows, and `SimFs` **SHALL** return its configured value (default `true`, the same on every host). **WHEN** the bit is supported and a file's owner-execute bit is set, **THEN** `scan` **SHALL** report `executable: true`.
  - **WHEN** `SimFs` is used, **THEN** it **SHALL** model executable bits, symlinks, `Other` entries and raw names, created through test helpers that are not syscalls. A raw name **SHALL** be stored as its bytes or code units, independent of the host. `scan` **SHALL** take no effect at its crash point and count as one syscall.
  - **WHEN** the contract tests for `scan` run, **THEN** they **SHALL** pass on `SimFs`, and on `RealFs` on Windows and Linux. On Windows a symlink case without the privilege to create one **SHALL** be reported as skipped, not passed, and `Other` entries **SHALL** be tested on Unix only.
  - **WHEN** `sim::os_seed()` is called twice, **THEN** it **SHALL** return operating-system randomness from `getrandom`.
  - **WHEN** `cargo policy` runs, **THEN** the `deps` rule **SHALL** reject `getrandom` as a dependency of any crate but `sim`. `sim/clippy.toml` **SHALL** ban `getrandom::fill`, `getrandom::u32` and `getrandom::u64`, which only `real.rs` may switch off, as for `std::fs`.
  - **WHEN** the store matrix computes its digest, **THEN** it **SHALL** cover the new node kinds, and the digest **SHALL** still be equal on Windows and Linux.
- **Notes:** `getrandom` 0.4.3 joins the workspace dependencies; `cargo deny` and the `credits` rule must pass.

### T-005: The workspace crate: init, snapshot, new and status
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0004](../../../intents/INT-0004-windows-and-linux.md), [INT-0013](../../../intents/INT-0013-derived-data.md)
- **Touches:** crates/workspace/Cargo.toml, crates/workspace/clippy.toml, crates/workspace/src/lib.rs, crates/workspace/src/ignore.rs, crates/workspace/src/tree.rs, crates/workspace/tests/workspace.rs, crates/workspace/tests/ignore.rs, crates/workspace/tests/fixtures/ignore-corpus.txt, crates/workspace/tests/real_workspace.rs, crates/xtask/src/policy.rs, Cargo.toml, Cargo.lock
- **Depends on:** T-002, T-003, T-004
- **Acceptance criterion:** INT-0014 AC-1 (auto-snapshot) and AC-2 (ignore rules); INT-0004 AC-2 and AC-7
- **Success criterion (EARS):**
  - **WHEN** `Workspace::init(fs, root, seed)` runs, **THEN** it **SHALL**:
    - create `root` if missing and sync its parent;
    - create `root/.praps` if missing, then sync `.praps` and `root` so the `.praps` entry is durable;
    - run `Store::init` on `root/.praps/store`;
    - pass the store to `init_with_store`.
  - **WHEN** `Workspace::init_with_store(store, root, seed)` runs on a store with no op, **THEN** it **SHALL** commit a root op whose view has heads {c} and workspace `default` → c, where c is an empty changeset with no parents and a change ID drawn from `seed`. **IF** the store already has an op, it **SHALL** report the existing repository and write nothing. An interrupted `init` is therefore completed by running `init` again. Tests pass a store built with `Store::init_variant` here.
  - **WHEN** `Workspace::open(fs, root, seed)` runs, **THEN** it **SHALL** make `.praps` and `root` durable as `init` does, then open the store. **WHEN** it finds no op, **THEN** it **SHALL** fail with an error saying the repository is not initialised. **WHEN** it finds more than one op head, **THEN** it and every command **SHALL** fail with an error naming every head and write nothing.
  - **WHEN** a workspace is built by any constructor, **THEN** every change ID it mints (`init`, `new`) **SHALL** come from an `Rng` seeded by the constructor's `seed`. `praps` passes `sim::os_seed()`, and tests pass fixed seeds, so runs are deterministic (INT-0005 AC-1).
  - **WHEN** `snapshot` runs and the working copy's tracked content differs from the working-copy change's tree, **THEN** it **SHALL** put the new blobs, then a changeset with the same change ID and parents and the path map of the change against its parent's tree, then a view, then an op whose parent is the single head. **WHEN** nothing differs, **THEN** it **SHALL** write no file.
  - **WHEN** `snapshot` writes a new changeset, **THEN** the new view's `heads` **SHALL** contain it in place of the changeset it replaces, and no other head **SHALL** change.
  - **WHEN** `new` runs, **THEN** it **SHALL** snapshot, then commit a changeset with the working-copy change as its only parent, a fresh change ID and no changes. The view **SHALL** point `default` at it and replace the parent in `heads`. No working-copy file **SHALL** be written.
  - **WHEN** a path is ignored, **THEN** it **SHALL** never be stored. A path is ignored when it lies under `.praps/` or `.git/` at the root, or when the `.gitignore` files on its directory chain match it. The matching rules are:
    - one matcher per `.gitignore`, rooted at that file's directory's repository path (`""` at the top), and always given full repository paths, so the matcher strips exactly its own directory;
    - the deepest file with a match decides;
    - within one file the last match wins;
    - matching is case-sensitive on every system;
    - an ignored directory is not descended into, except to visit tracked paths beneath it, and nothing under it can be re-included.
  - **WHEN** a tracked file becomes ignored, or lies under a directory that becomes ignored, **THEN** it **SHALL** stay tracked, with its edits still snapshotted, until it is deleted.
  - **WHEN** a scanned name has no UTF-8 form, or an entry is a symlink or other non-regular file, **THEN** `snapshot` **SHALL** neither track nor change it and **SHALL** return it in its report with the reason and display form. Its directory's other files **SHALL** still be snapshotted.
  - **WHEN** the filesystem has an executable bit and a file's bit changes, **THEN** the snapshot **SHALL** record the new mode. **WHEN** it has none, **THEN** a tracked file **SHALL** keep the mode the working-copy change, or else its parent, last recorded, and a new file **SHALL** be regular.
  - **WHEN** `status` runs, **THEN** it **SHALL** snapshot first and return, sorted, the added, modified and deleted paths of the working-copy change against its parent's tree (empty for a root change), plus the reported entries.
  - **WHEN** a working copy changes a file's content, mode, or presence after `new`, **THEN** `status` **SHALL** list it as `M`, `M` or `A`/`D` respectively, and the changeset **SHALL** hold the matching `put` or `delete` entry.
  - **WHEN** the same nested tree is snapshotted on NTFS and on ext4 with a fixed seed, **THEN** the working-copy changeset id **SHALL** equal one constant recorded in the test.
  - **WHEN** a changeset with more than one parent is met while deriving a tree, **THEN** the workspace **SHALL** fail with an error until INT-0012.
  - **WHEN** `cargo policy` runs, **THEN** `fs-boundary` and `names` **SHALL** cover the `workspace` crate, and `fs-boundary` **SHALL** require that any boundary crate depending on `ignore` bans its filesystem-reading APIs in its `clippy.toml`. Those APIs are `GitignoreBuilder::add`, `Gitignore::new`, `Gitignore::global`, `GitignoreBuilder::build_global`, `gitconfig_excludes_path`, `WalkBuilder` and `Walk`.
- **Notes:**
  - `gitconfig_excludes_path` also reads the real filesystem and joins the banned list.
  - The tree of a change is derived from its changeset chain (INT-0013).
  - `ignore::gitignore::GitignoreBuilder::add_line` is fed lines read through `Fs`. Global and `.git/info/exclude` files are not read, so the result is the same on every machine.
  - The corpus fixture has two expected columns:
    - the matcher's verdict, from `git -c core.ignorecase=false check-ignore --no-index`;
    - the snapshot outcome, tracked or not, which accounts for files tracked before. Rows that depend on tracking or re-inclusion are cross-checked with `git status --ignored` in a scratch repository.

    The Git version is recorded in the fixture. The corpus includes:
    - a tracked file under an ignored directory;
    - a `!` rule under an excluded parent;
    - a nested `.gitignore` overriding its parent's.

### T-006: praps init, praps new and praps status
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md)
- **Touches:** crates/praps/Cargo.toml, crates/praps/src/main.rs, crates/praps/tests/cli.rs, Cargo.lock
- **Depends on:** T-005
- **Acceptance criterion:** INT-0014 AC-1 through the CLI
- **Success criterion (EARS):**
  - **WHEN** `praps init [DIR]` runs, **THEN** it **SHALL** create a repository in DIR (default: the current directory) and exit 0. On an existing repository it **SHALL** say so and exit 0 without writing.
  - **WHEN** `praps new [DIR]` runs in a repository, **THEN** it **SHALL** start a new working-copy change, print its change ID, and exit 0.
  - **WHEN** `praps status [DIR]` runs in a repository, **THEN** it **SHALL** print `A`, `M` or `D` and the path's display form for each change, and a `skipped` line with the reason and display form for each reported entry, then exit 0. Outside a repository, or with several op heads, it **SHALL** exit 1.
  - **WHEN** `praps fsck DIR` is given a working copy, **THEN** it **SHALL** check `DIR/.praps/store`. Given a bare store it **SHALL** behave as in Sprint 1.
  - **WHEN** an unknown command, a flag or an extra argument is given, **THEN** `praps` **SHALL** print usage and exit 2.
- **Notes:**
  - `praps init`, `praps new` and `praps status` open the workspace and snapshot first. `praps fsck` opens the store read-only and does not snapshot, and `praps init` on an existing repository writes nothing. An edit made before either is captured by the next command that snapshots. INT-0014's Consequences record this.
  - The `praps` commands take no flags, following the convention in `crates/praps/src/main.rs`. That convention is not one of INT-0003 AC-9's commands; its intent home is INT-0022 (agent and human interfaces), which is not advanced this sprint.

### T-007: Snapshot crash scenario and edit corpora
- **Intent:** [INT-0005](../../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0007](../../../intents/INT-0007-crash-safe-storage.md)
- **Touches:** crates/workspace/Cargo.toml, crates/workspace/tests/crash.rs, crates/workspace/tests/edits.rs, crates/workspace/tests/real_edits.rs
- **Depends on:** T-005
- **Acceptance criterion:** INT-0005 AC-1 to AC-3; INT-0014 AC-1 (0 lost edits); INT-0007 AC-1 and AC-2
- **Success criterion (EARS):**
  - **WHEN** the Snapshot scenario runs with a crash at every syscall × 40 seeds × both modes, **THEN** after recovery there **SHALL** be 0 violations of:
    - closure;
    - durability of acknowledged ops;
    - recency: with init unacknowledged, the store may have no op; otherwise the single head's view's `default` change has the tree of the last acknowledged snapshot, or of the in-flight one;
    - liveness: `Workspace::init` followed by `snapshot` succeeds on the recovered state, and its tree equals the recovered working copy.

    The scenario is `init`, then four rounds of scripted edits each followed by a snapshot, with a `new` after the second round.
  - **WHEN** a Snapshot cell is run twice, **THEN** its recovered state, trace and outcome **SHALL** be identical.
  - **WHEN** the Snapshot scenario runs with the `UnsyncedObjects` variant through `Workspace::from_store`, **THEN** it **SHALL** report durability or closure violations in strict mode.
  - **WHEN** at least 500 seeded sequences of random creates, edits, deletes, executable-bit changes, `new`s and new ignore rules run on `SimFs`, each step followed by a snapshot, **THEN** after every snapshot the working-copy change's tree **SHALL** equal the expected tree: the regular files that are not ignored or were tracked before, with their modes under the filesystem's executable-bit rule, byte for byte. That means 0 lost edits.
  - **WHEN** a smaller corpus (at least 20 sequences, with the same oracle) runs on `RealFs` in a temporary directory on Windows and on Linux, **THEN** it **SHALL** also find 0 lost edits.
- **Notes:** The workspace tests enable `store/variants` through a dev-dependency only, which the `deps` rule allows. The crash harness is written in `crates/workspace/tests/` against `sim`'s public crash API, not shared with `store/tests/common/matrix.rs` by a `#[path]` include.

### T-008: Working-copy spec, README, credits and backlog
- **Intent:** [INT-0014](../../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0003](../../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../../intents/INT-0006-format-v1.md)
- **Touches:** docs/spec/working-copy.md, docs/SUMMARY.md, README.md, docs/prior-art.md, docs/work/tasks.md
- **Depends on:** T-005, T-006
- **Acceptance criterion:** INT-0014 AC-2 (rules documented); INT-0003 AC-5 (credits)
- **Success criterion (EARS):**
  - **WHEN** the docs are built, **THEN** `docs/spec/working-copy.md` **SHALL** state:
    - the repository layout;
    - the snapshot and `new` sequences;
    - the ignore rules: matcher composition, precedence, case sensitivity, the always-ignored directories, which Git files are not read, and tracked files under ignored paths;
    - reported entries and the display escaping;
    - the executable-bit rule;
    - the several-heads stop;
    - the costs deferred to the stat cache.

    `SUMMARY.md` **SHALL** link it.
  - **WHEN** the README is read, **THEN** it **SHALL** list the `workspace` crate and `praps init`, `praps new`, `praps status` and `praps fsck`.
  - **WHEN** `cargo policy` runs, **THEN** the `credits` rule **SHALL** pass, with `docs/prior-art.md` crediting jj's working-copy model, Mononoke's Bonsai changesets, `ignore` and `getrandom`.
  - **WHEN** the sprint's backlog is updated, **THEN** T-012 **SHALL** be revised to the fields still missing: the operation's actor, wall clock and signature, and the changeset's author, intent references, renames and conflict terms. T-011 **SHALL** remain.
