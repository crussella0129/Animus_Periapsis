# The owner's sign-offs of 2026-11-10

Given after Sprint 6 closed and before its pull request, [crussella0129/Animus_Periapsis#8](https://github.com/crussella0129/Animus_Periapsis/pull/8), was merged. The project owner asked to go through the sign-offs still open "as choices". Each was put as a question with two or three answers, and the owner chose one. The questions and the answers chosen are given here word for word. Nothing below was inferred from the approval of a plan or from a merge.

The eight were those backlog T-035 carried: the criteria whose signer is the project owner, or a human reviewer, in chapters that are planned, active or realized, and INT-0001 AC-1.

## Signed

| Criterion | Question | Answer chosen | What the answer said |
|---|---|---|---|
| [INT-0035](../../intents/INT-0035-scale-and-asynchrony.md) AC-12 | "do you approve the scale model (design for 10,000 agents and a billion operations; first test at 1,000 agents and a million operations) and the seven cost rules?" | "Approve" | "Sign it as written. It becomes the bar the merge, storage and stat-cache sprints are tested against." |
| [INT-0002](../../intents/INT-0002-typed-acceptance-criteria.md) AC-1 | "do you approve the typed acceptance-criteria grammar (necessary/supporting; formal, empirical, preference, asserted) as the Book's contract v1?" | "Approve" | "Sign it. Every chapter has used this grammar since Sprint 0 and cargo book checks it on every run." |
| [INT-0001](../../intents/INT-0001-periapsis-charter.md) AC-1 | "do you sign off the charter's decomposition into its child intents, as the intent map now lists them (38 chapters, four added this sprint)?" | "Approve" | "Sign it. Adding, removing or re-scoping a child later is a recorded revision of the charter, so this does not freeze the map." |
| [INT-0006](../../intents/INT-0006-format-v1.md) AC-6 | "do you sign off the changes made so far to docs/spec/format-v1.md (the provisional changeset, view and operation kinds added since Sprint 0)?" | "Approve as provisional" | "Sign the spec changes to date, as the provisional kinds the spec says they are. Later changes need a sign-off of their own." |
| [INT-0004](../../intents/INT-0004-windows-and-linux.md) AC-5 | "do you accept the Windows durability strategy and its crash-test evidence, with the assumptions listed in docs/spec/storage-durability.md (no documented guarantee for directory flushes on NTFS; no test cuts power)?" | "Accept with assumptions" | "Sign it, accepting the listed assumptions as stated: the evidence is the simulator's crash matrices plus real-filesystem runs, not a power-cut test." |

- INT-0006 AC-6 asks for a sign-off of every change to the specification. It is met for the changes to date and stays open for later ones.
- INT-0004 AC-5 names a human reviewer as its signer. The owner signed as that reviewer.

## Signed in part

| Criterion | Question | Answer chosen | What the answer said |
|---|---|---|---|
| [INT-0037](../../intents/INT-0037-command-set.md) AC-8 | "do you approve the command vocabulary table and the list of refused words (fsck, rebase, checkout, stash, reflog, cherry-pick)?" | "Built commands only" | "Approve the eleven commands that exist and the refused words. The names not built yet (diff, sync, lanes, move, squash and the rest) stay proposals, and the criterion stays open until they are fixed." |

The eleven are `status`, `describe`, `new`, `log`, `undo`, `ops`, `restore`, `init`, `verify`, `help` and `version`. The criterion is not met.

## Left open

| Criterion | Question | Answer chosen | What the answer said |
|---|---|---|---|
| [INT-0003](../../intents/INT-0003-engineering-baseline.md) AC-5 | "do you confirm that docs/prior-art.md credits every adopted design and that no code was copied from another version control project?" | "Not yet" | "Record nothing until you have read docs/prior-art.md yourself. This one asks a person to have looked, and I cannot confirm it for you." |
| INT-0003 AC-7 | "package names reserved on crates.io before any crate is published. Nothing is published and I have not checked that the names are reserved. Leave it open?" | "Leave open" | "Record nothing. It falls due only when a crate is about to be published." |

## Still to come

The ten criteria the owner signs in chapters that are still `proposed` fall due when those chapters are planned: INT-0011 AC-5, INT-0016 AC-8, INT-0020 AC-6, INT-0025 AC-3, INT-0031 AC-1, INT-0032 AC-7, INT-0033 AC-5, INT-0034 AC-8, INT-0036 AC-7 and INT-0038 AC-7.
