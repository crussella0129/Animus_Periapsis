# INT-0038 — The live view: one screen for directing many agents

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0038
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
A person directing thousands of agents sees the state of the project on one
screen and acts from it. `praps watch` shows three things, live:
- **what needs that person:** the decisions that are waiting, grouped by
  intent;
- **what the agents are doing:** lanes counted by state;
- **whether the project is healthy:** queue depth, verifier pass rate,
  divergence against its budget, and how far each followed peer is behind.

Rules:
- **Decisions first.** The top of the screen lists what only a person can do:
  - batches awaiting review
    ([INT-0021](INT-0021-lanes-and-review-requests.md));
  - divergences that policy sends to a person
    ([INT-0011](INT-0011-change-identity-and-rewriting.md));
  - work flagged because its intent changed
    ([INT-0032](INT-0032-intent-revisions-and-review-flags.md));
  - sign-offs that `asserted` criteria ask for.

  Each shows its evidence profile and the one command that decides it.
- **Counts, then detail.** A thousand lanes do not fit in a list. The screen
  shows counts by state, and a query
  ([INT-0023](INT-0023-query-language.md)) narrows them to a lane, a change
  or a file.
- **Calm.** The screen changes only when an answer changes. Nothing scrolls
  past, and no line is wider than the terminal
  ([INT-0022](INT-0022-agent-and-human-interfaces.md) AC-4).
- **Derived.** Everything shown is computed from the records
  ([INT-0013](INT-0013-derived-data.md)). The view stores nothing, so it can
  be closed and opened again anywhere and show the same thing.
- **Nothing only the screen can do.** Every action the view offers is a
  command in the registry
  ([INT-0022](INT-0022-agent-and-human-interfaces.md)), so an agent can do the
  same without the screen.
- **Any content.** What no layer reads is shown by type, size and hash
  ([INT-0034](INT-0034-any-content.md)).
- **In the terminal first.** The view is part of `praps`, so it works over
  SSH and needs nothing else installed. A graphical client is optional and is
  a separate program: it talks to `praps` through the interface agents use
  and is not part of the Periapsis workspace
  ([INT-0003](INT-0003-engineering-baseline.md)).

Boundaries: this intent owns what the screen shows and how it stays readable
at scale. The commands behind its actions belong to their own intents, and
the look follows [INT-0031](INT-0031-visual-identity.md). Non-goal: a hosted
dashboard or forge ([INT-0001](INT-0001-periapsis-charter.md)).

## Acceptance criteria
- **AC-1** `necessary` `formal` — The view is a function of the records: the same repository state and screen size give the same screen.
  - Obligation: `unstated`.
  - Proxy: render tests on fixture repositories, and the delete-and-recompute invariant of [INT-0013](INT-0013-derived-data.md) AC-1.
- **AC-2** `necessary` `empirical` — At scale the screen stays live: with 1,000 lanes and 10,000 open review requests in simulation, the first screen and every update are drawn within the interactive budget of [INT-0036](INT-0036-measured-speed.md) AC-5, and an update reads only what changed.
  - Sample: 100 updates.
  - Threshold: 100 of 100 within the budget.
- **AC-3** `necessary` `formal` — Every action the view offers is a registry command, and the view defines no action of its own.
  - Obligation: `unstated`.
  - Proxy: a test over the view's action table.
- **AC-4** `necessary` `empirical` — Nothing waits unseen: the decisions list holds exactly the items that the Book's "needs a person" queries select.
  - Threshold: 100% recall and 100% precision over a scenario corpus.
- **AC-5** `supporting` `judged` — Rubric, fixed before review: decisions are seen first; the state of the fleet is read at a glance; the screen is calm; it matches the visual identity. At least 2 of 3 designers score it 4 or more out of 5.
  - Quorum: 2 of 3 non-author designers.
- **AC-6** `supporting` `preference` — People directing a simulated fleet of 1,000 agents prefer the view to reading the output of `log` and `requests`.
  - Panel: 12 judges.
  - Threshold: preference > ½.
  - Error: δ = 1/20.
- **AC-7** `necessary` `asserted` — The project owner approves what the screen shows, and decides whether a graphical client is built.
  - Signers: project owner.

## Rationale
- Owner direction, 2026-11-09: thousands of agents, and "a far more intuitive
  command set and UI".
- A person cannot read the log of a thousand agents. With agents as the main
  writers, the scarce thing is the person's decision
  ([INT-0001](INT-0001-periapsis-charter.md) Rationale), so the screen's job
  is to spend attention on decisions and on nothing else.
- Gate 2 already asks for review "per batch, grouped by intent, showing
  evidence profiles" ([INT-0021](INT-0021-lanes-and-review-requests.md)), and
  the Sprint 0 plan listed operational dashboards for candidates, verifier
  pass rate and review time (I.5 step 27). No chapter owned either.
- Keeping every action a registry command means the view cannot become a
  second, untested way to change the repository.
- Sources: [Sprint 6 research](../sprints/s6/sprint-research/research-report.md);
  [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md) I.5.

## Alternatives
- **Lists only** (`lanes`, `requests`, `log`). Enough for ten agents. At a
  thousand, the person must run them over and over and count by eye.
- **A web dashboard on a server.** A hosted service is a charter non-goal,
  and a project with no server could not use it.
- **A graphical application first.** It would be the only part of Periapsis
  not written in Rust inside the workspace
  ([INT-0003](INT-0003-engineering-baseline.md)), and it would not work over
  SSH. It stays possible as a separate client.

## Consequences
- The view needs incremental derived data
  ([INT-0013](INT-0013-derived-data.md)) and the query language
  ([INT-0023](INT-0023-query-language.md)) before it can be built.
- Drawing in a terminal needs either a dependency or careful code of our own
  without `unsafe`; the choice is made under
  [INT-0003](INT-0003-engineering-baseline.md) AC-6 when this intent is
  planned.
- If the owner chooses to build a graphical client, it is its own project
  with its own stack, and it reaches Periapsis only through `praps`.

## Transition history
- 2026-11-09: created as `proposed` (Sprint 6 research; owner direction of 2026-11-09 for thousands of agents and a far more intuitive UI).
