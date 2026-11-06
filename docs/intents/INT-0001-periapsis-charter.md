# INT-0001 — Periapsis: an agent-human DVCS where every change carries why, who and proof

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0001
- **State:** proposed
- **Work evidence:** [Sprint 1 T-009 build plan](../sprints/s1/sprint-plans/build-plan.md#t-009-record-sprint-0s-github-actions-runs-and-owner-sign-offs)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Build Animus_Periapsis ("Periapsis", command `praps`): a snapshot-based,
content-addressed distributed version control system written from scratch in
Rust for many concurrent agents and the humans who direct them. Its core works
on any content as bytes and never needs to understand what it versions, whether
source code, natural-language text, binaries or a language no human reads
([INT-0034](INT-0034-any-content.md)). Every change answers three questions
inside the history itself: **why** (the Book intents it serves), **who** (a
human or named agent holding a signing key) and **is it proven** (a
verifier-signed confirmation plus a proof-pairing certificate). A confirmation
does not assume the content is code: it records the checks that repository
policy names for each kind of content (tests, a proof, another checker, or by
default a signed sign-off by a principal other than the author), so content
that no tool can run or read can still be confirmed and reach `public` under
AC-3.
Periapsis folds the Sprint Loops Project Book, CubiKan's intent-unit lifecycle
and array-test's confirmation DAG into the VCS, so project state is derived
from evidence rather than stored as mutable pointers.

Boundaries:
- From scratch: designs are adopted from Git, Jujutsu, Mercurial/Sapling,
  Mononoke, Radicle, Pijul, Xet and others with credit, but Periapsis depends
  on none of their code ([INT-0003](INT-0003-engineering-baseline.md)).
- Any content: a file's content may be any bytes of any size. Storage, object
  identity, history, operations and undo, working copies, sync and
  distribution, intents and their links, review requests, confirmations,
  signatures and queries never parse or interpret it. Line diff, diff3,
  structured merge, blame, test runners and rendering are optional layers
  chosen per kind of content: a layer changes only presentation and
  convenience, never how content is stored or identified or whether a core
  guarantee holds, and every command works on content that no enabled layer
  can read ([INT-0034](INT-0034-any-content.md)).
- This charter is realized only through its child intents (map below); each
  child owns its own boundaries and acceptance criteria.

Non-goals: a wrapper over git, jj-lib or libpijul (Git interop is a bridge,
INT-0025); a hosting service or forge; a blockchain; Nostr publication; a CRDT
record layer; a patch-theory core in format v1; an "entropy layer"
([INT-0028](INT-0028-entropy-layer.md)).

Intent map (dependency order, no timelines):
- Foundations — INT-0002 typed acceptance, INT-0003 engineering baseline,
  INT-0004 Windows + Linux, INT-0005 deterministic simulation testing,
  INT-0034 any content.
- Phase 1, reversible single-user core — INT-0006 format v1, INT-0007
  crash-safe storage, INT-0008 compact history, INT-0009 reversibility,
  INT-0013 derived data, INT-0014 working copy and workspaces.
- Phase 2, rewriting and conflicts — INT-0010 lock-free concurrency, INT-0011
  change identity, INT-0012 conflicts and merge, INT-0023 query language.
- Phase 3, agent-native layer — INT-0015 phases and confirmation, INT-0016
  actors and lifecycle, INT-0017 event records and the Book view, INT-0018
  proof-pairing certificates, INT-0019 cheat-resistant verifier, INT-0020
  intent calculus, INT-0021 lanes and review requests, INT-0022 interfaces,
  INT-0026 Sprint Loops on Periapsis, INT-0027 porting Sprint Loops projects.
- Phase 4, distribution — INT-0024 staged sync, INT-0025 Git bridge.
- Phase 5, scale and assurance — INT-0029 reproducible releases, INT-0030
  optional scale mode.
- Identity — INT-0031 visual identity.

## Acceptance criteria
- **AC-1** `necessary` `asserted` — The project owner signs off this charter's decomposition into the child intents of the intent map; adding, removing or re-scoping a child later is a recorded revision of this chapter.
  - Signers: project owner (crussella0129), not an agent.
- **AC-2** `necessary` `empirical` — Self-hosting: Sprint Loops completes one full Research → Plan → Build → Test → Loop cycle on Periapsis with at least three concurrent agents and no Git in the loop, on Windows and on Linux.
  - Evidence: the INT-0026 AC-1 run.
- **AC-3** `necessary` `formal` — Admission invariant: no change reaches the `public` phase without (i) at least one Book intent reference, (ii) a signature by a registered actor key and (iii) a confirmation signed by a verifier key that no agent principal holds.
  - Obligation: `unstated` (shares the phase-gate monotonicity obligation of INT-0015 AC-1).
  - Proxy: property tests over the promotion path.
- **AC-4** `necessary` `asserted` — Before Periapsis declares 1.0, every child intent is `realized`, or `deferred`/`abandoned` with a recorded reason.
- **AC-5** `necessary` `empirical` — Any content: with no content layer enabled, every `praps` command succeeds on a repository whose files mix source code, natural-language text, assembly, binaries and random bytes, and snapshotting that repository gives the same object ids with and without layers enabled.
  - Threshold: 100% of commands, over the conformance suite of [INT-0034](INT-0034-any-content.md).

## Rationale
- Each surveyed system holds part of the design and none has an agent-native
  layer: Jujutsu (lock-free op log, change IDs, conflicts as data), Mercurial
  evolve and Sapling (safe rewriting, phases), Mononoke (Bonsai changesets and
  derived data), Radicle (self-certifying repos, signed in-repo records,
  staged fetch), Git (Merkle snapshots). II §2.
- Git-based Sprint Loops hits structural limits: a shared Markdown ledger
  conflicts on 100% of concurrent appends while one immutable record per event
  conflicts on 0% (R3-D); checkpoints live in forge PR conventions; agents
  need worktrees; a rebase can stop half-way.
- With agents as the main writers the bottleneck is verification, not
  generation: human decision information per promotion is at least
  log₂(N_surv/a_surv) bits and inspection effort grows as 2^bits, so effort
  goes into verifier precision rather than agent count (R1-E6, corrected III §6).
- Name: the periapsis is an orbit's closest approach and its point of highest
  speed. Agents explore outward; Periapsis is where their work converges into
  confirmed history, the way a gravity assist happens at closest approach. II §1.
- Regulatory tailwind: EU AI Act Article 50 transparency duties apply from
  2026-08-02; whether committed code is in scope is unsettled, but signed
  per-change provenance is a credible answer. II §6.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  Parts I–II; [Biblio report](../sprints/s0/sprint-research/biblio-report.md);
  the root README's product statement.

## Alternatives
- **Build on jj-lib with its Git backend** (Biblio recommendation; Deep
  Research v1). Rejected by the owner's from-scratch decision (Deep Research
  v3): owning the format gives hash agility, native agent records and
  evidence-derived phases instead of non-standard Git headers that plain Git
  drops on rebase.
