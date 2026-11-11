# INT-0011 — Stable change identity and safe rewriting

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0011
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Every change has a stable change ID — 128 random bits minted when the change
is created — that survives amend, rebase, squash, split and absorb, so reviews,
review requests and intent links follow the change rather than one snapshot of
it. Where no diff layer reads a file ([INT-0034](INT-0034-any-content.md)),
`split` puts the file's whole change on one side, and `absorb` leaves it in
place and reports it. Rewrite operations record their old → new mappings;
obsolescence
("evolution") markers are a derived index of those mappings, not stored
objects. Descendants are rebased automatically, and rebase never fails:
conflicts become conflict terms ([INT-0012](INT-0012-conflicts-and-merge.md)).
Divergence (two live successors of one change ID), orphans and
phase-divergence are detected from the derived markers and resolved by
policy rules — for example "a human-authored side wins over an agent's" or "the
verifier-confirmed side wins" — with prompts for humans and structured results
for agents. Every kind of divergence has a default rule that settles it, or
sets it aside with both sides kept, without a person. A person is asked only
where policy says so: at thousands of agents nobody can answer each one
([INT-0035](INT-0035-scale-and-asynchrony.md)).

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Markers derived from rewrite ops equal explicitly maintained markers on random rewrite histories with concurrent rewrites on two replicas.
  - Sample: 2,000 histories. Threshold: 2,000 of 2,000 (R3-E).
- **AC-2** `necessary` `empirical` — Concurrent rewrites of one change are reported as divergence in every case except when the successors are identical, which is not divergence.
  - Note: R3-E detected 1,999 of 2,000; the single miss had identical successors.
- **AC-3** `necessary` `formal` — Rebase is total: rebasing any change onto any destination yields a change, possibly with conflict terms, and never an error or a half-finished state.
  - Obligation: `unstated`.
  - Proxy: property tests.
- **AC-4** `necessary` `empirical` — `squash`, `split`, `absorb`, `abandon` and `restack` preserve change IDs as specified and leave no orphans over a scenario corpus that includes files no diff layer reads, which `split` and `absorb` handle as the Intent says.
- **AC-5** `supporting` `asserted` — The divergence policy rules are written down and approved by the owner before agents rely on them.
  - Signers: project owner.
- **AC-6** `necessary` `empirical` — No divergence waits on a person by default: under the default policy, every divergence in the scenario corpus is settled by a rule or set aside with both sides kept, the same way on every replica, and no command prompts.
  - Threshold: 100% of divergences, with 0 prompts.

## Rationale
- Jujutsu's stable change IDs and a working copy that is itself a change let
  rewrites happen freely without breaking references (II exec summary, §2.1).
- Mercurial evolve's obsolescence markers are a second-order history of
  rewrites that syncs without the obsolete changesets and handles concurrent
  rewrites; Sapling uses the same idea for `restack` and `absorb` at scale
  (II §2.2).
- Storing old → new mappings in rewrite ops makes separate marker objects
  redundant (R3-E); change IDs stay in changesets as a cache.
- jj's own divergence-resolution command is still experimental, so policy rules
  must be designed in, with actor awareness (II §2.1).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.1–2.2, R3-E, R3 §4.3.

## Alternatives
- **Separately stored and synced marker objects (Mercurial).** Derivable, so
  removed (R3-E).
- **Identity by content hash only (Git).** Reviews and links break on every
  rewrite or force-push.

## Consequences
- Change IDs are randomly generated, so generation goes through the
  deterministic RNG trait of INT-0005 to keep simulation reproducible.
- Divergence policy is product behaviour that agents depend on, so rule changes
  are intent revisions.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §2.1–2.2, I.5 step 15 and R3-E).
- 2026-11-01: revised while `proposed`, on owner direction: every command must work on content no layer can read. The Intent says `split` puts such a file's whole change on one side and `absorb` leaves it in place and reports it, and AC-4's corpus includes such files.
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents): the Intent says every kind of divergence has a default rule that needs no person, and AC-6 checks it. The command other systems call rebase is proposed as `move` in [INT-0037](INT-0037-command-set.md); this chapter's use of "rebase" for the operation is unchanged.
