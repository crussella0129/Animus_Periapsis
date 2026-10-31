# INT-0022 — Agent-first interfaces that stay pleasant for humans

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0022
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
One engine, two faces. `peri` is the human CLI (later a TUI): calm, readable
output. `peri mcp` serves agents over the Model Context Protocol: it never
prompts, returns structured results and structured errors with stable codes,
supports sparse or API-only workspaces, and exposes every operation the human
CLI has, because both are generated from one command registry.

Command names are the simplest words that say what they do. A command is
**idempotent** when running it again changes nothing (`status`, `log`, `diff`);
idempotent commands take no flags, so their default behaviour is their only
behaviour. Flags exist only on non-idempotent commands, where they make a state
change explicit. The project's own tooling follows the same rule
([INT-0003](INT-0003-engineering-baseline.md) AC-9).

Human output quality is held to typed criteria — the R5 worked example for
`peri log` is adopted here with its thresholds fixed in advance.

## Acceptance criteria
- **AC-1** `necessary` `empirical` — Every operation served by `peri mcp` completes without a TTY and without prompting.
  - Threshold: 0 prompts across the command corpus.
- **AC-2** `necessary` `empirical` — Every `peri mcp` response and error validates against a published JSON schema, and error codes are stable across releases.
- **AC-3** `necessary` `formal` — Parity: the operations exposed through JSON/MCP equal the human CLI's operations.
  - Obligation: `unstated`.
  - Proxy: a test that compares both surfaces against the single registry.
- **AC-4** `necessary` `formal` — `peri log` never renders a line wider than the terminal width `w ≥ 1` (LogFits).
  - Obligation: `Periapsis.Render.logFits` (R5 proved the prototype's version for all inputs).
- **AC-5** `supporting` `empirical` — No-wrap on real logs: in at least 200 sampled real repositories' logs, more than 95% render without wrapping.
  - Error: δ = 1/1000. Note: the sampling seed is registered before the layout change.
- **AC-6** `supporting` `preference` — Whenever the `peri log` layout changes, a blind A/B panel of 12 judges prefers the new layout to the previous one.
  - Threshold: preference > ½. Error: δ = 1/20. Note: the rubric is registered before the change.
- **AC-7** `supporting` `judged` — Design rubric for `peri log` (calm, scannable, consistent): at least 2 of 3 designers score it 4 or more out of 5.
  - Quorum: 2 of 3 non-author designers.
- **AC-8** `necessary` `formal` — No command that the registry classifies as idempotent declares a flag.
  - Obligation: `unstated`.
  - Proxy: a registry test over every command.
- **AC-9** `necessary` `empirical` — Running any idempotent command twice in a row leaves repository state unchanged and prints identical output.
  - Threshold: 100% of idempotent commands in the corpus, on Windows and Linux.

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
  idempotent commands. Whether scripts use `peri mcp`, an environment setting
  or separate structured commands is open, to be decided when this intent is
  planned.
- Qualitative criteria need real panels (INT-0020 AC-5) before they can be
  certified.

## Transition history
- 2026-10-31: created as `proposed` (Sprint 0 research; transcribed from Master Report v6 II §4 Phase 3 and R5 §B.4).
- 2026-10-31: revised while `proposed`, on owner direction: simplest command names, flags only on non-idempotent commands, and agents served by `peri mcp` instead of a `--json` flag (AC-1 and AC-2 reworded; AC-8 and AC-9 added).
