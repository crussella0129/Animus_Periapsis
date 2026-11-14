# The owner's sign-offs of 2026-11-14

Given in Sprint 7's Plan, before the plan was approved, as answers to separate questions. Each question offered two or three answers, and the owner chose one. The questions and the answers chosen are given here word for word. Approving the plan is not a sign-off, and nothing below was inferred from it.

## Signed in part

| Criterion | Question | Answer chosen | What the answer said |
|---|---|---|---|
| [INT-0016](../../intents/INT-0016-actors-and-agent-lifecycle.md) AC-8 | "Ownership (INT-0016 AC-8): each workspace's entry in the view — its working-copy pointer and its changes until published — belongs to that workspace, every operation names the workspace that made it, and when op heads are merged an operation's effect on any other workspace's entry is not applied. Approve this as the first class of owner?" | "Approve (Recommended)" | "The merge is built on it, and your answer is recorded as a sign-off of INT-0016 AC-8 for this class only. The public line and the Book's records come later. Until operations are signed, a process can still claim another workspace's name." |
| [INT-0037](../../intents/INT-0037-command-set.md) AC-8 | "Names for the workspace commands (INT-0037 lists lanes / lane NAME as proposals). Which should this sprint build?" | "praps lane DIR, praps lanes (Recommended)" | "`praps lane ../agent-17` makes a workspace at that directory, named after its last directory (agent-17), sharing this repository's store; `praps lanes` lists the workspaces and the change each is on." |

- INT-0016 AC-8 asks the owner to approve the classes of owner and what each may change. One class is approved: the workspace. The public line and the Book's records are classes still to come, so the criterion is not met.
- INT-0037 AC-8 was signed on 2026-11-10 for the eleven commands then built ([Sprint 6 sign-offs](../s6/sign-offs.md)). This answer adds `lane` and `lanes`. The names in the table that are not built stay proposals, so the criterion is not met.

## Decisions that are not sign-offs

The same round of questions settled two choices that no criterion asks the owner to sign:
- **Two commands at once in one workspace.** The answer was "Continue from one, keep both". The next command continues from one change and keeps the other ([INT-0014](../../intents/INT-0014-working-copy-and-workspaces.md) Consequences).
- **GitHub during Test.** The answer was "Push, PR, and two re-runs": push `dev`, open the pull request, and re-run every job of the tested head twice, for INT-0003 AC-11's three samples.

## Still to come in this sprint

INT-0006 AC-6 asks for a sign-off of each change to the format specification. The owner is asked once T-002's text is written, and the answer is added here.
