# INT-0033 — Intent topics and dependencies: the Book can be searched by subject

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0033
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Each intent says what it is about and which intents it builds on, in header
fields a validator checks, so a person or an agent finds the intents that bear
on a question by looking them up instead of searching the whole Book.

- **Topics.** The chapter header gains a `Topics` field, for example
  `- **Topics:** history, review`: one to three words from a topic list kept in
  one place in the Book, each topic with a one-line definition. A topic that is
  not on the list is a violation, so a typo cannot start a new topic. The
  validator prints the index: each topic with its intents.
- **Dependencies.** The header gains a `Depends on` field of links to the
  intents this one builds on, for example
  `- **Depends on:** [INT-0017](INT-0017-event-records-and-book-view.md)`. Each
  link must resolve to another chapter, and the validator answers the reverse
  question: which intents depend on this one. With
  [INT-0032](INT-0032-intent-revisions-and-review-flags.md), a semantic
  revision of an intent also flags the intents that depend on it.
- **One header.** Both are fields of the bullet header that Sprint Loops
  already validates, not YAML or TOML front matter. `check-book.sh` reads only
  the fields it requires, so the Book stays a valid Book v2 for git-based tools.
- **In Periapsis.** Topics and dependencies become fields of intent records
  ([INT-0017](INT-0017-event-records-and-book-view.md)), and queries
  ([INT-0023](INT-0023-query-language.md)) filter and traverse them.

Boundaries: topics help find intents and do not change what one means, so
changing a chapter's topics is an editorial revision in the sense of INT-0032.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Every intent chapter in this Book names one to three topics from the Book's topic list, and the validator reports a chapter with no topic, more than three, or a topic not on the list as a violation.
  - Threshold: 0 violations; checked on every sprint that edits intents.
- **AC-2** `necessary` `empirical` — Every dependency link resolves to another intent chapter of the Book, and for any intent the validator lists the intents that depend on it.
  - Threshold: 0 unresolved links and 0 links from a chapter to itself.
- **AC-3** `necessary` `empirical` — The fields keep git-based Sprint Loops working: the installed `check-book.sh` passes this Book with the fields, and Books without them still validate.
  - Threshold: 0 `check-book.sh` failures.
- **AC-4** `supporting` `empirical` — Topics find what a question needs: for questions written before any topic is assigned, the intents filed under each question's topics include the intents the owner names as relevant.
  - Sample: 20 owner questions, each with its relevant intents, fixed before topics are assigned.
  - Threshold: at least 90% of the named intents found.
- **AC-5** `supporting` `asserted` — The project owner approves the topic list.
  - Signers: project owner.

## Rationale
- Owner direction during the Sprint 0 Test phase: intents should carry a topic
  field, so that finding "which intents fit this question" needs no manual
  search, in YAML, TOML or a better form.
- The cost showed the first time it came up: placing
  [INT-0032](INT-0032-intent-revisions-and-review-flags.md) took a listing of
  all 31 chapters, a keyword search and reading four of them. A topic index
  would have narrowed that to a handful, and dependency links from INT-0017
  would have pointed at the rest.
- `docs/SUMMARY.md` already groups the intents, but Sprint Loops treats it as
  navigation, not semantic state, and it files each intent under one heading.

## Alternatives
- **YAML front matter (the owner's first sketch).** A second syntax beside the
  header Sprint Loops validates, with implicit typing: under YAML 1.1, which
  many parsers still follow, a bare `no`, `on` or `off` is a boolean, not a
  word.
- **TOML front matter (`+++`).** GitHub shows YAML front matter as a table but
  TOML front matter as raw text at the top of every chapter.
- **Full-text search only.** Finds known words, not a question phrased
  differently from the chapter.
- **Embedding search.** A possible query backend later, but its answers are
  opaque and change with the model; topics are explicit and reviewable.

## Consequences
- Every existing chapter needs topics and dependencies once; a new chapter sets
  them when it is written.
- The topic list needs an owner, and adding a topic is a Book edit like any
  other.
- Sprint Loops' intent schema gains two optional fields, proposed upstream with
  the typed-acceptance grammar
  ([INT-0027](INT-0027-port-sprint-loops-projects.md)).

## Transition history
- 2026-11-01: created as `proposed` (Sprint 0 Loop; owner direction given during the Sprint 0 Test phase).
