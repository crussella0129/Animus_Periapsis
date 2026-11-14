# INT-0016 — Signed actors, op-log ordering and agent lifecycle

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0016
- **State:** active
- **Work evidence:** [Sprint 7 T-001 build plan](../sprints/s7/sprint-plans/build-plan.md#t-001-record-sprint-6s-last-runs-the-dependency-update-and-the-owners-two-sign-offs), [Sprint 7 T-003 build plan](../sprints/s7/sprint-plans/build-plan.md#t-003-the-canonical-merge-of-op-heads)
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

Ownership. A signature says who made an operation. Ownership says what that
actor may change. Each entry of the view has an owner:
- a workspace's working-copy pointer, and the changes made in it until they
  are published, belong to the actor the workspace was made for;
- the `public` line belongs to the verifier keys that policy names
  ([INT-0015](INT-0015-phases-and-confirmation.md));
- the Book's records belong to its `book` signers
  ([INT-0019](INT-0019-cheat-resistant-verifier.md)).

When operations are merged
([INT-0010](INT-0010-lock-free-concurrency.md)), an operation's effect on an
entry its actor does not own is not applied, and is reported. An agent,
faulty or hostile, can therefore change only its own work
([INT-0035](INT-0035-scale-and-asynchrony.md)).

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
- **AC-7** `necessary` `empirical` — An actor changes only what it owns: over histories in which faulty agents commit well-formed, validly signed operations that move, hide or rewrite entries they do not own, no entry changes on any replica except by an operation of its owner, and each refused effect is reported.
  - Threshold: 0 entries changed by a non-owner ([INT-0035](INT-0035-scale-and-asynchrony.md) AC-6).
- **AC-8** `supporting` `asserted` — The project owner approves the classes of owner and what each may change, before agents rely on them.
  - Signers: project owner.
  - Note: signed in part by the project owner (crussella0129), 2026-11-14, for one class of owner, the workspace: each workspace's entry in the view belongs to that workspace, every operation names the workspace that made it, and when op heads are merged an operation's effect on another workspace's entry is not applied. The other classes, the public line and the Book's records, are still to come, so this criterion is not met ([sign-offs of 2026-11-14](../sprints/s7/sign-offs.md)).

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
- Thousands of short-lived agents mean thousands of births, and an operator's
  key should not sign each one by hand. An operator may delegate birth
  signing to a key with a bounded mandate: how many agents, for which
  intents, and up to which operation. The delegation is itself a signed
  record, and a birth outside its bounds is invalid.
- "Registered at that point in the log" (AC-1) means registered in the
  operation's own ancestry. That is well defined on a log with many heads,
  and needs no clock.
- Ownership needs the view's entries to say whom they were made for, so the
  view's format carries it ([INT-0006](INT-0006-format-v1.md), backlog
  T-012).
- From Sprint 7 the first class of owner is built, and the owner approved
  it on 2026-11-14 (AC-8, for this class only): a workspace's entry, its
  working-copy pointer and its heads, is written only by operations that
  name that workspace, and a merge takes it only from them
  ([INT-0010](INT-0010-lock-free-concurrency.md)). The public line and the
  Book's records are classes still to come.
- Until operations are signed, the workspace an operation names is not
  proved: a process can name another. The rule therefore stops mistakes,
  not a hostile agent, and AC-7 waits for signatures. A refused effect is
  not reported yet either: finding one means computing the state every
  operation started from.
- The entry is keyed by the workspace's name; the actor the workspace was
  made for is not yet recorded (backlog T-012).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from the Biblio report §3–4 and Master Report v6 II §3.4 as slimmed by R3).
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents): signatures said who made an operation but nothing said what an actor may change. The Intent adds ownership of view entries, applied when operations are merged, so a faulty agent can change only its own work. AC-7 and AC-8 are added. Consequences record delegated birth signing, what "registered at that point" means on a log with many heads, and the format's part.
- 2026-11-14: `proposed` → `planned` (Sprint 7 build plan T-001 and T-003, approved by the owner on 2026-11-14: the workspace as the first class of owner, applied when op heads are merged; AC-8 for that class. AC-1 to AC-7 wait for signed operations). Consequences record what is built, what is not proved without signatures, and what is not yet reported.
- 2026-11-14: `planned` → `active` (Sprint 7 build, T-001 starts: the owner's approval of the workspace as a class of owner is recorded).
