# INT-0037 — The command set: a few plain words that say what they do

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0037
- **State:** planned
- **Work evidence:** [Sprint 6 T-006 build plan](../sprints/s6/sprint-plans/build-plan.md#t-006-the-command-registry-dispatch-help-and-suggestions-from-one-table), [Sprint 6 T-007 build plan](../sprints/s6/sprint-plans/build-plan.md#t-007-verify-replaces-fsck), [Sprint 6 T-008 build plan](../sprints/s6/sprint-plans/build-plan.md#t-008-commands-find-their-working-copy-from-any-directory-inside-it), [Sprint 6 T-009 build plan](../sprints/s6/sprint-plans/build-plan.md#t-009-log-lists-changes-and-ops-lists-operations), [Sprint 6 T-010 build plan](../sprints/s6/sprint-plans/build-plan.md#t-010-describe-sets-the-description-of-the-current-change), [Sprint 6 T-011 build plan](../sprints/s6/sprint-plans/build-plan.md#t-011-every-refusal-names-the-next-command), [Sprint 6 T-012 build plan](../sprints/s6/sprint-plans/build-plan.md#t-012-a-write-leaves-a-directory-it-cannot-remove), [Sprint 6 T-013 build plan](../sprints/s6/sprint-plans/build-plan.md#t-013-readme-specs-and-credits)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
`praps` has one small set of commands, named in plain words, that a person or
an agent can learn from the tool itself. Someone who has never used it does a
day's work with the everyday commands, finds every other command through
`praps help`, and is told what to do next whenever a command cannot do what
was asked.

Rules:
- **Plain words.** A command is one lower-case English word that says what it
  does: `status`, `describe`, `undo`. No abbreviation, and no word that only
  makes sense to someone who knows another system: not `fsck`, `rebase`,
  `checkout`, `stash`, `reflog` or `cherry-pick`.
- **A plural lists; a singular names one.** `ops`, `lanes`, `intents` and
  `requests` list. The singular with a name shows or makes that one.
- **Few arguments, in a fixed order.** A command takes what it acts on, then
  optionally the directory of the working copy. Text of several words is
  one quoted argument. Flags follow
  [INT-0022](INT-0022-agent-and-human-interfaces.md): an idempotent command,
  one whose repeat changes nothing and prints the same, takes none.
- **The tool teaches itself.**
  - `praps` alone lists the commands by purpose, one line each, the everyday
    ones first.
  - `praps help WORD` explains one command, with its arguments and an example.
  - A word that is not a command gets the nearest command names.
  - `--help` and `-h` are other spellings of `help`, and `--version` and `-V`
    of `version`. They are the only arguments beginning with `-` that every
    command accepts. Any other is refused, so the text given to `describe`
    cannot begin with `-`.
- **Anywhere inside.** A command run in any directory of a working copy acts
  on that working copy. `init` refuses to make a repository inside another.
- **Every answer says what happened and what to do next.** A command that
  changes something says what it changed in one line. A command that cannot do
  what was asked says what was wrong and what to do next: a command to run,
  or something to put right outside `praps`. Where this build has no remedy
  yet, it says so, and says that nothing was lost.
- **No questions.** No command prompts. One that needs a decision stops and
  names the commands that make it, so an agent never hangs
  ([INT-0022](INT-0022-agent-and-human-interfaces.md) AC-1).
- **One set of names.** The agent interface offers each command under the
  same name, from the same registry.
- **Names last.** After the first release a command's name and arguments are a
  compatibility surface. A renamed command keeps its old name working for at
  least one release, and the old name says the new one.

The vocabulary. "Built" means it works today. Every other name is a proposal,
fixed when the intent that owns the command is planned.

| Purpose | Command | What it does | State |
|---|---|---|---|
| Every day | `status` | Lists what the working copy changes | built |
| | `describe TEXT` | Says what the current change is for | Sprint 6 |
| | `new` | Starts the next change | built |
| | `log` | Lists the changes that lead to the working copy, newest first | Sprint 6 (it listed operations before) |
| | `diff` | Shows what the current change changes | [INT-0012](INT-0012-conflicts-and-merge.md) |
| | `undo` | Takes back the last thing this workspace did | built |
| | `sync` | Exchanges work with peers | [INT-0024](INT-0024-staged-sync.md) |
| Looking back | `ops` | Lists every operation, newest first | Sprint 6 (was `log`) |
| | `restore OP` | Returns to the state an operation left | built |
| | `show WHAT` | Shows one change, operation, intent or request in full | later |
| | `find QUERY` | Selects by query | [INT-0023](INT-0023-query-language.md) |
| Shaping changes | `edit`, `move`, `squash`, `split`, `absorb`, `abandon`, `resolve` | Rewrites changes; `move` is what other systems call rebase | [INT-0011](INT-0011-change-identity-and-rewriting.md) |
| Working together | `lanes`, `lane NAME` | Lists workspaces; makes or enters one | [INT-0014](INT-0014-working-copy-and-workspaces.md) |
| | `requests`, `request` | Lists review requests; asks for review of the current change | [INT-0021](INT-0021-lanes-and-review-requests.md) |
| | `intents`, `intent ID` | Lists the Book's intents; shows one | [INT-0017](INT-0017-event-records-and-book-view.md) |
| | `watch` | Opens the live view | [INT-0038](INT-0038-live-view.md) |
| Starting and care | `init` | Makes a directory a repository | built |
| | `help`, `version` | Explains the commands; names the build | Sprint 6 |
| | `verify` | Checks that everything stored is intact | built as `fsck`; renamed in Sprint 6 |
| | `import`, `export` | Brings history from Git; mirrors it back | [INT-0025](INT-0025-git-bridge.md) |
| | `mcp`, `serve` | Serves agents; runs the optional local service | [INT-0022](INT-0022-agent-and-human-interfaces.md) |

Boundaries: this intent owns the names, the arguments and what a command says
about itself. What each command does belongs to the intent that defines it.
The two faces and the registry are
[INT-0022](INT-0022-agent-and-human-interfaces.md). The screen for following
many agents is [INT-0038](INT-0038-live-view.md).

## Acceptance criteria
- **AC-1** `necessary` `formal` — Every command name in the registry is one word matching `[a-z]+`, and none is a word this chapter refuses (`fsck`, `rebase`, `checkout`, `stash`, `reflog`, `cherry-pick`).
  - Obligation: `unstated`.
  - Proxy: a registry test over every command.
- **AC-2** `necessary` `empirical` — The tool teaches itself: `praps` alone lists every command in the registry; `praps help WORD` exits 0 for every command and prints its one-line summary, its arguments and at least one example; and a word within two edits of a command name is answered with that name and exit code 2.
  - Threshold: 100% of registry commands, and 100% of a corpus of misspelt names.
- **AC-3** `necessary` `empirical` — Anywhere inside: every command that opens a working copy prints the same output and leaves the same state when run from the top of the working copy and from any directory below it, and `init` inside a working copy is refused with nothing written.
  - Threshold: identical output and state over the command corpus, on Windows and Linux.
- **AC-4** `necessary` `empirical` — Every refusal says what to do: each kind of error `praps` can print states what was wrong, and then one of three things: a command to run next; what to put right outside `praps`; or that this build has no remedy yet and nothing was lost.
  - Threshold: 100% of the error kinds `praps` defines: usage errors, unknown words, every error of the `workspace` crate, and every way `verify` and finding the working copy can fail.
- **AC-5** `necessary` `formal` — The everyday group of the registry holds at most ten commands.
  - Obligation: `unstated`.
  - Proxy: a registry test.
- **AC-6** `supporting` `empirical` — An agent learns it from help alone: given only the output of `praps` and `praps help WORD`, an agent with no other knowledge of Periapsis completes a fixed set of tasks without one failed command.
  - Sample: 20 tasks, fixed before the vocabulary next changes, each run by 5 fresh agents.
  - Threshold: at least 90 of the 100 runs.
- **AC-7** `supporting` `preference` — People prefer reading it: shown the same ten tasks done in `praps`, in Git and in Jujutsu, a blind panel prefers the `praps` transcript to each of the others.
  - Panel: 12 judges who use version control and did not work on Periapsis.
  - Threshold: preference > ½ against each.
  - Error: δ = 1/20.
- **AC-8** `necessary` `asserted` — The project owner approves the vocabulary table and the list of refused words.
  - Signers: project owner.

## Rationale
- Owner direction, 2026-11-09: Periapsis needs "a far more intuitive command
  set and UI".
- What a newcomer met before this chapter
  ([Sprint 6 research](../sprints/s6/sprint-research/research-report.md)):
  - `praps` alone printed a usage error and exit code 2;
  - `praps --help` did the same;
  - `praps status` in a subdirectory of a working copy said the directory was
    not a repository;
  - `log` listed operations, where every other system lists changes;
  - a change could not be given a description at all;
  - one command, `fsck`, was named after a Unix tool.
- Agents learn a tool from what it prints. A command set that explains itself
  needs no documentation in the prompt, and a refusal that names the next
  command lets an agent recover without a person.
- Jujutsu showed that a version control system can drop most of Git's
  vocabulary and be easier for it. Mercurial calls its integrity check
  `verify`.
- The Command Line Interface Guidelines collect the same practice from many
  tools: help that leads with examples, suggestions for mistyped commands, and
  errors written for the person reading them.

## Alternatives
- **Git's names, for familiarity.** They carry Git's model with them:
  `checkout`, `reset` and `rebase` each mean several things. People coming
  from Git are served by `help`, which can name the Git word a command
  replaces.
- **Noun, then verb** (`praps op log`, `praps workspace add`, as Jujutsu
  does). Easy to explore by noun, but every command is then two words, and
  the everyday ones are no shorter.
- **Short aliases** (`st`, `ci`). Two names for one thing, in every
  transcript and every answer. A shell can define them, and agents do not
  need them.
- **Questions and wizards.** A command that asks can hang an agent
  ([INT-0022](INT-0022-agent-and-human-interfaces.md) AC-1).

## Consequences
- `log` now lists changes and `ops` lists operations, and `fsck` is `verify`.
  The README, the working-copy spec and the tests change with them.
  [INT-0007](INT-0007-crash-safe-storage.md) still calls the command
  `praps fsck`: its criteria are about the check, not its name.
- The registry of [INT-0022](INT-0022-agent-and-human-interfaces.md) is built
  first, because help, suggestions and the tests of AC-1, AC-2 and AC-5 all
  read it.
- What a command prints becomes part of its tests, so wording changes are
  deliberate.
- The names of commands not yet built are proposals. Each is fixed, or
  changed with a recorded revision of this chapter, when its owning intent is
  planned.
- AC-6 needs agents that have never seen Periapsis, so its tasks and prompts
  are kept out of any agent's instructions.

## Transition history
- 2026-11-09: created as `proposed` (Sprint 6 research; owner direction of 2026-11-09 for a far more intuitive command set).
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, after a review of the build plan against the code): three kinds of error have no next command today (several op heads, a change with several parents, a failure of the filesystem), so the rule and AC-4 now allow an answer that says what to put right outside `praps`, or that this build has no remedy yet and nothing was lost. AC-4's threshold names the error kinds it covers. The rule on `--help` says that any other argument beginning with `-` is refused.
- 2026-11-10: `proposed` → `planned` (Sprint 6 build plan T-006 to T-013, approved by the owner on 2026-11-10 with `verify` as the name of the integrity check and with `log` for changes and `ops` for operations: the registry, help and suggestions; finding the working copy; `describe`, `log`, `ops` and `verify`; hints on every refusal; the README. AC-1 to AC-5. AC-6 and AC-7 need agents and a panel, and AC-8 the owner's sign-off on the whole table).
- 2026-11-10: revised while `planned` (Sprint 6 plan critique, round 1, C-006 and C-015; `editorial`: no criterion changes): the rule on arguments says that text of several words is one quoted argument, and words the flag rule as [INT-0022](INT-0022-agent-and-human-interfaces.md) now defines it. The build plan's tasks for this chapter are renumbered after three were split; with the task the second critique added, they are T-006 to T-013.
