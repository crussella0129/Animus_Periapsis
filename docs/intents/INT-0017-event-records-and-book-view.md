# INT-0017 — Project records are immutable signed events; the Book is a view

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0017
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Intents, work items, sprints, reviews, review requests and attestations are
immutable signed event records in the op DAG — one record per event, typed by
reverse-domain names such as `dev.animus.intent`. The Book (the Sprint Loops
`docs/` view of intents, work, sprints and navigation) is a derived projection
of those records. A record's content fields, such as an intent's statement or
an evidence payload, are byte strings that may hold any content
([INT-0034](INT-0034-any-content.md)). Where a record or field that the Book
shows has content no rendering rule covers, the Book shows the record's type
and the content's hash and size in its place; it never fails or leaves the
record out. Concurrent events never conflict spuriously. A genuine
collision on one record — two agents claiming the same task — surfaces as
divergence with an explicit resolution policy (for example leases, claims, or a
CubiKan-style expected-revision check), and is never resolved silently.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Concurrent appends by 2, 4 and 8 agents produce no spurious conflicts.
  - Sample: 300 trials per cell, 20–200 tasks. Threshold: 0% (R3-D measured 0% for per-record events against 100% for a single shared ledger file).
- **AC-2** `necessary` `empirical` — Genuine same-record collisions surface as divergence and are never merged automatically.
  - Threshold: 100% of collision trials.
- **AC-3** `necessary` `formal` — The Book view is a pure function of the event records (an instance of INT-0013 AC-1).
  - Obligation: `unstated`.
  - Proxy: delete-and-recompute invariant test.
- **AC-4** `necessary` `empirical` — Round trip: Book Markdown → event records → Book reproduces every intent chapter's fields and sections, byte for byte after the normalisation rules documented with the importer.
  - Note: the same path serves INT-0027.
- **AC-5** `supporting` `asserted` — New record types can be added under new reverse-domain names without a protocol change.
  - Signers: human reviewer of the format spec.
- **AC-6** `necessary` `empirical` — Records whose content fields hold invalid UTF-8, binary data or text in a synthetic language are stored and synced byte for byte, and the Book view shows every one of them, by type, hash and size where no rendering rule covers its content, without failing or leaving it out.
  - Threshold: 0 differing bytes, 0 failures and 0 records left out, over a corpus of such records.

## Rationale
- Radicle stores issues and patches in the repository as signed, typed
  operation DAGs extensible by reverse-domain names; Fossil keeps project
  records in the repository too (II §2.5).
- R3-D: under concurrent updates a shared Markdown ledger always conflicts on
  appends, per-record files never do, and a CRDT would silently "resolve" the
  genuine collisions — exactly the wrong behaviour.
- CubiKan intent units already use immutable origins, revisioned lifecycle
  transitions, and rejection of stale expected revisions.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.5, R3-D, R3 §6.

## Alternatives
- **Periapsis Collaborative Objects as CRDT operation DAGs (Radicle COB
  style).** Removed: immutable events are enough and a CRDT hides genuine
  collisions (R3-D).
- **Shared Markdown ledgers in Git (git-based Sprint Loops today).** 100%
  conflicts under concurrent appends (R3-D).

## Consequences
- The collision policy (leases, claims or expected revisions) is a product
  decision still to be made (R3 §6).
- The Markdown Book stays a first-class view for humans and Git mirrors, so its
  rendering rules are part of the format contract.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 steps 21–22 and R3-D).
- 2026-11-01: revised while `proposed`, on owner direction: records must hold any content. The Intent says content fields are byte strings, and that the Book shows a record it cannot render by type, hash and size instead of failing or leaving it out; AC-6 checks this.
