# INT-0024 — Verifiable distribution: staged sync by DAG negotiation

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0024
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Peers synchronise in three stages: (1) verify identity and signed pointers;
(2) DAG want/have negotiation with exponential batching — the single sync
mechanism for ops, changesets, event records and per-signer attestation
chains; (3) fetch the objects that are still wanted. `secret` changes never
leave a repository. Transports are SSH and QUIC, plus offline bundles.
Range-based set reconciliation (RBSR) is only an optional optimisation for
reconciling very large sets of heads.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — For d missing ops, sync transfers about 32·d bytes plus a term proportional to the number of heads, independent of history size, in at most 8 rounds for d ≤ 1,000.
  - Note: R2-E26 measured 768 B in 4 rounds (n = 10⁴, d = 10) and 32.5 KB in 8 rounds (d = 1,000).
- **AC-2** `necessary` `empirical` — Three-peer simulation with partitions, one hostile peer and protocol fuzzing: replicas converge, no invalid object or unsigned pointer is accepted, and nothing crashes (Phase 4 exit gate).
- **AC-3** `necessary` `formal` — Staging: no object is fetched for a pointer whose signature has not been verified.
  - Obligation: `unstated`.
  - Proxy: protocol tests with a hostile peer.
- **AC-4** `necessary` `formal` — No `secret` change is ever sent to a peer.
  - Obligation: `unstated`.
  - Proxy: property tests over push and bundle creation.
- **AC-5** `supporting` `empirical` — With 10,000 or more active signers, RBSR over the heads set uses fewer bytes than per-signer chains alone.
  - Note: R3-B measured 2,000 B against 13,600 B at d = 10.
- **AC-6** `supporting` `empirical` — End-to-end sync between the Windows host and the Debian host Nighthawk over Tailscale passes the same scenarios as the simulation.

## Rationale
- Radicle verifies identity and signed refs before downloading data, which
  defends against hostile peers (II §2.5); Scion fetches content by want from
  the heads (II §2.8).
- On realistic op DAGs, want/have negotiation needed about 32 bytes per missing
  op and 10–30× fewer bytes than RBSR (R2-E26).
- RBSR's cost grows with log(n/d) per difference and exceeds sending everything
  at large d/n (R1-E9); per-signer chains win up to about 1,000 signers, and a
  hybrid with RBSR over heads wins beyond that (R3-B).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.5, II §2.8, R1-E9, R2-E26, R3-B.

## Alternatives
- **RBSR / Negentropy as the protocol layer.** Removed (R3 §4.1).
- **Git's smart protocol.** Tied to Git's object model and SHA-1/SHA-256.

## Consequences
- Adversarial-peer behaviour and very long divergence still need measurement
  (R2 §4).
- Transport security and peer authentication depend on the actor keys of
  INT-0016.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 steps 30–34, R2-E26 and R3-B).
