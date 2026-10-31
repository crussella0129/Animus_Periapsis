# INT-0021 — Lanes, review requests and batched promotion (lane → RR → dev → main)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0021
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Many agents land work concurrently without git worktrees and without a shared
mutable branch to race on.

- **Lane** — a workspace ([INT-0014](INT-0014-working-copy-and-workspaces.md)).
- **Review Request (RR)** — an immutable signed event: {rr_id, lane, change
  IDs, base op, certificate hash, claimed intents, verifier-computed tier,
  touched modules (derived), status events}. RRs follow change IDs through
  rebases.
- **dev** — a derived speculative-integration view (base plus admitted RRs)
  recomputed by the queue; nobody pushes to it.
- **main** — the `public` phase.

**Gate 1 — lane → dev, per RR:** the full proof-pairing gate
([INT-0018](INT-0018-proof-pairing-certificates.md),
[INT-0019](INT-0019-cheat-resistant-verifier.md)), confirmations for tested
claims, and ledger checks for qualitative intents. The gate computes the RR's
tier and rollup class — `always` (proved/tested, no hot files), `maybe`
(default), `iffy` (hot or generated files, wide fan-out), `never` (any Book
edit, migrations, performance-sensitive changes) — and never reads them from
the RR.

**Gate 2 — dev → main, batched:** select with a filter query
([INT-0023](INT-0023-query-language.md)); pack the selection into
module/directory-disjoint groups (generated files excluded as derived data);
size groups with an AIMD window (start 8, +1 per green batch, halve on red,
cap 32); test each group against the current main; bisect on failure and
confirm single failures once; promote a green group with one signed op; review
per batch, grouped by intent, showing evidence profiles, strength, risk and any
Book mutation-score diff. An optional fast lane (file-level packing) is allowed
only with a post-merge sweep and automatic revert.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Zero main breaks: in simulation with real change shapes (300 and 1,000 concurrent RRs, 8 and 16 workers, 8 seeds each), the default pipeline never lands two interacting RRs untested together.
  - Threshold: 0 breaks (R5 §C.3: AIMD-DIR 0 in every configuration).
- **AC-2** `necessary` `empirical` — Throughput: the default pipeline lands RRs at least 1.8× faster than serial promotion, with at least 2.5× lower mean latency, in the same simulation.
  - Note: R5 measured 2.0–2.2× and 3.2–3.3×.
- **AC-3** `necessary` `formal` — Tier and rollup class are functions of verifier evidence only: no code path reads them from an RR's claimed fields.
  - Obligation: `unstated`.
  - Proxy: tests with over-claiming RRs, plus review.
- **AC-4** `necessary` `empirical` — An RR stays attached to its changes through every rewrite in the scenario corpus.
  - Threshold: 100%.
- **AC-5** `necessary` `formal` — Promotion is atomic: a batch becomes `public` through exactly one signed op, or not at all.
  - Obligation: `unstated`.
  - Proxy: crash-injection tests on promotion.
- **AC-6** `supporting` `empirical` — Real multi-agent telemetry confirms the simulation's interaction rates and the right disjointness unit (crate, module or build target).

## Rationale
- Prior art: Zuul's dependent pipeline with a TCP-like window; Rust's bors
  rollup classes; Uber SubmitQueue, which treats changes that share build
  targets as conflicting (R5 §C.2).
- Simulation on 4,997 real helix change shapes: directory/module-disjoint
  packing had 0 main breaks in every configuration, while file-level packing
  broke main 0.1–4 times per run, because cross-file semantic interactions
  landed untested together (R5 §C.3–C.4).
- An adaptive window matched tier-based batch sizing, and agents over-claiming
  their tier cost only about 4% more CI, so tiers were dropped from batching
  and kept for choosing mandatory checks and routing review (R5 §C.4).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  Part VIII §C.

## Alternatives
- **Serial promotion** (637 rounds for 300 RRs) and **FIFO batches of 4** (494
  rounds): safe but slow.
- **File-disjoint packing, with or without tier-based sizing:** faster but
  breaks main (0.12–4.12 breaks per run).
- **Git worktree per lane with a shared dev branch.** Bookkeeping plus a
  mutable branch that agents race on.

## Consequences
- The simulation used assumed failure and interaction rates and one
  repository's change shapes, so AC-6 must close the gap with telemetry.
- A Book edit is always rollup class `never`, so intent changes always get
  their own human-reviewed promotion.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 Part VIII §C).
