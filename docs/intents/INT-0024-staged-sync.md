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

Three rules make this hold for thousands of replicas
([INT-0035](INT-0035-scale-and-asynchrony.md)):
- **A replica follows a part.** Every replica follows the public line and
  the Book. Beyond those it follows the lanes and intents it names.
  Negotiation and fetch cover what it follows and nothing else, so its costs
  follow that choice. Local operations never leave their replica
  ([INT-0009](INT-0009-reversible-by-default.md)).
- **Admission by actor.** Stage 1 also decides whose records a replica takes
  at all. A record signed by a key that is not registered in the history the
  replica follows, or by an actor past the quota policy gives it, is dropped
  before anything is fetched for it. One actor cannot flood the rest.
- **Peer to peer.** No replica talks to all the others. Each exchanges with a
  few, and records spread from peer to peer. Any replica may pass on what it
  holds, and passing on needs no trust, because everything passed on is
  signed and named by its hash.

Sync assumes only that a message sent is delivered at some time. It assumes
no clock, no order and no limit on delay.

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
- **AC-7** `necessary` `empirical` — Following a part costs a part: in a project with 1,000 active lanes, a replica that follows the public line and one lane receives and stores at most twice the bytes it would if those were the whole project.
  - Threshold: [INT-0035](INT-0035-scale-and-asynchrony.md) AC-10.
- **AC-8** `necessary` `empirical` — A flood is dropped at stage 1: a hostile peer that sends 10⁶ records signed by an unregistered key, or by an actor past its quota, causes no object to be fetched and no record to be stored.
  - Threshold: 0 fetched and 0 stored.
- **AC-9** `supporting` `empirical` — A thousand peers: 1,000 simulated replicas, each exchanging with at most 8 others, hold the same view of what they all follow once partitions heal, and the number of rounds it takes grows with the logarithm of the number of replicas.
  - Sample: 100, 300 and 1,000 replicas, 8 seeds each.
  - Threshold: 0 replicas that differ; rounds at most 4 · log₂ of the number of replicas.

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
- Following a part means a replica can lack history that an object it holds
  refers to. It records what it lacks
  ([INT-0009](INT-0009-reversible-by-default.md) AC-5) and fetches it when
  asked, which is the lazy fetch
  [INT-0030](INT-0030-optional-scale-mode.md) once listed as optional.
- Quotas are policy, and policy is a record like any other, so every replica
  that follows the Book applies the same quota.
- AC-2's three peers remain the exit gate for correctness. AC-9 is about
  shape at scale and needs the network simulation of
  [INT-0005](INT-0005-deterministic-simulation-testing.md) AC-8.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 steps 30–34, R2-E26 and R3-B).
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents, decentralised and asynchronous): sync was defined over the whole op log, so every replica would have carried every agent's working operations. The Intent adds three rules (a replica follows a part, admission by actor, peer to peer) and states what sync assumes of the network. AC-7 to AC-9 are added.
