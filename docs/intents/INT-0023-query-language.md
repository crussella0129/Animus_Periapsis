# INT-0023 — One query language over history, evidence and review requests

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0023
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
A single revset-style language selects changes, operations, actors, intents,
phases, conflict state and review requests. The CLI, the agent API and Gate 2
batch selection ([INT-0021](INT-0021-lanes-and-review-requests.md)) all use it.
Reference queries:

- `resolved_by(human) & conflicted()`
- `rr(dev) & ready() & intent("INT-log-*") & ~rollup(never)`
- `rr(dev) & module(merge) & tier(>=tested)`
- `rr(dev) & lane(agent-7) & since(op:<id>)`

## Acceptance criteria
- **AC-1** `necessary` `formal` — `&`, `|` and `~` obey the Boolean-algebra laws (commutativity, associativity, distributivity, De Morgan) over the universe of the query's kind.
  - Obligation: `unstated`.
  - Proxy: property tests on random queries and fixture repositories.
- **AC-2** `necessary` `empirical` — Every reference query in this chapter parses and returns the expected result on a fixture repository.
- **AC-3** `necessary` `empirical` — Evaluation is deterministic: the same repository state gives the same ordered result on Windows and Linux.
- **AC-4** `supporting` `empirical` — Ancestry queries on large histories run in time proportional to the number of merges, using a segmented graph index.

## Rationale
- A query language over changes, actors, intents and phases is part of the
  Phase 2 plan (I.5 step 18), and Gate 2 needs filterable batches — what bors
  rollup labels do by hand (R5 §C.2, §C.5).
- Sapling's segmented changelog answers ancestry in O(number of merges)
  (II §2.2).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  I.5 step 18, II §2.2, R5 §C.5.

## Alternatives
- **Ad hoc flags per command.** Not composable, and Gate 2 would need its own
  mini-language anyway.

## Consequences
- The language is a compatibility surface for scripts and agents, so its
  grammar is versioned with the CLI.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 I.5 step 18 and R5 §C.5).
