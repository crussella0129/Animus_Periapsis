# Sprint 3 Meta

- **Sprint number:** 3
- **Book schema version:** 2
- **Start timestamp:** 2026-11-07T19:11:05Z
- **End timestamp:** 2026-11-08T03:36:41Z
- **Model:** claude-opus-5-5
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Going back: `praps log`, `praps undo` and `praps restore OP`, which write the working copy through a crash-safe journalled materialise that refuses case-fold collisions and Windows-unrepresentable names; plus the Sprint 2 CI record and the remaining randomness bans
- **Intents:** [INT-0009](../../intents/INT-0009-reversible-by-default.md), [INT-0014](../../intents/INT-0014-working-copy-and-workspaces.md), [INT-0004](../../intents/INT-0004-windows-and-linux.md), [INT-0007](../../intents/INT-0007-crash-safe-storage.md), [INT-0005](../../intents/INT-0005-deterministic-simulation-testing.md), [INT-0003](../../intents/INT-0003-engineering-baseline.md), [INT-0006](../../intents/INT-0006-format-v1.md), [INT-0001](../../intents/INT-0001-periapsis-charter.md), [INT-0002](../../intents/INT-0002-typed-acceptance-criteria.md)
- **Completion evidence:** praps log, undo and restore OP with a crash-safe journalled materialise and hazard refusal: 407/408 tests and every crash matrix (store, Snapshot, Materialise incl. Windows rules and second crash) clean on Windows and WSL at ad22319; test critique round 2 proceed-with-caveats; see docs/sprints/s3/sprint-tests/test-report.md
- **Checkpoint:** https://github.com/crussella0129/Animus_Periapsis/pull/5
