# INT-0006 — Format v1: canonical, versioned, hash-agile objects

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0006
- **State:** active
- **Work evidence:** [T-002 build plan](../sprints/s0/sprint-plans/build-plan.md#t-002-format-v1-specification), [T-003 build plan](../sprints/s0/sprint-plans/build-plan.md#t-003-algorithm-tagged-hashes-in-the-hash-crate), [T-004 build plan](../sprints/s0/sprint-plans/build-plan.md#t-004-canonical-binary-encoding-in-the-codec-crate), [T-005 build plan](../sprints/s0/sprint-plans/build-plan.md#t-005-golden-corpus-and-spec-consistency), [Sprint 1 T-009 build plan](../sprints/s1/sprint-plans/build-plan.md#t-009-record-sprint-0s-github-actions-runs-and-owner-sign-offs), [Sprint 1 T-001 build plan](../sprints/s1/sprint-plans/build-plan.md#t-001-specify-and-implement-the-provisional-operation-object-kind), [T-010 backlog](../work/tasks.md), [T-012 backlog](../work/tasks.md), [Sprint 2 T-002 build plan](../sprints/s2/sprint-plans/build-plan.md#t-002-specify-and-implement-the-provisional-changeset-and-view-kinds-and-repository-paths), [Sprint 2 T-001 build plan](../sprints/s2/sprint-plans/build-plan.md#t-001-record-sprint-1s-github-actions-runs-and-owner-sign-offs), [T-013 backlog](../work/tasks.md), [Sprint 3 T-001 build plan](../sprints/s3/sprint-plans/build-plan.md#t-001-record-sprint-2s-github-actions-runs-and-owner-sign-offs), [T-017 backlog](../work/tasks.md), [Sprint 4 T-001 build plan](../sprints/s4/sprint-plans/build-plan.md#t-001-record-the-sprint-3-checkpoints-github-actions-runs-and-owner-sign-offs)
- **Completion evidence:** [T-002 completion](../work/completed-tasks.md#t-002-sprint-0), [T-003 completion](../work/completed-tasks.md#t-003-sprint-0), [T-004 completion](../work/completed-tasks.md#t-004-sprint-0), [T-005 completion](../work/completed-tasks.md#t-005-sprint-0), [Sprint 1 T-001 completion](../work/completed-tasks.md#t-001-sprint-1), [Sprint 2 T-002 completion](../work/completed-tasks.md#t-002-sprint-2), [Sprint 2 T-003 completion](../work/completed-tasks.md#t-003-sprint-2)
- **Code evidence:** [hash crate](../../crates/hash/src/lib.rs), [codec crate](../../crates/codec/src/lib.rs), [golden test](../../crates/codec/tests/golden.rs), [operation.rs](../../crates/codec/src/operation.rs), [changeset.rs](../../crates/codec/src/changeset.rs), [view.rs](../../crates/codec/src/view.rs), [path.rs](../../crates/codec/src/path.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [Sprint 0 GitHub Actions record](../sprints/s1/sprint-tests/ci-sprint0.md), [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md), [Sprint 1 CI record](../sprints/s2/sprint-tests/ci-sprint1.md), [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md), [Sprint 2 CI record](../sprints/s3/sprint-tests/ci-sprint2.md)
- **Documentation evidence:** [format spec v1](../spec/format-v1.md)

## Intent
Every object Periapsis persists or transmits has exactly one byte encoding,
begins with a format-version byte, and refers to other objects by
`HashRef = (alg_id: u8, digest)`. BLAKE3-256 is the default algorithm. Every
signed payload names its hash and signature algorithms. A repository can hold
objects under more than one hash algorithm during a migration without
bidirectional mapping tables. The format is specified in
`docs/spec/format-v1.md` before the code that implements it, and golden files
pin every encoding.

Format v1 object set (after the R3 audit): `Blob`; `Changeset` (Bonsai-style:
ordered parents, change-ID cache, sorted map from path to
{Add/Modify(HashRef, mode) | Delete | Rename(from)}, author, intent references,
description, optional conflict terms); `Operation` (parent ops, view hash,
actor, advisory wall clock, signature; rewrite ops carry old → new mappings,
erasure ops carry erased ids and byte counts); `View` (heads, named pointers
with conflict states, per-workspace working-copy change); and typed event
records ([INT-0017](INT-0017-event-records-and-book-view.md)). Trees and every
index are derived ([INT-0013](INT-0013-derived-data.md)).

A kind's body may first be specified as *provisional*, so storage can be
built and crash-tested before every field it will carry is designed. A
provisional encoding is specified, implemented and pinned by golden files
like any other, but it is excluded from the AC-6 sign-off and from freezing:
until the Phase 1 exit gate, its spec text, golden files and encoder may
change together in one commit. Before it freezes it must carry the fields
the object set above names for it. Sprint 1 specifies `Operation`
provisionally as its parent ops and the ids of the objects it makes
reachable; the object ids stand in for the view hash until `View` is
specified. Sprint 2 specifies `Changeset` provisionally as its ordered
parents, change-ID cache, path map (put with blob and mode, or delete) and
description, and `View` as its heads and per-workspace working-copy changes;
the `Operation` body then names its view instead of the object ids. Author,
intent references, renames, conflict terms, named pointers, actor, time and
signature are still to come.

Removed from v1: a HEAD/pointer file, separate evolution-marker objects, CRDT
record operations, and chunk lists in core (R3 §4.1).

Object identity: an object's id is the `HashRef` of its whole envelope,
`format_version ‖ kind ‖ body`, so the same body under two kinds or versions
has two ids. A blob's id is therefore the hash of `01 ‖ 01 ‖ content`, not of
the bare content. A reserved kind (a kind the registry names but whose body is
not yet specified) is rejected with its own error, distinct from an unassigned
kind.

Freezing happens in stages. The spec states the exact bytes of every golden
value as a worked example, so golden files are checked against the spec
rather than only against the encoder that wrote them. An encoding becomes
frozen when the owner signs off the spec section that defines it (AC-6), not
when a golden file is first committed. Sprint 0 covers the primitives,
`HashRef`, the envelope header and the blob body. Format v1 as a whole is
frozen at the Phase 1 exit gate, once every object kind's body and the storage
representation (INT-0007, INT-0008) are specified.

Boundaries: this intent owns the logical encoding and hashing. How bytes are
laid out on disk is [INT-0007](INT-0007-crash-safe-storage.md) and
[INT-0008](INT-0008-compact-history-storage.md).

## Acceptance criteria
- **AC-1** `necessary` `formal` — Canonical round trip: for every value `v`, `decode(encode(v)) = v`, and every byte string `b` that `decode` accepts satisfies `encode(decode(b)) = b`. Non-minimal integers, unsorted or duplicate map keys, invalid UTF-8 and trailing bytes are rejected.
  - Obligation: `unstated`.
  - Proxy: property tests with at least 10,000 cases per encoded type, plus mutated valid encodings.
- **AC-2** `necessary` `formal` — Hash agility: every `HashRef` carries its algorithm id; decoding an unknown id, or a digest whose length does not match its algorithm, is an error; two `HashRef`s are equal only when algorithm and digest are both equal.
  - Obligation: `unstated`.
  - Proxy: unit tests.
- **AC-3** `necessary` `empirical` — Golden files: the encodings of a fixed object corpus are committed, and the canonical suite (`cargo ci`, run locally and in the GitHub matrix) fails on Windows and Linux whenever an encoding changes without a format-version change.
- **AC-4** `necessary` `empirical` — The BLAKE3-256 implementation matches the official BLAKE3 test vectors, including the empty input.
  - Threshold: 100% of the vectors used.
- **AC-5** `necessary` `formal` — Decoders are total: `decode` never panics on any input; it returns a value or an error.
  - Obligation: `unstated`.
  - Proxy: property tests over random byte strings (at least 100,000 cases) now; fuzz targets with clean runs at the Phase 1 exit gate.
- **AC-6** `necessary` `asserted` — `docs/spec/format-v1.md` specifies each object type's encoding before its implementation lands, and every spec change is signed off by a human reviewer.
  - Signers: project owner or human reviewer.
- **AC-7** `supporting` `empirical` — Every signature payload begins with the format version, the hash algorithm id and the signature algorithm id (checked when signing lands in INT-0016).

## Rationale
- Git's SHA-1 → SHA-256 transition needs bidirectional mapping tables, cannot
  mix algorithms in one repository, and its signed payloads do not name their
  hash; designing agility in from the start costs one byte per reference
  (II §2.6, R2-E27a).
- BLAKE3 measured 4.9 GB/s against 0.21 GB/s for SHA-256 on the test machine
  (23×; that CPU appears to lack SHA extensions) (R2-E27a).
- Bonsai-style deterministic serialization — ordered parents, sorted sets and
  maps, no added/modified distinction — gives one canonical changeset form
  (II §2.3).
- Spec before code and golden files guard against format lock-in (I.3
  principle 8; I.6).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.6, II §3.1, R2-E27a, R3 §4.

## Alternatives
- **SHA-256 as the default.** Slower on the measured hardware; it remains
  available as a second algorithm id.
- **Deterministic CBOR (RFC 8949 §4.2) or Protocol Buffers.** Workable, but
  each brings a larger rule surface or a dependency; an in-repo encoding keeps
  the decoder small, total and fully specified. Revisit if interop needs a
  self-describing form.
- **Multihash-style varint codes.** Same idea; a one-byte tag is enough for the
  algorithm space Periapsis needs.

## Consequences
- Any change to a frozen encoding (one the owner has signed off under AC-6)
  requires a format-version change and new golden files. Before that
  sign-off, the spec's worked example, the golden file and the encoder change
  together in one commit.
- Derived data must be keyed by algorithm id and version so that a hash
  migration is an explicit derived-data migration (INT-0013).
- Stores written with a provisional encoding are development stores: a
  change to that encoding before Phase 1 exit may make them unreadable, and
  no migration is owed for them.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §3.1 as slimmed by R3 §5).
- 2026-10-31: `proposed` → `planned` (Sprint 0 build plan T-002 to T-005: spec first, then hashes, canonical encoding, golden corpus). After the first plan critique: object identity as the hash of the whole envelope, the reserved-kind error, and staged freezing (what Sprint 0 freezes versus the Phase 1 freeze of format v1) recorded in Intent. After the second plan critique: AC-3's "CI" defined as the canonical suite, locally and in the GitHub matrix; golden values carried as worked examples in the spec; freezing takes effect at the owner's AC-6 sign-off. After the third: the Consequences bullet on encoding changes now distinguishes frozen (signed-off) encodings from unfrozen ones.
- 2026-10-31: `planned` → `active` (Sprint 0 build, T-002 starts: the spec lands before the code).
- 2026-11-06: revised while `active`, after the Sprint 1 plan critique (round 1, C-009): Intent defines provisional kinds (specified and pinned, excluded from AC-6 sign-off and freezing until the Phase 1 exit gate, required to carry the object set's fields before freezing) and records that Sprint 1's provisional `Operation` uses the reachable object ids in place of the view hash; Consequences add that stores written with a provisional encoding are development stores.
- 2026-11-07: revised while `active` (Sprint 2 Plan): Intent records Sprint 2's provisional `Changeset` and `View` and the `Operation` body naming its view in place of the object ids; the fields still missing are listed.
- 2026-11-07: evidence while `active` (Sprint 2 Loop): `changeset`, `view` and the revised `operation` are specified, implemented and pinned by golden files (provisional). The GitHub-matrix half of AC-3 for this head is pending backlog T-013; the remaining fields are backlog T-012.
