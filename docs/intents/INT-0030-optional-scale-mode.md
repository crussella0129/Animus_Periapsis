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
Servers make Periapsis faster for very large projects and never become
necessary. An optional server mode offers:
- horizontally scaled caches of objects and of derived data;
- relays that pass records between replicas that cannot reach each other;
- server-side pushrebase (rebase pushes onto the target; fail only when the
  same file was edited).

What a single replica needs in order to handle a large project is core, and
not part of this mode ([INT-0035](INT-0035-scale-and-asynchrony.md)):
- the segmented graph index (graph shape in a few MB, ancestry in O(number
  of merges)) and per-file history (log and blame in O(changes to that
  file)): [INT-0013](INT-0013-derived-data.md),
  [INT-0023](INT-0023-query-language.md);
- lazy fetch of what a replica does not follow:
  [INT-0024](INT-0024-staged-sync.md);
- sparse and API-only workspaces:
  [INT-0014](INT-0014-working-copy-and-workspaces.md).

A server holds nothing a replica must trust. What it serves is signed and
named by its hash, and derived data from it is checked before use. Local use,
a small team and a fleet of thousands all work with no server.

## Acceptance criteria
- **AC-1** `necessary` `asserted` — Scale mode stays optional: every feature of a local repository works with no server.
  - Signers: human reviewer.
- **AC-2** `supporting` `empirical` — For a history of 10⁶ changes, the graph shape downloads in at most 10 MB and ancestry queries run in time proportional to the number of merges.
- **AC-3** `supporting` `empirical` — Server-side pushrebase rejects a push only when the same file was edited concurrently.
- **AC-4** `necessary` `empirical` — Optional means optional: with no server and every cache empty, the scale suite of [INT-0035](INT-0035-scale-and-asynchrony.md) passes, and with a server it gives the same views, operations and output.
  - Threshold: 0 differences ([INT-0035](INT-0035-scale-and-asynchrony.md) AC-8).

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
- Scale now shapes the formats from Phase 1
  ([INT-0035](INT-0035-scale-and-asynchrony.md)); this chapter first said it
  should not. What remains here is last in priority because it is optional by
  definition.
- AC-2 measures the segmented graph index, which is now core, as a server
  would deliver it. It stays here until
  [INT-0013](INT-0013-derived-data.md) plans that index.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 35 and II §2.2–2.3).
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents): the chapter made scale optional and last. The Intent now keeps only servers optional (caches, relays, pushrebase) and moves the graph index, per-file history, lazy fetch and sparse workspaces to the chapters that own them. AC-4 is added, and the Consequence that scale should not shape the formats is reversed.
