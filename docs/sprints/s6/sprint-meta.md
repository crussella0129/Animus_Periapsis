# Sprint 6 Meta

- **Sprint number:** 6
- **Book schema version:** 2
- **Start timestamp:** 2026-11-10T04:27:17Z
- **End timestamp:** 2026-11-10T18:09:51Z
- **Model:** claude-fable-5-2
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** The owner's direction for scale, speed and use written into the Book, and its first steps in code: one read of the op log for each command, and a command set that explains itself (`help`, `describe`, `log` for changes, `ops`, `verify`, commands that work from any directory inside a working copy, a hint on every refusal); plus the Sprint 5 CI record
- **Intents:** [INT-0037](../../intents/INT-0037-command-set.md), [INT-0022](../../intents/INT-0022-agent-and-human-interfaces.md), [INT-0035](../../intents/INT-0035-scale-and-asynchrony.md), [INT-0014](../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0009](../../intents/INT-0009-reversible-by-default.md), [INT-0005](../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../intents/INT-0003-engineering-baseline.md), [INT-0004](../../intents/INT-0004-windows-and-linux.md), [INT-0001](../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../intents/INT-0002-typed-acceptance-criteria.md), [INT-0006](../../intents/INT-0006-format-v1.md)
- **Completion evidence:** A command set that explains itself and one read of the op log per command: PR #8 at bbffce6 passed all five checks with 588 tests on Windows and 590 on Linux; cargo ci 288 s on Windows and 186 s in WSL; five matrices with 0 violations and every control caught; the operation list at 800 operations fell from 21.6 s to 0.37 s on Windows; INT-0037 AC-4 shown for 31 of 32 kinds; test report and round-2 critique proceed-with-caveats
