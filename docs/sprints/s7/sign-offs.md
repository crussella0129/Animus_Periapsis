# The owner's sign-offs of 2026-11-14

Given in Sprint 7 as answers to separate questions: two in Plan, before the plan was approved, and one during Test. Each question offered two or three answers, and the owner chose one. The questions and the answers chosen are given here word for word. Approving the plan is not a sign-off, and neither is merging a pull request; nothing below was inferred from either.

## Signed in part

| Criterion | Question | Answer chosen | What the answer said |
|---|---|---|---|
| [INT-0016](../../intents/INT-0016-actors-and-agent-lifecycle.md) AC-8 | "Ownership (INT-0016 AC-8): each workspace's entry in the view — its working-copy pointer and its changes until published — belongs to that workspace, every operation names the workspace that made it, and when op heads are merged an operation's effect on any other workspace's entry is not applied. Approve this as the first class of owner?" | "Approve (Recommended)" | "The merge is built on it, and your answer is recorded as a sign-off of INT-0016 AC-8 for this class only. The public line and the Book's records come later. Until operations are signed, a process can still claim another workspace's name." |
| [INT-0037](../../intents/INT-0037-command-set.md) AC-8 | "Names for the workspace commands (INT-0037 lists lanes / lane NAME as proposals). Which should this sprint build?" | "praps lane DIR, praps lanes (Recommended)" | "`praps lane ../agent-17` makes a workspace at that directory, named after its last directory (agent-17), sharing this repository's store; `praps lanes` lists the workspaces and the change each is on." |

- INT-0016 AC-8 asks the owner to approve the classes of owner and what each may change. One class is approved: the workspace. The public line and the Book's records are classes still to come, so the criterion is not met.
- INT-0037 AC-8 was signed on 2026-11-10 for the eleven commands then built ([Sprint 6 sign-offs](../s6/sign-offs.md)). This answer adds `lane` and `lanes`. The names in the table that are not built stay proposals, so the criterion is not met.

## Signed during Test

| Criterion | Question | Answer chosen | What the answer said |
|---|---|---|---|
| [INT-0006](../../intents/INT-0006-format-v1.md) AC-6 | "INT-0006 AC-6 asks you to sign off each change to the format specification. Sprint 7's T-002 changed docs/spec/format-v1.md (commit 609a8df): an operation now names the workspace that made it, and a view holds one entry per workspace (its working-copy pointer and its heads) instead of one set of heads. Do you sign off this change?" | "Sign off (Recommended)" | "Your answer is recorded word for word in docs/sprints/s7/sign-offs.md as the sign-off of INT-0006 AC-6 for this change." |

- The owner had merged the sprint's pull request, [#10](https://github.com/crussella0129/Animus_Periapsis/pull/10), at `b951d6d` at 2026-11-14T22:36:56Z, while Test was still running and before this question was asked. The answer was given afterwards, on the same day. The merge is recorded, but it was not taken as the sign-off; only the answer above is.
- INT-0006 AC-6 is signed for this change, the one T-002 made. Each later change to the specification needs its own.

## Decisions that are not sign-offs

Three choices that no criterion asks the owner to sign were settled by the same questions, two in Plan and one during Test:
- **Two commands at once in one workspace.** The answer was "Continue from one, keep both (Recommended)". The next command continues from one change and keeps the other ([INT-0014](../../intents/INT-0014-working-copy-and-workspaces.md) Consequences).
- **GitHub during Test.** The answer was "Push, PR, and two re-runs (Recommended)": push `dev`, open the pull request, and re-run every job of the tested head twice, for INT-0003 AC-11's three samples.
- **The work after the merge of #10.** Asked during Test, in the same round as the format sign-off: "You merged PR #10 at b951d6d while Test was still running. The second test critique then found that in a lane, `praps verify` and commands run below the lane's top give the wrong hint (one even offers `praps init`). I fixed that in 4068f43, which is on dev but not in main; the sprint's records and its close still follow. How should that work reach main and get its three GitHub samples?" The answer was "Open a follow-up PR (Recommended)", which said: "Open a second dev → main pull request now. Its checks plus two re-runs of every job give the three samples at 4068f43, and the sprint's close lands on it too. You merge it when you choose." It is [#11](https://github.com/crussella0129/Animus_Periapsis/pull/11).

## How the format sign-off was asked

INT-0006 AC-6 asks for a sign-off of each change to the format specification. T-002 wrote this sprint's change: an operation names the workspace that made it, and a view holds one entry per workspace, its working-copy pointer and its heads (`docs/spec/format-v1.md`, the `operation` and `view` bodies and four worked examples). The plan was to ask at the end of the sprint, before the merge, as in Sprint 6. The merge of #10 came first, so the question was put as soon as it was seen, and the answer is recorded above in the owner's own words.