- **Build on libpijul.** Licence-compatible (GPL-2.0-or-later, R1 #20), but
  snapshots plus diff3 showed no silent-correctness gap in 5,325 real cases and
  a resolution store answers about 70% of repeat conflicts (R2-E19/21/28).
- **Keep Git and evolve the Sprint Loops shell helpers ("Biblio" as an app over
  Git).** Leaves ledger conflicts, forge-bound checkpoints and worktree
  bookkeeping in place.

## Consequences
- The scope is a full VCS. Mitigations: vertical slices, self-hosting after
  each phase, spec before code, deterministic simulation from the first storage
  code (I.6).
- Adoption without Git is hard, so the Git bridge ships before any public
  release (INT-0025).
- The plan runs in dependency order with no timelines: Phase 0 specification
  and scaffolding, then Phases 1–5 as in the intent map (I.5).
- The figures that child intents quote from the R1 to R5 prototypes and
  experiments are targets, not established results: that code is not in this
  repository. Each owning intent's sprint re-implements the measurement and
  records its own figure before a criterion that cites one can be certified.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 Part I and the root README).
- 2026-10-31: revised while `proposed`, after the Sprint 0 plan critique: research-report risk recorded in Consequences — quoted prototype figures are targets until re-measured.
- 2026-11-01: revised while `proposed`, on owner direction: the command is `praps`, short for periapsis, replacing `peri` ([INT-0003](INT-0003-engineering-baseline.md)).
- 2026-11-01: revised while `proposed`, on owner direction: Periapsis must work with almost any development system, from natural language to binaries and languages no human reads. The Intent states that the core never needs to understand content and that a confirmation records the checks policy names for each kind of content, so content no tool can run still reaches `public`; Boundaries gain an any-content rule; the intent map adds [INT-0034](INT-0034-any-content.md) under Foundations; AC-5 is added.
