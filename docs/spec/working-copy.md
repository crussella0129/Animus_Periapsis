# The working copy

This is how Periapsis records a working copy, as built in Sprint 2. It
realises [INT-0014](../intents/INT-0014-working-copy-and-workspaces.md) AC-1
and AC-2 and [INT-0004](../intents/INT-0004-windows-and-linux.md) AC-2 and
AC-7, on the provisional `changeset` and `view` kinds of
[format v1](format-v1.md). The `workspace` crate implements it, through
`sim::Fs`. Writing to the working copy (checkout, conflicts), several
workspaces and the stat cache come later.

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

## Costs deferred to the stat cache

Until the stat cache (INT-0014 AC-5), every snapshot reads and hashes every
tracked file. The store also re-reads each blob that a changed changeset
names, to check it. A file edited during a scan is captured as it was either
before or after the edit, and either is a correct snapshot. The race on
mtime granularity arrives with the cache, which must handle it.
