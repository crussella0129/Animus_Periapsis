# INT-0032 — Intent revisions: work made for an earlier revision is flagged for review

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0032
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Intents and code keep separate histories, and every piece of work records the
revision of each intent it was made for. When an intent changes in meaning,
everything made for an earlier revision is flagged for review, so a changed
intent cannot leave code silently out of step with it.

- **Separate histories.** Each intent has its own revision history, made of its
  event records ([INT-0017](INT-0017-event-records-and-book-view.md)).
  Revising an intent never rewrites code history, and changing code never
  rewrites an intent. Either history can be read, compared and reverted on its
  own.
- **Revisions.** Every revision of an intent names the revision it replaces, so
  an intent's revisions form a lineage even when two peers revise it at once (a
  divergence INT-0017 surfaces). A revision has an ID, an author, a wall-clock
  time for people to read, and a kind: `semantic` when it changes what is
  wanted (the Intent text, the boundaries or any acceptance criterion), or
  `editorial` when it changes wording only. A revision that changes any
  criterion's claim, polarity, modality or details is always `semantic`;
  calling a revision `editorial` is a signed claim that reviewers can see and
  dispute.
- **Links.** Work that serves an intent records the revision its author worked
  from: changes (through their change IDs,
  [INT-0011](INT-0011-change-identity-and-rewriting.md), so links survive
  rewriting), review requests
  ([INT-0021](INT-0021-lanes-and-review-requests.md)), tasks, tests, documents
  and evidence records. A change can also link a region of a file to an
  intent: the whole file, or a byte range in it. A region link is data kept
  with the change, not text in the file, so it works on any content
  ([INT-0034](INT-0034-any-content.md)), including binaries and languages no
  layer reads. Periapsis carries each region through later changes: a
  whole-file region follows the file through renames, and a byte range
  follows a byte-level comparison that needs no knowledge of the format, or
  widens to the whole file where it cannot be carried. A region keeps the
  revision recorded by the change that linked it, or by the last later change
  that touched it while claiming its intent; an edit that does not claim the
  intent, such as a reformat, leaves its revision alone. A change that serves
  several intents links to each of them, and every rule below applies to
  those links unchanged.
- **Comment tags.** A tag in a comment, such as `// intent: INT-0006`, is an
  optional layer for kinds of content whose comments a layer can read: the
  layer finds each tag, bounds its region and decides which changes touch it,
  and the tag never needs editing. Content no layer reads, such as a binary or
  a language no layer knows, has no tags, even where its bytes spell one.
- **Review flags.** A link is *suspect* when a `semantic` revision follows its
  revision in the intent's lineage, or when its revision is not in that lineage
  at all because it was made on a divergent revision. Suspicion is computed
  from the links and the revision history, never stored, so a merge or a lost
  flag cannot clear it. A suspect link clears only through a recorded review
  that re-affirms the work against the current revision (who, when, which
  revision) or a change that re-links it.
- **Order, not clocks.** "Earlier" means earlier in the lineage. Timestamps are
  shown, never compared: peers' clocks drift, and a rebased or cherry-picked
  change keeps its old date.
- **The review list.** One query ([INT-0023](INT-0023-query-language.md))
  answers "what needs review because INT-NNNN changed": every suspect item, the
  revision it was made for, and the intent's diff from that revision to the
  current one.
- **Status.** An intent's status ([INT-0020](INT-0020-intent-calculus.md)) is
  computed against its current revision: evidence recorded for an earlier
  semantic revision does not certify the current one.

Boundaries: the flags say what to look at, not whether it still fits. Whether
flagged work is still right is a review decision, recorded like any other.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — After any sequence of intent revisions, links and re-affirmations, the review list holds exactly the suspect links: nothing missing and nothing extra.
  - Sample: a seeded corpus of 50 intents, 2,000 links of every kind and 500 revisions and re-affirmations, with binary files and files no layer reads among the linked work, some containing bytes that spell a tag, run under deterministic simulation ([INT-0005](INT-0005-deterministic-simulation-testing.md)).
  - Threshold: 100% recall and 100% precision.
- **AC-2** `necessary` `empirical` — Timestamps decide nothing: with peers' clocks skewed by up to ±30 days and change dates set arbitrarily, every review list equals the one computed with true clocks.
  - Threshold: identical lists in every trial.
