# INT-0020 — Kernel-checked evidence for non-formal intents (the intent calculus)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0020
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Empirical, preference, judged and asserted acceptance criteria — including
aesthetic ones such as "`peri log` is calm and pleasant to scan" — are
certified by kernel-checked propositions about a verifier-owned, signed
evidence ledger, never by data the prover supplies. This chapter gives the
kernel semantics behind the Markdown grammar of
[INT-0002](INT-0002-typed-acceptance-criteria.md).

```
Spec   ::= formal P | empirical E | preference R | judged J | asserted k W
Intent ::= ⟨id, prose, necessary : List Spec, supporting : List Spec⟩
Certified L c I := I.necessary ++ I.supporting ≠ [] ∧ (∀ s ∈ I.necessary, ⟦s⟧) ∧ (∀ s ∈ I.supporting, ⟦s⟧)
Refuted   L c I := ∃ s ∈ I.necessary, ¬⟦s⟧
```

Here `L` is the ledger and `c` the change context (id, authors, artifact
sequence number); `⟦empirical E⟧` is the decidable proposition
`empiricalOK L E c = true`, and likewise for the other modalities. Every check
enforces pre-registration (rubric and seed sequence numbers precede the
artifact; ballots and scores follow it), independence (judge ∉ authors),
blinding, panel membership, distinct judges, completeness (trial ids are
exactly 0…n−1), a minimum sample size, and exact binomial statistics computed
in ℕ without floating point. Strength is the weakest link and risk is Σδ. An
evidence-ledger service writes signed trial, blinded-ballot and rubric-score
records with sequence numbers, and generates the trusted ledger module pinned
in `book.lock` ([INT-0019](INT-0019-cheat-resistant-verifier.md)).

Boundaries: the calculus certifies process integrity and measured outcomes
against pre-registered thresholds. It does not certify that something *is*
beautiful; that rests on the humans who chose the proxies and thresholds and on
the panel, which the Book records as attributed, signed facts.

## Acceptance criteria
- **AC-1** `necessary` `formal` — Consistency: `Certified L c I → ¬ Refuted L c I`.
  - Obligation: `Periapsis.Intent.certified_not_refuted` (R5 proved it for the prototype calculus).
- **AC-2** `necessary` `formal` — The exact tail check decides `P[X ≥ k] ≤ δ` for `X ~ Binomial(n, a/b)` exactly, and agrees with an independent implementation on the reference table: 10/12 at ½ → 0.019287; 199/200 at 0.95 → 0.000404; 970/1000 at 0.95 → 0.001277, certified at δ = 1/100; 960/1000 → 0.0806, refused at δ = 1/100.
  - Obligation: `unstated` (decided by `decide +kernel`, never native evaluation).
- **AC-3** `necessary` `formal` — Misconduct is uncertifiable: sock-puppet ballots, dropped trials, post-hoc rubrics, unblinded ballots, double votes, overclaimed confidence, underpowered samples, pre-dated scores and post-hoc seeds (R5 scenarios S1–S9) are each proved to make certification impossible.
  - Obligation: one theorem per scenario.
- **AC-4** `necessary` `empirical` — The worked example reproduces: the aesthetic `peri log` intent is certified for its honest change with strength `judged` and risk 1020/20000 (5.1%), and the later "cleanup" change is kernel-refuted by its necessary formal proxy.
- **AC-5** `supporting` `empirical` — Calibration with real human panels: inter-rater reliability is measured, and certified preference evidence is checked against later user satisfaction.
- **AC-6** `supporting` `asserted` — Panel and signer authority is expressed in an Abadi-style authorisation logic ("A says s", "A speaks for B") rooted in the Book's human signers (Phase 5 design).
  - Signers: project owner.

## Rationale
- Round 4 kept qualitative intents as opaque prose with `tested`/`asserted`
  data, which was logically safe but let an agent cite any run or sign-off it
  liked (R5 §B.1).
- Making the evidence a proposition about a ledger the prover cannot write
  removes cherry-picking, forgery and filtering: the only route to a
  qualitative intent is an artifact that real, independent, blinded humans
  prefer under pre-registered rules (R5 §B.5).
- Polarity separates falsifiers from sufficiency evidence, giving three
  outcomes: certified, refuted, undetermined (R5 §B.2).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  Part VIII §B.

## Alternatives
- **R4 `Status` data for qualitative intents.** Superseded: no integrity
  conditions on how support was obtained.
- **More formal proxies instead of human evidence.** Rejected: proxies are
  necessary, not sufficient (Goodhart); sufficiency comes from human preference
  evidence (R5 §B.6).

## Consequences
- Sampling assumptions ("trials are i.i.d. draws from real logs") stay outside
  the kernel; they are auditable through the seeds and sequence numbers in the
  ledger.
- Running qualitative intents needs real panels and a ledger service with key
  custody.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 Part VIII §B).
- 2026-10-31: revised while `proposed`, after the Sprint 0 plan critique: `Certified` requires a non-empty spec list, so an intent with no criteria is never vacuously certified, in line with INT-0002's Semantics.
