# INT-0022 — Agent-first interfaces that stay pleasant for humans

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0022
- **State:** active
- **Work evidence:** [Sprint 6 T-006 build plan](../sprints/s6/sprint-plans/build-plan.md#t-006-the-command-registry-dispatch-help-and-suggestions-from-one-table)
- **Completion evidence:** [Sprint 6 T-006 completion](../work/completed-tasks.md#t-006-sprint-6)
- **Code evidence:** [registry.rs](../../crates/praps/src/registry.rs), [main.rs](../../crates/praps/src/main.rs)
- **Test evidence:** [Sprint 6 test report](../sprints/s6/sprint-tests/test-report.md)
- **Documentation evidence:** [README](../../README.md)

## Intent
One engine, two faces. `praps` is the human CLI (later a TUI): calm, readable
output. `praps mcp` serves agents over the Model Context Protocol: it never
prompts, returns structured results and structured errors with stable codes,
supports sparse or API-only workspaces, and exposes every operation the human
CLI has, because both are generated from one command registry. File contents
cross `praps mcp` as exact bytes: a JSON string when they are valid UTF-8,
base64 otherwise, with a field that says which, and a file too large for one
message crosses in byte ranges. An API-only workspace therefore reads and
writes any content without loss ([INT-0034](INT-0034-any-content.md)).

Command names are the simplest words that say what they do. A command is
**idempotent** when a repeat changes nothing and prints the same: run twice
in a row, the second run leaves every file as the first left it and prints
what the first printed. `status`, `log`, `ops`, `diff`, `verify` and `help`
are idempotent; they are the commands that report. Idempotent commands take
no flags, so their default behaviour is their only behaviour. Flags exist
only on the other commands, where they make a state change explicit. `init`
and `describe` are not idempotent, although a repeat finds nothing left to
do: what each prints says whether it did anything, so a repeat does not
print the same. Every command that opens the working copy records it first.
A first run may therefore write, and a repeat still changes nothing, so that
recording does not make a command any less idempotent. The registry marks
each idempotent command, and `help` shows the mark. The project's own
tooling follows the same rule
([INT-0003](INT-0003-engineering-baseline.md) AC-9): `cargo reach` rewrites
a file, and a repeat changes nothing and prints the same.

Since `diff` takes no flags, how it shows a file depends only on the file and
the repository's declarations: through the diff layer chosen for that kind of
content, by default a line diff for UTF-8 text, and otherwise by the file's
old and new size and hash. Every command that shows content falls back the
same way, and no command fails on content it cannot read.

Human output quality is held to typed criteria — the R5 worked example for
`praps log` is adopted here with its thresholds fixed in advance.

The registry is the one place a command is described. Its name, arguments,
one-line summary, help text and examples are entries in the registry, and
dispatch, help and usage are all generated from them. Which commands exist
and what they are called is [INT-0037](INT-0037-command-set.md). The screen
for a person following many agents is [INT-0038](INT-0038-live-view.md).

Many agents on one machine can share one process. `praps serve` is an
optional local service: it holds the repository open once, serves each
agent's commands through the same registry, and gathers their writes into
shared flushes. It is an accelerator only
([INT-0035](INT-0035-scale-and-asynchrony.md)). The files on disk stay safe
for any number of processes without it, and every command gives the same
result with it or without.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Every operation served by `praps mcp` completes without a TTY and without prompting.
  - Threshold: 0 prompts across the command corpus.
- **AC-2** `necessary` `empirical` — Every `praps mcp` response and error validates against a published JSON schema, and error codes are stable across releases.
- **AC-3** `necessary` `formal` — Parity: the operations exposed through JSON/MCP equal the human CLI's operations.
  - Obligation: `unstated`.
  - Proxy: a test that compares both surfaces against the single registry.
- **AC-4** `necessary` `formal` — `praps log` never renders a line wider than the terminal width `w ≥ 1` (LogFits).
  - Obligation: `Periapsis.Render.logFits` (R5 proved the prototype's version for all inputs).
- **AC-5** `supporting` `empirical` — No-wrap on real logs: in at least 200 sampled real repositories' logs, more than 95% render without wrapping.
  - Error: δ = 1/1000. Note: the sampling seed is registered before the layout change.
- **AC-6** `supporting` `preference` — Whenever the `praps log` layout changes, a blind A/B panel of 12 judges prefers the new layout to the previous one.
  - Threshold: preference > ½. Error: δ = 1/20. Note: the rubric is registered before the change.
- **AC-7** `supporting` `judged` — Design rubric for `praps log` (calm, scannable, consistent): at least 2 of 3 designers score it 4 or more out of 5.
  - Quorum: 2 of 3 non-author designers.
- **AC-8** `necessary` `formal` — No command that the registry classifies as idempotent declares a flag.
  - Obligation: `unstated`.
  - Proxy: a registry test over every command.
- **AC-9** `necessary` `empirical` — Running any idempotent command twice in a row leaves repository state unchanged and prints identical output.
  - Threshold: 100% of idempotent commands in the corpus, on Windows and Linux.
- **AC-10** `necessary` `empirical` — Through `praps mcp` alone, an agent writes and reads back every file of the conformance corpus of [INT-0034](INT-0034-any-content.md) byte for byte, including invalid UTF-8, NUL bytes and files larger than one message.
  - Threshold: 0 differing bytes.
- **AC-11** `necessary` `formal` — One description of each command: the command list, each command's help and each usage line are generated from its registry entry, and no command is dispatched that the registry does not hold.
  - Obligation: `unstated`.
  - Proxy: tests that render the list, the help and the usage of every entry, and a test that every word `praps` accepts as a command is a registry name.
- **AC-12** `necessary` `empirical` — With the service or without: every command in the corpus prints the same output and leaves the same state with `praps serve` running and with none.
  - Threshold: 0 differences ([INT-0035](INT-0035-scale-and-asynchrony.md) AC-8).

## Rationale
- Agents need commands that cannot hang on a prompt, outputs a program can
  parse and errors it can act on (II §4 Phase 3 step 24).
- Humans remain the reviewers of intents and batches, so the human face is a
  product requirement, not decoration.
- AC-4 to AC-7 are R5's aesthetic worked example — one necessary formal proxy
  plus empirical, preference and judged sufficiency evidence — adopted as real
  Book criteria (R5 §B.4).
- Sources: [Master Report v6](../sprints/s0/sprint-research/master-report-v6.md)
  II §4 Phase 3, R5 §B.4.

## Alternatives
- **Agents scrape human output.** Brittle, and every wording change breaks
  agents.
- **A separate agent API server only.** Splits behaviour from the CLI; a shared
  registry keeps them identical.

## Consequences
- JSON schemas and error codes become a compatibility surface that needs
  versioning.
- Machine-readable output for shell scripts cannot be a `--json` flag on
  idempotent commands. Whether scripts use `praps mcp`, an environment setting
  or separate structured commands is open, to be decided when this intent is
  planned.
- Qualitative criteria need real panels (INT-0020 AC-5) before they can be
  certified.
- Sprint 6 builds the registry and the human face's dispatch, help and usage
  from it (AC-8, AC-11). `praps mcp`, the schemas and the service come later,
  each generated from the same table.
- `--help`, `-h`, `--version` and `-V` are accepted as spellings of the
  `help` and `version` commands ([INT-0037](INT-0037-command-set.md)). They
  are not flags of any command, so AC-8 stands: the registry declares no
  flag for an idempotent command.
- Sprint 6 changes what `praps log` lists, from operations to changes, as
  the owner chose on 2026-11-10 ([INT-0037](INT-0037-command-set.md)). That
  listing is provisional: plain lines, one for each change, not fitted to the
  terminal and judged by no panel. It does not meet AC-4, and nothing is
  claimed for AC-5 to AC-7. Those four criteria are about the designed
  layout, which a later sprint builds on a renderer that takes the terminal's
  width. The sampling seed of AC-5 and the rubric of AC-6 are registered
  before that sprint's first layout change, and the provisional listing is
  the layout the panel compares the new one with.
- Until the service exists, each agent command is a process that opens the
  repository itself. That is why the cost of opening must not grow with the
  store ([INT-0008](INT-0008-compact-history-storage.md) AC-8).

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §4 Phase 3 and R5 §B.4).
- 2026-10-31: revised while `proposed`, on owner direction: simplest command names, flags only on non-idempotent commands, and agents served by `peri mcp` instead of a `--json` flag (AC-1 and AC-2 reworded; AC-8 and AC-9 added).
- 2026-11-01: revised while `proposed`, on owner direction: the command is `praps`, short for periapsis, replacing `peri` ([INT-0003](INT-0003-engineering-baseline.md)). AC-1, AC-2, AC-4, AC-6 and AC-7 are reworded for the new name only.
- 2026-11-01: revised while `proposed`, on owner direction: the interfaces must carry and show any content. The Intent says file contents cross `praps mcp` as exact bytes, as base64 when they are not UTF-8 and in byte ranges when large, and that `diff`, which takes no flags, shows a file through the diff layer chosen for its kind, else by size and hash; AC-10 checks the round trip.
- 2026-11-10: revised while `proposed` (Sprint 6 Plan, on owner direction of 2026-11-09 for a far more intuitive command set and for thousands of agents): the Intent says the registry is the one description of each command, points to [INT-0037](INT-0037-command-set.md) for the command set and [INT-0038](INT-0038-live-view.md) for the live view, and adds an optional local service that many agents share. AC-11 and AC-12 are added.
- 2026-11-10: `proposed` → `planned` (Sprint 6 build plan T-006: the registry, with dispatch, help and usage generated from it; AC-8 and AC-11. `praps mcp`, the schemas and the service wait for later sprints).
- 2026-11-10: revised while `planned` (Sprint 6 plan critique, round 1, C-004 and C-006; `semantic`): the Intent defines the idempotent class as the commands that only report, names `init` and `describe` as outside it, and says the registry marks the class and `help` shows it; AC-8 and AC-9 are unchanged and now apply to that class. Consequences record that Sprint 6's `log` listing is provisional, that it does not meet AC-4, and when the seed and the rubric of AC-5 and AC-6 are registered.
- 2026-11-10: revised while `planned` (Sprint 6 plan critique, round 2, C-008; `semantic`): the idempotent class is defined by the repeat alone, that it changes nothing and prints the same, so that a command which writes on its first run, as `status` and `cargo reach` do, has a settled class. "The commands that report" stays as a description of the `praps` commands in it.
- 2026-11-10: `planned` → `active` (Sprint 6 build, T-006 starts: the command registry).
- 2026-11-10: kept `active` at the close of Sprint 6. AC-8 and AC-11 pass. AC-9 passes for the six commands built whose help says they are idempotent, and stays open for the rest. Its other criteria are not tested: `praps mcp`, the schemas and the renderer are not built.
