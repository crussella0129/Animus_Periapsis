# INT-0018 — Proof-pairing certificates on every change

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0018
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Every change carries a certificate pairing it with machine-checked claims:
(a) **local claims**, theorems about what the change's code does, and
(b) **intent claims**, which Book intents it discharges and with what evidence.
`formal` acceptance criteria are Lean 4 propositions; the verifier
([INT-0019](INT-0019-cheat-resistant-verifier.md)) recomputes every claim's
axiom profile and every intent's evidence profile with the Lean kernel, and
trusts nothing from the producer. Evidence tiers:

| Tier | Meaning |
|---|---|
| P — proved | Kernel-checked proof about Lean code or an extracted model of the Rust code |
| C — conformance | Production code differentially tested against a proved Lean executable spec |
| T — tested | Verifier-signed test confirmation |
| A — asserted | Signed sign-off by a non-author principal |

Certificate fields: change id; code hashes; Lean module hashes and toolchain
hash; claims {local: theorem names; intents: (intent id, spec term name)}; and
a kernel report computed by the verifier. Local proofs are required for
Periapsis's own core algorithms and optional, as the top tier, for user code.
Formalisation order: (1) canonical op-head merge, (2) conflict-term normal
form, (3) reversibility, (4) derived-data invariant, (5) phase-gate
monotonicity. Evidence is data, never axioms: only a proof carries logical
force.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — The gate rejects any certificate whose claims depend on `sorryAx` or on an axiom not registered in the Book (registered: `propext`, `Classical.choice`, `Quot.sound` and explicitly approved trusted primitives).
  - Threshold: 100% of the INT-0019 red-team corpus.
- **AC-2** `necessary` `formal` — INT-0010 AC-1 and AC-2 are discharged by kernel-checked proofs about the Lean model of the merge, and the Rust implementation conforms to that model (tier C).
  - Obligation: `Periapsis.OpMerge.conv`, `Periapsis.OpMerge.compat`.
  - Proxy: 0 disagreements over at least 20,000 random inputs (R4 §2.3: 0 for the canonical merge, 3,946 for the buggy pairwise one).
- **AC-3** `necessary` `empirical` — A change that breaks a formal intent is rejected, and the report names the broken intent.
  - Note: R4 §2.2 localised change c2's regression to INT-conv.
- **AC-4** `necessary` `empirical` — The Book view shows each intent's evidence profile: tier per claim, strength and risk.
- **AC-5** `supporting` `empirical` — The Lean toolchain is pinned per repository at 4.33.1 or later and recorded in every certificate.
- **AC-6** `supporting` `empirical` — Core Rust modules are extracted to Lean (Aeneas or equivalent) for tier-P proofs about the actual code (Phase 5).

## Rationale
- Intent pairing is cheap and general — it works even when code is not
  formalised — while local proofs cost from about 0.25× to 9× the code size;
  so intent pairing is core and local proofs are tiered (R4 §1).
- Writing the right statement is the bottleneck for agents (VeriBench
  theorem-gold coverage ≤ 0.11; VERINA 4.9% fully verified), so humans own the
  statements in the Book and machines check proofs (R4 §3).
- Design for provability: the canonical merge's convergence proof took 2
  lines, while equivalence with legacy behaviour took 69 (R4 §2.6).
- Treating tested evidence as Lean axioms made the logic inconsistent once a
  false claim met a proved counterexample, hence evidence as data (R4 §2.5).
- Precedent: proof-carrying code (Necula 1997); SymCrypt and Plonky3 Rust →
  Lean with AI provers (R4 §3).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  Part VII (R4).

## Alternatives
- **Tested evidence as Lean axioms.** Rejected: inconsistency risk (R4 §2.5).
- **Local proofs for all user code.** Not feasible with today's tools (R4 §1).
- **No formal layer.** Gives up the core differentiator: moving human review
  from code to intents, with a kernel checking the connection.

## Consequences
- Lean becomes part of the toolchain; it is not yet installed on the
  development machines.
- Proofs must be maintained whenever formalised code changes, and Lean
  toolchain drift must be managed by pinning.
- Proofs about Lean models cover Rust only through tier C or extraction.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 Part VII).
