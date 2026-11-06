# INT-0034 — Any content: the core never needs to understand what it versions

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0034
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Periapsis versions any content: source code in any language, natural-language
text in any script, assembly, compiled binaries, large data files, and
languages no human reads, such as one that agents use only among themselves.
Its core treats every file's content as bytes and never needs to parse, decode
or understand it. Everything that reads content is an optional layer, chosen
per kind of content.

- **The core.** Storage, object identity, history, operations and undo,
  working copies, sync and distribution, intents and their links, review
  requests, confirmations, signatures and queries work on any bytes of any
  size. None of them assumes that content is source code, text,
  line-structured, UTF-8, in a known language, readable by people, small, or
  checkable by a test runner. An intent's statement and the other content
  fields of a record are bytes too
  ([INT-0017](INT-0017-event-records-and-book-view.md)).
- **Exact bytes.** Every file is stored, synced and checked out exactly as it
  was found. Periapsis changes no line ending, encoding, Unicode form,
  byte-order mark, trailing space or final newline unless a project opts in.
  An opted-in rule names the paths it covers, is kept in repository policy and
  acts the same way on every replica and platform.
- **Layers.** Line diff and diff3, structured (AST) merge, syntax-aware diff,
  blame, comment tags, module detection, test runners, the generators of
  declared generated files, proof checking and rendering are layers. Each one
  reads some kinds of content. Periapsis ships some layers, such as line diff
  and diff3 for UTF-8 text, and turns them on by default, but no core
  operation needs any layer.
- **Kinds of content.** Repository policy declares kinds of content by path
  pattern and chooses the layers for each kind. A file that no declaration
  covers goes to the layers that are on by default, if one of them can read
  it, and otherwise to the core alone. Declaring a kind never changes a file's
  bytes or its object id.
- **What a layer may change.** A layer changes presentation and convenience
  only. Turning a layer on or off never changes an object already stored, the
  object id that given bytes get, or whether a core guarantee holds. A merge
  layer may resolve a conflict that the core alone would keep as a conflict
  term ([INT-0012](INT-0012-conflicts-and-merge.md)). Its result is stored and
  synced like any other version, with the name of the layer that made it, so a
  replica without that layer reads the same result.
- **Content no layer reads.** Every command works on it. A command that shows
  content for people to read shows such a file by its size and hash, and a
  change to it by the old and new size and hash; it never fails and never
  prints the raw bytes. A command that outputs a file for a program or a pipe
  outputs its exact bytes. A layer that fails on a file leaves that file to
  the core, as if no layer read it, and the command reports the failure.
- **Merge drivers.** A project can register a merge driver for a kind of
  content, including a format Periapsis has never seen. A driver is a
  program: it gets the base and both sides as bytes and returns a merged
  version or a conflict. A driver runs code, so a replica runs it only where
  it has been allowed on that machine; synced policy can name a driver but
  cannot make a replica run it. Where a driver is not allowed, fails, crashes
  or runs past its time limit, the file becomes a conflict term, so merging
  never blocks.
- **Conformance suite.** One suite runs every core operation over a fixed
  corpus: Rust and Python source code; prose in English, Arabic, Hebrew,
  Chinese, Japanese and Korean, with file names in those scripts too; a
  project written only in natural language; assembly source, and executables
  and object files for x86-64 and AArch64; large binary files, one of them
  larger than 4 GiB; files of random bytes; and files in a synthetic language
  made from a seeded random grammar, standing in for languages that only
  machines use. The corpus also holds the cases that normalisation would
  change: CRLF and lone CR line endings, no final newline, byte-order marks,
  NUL bytes, invalid UTF-8, empty files, and the same text in composed and
  decomposed Unicode forms.

