# INT-0010 — Lock-free concurrency: op log, views and one canonical op-head merge

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0010
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Any number of processes, agents and machines run Periapsis commands
concurrently without locks. Each command reads one consistent view and always
commits its operation, whose parent is the op it started from; no command
blocks on another writer or fails because of one. When a load finds several op
heads, it merges all of them in **one canonical n-way step** relative to their
lowest common ancestor (or a virtual base for criss-cross histories) and stores
the result in a normal form, so replicas that see the same heads in any order,
with any duplication and in any grouping converge on the same view. Pointer
conflicts are first-class values in the view ("moved from A to B or C"); their
resolution policy lives in [INT-0011](INT-0011-change-identity-and-rewriting.md).

## Acceptance criteria
- **AC-1** `necessary` `formal` — Convergence (INT-conv): merging a multiset of heads gives the same view for every permutation and duplication of those heads.
  - Obligation: `Periapsis.OpMerge.conv` (unstated in this repository; R4 proved the prototype's version, `v2_membership_determined`, for general N).
  - Proxy: property tests over random head multisets.
- **AC-2** `necessary` `formal` — Compatibility (INT-compat): with exactly two heads, the canonical merge equals the classic three-way merge.
  - Obligation: `Periapsis.OpMerge.compat` (unstated here; R4 gave a 69-line general proof for the prototype).
  - Proxy: differential tests against a reference three-way merge.
- **AC-3** `necessary` `formal` — Grouping independence: merging partial merges equals the flat merge under the support-set normal form.
  - Obligation: `unstated`.
  - Proxy: property tests (R1-E12: 0 of 20,000 failures with the normal form, 7,756 of 20,000 without).
- **AC-4** `necessary` `empirical` — Criss-cross correctness over at least 4,000 random criss-cross histories.
  - Threshold: 0 failures (R1-E12).
- **AC-5** `necessary` `empirical` — No lost work: a model checker or simulation with at least 8 writers on at least 2 replicas loses no committed op and no reachable change.
  - Threshold: 0 violations (Phase 2 exit gate).
- **AC-6** `necessary` `formal` — Adequacy: AC-1 together with AC-2 rejects every registered mutant merge (pairwise fold, "first head equals last head" fast path, always-base, and the others in the registry).
  - Obligation: `unstated` (R5 §A.7: the conjunction rejects 6/6 mutants; each half alone misses some).

## Rationale
- Jujutsu's op log gives lock-free concurrency across processes and machines:
  each command commits against a snapshot and divergence is merged on the next
  load, so commands cannot fail to commit — the guarantee agents need (II §2.1).
- Model-checked pointer merges (R1-E12): pairwise three-way merging with a
  common base is order-independent; a pure signed-sum algebra fails
  criss-cross correctness (466/4,000); adding a "both sides equal" rule breaks
  order independence (1,472/5,000); the canonical n-way merge is
  order-independent and criss-cross-correct (0 failures) but needs the
  support-set normal form for grouping independence.
- R4 formalised exactly this: the old pairwise fold is provably
  order-dependent, the canonical merge is provably convergent, and a later
  agent "fast path" was rejected and localised to INT-conv.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.1, II §3.3, R1-E12, R4 §2, R5 §A.7.

## Alternatives
- **Pairwise folding of heads.** Order-dependent (R4 a2:
  `mergeV1 5 1 [2,4,4] ≠ mergeV1 5 1 [4,4,2]`).
- **Locks or a central coordinator.** Blocks agents and cannot span machines
  that are offline from each other.

## Consequences
- Divergence is a normal state that policy must resolve (INT-0011).
- Content conflict terms need their own algebra and normal form (INT-0012
  AC-3); this intent covers views and pointers.
- Once formal obligations exist they must be re-established whenever the merge
  changes (proof maintenance is a feature: it localises regressions).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §3.3, R1-E12, R4 and R5 §A.7).
