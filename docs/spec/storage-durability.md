# Storage durability

How the Periapsis store makes an acknowledged operation survive a crash on
Linux and on Windows, what the evidence for that is, and what remains an
assumption. It realises [INT-0007](../intents/INT-0007-crash-safe-storage.md)
and answers [INT-0004](../intents/INT-0004-windows-and-linux.md) AC-5, which a
reviewer accepts or rejects.

**Status:** Sprint 1. The operation format it stores is provisional
([format v1](format-v1.md#change-control)).

## Layout

A store is a directory holding `objects/` and `ops/`. Each object and each
operation is one file whose content is the object's envelope and whose name
is its id: `<algorithm id as two hex digits>-<digest hex>`, for example
`01-af1349b9…`. The `HashRef` text form `blake3:af13…` is not used for names:
on NTFS a colon names an alternate data stream. There is no `HEAD` or other
pointer file: the op heads are the valid ops that no other valid op names as
a parent, recomputed from `ops/` whenever they are needed.

Files named `tmp-<16 hex digits>` are files being written. A crash can leave
one behind; nothing reads them, and `praps verify` lists them as harmless.

## The write sequence

Every object and op file is written the same way:

1. `create_new(dir/tmp-<nonce>)`, a fresh name (the nonce comes from the
   store's seeded generator; an existing name is skipped).
2. `write` the whole envelope.
3. `sync_file`: the data is durable.
4. `rename` to `dir/<id>`.
5. `sync_dir(dir)`: the name is durable.

If `dir/<id>` already holds exactly these bytes, steps 1 to 4 are skipped,
but the file and its directory are still synced, because another writer may
have renamed it without syncing yet. If it holds other bytes (a copy a crash
damaged), it is replaced through steps 1 to 5.

`put_object` first checks that the object decodes as its kind and that every
object it names (a changeset's parents and blobs, a view's heads and
workspace changesets) is present and hashes to its id; otherwise it writes
nothing. `commit_op` checks that every parent op and its view are present,
hash to their ids and (for the view) decode as a view, then syncs the view
with `objects/` and each parent op with `ops/`, and only then writes the op.
It returns after step 5 of the op file; that return is the acknowledgement.

Closure (INT-0007 AC-1) follows the references inside objects: op → view →
changesets → blobs. It rests on two facts:
- **Data before names.** Every writer syncs a file's data before renaming it
  into place, so any object present under its final name has durable data.
- **One shared directory.** Every object lives in `objects/`, and every
  object write ends with `sync_dir(objects)`, which makes every name then in
  the directory durable at once, including a name another writer renamed
  into place but has not synced yet.

So when an object's put is acknowledged, everything it references, which
the put checked was present, is durable too. `put_object` needs no separate
re-sync, even for a reference another writer has not acknowledged. Closure
then holds by induction over acknowledged puts. An object that a crash
leaves behind unacknowledged, with a reference lost, is reachable from no op,
and `praps verify` lists it as a leftover, not as corruption.

`commit_op` writes in another directory, `ops/`, so it re-syncs the view
with `objects/` before writing the op. It also re-syncs each parent op with
`ops/`: a crash between the new op's rename and its own directory sync could
otherwise keep the in-flight op and lose a parent another writer had not
synced. The crash matrix probes both re-syncs with a race of each kind and a
negative control. The race in which a second writer's `put_object` names a
first writer's unsynced object is covered by the shared-directory sync
above, not by a scenario of its own.

`init` creates whichever of the root, `objects/` and
`ops/` is missing and then syncs `objects/`, `ops/`, the root and the
directory holding the root's entry (its parent, or `root/..` for a bare
relative name), so an interrupted `init` is completed by running it again.
`open` makes the same syncs: a process that died inside `init` can leave a
layout that exists but whose entries were never synced, and ops acknowledged
into it would otherwise be lost with it.

## What the simulator models

`sim::SimFs` follows the fault model of Pillai et al., "All File Systems Are
Not Created Equal" (OSDI 2014):

- Data written since a file's last `sync_file` may, after a crash, be kept,
  dropped, or torn to a prefix of what was written. It follows the file
  across a rename: neither `rename` nor `sync_dir` makes data durable.
- A new name, a rename or a removal is pending until `sync_dir` of its
  directory; after a crash each pending change is applied or not, each
  atomically, and each acts only on the file it was made for. A directory
  whose own entry is lost takes everything under it.
- **Strict mode** assumes only that. **Lenient mode** adds what common
  journaling filesystems do: a file's `sync_file` also makes its own name
  durable, together with every earlier entry change in its directory (the
  journal commits in order), and `create_dir` is durable at once.

The crash matrix (`crates/store/tests/crash.rs`) runs four scenarios — `init`
alone, a chain of five ops, a fork and merge, and two writers interleaved at
the syscall level — with a crash at every syscall, 40 seeds and both modes,
and checks every recovered state for referential closure, durability of
acknowledged ops and objects, derived heads, atomicity, torn files and stray
files. Four known-bad stores, as negative controls, must fail it: a pointer
file written in place, objects not synced, `init` without directory syncs, and
files written in place under their final names. The counts of each
run are in the sprint's test evidence.

## Linux

`rename` is `rename(2)`, atomic within a directory. `sync_dir` opens the
directory read-only and calls `fsync` on it (`File::sync_all`), the POSIX way
to make its entries durable. The store relies on nothing beyond strict mode.

## Windows

`rename` is `std::fs::rename`. Rust's standard library uses
`MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`, and where Windows supports it
a rename with POSIX semantics (`FileRenameInfoEx`), so replacing a name is
atomic ([rust-lang/rust#137528](https://github.com/rust-lang/rust/pull/137528)).

Windows has no POSIX directory `fsync`. `sync_dir` opens the directory with
`FILE_FLAG_BACKUP_SEMANTICS` (needed to open a directory at all) **and write
access**, and calls `FlushFileBuffers` on it (`File::sync_all`). It does this
through safe `std` only (`OpenOptionsExt::custom_flags`); the workspace
forbids `unsafe`.

Evidence:

- `crates/sim/tests/real_durability.rs`, `directory_flush_needs_write_access`
  (Windows): the handle `RealFs` opens flushes with success; a read-only
  backup-semantics handle is refused with "Access is denied"; a plain open of
  a directory fails. First probed on the development host (Windows 11, NTFS)
  during Sprint 1 research.
- `write_sequence_succeeds_on_the_real_filesystem` and
  `crates/store/tests/real_store.rs` (`real_store_reopens_clean`) run the full
  sequence on NTFS and on ext4, reopen the store, and find it clean.
- Other storage engines take the same approach and report it works on NTFS
  ([apache/maka#4520](https://github.com/apache/maka/pull/4520)); misusing a
  read-only handle is a known pitfall
  ([dgraph-io/badger#699](https://github.com/dgraph-io/badger/issues/699)).

## What remains asserted

- Microsoft does not document `FlushFileBuffers` for directory handles, and a
  volume flush needs administrator rights
  ([Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/6057938/supported-non-administrative-ntfs-namespace-durabi)).
  That the directory flush makes a new name durable on NTFS rests on NTFS
  journaling its metadata and on the empirical evidence above, not on a
  documented guarantee.
- No test cuts power. The crash evidence is the simulator's, under a fault
  model at least as harsh as the filesystems it models; the real-filesystem
  tests show only that every step runs and the result reads back.
- Only NTFS on a local disk (Windows) and ext4 (WSL Ubuntu) are tested. ReFS,
  FAT, network shares (where a directory flush fails with an error) and
  macOS are not.
- Concurrent writers (INT-0007 AC-5) are tested in two ways since Sprint 4,
  and neither is a proof. On the simulator, four processes write under a
  seeded scheduler, with kills and crashes ([Simulation](simulation.md));
  the interleavings tried are those the seeds draw. On the real filesystem,
  four operating-system processes commit to one store
  (`crates/store/tests/real_writers.rs`); there the kernel decides the
  interleavings, so that run is evidence and cannot be replayed.
- A process that dies while the system keeps running leaves its unsynced
  changes visible to the next process. The simulator runs that case too
  since Sprint 4, as a kill, in the four-writer scenario. No test kills a
  real process in the middle of a write.

A reviewer accepting INT-0004 AC-5 accepts these assumptions with the
evidence above.