Boundaries: the charter ([INT-0001](INT-0001-periapsis-charter.md)) states
the principle; this intent owns the line between the core and the layers, and
the suite that checks it. What a layer does belongs to the intent that defines
it, such as merge in [INT-0012](INT-0012-conflicts-and-merge.md) and the
checks behind confirmations in
[INT-0015](INT-0015-phases-and-confirmation.md). Paths are names, not content,
and keep the UTF-8 form of [INT-0004](INT-0004-windows-and-linux.md).
Periapsis's own code stays 100% Rust
([INT-0003](INT-0003-engineering-baseline.md)) and this Book stays Markdown:
both are about how Periapsis is built, not what it versions. Lean proofs stay
one optional kind of evidence for `formal` criteria
([INT-0018](INT-0018-proof-pairing-certificates.md)).

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Conformance: with no layer on, every core operation in the conformance suite succeeds on every kind of content in the corpus, and each scenario's outcomes are the same for every kind.
  - Sample: scenarios covering snapshot, describe, amend, rebase, split, squash, absorb, merge with and without conflicts, conflict resolution, undo, op restore, sync between two replicas, bundle export and import, intent and region links, review requests, confirmations, signatures and queries, each run once per kind on Windows and on Linux.
  - Threshold: 100% of operations succeed, with 0 differences between kinds in merge results (clean or conflict), in the changes, links and review requests made, or in any change's phase.
- **AC-2** `necessary` `empirical` — Exact bytes: with no normalisation rule opted in, every file in the corpus reads back byte for byte after a snapshot, storage, sync to a replica on the other platform, bundle export and import, and checkout; with a rule opted in, only the files it covers change, and they change the same way on Windows and Linux.
  - Threshold: 0 differing bytes without a rule; with a rule, 0 changed bytes outside its files and identical results on both platforms.
- **AC-3** `necessary` `empirical` — Every command works on content no layer reads: each command in the registry ([INT-0022](INT-0022-agent-and-human-interfaces.md)) that succeeds on a plain-text copy of a scenario also succeeds on every kind in the corpus with no layer on, shows each file it cannot read by size and hash, and still completes when a layer is made to fail on a file.
  - Threshold: 0 failures, and 0 raw bytes of unreadable content in output meant for people to read.
- **AC-4** `necessary` `empirical` — Layers change no stored object: snapshotting the corpus with every shipped layer on and with every layer off gives the same object ids; turning layers on or off afterwards leaves every stored object, op and record unchanged; and a replica with no layers syncs, verifies and checks out everything that a replica with layers made.
  - Threshold: 0 differing object ids, 0 changed objects and 0 failures.
- **AC-5** `necessary` `empirical` — Merge drivers: a project registers a driver for the corpus's synthetic language; on a replica that allows it, every merge of such a file that both sides changed goes through the driver; the result is recorded with the driver's name and version and reads the same on a replica without the driver; and a driver that is not allowed, fails, crashes or runs past its time limit leaves a conflict term.
  - Threshold: 100% of driver merges in the scenario corpus, with 0 blocked merges.
- **AC-6** `necessary` `empirical` — No replica runs a merge driver that has not been allowed on its machine, even when synced repository policy names one.
  - Threshold: 0 runs across adversarial tests, including policy synced from a hostile peer.
- **AC-7** `necessary` `formal` — Layer independence: no core crate depends on a layer crate, directly or through another crate.
  - Obligation: `unstated`.
  - Proxy: a dependency rule in `cargo policy` over crates marked as core or layer, plus a CI build and test run of the core crates with every layer crate left out.
- **AC-8** `necessary` `asserted` — The project owner approves the line between the core and the layers, and the conformance corpus, before the suite first runs.
  - Signers: project owner.

## Rationale
- Owner direction during the Sprint 0 Test phase: Periapsis must work with
  almost any development system, including projects written only in natural
  language, binaries and assembly, and languages that no human knows or
  speaks, such as a neuralese coding language. It is a version control and
  distribution substrate, like Git but refined for people and agents building
  together in real time.
- Agents need not produce code that people can read. A core that assumed
  text, lines or a known language would fail first on the work agents are
  likely to produce next, and its guarantees would hold for only some
  repositories.
