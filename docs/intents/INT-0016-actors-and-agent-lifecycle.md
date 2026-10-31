# INT-0016 — Signed actors, op-log ordering and agent lifecycle

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0016
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Every human and agent is an actor with an Ed25519 key, and every operation is
signed by its actor. The op log orders events; wall-clock timestamps are
advisory and never trusted. Agent lifecycle is recorded as signed records:

- **birth** — the operator signs {agent key, configuration-manifest hash,
  mandate and intents, parent manifests}; the agent counter-signs to prove it
  holds the key;
- **sunset** — agent and operator co-sign {final state hash, outcome evidence};
  anything signed by that key that is not an ancestor of the sunset op is
  invalid by rule, whatever its timestamp says;
- **descent** — a new configuration manifest names its parent manifests and
  the evaluation records that justified selecting them, so lineage is a
  queryable graph.

Failed or revoked agents stay in the record (negative lineage). Evaluation keys
are separate from agent keys: no agent can sign its own evaluation. Agent Trace
line attribution and in-toto/DSSE statements are export formats only.

Non-goal: proving that a key was destroyed. "Verified sunset" means "nothing
signed later counts".

## Acceptance criteria
- **AC-1** `necessary` `formal` — Every op in the log carries a valid signature by an actor key that is registered at that point in the log.
  - Obligation: `unstated`.
  - Proxy: verification tests, including tampered and unregistered-key cases.
- **AC-2** `necessary` `formal` — Sunset rule: a record signed by a sunset key that is not an ancestor of the sunset op is rejected, whatever its timestamp claims.
  - Obligation: `unstated`.
  - Proxy: backdating tests.
- **AC-3** `necessary` `empirical` — An evaluation or confirmation signed by the evaluated agent's own key is rejected.
  - Threshold: 0 accepted.
- **AC-4** `necessary` `empirical` — Records of revoked or failed agents are retained and queryable; only a signed erasure op (INT-0009) can remove them.
- **AC-5** `supporting` `empirical` — Exporters emit Agent Trace attribution and in-toto/DSSE confirmation statements that validate against their published schemas.
- **AC-6** `supporting` `asserted` — User documentation states what "verified sunset" does and does not prove.
  - Signers: human reviewer.

## Rationale
- Nostr cannot carry this identity model: NIP-26 delegation is deprecated,
  `created_at` can be backdated, NIP-41 key migration is best-effort and
  unmerged, and NIP-03 timestamps are flagged as vulnerable; ordering must come
  from a hash-chained log (Biblio §3).
- SLSA's trust model: the signer must be trusted, and Build L3 keeps signing
  secrets away from user-defined steps — for agents, the agent must not hold
  the key that signs its own evaluation (Biblio §4).
- Selecting descendants by performance invites Goodhart effects, so scores
  must come from independent verifiers, and negative lineage keeps known-bad
  traits visible (Biblio §4).
- Radicle uses Ed25519 node identities, and repositories are self-certifying
  (II §2.5); the R3 audit made attestations signed op records with DSSE/in-toto
  as export only (R3 §4.2).
- EU AI Act Article 50 makes machine-readable provenance valuable (II §6).
- Sources: [Biblio report](../sprints/s0/sprint-research/biblio-report.md)
  §3–4; [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.5, II §3.4, R3 §4.1–4.2.

## Alternatives
- **Nostr identity and publication.** Removed: no requirement or benefit once
  the op log orders events (R3 §4.1).
- **BIP-340/secp256k1 keys for Nostr compatibility.** Dropped with Nostr.
- **W3C DIDs and Verifiable Credentials, Sigstore keyless signing, hardware/TEE
  attestation.** Not researched yet (Biblio gaps); candidates for a later
  revision.

## Consequences
- Key registration, rotation and revocation need their own record types and a
  genesis rule for the first principal.
- Ed25519 must come from a pure-Rust implementation (INT-0003 AC-6).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from the Biblio report §3–4 and Master Report v6 II §3.4 as slimmed by R3).
