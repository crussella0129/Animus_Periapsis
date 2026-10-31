# INT-0030 — Optional scale mode for very large repositories

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0030
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
For very large repositories Periapsis offers an optional server mode:
horizontally scaled caches, virtual or sparse checkouts, server-side
pushrebase (rebase pushes onto the target; fail only when the same file was
edited), a segmented graph index (graph shape in a few MB, ancestry in
O(number of merges)), per-file history (log and blame in O(changes to that
file)), and lazy metadata fetch. Local and small-team use never needs a server.

## Acceptance criteria
- **AC-1** `necessary` `asserted` — Scale mode stays optional: every feature of a local repository works with no server.
  - Signers: human reviewer.
- **AC-2** `supporting` `empirical` — For a history of 10⁶ changes, the graph shape downloads in at most 10 MB and ancestry queries run in time proportional to the number of merges.
- **AC-3** `supporting` `empirical` — Server-side pushrebase rejects a push only when the same file was edited concurrently.

## Rationale
- Sapling and Mononoke show the techniques at scale: segmented changelog,
  per-file history with explicit renames, pushrebase and derived indexes
  (II §2.2–2.3).
- Scion's reported results — worktree creation from 7.1 minutes to 5.1 seconds
  — show the payoff for large monorepos (II §2.3; first-party figures).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.2–2.3, I.5 step 35.

## Alternatives
- **Server-first architecture.** Rejected: Periapsis is distributed first;
  servers are caches and accelerators.

## Consequences
- Lowest priority of the charter's children; it should not shape the Phase 1–4
  formats beyond keeping graph and metadata separable.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 35 and II §2.2–2.3).
