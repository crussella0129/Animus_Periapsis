# INT-0010 — Lock-free concurrency: op log, views and one canonical op-head merge

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0010
- **State:** active
- **Work evidence:** [Sprint 7 T-002 build plan](../sprints/s7/sprint-plans/build-plan.md#t-002-an-operation-names-its-workspace-and-a-view-holds-one-entry-per-workspace), [Sprint 7 T-003 build plan](../sprints/s7/sprint-plans/build-plan.md#t-003-the-canonical-merge-of-op-heads), [Sprint 7 T-004 build plan](../sprints/s7/sprint-plans/build-plan.md#t-004-commands-work-from-merged-states), [Sprint 7 T-005 build plan](../sprints/s7/sprint-plans/build-plan.md#t-005-undo-and-restore-are-scoped-to-their-workspace), [Sprint 7 T-009 build plan](../sprints/s7/sprint-plans/build-plan.md#t-009-the-concurrency-scenario), [Sprint 7 T-010 build plan](../sprints/s7/sprint-plans/build-plan.md#t-010-lanes-at-once-through-the-binary-spec-readme-credits-and-backlog)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Any number of processes, agents and machines run Periapsis commands
concurrently without locks. Each command reads one consistent view and always
commits its operation, whose parent is the op it started from; no command
blocks on another writer or fails because of one. When a load finds several op
heads, it merges all of them in **one canonical n-way step** and stores the
result in a normal form, so replicas that see the same heads in any order,
with any duplication and in any grouping converge on the same view.

The merge goes entry by entry, and an entry is written only by operations of
its owner ([INT-0016](INT-0016-actors-and-agent-lifecycle.md)). For each
entry it takes its **writers**: the latest operations of the entry's owner
among the heads and their ancestors, those no other operation of that owner
follows. One writer gives its value. Several are merged against their lowest
common operation of that owner, or against a virtual base, the merge of
several, when that history criss-crosses. A pointer keeps the base's value
when no writer changed it, and otherwise becomes the set of the values the
writers that changed it hold: the support-set normal form. Copies of the
entry that other owners' operations carry are only what those operations saw,
and change nothing. Pointer
conflicts are first-class values in the view ("moved from A to B or C"); their
resolution policy lives in [INT-0011](INT-0011-change-identity-and-rewriting.md).

A merge is computed, never committed. The merged view is a function of the
set of heads, so loading writes no operation, however many heads it finds.
The operation a command then commits names every head it merged as a parent.
A reader therefore never adds a head, and a thousand readers leave the log as
they found it. The merged view may be kept as derived data
([INT-0013](INT-0013-derived-data.md)).

Three rules hold the merge to [INT-0035](INT-0035-scale-and-asynchrony.md):
- **Cost follows the difference.** Merging costs in proportion to how much
  the heads differ from their base, not to the size of the view or the
  length of the log.
- **Ownership.** An operation's effect is applied only to view entries its
  actor owns ([INT-0016](INT-0016-actors-and-agent-lifecycle.md)).
- **Scope.** The merge needs the shared operations a replica follows and its
  own local ones, never another replica's local operations
  ([INT-0009](INT-0009-reversible-by-default.md)).

## Acceptance criteria
- **AC-1** `necessary` `formal` — Convergence (INT-conv): merging a multiset of heads gives the same view for every permutation and duplication of those heads.
  - Obligation: `Periapsis.OpMerge.conv` (unstated in this repository; R4 proved the prototype's version, `v2_membership_determined`, for general N).
  - Proxy: property tests over random head multisets.
- **AC-2** `necessary` `formal` — Compatibility (INT-compat): with exactly two heads, the canonical merge equals the classic three-way merge.
  - Obligation: `Periapsis.OpMerge.compat` (unstated here; R4 gave a 69-line general proof for the prototype).
  - Proxy: differential tests against a reference three-way merge.
- **AC-3** `necessary` `formal` — Grouping independence: merging partial merges equals the flat merge under the support-set normal form.
  - Obligation: `unstated`.
  - Proxy: property tests (R1-E12: 0 of 20,000 failures with the normal form, 7,756 of 20,000 without).
- **AC-4** `necessary` `empirical` — Criss-cross correctness over at least 4,000 random criss-cross histories.
  - Threshold: 0 failures (R1-E12).
- **AC-5** `necessary` `empirical` — No lost work: a model checker or simulation with at least 8 writers on at least 2 replicas loses no committed op and no reachable change.
  - Threshold: 0 violations (Phase 2 exit gate).
- **AC-6** `necessary` `formal` — Adequacy: AC-1 together with AC-2 rejects every registered mutant merge (pairwise fold, "first head equals last head" fast path, always-base, and the others in the registry).
  - Obligation: `unstated` (R5 §A.7: the conjunction rejects 6/6 mutants; each half alone misses some).
- **AC-7** `necessary` `empirical` — Readers add nothing: processes that load a store holding several op heads and run only commands that change nothing leave the set of stored operations exactly as it was.
  - Sample: 2, 8 and 1,000 simulated readers.
  - Threshold: 0 operations written.
- **AC-8** `necessary` `empirical` — At scale: the check of AC-5 holds with 1,000 writers on 100 replicas. No committed op and no reachable change is lost, no command fails because of another writer, and every replica that holds the same operations computes the same view.
  - Threshold: 0 violations ([INT-0035](INT-0035-scale-and-asynchrony.md) AC-3 and AC-9).
- **AC-9** `necessary` `empirical` — Merge cost follows the difference: joining *h* heads that each differ from their base in at most *c* entries reads objects in proportion to *h* · *c*, whatever the size of the view.
  - Threshold: [INT-0035](INT-0035-scale-and-asynchrony.md) AC-5.

## Rationale
- Jujutsu's op log gives lock-free concurrency across processes and machines:
  each command commits against a snapshot and divergence is merged on the next
  load, so commands cannot fail to commit — the guarantee agents need (II §2.1).
- Model-checked pointer merges (R1-E12): pairwise three-way merging with a
  common base is order-independent; a pure signed-sum algebra fails
  criss-cross correctness (466/4,000); adding a "both sides equal" rule breaks
  order independence (1,472/5,000); the canonical n-way merge is
  order-independent and criss-cross-correct (0 failures) but needs the
  support-set normal form for grouping independence.
- R4 formalised exactly this: the old pairwise fold is provably
  order-dependent, the canonical merge is provably convergent, and a later
  agent "fast path" was rejected and localised to INT-conv.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §2.1, II §3.3, R1-E12, R4 §2, R5 §A.7.

## Alternatives
- **Pairwise folding of heads.** Order-dependent (R4 a2:
  `mergeV1 5 1 [2,4,4] ≠ mergeV1 5 1 [4,4,2]`).
- **Locks or a central coordinator.** Blocks agents and cannot span machines
  that are offline from each other.

## Consequences
- Divergence is a normal state that policy must resolve (INT-0011).
- Content conflict terms need their own algebra and normal form (INT-0012
  AC-3); this intent covers views and pointers.
- Once formal obligations exist they must be re-established whenever the merge
  changes (proof maintenance is a feature: it localises regressions).
- Until this intent's merge is built, commands that find several op heads stop
  and name them (Sprint 2,
  [INT-0014](INT-0014-working-copy-and-workspaces.md) Consequences). That
  contradicts this Intent's "no command fails because of another writer"; the
  gap closes when the canonical merge lands.
- That gap is the first thing in the way of several agents. Measured in
  Sprint 6: two `praps new` commands that overlapped left two op heads, every
  later command stopped, and no command could join them
  ([research](../sprints/s6/sprint-research/research-report.md)). The merge
  is therefore the next sprint's work (backlog T-029).
- A merge stored as an operation would never settle once operations are
  signed ([INT-0016](INT-0016-actors-and-agent-lifecycle.md)). Two readers
  that merged the same heads would sign two different merge operations, each
  a new head for the next reader to merge. Hence the rule that a merge is
  computed and only a command's own operation is written.
- Jujutsu, the source of this design, documents no number of writers it was
  built for, and warns that use from several machines can lose pointers. The
  criteria here are what that design has to meet at a thousand writers.
- Each entry is merged over its owner's operations only (Sprint 7 Plan).
  One base for every head, their lowest common ancestor, gives spurious
  conflicts as soon as one head was made by an agent that loaded early: its
  copy of another workspace's entry is older than that entry's later
  values, yet differs from the base. Taking an entry only from its owner's
  operations leaves those copies out, which is what the ownership rule
  asks.
- The known limit of that merge: a value set back to an earlier one reads
  as no change against an older base. Two cases show it, and Sprint 7's
  T-003 pins both with their values:
  - A head that carries a merged copy of an entry. Lane Y's operation is
    made over two of W's writers, so it carries W as {a1, a2}; W then
    replaces a1 with a3. For the heads W's last operation and Y, the merge
    gives {a2, a3}, and the classic three-way merge of the two views gives
    {a1, a2, a3} (Sprint 7 plan critique, round 1, C-001).
  - Writers of one entry whose ancestry nests, one of which sets the value
    back. The merge can keep a value the owner moved away from, or gain a
    spurious member. In workspace N: r sets 0; p sets 1 and q sets 0, both
    over r; L, an operation of another workspace over p and q, is the two
    heads' one latest common ancestor; n_a sets 0 and n_b keeps 1, both
    over p alone; the heads are two operations of that other workspace, one
    over L and n_a, one over L and n_b. N's writers are n_a, n_b and q,
    whose base is r, so n_a reads as unchanged: the merge gives {1}, and the
    classic merge over L gives {0}. With q at 2 instead, the merge gives
    {1, 2} and the classic merge {2}.

  Matching the classic merge in those cases would mean grouping the writers
  by the heads that reach them. An operation of another workspace over the
  same heads would then change this entry, which the ownership rule
  forbids. Every value stays in the operations that set it, and `restore`
  returns to any of them. AC-2 is therefore shown for two heads neither of
  which carries a merged copy. AC-3 is shown for writers that diverge from
  one base whose pointer holds one changeset, as R1-E12 tested it: with a
  base that is itself a conflict, two writers that resolve it both ways
  merge back to the base and read as unchanged.
- A working-copy pointer that two commands of one workspace moved at once is
  resolved by that workspace's next command, by the rule
  [INT-0014](INT-0014-working-copy-and-workspaces.md) records. No other
  workspace resolves it.
- In Sprint 7 replicas are two stores in one simulated filesystem that copy
  what each lacks (AC-5). Sync proper is
  [INT-0024](INT-0024-staged-sync.md)'s.
- Those replicas exchange every operation, because operations carry no scope
  yet ([INT-0009](INT-0009-reversible-by-default.md)). Once local operations
  stay on their replica, an entry whose owner's operations a replica lacks
  is absent there, by this merge's rule. A view shared between replicas then
  needs each owner's shared operations; the backlog carries this with the
  scope fields (backlog T-012).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §3.3, R1-E12, R4 and R5 §A.7).
- 2026-11-07: revised while `proposed` (Sprint 2 plan critique, round 1): Consequences record the interim several-heads stop and that it contradicts this Intent until the canonical merge lands.
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents): the Intent says a merge of op heads is computed and never committed as an operation, and holds the merge to three rules of [INT-0035](INT-0035-scale-and-asynchrony.md): cost, ownership and scope. AC-7 to AC-9 are added. Consequences record Sprint 6's measurement of the several-heads stop, why a stored merge would not settle, and what Jujutsu's documentation leaves open.
- 2026-11-14: `proposed` → `planned` (Sprint 7 build plan T-002 to T-005, T-009 and T-010, approved by the owner on 2026-11-14: the canonical merge, commands over merged states, scoped history and the Concurrency scenario; AC-1 to AC-7, with AC-3 for writers diverging from one base and AC-5 over simulated replicas. AC-8 and AC-9 wait for cost and scale). Revised in the same Plan (`semantic`): the Intent states that each entry is merged over its owner's operations, which the ownership rule already implied; Consequences record why, the merge's known limit, and who resolves a working-copy pointer.
- 2026-11-14: revised while `planned` (Sprint 7 plan critique, round 1, C-001 and C-015; `editorial`: no criterion changes): Consequences give the known limit's two cases with their values, the scopes in which AC-2 and AC-3 are shown, and why the merge does not group writers by heads; they record that Sprint 7's simulated replicas exchange every operation.
- 2026-11-14: revised while `planned` (Sprint 7 plan critique, round 2, C-007; `editorial`: no criterion changes): the known limit's nested case is given with its history and both pairs of values.
- 2026-11-14: `planned` → `active` (Sprint 7 build, T-003 starts: the canonical merge of op heads).
