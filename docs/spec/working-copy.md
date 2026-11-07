# The working copy

This is how Periapsis records a working copy, as built in Sprint 2, and writes one, as built in Sprint 3. It
realises [INT-0014](../intents/INT-0014-working-copy-and-workspaces.md) AC-1
and AC-2 and [INT-0004](../intents/INT-0004-windows-and-linux.md) AC-2 and
AC-7, on the provisional `changeset` and `view` kinds of
[format v1](format-v1.md). The `workspace` crate implements it, through
`sim::Fs`. Sprint 3 adds writing the working copy for `undo` and
`restore` ([Writing the working copy](#writing-the-working-copy)) and INT-0004
AC-3 and AC-4. Checkout of another change, conflicts, several workspaces and
the stat cache come later.

## Layout

A repository is a directory, the working copy. Its store is
`<root>/.praps/store`, with the layout and write sequence of
[Storage durability](storage-durability.md).

`praps init [DIR]` does the following, in order:
1. If `DIR` is missing, creates it. Either way it syncs `DIR`'s parent, so
   a root left by an interrupted `init` becomes durable.
2. If `.praps` is missing, creates it, then syncs `.praps` and `DIR`, so the
   `.praps` entry is durable.
3. Initialises the store.
4. If the store holds no op, commits the root op.

The root op is three objects:
- an empty changeset with no parents and a fresh change ID;
- a view whose single head, and whose `default` workspace, is that changeset;
- an op naming that view, with no parents.

On a repository that already has an op, `init` writes nothing. Running it
again therefore completes an interrupted `init`. Opening a repository makes
`.praps` and the root durable in the same way.

## The working-copy change

The view names the working-copy change of workspace `default`. A
changeset's `changes` are relative to its first parent's tree; a root's are
relative to the empty tree. A tree is derived by applying the changes along
the chain of first parents, and it is never stored
([INT-0013](../intents/INT-0013-derived-data.md)). A changeset with several
parents is refused with an error until merges exist
([INT-0012](../intents/INT-0012-conflicts-and-merge.md)).

## Snapshot

Every command that opens the working copy snapshots it first: `praps new` and
`praps status`. `praps fsck` only reads, and `praps init` on an existing
repository writes nothing, so neither snapshots. An edit made before one of
them is captured by the next command that does.

A snapshot does the following:
1. **Loads the state.** If `ops/` holds an op file that does not hash to
   its name or does not decode, it stops and points to `praps fsck`. It finds
   the single op head. With no op it reports
   that the repository is not initialised. With several op heads it stops,
   names every head, and writes nothing. Concurrent ops are not merged
   until [INT-0010](../intents/INT-0010-lock-free-concurrency.md).
2. **Scans** the working copy, from the top down, following the ignore rules
   below. Each regular file's bytes are read exactly as found, with no
   normalisation ([INT-0034](../intents/INT-0034-any-content.md)). A file
   deleted between the listing and its read counts as deleted. On Windows
   every path is passed verbatim (`\\?\`), so a name with a trailing dot or
   space, or a device name such as `aux.txt`, is read as itself.
3. **Compares** the scanned tree with the working-copy change's tree. If
   they are equal, it writes nothing.
4. **Records** the new tree, writing in this order:
   - each new blob;
   - a changeset with the same change ID, parents and description, whose
     changes are the new tree against the parent's tree (a `put` for each
     added or changed path, a `delete` for each removed one);
   - a view in which that changeset replaces the old one, both as a head and
     as the `default` workspace's change;
   - an op whose single parent is the head.

   The store refuses an object whose references are missing, and makes the
   op the acknowledgement.

`praps new` snapshots, then starts a new working-copy change on top of the
current one: a changeset whose only parent is the current change, with a
fresh change ID and no changes. It replaces its parent as a head. No
working-copy file is written.

`praps status` snapshots, then lists each path of the working-copy change
against its parent: `A` added, `M` modified (content or mode), `D` deleted.
It also prints a `skipped` line for each reported entry.

Change IDs are 128 bits from a generator seeded by the caller: by the
operating system's randomness in `praps` (`sim::os_seed`), and by a fixed
seed in tests.

## Ignore rules

The rules come from the `.gitignore` files inside the working copy, with Git's
pattern semantics (anchoring, `*`, `**`, `?`, character classes, `!`
negation, a trailing `/` for directories only, `#` comments, `\` escapes):
- **Scope.** Each `.gitignore` applies to its own directory and below.
- **Precedence.** The deepest `.gitignore` with a matching pattern decides.
  Within one file, the last matching pattern wins.
- **Case.** Matching is case-sensitive on every system, so a repository
  ignores the same paths on Windows and Linux.
- **Ignored directories.** The scan does not enter an ignored directory, so
  nothing under it can be re-included, as in Git. The one exception is that
  it enters to visit paths that are already tracked.
- **Tracked files.** A file that is already tracked stays tracked, with its
  edits snapshotted, even if a rule now ignores it or its directory. Once it
  is deleted, the rule applies to it again.
- **Always ignored.** `.praps` and `.git` at the top of the working copy are
  never snapshotted, whatever their letter case (a case-insensitive
  filesystem treats `.Git` as `.git`). Below the top they are ordinary
  names.
- **Byte-order mark.** A UTF-8 byte-order mark at the start of a
  `.gitignore` is not part of its first pattern, as in Git.
- **Files not read.** Git's global excludes file and `.git/info/exclude` are
  not read, so a snapshot does not depend on the machine it runs on.
- **Line endings.** A carriage return at the end of a `.gitignore` line is
  not part of the pattern.
- **Implementation.** Each `.gitignore` is read through `Fs` and fed line by
  line to `ignore::gitignore::GitignoreBuilder::add_line`. The matcher is
  rooted at `.` and given paths relative to the file's own directory. The
  crate's filesystem readers are banned in `crates/workspace/clippy.toml`
  (`cargo policy`, rule `fs-boundary`).

`crates/workspace/tests/fixtures/ignore-corpus.txt` pins these rules
against Git's own answers.

## Reported entries

A snapshot never tracks, changes or deletes the following entries. It
reports each one, and its siblings are still snapshotted:
- a name with no UTF-8 form (bytes that are not UTF-8 on ext4, an unpaired
  surrogate on NTFS; INT-0004 AC-7);
- a UTF-8 name that is not a valid repository path component (one holding a
  `\` or a NUL);
- a symbolic link (symlinks are not specified yet; mode `02` is reserved);
- a FIFO, socket or device.

Every component of a reported path is shown in one escaped form:
- every `\` is shown as `\\`, and a control character as `\u{NN}`;
- in a name with no UTF-8 form, a byte that is not valid UTF-8 is shown as
  `\xNN`, and an unpaired surrogate as `\u{DNNN}`.

So a name with no UTF-8 form never displays like a UTF-8 name.

## The executable bit

Where files carry an owner-execute bit (Linux), the snapshot records
`executable` or `regular` from it. Where they do not (Windows), a tracked
file keeps the mode that the working-copy change, or else its parent, last
recorded, and a new file is recorded as `regular`. A checkout made on Linux
therefore keeps its executable bits when edited on Windows.

## Writing the working copy

Sprint 3 adds the commands that write files: `undo` and `restore`
([INT-0009](../intents/INT-0009-reversible-by-default.md)). Each moves the
working copy from a tree F, the working-copy change after the command's
opening snapshot, to a tree T, the working-copy change of a target view.
This is called *materialising* F → T.

### History commands

- **`log`** snapshots, then lists every op, newest first: its id, parents and
  working-copy change. "Newest first" means a topological order, children
  before parents, ties broken by id.
- **`undo`** takes H, the single op head when the command starts, before its
  snapshot. It then snapshots. If H has no parent, it fails with "nothing to
  undo", having written only that snapshot. Otherwise the target view is the
  view object of H's parent. A second `undo` restores the view before the
  first, so `undo` toggles. The edits a snapshot captured stay reachable by
  `restore`.
- **`restore OP`** snapshots, then takes OP's view as the target. OP is an op
  id in text form or a unique prefix of its hex digest (with or without the
  `blake3:` part). A prefix that matches no op, or several, is an error
  listing the candidates, written after the snapshot only.

Both `undo` and `restore` then:
1. check hazards and blocking entries (below), failing with nothing more
   written;
2. write the record;
3. commit an op whose single parent is the head after the snapshot and whose
   view is the target view, the exact view object, unchanged;
4. materialise F → T.

No command removes an op (INT-0009 AC-1).

### The record

`.praps/working-copy` holds two lines, `from <op>` and `target <op>`, each
op id in its text form. `from` is the head after the snapshot, whose
working-copy tree is F. `target` is the op step 3 commits, whose view's
working-copy tree is T. The target op's id is computed from its envelope
before it is committed.

The record is written as `.praps/working-copy.new` (replacing any left
over), synced, renamed to `.praps/working-copy`, and then `.praps` is synced,
all before the target op is committed. A record exists only while a write is
in flight. One that does not decode is a `Corrupt` error.

### Classifying paths

"Matches F" means the file's bytes equal F's version and, where the
filesystem has an executable bit, so does its mode. Each path in F or T, or
present on disk, falls into one class:
- **to remove:** tracked by F, not by T, and still matching F;
- **to write:** in T with an entry different from F's, and either matching F
  or absent where F has none;
- **done:** already equal to T's entry, or absent where T has none, as after
  a partial run. Nothing is done and nothing is noted;
- **kept:** everything else. It is a user edit and is left as it is, for the
  next snapshot.

The F→T diff is the sorted list of paths whose entry differs between F and
T. The *n*th path of the diff (from 0) has the temporary name
`.praps-tmp-<n>` in its own directory. The name is a pure function of F and
T, so a resumed run uses the same names. No name is ignored by snapshots: a
user's file called `.praps-tmp-1` is tracked like any other.

An *F-owned* entry is any of these:
- a path to remove;
- a directory holding only paths to remove;
- an empty directory where F had files and T has none.

A *blocking* entry is any of the following that is not F-owned:
- an entry at a path to write;
- an entry where a path to write needs a directory;
- a directory where a path to write needs a file;
- an entry, or any F or T path, equal to a temporary name this run needs.
  On a case-insensitive filesystem, "equal" ignores case.

Blocking entries are refused like hazards, before anything is written.

### Hazards

The naming rules come from `Fs::name_rules`. They are checked for every
component of each path to write, and of each directory to create. Paths that
stay unchanged are not checked, so a tracked file with a hazardous name never
blocks `undo`.

- **Case-insensitive filesystems.** Two names are equal if they match under
  `to_lowercase` or under `to_uppercase`. This test is deliberately
  conservative. A hazard is either of:
  - two such components that are equal yet spelled differently, among T's
    paths;
  - such a component that equals an existing entry spelled differently in
    the same directory.

  The second case includes a case-only rename, which is refused as a known
  limit (backlog T-016).
- **Windows names.** A hazard is a component with any of:
  - a character from `<>:"|?*`, or a code below 32;
  - a trailing `.` or space;
  - a reserved base name. The base is the part before the first `.`, with
    trailing spaces removed, compared ignoring case. The reserved bases are
    `CON`, `PRN`, `AUX`, `NUL`, `CONIN$`, `CONOUT$`, `COM0` to `COM9`,
    `LPT0` to `LPT9`, and `COM` or `LPT` followed by `¹`, `²` or `³`.

A command that finds a hazard fails with an error naming every offending
path. Its only write is its opening snapshot.

### Materialise

With nothing blocking, materialise does the following, in order:
1. remove the paths to remove, then sync each parent directory touched;
2. remove the emptied directories (empty, holding no T path, where F had
   files), deepest first: sync the directory, remove it, sync its parent;
3. create missing directories, shallowest first, syncing each parent;
4. write each path to write through its temporary name: create it, write
   T's bytes, set the executable bit, `sync_file`, rename over the path;
5. sync every touched directory;
6. remove the record;
7. sync `.praps`.

Setting the bit before the rename means T's bytes never appear with F's
mode. Where files have no executable bit, setting it does nothing, and the
mode is kept as described in [The executable bit](#the-executable-bit).

### Resume

Every command that opens the working copy (every one but `init`) first looks
for a record:
- **The target op is the current single head.** Resume removes the diff's
  temporary names that exist, then materialises F → T again with the same
  rules. A path that still blocks, or has since become a hazard (a case
  variant created after the crash, say), does not fail the command. Resume
  finishes every other path, drops the record, and adds a note naming the
  paths it left as they are.
- **The target op is absent.** The files were never touched. Resume drops
  the record and touches no file.
- **The target op is present but not the head.** Resume removes the diff's
  temporary names that exist, drops the record, touches no other file, and
  adds a note.

Dropping the record means removing it and syncing `.praps`. While a record
exists, its diff's temporary names belong to `praps`. Notes are returned by
`Workspace::notes()`, and `praps` prints each on stderr as `note: …`. `init`
never resumes, so a repeated `init` still writes nothing.

## Costs deferred to the stat cache

Until the stat cache (INT-0014 AC-5), every snapshot reads and hashes every
tracked file. The store also re-reads each blob that a changed changeset
names, to check it. A file edited during a scan is captured as it was either
before or after the edit, and either is a correct snapshot. The race on
mtime granularity arrives with the cache, which must handle it.
