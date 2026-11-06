# INT-0028 — An information-theoretic entropy layer (abandoned)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0028
- **State:** abandoned
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
As proposed in Part III of the master report: treat every repository state as
a microstate and measure operations, conflicts, tests and agent selection in
bits, through a `peri-entropy` crate and a `peri entropy` command. Components:
a Landauer ledger (`bits_destroyed` on every op); conflict entropy with
decision-bit attribution per actor; specification entropy
S_spec = log₂(1 + D) as a quality gate for agent-written code; change entropy
(Hassan) and per-hunk surprisal as risk and review-ordering signals;
selection-information and verifier-throughput accounting; a Shannon divergence
budget H_heads per intent.

This chapter is kept so the decision is not re-litigated. What survived lives
elsewhere:
- the reversibility principle → [INT-0009](INT-0009-reversible-by-default.md);
- the definition of derived data as zero conditional entropy given
  authoritative state, algorithm and version → [INT-0013](INT-0013-derived-data.md);
- information-gain test ordering → [INT-0015](INT-0015-phases-and-confirmation.md) AC-5;
- a head-count (Hartley) divergence budget →
  [INT-0026](INT-0026-sprint-loops-on-periapsis.md) AC-3;
- change entropy → an optional analytics plugin, not planned;
- mutation testing → array-test's optional test-improvement advisor, outside
  the VCS.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — S_spec adds significant predictive information about real defects beyond function size, complexity and coverage.
  - Note: refuted by R3 §2 — likelihood-ratio p = 0.145 once complexity is controlled; held-out accuracy 0.738 → 0.736 when added.
- **AC-2** `necessary` `empirical` — Hunk surprisal distinguishes buggy lines from their fixes.
  - Note: not replicated by R1-E8 (helix p = 0.57, bat p = 0.91).
- **AC-3** `necessary` `formal` — Decision-bit attribution is order-independent and adds information beyond a count of resolved conflict blocks.
  - Note: R1-E5 showed order dependence without Shapley weighting; R3 §4.1 showed the only computable model reduces it to a count.
- **AC-4** `necessary` `empirical` — The Shannon H_heads budget cannot be gamed.
  - Note: refuted by R1-E13 — one dominant head plus 1,000 trickle heads scored 0.18 bits and passed a 4-bit budget.

## Rationale
Reasons for abandonment, all from Sprint 0's research artifacts:
- Across 127 real bugs in 9 Python projects, S_spec's apparent signal was fully
  explained by size and complexity; function size alone (AUC 0.725, free) beat
  S_spec (0.588, about 10 suite runs per function) (R3 §2). This matches
  Papadakis et al. (ICSE 2018).
- The per-op `bits_destroyed` field is 0 for every op except erasure ops, so it
  duplicates the op type (R3 §4.1).
- The selection bound log₂C(N,k) was refuted as a lower bound (the floor is
  log₂(N/a)), and throughput accounting reduces to ordinary metrics: candidates,
  verifier pass rate, acceptable fraction, review time (R1-E6, R3 §4.1).
- The physical energy is negligible — about 2.87×10⁻²¹ J per bit, about 2.87 nJ
  for 10¹² bits — so the value was only ever in the accounting (R1 #19, III §8).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  Parts III, IV and VI.

## Alternatives
- **Keep S_spec as an advisory metric.** Rejected: all cost and no benefit; the
  literature supports mutation testing only as a test-writing aid (R3 §2.4).
- **Keep change entropy in core.** Demoted: it adds modest signal beyond change
  counts (8 of 8 splits) but does not consistently beat prior-fault counts
  (R1-E7).

## Consequences
- No `peri-entropy` crate and no `peri entropy` command.
- Open research questions stay recorded in the master report (I.7, III §10)
  and are non-binding: a minimal-decision-bit merge pipeline, entropy-driven
  sprint exits, and a multi-project S_spec study.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 Part III).
- 2026-10-31: `proposed` → `abandoned`: the Round 1 validation and Round 3 necessity audit refuted or removed every component (R1-E5/E6/E8/E13, R3 §2 and §4.1); surviving ideas moved to INT-0009, INT-0013, INT-0015 and INT-0026.
