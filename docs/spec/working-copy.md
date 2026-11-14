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

## Finding the working copy

A directory is the **top** of a working copy when listing its `.praps/store`
succeeds. A command that opens a working copy (`status`, `describe`, `new`,
`log`, `ops`, `undo`, `restore`) acts on the nearest top among the directory
it is given, or the current directory when it is given none, and the
directories above that one. So a command does the same thing wherever inside
the working copy it is run.

- **Above the starting directory.** `praps` then makes the top its current
  directory and runs as if started there, so what it writes and syncs is what
  a run from the top writes and syncs.
- **A directory that is not there**, or is not a directory, is refused by the
  name given. Nothing above it is acted on.
- **No top there or above** is a refusal that says so.
- **A `.praps/store` that cannot be listed** for a reason other than absence
  stops the search. A repository further up is never used in its place.
- **Names.** A refusal names the working copy by the directory given, when
  one was given and it is the top, and by the top's full path otherwise.
- **Links.** The directories above are taken from the starting directory's
  full path as written. A symbolic link in it is not resolved first.

`praps init` never makes a repository inside another: with a top above the
directory it is given, it writes nothing and names that working copy,
whatever the directory holds. An entry named `.praps` below the top that is a
file, or a directory with no `store` in it, is tracked like any other entry.

`praps verify DIR` checks the first of: the store of DIR, if DIR is a top;
DIR itself, if it holds `objects` and `ops`; the store of the nearest top
above DIR; DIR itself, which then fails as not a store. If whether DIR holds
them cannot be read, `verify` stops there and checks no other store.

Every refusal ends with one line that starts `hint:` and says what to do
next ([INT-0037](../intents/INT-0037-command-set.md) AC-4).

## The working-copy change

Each workspace's entry in the view names its working-copy change, and a
working copy reads and writes only its own workspace's entry, `default`
for the repository `init` makes. A changeset's `changes` are relative to its first parent's tree; a root's are
relative to the empty tree. A tree is derived by applying the changes along
the chain of first parents, and it is never stored
([INT-0013](../intents/INT-0013-derived-data.md)). A changeset with several
parents is refused with an error until merges exist
([INT-0012](../intents/INT-0012-conflicts-and-merge.md)).

## Reading the op log