- **AC-3** `necessary` `empirical` — Rewriting keeps links: rebasing, amending, splitting, squashing or cherry-picking a linked change keeps its intent links and region links, whatever its files hold; and a change that touches a linked region, or a tagged region where a layer reads tags, without claiming its intent leaves the region's revision unchanged.
  - Threshold: 100% of trials over a rewrite corpus that includes binary files and files no layer reads.
- **AC-4** `necessary` `empirical` — A flag clears only through a recorded re-affirmation against the current revision or a re-linking change, and a revision that changes any acceptance criterion cannot be recorded as `editorial`.
  - Threshold: every attempted bypass is refused.
- **AC-5** `necessary` `empirical` — Evidence recorded for an earlier semantic revision never certifies the current revision.
  - Threshold: 0 certifications from such evidence.
- **AC-6** `supporting` `empirical` — A git-based Sprint Loops Book gets the same review list, with revisions read from each chapter's Git history and links read from commit trailers, region links included, and from comment tags, so a project can use the flags before it ports ([INT-0027](INT-0027-port-sprint-loops-projects.md)) and keeps its links when it does.
  - Threshold: the same list as the native one over the AC-1 corpus mirrored to Git.
- **AC-7** `supporting` `asserted` — The project owner approves the `semantic`/`editorial` split and the kinds of link.
  - Signers: project owner.

## Rationale
- Owner direction during the Sprint 0 Test phase: the Book and the code should
  move somewhat independently, a change to an intent should be trackable
  against the code made for it, and everything that needs review after an
  intent changes should be flagged.
- Requirements-management tools solve this with suspect links: IBM DOORS and
  Siemens Polarion mark every item traced to a requirement as suspect when the
  requirement changes, until someone reviews it. A version control system can
  do better, because its links can follow changes through rewriting.
- Without it, an intent revised after its code was written leaves a Book that
  says one thing and code that does another, and nothing signals it. Agents
  work from the Book, so they would build on the mismatch.
- Typed acceptance ([INT-0002](INT-0002-typed-acceptance-criteria.md)) makes
  part of the `semantic`/`editorial` split mechanical: a change to a typed
  criterion is detectable.
- Periapsis versions any content, and a binary or a language no tool knows has
  no comments to hold a tag. Region links kept as data work on every file;
  comment tags are a convenience on top of them.

## Alternatives
- **Order by wall-clock timestamp (the owner's first sketch).** Kept for
  display, rejected as the deciding key: clocks differ between peers, and a
  rewritten change keeps its old date, so a timestamp comparison could both
  miss drift and flag current work.
- **Flag on every revision, with no editorial kind.** Simpler, but typo fixes
  would flood the review list until people stop reading it.
- **Store a "needs review" flag on each item.** A stored flag can be lost or
  cleared by a merge; a computed one cannot.
- **Links on whole changes only.** Kept as the minimum, but region links are
  needed when one change serves several intents.
- **Comment tags as the only region links.** A
  tag needs a format with comments and a layer that reads them, so binaries,
  data and languages no layer knows could have no regions. Tags stay as a
  convenience where they work.

## Consequences
- Links carry an intent revision, and region links a path and an optional
  byte range, so the record formats of INT-0017 and INT-0021, and the Sprint
  Loops task and commit conventions, need those fields. Where a region lies
  now is derived data ([INT-0013](INT-0013-derived-data.md)), recomputed by
  carrying each region link through later changes.
- Until event records exist, the Book has to record each revision's kind in
  Transition history, so that the Git-based form (AC-6) can tell semantic
  revisions from editorial ones.
- A broad semantic revision can flag much of the codebase. That is the intended
  effect: the review it asks for is the cost of changing what is wanted.
- An agent should learn that its intent changed before it finishes, not at
  review. Every replica therefore follows the Book's records, whatever else
  it follows ([INT-0024](INT-0024-staged-sync.md)), and the review list is
  kept up to date as derived data ([INT-0013](INT-0013-derived-data.md)), not
  recomputed over the whole history for each question.

## Transition history
- 2026-11-01: created as `proposed` (Sprint 0 Loop; owner direction given during the Sprint 0 Test phase).
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for thousands of agents; `editorial`: no criterion changes): Consequences record that every replica follows the Book's records and that the review list is kept as derived data.
