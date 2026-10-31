# INT-0012 — Conflicts as data and a measured merge pipeline

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0012
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Merging never blocks. Conflicts are stored as conflict terms inside changesets
(zdiff3-style, including the base), so every operation completes and anyone
can resolve the conflict later. The merge pipeline is: histogram diff3 → a
resolution store keyed by the full conflict block, with replay → optional
structured (AST) merge for supported languages → optional loose-adjacency
mode. Output of the last two stages is admitted only after green build/test
confirmation plus a duplicate-definition check for moved blocks. Every resolved
conflict block records which actor resolved it and how (by hand, from the
resolution store, or by structured merge). Myers-minimal diff is available as
an option; histogram is the default.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — diff3 parity: clean-versus-conflict status agrees with `git merge-file` on at least 99.5% of at least 800 random cases, with identical output whenever both are clean.
  - Note: R2-E20 measured 99.75% for the prototype `peri-merge`.
- **AC-2** `necessary` `formal` — Clean-merge laws: identity `merge(b, b, x) = x`, idempotence `merge(b, x, x) = x`, and symmetry `merge(b, x, y) = merge(b, y, x)` up to the order of conflict sides.
  - Obligation: `unstated`.
  - Proxy: property tests.
- **AC-3** `necessary` `formal` — Conflict-term algebra: combining conflict terms is commutative, idempotent and associative up to a normal form, and replicas that combine the same terms converge.
  - Obligation: `unstated` (R4 §4.4 formalization priority 2; left open by R1 §4).
  - Proxy: property tests.
- **AC-4** `necessary` `empirical` — Resolution reuse: when real changes are rebased repeatedly onto successive trunk versions, the resolution store answers at least 65% of conflict blocks automatically.
  - Note: R2-E28 measured 69–75% exact repeats over 573 changes.
- **AC-5** `necessary` `empirical` — Structured and loose-adjacency results are never admitted without green build/test confirmation, and the duplicate-definition check flags every moved-block duplication in its fixture corpus.
  - Note: R2-E22 found Mergiraf duplicated a moved `impl` block in 2 of 45 verifiable resolutions.
- **AC-6** `supporting` `empirical` — Loose adjacency reduces conflicts by at least 10% relative on real edit pairs while independent clean merges stay 100% exact.
  - Note: R2-E29 measured −12% to −14% with 3,028 of 3,028 exact.
- **AC-7** `supporting` `empirical` — Structured merge resolves at least 30% of real diff3 conflicts in supported languages (Phase 5).
  - Note: R2-E22 measured 34–40% with Mergiraf 0.20.

## Rationale
- Conflicts as data mean rebase never stops half-way, which matters most for
  autonomous agents (II §2.1).
- diff3 is sound in practice: never order-dependent; silent wrong results only
  in 0.03–0.3% of synthetic cases with 70% repeated lines (R2-E14); and 0
  genuine silent errors across 5,325 real reconstructed edit pairs (R2-E19/21).
- Histogram versus Myers barely changes merge outcomes (21.1% → 20.8% conflicts
  on helix) but reads better (R2-E21, II §2.7).
- Most conflicts recur under repeated rebase, so a shared resolution store is
  the cheap way to "resolve once" (R2-E28).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.4, II §2.7, R2 §1–3.

## Alternatives
- **Pijul-style patch theory as the core (format v2).** Removed from the
  roadmap: no correctness gap observed, and reuse captures most of the
  "conflicts never come back" benefit (R2-E19/21/28, R3 §4.1). Revisit only on
  real multi-agent evidence.
- **Local-only resolution reuse (git rerere).** Not shared between agents or
  machines.
- **Entropy-based decision-bit attribution.** Removed: with the only computable
  probability model it reduces to a count of resolved blocks (R3 §4.1); actor
  attribution stays.

## Consequences
- Structured merge needs parsers per language and move detection before it can
  be trusted, and always sits behind confirmation (INT-0015).
- The resolution store is synchronised data with its own key format.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 steps 14–17 and R2 §3).
