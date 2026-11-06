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
each orbit's `work → base` checkpoint is an RR batch promotion
([INT-0021](INT-0021-lanes-and-review-requests.md)); signed confirmations
replace "CI conclusion as authority"; concurrent agents append work without
ledger conflicts; and exploration is bounded by a budget on the number of live
divergent heads per intent. Confirmations
([INT-0015](INT-0015-phases-and-confirmation.md)) do not need a test runner:
work that no test runner can check, such as prose, a binary or a language no
one reads, is confirmed by the checks that repository policy names for it, by
default a non-author sign-off, and an orbit over such work closes the same
way.

Vocabulary: on Periapsis one pass of the loop is an **orbit**. Orbit N is
Sprint Loops' sprint N, with the same phases, records and evidence; tasks and
phases keep their names. An orbit's checkpoint, where the work comes closest
to `main` and a person decides, is its periapsis. The Book projected for Git
keeps Sprint Loops' `docs/sprints/sN/` paths and wording, so Sprint Loops'
own tools keep working on it
([INT-0027](INT-0027-port-sprint-loops-projects.md)).

## Acceptance criteria
- **AC-1** `necessary` `empirical` — A full orbit, every phase of the Sprint Loops cycle, completes on Periapsis with at least three concurrent agents and no Git, on Windows and on Linux (Phase 3 exit gate).
- **AC-2** `necessary` `empirical` — Concurrent Book record updates during that orbit produce no spurious conflicts.
  - Threshold: 0.
- **AC-3** `necessary` `empirical` — The orbit closes on green confirmations, with the number of live divergent heads for each of its intents at or below the configured budget.
  - Note: the budget is on the head count (Hartley, log₂ of the count); Shannon entropy of head activity is reported but never gates, because it can be gamed (R1-E13).
- **AC-4** `necessary` `formal` — The current phase of an orbit is a pure function of Book evidence and routes exactly as the git-based Sprint Loops router does on the projected Book.
  - Obligation: `unstated`.
  - Proxy: differential tests against the router on projected Books.
- **AC-5** `necessary` `empirical` — Orbit N and sprint N are the same record: projecting orbit N for Git gives `docs/sprints/sN/`, and importing `docs/sprints/sN/` gives orbit N, with no field lost either way.
  - Threshold: 0 differences over the INT-0027 import corpus.
- **AC-6** `supporting` `empirical` — A full orbit over a project written only in natural language, with no test runner installed, completes and closes on its confirmations.

## Rationale
- Sprint Loops already treats phase as derived from evidence — a macrostate
  function of repository state, not extra state (III §1.3).
- In the git-based version, shared Markdown ledgers conflict on every
  concurrent append (R3-D), and the checkpoint depends on forge pull requests.
- A Shannon budget on head activity can be gamed by one dominant head plus
  many trickle heads (1,000 trickle heads still scored 0.18 bits); a head-count
  budget cannot (R1-E13).
- The owner asked for an iteration name that fits Periapsis and is simple and
  versatile (2026-10-31). An orbit has exactly one periapsis, as each pass
  has exactly one checkpoint, so the name explains itself.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  I.5 step 29, III §1.3, R1-E13, R3-D.

## Alternatives
- **Keep running Sprint Loops on Git and mirror into Periapsis.** Useful during
  migration (INT-0027) but never exercises the agent-native layer.
- **Other words for one pass.** *Sprint* is Sprint Loops' word and stays in
  the Git projection. *Cycle* is plain, but in a version control system it
  already names a loop in the history graph, which must never exist.
  *Rotation* is a spin about an axis, not a trip around a body, and names key
  and log rotation elsewhere. *Revolution* is right but long, and its short
  form `rev` would collide with intent revisions
  ([INT-0032](INT-0032-intent-revisions-and-review-flags.md)). Rotations
  inside a revolution would add a word for what tasks already are.

## Consequences
- The Sprint Loops helpers need a Periapsis backend in place of their Git and
  forge calls.
- This intent is the self-hosting milestone of the charter
  ([INT-0001](INT-0001-periapsis-charter.md) AC-2).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 29 and II §4 Phase 3).
- 2026-11-01: revised while `proposed`, on owner direction: one pass of the loop is an orbit, and orbit N is Sprint Loops' sprint N. The Intent, AC-1 to AC-4, Rationale and Alternatives are reworded, and AC-5 is added for the one-to-one mapping.
- 2026-11-01: revised while `proposed`, on owner direction: orbits must close on work that no test runner can check. The Intent points to the confirmations of INT-0015, made by the checks policy names and by default a non-author sign-off, and AC-6 runs an orbit over a project written only in natural language.
