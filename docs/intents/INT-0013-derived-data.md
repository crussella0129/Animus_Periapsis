# INT-0013 — Derived data is recomputable and versioned

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0013
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Everything computable from authoritative state (objects and op log) is derived
data and never authoritative: trees, per-file history, blame, ancestry and
segment indexes, evolution markers, op heads, intent ↔ change and actor ↔
change indexes, the Book view, the dev integration view, review-request
batches, and declared generated files such as lockfiles. Derived data is keyed
by (input hashes, algorithm id, algorithm version); it can be deleted and
recomputed at any time, and changing an algorithm version is an explicit
derived-data migration. Regenerating a declared generated file needs its
declared generator, an optional layer ([INT-0034](INT-0034-any-content.md));
where that generator is not available, the file is kept and merged like any
other file.

Derived data is what lets cost follow the change
([INT-0035](INT-0035-scale-and-asynchrony.md)), so three more rules bind it:
- **Kept up to date in proportion to what is new.** Bringing derived data up
  to date after *k* new operations costs O(*k* log *n*). It is never
  recomputed from the start because something was added.
- **Never trusted.** A derived entry is bound to its inputs and checked
  before it is used. One that does not match is discarded and recomputed. So
  derived data can be shared between replicas or served by a cache, and a
  wrong or hostile copy costs time, never a wrong answer.
- **Optional.** Every command gives the same result with all derived data
  deleted. Only its cost differs.

The structures scale needs first are the op heads and the merged view of a
set of heads ([INT-0010](INT-0010-lock-free-concurrency.md)), trees with
shared structure, the index of where each object is stored
([INT-0008](INT-0008-compact-history-storage.md)), and the record of each
file's metadata at the last snapshot
([INT-0014](INT-0014-working-copy-and-workspaces.md)).

## Acceptance criteria
- **AC-1** `necessary` `formal` — Derived data is a function of (authoritative state, algorithm id, algorithm version): deleting all of it and recomputing under the same versions reproduces it byte for byte.
  - Obligation: `unstated` (R4 §4.4 formalization priority 4).
  - Proxy: a simulation invariant run after every scenario (INT-0005).
- **AC-2** `necessary` `empirical` — Bumping an algorithm version invalidates exactly the entries keyed by it, and recomputation writes a migration record.
- **AC-3** `supporting` `empirical` — Where its generator is available, a declared generated file takes no part in conflicts, whatever it contains; it is regenerated instead.
  - Note: R5 §C.3 measured 17.5–20% fewer textual conflicts and 8–18% lower mean latency when generated files are treated as derived data.
- **AC-4** `necessary` `empirical` — Upkeep follows what is new: with derived data up to date, a command reads the same number of bytes, within a factor of 2, on stores of 10³ to 10⁶ operations, and after *k* new operations the cost of bringing it up to date grows no faster than *k*.
  - Threshold: [INT-0035](INT-0035-scale-and-asynchrony.md) AC-2.
- **AC-5** `necessary` `empirical` — A wrong entry is never used: every derived entry that is corrupted, cut short, swapped for another entry or written by a hostile peer is detected and recomputed.
  - Threshold: 0 wrong results over a corpus of damaged and forged entries of every derived structure.
- **AC-6** `necessary` `empirical` — Optional: every command in the corpus prints the same output and leaves the same authoritative state with all derived data deleted before it runs.
  - Threshold: 0 differences ([INT-0035](INT-0035-scale-and-asynchrony.md) AC-8).

## Rationale
- Mononoke keeps one canonical authoritative model and computes expensive
  indexes asynchronously as derived data (II §2.3).
- Formally, D is derived iff H(D | authoritative, algorithm id, version) = 0
  (III §2.3). R1-E11 showed the version must be part of the input: the same
  diff algorithm with one setting changed produced 28 versus 30 hunks.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.3, III §2.3, R1-E11, R5 §C.3–C.4.

## Alternatives
- **Storing indexes as authoritative state.** Creates two sources of truth that
  can disagree after a crash or an algorithm change.

## Consequences
- Every derived structure needs a recompute path and an algorithm version from
  the start.
- Hash-algorithm migrations (INT-0006) become derived-data migrations instead
  of rewrites of authoritative state.
- Nothing of this chapter was built by Sprint 6, and the cost shows. Every
  command read the whole op log three to six times and rebuilt its trees
  from the first change: on Linux, `status` with nothing changed took 7.5 ms
  at 10 operations and 42 ms at 800
  ([Sprint 6 research](../sprints/s6/sprint-research/research-report.md)).
- Sprint 6's T-003 keeps what a command derives in memory for as long as the
  command runs, so the log is read once. That is not this chapter's engine:
  the next command derives it all again. Stored derived data, with AC-4 to
  AC-6, comes with the merge and the packed storage form (backlog T-029,
  T-030).
- Whether a partial clone may use derived trees it did not compute depends
  on the open question in [INT-0006](INT-0006-format-v1.md): if a changeset
  commits to its tree, a tree it is given can be checked; if not, it is
  trusted or recomputed.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 12 and R1-E11).
- 2026-11-01: revised while `proposed`, on owner direction: a generator is an optional layer, and generated files can hold anything. The Intent says a declared generated file is kept and merged like any other file where its generator is not available, and AC-3 covers generated files whatever they contain, not only text.
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents and for speed): derived data is what lets cost follow the change, so the Intent adds three rules (kept up to date in proportion to what is new, never trusted, optional) and names the structures needed first. AC-4 to AC-6 are added. Consequences record what its absence cost in Sprint 6's measurements and the in-memory step Sprint 6 takes.