- Most of the core already needs nothing from content. A blob's id is the hash
  of its raw bytes in an envelope ([INT-0006](INT-0006-format-v1.md)), and
  predecessor compression won on source history and on a 64 MiB binary alike
  ([INT-0008](INT-0008-compact-history-storage.md); R2-E27b, R3-C). The gaps
  were where chapters assumed lines, tests or code: conflict hunks,
  confirmations, comment tags, modules and diff output.
- Conflicts held as whole versions work on any bytes, and the research already
  describes a conflicted file as a term (base, side₁, side₂, …), following
  Jujutsu (III §3.2).
- Git shows what goes wrong when the core reads content. A line merge cuts a
  binary at arbitrary newline bytes, so Git refuses to merge binaries by line.
  `core.autocrlf` rewrites bytes on the way in and out. An attribute can name
  a merge driver, but its command must be set up on each machine, so cloning
  never runs a program the repository chose.
- Parsers are large, change often and fail on input they do not expect.
  Keeping them in layers keeps reversibility, crash safety, convergence and
  signatures independent of them.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  III §3.2, R2-E27b, R3-C.

## Alternatives
- **Text first, with binaries as a special case (Git's model).** Every feature
  then needs a text path and a binary path, and content that is neither, such
  as a language no tool knows, falls between them.
- **Detect the kind of content from its bytes.** Kept inside layers only.
  Detection is a guess, and a core that guessed would treat the same bytes
  differently whenever its detectors changed. Declarations decide; a layer can
  still check for itself whether it can read a file.
- **Normalise text on the way in, as Git's `core.autocrlf` does.** Rejected as
  a default: it changes bytes the author wrote and breaks exact round trips.
  Kept as an opt-in rule for named paths.
- **Merge drivers that synced policy can run.** Rejected: a hostile peer could
  then run code on every replica that syncs with it.
- **A fixed set of built-in layers.** No fixed set covers formats that do not
  exist yet, such as languages agents invent, so projects can register their
  own.

## Consequences
- Every core feature needs a path that works with no layer, and every chapter
  that mentions lines, text, tests or code has to say what happens to other
  content. The chapters revised with this intent do so: storage
  ([INT-0008](INT-0008-compact-history-storage.md)), rewriting
  ([INT-0011](INT-0011-change-identity-and-rewriting.md)), merge
  ([INT-0012](INT-0012-conflicts-and-merge.md)), generated files
  ([INT-0013](INT-0013-derived-data.md)), the working copy
  ([INT-0014](INT-0014-working-copy-and-workspaces.md),
  [INT-0004](INT-0004-windows-and-linux.md)), confirmations
  ([INT-0015](INT-0015-phases-and-confirmation.md)), records
  ([INT-0017](INT-0017-event-records-and-book-view.md)), certificates
  ([INT-0018](INT-0018-proof-pairing-certificates.md)), review requests
  ([INT-0021](INT-0021-lanes-and-review-requests.md)), interfaces
  ([INT-0022](INT-0022-agent-and-human-interfaces.md)), the Git bridge
  ([INT-0025](INT-0025-git-bridge.md)), orbits
  ([INT-0026](INT-0026-sprint-loops-on-periapsis.md)) and intent links
  ([INT-0032](INT-0032-intent-revisions-and-review-flags.md)).
- With no layer, people see less. A diff shows sizes and hashes, and a
  conflict is resolved by choosing a side or writing a new version. Projects
  that want more add layers.
- Repository policy needs a place to declare kinds of content, normalisation
  rules and merge drivers, and each machine needs a local list of the drivers
  it allows.
- Layers read untrusted bytes, so each one needs the fuzzing that the decoders
  of [INT-0006](INT-0006-format-v1.md) get.
- The corpus is large, since one file is over 4 GiB. A smaller corpus without
  the largest files can run on every change, and the full one on changes to
  storage, sync, merge and the working copy.

## Transition history
- 2026-11-01: created as `proposed` (Sprint 0 Loop; owner direction given during the Sprint 0 Test phase).
