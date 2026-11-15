# Sprint 7 Meta

- **Sprint number:** 7
- **Book schema version:** 2
- **Start timestamp:** 2026-11-14T13:06:25Z
- **End timestamp:** 2026-11-15T00:36:14Z
- **Model:** claude-opus-5-5
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Several agents in one repository: the canonical merge of op heads, computed and never committed, so overlapping commands no longer stop the repository; workspaces over one store (`praps lane DIR`, `praps lanes`); `undo` and `restore` scoped to their workspace; plus the Sprint 6 CI record
- **Intents:** [INT-0010](../../intents/INT-0010-lock-free-concurrency.md), [INT-0014](../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../intents/INT-0009-reversible-by-default.md), [INT-0006](../../intents/INT-0006-format-v1.md), [INT-0016](../../intents/INT-0016-actors-and-agent-lifecycle.md), [INT-0037](../../intents/INT-0037-command-set.md), [INT-0022](../../intents/INT-0022-agent-and-human-interfaces.md), [INT-0005](../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../intents/INT-0004-windows-and-linux.md)
- **Completion evidence:** Several agents in one repository: the canonical merge of op heads, lanes over one store, and scoped undo and restore. PR #11 at 4068f43 passed all five checks in three samples (Windows 17 min 10 s, 16 min 57 s, 9 min 51 s) with 663 tests on Windows and 666 on Linux, after the owner merged #10 at b951d6d; cargo ci 381 s on Windows and 336 s in a fresh WSL clone; seven matrices with 0 violations, every control caught and every Concurrency control swept over all its cells; 8 praps new at once all succeed (3 of 8 before); INT-0006 AC-6 signed; test report and round-3 critique proceed-with-caveats
