# INT-0006 — Format v1: canonical, versioned, hash-agile objects

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0006
- **State:** planned
- **Work evidence:** [T-002 build plan](../sprints/s0/sprint-plans/build-plan.md#t-002-format-v1-specification), [T-003 build plan](../sprints/s0/sprint-plans/build-plan.md#t-003-algorithm-tagged-hashes-in-the-hash-crate), [T-004 build plan](../sprints/s0/sprint-plans/build-plan.md#t-004-canonical-binary-encoding-in-the-codec-crate), [T-005 build plan](../sprints/s0/sprint-plans/build-plan.md#t-005-golden-corpus-and-spec-consistency)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

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
- Any encoding change requires a format-version change and new golden files.
- Derived data must be keyed by algorithm id and version so that a hash
  migration is an explicit derived-data migration (INT-0013).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §3.1 as slimmed by R3 §5).
- 2026-10-31: `proposed` → `planned` (Sprint 0 build plan T-002 to T-005: spec first, then hashes, canonical encoding, golden corpus). After the first plan critique: object identity as the hash of the whole envelope, the reserved-kind error, and staged freezing (what Sprint 0 freezes versus the Phase 1 freeze of format v1) recorded in Intent. After the second plan critique: AC-3's "CI" defined as the canonical suite, locally and in the GitHub matrix; golden values carried as worked examples in the spec; freezing takes effect at the owner's AC-6 sign-off.
