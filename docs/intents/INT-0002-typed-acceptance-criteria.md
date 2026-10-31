# INT-0002 — Typed acceptance criteria: every criterion carries polarity and modality

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0002
- **State:** active
- **Work evidence:** [T-006 build plan](../sprints/s0/sprint-plans/build-plan.md#t-006-typed-acceptance-validator-in-the-book-crate), [T-007 build plan](../sprints/s0/sprint-plans/build-plan.md#t-007-canonical-suite-runner-and-ci-workflow)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Every acceptance criterion in a Book is typed with a polarity and a modality
using one Markdown grammar that (a) git-based Sprint Loops Books can adopt
without breaking Book schema v2, (b) Periapsis imports losslessly into native
intent records (INT-0017, INT-0027) and (c) lets a machine compute each
intent's status, strength and statistical risk. This chapter is the normative
definition of that grammar ("typed acceptance v1").

Grammar. Inside `## Acceptance criteria`, each criterion is one top-level list
item, optionally followed by indented detail items:

```markdown
- **AC-<n>** `<polarity>` `<modality>` — <claim>
  - <Key>: <value>
```

- `<n>` is a positive integer unique within the chapter and never reused, so
  `INT-NNNN/AC-n` is a stable reference.
- `<polarity>` is `necessary` (a falsifier) or `supporting` (sufficiency
  evidence).
- `<modality>` is one of:
  - `formal` — a machine-checkable proposition. Detail `Obligation:` names the
    Lean 4 declaration or says `unstated`; `Proxy:` names the check used until
    the obligation is kernel-checked.
  - `empirical` — a measured rate or bound against a threshold fixed before
    measurement (`Threshold:`, `Sample:`, `Error:` δ where statistical).
  - `preference` — a blind A/B preference by a panel (`Panel:`, `Threshold:`,
    `Error:`).
  - `judged` — rubric scores by a quorum of non-author judges (`Rubric:`,
    `Quorum:`).
  - `asserted` — a sign-off by a named class of non-author principals
    (`Signers:`).
- Other detail keys: `Evidence:` (current verification link) and `Note:`.
- A claim may wrap onto indented continuation lines that do not begin with
  `- `.

Declaration: a Book opts into this contract with the line
`typed-acceptance: 1` in its `docs/.sprint-loop-book` marker. In a declared
Book every criterion must be typed; an undeclared Book is read with the port
rule below. The Book itself says which rule applies, so checking it needs no
flags. A `typed-acceptance` value other than `1` is refused rather than read
with the port rule, because it names a contract this version does not know.
A directory without a `.sprint-loop-book` marker is not a Book and is refused.

Semantics:
- **Certified** — every criterion holds with evidence. **Refuted** — some
  `necessary` criterion fails. **Undetermined** — neither. An intent with no
  criteria is Undetermined, never vacuously Certified, and in a declared Book a
  chapter without criteria is a grammar violation.
- **Strength** — the weakest modality among the criteria, ordered `formal` >
  `empirical` > `preference` > `judged` > `asserted`. A `formal` criterion
  supported only by its proxy counts as `empirical` until its obligation is
  kernel-checked (evidence tiers P/C/T/A of INT-0018).
- **Risk** — the sum of δ over statistical criteria: an exact union bound on
  the chance that some statistical claim passed although false.
- In this Book an intent moves to `realized` only when Certified, on top of
  the Sprint Loops evidence rules.

Port rule: an untyped criterion in an existing Book v2 chapter is read as
`necessary` `asserted` — the most conservative lift, claiming nothing beyond a
sign-off. Every Book v2 is therefore already a valid typed Book; typing a
criterion later is a revision recorded in Transition history.

Boundaries: the grammar types claims; it does not verify them. Kernel-checked
evidence for the types is INT-0018 (formal) and INT-0020 (non-formal). Non-goal:
a second lifecycle — chapters keep Sprint Loops' seven states.

## Acceptance criteria
- **AC-1** `necessary` `asserted` — The project owner approves this grammar and its semantics as the Book's typed-acceptance contract v1.
  - Signers: project owner.
- **AC-2** `necessary` `empirical` — Every intent chapter in this Book has at least one criterion, and a typed-acceptance validator reports 0 grammar violations over `docs/intents/`.
  - Threshold: 0 violations; checked on every sprint that edits intents.
- **AC-3** `necessary` `formal` — The status function is consistent: no intent is both Certified and Refuted.
  - Obligation: `unstated` (R5 proved the analogous `Certified L c I → ¬ Refuted L c I` for the intent calculus).
  - Proxy: unit tests of the validator's status computation.
- **AC-4** `necessary` `empirical` — Port rule holds in practice: the validator accepts every intent chapter of the Animus_Sprint_Loops Book unchanged and reads each untyped criterion as `necessary` `asserted`.
  - Threshold: 0 rejections over that corpus.
- **AC-5** `supporting` `asserted` — Animus_Sprint_Loops adopts the grammar as an optional, backward-compatible part of its intent schema, validated by `check-book.sh` when present.
  - Signers: the Animus_Sprint_Loops maintainer.

## Rationale
- Qualitative intents need types so their evidence cannot be cherry-picked;
  R5 Part B defines five modalities, the falsifier/sufficiency polarity,
  weakest-link strength and union-bound risk, and proves the consistency
  theorem in the Lean kernel.
- Sprint Loops intents are mostly prose. R4 showed opaque prose propositions
  still compose and report exact dependencies, so typing can be added on top of
  prose rather than replacing it.
- Agents game specifications: ImpossibleBench measured a 76% test-exploitation
  rate in one setting (R5 §A.1). Typed criteria plus a human-signed Book make
  the "weaken the spec" path explicit and reviewable.
- Defaulting untyped criteria to `necessary` `asserted` gives git-based
  projects a lossless path into Periapsis with no rewriting on day one.
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  R4 §2.4–2.5, R5 Part B.

## Alternatives
- **A separate YAML/TOML spec file per intent.** Rejected: it splits semantic
  authority away from the chapter Sprint Loops treats as authoritative.
- **Untyped prose plus R4 `Status` data** (`tested ref`, `asserted actor`).
  Superseded by R5: an agent could cite any test run or sign-off it liked.
- **Lean-only intents.** Rejected: excludes qualitative intents and teams
  without Lean.

## Consequences
- Criteria must be phrased as checkable claims with thresholds fixed before
  measurement, which makes chapters longer and slower to write.
- A validator has to exist (Rust, in this repository) and later ship to Sprint
  Loops as well; until then AC-2 is checked by review.
- Changing a criterion's polarity or modality, or removing it, is a semantic
  revision with a Transition history entry.
- The validator sees one version of a chapter, so it can reject a duplicate
  `AC-n` but not a number reused after a criterion was removed. Until
  INT-0017's event records make history mechanical, "never reused" is enforced
  by review and by the Transition history entry that every removal requires.
- Strength needs evidence tiers (INT-0018, INT-0020) to tell a kernel-checked
  `formal` criterion from one held only by its proxy, so the Sprint 0
  validator reports grammar and counts, not strength.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; grammar derived from Master Report v6 R5 Part B and R4).
- 2026-10-31: `proposed` → `planned` (Sprint 0 build plan T-006, T-007). Clarified before planning: claims may wrap onto continuation lines, and a Book declares the contract with `typed-acceptance: 1` in its marker instead of the checker taking a mode flag (owner direction: no flags on idempotent commands). After the first plan critique: an empty criteria list is Undetermined, and a declared Book with a criterion-less chapter is in violation. After the second plan critique: how "never reused" is enforced, and why strength is not computed yet, recorded in Consequences. After the third: the Declaration paragraph refuses unknown `typed-acceptance` values and marker-less directories.
- 2026-10-31: `planned` → `active` (Sprint 0 build, T-006 starts: this Book declares `typed-acceptance: 1` and the `book` validator checks it).