A command reads the op log once, at its start: one listing of `ops/` and one
read of each op file. From that it knows every valid op, the op heads, the
view they leave together ([Merging op heads](#merging-op-heads)) and the
working-copy change, and it keeps them for as long as it runs. A
step that commits an op adds that op to what the command holds, so a later
step of the same command reads nothing again.

Each command starts from the log as it is on disk, with one exception. The
first command on a workspace that was just opened uses the reading that
opening made, so a `praps` process, which opens the repository and runs one
command, reads the log once. An op another process writes between the opening
and that command is treated like any op written while a command runs: both
stand, the log has two heads, and the next command merges them.

`Workspace::working_copy` and `Workspace::parent_tree` read the store every
time they are called. They exist for tests that compare what is stored with a
model, and never return what a command kept.

Two costs that grow with the history remain. Opening a store lists `objects/`
and `ops/` once each, to see that they exist. And every command reads the
whole chain of changes to build a tree. Both wait for the storage form
([INT-0008](../intents/INT-0008-compact-history-storage.md)).

## Merging op heads

Commands that run at the same time each commit an op over the heads they
read, so the log can have several heads. A command that finds several works
from the view they leave together, their canonical merge
([INT-0010](../intents/INT-0010-lock-free-concurrency.md)). The merge is
computed in memory and never committed: it is a function of the set of heads,
so every process that holds the same ops computes the same view, whatever
order it found them in, and a command that changes nothing writes nothing.

An entry is written only by ops made in its workspace
([INT-0016](../intents/INT-0016-actors-and-agent-lifecycle.md)): an op names
the workspace that made it, and its copies of the other entries are only what
it saw. The merge takes each entry from its own workspace's ops:
- Its **writers** are the ops made in that workspace, among the heads and
  every op they follow, that no other op made there follows.
- With no writer the entry is absent. With one, it is that op's entry.
- With several, their entries are merged against a **base**: the entry at
  their latest common op made in that workspace; or, when there are several
  such ops (a criss-cross), their entries merged the same way; or no entry
  when they have none in common.
- Against the base, the working-copy pointer keeps the base's value when no
  writer changed it, and otherwise holds every value the writers that
  changed it hold: one changeset, or several. A changeset is among the heads
  when it is in the base and every writer kept it, or is not in the base and
  some writer has it.

With one head the merge is that head's view. A command that changes
something commits one op, made in its own workspace, whose parents are every
head it merged and whose view is the merge with only its own entry changed;
that entry's pointer holds one changeset. INT-0010 records the merge's known
limit: a value set back to an earlier one reads as no change against an older
base.

**The race rule.** When two commands of one workspace move its pointer at
the same time, the merged pointer holds both changes. The next command there
continues from the one with the smallest id, keeps the others among the
workspace's heads, and prints one note naming every one of them. Once it
commits, the pointer holds one change again
([INT-0014](../intents/INT-0014-working-copy-and-workspaces.md)).

## Snapshot

Every command that opens the working copy snapshots it first: `status`,
`describe`, `new`, `log`, `ops`, `undo` and `restore`. `praps verify` only
reads, `praps init` on an existing repository writes nothing, and `help` and
`version` look at no directory, so none of those snapshots. An edit made before one of
them is captured by the next command that does.

A snapshot does the following:
1. **Loads the state**, or takes the one its command already read
   ([Reading the op log](#reading-the-op-log)). If `ops/` holds an op file
   that does not hash to its name or does not decode, it stops and says how
   many are not whole; `praps` then ends its refusal with a hint that names
   `praps verify`. With no op it reports that the repository is not
   initialised. With several op heads it works from their merge
   ([Merging op heads](#merging-op-heads)).
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
   - a view, the merge with only this workspace's entry changed, in which
     that changeset replaces the old one, both as a head and as the
     working copy;
   - an op made in this workspace whose parents are every op head.

   The store refuses an object whose references are missing, and makes the
   op the acknowledgement.

`praps new` snapshots, then starts a new working-copy change on top of the
current one: a changeset whose only parent is the current change, with a
fresh change ID and no changes. It replaces its parent as a head. No
working-copy file is written.

`praps describe TEXT` snapshots, then sets the description of the
working-copy change. It writes a changeset with the same change ID, parents
and changes and with TEXT's bytes as its description, a view in which that
changeset replaces the current one, both as a head and as the `default`
workspace's change, and one op whose single parent is the head. No
working-copy file is written. It prints one line that names the change and
shows the first line of the text, escaped as names are. If TEXT is already
the change's description, it writes nothing for it and says that nothing
changed. An empty TEXT removes a description.

What is stored is what was given:
- on Linux, the argument's bytes, whatever they are;
- on Windows, the argument's UTF-8 form. Text that has none, because it
  holds an unpaired surrogate, is refused as a usage error with nothing
  written.

TEXT is one argument, so more than one word needs quotes. It cannot begin
with `-` ([INT-0037](../intents/INT-0037-command-set.md)).

A later snapshot keeps the description (step 4 above), and `praps new`
starts a change with none. `describe` commits an op like any other, so
`undo` takes the description back and `restore` returns to it, and neither
writes a working-copy file for it.

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

- **`ops`** snapshots, then lists every op, newest first: its id, the change
  ID of its working-copy change, and the number of paths that change
  changes. "Newest first" means a topological order, children before
  parents, ties broken by id. It reads two objects for each op, the view and
  the working-copy changeset, however long the history is. (Until Sprint 6
  this command was `log`, and it counted the files of each op's whole tree,
  which read the chain of changes again for every line.)
- **`log`** snapshots, then lists the working-copy change and the changes
  before it along first parents, newest first: the change ID, the number of
  paths the change changes, and the first line of its description, or
  `(no description)`. The line is escaped as names are, so it stays one line
  whatever bytes the description holds. It reads one object for each change.
- **`undo`** takes this workspace's writers when the command starts, before
  its snapshot ([Merging op heads](#merging-op-heads)); with one head, that
  head. It then snapshots. If they have no parent, it fails with "nothing to
  undo", having written only that snapshot. Otherwise the target view is the
  view their parents leave together: with one parent, that op's view object;
  with several, their merge, written as a new view. A second `undo` restores
  the view before the first, so `undo` toggles. The edits a snapshot
  captured stay reachable by `restore`.
- **`restore OP`** snapshots, then takes OP's view as the target. OP is an op
  id in text form or a unique prefix of its hex digest (with or without the
  `blake3:` part). A prefix that matches no op, or several, is an error
  listing the candidates, written after the snapshot only.

Both `undo` and `restore` then:
1. check hazards and blocking entries (below), failing with nothing more
   written;
2. write the record;
3. commit an op made in this workspace, whose parents are every op head
   after the snapshot and whose view is the target view, the exact view
   object, unchanged;
4. materialise F → T.

Each of them scans the working copy once: the snapshot named above is the
only one. An edit made before the command is in that snapshot, in an op of
its own, and step 4 reads each path again just before it removes or replaces
it and leaves one that has changed since
([Materialise](#materialise)).

No command removes an op (INT-0009 AC-1).

### The record

`.praps/working-copy` holds two lines, `from <changeset>` or `from empty`,
and `target <op>`, each id in its text form. `from` is the working-copy
changeset after the snapshot, whose tree is F; a merged state has no single
op, so the record names the change. `empty` names the empty tree, which a
lane's first write starts from. `target` is the op step 3 commits, whose
view's working-copy tree is T. The target op's id is computed from its
envelope before it is committed. A `from` that names no changeset in the
store is a `Corrupt` error.

The record is written as `.praps/working-copy.new` (replacing any left
over), synced, renamed to `.praps/working-copy`, and then `.praps` is synced,
all before the target op is committed. A record exists only while a write is
in flight. One that does not decode is a `Corrupt` error.

### Classifying paths

"Matches F" means the file's bytes equal F's version and, where the
filesystem has an executable bit, so does its mode. Each path in F or T, or
present on disk, falls into one class:
- **to remove:** tracked by F, not by T, still matching F, and reached
  through real directories only (a directory replaced by a symbolic link is
  never followed, and neither is an emptied directory removed through one);
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
- an entry, or any F or T path, equal to the temporary name of any path of
  the diff, written or not, since resume removes them all. On a
  case-insensitive filesystem, "equal" ignores case.

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

A directory that step 2 cannot remove is left, empty, and a note names it.
On Windows a directory cannot be removed while a process has it as its
current directory, which is common now that a command may be run from any
directory inside the working copy. Directories are not tracked, so an empty
one changes nothing a snapshot records, and the write goes on to steps 3 to
7.

There is one exception. If the write must put a file at the directory's
path, or at a path above it, that file could not be put while the directory
stays. The write then stops with a failure that names the directory, the
record is kept, and each later command resumes the write and stops the same
way until the directory can be removed. Going on there would drop the record with a file of T
missing, and the next snapshot would record it as deleted.

A removal that fails because the process crashed is not a refusal: it stops
the write like a crash at any other step. A directory that is already gone
when step 2 comes to remove it is passed over.

Just before a path is removed (step 1) or renamed over (step 4), it is read
again. If it no longer holds F's bytes (or, for a path F did not have, is no
longer absent), it was edited during the write: it is kept, the temporary
file is removed, and a note names it. This shrinks the window in which an
edit saved during the write could be lost to the gap between that read and
the syscall that follows; the mode is not read again.

Setting the bit before the rename means T's bytes never appear with F's
mode. Where files have no executable bit, setting it does nothing, and the
mode is kept as described in [The executable bit](#the-executable-bit).

### Resume

Every command that opens the working copy (every one but `init`) first
checks that every file in `ops/` is a whole op, and stops if one is not,
leaving the record and every file as they are. Resume looks for the record's
target among the valid ops, so without that check a target whose file was
damaged would look like a write that never began: the record would be
dropped, though files might already have been written, and the half-written
working copy would be snapshotted as edits once the op file was repaired.

It then looks for a record:
- **The target op is its workspace's only writer**: no other op made in
  that workspace follows it or stands beside it, whatever ops of other
  workspaces do. Resume removes the files
  at the diff's temporary names (never an F or T path, and never anything
  but a file), then materialises F → T again with the same rules. A
  temporary name still taken leaves its path as it is, with a note. A path that still blocks, or has since become a hazard (a case
  variant created after the crash, say), does not fail the command. Resume
  finishes every other path, drops the record, and adds a note naming the
  paths it left as they are.
- **The target op is absent.** The files were never touched. Resume drops
  the record and touches no file.
- **The target op is present, but another op of its workspace followed it
  or stands beside it.** Resume removes the diff's temporary names that
  exist, drops the record, touches no other file, and adds a note.

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
