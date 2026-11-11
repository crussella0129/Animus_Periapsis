# INT-0015 — Phases and the confirmation gate

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0015
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Every change has a phase — `secret → draft → confirmed → public` — computed
from evidence, never stored as a pointer. `draft → confirmed` requires a
confirmation: an array-test-style confirmation root over exactly that change's
tree, together with the result of its proof-pairing certificate
([INT-0018](INT-0018-proof-pairing-certificates.md)), signed by a verifier key
that no agent principal holds. The root records the results of the checks that
repository policy names for each kind of content in the tree, such as tests, a
build, a proof or another checker. Where policy names none, the check is a
signed sign-off by a non-author principal (tier A of INT-0018). Content that no
test runner can check, such as prose, a binary or a language no one reads
([INT-0034](INT-0034-any-content.md)), is therefore confirmed like any other,
and a root that records no check confirms nothing. `confirmed → public`
follows repository policy (for example human approval per intent species, or
batched promotion in [INT-0021](INT-0021-lanes-and-review-requests.md)).
Rewriting `public` history is a phase-divergence error, and `secret` changes
are never pushed. When a change fails a confirmation made of tests with
per-test coverage, the verifier orders re-runs and bisection by expected
information gain, then ranks suspects with Ochiai-style scoring once all
results are in. Any other failed check is reported with its result and no
suspect ranking.

## Acceptance criteria
- **AC-1** `necessary` `formal` — Monotonicity: no change reaches `public` without a valid confirmation signed by a verifier key.
  - Obligation: `unstated` (R4 §4.4 formalization priority 5).
  - Proxy: property tests over every phase-changing op.
- **AC-2** `necessary` `formal` — Phase is a function of evidence: recomputing every change's phase from the op log and confirmations gives the phases shown in the view, and no independent phase pointer exists.
  - Obligation: `unstated`.
  - Proxy: a derived-data invariant test (INT-0013 AC-1).
- **AC-3** `necessary` `empirical` — A confirmation signed with a key registered to an agent principal is rejected.
  - Threshold: 0 accepted across adversarial tests.
- **AC-4** `necessary` `empirical` — Any operation that rewrites a `public` change is refused with a phase-divergence error.
- **AC-5** `supporting` `empirical` — Information-gain ordering reaches full-suite Ochiai localisation quality after a median of at most 20% of the suite on a real suite with injected faults.
  - Note: R2-E16 measured a median of 11 tests (19% of the suite) for 111 of 113 toolz faults; in simulation it needed 9.7 tests against 15.3 for coverage-first and 19.7 for random (R1-E4).
- **AC-6** `necessary` `empirical` — A change whose files have no tests, such as prose, binaries or files in a synthetic language, is confirmed through the checks that policy names for them, by default a non-author sign-off, and a confirmation root that records no check confirms nothing.
  - Threshold: every such change in a scenario corpus is confirmed once its checks pass, and 0 changes are confirmed on a root that records no check.

## Rationale
- Mercurial phases protect published history; Sprint Loops' rule that phase is
  derived from evidence is the same structure (II §2.2, III §1.3).
- "Merged" means "measured": structured-merge output can be silently wrong
  (R2-E22), so admission needs a measured confirmation (build and tests, for
  code), signed by keys agents cannot reach (I.3 principles 5 and 7;
  Biblio §4).
- Information-gain test ordering is the part of the entropy work that survived
  validation on a real suite (FLINT/EntBug; R1-E4; R2-E16).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §3.2, III §4.2, R1-E4, R2-E16, R2-E22; [Biblio report](../sprints/s0/sprint-research/biblio-report.md) §4.

## Alternatives
- **Specification entropy (S_spec) as a quality gate for agent code.** Removed:
  across 127 real bugs in 9 projects it added nothing beyond free size and
  complexity metrics (R3 §2). See [INT-0028](INT-0028-entropy-layer.md).
- **Mutation testing as a gate.** Kept only as array-test's optional
  test-improvement advisor outside the VCS (R3 §2.4).
- **CI conclusion as the authority (git-based Sprint Loops today).** Replaced
  by signed confirmations stored in the op log.

## Consequences
- A verifier service with its own key custody must exist before Phase 3 can
  admit agent changes.
- Information-gain ordering needs per-test coverage data and a noise model; it
  orders re-runs, it does not replace the final full ranking.
- Verification sets the pace of the whole system
  ([INT-0001](INT-0001-periapsis-charter.md) Rationale), so verifiers must be
  able to be many. Policy may name any number of verifier keys. A
  confirmation is an event record
  ([INT-0017](INT-0017-event-records-and-book-view.md)), so verifiers need no
  coordination, and two confirmations of one change do not conflict.
  [INT-0035](INT-0035-scale-and-asynchrony.md) AC-11 measures the pace a real
  fleet reaches.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §3.2, I.5 steps 23–25 and R2-E16).
- 2026-11-01: revised while `proposed`, on owner direction: confirmations must not assume the content is code. The Intent says the confirmation root records the checks repository policy names for each kind of content, by default a non-author sign-off, and keeps information-gain ordering and Ochiai ranking for confirmations made of tests with per-test coverage; AC-6 is added and the Rationale reworded.
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents): Consequences record that verifiers can be many, that confirmations need no coordination between them, and where the pace of verification is measured.
