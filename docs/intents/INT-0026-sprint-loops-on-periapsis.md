# INT-0026 — Sprint Loops runs natively on Periapsis

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0026
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
The Sprint Loops workflow — Research → Plan → Build → Test → Loop, with the
current phase derived from Book evidence — runs on Periapsis without Git. Book
records are event records ([INT-0017](INT-0017-event-records-and-book-view.md));
the sprint's `work → base` checkpoint is an RR batch promotion
([INT-0021](INT-0021-lanes-and-review-requests.md)); signed confirmations
replace "CI conclusion as authority"; concurrent agents append work without
ledger conflicts; and exploration is bounded by a budget on the number of live
divergent heads per intent.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — A full Sprint Loops cycle completes on Periapsis with at least three concurrent agents and no Git, on Windows and on Linux (Phase 3 exit gate).
- **AC-2** `necessary` `empirical` — Concurrent Book record updates during that cycle produce no spurious conflicts.
  - Threshold: 0.
- **AC-3** `necessary` `empirical` — The sprint closes on green confirmations, with the number of live divergent heads for each of its intents at or below the configured budget.
  - Note: the budget is on the head count (Hartley, log₂ of the count); Shannon entropy of head activity is reported but never gates, because it can be gamed (R1-E13).
- **AC-4** `necessary` `formal` — The current sprint phase is a pure function of Book evidence and routes exactly as the git-based Sprint Loops router does on the projected Book.
  - Obligation: `unstated`.
  - Proxy: differential tests against the router on projected Books.

## Rationale
- Sprint Loops already treats phase as derived from evidence — a macrostate
  function of repository state, not extra state (III §1.3).
- In the git-based version, shared Markdown ledgers conflict on every
  concurrent append (R3-D), and the checkpoint depends on forge pull requests.
- A Shannon budget on head activity can be gamed by one dominant head plus
  many trickle heads (1,000 trickle heads still scored 0.18 bits); a head-count
  budget cannot (R1-E13).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  I.5 step 29, III §1.3, R1-E13, R3-D.

## Alternatives
- **Keep running Sprint Loops on Git and mirror into Periapsis.** Useful during
  migration (INT-0027) but never exercises the agent-native layer.

## Consequences
- The Sprint Loops helpers need a Periapsis backend in place of their Git and
  forge calls.
- This intent is the self-hosting milestone of the charter
  ([INT-0001](INT-0001-periapsis-charter.md) AC-2).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 29 and II §4 Phase 3).
